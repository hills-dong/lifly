use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use super::response::ApiResponse;

/// Error codes used in API responses.
///
/// Ranges:
///   400xx - client errors
///   500xx - server errors
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    NotFound = 40401,
    Validation = 40001,
    Unauthorized = 40101,
    Internal = 50001,
    Database = 50002,
    ExternalService = 50003,
}

/// Unified application error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Validation(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Internal(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("external service error: {0}")]
    ExternalService(String),
}

impl AppError {
    /// Return the numeric error code for this variant.
    pub fn code(&self) -> i32 {
        match self {
            Self::NotFound(_) => ErrorCode::NotFound as i32,
            Self::Validation(_) => ErrorCode::Validation as i32,
            Self::Unauthorized(_) => ErrorCode::Unauthorized as i32,
            Self::Internal(_) => ErrorCode::Internal as i32,
            Self::Database(_) => ErrorCode::Database as i32,
            Self::ExternalService(_) => ErrorCode::ExternalService as i32,
        }
    }

    /// Return the HTTP status code that should accompany this error.
    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Internal(_) | Self::Database(_) | Self::ExternalService(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let code = self.code();
        let message = self.to_string();

        // Log server-side errors at error level; client errors at warn level.
        match &self {
            AppError::Internal(_) | AppError::Database(_) | AppError::ExternalService(_) => {
                tracing::error!(%code, %message, "server error");
            }
            _ => {
                tracing::warn!(%code, %message, "client error");
            }
        }

        let body = ApiResponse::<()>::error(code, message);
        (status, axum::Json(body)).into_response()
    }
}

/// Convenience type alias for handler return values.
pub type AppResult<T> = Result<T, AppError>;
