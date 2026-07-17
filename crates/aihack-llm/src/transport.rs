use std::{
    io::{ErrorKind, Read},
    net::ToSocketAddrs,
    time::Duration,
};

use aihack_ai_contract::{
    ActionSpace, ClientRevision, EntityObservation, GameEvent, ItemObservation, Observation,
    PlayerObservation, RunStateSummary, TileObservation,
};
use reqwest::{blocking::Client, redirect::Policy, Url};
use serde::{Deserialize, Serialize};

use crate::{
    config::{validate_endpoint, LlmConfigError, LlmRequestKind, LocalLlmConfig},
    decision::{
        parse_decision_payload_json, validate_decision_payload, DecisionPayload, DecisionRequest,
    },
    is_forbidden_control,
    narrative::{NarrativeError, NarrativeProvider, NarrativeRequest},
    soft_adjudication::{parse_soft_adjudication_payload_json, SoftAdjudicationRequest},
};
use aihack_ai_contract::llm::SoftAdjudicationPayload;

pub const REQUEST_BODY_LIMIT: usize = 32_768;
pub const RESPONSE_BODY_LIMIT: usize = 65_536;

const SYSTEM_PROMPT: &str =
    "Return one JSON object matching the requested AIHack schema. Never emit a state patch.";

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmResponseError {
    Disabled,
    InvalidEndpoint,
    Unavailable,
    Timeout,
    HttpStatus { code: u16 },
    BodyTooLarge { limit_bytes: usize },
    InvalidSchema { code: LlmValidationCode },
    Stale,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmValidationCode {
    InvalidJson,
    PayloadTooLarge,
    MissingChoice,
    NonTextContent,
    WrongKind,
    EmptyText,
    TextTooLong,
    ControlCharacter,
    UnknownRequestId,
    InvalidAction,
    InvalidConfidence,
    InvalidReasonCode,
}

#[derive(Debug, Clone)]
pub struct OpenAiNarrativeTransport {
    config: LocalLlmConfig,
    client: Client,
    chat_completions_url: Url,
}

impl OpenAiNarrativeTransport {
    pub fn new(config: LocalLlmConfig) -> Result<Self, LlmResponseError> {
        let base_url = validate_endpoint(config.endpoint()).map_err(map_config_error)?;
        let chat_completions_url = Url::parse(&format!(
            "{}/chat/completions",
            base_url.as_str().trim_end_matches('/')
        ))
        .map_err(|_| LlmResponseError::InvalidEndpoint)?;
        let resolved_addresses = resolve_loopback(&chat_completions_url)?;
        let host = chat_completions_url
            .host_str()
            .ok_or(LlmResponseError::InvalidEndpoint)?;
        let client = Client::builder()
            .connect_timeout(Duration::from_millis(config.connect_timeout_ms()))
            .redirect(Policy::none())
            .no_proxy()
            .referer(false)
            .resolve_to_addrs(host, &resolved_addresses)
            .build()
            .map_err(|_| LlmResponseError::Unavailable)?;
        Ok(Self {
            config,
            client,
            chat_completions_url,
        })
    }

    pub fn complete(&self, request: &NarrativeRequest) -> Result<String, LlmResponseError> {
        let timeout =
            Duration::from_millis(self.config.request_timeout_ms(&LlmRequestKind::Narrative));
        self.complete_with_timeout(request, timeout)
    }

    pub fn complete_decision(
        &self,
        request: &DecisionRequest,
    ) -> Result<DecisionPayload, LlmResponseError> {
        validate_observation_bounds(&request.observation)?;
        let content = self.complete_request(
            &request.revision,
            &request.observation,
            &request.action_space,
            WireRequestKind::Decision,
            None,
            Duration::from_millis(self.config.request_timeout_ms(&LlmRequestKind::Decision)),
        )?;
        let payload = parse_decision_payload_json(&content, &request.action_space)?;
        let validated = validate_decision_payload(
            request.revision.clone(),
            payload,
            &request.revision,
            &request.action_space,
        )?;
        Ok(DecisionPayload {
            action: validated.action(),
            rationale: validated.rationale().to_string(),
            confidence: validated.confidence(),
        })
    }

    pub fn complete_soft_adjudication(
        &self,
        request: &SoftAdjudicationRequest,
    ) -> Result<SoftAdjudicationPayload, LlmResponseError> {
        validate_observation_bounds(&request.observation)?;
        let user_text = crate::config::validate_user_text(&request.user_text)
            .map_err(|code| invalid(map_input_code(code)))?;
        let content = self.complete_request(
            &request.revision,
            &request.observation,
            &request.observation.action_space,
            WireRequestKind::SoftAdjudication,
            Some(&user_text),
            Duration::from_millis(self.config.request_timeout_ms(
                &LlmRequestKind::SoftAdjudication {
                    user_text: user_text.clone(),
                },
            )),
        )?;
        parse_soft_adjudication_payload_json(&content)
    }

    fn complete_with_timeout(
        &self,
        request: &NarrativeRequest,
        timeout: Duration,
    ) -> Result<String, LlmResponseError> {
        validate_observation_bounds(&request.observation)?;
        let content = self.complete_request(
            &request.revision,
            &request.observation,
            &request.observation.action_space,
            WireRequestKind::Narrative,
            None,
            timeout,
        )?;
        parse_narrative_content(&content, self.config.max_output_chars())
    }

    fn complete_request(
        &self,
        revision: &ClientRevision,
        observation: &Observation,
        action_space: &ActionSpace,
        kind: WireRequestKind,
        user_text: Option<&str>,
        timeout: Duration,
    ) -> Result<String, LlmResponseError> {
        if !self.config.enabled() {
            return Err(LlmResponseError::Disabled);
        }
        validate_resolved_loopback(&self.chat_completions_url)?;
        let canonical_input = serde_json::to_string(&LlmWireInput {
            schema_version: 1,
            revision,
            kind,
            observation: LlmObservationView::from(observation),
            action_space,
            user_text,
        })
        .map_err(|_| invalid(LlmValidationCode::InvalidJson))?;
        if canonical_input.len() > REQUEST_BODY_LIMIT {
            return Err(invalid(LlmValidationCode::PayloadTooLarge));
        }
        let body = serde_json::to_vec(&ChatCompletionRequest {
            model: self.config.model(),
            messages: [
                ChatMessage {
                    role: "system",
                    content: SYSTEM_PROMPT,
                },
                ChatMessage {
                    role: "user",
                    content: &canonical_input,
                },
            ],
            temperature: 0.0,
            max_tokens: 128,
            stream: false,
        })
        .map_err(|_| invalid(LlmValidationCode::InvalidJson))?;
        let mut response = self
            .client
            .post(self.chat_completions_url.clone())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body)
            .timeout(timeout)
            .send()
            .map_err(map_reqwest_error)?;
        if !response.status().is_success() {
            return Err(LlmResponseError::HttpStatus {
                code: response.status().as_u16(),
            });
        }
        if response
            .content_length()
            .is_some_and(|length| length > RESPONSE_BODY_LIMIT as u64)
        {
            return Err(LlmResponseError::BodyTooLarge {
                limit_bytes: RESPONSE_BODY_LIMIT,
            });
        }

        let mut body = Vec::with_capacity(
            response
                .content_length()
                .unwrap_or(0)
                .min(RESPONSE_BODY_LIMIT as u64) as usize,
        );
        response
            .by_ref()
            .take((RESPONSE_BODY_LIMIT + 1) as u64)
            .read_to_end(&mut body)
            .map_err(|error| {
                if matches!(error.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) {
                    LlmResponseError::Timeout
                } else {
                    LlmResponseError::Unavailable
                }
            })?;
        if body.len() > RESPONSE_BODY_LIMIT {
            return Err(LlmResponseError::BodyTooLarge {
                limit_bytes: RESPONSE_BODY_LIMIT,
            });
        }
        extract_response_content(&body)
    }
}

