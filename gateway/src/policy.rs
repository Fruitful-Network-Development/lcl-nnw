use crate::session::Session;

#[derive(Debug, Clone)]
pub struct Policy {
    pub default_backend: String,
    pub default_endpoint: String,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            default_backend: "llama_cpp".to_string(),
            default_endpoint: "http://127.0.0.1:8080".to_string(),
        }
    }
}

impl Policy {
    pub fn resolve_profile(&self, requested_profile: &str, _session: &Session) -> String {
        match requested_profile {
            "chat" | "coding" | "research" | "rag" => requested_profile.to_string(),
            _ => "chat".to_string(),
        }
    }
}
