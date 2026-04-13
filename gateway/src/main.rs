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

    let route = router::select_route(&registry, &policy, &session, "health ping", "chat");

    println!("gateway status: ok");
    println!("registry root: {}", registry_root.display());
    println!(
        "selected route -> profile={} model={} backend={} endpoint={}",
        route.profile, route.model_alias, route.backend, route.endpoint
    );
    println!("adapter hook: {}", router::adapter_hook_description());

    Ok(())
}