impl NarrativeProvider for OpenAiNarrativeTransport {
    fn generate(
        &self,
        request: &NarrativeRequest,
        timeout: Duration,
    ) -> Result<String, NarrativeError> {
        self.complete_with_timeout(request, timeout)
            .map_err(|error| match error {
                LlmResponseError::Timeout => NarrativeError::Timeout,
                LlmResponseError::InvalidSchema { .. }
                | LlmResponseError::BodyTooLarge { .. }
                | LlmResponseError::HttpStatus { .. }
                | LlmResponseError::InvalidEndpoint => {
                    NarrativeError::Invalid(format!("{error:?}"))
                }
                LlmResponseError::Disabled | LlmResponseError::Unavailable => {
                    NarrativeError::Provider(format!("{error:?}"))
                }
                LlmResponseError::Stale => NarrativeError::Invalid("Stale".to_string()),
            })
    }
}

fn validate_resolved_loopback(url: &Url) -> Result<(), LlmResponseError> {
    resolve_loopback(url).map(|_| ())
}

fn resolve_loopback(url: &Url) -> Result<Vec<std::net::SocketAddr>, LlmResponseError> {
    let host = url.host_str().ok_or(LlmResponseError::InvalidEndpoint)?;
    let port = url.port().ok_or(LlmResponseError::InvalidEndpoint)?;
    let addresses = (host, port)
        .to_socket_addrs()
        .map_err(|_| LlmResponseError::Unavailable)?
        .collect::<Vec<_>>();
    if addresses.is_empty() || addresses.iter().any(|address| !address.ip().is_loopback()) {
        return Err(LlmResponseError::InvalidEndpoint);
    }
    Ok(addresses)
}

fn validate_observation_bounds(observation: &Observation) -> Result<(), LlmResponseError> {
    if observation.visible_tiles.len() > 800
        || observation.visible_entities.len() > 128
        || observation.inventory.len() > 52
        || observation.last_events.len() > 20
        || observation.action_space.commands.len() > 64
    {
        return Err(invalid(LlmValidationCode::PayloadTooLarge));
    }
    Ok(())
}

