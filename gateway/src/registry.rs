use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

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
            }
        }

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
    }
}

fn parse_toml_string(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() || line.starts_with('[') {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return Some(rhs.trim().trim_matches('"').to_string());
        }
    }
    None
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
