use axum::extract::{Json, State};
use axum::routing::{get, post, put};
use axum::Router;

use crate::common::state::AppState;
use crate::common::{ApiResponse, AppResult, AuthUser};

use super::models::{CreateDeviceRequest, DeviceResponse, LoginRequest, LoginResponse, UpdateProfileRequest, UserProfile};
use super::service;

/// POST /api/auth/login
async fn login_handler(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<ApiResponse<LoginResponse>> {
    let resp = service::login(&state.pool, &state.jwt_secret, &body.username, &body.password).await?;
    Ok(ApiResponse::success(resp))
}

/// POST /api/auth/logout
async fn logout_handler() -> AppResult<ApiResponse<()>> {
    // Server-side logout is a no-op; the client discards its token.
    Ok(ApiResponse::success(()))
}

/// GET /api/user/profile
async fn get_profile_handler(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<ApiResponse<UserProfile>> {
    let profile = service::get_profile(&state.pool, auth.user_id).await?;
    Ok(ApiResponse::success(profile))
}

/// PUT /api/user/profile
async fn update_profile_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> AppResult<ApiResponse<UserProfile>> {
    let profile = service::update_profile(&state.pool, auth.user_id, body).await?;
    Ok(ApiResponse::success(profile))
}

/// GET /api/user/devices
async fn list_devices_handler(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<ApiResponse<Vec<DeviceResponse>>> {
    let devices = service::list_devices(&state.pool, auth.user_id).await?;
    Ok(ApiResponse::success(devices))
}

/// POST /api/user/devices
async fn create_device_handler(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateDeviceRequest>,
) -> AppResult<ApiResponse<DeviceResponse>> {
    let device = service::create_device(&state.pool, auth.user_id, body).await?;
    Ok(ApiResponse::success(device))
}

/// Build the router subtree for the identity module.
///
/// Mount points:
///   - `/api/auth/*`  — login, logout
///   - `/api/user/*`  — profile, devices
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/login", post(login_handler))
        .route("/api/auth/logout", post(logout_handler))
        .route("/api/user/profile", get(get_profile_handler).put(update_profile_handler))
        .route("/api/user/devices", get(list_devices_handler).post(create_device_handler))
}
