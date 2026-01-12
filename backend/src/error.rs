use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Proxy error: {0}")]
    Proxy(String),

    #[error("Lock error: {0}")]
    Lock(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR"),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR"),
            AppError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            AppError::Proxy(_) => (StatusCode::BAD_GATEWAY, "PROXY_ERROR"),
            AppError::Lock(_) => (StatusCode::INTERNAL_SERVER_ERROR, "LOCK_ERROR"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
        };

        let body = Json(ErrorResponse {
            error: error_code.to_string(),
            message: self.to_string(),
        });

        (status, body).into_response()
    }
}
