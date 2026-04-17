use crate::policy::Policy;
use crate::registry::{ModelManifest, Registry};
use crate::session::Session;

#[derive(Debug, Clone)]
pub struct ModelReference {
    pub alias: String,
    pub name: String,
    pub backend: String,
}

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub profile: String,
    pub adapter_key: String,
    pub model: ModelReference,
    pub model_alias: String,
    pub temperature: f32,
    pub max_context_tokens: usize,
    pub fallback_model_alias: String,
    pub embedding_model_alias: Option<String>,
    pub backend: String,
    pub endpoint: String,
    pub quantization: String,
    pub rationale: String,
    pub embedding_backend: Option<String>,
    pub embedding_endpoint: Option<String>,
    pub session_id: String,
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

    let profile_cfg = registry.profile_config(&profile);
    let primary_alias = profile_cfg
        .map(|cfg| cfg.model_alias.as_str())
        .unwrap_or("lead");
    let fallback_alias = profile_cfg.and_then(|cfg| cfg.fallback_model_alias.as_deref());

    let selected_alias = if is_model_healthy(registry, primary_alias) {
        primary_alias
    } else if let Some(fallback_alias) = fallback_alias {
        if is_model_healthy(registry, fallback_alias) {
            fallback_alias
        } else {
            "lead"
        }
    } else {
        "lead"
    };

    let model_cfg = registry.model_config(selected_alias);
    let backend = model_cfg
        .map(|cfg| cfg.backend.clone())
        .unwrap_or_else(|| policy.default_backend.clone());
    let endpoint = model_cfg
        .and_then(|cfg| cfg.endpoint_override.clone())
        .unwrap_or_else(|| policy.default_endpoint.clone());

    RouteDecision {
        profile,
        model_alias: selected_alias.to_string(),
        backend,
        endpoint,
    let evaluation = policy.evaluate_model_for_profile(registry, &profile);

    RouteDecision {
        profile,
        model_alias: evaluation.model_alias,
        backend: evaluation.backend,
        endpoint: policy.default_endpoint.clone(),
        quantization: evaluation.quantization,
        rationale: evaluation.rationale,
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
    let embedding_backend = embedding_model_alias
        .as_deref()
        .and_then(|alias| registry.backend_for_alias(alias))
        .map(ToString::to_string);
    let embedding_endpoint = embedding_model_alias
        .as_ref()
        .map(|_| policy.default_endpoint.clone());

    let selected_model = registry
        .model_by_alias(&model_alias)
        .cloned()
        .or_else(|| registry.model_by_alias("lead").cloned())
        .unwrap_or_else(|| ModelManifest {
            alias: "lead".to_string(),
            name: "lead".to_string(),
            backend: policy.default_backend.clone(),
        });

    RouteDecision {
        profile,
        adapter_key: selected_model.backend.clone(),
        model: ModelReference {
            alias: selected_model.alias,
            name: selected_model.name,
            backend: selected_model.backend,
        },
        model_alias,
        temperature,
        max_context_tokens,
        fallback_model_alias,
        embedding_model_alias,
        backend,
        endpoint: policy.default_endpoint.clone(),
        embedding_backend,
        embedding_endpoint,
        session_id: session.id.clone(),
    }
}

fn is_model_healthy(registry: &Registry, alias: &str) -> bool {
    registry
        .model_config(alias)
        .map(|cfg| cfg.enabled)
        .unwrap_or(false)
}

pub fn adapter_hook_description() -> &'static str {
    "Backend adapters are selected from model backend keys using gateway/src/adapters/."
    "Adapters are selected via backend registry and route decisions carry profile, backend, and session constraints."
}
