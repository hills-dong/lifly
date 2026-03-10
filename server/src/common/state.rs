use sqlx::PgPool;

use super::auth::JwtSecret;
use super::config::AppConfig;

/// Shared application state passed to all Axum handlers via `State`.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: JwtSecret,
    pub config: AppConfig,
}
