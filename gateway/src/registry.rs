use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registry {
    profile_map: HashMap<String, ProfileConfig>,
}

#[derive(Debug, Clone)]
pub struct ProfileConfig {
    pub model_alias: String,
    pub temperature: f32,
    pub max_context_tokens: usize,
    pub fallback_model_alias: String,
    pub embedding_model_alias: Option<String>,
}

impl Registry {
    pub fn load_from_dir(root: &Path) -> io::Result<Self> {
        let profiles_dir = root.join("profiles");
        let mut profile_map = HashMap::new();

        if profiles_dir.is_dir() {
            for entry in fs::read_dir(&profiles_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }

                let contents = fs::read_to_string(&path)?;
                let name =
                    parse_toml_string(&contents, "name").unwrap_or_else(|| "chat".to_string());
                let model_alias = parse_toml_string(&contents, "model_alias")
                    .unwrap_or_else(|| "lead".to_string());
                let temperature = parse_toml_f32(&contents, "temperature").unwrap_or(0.4);
                let max_context_tokens =
                    parse_toml_usize(&contents, "max_context_tokens").unwrap_or(8192);
                let fallback_model_alias = parse_toml_string(&contents, "fallback_model_alias")
                    .unwrap_or_else(|| "lead".to_string());
                let embedding_model_alias = parse_toml_string(&contents, "embedding_model_alias");

                profile_map.insert(
                    name,
                    ProfileConfig {
                        model_alias,
                        temperature,
                        max_context_tokens,
                        fallback_model_alias,
                        embedding_model_alias,
                    },
                );
            }
        }

        Ok(Self { profile_map })
    }

    pub fn profile_config(&self, profile: &str) -> Option<ProfileConfig> {
        self.profile_map.get(profile).cloned()
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

fn parse_toml_usize(contents: &str, key: &str) -> Option<usize> {
    parse_toml_raw(contents, key)?.parse::<usize>().ok()
}

fn parse_toml_f32(contents: &str, key: &str) -> Option<f32> {
    parse_toml_raw(contents, key)?.parse::<f32>().ok()
}

fn parse_toml_raw(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return Some(rhs.trim().to_string());
        }
    }
    None
}
