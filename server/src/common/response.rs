use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Unified JSON envelope returned by every API endpoint.
///
/// Success:
/// ```json
/// {"code": 0, "data": {...}, "message": "ok"}
/// ```
///
/// Error:
/// ```json
/// {"code": 40001, "data": null, "message": "Validation failed: title is required"}
/// ```
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub data: Option<T>,
    pub message: String,
}

impl<T: Serialize> ApiResponse<T> {
    /// Build a success response wrapping `data`.
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            data: Some(data),
            message: "ok".to_string(),
        }
    }

    /// Build an error response (no data payload).
    pub fn error(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            data: None,
            message: message.into(),
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        axum::Json(self).into_response()
    }
}
