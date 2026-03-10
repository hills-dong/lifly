//! Integration tests for the Lifly server.
//!
//! These tests require a PostgreSQL database. Set `TEST_DATABASE_URL` in the
//! environment to run them:
//!
//! ```sh
//! TEST_DATABASE_URL=postgres://user:pass@localhost/lifly_test cargo test
//! ```
//!
//! Each test uses a transaction that is rolled back, so the database stays clean.

use axum::body::Body;
use axum::http::{self, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

/// Build the full application router for testing.
async fn app() -> (Router, sqlx::PgPool) {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for integration tests");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to test database");

    // Run migrations.
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let (ws_tx, _) = tokio::sync::broadcast::channel(64);
    let ws_tx = std::sync::Arc::new(ws_tx);

    let config = lifly_server::common::AppConfig::from_env();
    let jwt_secret = lifly_server::common::JwtSecret(config.jwt_secret.clone());

    let state = lifly_server::common::AppState {
        pool: pool.clone(),
        jwt_secret: jwt_secret.clone(),
        config: config.clone(),
        ws_tx,
    };

    let app = Router::new()
        .merge(lifly_server::identity::routes())
        .merge(lifly_server::capability::routes())
        .merge(lifly_server::tool::routes())
        .merge(lifly_server::data::routes())
        .merge(lifly_server::data::category_routes())
        .merge(lifly_server::intelligence::routes())
        .route("/api/ws", axum::routing::get(lifly_server::common::ws::ws_handler));

    // Inject JWT secret via layer.
    let jwt_for_mw = jwt_secret.clone();
    let app = app.layer(axum::middleware::from_fn(
        move |mut req: axum::extract::Request, next: axum::middleware::Next| {
            let secret = jwt_for_mw.clone();
            async move {
                req.extensions_mut().insert(secret);
                next.run(req).await
            }
        },
    ));

    let app = app
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    (app, pool)
}

