use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registry {
    profile_map: HashMap<String, ProfileConfig>,
    model_map: HashMap<String, ModelConfig>,
}

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub model_alias: String,
    pub fallback_model_alias: Option<String>,
    pub params: ProfileParams,
}

#[derive(Debug, Clone, Default)]
pub struct ProfileParams {
    pub max_context_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub backend: String,
    pub local_weight_path: String,
    pub enabled: bool,
    pub endpoint_override: Option<String>,
}

impl Registry {
    pub fn load_from_dir(root: &Path) -> io::Result<Self> {
        let profiles_dir = root.join("profiles");
        let models_dir = root.join("models");

        let mut profile_map = HashMap::new();
        let mut model_map = HashMap::new();

        if models_dir.is_dir() {
            for entry in fs::read_dir(&models_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }

                let contents = fs::read_to_string(&path)?;
                let alias = parse_toml_string(&contents, "alias").unwrap_or_else(|| {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("lead")
                        .to_string()
                });

                let backend =
                    parse_toml_string(&contents, "backend").unwrap_or_else(|| "llama_cpp".to_string());
                let local_weight_path =
                    parse_toml_string(&contents, "local_weight_path").unwrap_or_default();
                let enabled = parse_toml_bool(&contents, "enabled").unwrap_or(true);
                let endpoint_override = parse_toml_string(&contents, "endpoint_override")
                    .or_else(|| parse_toml_string(&contents, "endpoint"));

                model_map.insert(
                    alias,
                    ModelConfig {
                        backend,
                        local_weight_path,
                        enabled,
                        endpoint_override,
                    },
                );
            }
        }

        if profiles_dir.is_dir() {
            for entry in fs::read_dir(&profiles_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }

                let contents = fs::read_to_string(&path)?;
                let name = parse_toml_string(&contents, "name").unwrap_or_else(|| "chat".to_string());
                let model_alias =
                    parse_toml_string(&contents, "model_alias").unwrap_or_else(|| "lead".to_string());
                let fallback_model_alias = parse_toml_string(&contents, "fallback_model_alias");
                let params = ProfileParams {
                    max_context_tokens: parse_toml_u32(&contents, "max_context_tokens"),
                    temperature: parse_toml_f32(&contents, "temperature"),
                };

                profile_map.insert(
                    name,
                    ProfileConfig {
                        model_alias,
                        fallback_model_alias,
                        params,
                    },
                );
            }
        }

        Ok(Self {
            profile_map,
            model_map,
        })
    }

    pub fn profile_config(&self, profile_name: &str) -> Option<&ProfileConfig> {
        self.profile_map.get(profile_name)
    }

    pub fn model_config(&self, alias: &str) -> Option<&ModelConfig> {
        self.model_map.get(alias)
    }
}

fn parse_toml_string(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return Some(rhs.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn parse_toml_bool(contents: &str, key: &str) -> Option<bool> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return match rhs.trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
        }
    }
    None
}

fn parse_toml_u32(contents: &str, key: &str) -> Option<u32> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return rhs.trim().parse::<u32>().ok();
        }
    }
    None
}

fn parse_toml_f32(contents: &str, key: &str) -> Option<f32> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return rhs.trim().parse::<f32>().ok();
        }
    }
    None
}
