mod adapters;
mod http_server;
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
use adapters::AdapterRegistry;
use std::path::Path;

#[derive(Debug)]
pub struct GenerationRequest {
    pub profile: String,
    pub model_alias: String,
    pub fallback_model_alias: String,
    pub temperature: f32,
    pub max_context_tokens: usize,
}

#[derive(Debug)]
pub struct EmbeddingRequest {
    pub profile: String,
    pub model_alias: String,
    pub max_context_tokens: usize,
}

fn build_generation_request(route: &router::RouteDecision) -> GenerationRequest {
    GenerationRequest {
        profile: route.profile.clone(),
        model_alias: route.model_alias.clone(),
        fallback_model_alias: route.fallback_model_alias.clone(),
        temperature: route.temperature,
        max_context_tokens: route.max_context_tokens,
    }
}

fn build_embedding_request(route: &router::RouteDecision) -> Option<EmbeddingRequest> {
    route
        .embedding_model_alias
        .as_ref()
        .map(|embedding_model_alias| EmbeddingRequest {
            profile: route.profile.clone(),
            model_alias: embedding_model_alias.clone(),
            max_context_tokens: route.max_context_tokens,
        })
}

fn is_embedding_required(profile: &str) -> bool {
    profile == "rag"
}

fn validate_embedding_alias(
    route: &router::RouteDecision,
    registry: &registry::Registry,
) -> Result<(), String> {
    let Some(alias) = route.embedding_model_alias.as_deref() else {
        return Ok(());
    };

    if !registry.has_model_alias(alias) {
        return Err(format!(
            "embedding model alias `{alias}` does not exist in registry"
        ));
    }
    if registry.backend_for_alias(alias).is_none() {
        return Err(format!(
            "embedding model alias `{alias}` is disabled or has no resolvable backend"
        ));
    }
    if route.embedding_backend.is_none() {
        return Err(format!(
            "embedding model alias `{alias}` does not resolve to a backend in route decision"
        ));
    }
    if route.embedding_endpoint.is_none() {
        return Err(format!(
            "embedding model alias `{alias}` does not resolve to an endpoint in route decision"
        ));
    }

    Ok(())
}

fn main() -> std::io::Result<()> {
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
    println!("gateway status: ok");
    println!("registry root: {}", registry_root.display());
    if let Some(model_name) = registry.model_name_for_alias(&route.model_alias) {
        println!("selected model name: {model_name}");
    }
    let generation_request = build_generation_request(&route);
    let embedding_request = match validate_embedding_alias(&route, &registry) {
        Ok(()) => build_embedding_request(&route),
        Err(error) if is_embedding_required(&route.profile) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "profile `{}` requires a valid embedding model alias: {error}",
                    route.profile
                ),
            ));
        }
        Err(error) => {
            eprintln!(
                "warning: dropping embedding request for profile `{}` because embedding alias is invalid: {error}",
                route.profile
            );
            None
        }
    };
    println!(
        "selected route -> session={} profile={} model={} fallback={} temp={} max_ctx={} embed_model={:?} backend={} endpoint={} embed_backend={:?} embed_endpoint={:?}",
        route.session_id,
        route.profile,
        route.model_alias,
        route.fallback_model_alias,
        route.temperature,
        route.max_context_tokens,
        route.embedding_model_alias,
        route.backend,
        route.endpoint,
        route.embedding_backend,
        route.embedding_endpoint
    );

    let adapter_registry = AdapterRegistry;
    let generation_adapter = adapter_registry.build(&route.backend, &route.endpoint);
    let generation_response = generation_adapter.generate(&generation_request);
    println!("generation adapter payload: {:?}", generation_request);
    println!(
        "selected route -> profile={} model={} backend={} endpoint={} quantization={} rationale={}\n",
        route.profile,
        route.model_alias,
        route.backend,
        route.endpoint,
        route.quantization,
        route.rationale
        "generation adapter response: backend={} endpoint={} accepted={}",
        generation_response.backend, generation_response.endpoint, generation_response.accepted
    );

    if let Some(embed_req) = embedding_request {
        let embedding_backend = route
            .embedding_backend
            .as_deref()
            .unwrap_or(route.backend.as_str());
        let embedding_endpoint = route
            .embedding_endpoint
            .as_deref()
            .unwrap_or(route.endpoint.as_str());
        let embedding_adapter = adapter_registry.build(embedding_backend, embedding_endpoint);
        let embed_response = embedding_adapter.embed(&embed_req);
        println!("embedding adapter payload: {:?}", embed_req);
        println!(
            "embedding adapter response: backend={} endpoint={} accepted={}",
            embed_response.backend, embed_response.endpoint, embed_response.accepted
        );
    }

    println!("adapter hook: {}", router::adapter_hook_description());

    if std::env::var("GATEWAY_HTTP_ONCE")
        .ok()
        .as_deref()
        .is_some_and(|value| value == "1")
    {
        let addr =
            std::env::var("GATEWAY_HTTP_ADDR").unwrap_or_else(|_| "127.0.0.1:9090".to_string());
        println!("http gateway listening for one request on {addr}");
        http_server::serve_once(&addr, &route)?;
    }

    Ok(())
}
