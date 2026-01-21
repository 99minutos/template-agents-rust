#![allow(dead_code)]
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

// ============================================================================
// 1. ERROR KINDS - Categorías de Errores de Bajo Nivel
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RedisKind {
    #[error("connection failed")]
    Connection,
    #[error("serialization error")]
    Serialization,
    #[error("session not found")]
    SessionNotFound,
    #[error("operation timeout")]
    Timeout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum LlmKind {
    #[error("model unavailable")]
    Unavailable,
    #[error("rate limit exceeded")]
    RateLimit,
    #[error("context too long")]
    ContextTooLong,
    #[error("invalid response")]
    InvalidResponse,
    #[error("timeout")]
    Timeout,
}

// ============================================================================
// 2. ERROR KIND - Categoría Principal de Errores
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ErrorKind {
    #[error("not found")]
    NotFound,

    #[error("validation error")]
    Validation,

    #[error("redis error: {0}")]
    Redis(#[from] RedisKind),

    #[error("llm error: {0}")]
    Llm(#[from] LlmKind),

    #[error("unauthorized")]
    Unauthorized,

    #[error("service unavailable")]
    ServiceUnavailable,

    #[error("internal error")]
    Internal,
}

impl ErrorKind {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            ErrorKind::Validation => StatusCode::BAD_REQUEST,
            ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorKind::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ErrorKind::Redis(RedisKind::SessionNotFound) => StatusCode::NOT_FOUND,
            ErrorKind::Redis(_) => StatusCode::SERVICE_UNAVAILABLE,
            ErrorKind::Llm(LlmKind::RateLimit) => StatusCode::TOO_MANY_REQUESTS,
            ErrorKind::Llm(_) => StatusCode::SERVICE_UNAVAILABLE,
            ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            ErrorKind::NotFound => "NOT_FOUND",
            ErrorKind::Validation => "VALIDATION_ERROR",
            ErrorKind::Unauthorized => "UNAUTHORIZED",
            ErrorKind::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            ErrorKind::Redis(RedisKind::SessionNotFound) => "SESSION_NOT_FOUND",
            ErrorKind::Redis(_) => "REDIS_ERROR",
            ErrorKind::Llm(LlmKind::RateLimit) => "RATE_LIMIT_EXCEEDED",
            ErrorKind::Llm(LlmKind::ContextTooLong) => "CONTEXT_TOO_LONG",
            ErrorKind::Llm(_) => "LLM_ERROR",
            ErrorKind::Internal => "INTERNAL_ERROR",
        }
    }
}

// ============================================================================
// 3. DOMAIN ERROR - Error Principal del Sistema
// ============================================================================

#[derive(Debug, Error)]
#[error("{kind}: {message}")]
pub struct DomainError {
    kind: ErrorKind,
    message: String,
    data: Option<serde_json::Value>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl DomainError {
    pub fn new<M: Into<String>>(kind: ErrorKind, message: M) -> Self {
        Self {
            kind,
            message: message.into(),
            data: None,
            source: None,
        }
    }

    pub fn with_data<D: Serialize>(mut self, data: D) -> Self {
        self.data = serde_json::to_value(data).ok();
        self
    }

    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn data(&self) -> Option<&serde_json::Value> {
        self.data.as_ref()
    }

    pub fn is_not_found(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::NotFound | ErrorKind::Redis(RedisKind::SessionNotFound)
        )
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::ServiceUnavailable
                | ErrorKind::Redis(RedisKind::Connection)
                | ErrorKind::Redis(RedisKind::Timeout)
                | ErrorKind::Llm(LlmKind::Unavailable)
                | ErrorKind::Llm(LlmKind::Timeout)
                | ErrorKind::Llm(LlmKind::RateLimit)
        )
    }

    pub fn is_client_error(&self) -> bool {
        matches!(
            self.kind,
            ErrorKind::NotFound | ErrorKind::Validation | ErrorKind::Unauthorized
        )
    }
}

