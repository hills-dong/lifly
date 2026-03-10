use argon2::{Argon2, PasswordHash, PasswordVerifier};
use sqlx::PgPool;
use uuid::Uuid;

use crate::common::{AppError, AppResult, JwtSecret, create_token};

use super::models::{
    CreateDeviceRequest, DeviceResponse, LoginResponse, UpdateProfileRequest, UserProfile,
};
use super::repo;

/// Authenticate a user by username and password, returning a JWT and profile.
pub async fn login(
    pool: &PgPool,
    jwt_secret: &JwtSecret,
    username: &str,
    password: &str,
) -> AppResult<LoginResponse> {
    let user = repo::find_user_by_username(pool, username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid username or password".to_string()))?;

    // Verify password against stored Argon2 hash.
    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|e| AppError::Internal(format!("invalid stored password hash: {e}")))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized("invalid username or password".to_string()))?;

    let token = create_token(user.id, &jwt_secret.0)?;
    let profile = UserProfile::from(user);

    Ok(LoginResponse {
        token,
        user: profile,
    })
}

/// Return the profile for the given user ID.
pub async fn get_profile(pool: &PgPool, user_id: Uuid) -> AppResult<UserProfile> {
    let user = repo::find_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

    Ok(UserProfile::from(user))
}

/// Update the authenticated user's profile.
pub async fn update_profile(
    pool: &PgPool,
    user_id: Uuid,
    req: UpdateProfileRequest,
) -> AppResult<UserProfile> {
    let user = repo::update_user_profile(
        pool,
        user_id,
        req.display_name.as_deref(),
        req.preferences.as_ref(),
    )
    .await?;

    Ok(UserProfile::from(user))
}

/// List all devices belonging to the authenticated user.
pub async fn list_devices(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<DeviceResponse>> {
    let devices = repo::list_devices(pool, user_id).await?;
    Ok(devices.into_iter().map(DeviceResponse::from).collect())
}

/// Register a new device for the authenticated user.
pub async fn create_device(
    pool: &PgPool,
    user_id: Uuid,
    req: CreateDeviceRequest,
) -> AppResult<DeviceResponse> {
    let device = repo::create_device(
        pool,
        user_id,
        &req.name,
        &req.device_type,
        req.platform.as_deref(),
        req.token.as_deref(),
    )
    .await?;

    Ok(DeviceResponse::from(device))
}
