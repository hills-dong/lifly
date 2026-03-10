//! Integration tests for pipeline execution with a real Gemini LLM API.
//!
//! These tests are gated by the `LLM_API_KEY` environment variable. If the key
//! is not set or is "not-configured", the tests will be skipped.
//!
//! To run:
//! ```sh
//! LLM_API_KEY=your-key cargo test llm_integration -- --test-threads=1
//! ```

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

fn has_llm_key() -> bool {
    std::env::var("LLM_API_KEY")
        .map(|k| !k.is_empty() && k != "not-configured")
        .unwrap_or(false)
}

/// Build the full application router for testing (same as integration.rs).
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
        .route(
            "/api/ws",
            axum::routing::get(lifly_server::common::ws::ws_handler),
        );

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

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

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

#[tokio::test]
async fn test_todo_pipeline_with_llm() {
    if !has_llm_key() {
        eprintln!("SKIPPED: test_todo_pipeline_with_llm (LLM_API_KEY not set)");
        return;
    }

    let (app, _pool) = app().await;
    let token = login(&app).await;

    let todo_tool_id = "00000000-0000-0000-0000-000000000201";

    // Submit a raw input for the Todo tool.
    let (status, body) = auth_post(
        &app,
        "/api/raw-inputs",
        &token,
        serde_json::json!({
            "tool_id": todo_tool_id,
            "input_type": "text",
            "raw_content": "明天下午3点去超市买菜"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["data"]["processing_status"].as_str().unwrap(),
        "processing"
    );
    let pipeline_id = body["data"]["pipeline_id"]
        .as_str()
        .expect("should have pipeline_id");

    // Wait for the pipeline to complete (runs async via tokio::spawn).
    // Poll up to 15 seconds.
    let mut pipeline_status = String::new();
    for _ in 0..15 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let (st, pb) = auth_get(&app, &format!("/api/pipelines/{pipeline_id}"), &token).await;
        if st == StatusCode::OK {
            pipeline_status = pb["data"]["status"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if pipeline_status == "completed" || pipeline_status == "failed" {
                break;
            }
        }
    }

    assert_eq!(
        pipeline_status, "completed",
        "pipeline should complete successfully with LLM"
    );

    // Verify a DataObject was created.
    let (status, body) = auth_get(
        &app,
        &format!("/api/data-objects?tool_id={todo_tool_id}"),
        &token,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let objects = body["data"].as_array().unwrap();
    assert!(
        !objects.is_empty(),
        "at least one data object should have been created by the pipeline"
    );

    // The most recent data object should have structured attributes.
    let latest = &objects[0];
    assert!(
        latest["attributes"].is_object(),
        "data object should have structured attributes"
    );
}

#[tokio::test]
async fn test_ocr_pipeline_with_llm() {
    if !has_llm_key() {
        eprintln!("SKIPPED: test_ocr_pipeline_with_llm (LLM_API_KEY not set)");
        return;
    }

    let (app, _pool) = app().await;
    let token = login(&app).await;

    let id_tool_id = "00000000-0000-0000-0000-000000000202";

    // A minimal 1x1 white PNG encoded in base64.
    let tiny_png_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8/5+hHgAHggJ/PchI7wAAAABJRU5ErkJggg==";

    // Submit a raw input with image content for the ID management tool.
    let (status, body) = auth_post(
        &app,
        "/api/raw-inputs",
        &token,
        serde_json::json!({
            "tool_id": id_tool_id,
            "input_type": "image",
            "raw_content": tiny_png_base64,
            "metadata": {
                "mime_type": "image/png"
            }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let pipeline_id = body["data"]["pipeline_id"]
        .as_str()
        .expect("should have pipeline_id");

    // Wait for the pipeline to complete.
    let mut pipeline_status = String::new();
    let mut pipeline_body = serde_json::json!(null);
    for _ in 0..15 {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let (st, pb) = auth_get(&app, &format!("/api/pipelines/{pipeline_id}"), &token).await;
        if st == StatusCode::OK {
            pipeline_status = pb["data"]["status"]
                .as_str()
                .unwrap_or("")
                .to_string();
            pipeline_body = pb;
            if pipeline_status == "completed" || pipeline_status == "failed" {
                break;
            }
        }
    }

    // The pipeline should at least finish (completed or failed).
    assert!(
        pipeline_status == "completed" || pipeline_status == "failed",
        "pipeline should finish, got status: {pipeline_status}, body: {pipeline_body}"
    );

    // If completed, verify a DataObject was created.
    if pipeline_status == "completed" {
        let (status, body) = auth_get(
            &app,
            &format!("/api/data-objects?tool_id={id_tool_id}"),
            &token,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        let objects = body["data"].as_array().unwrap();
        assert!(
            !objects.is_empty(),
            "at least one data object should have been created by the OCR pipeline"
        );
    }
}
