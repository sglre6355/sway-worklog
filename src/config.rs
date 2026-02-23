use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

use crate::error::Result;
use crate::paths;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub work_workspaces: Vec<String>,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_minutes: u64,
    pub log_path: Option<PathBuf>,
}

fn default_idle_timeout() -> u64 {
    30
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = paths::config_path()?;
        if !path.exists() {
            return Err(crate::error::Error::ConfigNotFound(path));
        }
        let contents = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn log_path(&self) -> Result<PathBuf> {
        match &self.log_path {
            Some(p) => Ok(p.clone()),
            None => paths::default_log_path(),
        }
    }

    pub fn is_work_workspace(&self, name: &str) -> bool {
        self.work_workspaces.iter().any(|w| w == name)
    }
}
