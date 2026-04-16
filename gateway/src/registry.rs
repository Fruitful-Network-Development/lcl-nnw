use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registry {
    profile_map: HashMap<String, ProfileConfig>,
    model_map: HashMap<String, ModelManifest>,
}

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub model_alias: String,
    pub temperature: f32,
    pub max_context_tokens: usize,
    pub fallback_model_alias: String,
    pub embedding_model_alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelManifest {
    pub name: String,
    pub alias: String,
    pub backend: String,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
struct ProfileManifest {
    pub name: String,
    pub model_alias: String,
    pub temperature: f32,
    pub max_context_tokens: usize,
    pub fallback_model_alias: String,
    pub embedding_model_alias: Option<String>,
}

impl Registry {
    pub fn load_from_dir(root: &Path) -> io::Result<Self> {
        let model_map = load_models(&root.join("models"))?;
        let profile_map = load_profiles(&root.join("profiles"))?;

        Ok(Self {
            profile_map,
            model_map,
        })
    }

    pub fn profile_config(&self, profile: &str) -> Option<ProfileConfig> {
        self.profile_map.get(profile).cloned()
    }

    pub fn backend_for_alias(&self, alias: &str) -> Option<&str> {
        self.model_map.get(alias).and_then(|m| {
            if m.enabled {
                Some(m.backend.as_str())
            } else {
                None
            }
        })
    }

    pub fn has_model_alias(&self, alias: &str) -> bool {
        self.model_map.contains_key(alias)
    }
    pub fn model_name_for_alias(&self, alias: &str) -> Option<&str> {
        self.model_map.get(alias).map(|m| m.name.as_str())
    }
}

fn load_profiles(path: &Path) -> io::Result<HashMap<String, ProfileConfig>> {
    let mut profiles = HashMap::new();
    if !path.is_dir() {
        return Ok(profiles);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file = entry.path();
        if file.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }

        let data = fs::read_to_string(&file)?;
        let parsed = parse_profile_manifest(&data);

        profiles.insert(
            parsed.name,
            ProfileConfig {
                model_alias: parsed.model_alias,
                temperature: parsed.temperature,
                max_context_tokens: parsed.max_context_tokens,
                fallback_model_alias: parsed.fallback_model_alias,
                embedding_model_alias: parsed.embedding_model_alias,
            },
        );
    }

    Ok(profiles)
}

fn load_models(path: &Path) -> io::Result<HashMap<String, ModelManifest>> {
    let mut models = HashMap::new();
    if !path.is_dir() {
        return Ok(models);
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file = entry.path();
        if file.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }

        let data = fs::read_to_string(&file)?;
        let parsed = parse_model_manifest(&data);

        models.insert(parsed.alias.clone(), parsed);
    }

    Ok(models)
}

fn parse_model_manifest(data: &str) -> ModelManifest {
    ModelManifest {
        name: parse_string(data, "name").unwrap_or_else(|| "unknown".to_string()),
        alias: parse_string(data, "alias").unwrap_or_else(|| "lead".to_string()),
        backend: parse_string(data, "backend").unwrap_or_else(|| "llama_cpp".to_string()),
        enabled: parse_bool(data, "enabled").unwrap_or(true),
    }
}

fn parse_profile_manifest(data: &str) -> ProfileManifest {
    ProfileManifest {
        name: parse_string(data, "name").unwrap_or_else(|| "chat".to_string()),
        model_alias: parse_string(data, "model_alias").unwrap_or_else(|| "lead".to_string()),
        temperature: parse_f32(data, "temperature").unwrap_or(0.4),
        max_context_tokens: parse_usize(data, "max_context_tokens").unwrap_or(8192),
        fallback_model_alias: parse_string(data, "fallback_model_alias")
            .unwrap_or_else(|| "lead".to_string()),
        embedding_model_alias: parse_string(data, "embedding_model_alias"),
    }
}

fn parse_string(data: &str, key: &str) -> Option<String> {
    parse_raw(data, key).map(|v| v.trim_matches('"').to_string())
}

fn parse_bool(data: &str, key: &str) -> Option<bool> {
    match parse_raw(data, key)?.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_f32(data: &str, key: &str) -> Option<f32> {
    parse_raw(data, key)?.parse::<f32>().ok()
}

fn parse_usize(data: &str, key: &str) -> Option<usize> {
    parse_raw(data, key)?.parse::<usize>().ok()
}

fn parse_raw(data: &str, key: &str) -> Option<String> {
    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return Some(rhs.trim().to_string());
        }
    }
    None
}
