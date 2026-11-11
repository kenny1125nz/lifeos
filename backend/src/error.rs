use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Token error: {0}")]
    Token(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn internal() -> Self {
        AppError::Internal("Internal server error".to_string())
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::Token(_) => StatusCode::UNAUTHORIZED,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Database(_) | AppError::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Auth(_) => "AUTH_ERROR",
            AppError::Token(_) => "TOKEN_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
        }
    }

    pub fn with_context<F>(self, context: F) -> Self
    where
        F: FnOnce() -> String,
    {
        match self {
            AppError::Auth(msg) => AppError::Auth(format!("{}: {}", msg, context())),
            AppError::Token(msg) => AppError::Token(format!("{}: {}", msg, context())),
            AppError::Validation(msg) => AppError::Validation(format!("{}: {}", msg, context())),
            AppError::NotFound(msg) => AppError::NotFound(format!("{}: {}", msg, context())),
            AppError::Database(e) => AppError::Database(e),
            AppError::Internal(msg) => AppError::Internal(format!("{}: {}", msg, context())),
        }
    }

    pub fn with_message(self, message: String) -> Self {
        match self {
            AppError::Auth(_) => AppError::Auth(message),
            AppError::Token(_) => AppError::Token(message),
            AppError::Validation(_) => AppError::Validation(message),
            AppError::NotFound(_) => AppError::NotFound(message),
            AppError::Database(_) => AppError::Database(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(0),
                Some(message),
            )),
            AppError::Internal(_) => AppError::Internal(message),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = json!({
            "error": {
                "code": self.error_code(),
                "message": self.to_string()
            }
        });

        (status, Json(error_response)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
