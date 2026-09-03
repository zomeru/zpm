use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZpmError {
    #[error("unsupported command \"{command}\" for agent \"{agent}\"")]
    UnsupportedCommand { agent: String, command: String },

    #[error("unknown agent \"{0}\"")]
    UnknownAgent(String),

    #[error("detection failed: {0}")]
    Detection(String),

    #[error("ambiguous package manager: {0}")]
    Ambiguous(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("config error: {0}")]
    Config(String),

    #[error("command failed with exit code {code:?}: {program} {args}")]
    CommandFailed {
        program: String,
        args: String,
        code: Option<i32>,
    },

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ZpmError>;
