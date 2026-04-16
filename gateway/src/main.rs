mod adapters;
mod http_server;
mod policy;
mod registry;
mod router;
mod session;

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

fn main() -> std::io::Result<()> {
    let repo_root = std::env::var("HOME_LLM_ROOT").unwrap_or_else(|_| "..".to_string());
    let registry_root = Path::new(&repo_root).join("model_registry");

    let registry = registry::Registry::load_from_dir(&registry_root)?;
    let policy = policy::Policy::default();
    let session = session::Session::new("bootstrap-session");

    let route = router::select_route(&registry, &policy, &session, "health ping", "chat");

    println!("gateway status: ok");
    println!("registry root: {}", registry_root.display());
    if let Some(model_name) = registry.model_name_for_alias(&route.model_alias) {
        println!("selected model name: {model_name}");
    }
    let generation_request = build_generation_request(&route);
    let embedding_request = build_embedding_request(&route);
    println!(
        "selected route -> session={} profile={} model={} fallback={} temp={} max_ctx={} embed_model={:?} backend={} endpoint={}",
        route.session_id,
        route.profile,
        route.model_alias,
        route.fallback_model_alias,
        route.temperature,
        route.max_context_tokens,
        route.embedding_model_alias,
        route.backend,
        route.endpoint
    );

    let adapter_registry = AdapterRegistry;
    let adapter = adapter_registry.build(&route.backend, &route.endpoint);
    let generation_response = adapter.generate(&generation_request);
    println!("generation adapter payload: {:?}", generation_request);
    println!(
        "generation adapter response: backend={} endpoint={} accepted={}",
        generation_response.backend, generation_response.endpoint, generation_response.accepted
    );

    if let Some(embed_req) = embedding_request {
        let embed_response = adapter.embed(&embed_req);
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
