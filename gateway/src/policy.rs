use crate::registry::{ModelSpec, Registry};
use crate::session::Session;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Policy {
    pub default_backend: String,
    pub default_endpoint: String,
    pub hardware_profile: HardwareProfile,
}

#[derive(Debug, Clone)]
pub struct HardwareProfile {
    pub cpu_threads: u32,
    pub ram_budget_gb: f32,
    pub gpu_present: bool,
    pub target_latency_ms: u32,
}

#[derive(Debug, Clone)]
pub struct ModelEvaluation {
    pub model_alias: String,
    pub backend: String,
    pub quantization: String,
    pub rationale: String,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            default_backend: "llama_cpp".to_string(),
            default_endpoint: "http://127.0.0.1:8080".to_string(),
            hardware_profile: HardwareProfile::load_from_file(Path::new(
                "../ops/env/hardware-profile.toml",
            ))
            .unwrap_or_else(HardwareProfile::default),
        }
    }
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            cpu_threads: 8,
            ram_budget_gb: 16.0,
            gpu_present: false,
            target_latency_ms: 1500,
        }
    }
}

impl HardwareProfile {
    pub fn load_from_file(path: &Path) -> Option<Self> {
        let contents = fs::read_to_string(path).ok()?;
        Some(Self {
            cpu_threads: parse_toml_u32(&contents, "cpu_threads").unwrap_or(8),
            ram_budget_gb: parse_toml_f32(&contents, "ram_budget_gb").unwrap_or(16.0),
            gpu_present: parse_toml_bool(&contents, "gpu_present").unwrap_or(false),
            target_latency_ms: parse_toml_u32(&contents, "target_latency_ms").unwrap_or(1500),
        })
    }
}

impl Policy {
    pub fn resolve_profile(&self, requested_profile: &str, _session: &Session) -> String {
        match requested_profile {
            "chat" | "coding" | "research" | "rag" => requested_profile.to_string(),
            _ => "chat".to_string(),
        }
    }

    pub fn evaluate_model_for_profile(
        &self,
        registry: &Registry,
        profile: &str,
    ) -> ModelEvaluation {
        let profile_spec = registry.profile_spec(profile);
        let primary = profile_spec
            .map(|spec| spec.model_alias.clone())
            .unwrap_or_else(|| "lead".to_string());

        let mut candidates = vec![primary.clone()];
        if let Some(fallback) = profile_spec.and_then(|spec| spec.fallback_model_alias.clone()) {
            candidates.push(fallback);
        }
        candidates.extend([
            "lead".to_string(),
            "coder".to_string(),
            "reasoning".to_string(),
        ]);

        let mut deduped = Vec::new();
        let mut seen = HashSet::new();
        for alias in candidates {
            if seen.insert(alias.clone()) {
                deduped.push(alias);
            }
        }

        let mut rejection_notes = Vec::new();
        let mut best_effort: Option<(ModelSpec, String, usize)> = None;

        for alias in deduped {
            let Some(model) = registry.model_spec(&alias).cloned() else {
                rejection_notes.push(format!("{alias}: model alias not found in registry"));
                continue;
            };

            let quantization = choose_quantization(registry, &model);
            let mut failures = Vec::new();

            if self.hardware_profile.cpu_threads < model.min_cpu_threads {
                failures.push(format!(
                    "cpu {} < required {}",
                    self.hardware_profile.cpu_threads, model.min_cpu_threads
                ));
            }
            if self.hardware_profile.ram_budget_gb < model.min_ram_gb {
                failures.push(format!(
                    "ram {:.1}GB < required {:.1}GB",
                    self.hardware_profile.ram_budget_gb, model.min_ram_gb
                ));
            }
            if model.requires_gpu && !self.hardware_profile.gpu_present {
                failures.push("gpu required but unavailable".to_string());
            }
            if model.estimated_latency_ms > self.hardware_profile.target_latency_ms {
                failures.push(format!(
                    "latency {}ms > target {}ms",
                    model.estimated_latency_ms, self.hardware_profile.target_latency_ms
                ));
            }

            if failures.is_empty() {
                let rationale = if alias == primary {
                    format!(
                        "selected primary model '{alias}' with quantization '{quantization}'; fits hardware profile"
                    )
                } else {
                    format!(
                        "primary model '{primary}' rejected by hardware budget; selected fallback '{alias}' with quantization '{quantization}'"
                    )
                };
                return ModelEvaluation {
                    model_alias: alias,
                    backend: model.backend,
                    quantization,
                    rationale,
                };
            }

            rejection_notes.push(format!("{}: {}", model.alias, failures.join(", ")));
            let score = failures.len();
            match &best_effort {
                Some((_, _, best_score)) if *best_score <= score => {}
                _ => {
                    best_effort = Some((model, quantization, score));
                }
            }
        }

        if let Some((model, quantization, _)) = best_effort {
            return ModelEvaluation {
                model_alias: model.alias.clone(),
                backend: model.backend.clone(),
                quantization: quantization.clone(),
                rationale: format!(
                    "all candidates exceeded budget ({}); selected lowest-risk fallback '{}' with quantization '{}'",
                    rejection_notes.join(" | "),
                    model.alias,
                    quantization
                ),
            };
        }

        ModelEvaluation {
            model_alias: "lead".to_string(),
            backend: self.default_backend.clone(),
            quantization: "Q4_K_M".to_string(),
            rationale: "registry did not provide viable models; defaulted to lead".to_string(),
        }
    }
}

fn choose_quantization(registry: &Registry, model: &ModelSpec) -> String {
    let preferred = registry
        .preferred_quantizations(&model.backend)
        .unwrap_or(&[])
        .iter()
        .find(|tier| tier.as_str() == model.default_quantization)
        .cloned();

    preferred
        .or_else(|| {
            registry
                .preferred_quantizations(&model.backend)
                .and_then(|tiers| tiers.first().cloned())
        })
        .or_else(|| {
            registry
                .default_quantization(&model.backend)
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| model.default_quantization.clone())
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

fn parse_toml_bool(contents: &str, key: &str) -> Option<bool> {
    for line in contents.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (lhs, rhs) = line.split_once('=')?;
        if lhs.trim() == key {
            return rhs.trim().parse::<bool>().ok();
        }
    }
    None
}
