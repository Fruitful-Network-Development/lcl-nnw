use std::{fs, path::Path, time::Duration};

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use gateway::{
    adapters::dispatch_chat,
    config::AppConfig,
    load_state_from_root,
    lanes::{LaneConfig, LaneRegistry},
    types::{ChatCompletionResponse, ChatMessage, HealthResponse, ListLanesResponse},
};
use reqwest::Client;
use serde_json::json;
use tempfile::tempdir;
use tokio::{task::JoinHandle, time::sleep};

#[tokio::test]
async fn loads_and_validates_lane_manifests() {
    let temp = tempdir().unwrap();
    write_registry(temp.path(), "http://127.0.0.1:9000", true, "http://127.0.0.1:8080", true);

    let config = AppConfig::load_from_root(temp.path()).unwrap();
    let registry = LaneRegistry::load_from_root(temp.path(), config.default_lane.clone()).unwrap();

    assert_eq!(config.default_lane, "remote_frontier");
    assert_eq!(registry.enabled_lanes().len(), 2);
    assert_eq!(
        registry
            .get("local_cpu16")
            .unwrap()
            .local_weight_path
            .as_deref(),
        Some("data/models/qwen2.5-coder-7b-instruct/Qwen2.5-Coder-7B-Instruct.Q4_K_M.gguf")
    );
}

#[tokio::test]
async fn rejects_invalid_local_lane_manifest() {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("model_registry/lanes")).unwrap();
    fs::write(
        temp.path().join("model_registry/gateway.toml"),
        "default_lane = \"local_cpu16\"\n",
    )
    .unwrap();
    fs::write(
        temp.path().join("model_registry/lanes/local_cpu16.toml"),
        r#"
name = "local_cpu16"
backend = "local_llama_cpp"
endpoint = "http://127.0.0.1:8080"
model_id = "broken"
enabled = true
temperature = 0.2
max_context_tokens = 8192
"#,
    )
    .unwrap();

    let error = LaneRegistry::load_from_root(temp.path(), "local_cpu16".to_string()).unwrap_err();
    assert!(error.to_string().contains("requires local_weight_path"));
}

#[tokio::test]
async fn local_adapter_translates_llama_cpp_response() {
    let upstream = spawn_local_server(StatusCode::OK, "local adapter ok", Duration::from_millis(0)).await;
    let client = Client::builder().timeout(Duration::from_secs(1)).build().unwrap();
    let lane = LaneConfig {
        name: "local_cpu16".to_string(),
        backend: "local_llama_cpp".to_string(),
        endpoint: upstream.base_url.clone(),
        model_id: "qwen".to_string(),
        enabled: true,
        temperature: 0.2,
        max_context_tokens: 8192,
        fallback_lane: None,
        local_weight_path: Some("data/models/qwen.gguf".to_string()),
    };

    let response = dispatch_chat(
        &client,
        &lane,
        &[ChatMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
        }],
        Some("sess-local"),
        Some("coding"),
    )
    .await
    .unwrap();

    assert_eq!(response.content, "local adapter ok");
}

#[tokio::test]
async fn remote_adapter_forwards_reduced_request_shape() {
    let upstream =
        spawn_remote_server(StatusCode::OK, "remote adapter ok", Duration::from_millis(0)).await;
    let client = Client::builder().timeout(Duration::from_secs(1)).build().unwrap();
    let lane = LaneConfig {
        name: "remote_frontier".to_string(),
        backend: "remote_custom_llama".to_string(),
        endpoint: upstream.base_url.clone(),
        model_id: "llama4-frontier".to_string(),
        enabled: true,
        temperature: 0.3,
        max_context_tokens: 32768,
        fallback_lane: Some("local_cpu16".to_string()),
        local_weight_path: None,
    };

    let response = dispatch_chat(
        &client,
        &lane,
        &[ChatMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
        }],
        Some("sess-remote"),
        Some("research"),
    )
    .await
    .unwrap();

    assert_eq!(response.content, "remote adapter ok");
}

#[tokio::test]
async fn smoke_health_and_lanes_endpoints_work() {
    let remote = spawn_remote_server(StatusCode::OK, "remote ok", Duration::from_millis(0)).await;
    let local = spawn_local_server(StatusCode::OK, "local ok", Duration::from_millis(0)).await;
    let gateway = spawn_gateway(remote.base_url.as_str(), true, local.base_url.as_str(), true, Duration::from_secs(1)).await;
    let client = Client::new();

    let health = client
        .get(format!("{}/health", gateway.base_url))
        .send()
        .await
        .unwrap()
        .json::<HealthResponse>()
        .await
        .unwrap();
    assert_eq!(health.status, "ok");

    let lanes = client
        .get(format!("{}/lanes", gateway.base_url))
        .send()
        .await
        .unwrap()
        .json::<ListLanesResponse>()
        .await
        .unwrap();
    assert_eq!(lanes.default_lane, "remote_frontier");
    assert_eq!(lanes.data.len(), 2);
}