// ============================================================================
// 4. CONSTRUCTORES CONVENIENTES
// ============================================================================

impl DomainError {
    pub fn not_found<M: Into<String>>(message: M) -> Self {
        Self::new(ErrorKind::NotFound, message)
    }

    pub fn validation<M: Into<String>>(message: M) -> Self {
        Self::new(ErrorKind::Validation, message)
    }

    pub fn unauthorized<M: Into<String>>(message: M) -> Self {
        Self::new(ErrorKind::Unauthorized, message)
    }

    pub fn internal<M: Into<String>>(message: M) -> Self {
        Self::new(ErrorKind::Internal, message)
    }

    pub fn redis<M: Into<String>>(kind: RedisKind, message: M) -> Self {
        Self::new(ErrorKind::Redis(kind), message)
    }

    pub fn llm<M: Into<String>>(kind: LlmKind, message: M) -> Self {
        Self::new(ErrorKind::Llm(kind), message)
    }

    pub fn session_not_found(session_id: &str) -> Self {
        Self::new(
            ErrorKind::Redis(RedisKind::SessionNotFound),
            format!("Session '{}' not found", session_id),
        )
    }
}

// ============================================================================
// 5. CONVERSIONES DESDE OTROS ERRORES
// ============================================================================

impl From<redis::RedisError> for DomainError {
    fn from(err: redis::RedisError) -> Self {
        let kind = if err.is_connection_dropped() || err.is_io_error() {
            RedisKind::Connection
        } else if err.is_timeout() {
            RedisKind::Timeout
        } else {
            RedisKind::Connection
        };

        Self::new(ErrorKind::Redis(kind), err.to_string()).with_source(err)
    }
}

impl From<serde_json::Error> for DomainError {
    fn from(err: serde_json::Error) -> Self {
        Self::new(
            ErrorKind::Redis(RedisKind::Serialization),
            format!("JSON serialization error: {}", err),
        )
        .with_source(err)
    }
}

impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        Self::new(ErrorKind::Internal, err.to_string())
    }
}

// ============================================================================
// 6. RESPUESTA HTTP (Axum IntoResponse)
// ============================================================================

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

impl IntoResponse for DomainError {
    fn into_response(self) -> Response {
        let status = self.kind.status_code();
        let error_code = self.kind.error_code();

        if !self.is_client_error() {
            tracing::error!(
                error_code = error_code,
                message = %self.message,
                "Domain error occurred"
            );
        }

        let body = ErrorResponse {
            error: ErrorBody {
                code: error_code,
                message: self.message,
                data: self.data,
            },
        };

        (status, Json(body)).into_response()
    }
}

// ============================================================================
// 7. TYPE ALIAS PARA RESULTS
// ============================================================================

pub type DomainResult<T> = Result<T, DomainError>;

// ============================================================================
// 8. TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = DomainError::validation("Invalid prompt");
        assert_eq!(error.kind(), ErrorKind::Validation);
        assert_eq!(error.message(), "Invalid prompt");
    }

    #[test]
    fn test_error_with_data() {
        #[derive(Serialize)]
        struct Details {
            field: String,
        }

        let error = DomainError::validation("Invalid field").with_data(Details {
            field: "prompt".to_string(),
        });

        assert!(error.data().is_some());
    }

    #[test]
    fn test_is_retryable() {
        let retryable = DomainError::llm(LlmKind::RateLimit, "Too many requests");
        assert!(retryable.is_retryable());

        let not_retryable = DomainError::validation("Bad input");
        assert!(!not_retryable.is_retryable());
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(ErrorKind::NotFound.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(ErrorKind::Validation.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(
            ErrorKind::Llm(LlmKind::RateLimit).status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(ErrorKind::NotFound.error_code(), "NOT_FOUND");
        assert_eq!(
            ErrorKind::Redis(RedisKind::SessionNotFound).error_code(),
            "SESSION_NOT_FOUND"
        );
    }
}
