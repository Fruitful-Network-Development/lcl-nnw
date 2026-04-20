use std::sync::atomic::{AtomicU64, Ordering};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde::Serialize;

use crate::{
    adapters::{dispatch_chat, AdapterError, AdapterErrorKind},
    config::AppConfig,
    lanes::{LaneConfig, LaneRegistry},
    types::{
        ChatChoice, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, HealthResponse,
        LaneSummary, ListLanesResponse, RouteMetadata,
    },
};

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub lanes: LaneRegistry,
    pub client: Client,
}

impl AppState {
    pub fn new(config: AppConfig, lanes: LaneRegistry, client: Client) -> Self {
        Self {
            config,
            lanes,
            client,
        }
    }
}

pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/lanes", get(list_lanes))
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

async fn list_lanes(State(state): State<AppState>) -> Json<ListLanesResponse> {
    let data = state
        .lanes
        .enabled_lanes()
        .iter()
        .map(LaneSummary::from)
        .collect();

    Json(ListLanesResponse {
        object: "list".to_string(),
        default_lane: state.lanes.default_lane_name().to_string(),
        data,
    })
}

async fn chat_completions(
    State(state): State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, AppError> {
    if payload.messages.is_empty() {
        return Err(AppError::bad_request(
            "messages must contain at least one chat message",
        ));
    }

    let session_id = payload.session_id.clone().unwrap_or_else(next_session_id);
    let selected = select_lane(&state.lanes, payload.lane.as_deref())?;
    let mut active_lane = selected.lane.clone();
    let mut fallback_used = selected.fallback_used;

    let completion = match dispatch_chat(
        &state.client,
        &active_lane,
        &payload.messages,
        Some(&session_id),
        payload.intent.as_deref(),
    )
    .await
    {
        Ok(output) => output,
        Err(error) if !selected.explicit && !fallback_used && error.is_retryable() => {
            let Some(fallback_name) = active_lane.fallback_lane.clone() else {
                return Err(AppError::from_adapter(error));
            };

            let fallback_lane = state
                .lanes
                .get(&fallback_name)
                .cloned()
                .ok_or_else(|| AppError::internal("configured fallback lane is missing"))?;

            if !fallback_lane.enabled {
                return Err(AppError::service_unavailable(format!(
                    "fallback lane `{}` is disabled",
                    fallback_lane.name
                )));
            }

            fallback_used = true;
            active_lane = fallback_lane;
            dispatch_chat(
                &state.client,
                &active_lane,
                &payload.messages,
                Some(&session_id),
                payload.intent.as_deref(),
            )
            .await
            .map_err(AppError::from_adapter)?
        }
        Err(error) => return Err(AppError::from_adapter(error)),
    };

    Ok(Json(ChatCompletionResponse {
        id: format!("chatcmpl-{session_id}"),
        object: "chat.completion".to_string(),
        lane: active_lane.name.clone(),
        model: active_lane.model_id.clone(),
        session_id: session_id.clone(),
        route: RouteMetadata {
            lane: active_lane.name.clone(),
            backend: active_lane.backend.clone(),
            endpoint: active_lane.endpoint.clone(),
            fallback_used,
        },
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: completion.content,
            },
            finish_reason: completion.finish_reason,
        }],
    }))
}

#[derive(Debug, Clone)]
struct SelectedLane {
    lane: LaneConfig,
    explicit: bool,
    fallback_used: bool,
}

fn select_lane(registry: &LaneRegistry, requested_lane: Option<&str>) -> Result<SelectedLane, AppError> {
    match requested_lane {
        Some(lane_name) => {
            let lane = registry
                .get(lane_name)
                .cloned()
                .ok_or_else(|| AppError::bad_request(format!("unknown lane `{lane_name}`")))?;

            if !lane.enabled {
                return Err(AppError::service_unavailable(format!(
                    "requested lane `{lane_name}` is disabled"
                )));
            }

            Ok(SelectedLane {
                lane,
                explicit: true,
                fallback_used: false,
            })
        }
        None => {
            let lane = registry
                .get(registry.default_lane_name())
                .cloned()
                .ok_or_else(|| AppError::internal("configured default lane is missing"))?;

            if lane.enabled {
                return Ok(SelectedLane {
                    lane,
                    explicit: false,
                    fallback_used: false,
                });
            }

            let Some(fallback_name) = lane.fallback_lane.clone() else {
                return Err(AppError::service_unavailable(format!(
                    "default lane `{}` is disabled and has no fallback",
                    lane.name
                )));
            };

            let fallback_lane = registry
                .get(&fallback_name)
                .cloned()
                .ok_or_else(|| AppError::internal("configured fallback lane is missing"))?;

            if !fallback_lane.enabled {
                return Err(AppError::service_unavailable(format!(
                    "default lane `{}` is disabled and fallback lane `{}` is disabled",
                    lane.name, fallback_lane.name
                )));
            }

            Ok(SelectedLane {
                lane: fallback_lane,
                explicit: false,
                fallback_used: true,
            })
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

#[derive(Debug)]
struct AppError {
    status: StatusCode,
    code: String,
    message: String,
}

impl AppError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_request".to_string(),
            message: message.into(),
        }
    }

    fn service_unavailable(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "lane_unavailable".to_string(),
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "gateway_misconfigured".to_string(),
            message: message.into(),
        }
    }

    fn from_adapter(error: AdapterError) -> Self {
        let code = match error.kind {
            AdapterErrorKind::Timeout => "upstream_timeout",
            AdapterErrorKind::Upstream5xx => "upstream_5xx",
            AdapterErrorKind::Upstream4xx => "upstream_4xx",
            AdapterErrorKind::Network => "upstream_network_error",
            AdapterErrorKind::InvalidResponse => "upstream_invalid_response",
        };

        Self {
            status: StatusCode::BAD_GATEWAY,
            code: code.to_string(),
            message: error.message,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: self.code,
                message: self.message,
            },
        });

        (self.status, body).into_response()
    }
}

fn next_session_id() -> String {
    format!("sess-{}", REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed))
}
