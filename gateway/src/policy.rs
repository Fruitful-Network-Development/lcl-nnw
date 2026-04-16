use crate::session::Session;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    Chat,
    Coding,
    Research,
    Retrieval,
}

#[derive(Debug, Clone)]
pub struct CapabilityRule {
    pub profile: String,
    pub requires: Capability,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub default_backend: String,
    pub default_endpoint: String,
    pub rules: Vec<CapabilityRule>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            default_backend: "llama_cpp".to_string(),
            default_endpoint: "http://127.0.0.1:8080".to_string(),
            rules: vec![
                CapabilityRule {
                    profile: "chat".to_string(),
                    requires: Capability::Chat,
                },
                CapabilityRule {
                    profile: "coding".to_string(),
                    requires: Capability::Coding,
                },
                CapabilityRule {
                    profile: "research".to_string(),
                    requires: Capability::Research,
                },
                CapabilityRule {
                    profile: "rag".to_string(),
                    requires: Capability::Retrieval,
                },
            ],
        }
    }
}

impl Policy {
    pub fn resolve_profile(
        &self,
        requested_profile: &str,
        session: &Session,
        prompt: &str,
    ) -> String {
        let capability = self.infer_capability(prompt, session);
        let requested = match requested_profile {
            "chat" | "coding" | "research" | "rag" => requested_profile,
            _ => "chat",
        };

        if self.is_profile_allowed(requested, &capability) {
            requested.to_string()
        } else {
            "chat".to_string()
        }
    }

    fn infer_capability(&self, prompt: &str, _session: &Session) -> Capability {
        let lower = prompt.to_lowercase();
        if lower.contains("code") || lower.contains("rust") || lower.contains("bug") {
            Capability::Coding
        } else if lower.contains("source")
            || lower.contains("citation")
            || lower.contains("research")
        {
            Capability::Research
        } else if lower.contains("retrieve")
            || lower.contains("search index")
            || lower.contains("vector")
        {
            Capability::Retrieval
        } else {
            Capability::Chat
        }
    }

    fn is_profile_allowed(&self, profile: &str, capability: &Capability) -> bool {
        self.rules
            .iter()
            .find(|rule| rule.profile == profile)
            .is_none_or(|rule| &rule.requires == capability)
    }
}

#[cfg(test)]
mod tests {
    use super::Policy;
    use crate::session::Session;

    #[test]
    fn coding_prompt_allows_coding_profile() {
        let p = Policy::default();
        let s = Session::new("s1");
        assert_eq!(
            p.resolve_profile("coding", &s, "please debug rust code"),
            "coding"
        );
    }

    #[test]
    fn mismatched_profile_falls_back_to_chat() {
        let p = Policy::default();
        let s = Session::new("s1");
        assert_eq!(p.resolve_profile("coding", &s, "hello there"), "chat");
    }
}
