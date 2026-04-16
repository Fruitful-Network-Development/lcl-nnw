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
    pub session_id: String,
}

pub fn select_route(
    registry: &Registry,
    policy: &Policy,
    session: &Session,
    prompt: &str,
    requested_profile: &str,
) -> RouteDecision {
    let profile = policy.resolve_profile(requested_profile, session, prompt);
    let profile_config = registry.profile_config(&profile);
    let temperature = profile_config
        .as_ref()
        .map(|config| config.temperature)
        .unwrap_or(0.4);
    let max_context_tokens = profile_config
        .as_ref()
        .map(|config| config.max_context_tokens)
        .unwrap_or(8192);

    let Some(default_alias) = registry.preferred_routable_alias().map(ToString::to_string) else {
        return RouteDecision {
            profile,
            model_alias: "unavailable".to_string(),
            temperature,
            max_context_tokens,
            fallback_model_alias: "unavailable".to_string(),
            embedding_model_alias: None,
            backend: "unavailable".to_string(),
            endpoint: "unavailable://no-routable-model".to_string(),
            session_id: session.id.clone(),
        };
    };

    let model_alias = profile_config
        .as_ref()
        .map(|config| config.model_alias.clone())
        .filter(|alias| registry.is_model_routable(alias))
        .unwrap_or_else(|| default_alias.clone());
    let fallback_model_alias = profile_config
        .as_ref()
        .map(|config| config.fallback_model_alias.clone())
        .filter(|alias| registry.is_model_routable(alias))
        .unwrap_or_else(|| default_alias.clone());
    let embedding_model_alias = profile_config
        .and_then(|config| config.embedding_model_alias)
        .filter(|alias| registry.is_model_routable(alias));

    let backend = registry
        .backend_for_alias(&model_alias)
        .unwrap_or(policy.default_backend.as_str())
        .to_string();

    RouteDecision {
        profile,
        model_alias,
        temperature,
        max_context_tokens,
        fallback_model_alias,
        embedding_model_alias,
        backend,
        endpoint: policy.default_endpoint.clone(),
        session_id: session.id.clone(),
    }
}

pub fn adapter_hook_description() -> &'static str {
    "Adapters are selected via backend registry and route decisions carry profile, backend, and session constraints."
}
