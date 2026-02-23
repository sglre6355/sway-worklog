use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Sway IPC error: {0}")]
    SwayIpc(#[from] swayipc::Error),

    #[error("Config file not found: {}", .0.display())]
    ConfigNotFound(PathBuf),

    #[error("Could not determine {0} directory")]
    NoDirectory(&'static str),

    #[error("Invalid date: {0}")]
    InvalidDate(String),
}

pub type Result<T> = std::result::Result<T, Error>;