#[tokio::test]
async fn explicit_lane_selection_uses_local_lane() {
    let remote = spawn_remote_server(StatusCode::OK, "remote ok", Duration::from_millis(0)).await;
    let local = spawn_local_server(StatusCode::OK, "local ok", Duration::from_millis(0)).await;
    let gateway = spawn_gateway(remote.base_url.as_str(), true, local.base_url.as_str(), true, Duration::from_secs(1)).await;
    let client = Client::new();

    let response = client
        .post(format!("{}/v1/chat/completions", gateway.base_url))
        .json(&json!({
            "lane": "local_cpu16",
            "messages": [{ "role": "user", "content": "write code" }]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let payload = response.json::<ChatCompletionResponse>().await.unwrap();
    assert_eq!(payload.lane, "local_cpu16");
    assert_eq!(payload.choices[0].message.content, "local ok");
    assert!(!payload.route.fallback_used);
}

#[tokio::test]
async fn default_lane_selection_uses_remote_frontier() {
    let remote = spawn_remote_server(StatusCode::OK, "remote ok", Duration::from_millis(0)).await;
    let local = spawn_local_server(StatusCode::OK, "local ok", Duration::from_millis(0)).await;
    let gateway = spawn_gateway(remote.base_url.as_str(), true, local.base_url.as_str(), true, Duration::from_secs(1)).await;
    let client = Client::new();

    let payload = client
        .post(format!("{}/v1/chat/completions", gateway.base_url))
        .json(&json!({
            "messages": [{ "role": "user", "content": "hello" }]
        }))
        .send()
        .await
        .unwrap()
        .json::<ChatCompletionResponse>()
        .await
        .unwrap();

    assert_eq!(payload.lane, "remote_frontier");
    assert_eq!(payload.choices[0].message.content, "remote ok");
}

#[tokio::test]
async fn rejects_unknown_lane_requests() {
    let remote = spawn_remote_server(StatusCode::OK, "remote ok", Duration::from_millis(0)).await;
    let local = spawn_local_server(StatusCode::OK, "local ok", Duration::from_millis(0)).await;
    let gateway = spawn_gateway(remote.base_url.as_str(), true, local.base_url.as_str(), true, Duration::from_secs(1)).await;
    let client = Client::new();

    let response = client
        .post(format!("{}/v1/chat/completions", gateway.base_url))
        .json(&json!({
            "lane": "missing_lane",
            "messages": [{ "role": "user", "content": "hello" }]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn rejects_disabled_explicit_lane_requests() {
    let remote = spawn_remote_server(StatusCode::OK, "remote ok", Duration::from_millis(0)).await;
    let local = spawn_local_server(StatusCode::OK, "local ok", Duration::from_millis(0)).await;
    let gateway = spawn_gateway(remote.base_url.as_str(), true, local.base_url.as_str(), false, Duration::from_secs(1)).await;
    let client = Client::new();

    let response = client
        .post(format!("{}/v1/chat/completions", gateway.base_url))
        .json(&json!({
            "lane": "local_cpu16",
            "messages": [{ "role": "user", "content": "hello" }]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn falls_back_to_local_on_remote_5xx() {
    let remote = spawn_remote_server(
        StatusCode::INTERNAL_SERVER_ERROR,
        "remote failure",
        Duration::from_millis(0),
    )
    .await;
    let local = spawn_local_server(StatusCode::OK, "local fallback", Duration::from_millis(0)).await;
    let gateway = spawn_gateway(remote.base_url.as_str(), true, local.base_url.as_str(), true, Duration::from_secs(1)).await;
    let client = Client::new();

    let payload = client
        .post(format!("{}/v1/chat/completions", gateway.base_url))
        .json(&json!({
            "messages": [{ "role": "user", "content": "hello" }]
        }))
        .send()
        .await
        .unwrap()
        .json::<ChatCompletionResponse>()
        .await
        .unwrap();

    assert_eq!(payload.lane, "local_cpu16");
    assert!(payload.route.fallback_used);
    assert_eq!(payload.choices[0].message.content, "local fallback");
}

#[tokio::test]
async fn falls_back_to_local_on_remote_timeout() {
    let remote =
        spawn_remote_server(StatusCode::OK, "remote too slow", Duration::from_millis(200)).await;
    let local = spawn_local_server(StatusCode::OK, "local timeout fallback", Duration::from_millis(0)).await;
    let gateway = spawn_gateway(remote.base_url.as_str(), true, local.base_url.as_str(), true, Duration::from_millis(50)).await;
    let client = Client::new();

    let payload = client
        .post(format!("{}/v1/chat/completions", gateway.base_url))
        .json(&json!({
            "messages": [{ "role": "user", "content": "hello" }]
        }))
        .send()
        .await
        .unwrap()
        .json::<ChatCompletionResponse>()
        .await
        .unwrap();

    assert_eq!(payload.lane, "local_cpu16");
    assert!(payload.route.fallback_used);
    assert_eq!(payload.choices[0].message.content, "local timeout fallback");
}

#[tokio::test]
async fn smoke_remote_and_local_generation_paths_work() {
    let remote = spawn_remote_server(StatusCode::OK, "remote smoke", Duration::from_millis(0)).await;
    let local = spawn_local_server(StatusCode::OK, "local smoke", Duration::from_millis(0)).await;
    let gateway = spawn_gateway(remote.base_url.as_str(), true, local.base_url.as_str(), true, Duration::from_secs(1)).await;
    let client = Client::new();

    let remote_payload = client
        .post(format!("{}/v1/chat/completions", gateway.base_url))
        .json(&json!({
            "messages": [{ "role": "user", "content": "remote" }]
        }))
        .send()
        .await
        .unwrap()
        .json::<ChatCompletionResponse>()
        .await
        .unwrap();
    assert_eq!(remote_payload.choices[0].message.content, "remote smoke");

    let local_payload = client
        .post(format!("{}/v1/chat/completions", gateway.base_url))
        .json(&json!({
            "lane": "local_cpu16",
            "messages": [{ "role": "user", "content": "local" }]
        }))
        .send()
        .await
        .unwrap()
        .json::<ChatCompletionResponse>()
        .await
        .unwrap();
    assert_eq!(local_payload.choices[0].message.content, "local smoke");
}

struct SpawnedServer {
    base_url: String,
    handle: JoinHandle<()>,
}

impl Drop for SpawnedServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

async fn spawn_gateway(
    remote_endpoint: &str,
    remote_enabled: bool,
    local_endpoint: &str,
    local_enabled: bool,
    timeout: Duration,
) -> SpawnedServer {
    let temp = tempdir().unwrap();
    write_registry(
        temp.path(),
        remote_endpoint,
        remote_enabled,
        local_endpoint,
        local_enabled,
    );
    let client = Client::builder().timeout(timeout).build().unwrap();
    let state = load_state_from_root(temp.path(), client).unwrap();
    spawn_router(gateway::http::build_app(state)).await
}

async fn spawn_remote_server(
    status: StatusCode,
    content: &str,
    delay: Duration,
) -> SpawnedServer {
    let content = content.to_string();
    let router = Router::new()
        .route("/health", get(|| async { Json(json!({ "status": "ok" })) }))
        .route(
            "/v1/chat/completions",
            post(move || {
                let content = content.clone();
                async move {
                    if !delay.is_zero() {
                        sleep(delay).await;
                    }
                    (
                        status,
                        Json(json!({
                            "content": content,
                            "finish_reason": "stop"
                        })),
                    )
                }
            }),
        );

    spawn_router(router).await
}

async fn spawn_local_server(
    status: StatusCode,
    content: &str,
    delay: Duration,
) -> SpawnedServer {
    let content = content.to_string();
    let router = Router::new()
        .route("/health", get(|| async { Json(json!({ "status": "ok" })) }))
        .route(
            "/v1/chat/completions",
            post(move || {
                let content = content.clone();
                async move {
                    if !delay.is_zero() {
                        sleep(delay).await;
                    }
                    (
                        status,
                        Json(json!({
                            "choices": [{
                                "index": 0,
                                "message": {
                                    "role": "assistant",
                                    "content": content
                                },
                                "finish_reason": "stop"
                            }]
                        })),
                    )
                }
            }),
        );

    spawn_router(router).await
}

async fn spawn_router(app: Router) -> SpawnedServer {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    SpawnedServer {
        base_url: format!("http://{}", addr),
        handle,
    }
}

fn write_registry(
    root: &Path,
    remote_endpoint: &str,
    remote_enabled: bool,
    local_endpoint: &str,
    local_enabled: bool,
) {
    fs::create_dir_all(root.join("model_registry/lanes")).unwrap();

    fs::write(
        root.join("model_registry/gateway.toml"),
        r#"
default_lane = "remote_frontier"
bind_address = "127.0.0.1:8787"
"#,
    )
    .unwrap();

    fs::write(
        root.join("model_registry/lanes/remote_frontier.toml"),
        format!(
            r#"
name = "remote_frontier"
backend = "remote_custom_llama"
endpoint = "{remote_endpoint}"
model_id = "llama4-frontier"
enabled = {remote_enabled}
temperature = 0.2
max_context_tokens = 32768
fallback_lane = "local_cpu16"
"#
        ),
    )
    .unwrap();

    fs::write(
        root.join("model_registry/lanes/local_cpu16.toml"),
        format!(
            r#"
name = "local_cpu16"
backend = "local_llama_cpp"
endpoint = "{local_endpoint}"
model_id = "qwen2.5-coder-7b-instruct-q4_k_m"
enabled = {local_enabled}
temperature = 0.2
max_context_tokens = 8192
local_weight_path = "data/models/qwen2.5-coder-7b-instruct/Qwen2.5-Coder-7B-Instruct.Q4_K_M.gguf"
"#
        ),
    )
    .unwrap();
}
