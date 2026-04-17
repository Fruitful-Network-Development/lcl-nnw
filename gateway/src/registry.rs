use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize)]
pub struct ModelManifest {
    pub name: String,
    pub alias: String,
    pub backend: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
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
                profile_map.insert(name, model_alias);
            }
        }
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

    pub fn models(&self) -> Vec<(String, String)> {
        let mut entries = self
            .profile_map
            .iter()
            .map(|(profile, model)| (profile.clone(), model.clone()))
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    pub fn backend_for_alias(&self, alias: &str) -> Option<&str> {
        self.model_map.get(alias).and_then(|m| {
            if m.enabled {
                Some(m.backend.as_str())
            } else {
                None
            }
        })
    }

    pub fn is_model_routable(&self, alias: &str) -> bool {
        self.model_map.get(alias).is_some_and(|model| model.enabled)
    }

    pub fn first_enabled_model_alias(&self) -> Option<&str> {
        let mut aliases: Vec<&str> = self
            .model_map
            .iter()
            .filter_map(|(alias, model)| model.enabled.then_some(alias.as_str()))
            .collect();
        aliases.sort_unstable();
        aliases.into_iter().next()
    }

    pub fn preferred_routable_alias(&self) -> Option<&str> {
        if self.is_model_routable("lead") {
            Some("lead")
        } else {
            self.first_enabled_model_alias()
        }
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
        let parsed = parse_profile_manifest(&file, &data)?;

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
        let parsed = parse_model_manifest(&file, &data)?;

        models.insert(parsed.alias.clone(), parsed);
    }

    Ok(models)
}

fn parse_model_manifest(file: &Path, data: &str) -> io::Result<ModelManifest> {
    toml::from_str(data).map_err(|err| invalid_manifest(file, err))
}

fn parse_profile_manifest(file: &Path, data: &str) -> io::Result<ProfileManifest> {
    toml::from_str(data).map_err(|err| invalid_manifest(file, err))
}

fn invalid_manifest(file: &Path, error: toml::de::Error) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("Invalid manifest {}: {}", file.display(), error),
    )
}
