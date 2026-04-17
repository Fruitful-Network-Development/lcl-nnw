use crate::router::RouteDecision;

pub trait Adapter: Send + Sync {
    fn chat_completion(&self, route: &RouteDecision, prompt: &str) -> String;
    fn embedding(&self, route: &RouteDecision, input: &str) -> Vec<f32>;
}

#[derive(Default)]
pub struct StubAdapter;

impl Adapter for StubAdapter {
    fn chat_completion(&self, route: &RouteDecision, prompt: &str) -> String {
        format!(
            "stubbed response via {} at {} ({} chars)",
            route.backend,
            route.endpoint,
            prompt.chars().count()
        )
    }

    fn embedding(&self, route: &RouteDecision, input: &str) -> Vec<f32> {
        let seed = (input.len() as f32).max(1.0);
        vec![
            seed,
            (route.profile.len() as f32) / 10.0,
            (route.backend.len() as f32) / 10.0,
        ]
    }
}

pub struct AdapterDispatcher {
    default_adapter: Box<dyn Adapter>,
}

impl Default for AdapterDispatcher {
    fn default() -> Self {
        Self {
            default_adapter: Box::<StubAdapter>::default(),
        }
    }
}

impl AdapterDispatcher {
    pub fn chat_completion(&self, route: &RouteDecision, prompt: &str) -> String {
        self.default_adapter.chat_completion(route, prompt)
    }

    pub fn embedding(&self, route: &RouteDecision, input: &str) -> Vec<f32> {
        self.default_adapter.embedding(route, input)
use crate::{EmbeddingRequest, GenerationRequest};

#[derive(Debug, Clone)]
pub struct AdapterResponse {
    pub backend: String,
    pub endpoint: String,
    pub accepted: bool,
}

pub trait BackendAdapter {
    fn name(&self) -> &'static str;
    fn generate(&self, request: &GenerationRequest) -> AdapterResponse;
    fn embed(&self, request: &EmbeddingRequest) -> AdapterResponse;
}

#[derive(Debug, Clone)]
pub struct LlamaCppAdapter {
    endpoint: String,
}

impl LlamaCppAdapter {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

impl BackendAdapter for LlamaCppAdapter {
    fn name(&self) -> &'static str {
        "llama_cpp"
    }

    fn generate(&self, _request: &GenerationRequest) -> AdapterResponse {
        AdapterResponse {
            backend: self.name().to_string(),
            endpoint: self.endpoint.clone(),
            accepted: true,
        }
    }

    fn embed(&self, _request: &EmbeddingRequest) -> AdapterResponse {
        AdapterResponse {
            backend: self.name().to_string(),
            endpoint: self.endpoint.clone(),
            accepted: true,
        }
    }
}

#[derive(Debug, Default)]
pub struct AdapterRegistry;

impl AdapterRegistry {
    pub fn build(&self, backend: &str, endpoint: &str) -> Box<dyn BackendAdapter> {
        match backend {
            "llama_cpp" => Box::new(LlamaCppAdapter::new(endpoint)),
            _ => Box::new(LlamaCppAdapter::new(endpoint)),
        }
    }
}
