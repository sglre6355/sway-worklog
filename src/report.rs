use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::Serialize;

use crate::summary::Session;

pub fn format_duration(dur: chrono::Duration) -> String {
    let total_minutes = dur.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m")
    } else {
        format!("{minutes}m")
    }
}

pub fn print_summary(
    date: NaiveDate,
    sessions: &[&Session],
    totals: &BTreeMap<String, chrono::Duration>,
) {
    println!("Work summary for {date}");
    println!("===========================");

    let grand_total: chrono::Duration = totals.values().copied().sum();

    for (ws, dur) in totals {
        println!("  {ws:<10}{}", format_duration(*dur));
    }
    println!("  -------------------------");
    println!("  {:<10}{}", "Total", format_duration(grand_total));

    if !sessions.is_empty() {
        println!();
        println!("Sessions:");
        for s in sessions {
            let start = s.start.format("%H:%M");
            let end = if s.open {
                "now   ".to_string()
            } else {
                s.end.format("%H:%M").to_string()
            };
            let dur = format_duration(s.duration());
            println!("  {start} - {end}  {:<7}({dur})", s.workspace);
        }
    }
}

#[derive(Serialize)]
struct JsonSession {
    workspace: String,
    start: String,
    end: String,
    open: bool,
    duration_minutes: i64,
}

#[derive(Serialize)]
struct JsonSummary {
    date: String,
    total_minutes: i64,
    by_workspace: BTreeMap<String, i64>,
    sessions: Vec<JsonSession>,
}

pub fn print_summary_json(
    date: NaiveDate,
    sessions: &[&Session],
    totals: &BTreeMap<String, chrono::Duration>,
) {
    let grand_total: chrono::Duration = totals.values().copied().sum();
    let by_workspace: BTreeMap<String, i64> = totals
        .iter()
        .map(|(k, v)| (k.clone(), v.num_minutes()))
        .collect();

    let json_sessions: Vec<JsonSession> = sessions
        .iter()
        .map(|s| JsonSession {
            workspace: s.workspace.clone(),
            start: s.start.to_rfc3339(),
            end: s.end.to_rfc3339(),
            open: s.open,
            duration_minutes: s.duration().num_minutes(),
        })
        .collect();

    let summary = JsonSummary {
        date: date.to_string(),
        total_minutes: grand_total.num_minutes(),
        by_workspace,
        sessions: json_sessions,
    };

    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
}
