use std::{
    io::{ErrorKind, Read},
    net::ToSocketAddrs,
    time::Duration,
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
    service::{canonical_request_json, prepare_request_input, LlmRequestInput},
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
    UnsupportedSchema { expected: u16, actual: u16 },
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
        let mut input = LlmRequestInput::from_observation(
            request.revision.clone(),
            &request.observation,
            LlmRequestKind::Narrative,
        );
        prepare_request_input(&mut input).map_err(map_enqueue_error)?;
        match self.complete_input(&input)? {
            aihack_ai_contract::llm::LlmPayload::Narrative(payload) => Ok(payload.text),
            _ => Err(invalid(LlmValidationCode::WrongKind)),
        }
    }

    pub fn complete_decision(
        &self,
        request: &DecisionRequest,
    ) -> Result<DecisionPayload, LlmResponseError> {
        let mut input = LlmRequestInput::from_observation(
            request.revision.clone(),
            &request.observation,
            LlmRequestKind::Decision,
        );
        input.action_space = request.action_space.clone();
        prepare_request_input(&mut input).map_err(map_enqueue_error)?;
        match self.complete_input(&input)? {
            aihack_ai_contract::llm::LlmPayload::Decision(payload) => Ok(payload),
            _ => Err(invalid(LlmValidationCode::WrongKind)),
        }
    }

    pub fn complete_soft_adjudication(
        &self,
        request: &SoftAdjudicationRequest,
    ) -> Result<SoftAdjudicationPayload, LlmResponseError> {
        let mut input = LlmRequestInput::from_observation(
            request.revision.clone(),
            &request.observation,
            LlmRequestKind::SoftAdjudication {
                user_text: request.user_text.clone(),
            },
        );
        prepare_request_input(&mut input).map_err(map_enqueue_error)?;
        match self.complete_input(&input)? {
            aihack_ai_contract::llm::LlmPayload::SoftAdjudication(payload) => Ok(payload),
            _ => Err(invalid(LlmValidationCode::WrongKind)),
        }
    }

    fn complete_with_timeout(
        &self,
        request: &NarrativeRequest,
        timeout: Duration,
    ) -> Result<String, LlmResponseError> {
        let mut input = LlmRequestInput::from_observation(
            request.revision.clone(),
            &request.observation,
            LlmRequestKind::Narrative,
        );
        prepare_request_input(&mut input).map_err(map_enqueue_error)?;
        let content = self.complete_request(&input, timeout)?;
        parse_narrative_content(&content, self.config.max_output_chars())
    }

    pub(crate) fn complete_input(
        &self,
        input: &LlmRequestInput,
    ) -> Result<aihack_ai_contract::llm::LlmPayload, LlmResponseError> {
        let timeout = Duration::from_millis(self.config.request_timeout_ms(&input.kind));
        let content = self.complete_request(input, timeout)?;
        match &input.kind {
            LlmRequestKind::Narrative => {
                parse_narrative_content(&content, self.config.max_output_chars()).map(|text| {
                    aihack_ai_contract::llm::LlmPayload::Narrative(
                        aihack_ai_contract::llm::NarrativePayload { text },
                    )
                })
            }
            LlmRequestKind::Decision => {
                let payload = parse_decision_payload_json(&content, &input.action_space)?;
                let validated = validate_decision_payload(
                    input.revision.clone(),
                    payload,
                    &input.revision,
                    &input.action_space,
                )?;
                Ok(aihack_ai_contract::llm::LlmPayload::Decision(
                    DecisionPayload {
                        action: validated.action(),
                        rationale: validated.rationale().to_string(),
                        confidence: validated.confidence(),
                    },
                ))
            }
            LlmRequestKind::SoftAdjudication { .. } => {
                parse_soft_adjudication_payload_json(&content)
                    .map(aihack_ai_contract::llm::LlmPayload::SoftAdjudication)
            }
        }
    }

    fn complete_request(
        &self,
        input: &LlmRequestInput,
        timeout: Duration,
    ) -> Result<String, LlmResponseError> {
        if !self.config.enabled() {
            return Err(LlmResponseError::Disabled);
        }
        validate_resolved_loopback(&self.chat_completions_url)?;
        let canonical_input =
            canonical_request_json(input).map_err(|_| invalid(LlmValidationCode::InvalidJson))?;
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
                | LlmResponseError::UnsupportedSchema { .. }
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
        crate::config::LlmInputCode::PayloadTooLarge => LlmValidationCode::PayloadTooLarge,
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

fn map_enqueue_error(error: crate::worker::LlmEnqueueError) -> LlmResponseError {
    match error {
        crate::worker::LlmEnqueueError::UnsupportedSchema { expected, actual } => {
            LlmResponseError::UnsupportedSchema { expected, actual }
        }
        crate::worker::LlmEnqueueError::InvalidInput { code } => invalid(map_input_code(code)),
        crate::worker::LlmEnqueueError::Disabled => LlmResponseError::Disabled,
        crate::worker::LlmEnqueueError::Busy { .. }
        | crate::worker::LlmEnqueueError::InvalidEndpoint
        | crate::worker::LlmEnqueueError::InvalidModel
        | crate::worker::LlmEnqueueError::WorkerStopped => LlmResponseError::Unavailable,
    }
}

fn invalid(code: LlmValidationCode) -> LlmResponseError {
    LlmResponseError::InvalidSchema { code }
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
