use crate::policy::Policy;
use crate::registry::Registry;
use crate::session::Session;

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub profile: String,
    pub model_alias: String,
    pub temperature: f32,
    pub max_context_tokens: usize,
    pub fallback_model_alias: String,
    pub embedding_model_alias: Option<String>,
    pub backend: String,
    pub endpoint: String,
}

pub fn select_route(
    registry: &Registry,
    policy: &Policy,
    session: &Session,
    prompt: &str,
    requested_profile: &str,
) -> RouteDecision {
    let _prompt_size = prompt.len();
    let profile = policy.resolve_profile(requested_profile, session);
    let profile_config = registry.profile_config(&profile);
    let model_alias = profile_config
        .as_ref()
        .map(|config| config.model_alias.clone())
        .unwrap_or_else(|| "lead".to_string());
    let temperature = profile_config
        .as_ref()
        .map(|config| config.temperature)
        .unwrap_or(0.4);
    let max_context_tokens = profile_config
        .as_ref()
        .map(|config| config.max_context_tokens)
        .unwrap_or(8192);
    let fallback_model_alias = profile_config
        .as_ref()
        .map(|config| config.fallback_model_alias.clone())
        .unwrap_or_else(|| "lead".to_string());
    let embedding_model_alias = profile_config.and_then(|config| config.embedding_model_alias);

    RouteDecision {
        profile,
        model_alias,
        temperature,
        max_context_tokens,
        fallback_model_alias,
        embedding_model_alias,
        backend: policy.default_backend.clone(),
        endpoint: policy.default_endpoint.clone(),
    }
}

pub fn adapter_hook_description() -> &'static str {
    "Implement backend adapters under gateway/src/adapters/ and pass RouteDecision profile constraints to generation/embedding requests."
}
