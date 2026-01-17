use thiserror::Error;

/// Standardized error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorCode {
    #[error("INVALID_REQUEST")]
    InvalidRequest,
    #[error("INSUFFICIENT_FUNDS")]
    InsufficientFunds,
    #[error("NETWORK_FAILURE")]
    NetworkFailure,
    #[error("PROOF_GENERATION_FAILED")]
    ProofGeneration,
    #[error("AGENT_UNAVAILABLE")]
    AgentUnavailable,
    #[error("TIMEOUT")]
    Timeout,
}

/// Structured error type for better error handling
#[derive(Debug, Error)]
#[error("[{code}] {message}")]
pub struct SdkError {
    pub code: ErrorCode,
    pub message: String,
    #[source]
    pub cause: Option<anyhow::Error>,
}

impl SdkError {
    /// Creates a new SDK error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            cause: None,
        }
    }

    /// Wraps an existing error with SDK error context
    pub fn wrap(code: ErrorCode, message: impl Into<String>, cause: impl Into<anyhow::Error>) -> Self {
        Self {
            code,
            message: message.into(),
            cause: Some(cause.into()),
        }
    }
}

// Convenience type alias
pub type Result<T> = std::result::Result<T, SdkError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_error_new() {
        let err = SdkError::new(ErrorCode::InvalidRequest, "test error");
        assert_eq!(err.code, ErrorCode::InvalidRequest);
        assert_eq!(err.message, "test error");
        assert!(err.cause.is_none());
    }

    #[test]
    fn test_sdk_error_wrap() {
        let cause = anyhow::anyhow!("underlying error");
        let err = SdkError::wrap(ErrorCode::NetworkFailure, "network error", cause);
        assert_eq!(err.code, ErrorCode::NetworkFailure);
        assert_eq!(err.message, "network error");
        assert!(err.cause.is_some());
    }

    #[test]
    fn test_error_display() {
        let err = SdkError::new(ErrorCode::Timeout, "request timeout");
        let display = format!("{}", err);
        assert!(display.contains("TIMEOUT"));
        assert!(display.contains("request timeout"));
    }
}
