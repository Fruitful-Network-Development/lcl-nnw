use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    lanes::{BackendKind, LaneConfig},
    types::{ChatMessage, UpstreamChatRequest, UpstreamChatResponse},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterOutput {
    pub content: String,
    pub finish_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterErrorKind {
    Timeout,
    Upstream5xx,
    Upstream4xx,
    Network,
    InvalidResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdapterError {
    pub kind: AdapterErrorKind,
    pub message: String,
}

impl AdapterError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.kind,
            AdapterErrorKind::Timeout | AdapterErrorKind::Upstream5xx | AdapterErrorKind::Network
        )
    }

    fn timeout(message: impl Into<String>) -> Self {
        Self {
            kind: AdapterErrorKind::Timeout,
            message: message.into(),
        }
    }

    fn upstream_5xx(message: impl Into<String>) -> Self {
        Self {
            kind: AdapterErrorKind::Upstream5xx,
            message: message.into(),
        }
    }

    fn upstream_4xx(message: impl Into<String>) -> Self {
        Self {
            kind: AdapterErrorKind::Upstream4xx,
            message: message.into(),
        }
    }

    fn network(message: impl Into<String>) -> Self {
        Self {
            kind: AdapterErrorKind::Network,
            message: message.into(),
        }
    }

    fn invalid_response(message: impl Into<String>) -> Self {
        Self {
            kind: AdapterErrorKind::InvalidResponse,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AdapterError {}

pub async fn dispatch_chat(
    client: &Client,
    lane: &LaneConfig,
    messages: &[ChatMessage],
    session_id: Option<&str>,
    intent: Option<&str>,
) -> Result<AdapterOutput, AdapterError> {
    match lane
        .backend_kind()
        .map_err(|err| AdapterError::invalid_response(err.to_string()))?
    {
        BackendKind::LocalLlamaCpp => {
            LocalLlamaCppAdapter.chat_completion(client, lane, messages).await
        }
        BackendKind::RemoteCustomLlama => {
            RemoteCustomLlamaAdapter
                .chat_completion(client, lane, messages, session_id, intent)
                .await
        }
    }
}

struct LocalLlamaCppAdapter;

impl LocalLlamaCppAdapter {
    async fn chat_completion(
        &self,
        client: &Client,
        lane: &LaneConfig,
        messages: &[ChatMessage],
    ) -> Result<AdapterOutput, AdapterError> {
        let body = LlamaCppChatRequest {
            messages: messages.to_vec(),
            temperature: lane.temperature,
            stream: false,
        };

        let response = client
            .post(chat_endpoint(&lane.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(map_transport_error)?;

        let status = response.status();
        if status.is_server_error() {
            let body = response.text().await.unwrap_or_default();
            return Err(AdapterError::upstream_5xx(format!(
                "local_llama_cpp upstream error {}: {}",
                status, body
            )));
        }
        if status.is_client_error() {
            let body = response.text().await.unwrap_or_default();
            return Err(AdapterError::upstream_4xx(format!(
                "local_llama_cpp request rejected {}: {}",
                status, body
            )));
        }

        let payload = response
            .json::<LlamaCppChatResponse>()
            .await
            .map_err(|err| AdapterError::invalid_response(err.to_string()))?;

        let Some(choice) = payload.choices.into_iter().next() else {
            return Err(AdapterError::invalid_response(
                "local_llama_cpp returned no choices",
            ));
        };

        Ok(AdapterOutput {
            content: choice.message.content,
            finish_reason: choice.finish_reason.unwrap_or_else(|| "stop".to_string()),
        })
    }
}

struct RemoteCustomLlamaAdapter;

impl RemoteCustomLlamaAdapter {
    async fn chat_completion(
        &self,
        client: &Client,
        lane: &LaneConfig,
        messages: &[ChatMessage],
        session_id: Option<&str>,
        intent: Option<&str>,
    ) -> Result<AdapterOutput, AdapterError> {
        let body = UpstreamChatRequest {
            model: lane.model_id.clone(),
            messages: messages.to_vec(),
            temperature: lane.temperature,
            max_context_tokens: lane.max_context_tokens,
            session_id: session_id.map(ToOwned::to_owned),
            intent: intent.map(ToOwned::to_owned),
        };

        let response = client
            .post(chat_endpoint(&lane.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(map_transport_error)?;

        let status = response.status();
        if status.is_server_error() {
            let body = response.text().await.unwrap_or_default();
            return Err(AdapterError::upstream_5xx(format!(
                "remote_custom_llama upstream error {}: {}",
                status, body
            )));
        }
        if status.is_client_error() {
            let body = response.text().await.unwrap_or_default();
            return Err(AdapterError::upstream_4xx(format!(
                "remote_custom_llama request rejected {}: {}",
                status, body
            )));
        }

        let payload = response
            .json::<UpstreamChatResponse>()
            .await
            .map_err(|err| AdapterError::invalid_response(err.to_string()))?;

        Ok(AdapterOutput {
            content: payload.content,
            finish_reason: payload.finish_reason,
        })
    }
}

fn chat_endpoint(base: &str) -> String {
    format!("{}/v1/chat/completions", base.trim_end_matches('/'))
}

fn map_transport_error(err: reqwest::Error) -> AdapterError {
    if err.is_timeout() {
        AdapterError::timeout(err.to_string())
    } else {
        AdapterError::network(err.to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
struct LlamaCppChatRequest {
    messages: Vec<ChatMessage>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct LlamaCppChatResponse {
    choices: Vec<LlamaCppChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct LlamaCppChoice {
    message: ChatMessage,
    finish_reason: Option<String>,
}
