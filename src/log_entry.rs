use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    Switch,
    Shutdown,
    Signal,
    Idle,
    WorkspaceChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LogEntry {
    Start {
        workspace: String,
        timestamp: DateTime<Local>,
    },
    Stop {
        workspace: String,
        timestamp: DateTime<Local>,
        reason: StopReason,
    },
}

impl LogEntry {
    pub fn timestamp(&self) -> &DateTime<Local> {
        match self {
            LogEntry::Start { timestamp, .. } => timestamp,
            LogEntry::Stop { timestamp, .. } => timestamp,
        }
    }
}

pub fn append_entry(path: &Path, entry: &LogEntry) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(entry)?;
    writeln!(file, "{line}")?;
    Ok(())
}

pub fn read_entries(path: &Path) -> Result<Vec<LogEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: LogEntry = serde_json::from_str(&line)?;
        entries.push(entry);
    }
    Ok(entries)
}

pub fn read_last_entry(path: &Path) -> Result<Option<LogEntry>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let last_line = content.lines().rev().find(|l| !l.trim().is_empty());
    match last_line {
        Some(line) => Ok(Some(serde_json::from_str(line)?)),
        None => Ok(None),
    }
}
