use std::{collections::HashMap, env, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::config::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LaneConfig {
    pub name: String,
    pub backend: String,
    pub endpoint: String,
    pub model_id: String,
    pub enabled: bool,
    pub temperature: f32,
    pub max_context_tokens: usize,
    #[serde(default)]
    pub fallback_lane: Option<String>,
    #[serde(default)]
    pub local_weight_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    LocalLlamaCpp,
    RemoteCustomLlama,
}

impl BackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalLlamaCpp => "local_llama_cpp",
            Self::RemoteCustomLlama => "remote_custom_llama",
        }
    }
}

impl TryFrom<&str> for BackendKind {
    type Error = ConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "local_llama_cpp" => Ok(Self::LocalLlamaCpp),
            "remote_custom_llama" => Ok(Self::RemoteCustomLlama),
            other => Err(ConfigError::Validation(format!(
                "unsupported backend `{other}`"
            ))),
        }
    }
}

impl LaneConfig {
    pub fn backend_kind(&self) -> Result<BackendKind, ConfigError> {
        BackendKind::try_from(self.backend.as_str())
    }

    fn apply_env_overrides(&mut self) {
        match self.name.as_str() {
            "remote_frontier" => {
                if let Ok(value) = env::var("REMOTE_FRONTIER_ENDPOINT") {
                    self.endpoint = value;
                }
                if let Ok(value) = env::var("REMOTE_FRONTIER_MODEL_ID") {
                    self.model_id = value;
                }
                if let Some(value) = env_bool("REMOTE_FRONTIER_ENABLED") {
                    self.enabled = value;
                }
            }
            "local_cpu16" => {
                if let Ok(value) = env::var("LOCAL_CPU16_ENDPOINT") {
                    self.endpoint = value;
                }
                if let Ok(value) = env::var("LOCAL_CPU16_MODEL_ID") {
                    self.model_id = value;
                }
                if let Ok(value) = env::var("LOCAL_CPU16_MODEL_PATH") {
                    self.local_weight_path = Some(value);
                }
                if let Some(value) = env_bool("LOCAL_CPU16_ENABLED") {
                    self.enabled = value;
                }
            }
            _ => {}
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.name.trim().is_empty() {
            return Err(ConfigError::Validation("lane name must not be empty".to_string()));
        }
        if self.endpoint.trim().is_empty() {
            return Err(ConfigError::Validation(format!(
                "lane `{}` has an empty endpoint",
                self.name
            )));
        }
        if self.model_id.trim().is_empty() {
            return Err(ConfigError::Validation(format!(
                "lane `{}` has an empty model_id",
                self.name
            )));
        }

        match self.backend_kind()? {
            BackendKind::LocalLlamaCpp => {
                let Some(path) = self.local_weight_path.as_deref() else {
                    return Err(ConfigError::Validation(format!(
                        "lane `{}` requires local_weight_path",
                        self.name
                    )));
                };
                if path.trim().is_empty() {
                    return Err(ConfigError::Validation(format!(
                        "lane `{}` has an empty local_weight_path",
                        self.name
                    )));
                }
            }
            BackendKind::RemoteCustomLlama => {
                if self.local_weight_path.is_some() {
                    return Err(ConfigError::Validation(format!(
                        "lane `{}` must not set local_weight_path for remote backend",
                        self.name
                    )));
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LaneRegistry {
    default_lane: String,
    lanes: HashMap<String, LaneConfig>,
}

impl LaneRegistry {
    pub fn load_from_root(root: &Path, default_lane: String) -> Result<Self, ConfigError> {
        let lanes_dir = root.join("model_registry").join("lanes");
        if !lanes_dir.is_dir() {
            return Err(ConfigError::Validation(format!(
                "missing lane directory: {}",
                lanes_dir.display()
            )));
        }

        let mut lane_files = fs::read_dir(&lanes_dir)
            .map_err(|err| ConfigError::Io(err.to_string()))?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("toml"))
            .collect::<Vec<_>>();
        lane_files.sort();

        let mut lanes = HashMap::new();
        for path in lane_files {
            let contents =
                fs::read_to_string(&path).map_err(|err| ConfigError::Io(err.to_string()))?;
            let mut lane = toml::from_str::<LaneConfig>(&contents)
                .map_err(|err| ConfigError::Parse(err.to_string()))?;
            lane.apply_env_overrides();
            lane.validate()?;

            if lanes.insert(lane.name.clone(), lane).is_some() {
                return Err(ConfigError::Validation(format!(
                    "duplicate lane manifest found for `{}`",
                    path.display()
                )));
            }
        }

        if !lanes.contains_key(&default_lane) {
            return Err(ConfigError::Validation(format!(
                "default lane `{default_lane}` is not defined"
            )));
        }

        for lane in lanes.values() {
            if let Some(fallback_lane) = lane.fallback_lane.as_deref() {
                if fallback_lane == lane.name {
                    return Err(ConfigError::Validation(format!(
                        "lane `{}` cannot fall back to itself",
                        lane.name
                    )));
                }

                if !lanes.contains_key(fallback_lane) {
                    return Err(ConfigError::Validation(format!(
                        "lane `{}` references missing fallback lane `{fallback_lane}`",
                        lane.name
                    )));
                }
            }
        }

        Ok(Self {
            default_lane,
            lanes,
        })
    }

    pub fn default_lane_name(&self) -> &str {
        &self.default_lane
    }

    pub fn get(&self, lane_name: &str) -> Option<&LaneConfig> {
        self.lanes.get(lane_name)
    }

    pub fn enabled_lanes(&self) -> Vec<LaneConfig> {
        let mut lanes = self
            .lanes
            .values()
            .filter(|lane| lane.enabled)
            .cloned()
            .collect::<Vec<_>>();
        lanes.sort_by(|left, right| left.name.cmp(&right.name));
        lanes
    }
}

fn env_bool(key: &str) -> Option<bool> {
    match env::var(key).ok()?.to_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
