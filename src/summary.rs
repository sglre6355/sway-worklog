use std::collections::BTreeMap;

use chrono::{DateTime, Local, NaiveDate};

use crate::log_entry::LogEntry;

#[derive(Debug)]
pub struct Session {
    pub workspace: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub open: bool,
}

impl Session {
    pub fn duration(&self) -> chrono::Duration {
        self.end - self.start
    }
}

pub fn pair_sessions(entries: &[LogEntry]) -> Vec<Session> {
    let mut sessions = Vec::new();
    let mut pending_start: Option<(&str, DateTime<Local>)> = None;

    for entry in entries {
        match entry {
            LogEntry::Start {
                workspace,
                timestamp,
            } => {
                // If there's already a pending start, it's orphaned — close it at this timestamp
                if let Some((ws, start)) = pending_start.take() {
                    sessions.push(Session {
                        workspace: ws.to_string(),
                        start,
                        end: *timestamp,
                        open: false,
                    });
                }
                pending_start = Some((workspace, *timestamp));
            }
            LogEntry::Stop {
                workspace,
                timestamp,
                ..
            } => {
                if let Some((ws, start)) = pending_start.take() {
                    sessions.push(Session {
                        workspace: ws.to_string(),
                        start,
                        end: *timestamp,
                        open: false,
                    });
                }
                // Ignore stop without matching start (can happen after crash)
                let _ = workspace;
            }
        }
    }

    // Orphaned start at end → use now as tentative end
    if let Some((ws, start)) = pending_start.take() {
        sessions.push(Session {
            workspace: ws.to_string(),
            start,
            end: Local::now(),
            open: true,
        });
    }

    sessions
}

pub fn filter_sessions_by_date(sessions: &[Session], date: NaiveDate) -> Vec<&Session> {
    sessions
        .iter()
        .filter(|s| s.start.date_naive() == date || s.end.date_naive() == date)
        .collect()
}

pub fn filter_sessions_by_range(
    sessions: &[Session],
    from: NaiveDate,
    to: NaiveDate,
) -> Vec<&Session> {
    sessions
        .iter()
        .filter(|s| {
            let start_date = s.start.date_naive();
            let end_date = s.end.date_naive();
            (start_date >= from && start_date <= to) || (end_date >= from && end_date <= to)
        })
        .collect()
}

pub fn totals_by_workspace(sessions: &[&Session]) -> BTreeMap<String, chrono::Duration> {
    let mut totals: BTreeMap<String, chrono::Duration> = BTreeMap::new();
    for session in sessions {
        let entry = totals
            .entry(session.workspace.clone())
            .or_insert_with(chrono::Duration::zero);
        *entry += session.duration();
    }
    totals
}
