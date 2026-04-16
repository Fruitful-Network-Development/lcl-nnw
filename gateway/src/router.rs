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
    let model_alias = registry
        .profile_model(&profile)
        .unwrap_or_else(|| "lead".to_string());

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
    }
}

pub fn adapter_hook_description() -> &'static str {
    "Backend adapters are selected from model backend keys using gateway/src/adapters/."
}
