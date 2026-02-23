use std::path::PathBuf;

use crate::error::{Error, Result};

pub fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or(Error::NoDirectory("config"))?
        .join("sway-worklog");
    Ok(dir)
}

pub fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn data_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .ok_or(Error::NoDirectory("data"))?
        .join("sway-worklog");
    Ok(dir)
}

pub fn default_log_path() -> Result<PathBuf> {
    Ok(data_dir()?.join("worklog.jsonl"))
}
