use crate::proto::parser::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("transport error: {0}")]
    Transport(Box<dyn std::error::Error + Send + Sync + 'static>),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
    #[error("device error: {0}")]
    Device(String),
    #[error("timed out waiting for response")]
    Timeout,
    #[error("disconnected")]
    Disconnected,
}
