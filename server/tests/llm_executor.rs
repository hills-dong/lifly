//! Unit tests for the remote LLM executor using wiremock to mock the Gemini API.

use serde_json::{json, Value};
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use lifly_server::common::{AppConfig, AppError};
use lifly_server::tool::models::AtomicCapability;
use lifly_server::tool::pipeline::executor::{ExecutionContext, StepExecutor};

/// Create an AppConfig pointing at the given mock server URI.
fn test_config(mock_uri: &str) -> AppConfig {
    AppConfig {
        database_url: String::new(),
        file_storage_path: PathBuf::new(),
        llm_api_key: "test-key".to_string(),
        llm_api_url: mock_uri.to_string(),
        jwt_secret: "test-secret".to_string(),
        server_host: IpAddr::from_str("127.0.0.1").unwrap(),
        server_port: 0,
    }
}

/// Create a dummy ExecutionContext. remote_llm doesn't use pool/ids,
/// but we need a PgPool. Connect to the test database.
async fn test_context() -> ExecutionContext {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TEST_DATABASE_URL or DATABASE_URL must be set");

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("failed to connect to test database");

    ExecutionContext {
        pool,
        tool_id: Uuid::nil(),
        pipeline_id: Uuid::nil(),
        user_id: Uuid::nil(),
    }
}

/// Create a capability with runtime_type = "remote_llm" and the given runtime_config.
fn llm_capability(runtime_config: Option<Value>) -> AtomicCapability {
    AtomicCapability {
        id: Uuid::new_v4(),
        name: "test_llm".to_string(),
        runtime_type: "remote_llm".to_string(),
        runtime_config,
    }
}

/// Standard Gemini text response.
fn gemini_text_response(text: &str) -> Value {
    json!({
        "candidates": [{
            "content": {
                "role": "model",
                "parts": [{"text": text}]
            }
        }]
    })
}

/// Gemini response with image + text.
fn gemini_image_response(text: &str, mime_type: &str, data: &str) -> Value {
    json!({
        "candidates": [{
            "content": {
                "role": "model",
                "parts": [
                    {"inlineData": {"mimeType": mime_type, "data": data}},
                    {"text": text}
                ]
            }
        }]
    })
}

#[tokio::test]
async fn test_text_mode_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"/v1beta/models/.+:generateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gemini_text_response("Hello from LLM")))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let ctx = test_context().await;
    let cap = llm_capability(Some(json!({
        "mode": "text",
        "system_prompt": "Be helpful"
    })));

    let input = json!({"text": "What is 2+2?"});
    let result = StepExecutor::execute(&cap, input, &config, &ctx).await.unwrap();

    assert_eq!(result["result"].as_str().unwrap(), "Hello from LLM");
    assert!(result["model"].is_string());
}

#[tokio::test]
async fn test_vision_mode_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"/v1beta/models/.+:generateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gemini_text_response("OCR: some text here")))
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let ctx = test_context().await;
    let cap = llm_capability(Some(json!({
        "mode": "vision",
        "system_prompt": "Extract text from this image"
    })));

    let input = json!({
        "image_base64": "iVBORw0KGgoAAAANSUhEUg==",
        "mime_type": "image/png",
        "text": "What text is in this image?"
    });
    let result = StepExecutor::execute(&cap, input, &config, &ctx).await.unwrap();

    assert_eq!(result["result"].as_str().unwrap(), "OCR: some text here");
    assert!(result["model"].is_string());
}

#[tokio::test]
async fn test_image_generation_mode_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"/v1beta/models/.+:generateContent"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(gemini_image_response("processed", "image/png", "base64imagedata")),
        )
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let ctx = test_context().await;
    let cap = llm_capability(Some(json!({
        "mode": "image_generation",
        "system_prompt": "Process this image"
    })));

    let input = json!({
        "image_base64": "iVBORw0KGgoAAAANSUhEUg==",
        "mime_type": "image/png",
        "text": "Enhance this image"
    });
    let result = StepExecutor::execute(&cap, input, &config, &ctx).await.unwrap();

    assert_eq!(result["result"].as_str().unwrap(), "processed");
    assert_eq!(result["image"]["mime_type"].as_str().unwrap(), "image/png");
    assert_eq!(result["image"]["data"].as_str().unwrap(), "base64imagedata");
    assert!(result["model"].is_string());
}

#[tokio::test]
async fn test_missing_runtime_config() {
    let mock_server = MockServer::start().await;
    let config = test_config(&mock_server.uri());
    let ctx = test_context().await;
    let cap = llm_capability(None);

    let input = json!({"text": "hello"});
    let result = StepExecutor::execute(&cap, input, &config, &ctx).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        AppError::Internal(msg) => {
            assert!(
                msg.contains("runtime_config"),
                "error should mention runtime_config, got: {msg}"
            );
        }
        other => panic!("expected Internal error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_api_error_response() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path_regex(r"/v1beta/models/.+:generateContent"))
        .respond_with(
            ResponseTemplate::new(400).set_body_string(r#"{"error":{"message":"Invalid API key"}}"#),
        )
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let ctx = test_context().await;
    let cap = llm_capability(Some(json!({"mode": "text"})));

    let input = json!({"text": "hello"});
    let result = StepExecutor::execute(&cap, input, &config, &ctx).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        AppError::ExternalService(msg) => {
            assert!(msg.contains("400"), "error should contain status code, got: {msg}");
            assert!(
                msg.contains("Invalid API key"),
                "error should contain response body, got: {msg}"
            );
        }
        other => panic!("expected ExternalService error, got: {other:?}"),
    }
}

#[tokio::test]
async fn test_default_model_is_gemini_pro() {
    let mock_server = MockServer::start().await;

    // Specifically match the default model in the URL path.
    Mock::given(method("POST"))
        .and(path_regex(r"/v1beta/models/gemini-3\.1-pro:generateContent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(gemini_text_response("ok")))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = test_config(&mock_server.uri());
    let ctx = test_context().await;
    // No "model" field in runtime_config — should default to gemini-3.1-pro.
    let cap = llm_capability(Some(json!({
        "mode": "text",
        "system_prompt": "Be helpful"
    })));

    let input = json!({"text": "test"});
    let result = StepExecutor::execute(&cap, input, &config, &ctx).await.unwrap();

    assert_eq!(result["model"].as_str().unwrap(), "gemini-3.1-pro");
    // The mock expectation (expect(1)) will verify the correct URL was hit.
}
