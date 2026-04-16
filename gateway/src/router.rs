use crate::policy::Policy;
use crate::registry::Registry;
use crate::session::Session;

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub profile: String,
    pub model_alias: String,
    pub backend: String,
    pub endpoint: String,
    pub quantization: String,
    pub rationale: String,
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
    let evaluation = policy.evaluate_model_for_profile(registry, &profile);

    RouteDecision {
        profile,
        model_alias: evaluation.model_alias,
        backend: evaluation.backend,
        endpoint: policy.default_endpoint.clone(),
        quantization: evaluation.quantization,
        rationale: evaluation.rationale,
    }
}

pub fn adapter_hook_description() -> &'static str {
    "Implement backend adapters under gateway/src/adapters/ and call them from select_route()."
}
