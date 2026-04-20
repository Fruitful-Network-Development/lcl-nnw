use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatCompletionRequest {
    pub lane: Option<String>,
    pub session_id: Option<String>,
    pub intent: Option<String>,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatChoice {
    pub index: usize,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteMetadata {
    pub lane: String,
    pub backend: String,
    pub endpoint: String,
    pub fallback_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub lane: String,
    pub model: String,
    pub session_id: String,
    pub route: RouteMetadata,
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaneSummary {
    pub name: String,
    pub backend: String,
    pub endpoint: String,
    pub model_id: String,
    pub max_context_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListLanesResponse {
    pub object: String,
    pub default_lane: String,
    pub data: Vec<LaneSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpstreamChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub max_context_tokens: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpstreamChatResponse {
    pub content: String,
    #[serde(default = "default_finish_reason")]
    pub finish_reason: String,
}

impl From<&crate::lanes::LaneConfig> for LaneSummary {
    fn from(value: &crate::lanes::LaneConfig) -> Self {
        Self {
            name: value.name.clone(),
            backend: value.backend.clone(),
            endpoint: value.endpoint.clone(),
            model_id: value.model_id.clone(),
            max_context_tokens: value.max_context_tokens,
        }
    }
}

fn default_finish_reason() -> String {
    "stop".to_string()
}
