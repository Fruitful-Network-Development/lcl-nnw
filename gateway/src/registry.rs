use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct ModelManifest {
    pub name: String,
    pub alias: String,
    pub family: Option<String>,
    pub role: Option<String>,
    pub backend: String,
    pub default_quantization: Option<String>,
    pub weight_source: Option<String>,
    pub local_weight_path: Option<String>,
    pub enabled: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProfileManifest {
    pub name: String,
    pub description: Option<String>,
    pub model_alias: String,
    pub fallback_model_alias: Option<String>,
    pub max_context_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub embedding_model_alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    pub models: HashMap<String, ModelManifest>,
    pub profiles: HashMap<String, ProfileManifest>,
}

impl Registry {
    pub fn load_from_dir(root: &Path) -> io::Result<Self> {
        let models = load_models(&root.join("models"))?;
        let profiles = load_profiles(&root.join("profiles"))?;

        validate_profile_references(&profiles, &models)?;

        Ok(Self { models, profiles })
    }

    pub fn profile_model(&self, profile: &str) -> Option<String> {
        self.profiles.get(profile).map(|p| p.model_alias.clone())
    }
}

fn load_models(models_dir: &Path) -> io::Result<HashMap<String, ModelManifest>> {
    let mut models = HashMap::new();
    for path in toml_files(models_dir)? {
        let manifest: ModelManifest = parse_toml_file(&path)?;
        let alias = manifest.alias.clone();

        if alias.trim().is_empty() {
            return Err(invalid_data(format!(
                "model manifest '{}' has empty alias",
                path.display()
            )));
        }

        if models.insert(alias.clone(), manifest).is_some() {
            return Err(invalid_data(format!("duplicate model alias '{alias}'")));
        }
    }

    Ok(models)
}

fn load_profiles(profiles_dir: &Path) -> io::Result<HashMap<String, ProfileManifest>> {
    let mut profiles = HashMap::new();
    for path in toml_files(profiles_dir)? {
        let manifest: ProfileManifest = parse_toml_file(&path)?;
        let name = manifest.name.clone();

        if name.trim().is_empty() {
            return Err(invalid_data(format!(
                "profile manifest '{}' has empty name",
                path.display()
            )));
        }

        if profiles.insert(name.clone(), manifest).is_some() {
            return Err(invalid_data(format!("duplicate profile name '{name}'")));
        }
    }

    Ok(profiles)
}

fn validate_profile_references(
    profiles: &HashMap<String, ProfileManifest>,
    models: &HashMap<String, ModelManifest>,
) -> io::Result<()> {
    for profile in profiles.values() {
        require_enabled_model(models, &profile.model_alias, &profile.name, "model_alias")?;

        if let Some(alias) = &profile.fallback_model_alias {
            require_enabled_model(models, alias, &profile.name, "fallback_model_alias")?;
        }

        if let Some(alias) = &profile.embedding_model_alias {
            require_enabled_model(models, alias, &profile.name, "embedding_model_alias")?;
        }
    }

    Ok(())
}

fn require_enabled_model(
    models: &HashMap<String, ModelManifest>,
    alias: &str,
    profile_name: &str,
    field_name: &str,
) -> io::Result<()> {
    let model = models.get(alias).ok_or_else(|| {
        invalid_data(format!(
            "profile '{profile_name}' references unknown model alias '{alias}' in {field_name}"
        ))
    })?;

    if !model.enabled {
        return Err(invalid_data(format!(
            "profile '{profile_name}' references disabled model alias '{alias}' in {field_name}"
        )));
    }

    Ok(())
}

fn toml_files(dir: &Path) -> io::Result<Vec<PathBuf>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn parse_toml_file<T: for<'de> Deserialize<'de>>(path: &Path) -> io::Result<T> {
    let contents = fs::read_to_string(path)?;
    toml::from_str(&contents)
        .map_err(|err| invalid_data(format!("failed to parse '{}': {err}", path.display())))
}

fn invalid_data(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
