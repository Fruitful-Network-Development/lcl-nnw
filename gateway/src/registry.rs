use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ModelManifest {
    pub alias: String,
    pub name: String,
    pub backend: String,
}

#[derive(Debug, Clone)]
pub struct Registry {
    profile_map: HashMap<String, String>,
    models: HashMap<String, ModelManifest>,
}

impl Registry {
    pub fn load_from_dir(root: &Path) -> io::Result<Self> {
        let profiles_dir = root.join("profiles");
        let models_dir = root.join("models");

        let mut profile_map = HashMap::new();
        let mut models = HashMap::new();

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
                profile_map.insert(name, model_alias);
            }
        }

        if models_dir.is_dir() {
            for entry in fs::read_dir(&models_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                    continue;
                }

                let contents = fs::read_to_string(&path)?;
                let alias =
                    parse_toml_string(&contents, "alias").unwrap_or_else(|| "lead".to_string());
                let name = parse_toml_string(&contents, "name").unwrap_or_else(|| alias.clone());
                let backend = parse_toml_string(&contents, "backend")
                    .unwrap_or_else(|| "llama_cpp".to_string());

                models.insert(
                    alias.clone(),
                    ModelManifest {
                        alias,
                        name,
                        backend,
                    },
                );
            }
        }

        Ok(Self {
            profile_map,
            models,
        })
    }

    pub fn profile_model(&self, profile: &str) -> Option<String> {
        self.profile_map.get(profile).cloned()
    }

    pub fn model_by_alias(&self, alias: &str) -> Option<&ModelManifest> {
        self.models.get(alias)
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
