use crate::policy::Policy;
use crate::registry::Registry;
use crate::session::Session;

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub profile: String,
    pub model_alias: String,
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
    }
}

fn is_model_healthy(registry: &Registry, alias: &str) -> bool {
    registry
        .model_config(alias)
        .map(|cfg| cfg.enabled)
        .unwrap_or(false)
}

pub fn adapter_hook_description() -> &'static str {
    "Implement backend adapters under gateway/src/adapters/ and call them from select_route()."
}
