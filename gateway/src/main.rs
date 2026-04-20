use gateway::{default_http_client, http::build_app, load_state_from_root};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo_root = gateway::config::AppConfig::discover_root();
    let client = default_http_client()?;
    let state = load_state_from_root(&repo_root, client)?;
    let bind_address = state.config.bind_address.clone();
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    println!("gateway listening on http://{bind_address}");
    axum::serve(listener, build_app(state)).await?;
    Ok(())
}
