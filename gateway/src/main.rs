mod adapters;
mod policy;
mod registry;
mod router;
mod session;

use std::{net::SocketAddr, path::Path, sync::Arc};

use adapters::AdapterDispatcher;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use policy::Policy;
use registry::Registry;
use router::select_route;
use serde::{Deserialize, Serialize};
use session::Session;

#[derive(Clone)]
struct AppState {
    registry: Arc<Registry>,
    policy: Arc<Policy>,
    adapters: Arc<AdapterDispatcher>,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let repo_root = std::env::var("HOME_LLM_ROOT").unwrap_or_else(|_| "..".to_string());
    let registry_root = Path::new(&repo_root).join("model_registry");

    let state = AppState {
        registry: Arc::new(Registry::load_from_dir(&registry_root)?),
        policy: Arc::new(Policy::default()),
        adapters: Arc::new(AdapterDispatcher::default()),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/models", get(models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/embeddings", post(embeddings))
        .with_state(state);

    let bind_addr: SocketAddr = std::env::var("GATEWAY_BIND")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(([127, 0, 0, 1], 4000).into());

    println!("gateway http listening on http://{bind_addr}");
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health() -> impl IntoResponse {
    Json(HealthResponse { status: "ok" })
}

#[derive(Serialize)]
struct ModelsResponse {
    object: &'static str,
    data: Vec<ModelEntry>,
}

#[derive(Serialize)]
struct ModelEntry {
    id: String,
    profile: String,
}

async fn models(State(state): State<AppState>) -> impl IntoResponse {
    let data = state
        .registry
        .models()
        .into_iter()
        .map(|(profile, model_alias)| ModelEntry {
            id: model_alias,
            profile,
        })
        .collect();

    Json(ModelsResponse {
        object: "list",
        data,
    })
}

#[derive(Deserialize)]
struct ChatCompletionRequest {
    model: Option<String>,
    profile: Option<String>,
    session_id: Option<String>,
    messages: Option<Vec<ChatMessage>>,
    input: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct RouteMetadata {
    profile: String,
    backend: String,
    endpoint: String,
}

#[derive(Serialize)]
struct ChatCompletionResponse {
    id: String,
    object: &'static str,
    model: String,
    profile: String,
    session_id: String,
    route: RouteMetadata,
    choices: Vec<ChatChoice>,
}

#[derive(Serialize)]
struct ChatChoice {
    index: usize,
    message: ChatMessage,
    finish_reason: &'static str,
}

async fn chat_completions(
    State(state): State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, (StatusCode, String)> {
    let session_id = payload
        .session_id
        .unwrap_or_else(|| "default-session".to_string());
    let session = Session::new(session_id.clone());

    let profile = payload
        .profile
        .or(payload.model)
        .unwrap_or_else(|| "chat".to_string());

    let prompt = payload
        .input
        .or_else(|| {
            payload.messages.as_ref().map(|messages| {
                messages
                    .iter()
                    .map(|m| m.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        })
        .ok_or((
            StatusCode::BAD_REQUEST,
            "missing input or messages".to_string(),
        ))?;

    let route = select_route(&state.registry, &state.policy, &session, &prompt, &profile);
    let completion_text = state.adapters.chat_completion(&route, &prompt);

    Ok(Json(ChatCompletionResponse {
        id: format!("chatcmpl-{}", session.id),
        object: "chat.completion",
        model: route.model_alias.clone(),
        profile: route.profile.clone(),
        session_id,
        route: RouteMetadata {
            profile: route.profile,
            backend: route.backend,
            endpoint: route.endpoint,
        },
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: completion_text,
            },
            finish_reason: "stop",
        }],
    }))
}

#[derive(Deserialize)]
#[serde(untagged)]
enum EmbeddingInput {
    Single(String),
    Many(Vec<String>),
}

#[derive(Deserialize)]
struct EmbeddingsRequest {
    model: Option<String>,
    profile: Option<String>,
    session_id: Option<String>,
    input: EmbeddingInput,
}

#[derive(Serialize)]
struct EmbeddingsResponse {
    object: &'static str,
    data: Vec<EmbeddingData>,
    model: String,
    profile: String,
    session_id: String,
    route: RouteMetadata,
}

#[derive(Serialize)]
struct EmbeddingData {
    object: &'static str,
    index: usize,
    embedding: Vec<f32>,
}

async fn embeddings(
    State(state): State<AppState>,
    Json(payload): Json<EmbeddingsRequest>,
) -> Result<Json<EmbeddingsResponse>, (StatusCode, String)> {
    let session_id = payload
        .session_id
        .unwrap_or_else(|| "default-session".to_string());
    let session = Session::new(session_id.clone());

    let profile = payload
        .profile
        .or(payload.model)
        .unwrap_or_else(|| "rag".to_string());

    let inputs = match payload.input {
        EmbeddingInput::Single(value) => vec![value],
        EmbeddingInput::Many(values) if !values.is_empty() => values,
        EmbeddingInput::Many(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "input array must not be empty".to_string(),
            ));
        }
    };

    let route = select_route(
        &state.registry,
        &state.policy,
        &session,
        inputs.first().map(String::as_str).unwrap_or_default(),
        &profile,
    );

    let data = inputs
        .iter()
        .enumerate()
        .map(|(index, input)| EmbeddingData {
            object: "embedding",
            index,
            embedding: state.adapters.embedding(&route, input),
        })
        .collect::<Vec<_>>();

    Ok(Json(EmbeddingsResponse {
        object: "list",
        data,
        model: route.model_alias.clone(),
        profile: route.profile.clone(),
        session_id,
        route: RouteMetadata {
            profile: route.profile,
            backend: route.backend,
            endpoint: route.endpoint,
        },
    }))
}
