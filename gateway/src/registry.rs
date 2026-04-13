use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Registry {
    profile_map: HashMap<String, String>,
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
                let name = parse_toml_string(&contents, "name").unwrap_or_else(|| "chat".to_string());
                let model_alias =
                    parse_toml_string(&contents, "model_alias").unwrap_or_else(|| "lead".to_string());
                profile_map.insert(name, model_alias);
            }
        }

        Ok(Self { profile_map })
    }

    pub fn profile_model(&self, profile: &str) -> Option<String> {
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
