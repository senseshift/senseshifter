use std::fmt;
use derivative::Derivative;

/// Errors that can occur in the connection manager
#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub enum ConnectionError {
    /// Transport-specific connection error
    Transport(String),
    /// Error occurred while sending a packet
    Send(String),
    /// Error occurred while encoding a packet
    Encoding(String),
    /// Target already exists
    TargetExists(String),
    /// Target not found
    TargetNotFound(String),
    /// Connection monitor already started
    MonitorAlreadyStarted,
    /// Connection manager is shutting down
    ShuttingDown,
    /// Invalid configuration
    InvalidConfig(String),
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionError::Transport(msg) => write!(f, "Transport error: {}", msg),
            ConnectionError::Send(msg) => write!(f, "Send error: {}", msg),
            ConnectionError::Encoding(msg) => write!(f, "Encoding error: {}", msg),
            ConnectionError::TargetExists(name) => write!(f, "Target '{}' already exists", name),
            ConnectionError::TargetNotFound(name) => write!(f, "Target '{}' not found", name),
            ConnectionError::MonitorAlreadyStarted => write!(f, "Connection monitor already started"),
            ConnectionError::ShuttingDown => write!(f, "Connection manager is shutting down"),
            ConnectionError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for ConnectionError {}

/// Result type for connection manager operations
pub type ConnectionResult<T> = Result<T, ConnectionError>;

/// Convert from common error types
impl From<tokio::sync::mpsc::error::SendError<rosc::OscPacket>> for ConnectionError {
    fn from(err: tokio::sync::mpsc::error::SendError<rosc::OscPacket>) -> Self {
        ConnectionError::Send(format!("MPSC send error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ConnectionError::TargetNotFound("test".to_string());
        assert_eq!(err.to_string(), "Target 'test' not found");

        let err = ConnectionError::Transport("connection refused".to_string());
        assert_eq!(err.to_string(), "Transport error: connection refused");
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<ConnectionError>();
        assert_sync::<ConnectionError>();
    }
}