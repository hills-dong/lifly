use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;

use super::auth::JwtSecret;
use super::config::AppConfig;

/// A WebSocket event broadcast to connected clients.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WsEvent {
    /// Event type, e.g. "pipeline.status"
    #[serde(rename = "type")]
    pub event_type: String,
    /// JSON payload
    pub payload: serde_json::Value,
}

/// Shared application state passed to all Axum handlers via `State`.
#[derive(Debug, Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: JwtSecret,
    pub config: AppConfig,
    pub ws_tx: Arc<broadcast::Sender<WsEvent>>,
}