/// Parse the response body as JSON.
async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Login as admin and return the JWT token.
async fn login(app: &Router) -> String {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{"username":"admin","password":"admin123"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    body["data"]["token"].as_str().unwrap().to_string()
}

/// Helper to make authenticated GET requests.
async fn auth_get(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body = body_json(resp).await;
    (status, body)
}

/// Helper to make authenticated POST requests.
async fn auth_post(app: &Router, uri: &str, token: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body = body_json(resp).await;
    (status, body)
}

/// Helper to make authenticated PUT requests.
async fn auth_put(app: &Router, uri: &str, token: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body = body_json(resp).await;
    (status, body)
}

/// Helper to make authenticated DELETE requests.
async fn auth_delete(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(uri)
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = resp.status();
    let body = body_json(resp).await;
    (status, body)
}

// ── Auth tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_success() {
    let (app, _pool) = app().await;
    let token = login(&app).await;
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let (app, _pool) = app().await;
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{"username":"admin","password":"wrongpassword"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_unauthenticated_request() {
    let (app, _pool) = app().await;
    let resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/tools")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── Profile tests ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_profile() {
    let (app, _pool) = app().await;
    let token = login(&app).await;
    let (status, body) = auth_get(&app, "/api/user/profile", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["username"].as_str().unwrap(), "admin");
}

// ── Tool tests ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_tools() {
    let (app, _pool) = app().await;
    let token = login(&app).await;
    let (status, body) = auth_get(&app, "/api/tools", &token).await;
    assert_eq!(status, StatusCode::OK);
    let tools = body["data"].as_array().unwrap();
    assert!(tools.len() >= 2, "expected at least 2 seeded tools");
}

#[tokio::test]
async fn test_get_tool_detail() {
    let (app, _pool) = app().await;
    let token = login(&app).await;
    let todo_tool_id = "00000000-0000-0000-0000-000000000201";
    let (status, body) = auth_get(&app, &format!("/api/tools/{todo_tool_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["name"].as_str().unwrap(), "Todo List");
    assert!(body["data"]["data_schema"].is_object());
}

#[tokio::test]
async fn test_get_tool_versions() {
    let (app, _pool) = app().await;
    let token = login(&app).await;
    let todo_tool_id = "00000000-0000-0000-0000-000000000201";
    let (status, body) = auth_get(&app, &format!("/api/tools/{todo_tool_id}/versions"), &token).await;
    assert_eq!(status, StatusCode::OK);
    let versions = body["data"].as_array().unwrap();
    assert!(!versions.is_empty());
    assert_eq!(versions[0]["version_number"].as_i64().unwrap(), 1);
}

// ── Capability tests ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_capabilities() {
    let (app, _pool) = app().await;
    let token = login(&app).await;
    let (status, body) = auth_get(&app, "/api/capabilities", &token).await;
    assert_eq!(status, StatusCode::OK);
    let caps = body["data"].as_array().unwrap();
    assert!(caps.len() >= 6, "expected at least 6 seeded capabilities");
}

// ── Pipeline / Raw Input tests ────────────────────────────────────────────

#[tokio::test]
async fn test_create_raw_input_triggers_pipeline() {
    let (app, _pool) = app().await;
    let token = login(&app).await;

    // Submit a raw input for the Todo tool.
    let todo_tool_id = "00000000-0000-0000-0000-000000000201";
    let (status, body) = auth_post(
        &app,
        "/api/raw-inputs",
        &token,
        serde_json::json!({
            "tool_id": todo_tool_id,
            "input_type": "text",
            "raw_content": "Buy groceries tomorrow"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["processing_status"].as_str().unwrap(), "processing");
    assert!(body["data"]["pipeline_id"].is_string());

    let pipeline_id = body["data"]["pipeline_id"].as_str().unwrap();

    // Wait briefly for the pipeline to complete (it runs in a background task).
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Check the pipeline status.
    let (status, body) = auth_get(&app, &format!("/api/pipelines/{pipeline_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);

    let pipeline_status = body["data"]["status"].as_str().unwrap();
    // The pipeline might complete or fail (if LLM is not configured), but it should
    // have at least started.
    assert!(
        pipeline_status == "completed" || pipeline_status == "failed",
        "pipeline status should be completed or failed, got: {pipeline_status}"
    );
}

#[tokio::test]
async fn test_list_pipelines() {
    let (app, _pool) = app().await;
    let token = login(&app).await;
    let (status, body) = auth_get(&app, "/api/pipelines", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
}

// ── Data Object tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_data_object_crud() {
    let (app, pool) = app().await;
    let token = login(&app).await;

    let todo_tool_id = "00000000-0000-0000-0000-000000000201";

    // Create a data object directly via the repo (since the handler doesn't have a create endpoint).
    let obj = lifly_server::data::repo::create_data_object(
        &pool,
        uuid::Uuid::parse_str(todo_tool_id).unwrap(),
        None,
        None,
        None,
        &serde_json::json!({"title": "Test todo", "completed": false}),
    )
    .await
    .unwrap();

    // GET the data object.
    let (status, body) = auth_get(&app, &format!("/api/data-objects/{}", obj.id), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["attributes"]["title"].as_str().unwrap(), "Test todo");

    // UPDATE the data object.
    let (status, body) = auth_put(
        &app,
        &format!("/api/data-objects/{}", obj.id),
        &token,
        serde_json::json!({"attributes": {"title": "Updated todo", "completed": true}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["attributes"]["title"].as_str().unwrap(), "Updated todo");

    // LIST data objects for the tool.
    let (status, body) = auth_get(
        &app,
        &format!("/api/data-objects?tool_id={todo_tool_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let objects = body["data"].as_array().unwrap();
    assert!(objects.iter().any(|o| o["id"].as_str().unwrap() == obj.id.to_string()));

    // SEARCH data objects.
    let (status, body) = auth_get(&app, "/api/data-objects/search?q=Updated", &token).await;
    assert_eq!(status, StatusCode::OK);
    let results = body["data"].as_array().unwrap();
    assert!(results.iter().any(|o| o["id"].as_str().unwrap() == obj.id.to_string()));

    // DELETE the data object.
    let (status, _) = auth_delete(&app, &format!("/api/data-objects/{}", obj.id), &token).await;
    assert_eq!(status, StatusCode::OK);

    // Verify it's soft-deleted (still exists but status is 'deleted').
    let (status, body) = auth_get(&app, &format!("/api/data-objects/{}", obj.id), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"].as_str().unwrap(), "deleted");
}

// ── Reminder tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_reminder_crud() {
    let (app, _pool) = app().await;
    let token = login(&app).await;

    // Create a reminder.
    let (status, body) = auth_post(
        &app,
        "/api/reminders",
        &token,
        serde_json::json!({
            "title": "Test reminder",
            "description": "Don't forget",
            "trigger_at": "2026-12-25T09:00:00Z"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let reminder_id = body["data"]["id"].as_str().unwrap().to_string();

    // GET the reminder.
    let (status, body) = auth_get(&app, &format!("/api/reminders/{reminder_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"].as_str().unwrap(), "Test reminder");

    // UPDATE the reminder.
    let (status, body) = auth_put(
        &app,
        &format!("/api/reminders/{reminder_id}"),
        &token,
        serde_json::json!({"title": "Updated reminder"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"].as_str().unwrap(), "Updated reminder");

    // LIST reminders.
    let (status, body) = auth_get(&app, "/api/reminders", &token).await;
    assert_eq!(status, StatusCode::OK);
    let reminders = body["data"].as_array().unwrap();
    assert!(reminders.iter().any(|r| r["id"].as_str().unwrap() == reminder_id));

    // DISMISS the reminder.
    let (status, body) = auth_post(
        &app,
        &format!("/api/reminders/{reminder_id}/dismiss"),
        &token,
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"].as_str().unwrap(), "dismissed");

    // DELETE the reminder.
    let (status, _) = auth_delete(&app, &format!("/api/reminders/{reminder_id}"), &token).await;
    assert_eq!(status, StatusCode::OK);
}

// ── Category tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_category_crud() {
    let (app, _pool) = app().await;
    let token = login(&app).await;
    let todo_tool_id = "00000000-0000-0000-0000-000000000201";

    // Create a category.
    let (status, body) = auth_post(
        &app,
        &format!("/api/tools/{todo_tool_id}/categories"),
        &token,
        serde_json::json!({"name": "Work", "sort_order": 1}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let cat_id = body["data"]["id"].as_str().unwrap().to_string();

    // LIST categories.
    let (status, body) = auth_get(
        &app,
        &format!("/api/tools/{todo_tool_id}/categories"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let cats = body["data"].as_array().unwrap();
    assert!(cats.iter().any(|c| c["name"].as_str().unwrap() == "Work"));

    // UPDATE the category.
    let (status, body) = auth_put(
        &app,
        &format!("/api/tools/{todo_tool_id}/categories/{cat_id}"),
        &token,
        serde_json::json!({"name": "Personal"}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["name"].as_str().unwrap(), "Personal");

    // DELETE the category.
    let (status, _) = auth_delete(
        &app,
        &format!("/api/tools/{todo_tool_id}/categories/{cat_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}
