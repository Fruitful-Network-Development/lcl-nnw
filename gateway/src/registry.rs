use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct ProfileSpec {
    pub model_alias: String,
    pub fallback_model_alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ModelSpec {
    pub alias: String,
    pub backend: String,
    pub default_quantization: String,
    pub min_cpu_threads: u32,
    pub min_ram_gb: f32,
    pub requires_gpu: bool,
    pub estimated_latency_ms: u32,
}

#[derive(Debug, Clone)]
pub struct QuantizationSpec {
    pub default_quantization: String,
    pub preferred: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    profile_specs: HashMap<String, ProfileSpec>,
    model_specs: HashMap<String, ModelSpec>,
    quantization_specs: HashMap<String, QuantizationSpec>,
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
        let models_dir = root.join("models");
        let quantizations_file = root.join("quantizations").join("defaults.toml");

        let mut profile_specs = HashMap::new();
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
                let fallback_model_alias = parse_toml_string(&contents, "fallback_model_alias");

                profile_specs.insert(
                    name,
                    ProfileSpec {
                        model_alias,
                        fallback_model_alias,
                    },
                );
            }
        }

        let mut model_specs = HashMap::new();
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
                let backend = parse_toml_string(&contents, "backend")
                    .unwrap_or_else(|| "llama_cpp".to_string());
                let default_quantization = parse_toml_string(&contents, "default_quantization")
                    .unwrap_or_else(|| "Q4_K_M".to_string());
                let min_cpu_threads = parse_toml_u32(&contents, "min_cpu_threads").unwrap_or(4);
                let min_ram_gb = parse_toml_f32(&contents, "min_ram_gb").unwrap_or(8.0);
                let requires_gpu = parse_toml_bool(&contents, "requires_gpu").unwrap_or(false);
                let estimated_latency_ms =
                    parse_toml_u32(&contents, "estimated_latency_ms").unwrap_or(1200);

                model_specs.insert(
                    alias.clone(),
                    ModelSpec {
                        alias,
                        backend,
                        default_quantization,
                        min_cpu_threads,
                        min_ram_gb,
                        requires_gpu,
                        estimated_latency_ms,
                    },
                );
                profile_map.insert(name, model_alias);
            }
        }
        let model_map = load_models(&root.join("models"))?;
        let profile_map = load_profiles(&root.join("profiles"))?;

        let quantization_specs = if quantizations_file.is_file() {
            let contents = fs::read_to_string(&quantizations_file)?;
            parse_quantization_specs(&contents)
        } else {
            HashMap::new()
        };

        Ok(Self {
            profile_specs,
            model_specs,
            quantization_specs,
        })
    }

    pub fn profile_model(&self, profile: &str) -> Option<String> {
        self.profile_specs
            .get(profile)
            .map(|spec| spec.model_alias.clone())
    }

    pub fn profile_spec(&self, profile: &str) -> Option<&ProfileSpec> {
        self.profile_specs.get(profile)
    }

    pub fn model_spec(&self, alias: &str) -> Option<&ModelSpec> {
        self.model_specs.get(alias)
    }

    pub fn preferred_quantizations(&self, backend: &str) -> Option<&[String]> {
        self.quantization_specs
            .get(backend)
            .map(|spec| spec.preferred.as_slice())
    }

    pub fn default_quantization(&self, backend: &str) -> Option<&str> {
        self.quantization_specs
            .get(backend)
            .map(|spec| spec.default_quantization.as_str())
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

fn parse_toml_string(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
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

fn parse_toml_u32(contents: &str, key: &str) -> Option<u32> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
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
        if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return rhs.trim().parse::<f32>().ok();
        }
    }
    None
}

fn parse_toml_bool(contents: &str, key: &str) -> Option<bool> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return rhs.trim().parse::<bool>().ok();
        }
    }
    None
}

fn parse_quantization_specs(contents: &str) -> HashMap<String, QuantizationSpec> {
    let mut specs = HashMap::new();
    let mut current_backend: Option<String> = None;

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let backend = line
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string();
            specs.entry(backend.clone()).or_insert(QuantizationSpec {
                default_quantization: String::new(),
                preferred: Vec::new(),
            });
            current_backend = Some(backend);
            continue;
        }

        let Some(backend) = current_backend.as_ref() else {
            continue;
        };

        let Some((lhs, rhs)) = line.split_once('=') else {
            continue;
        };

        let spec = specs.get_mut(backend).expect("backend initialized");
        match lhs.trim() {
            "default" => {
                spec.default_quantization = rhs.trim().trim_matches('"').to_string();
            }
            "preferred" => {
                spec.preferred = parse_string_array(rhs.trim());
            }
            _ => {}
        }
    }

    specs
}

fn parse_string_array(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let value = value.trim();
    if !(value.starts_with('[') && value.ends_with(']')) {
        return out;
    }

    let inner = &value[1..value.len() - 1];
    for part in inner.split(',') {
        let entry = part.trim().trim_matches('"');
        if !entry.is_empty() {
            out.push(entry.to_string());
        }
    }

    out
}
