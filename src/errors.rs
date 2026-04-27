use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Idempotency key already exists")]
    IdempotencyConflict,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::IdempotencyConflict => (StatusCode::CONFLICT, "Idempotency key already exists".to_string()),
            AppError::InsufficientFunds => (StatusCode::BAD_REQUEST, "Insufficient funds".to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };
        (status, message).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_validation_error() {
        let error = AppError::Validation("Test message".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_idempotency_conflict_error() {
        let error = AppError::IdempotencyConflict;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_insufficient_funds_error() {
        let error = AppError::InsufficientFunds;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_database_error() {
        let db_error = sqlx::Error::RowNotFound;
        let error = AppError::Database(db_error);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_internal_error() {
        let internal_error = anyhow::anyhow!("Test error");
        let error = AppError::Internal(internal_error);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display() {
        let error = AppError::Validation("Test".to_string());
        assert_eq!(format!("{}", error), "Validation error: Test");

        let error = AppError::IdempotencyConflict;
        assert_eq!(format!("{}", error), "Idempotency key already exists");
    }
}