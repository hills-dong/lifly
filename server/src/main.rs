mod capability;
mod common;
mod data;
mod identity;
mod intelligence;
mod tool;

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::middleware;
use axum::routing::get;
use chrono::Utc;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use common::{AppConfig, AppState, JwtSecret, WsEvent};

fn static_files_path() -> String {
    std::env::var("STATIC_DIR")
        .or_else(|_| std::env::var("static_dir"))
        .unwrap_or_else(|_| "./static".to_string())
}

#[tokio::main]
async fn main() {
    // Load .env file (ignore if missing).
    dotenvy::dotenv().ok();

    // Initialize tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lifly_server=debug,tower_http=debug".into()),
        )
        .init();

    // Read configuration from environment variables.
    let config = AppConfig::from_env();

    // Create the Postgres connection pool.
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    // Run pending migrations.
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run database migrations");

    // Create WebSocket broadcast channel.
    let (ws_tx, _) = broadcast::channel::<WsEvent>(256);
    let ws_tx = Arc::new(ws_tx);

    // Build shared application state.
    let jwt_secret = JwtSecret(config.jwt_secret.clone());
    let state = AppState {
        pool: pool.clone(),
        jwt_secret: jwt_secret.clone(),
        config: config.clone(),
        ws_tx: ws_tx.clone(),
    };

    // Build the application router by merging all module routes.
    let mut app = Router::new()
        .merge(identity::routes())
        .merge(capability::routes())
        .merge(tool::routes())
        .merge(data::routes())
        .merge(data::category_routes())
        .merge(intelligence::routes())
        .route("/api/ws", get(common::ws::ws_handler));

    // Serve static files from STATIC_FILES_PATH if the directory exists.
    let static_dir = static_files_path();
    if Path::new(&static_dir).is_dir() {
        let index_path = format!("{static_dir}/index.html");
        app = app.fallback_service(
            tower_http::services::ServeDir::new(&static_dir).fallback(
                tower_http::services::ServeFile::new(index_path),
            ),
        );
    }

    // Inject JWT secret into request extensions so AuthUser extractor can access it.
    let jwt_for_middleware = jwt_secret.clone();
    let app = app.layer(middleware::from_fn(move |mut req: axum::extract::Request, next: middleware::Next| {
        let secret = jwt_for_middleware.clone();
        async move {
            req.extensions_mut().insert(secret);
            next.run(req).await
        }
    }));

    // Add middleware layers.
    let app = app
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Spawn background task for checking pending reminders every 60 seconds.
    let reminder_pool = pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            match intelligence::repo::list_pending_reminders_before(&reminder_pool, Utc::now())
                .await
            {
                Ok(reminders) => {
                    for reminder in reminders {
                        tracing::info!(
                            reminder_id = %reminder.id,
                            title = %reminder.title,
                            "Reminder triggered"
                        );
                        // Mark the reminder as dismissed so it won't fire again.
                        if let Err(e) =
                            intelligence::repo::dismiss_reminder(&reminder_pool, reminder.id).await
                        {
                            tracing::error!(
                                reminder_id = %reminder.id,
                                error = %e,
                                "failed to dismiss triggered reminder"
                            );
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "failed to check pending reminders");
                }
            }
        }
    });

    // Start the HTTP server.
    let addr = config.socket_addr();
    tracing::info!("Lifly server listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");
    axum::serve(listener, app).await.expect("server error");
}
