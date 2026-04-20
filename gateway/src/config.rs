use std::{
    env, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct GatewayDefaultsFile {
    #[serde(default = "default_lane")]
    default_lane: String,
    #[serde(default = "default_bind_address")]
    bind_address: String,
}

fn default_lane() -> String {
    "remote_frontier".to_string()
}

fn default_bind_address() -> String {
    "127.0.0.1:8787".to_string()
}

impl Default for GatewayDefaultsFile {
    fn default() -> Self {
        Self {
            default_lane: default_lane(),
            bind_address: default_bind_address(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub repo_root: PathBuf,
    pub default_lane: String,
    pub bind_address: String,
}

impl AppConfig {
    pub fn discover_root() -> PathBuf {
        if let Ok(root) = env::var("HOME_LLM_ROOT") {
            return PathBuf::from(root);
        }

        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if cwd.join("model_registry").is_dir() {
            return cwd;
        }

        if let Some(parent) = cwd.parent() {
            if parent.join("model_registry").is_dir() {
                return parent.to_path_buf();
            }
        }

        PathBuf::from("..")
    }

    pub fn load_from_root(root: &Path) -> Result<Self, ConfigError> {
        let defaults_path = root.join("model_registry").join("gateway.toml");
        let defaults = if defaults_path.exists() {
            let contents =
                fs::read_to_string(&defaults_path).map_err(|err| ConfigError::Io(err.to_string()))?;
            toml::from_str::<GatewayDefaultsFile>(&contents)
                .map_err(|err| ConfigError::Parse(err.to_string()))?
        } else {
            GatewayDefaultsFile::default()
        };

        let default_lane = env::var("DEFAULT_LANE").unwrap_or(defaults.default_lane);
        let bind_address = env::var("GATEWAY_BIND").unwrap_or(defaults.bind_address);

        Ok(Self {
            repo_root: root.to_path_buf(),
            default_lane,
            bind_address,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "io error: {message}"),
            Self::Parse(message) => write!(f, "parse error: {message}"),
            Self::Validation(message) => write!(f, "validation error: {message}"),
        }
    }
}

impl std::error::Error for ConfigError {}
