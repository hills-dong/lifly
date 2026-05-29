//! Integration tests for the operations admin panel (`/api/admin/*`).
//!
//! Requires a PostgreSQL database via `TEST_DATABASE_URL` or `DATABASE_URL`.
//! Exercises the generic registry-driven CRUD against the real schema, plus the
//! config-based auth isolation. The CRUD round-trip cleans up after itself.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

/// Build the full application router (including admin routes) for testing.
async fn app() -> Router {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set for integration tests");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to test database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run migrations");

    let (ws_tx, _) = tokio::sync::broadcast::channel(64);
    let ws_tx = std::sync::Arc::new(ws_tx);

    let config = lifly_server::common::AppConfig::from_env();
    let jwt_secret = lifly_server::common::JwtSecret(config.jwt_secret.clone());

    let state = lifly_server::common::AppState {
        pool,
        jwt_secret: jwt_secret.clone(),
        config,
        ws_tx,
    };

    let app = Router::new()
        .merge(lifly_server::identity::routes())
        .merge(lifly_server::admin::routes());

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

    app.layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let req = if let Some(b) = body {
        req.header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        req.body(Body::empty()).unwrap()
    };
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    (status, body_json(resp).await)
}

/// Log into the admin panel and return the admin token.
async fn admin_login(app: &Router) -> String {
    let (status, body) = send(
        app,
        "POST",
        "/api/admin/login",
        None,
        Some(json!({"username": "admin", "password": "admin123"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "admin login failed: {body}");
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn admin_login_succeeds_and_me_returns_identity() {
    let app = app().await;
    let token = admin_login(&app).await;

    let (status, body) = send(&app, "GET", "/api/admin/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["username"], "admin");
}

#[tokio::test]
async fn admin_login_rejects_bad_credentials() {
    let app = app().await;
    let (status, _) = send(
        &app,
        "POST",
        "/api/admin/login",
        None,
        Some(json!({"username": "admin", "password": "wrong"})),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn admin_endpoints_require_admin_token() {
    let app = app().await;

    // No token at all.
    let (status, _) = send(&app, "GET", "/api/admin/meta", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // A normal *user* token must not be accepted (token-type isolation).
    let (status, body) = send(
        &app,
        "POST",
        "/api/auth/login",
        None,
        Some(json!({"username": "admin", "password": "admin123"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let user_token = body["data"]["token"].as_str().unwrap().to_string();

    let (status, _) = send(&app, "GET", "/api/admin/meta", Some(&user_token), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn meta_lists_all_14_resources() {
    let app = app().await;
    let token = admin_login(&app).await;
    let (status, body) = send(&app, "GET", "/api/admin/meta", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    let resources = body["data"]["resources"].as_array().unwrap();
    assert_eq!(resources.len(), 14);
}

#[tokio::test]
async fn list_users_hides_password_hash() {
    let app = app().await;
    let token = admin_login(&app).await;
    let (status, body) = send(&app, "GET", "/api/admin/data/users", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["total"].as_i64().unwrap() >= 1, "seed admin user expected");
    let first = &body["data"]["items"][0];
    assert!(first.get("username").is_some(), "username should be present");
    assert!(
        first.get("password_hash").is_none(),
        "password_hash must never be exposed"
    );
}

#[tokio::test]
async fn list_data_objects_does_not_serialize_vector() {
    // data_objects has a vector(1536) column that must be excluded; listing it
    // must not error regardless of whether rows exist.
    let app = app().await;
    let token = admin_login(&app).await;
    let (status, body) =
        send(&app, "GET", "/api/admin/data/data_objects", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK, "data_objects list failed: {body}");
}

#[tokio::test]
async fn unknown_resource_is_404_and_unknown_filter_is_400() {
    let app = app().await;
    let token = admin_login(&app).await;

    let (status, _) = send(&app, "GET", "/api/admin/data/robots", Some(&token), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (status, _) = send(
        &app,
        "GET",
        "/api/admin/data/users?not_a_column=x",
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn reminder_crud_round_trip() {
    let app = app().await;
    let token = admin_login(&app).await;

    // Grab a real user id to satisfy the NOT NULL FK.
    let (_, users) = send(&app, "GET", "/api/admin/data/users", Some(&token), None).await;
    let user_id = users["data"]["items"][0]["id"].as_str().unwrap().to_string();

    // CREATE
    let (status, created) = send(
        &app,
        "POST",
        "/api/admin/data/reminders",
        Some(&token),
        Some(json!({
            "user_id": user_id,
            "title": "admin-test-reminder",
            "trigger_at": "2099-01-01T00:00:00Z",
            "status": "pending"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "create failed: {created}");
    let id = created["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(created["data"]["title"], "admin-test-reminder");

    // GET
    let (status, got) = send(
        &app,
        "GET",
        &format!("/api/admin/data/reminders/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(got["data"]["title"], "admin-test-reminder");

    // UPDATE
    let (status, updated) = send(
        &app,
        "PUT",
        &format!("/api/admin/data/reminders/{id}"),
        Some(&token),
        Some(json!({"title": "admin-test-updated"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["data"]["title"], "admin-test-updated");

    // DELETE
    let (status, _) = send(
        &app,
        "DELETE",
        &format!("/api/admin/data/reminders/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // GONE
    let (status, _) = send(
        &app,
        "GET",
        &format!("/api/admin/data/reminders/{id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
