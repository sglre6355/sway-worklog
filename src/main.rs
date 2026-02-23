mod config;
mod daemon;
mod error;
mod log_entry;
mod paths;
mod report;
mod summary;

use chrono::{Datelike, Days, Local, NaiveDate};
use clap::{Parser, Subcommand};

use crate::config::Config;
use crate::error::Result;
use crate::log_entry::{LogEntry, read_entries, read_last_entry};
use crate::report::{format_duration, print_summary, print_summary_json};
use crate::summary::{
    filter_sessions_by_date, filter_sessions_by_range, pair_sessions, totals_by_workspace,
};

#[derive(Parser)]
#[command(name = "sway-worklog", about = "Automated work hour tracker for Sway")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the workspace event listener (foreground)
    Daemon,
    /// Show work hour summary
    Summary {
        /// Show this week's summary
        #[arg(long)]
        week: bool,
        /// Show summary for a specific date (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Check if a work session is currently active
    Status,
    /// Dump raw log entries
    Log {
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
    },
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| error::Error::InvalidDate(s.to_string()))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Command::Daemon => daemon::run(&config),
        Command::Summary { week, date, json } => cmd_summary(&config, week, date, json),
        Command::Status => cmd_status(&config),
        Command::Log { from, to } => cmd_log(&config, from, to),
    }
}

fn cmd_summary(config: &Config, week: bool, date: Option<String>, json: bool) -> Result<()> {
    let log_path = config.log_path()?;
    let entries = read_entries(&log_path)?;
    let all_sessions = pair_sessions(&entries);

    let now = Local::now().date_naive();

    if week {
        // Show Mon-Sun of current week
        let weekday = now.weekday().num_days_from_monday();
        let monday = now - Days::new(weekday as u64);
        let sunday = monday + Days::new(6);

        let sessions = filter_sessions_by_range(&all_sessions, monday, sunday);
        let totals = totals_by_workspace(&sessions);

        if json {
            print_summary_json(now, &sessions, &totals);
        } else {
            println!("Work summary for week of {monday}");
            println!("===========================");
            let grand_total: chrono::Duration = totals.values().copied().sum();
            for (ws, dur) in &totals {
                println!("  {ws:<10}{}", format_duration(*dur));
            }
            println!("  -------------------------");
            println!("  {:<10}{}", "Total", format_duration(grand_total));

            // Per-day breakdown
            println!();
            let mut day = monday;
            while day <= sunday {
                let day_sessions = filter_sessions_by_date(&all_sessions, day);
                if !day_sessions.is_empty() {
                    let day_totals = totals_by_workspace(&day_sessions);
                    let day_total: chrono::Duration = day_totals.values().copied().sum();
                    println!("  {}  {}", day, format_duration(day_total));
                }
                day = day.succ_opt().unwrap();
            }
        }
    } else {
        let target_date = if let Some(ref d) = date {
            parse_date(d)?
        } else {
            now
        };

        let sessions = filter_sessions_by_date(&all_sessions, target_date);
        let totals = totals_by_workspace(&sessions);

        if json {
            print_summary_json(target_date, &sessions, &totals);
        } else {
            print_summary(target_date, &sessions, &totals);
        }
    }

    Ok(())
}

fn cmd_status(config: &Config) -> Result<()> {
    let log_path = config.log_path()?;
    match read_last_entry(&log_path)? {
        Some(LogEntry::Start {
            workspace,
            timestamp,
        }) => {
            let dur = Local::now() - timestamp;
            println!(
                "Active work session on workspace {workspace} ({})",
                format_duration(dur)
            );
        }
        _ => {
            println!("No active work session");
        }
    }
    Ok(())
}

fn cmd_log(config: &Config, from: Option<String>, to: Option<String>) -> Result<()> {
    let log_path = config.log_path()?;
    let entries = read_entries(&log_path)?;

    let from_date = from.as_deref().map(parse_date).transpose()?;
    let to_date = to.as_deref().map(parse_date).transpose()?;

    for entry in &entries {
        let date = entry.timestamp().date_naive();
        if let Some(f) = from_date
            && date < f
        {
            continue;
        }
        if let Some(t) = to_date
            && date > t
        {
            continue;
        }
        println!("{}", serde_json::to_string(entry).unwrap());
    }

    Ok(())
}