fn extract_response_content(body: &[u8]) -> Result<String, LlmResponseError> {
    let response: ChatCompletionResponse =
        serde_json::from_slice(body).map_err(|_| invalid(LlmValidationCode::InvalidJson))?;
    let choice = response
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| invalid(LlmValidationCode::MissingChoice))?;
    choice
        .message
        .content
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| invalid(LlmValidationCode::NonTextContent))
}

fn parse_narrative_content(
    content: &str,
    max_output_chars: usize,
) -> Result<String, LlmResponseError> {
    let payload: NarrativeWirePayload =
        serde_json::from_str(content).map_err(|_| invalid(LlmValidationCode::InvalidJson))?;
    if payload.kind != WireResponseKind::Narrative {
        return Err(invalid(LlmValidationCode::WrongKind));
    }
    validate_output_text(&payload.text, max_output_chars)
}

fn map_input_code(code: crate::config::LlmInputCode) -> LlmValidationCode {
    match code {
        crate::config::LlmInputCode::EmptyUserText => LlmValidationCode::EmptyText,
        crate::config::LlmInputCode::TextTooLong => LlmValidationCode::TextTooLong,
        crate::config::LlmInputCode::ControlCharacter => LlmValidationCode::ControlCharacter,
    }
}

fn validate_output_text(text: &str, max_chars: usize) -> Result<String, LlmResponseError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(invalid(LlmValidationCode::EmptyText));
    }
    if text.chars().count() > max_chars {
        return Err(invalid(LlmValidationCode::TextTooLong));
    }
    if text.chars().any(is_forbidden_control) {
        return Err(invalid(LlmValidationCode::ControlCharacter));
    }
    Ok(text.to_string())
}

fn map_reqwest_error(error: reqwest::Error) -> LlmResponseError {
    if error.is_timeout() {
        LlmResponseError::Timeout
    } else {
        LlmResponseError::Unavailable
    }
}

fn map_config_error(error: LlmConfigError) -> LlmResponseError {
    match error {
        LlmConfigError::InvalidEndpoint => LlmResponseError::InvalidEndpoint,
        _ => LlmResponseError::Unavailable,
    }
}

fn invalid(code: LlmValidationCode) -> LlmResponseError {
    LlmResponseError::InvalidSchema { code }
}

#[derive(Serialize)]
struct LlmWireInput<'a> {
    schema_version: u16,
    revision: &'a ClientRevision,
    kind: WireRequestKind,
    observation: LlmObservationView<'a>,
    action_space: &'a ActionSpace,
    #[serde(rename = "userText", skip_serializing_if = "Option::is_none")]
    user_text: Option<&'a str>,
}

#[derive(Serialize)]
struct LlmObservationView<'a> {
    turn: u64,
    run_state: &'a RunStateSummary,
    player: &'a PlayerObservation,
    visible_tiles: &'a [TileObservation],
    visible_entities: &'a [EntityObservation],
    inventory: &'a [ItemObservation],
    last_events: &'a [GameEvent],
}

impl<'a> From<&'a Observation> for LlmObservationView<'a> {
    fn from(observation: &'a Observation) -> Self {
        Self {
            turn: observation.turn,
            run_state: &observation.run_state,
            player: &observation.player,
            visible_tiles: &observation.visible_tiles,
            visible_entities: &observation.visible_entities,
            inventory: &observation.inventory,
            last_events: &observation.last_events,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum WireRequestKind {
    Narrative,
    Decision,
    SoftAdjudication,
}

#[derive(Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: [ChatMessage<'a>; 2],
    temperature: f32,
    max_tokens: u16,
    stream: bool,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    #[serde(default)]
    #[serde(rename = "id")]
    _id: Option<String>,
    #[serde(default)]
    #[serde(rename = "object")]
    _object: Option<String>,
    #[serde(default)]
    #[serde(rename = "created")]
    _created: Option<u64>,
    #[serde(default)]
    #[serde(rename = "model")]
    _model: Option<String>,
    #[serde(default)]
    #[serde(rename = "system_fingerprint")]
    _system_fingerprint: Option<String>,
    #[serde(default)]
    #[serde(rename = "usage")]
    _usage: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatChoice {
    message: ChatResponseMessage,
    #[serde(default)]
    #[serde(rename = "index")]
    _index: Option<u64>,
    #[serde(default)]
    #[serde(rename = "finish_reason")]
    _finish_reason: Option<String>,
    #[serde(default)]
    #[serde(rename = "logprobs")]
    _logprobs: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChatResponseMessage {
    content: serde_json::Value,
    #[serde(default)]
    #[serde(rename = "role")]
    _role: Option<String>,
    #[serde(default)]
    #[serde(rename = "refusal")]
    _refusal: Option<serde_json::Value>,
    #[serde(default)]
    #[serde(rename = "annotations")]
    _annotations: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NarrativeWirePayload {
    kind: WireResponseKind,
    text: String,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum WireResponseKind {
    Narrative,
}
