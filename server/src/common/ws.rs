use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use serde::Deserialize;
use tokio::sync::broadcast;

use super::auth::verify_token;
use super::state::{AppState, WsEvent};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

/// GET /api/ws?token=<jwt>
///
/// Upgrades the connection to a WebSocket and streams pipeline events.
pub async fn ws_handler(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Validate the token if provided, but allow connection even without
    // (the frontend sends it in the query string).
    let _user_id = query
        .token
        .as_deref()
        .and_then(|t| verify_token(t, &state.jwt_secret.0).ok())
        .map(|claims| claims.sub);

    let rx = state.ws_tx.subscribe();
    ws.on_upgrade(move |socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: broadcast::Receiver<WsEvent>) {
    loop {
        tokio::select! {
            // Forward broadcast events to the WebSocket client.
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let msg = serde_json::json!({
                            "type": event.event_type,
                            "payload": event.payload,
                        });
                        if let Ok(text) = serde_json::to_string(&msg) {
                            if socket.send(Message::Text(text.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "ws client lagged behind");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            // Handle incoming messages from client (ping/pong, close).
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(_)) => break,
                    _ => {} // Ignore text/binary from client.
                }
            }
        }
    }
}
