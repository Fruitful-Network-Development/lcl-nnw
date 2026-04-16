mod adapters;
mod policy;
mod registry;
mod router;
mod session;

use std::path::Path;

fn main() -> std::io::Result<()> {
    let repo_root = std::env::var("HOME_LLM_ROOT").unwrap_or_else(|_| "..".to_string());
    let registry_root = Path::new(&repo_root).join("model_registry");

    let registry = registry::Registry::load_from_dir(&registry_root)?;
    let policy = policy::Policy::default();
    let session = session::Session::new("bootstrap-session");
    let adapters = adapters::AdapterRegistry::default_with_builtin();

    let route = router::select_route(&registry, &policy, &session, "health ping", "chat");

    println!("gateway status: ok");
    println!("registry root: {}", registry_root.display());
    println!(
        "selected route -> profile={} model={} ({}) adapter_key={}",
        route.profile, route.model.alias, route.model.name, route.adapter_key
    );

    let adapter = adapters.get(&route.adapter_key).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("no adapter registered for backend '{}'", route.adapter_key),
        )
    })?;

    let health = adapter
        .health(&policy.default_endpoint)
        .map_err(std::io::Error::other)?;
    println!("adapter health: {health}");

    let inference = adapter
        .infer(&policy.default_endpoint, &route.model, "health ping")
        .map_err(std::io::Error::other)?;
    println!("adapter inference: {inference}");

    println!("adapter hook: {}", router::adapter_hook_description());

    Ok(())
}
