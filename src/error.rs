use thiserror::Error;

pub type Result<T> = std::result::Result<T, MinecraftError>;

#[derive(Error, Debug)]
pub enum MinecraftError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Invalid VarInt: {0}")]
    VarInt(String),

    #[error("Buffer underrun: {0}")]
    BufferUnderrun(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
