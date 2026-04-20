pub mod adapters;
pub mod config;
pub mod http;
pub mod lanes;
pub mod types;

use std::{path::Path, time::Duration};

use config::ConfigError;
use http::AppState;
use lanes::LaneRegistry;
use reqwest::Client;

pub fn default_http_client() -> Result<Client, reqwest::Error> {
    Client::builder().timeout(Duration::from_secs(5)).build()
}

pub fn load_state_from_root(root: &Path, client: Client) -> Result<AppState, ConfigError> {
    let config = config::AppConfig::load_from_root(root)?;
    let lanes = LaneRegistry::load_from_root(root, config.default_lane.clone())?;
    Ok(AppState::new(config, lanes, client))
}
