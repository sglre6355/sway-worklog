use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use chrono::Local;
use swayipc::{Connection, Event, EventType, WorkspaceChange};

use crate::config::Config;
use crate::error::Result;
use crate::log_entry::{self, LogEntry, StopReason};

struct Session {
    workspace: String,
}

pub fn run(config: &Config) -> Result<()> {
    let log_path = config.log_path()?;

    let shutdown = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))?;

    let mut active_session: Option<Session> = None;
    let mut last_event_time = Local::now();
    let idle_timeout = chrono::Duration::minutes(config.idle_timeout_minutes as i64);

    // Check current workspace on startup
    if let Some(ws) = get_focused_workspace()?
        && config.is_work_workspace(&ws)
    {
        write_start(&log_path, &ws)?;
        active_session = Some(Session { workspace: ws });
        last_event_time = Local::now();
    }

    let sub = Connection::new()?.subscribe([EventType::Workspace, EventType::Shutdown])?;

    // Read sway events in a background thread so signals can interrupt the main loop.
    // EventStream blocks on read and doesn't expose the fd for polling.
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for event in sub {
            if tx.send(event).is_err() {
                break;
            }
        }
    });

    loop {
        if shutdown.load(Ordering::Relaxed) {
            if let Some(session) = active_session.take() {
                write_stop(&log_path, &session.workspace, StopReason::Signal)?;
            }
            break;
        }

        let event = match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(event) => event?,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };

        match event {
            Event::Workspace(ws_event) => {
                let now = Local::now();

                // Idle detection: if too much time passed, backdate a stop
                if let Some(ref session) = active_session
                    && now - last_event_time > idle_timeout
                {
                    write_stop_at(
                        &log_path,
                        &session.workspace,
                        StopReason::Idle,
                        last_event_time,
                    )?;
                    active_session = None;
                }

                last_event_time = now;

                if ws_event.change != WorkspaceChange::Focus {
                    continue;
                }

                let new_ws = match ws_event.current {
                    Some(ref node) => node.name.clone().unwrap_or_default(),
                    None => continue,
                };

                let is_work = config.is_work_workspace(&new_ws);

                match (&active_session, is_work) {
                    (None, true) => {
                        write_start(&log_path, &new_ws)?;
                        active_session = Some(Session { workspace: new_ws });
                    }
                    (Some(session), false) => {
                        write_stop(&log_path, &session.workspace, StopReason::Switch)?;
                        active_session = None;
                    }
                    (Some(session), true) if session.workspace != new_ws => {
                        write_stop(&log_path, &session.workspace, StopReason::WorkspaceChange)?;
                        write_start(&log_path, &new_ws)?;
                        active_session = Some(Session { workspace: new_ws });
                    }
                    _ => {}
                }
            }
            Event::Shutdown(_) => {
                if let Some(session) = active_session.take() {
                    write_stop(&log_path, &session.workspace, StopReason::Shutdown)?;
                }
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

fn get_focused_workspace() -> Result<Option<String>> {
    let mut conn = Connection::new()?;
    let workspaces = conn.get_workspaces()?;
    Ok(workspaces
        .into_iter()
        .find(|ws| ws.focused)
        .map(|ws| ws.name))
}

fn write_start(path: &Path, workspace: &str) -> Result<()> {
    let entry = LogEntry::Start {
        workspace: workspace.to_string(),
        timestamp: Local::now(),
    };
    log_entry::append_entry(path, &entry)?;
    eprintln!("▶ work started on workspace {workspace}");
    Ok(())
}

fn write_stop(path: &Path, workspace: &str, reason: StopReason) -> Result<()> {
    write_stop_at(path, workspace, reason, Local::now())
}

fn write_stop_at(
    path: &Path,
    workspace: &str,
    reason: StopReason,
    timestamp: chrono::DateTime<Local>,
) -> Result<()> {
    let entry = LogEntry::Stop {
        workspace: workspace.to_string(),
        timestamp,
        reason,
    };
    log_entry::append_entry(path, &entry)?;
    eprintln!("⏹ work stopped on workspace {workspace}");
    Ok(())
}
