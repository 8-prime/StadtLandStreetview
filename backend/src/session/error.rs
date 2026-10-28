use std::fmt;

/// Transport-independent errors that API handlers can map to HTTP status codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    NotFound(&'static str),
    Forbidden(&'static str),
    InvalidInput(&'static str),
    Conflict(&'static str),
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (Self::NotFound(message)
        | Self::Forbidden(message)
        | Self::InvalidInput(message)
        | Self::Conflict(message)) = self;
        formatter.write_str(message)
    }
}

impl std::error::Error for SessionError {}

pub type Result<T> = std::result::Result<T, SessionError>;
