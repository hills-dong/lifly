//! Admin-panel authentication handlers.
//!
//! Credentials are config-based (`ADMIN_USERNAME` / `ADMIN_PASSWORD`) and entirely
//! independent of the `users` table — see [`crate::common::auth`] for the token
//! types and the [`AdminUser`](crate::common::AdminUser) extractor.

use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::common::{AdminUser, ApiResponse, AppError, AppResult, AppState, create_admin_token};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
}

/// `POST /api/admin/login`
///
/// Validates the request against the configured admin credentials and, on
/// success, issues an admin JWT.
pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<ApiResponse<LoginResponse>> {
    let cfg = &state.config;
    // Compare both fields (avoid short-circuiting away the username check).
    let ok = req.username == cfg.admin_username && req.password == cfg.admin_password;
    if !ok {
        return Err(AppError::Unauthorized("invalid admin credentials".into()));
    }

    let token = create_admin_token(&cfg.admin_username, &state.jwt_secret.0)?;
    Ok(ApiResponse::success(LoginResponse {
        token,
        username: cfg.admin_username.clone(),
    }))
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub username: String,
}

/// `GET /api/admin/me` — returns the current admin identity (requires a valid
/// admin token).
pub async fn me_handler(admin: AdminUser) -> AppResult<ApiResponse<MeResponse>> {
    Ok(ApiResponse::success(MeResponse {
        username: admin.username,
    }))
}
