use thiserror::Error;

#[derive(Error, Debug)]
pub enum RmzError {
    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
