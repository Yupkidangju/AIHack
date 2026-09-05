use std::env;

use crate::is_forbidden_control;

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434/v1";
pub const DEFAULT_CONNECT_TIMEOUT_MS: u64 = 500;
pub const DEFAULT_NARRATIVE_TIMEOUT_MS: u64 = 2_000;
pub const DEFAULT_DECISION_TIMEOUT_MS: u64 = 1_500;
const DEFAULT_MAX_OUTPUT_CHARS: usize = 240;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalLlmConfig {
    enabled: bool,
    base_url: String,
    model: String,
    connect_timeout_ms: u64,
    narrative_timeout_ms: u64,
    decision_timeout_ms: u64,
    max_output_chars: usize,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmRequestKind {
    Narrative,
    Decision,
    SoftAdjudication { user_text: String },
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmConfigError {
    InvalidBoolean { name: String },
    InvalidRange { name: String, min: u64, max: u64 },
    MissingModel,
    InvalidEndpoint,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmInputCode {
    EmptyUserText,
    TextTooLong,
    ControlCharacter,
    PayloadTooLarge,
}

pub fn validate_user_text(text: &str) -> Result<String, LlmInputCode> {
    let text = text.trim();
    if text.is_empty() {
        return Err(LlmInputCode::EmptyUserText);
    }
    if text.chars().count() > 240 {
        return Err(LlmInputCode::TextTooLong);
    }
    if text.chars().any(is_forbidden_control) {
        return Err(LlmInputCode::ControlCharacter);
    }
    Ok(text.to_string())
}

impl LocalLlmConfig {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            base_url: DEFAULT_ENDPOINT.to_string(),
            model: String::new(),
            connect_timeout_ms: DEFAULT_CONNECT_TIMEOUT_MS,
            narrative_timeout_ms: DEFAULT_NARRATIVE_TIMEOUT_MS,
            decision_timeout_ms: DEFAULT_DECISION_TIMEOUT_MS,
            max_output_chars: DEFAULT_MAX_OUTPUT_CHARS,
        }
    }

    pub fn from_env() -> Result<Self, LlmConfigError> {
        let enabled = parse_bool("AIHACK_LLM_ENABLED", false)?;
        let base_url =
            env::var("AIHACK_LLM_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
        validate_endpoint(&base_url)?;

        let model = env::var("AIHACK_LLM_MODEL").unwrap_or_default();
        let model_chars = model.chars().count();
        if enabled && model_chars == 0 {
            return Err(LlmConfigError::MissingModel);
        }
        if model_chars > 128 {
            return Err(LlmConfigError::InvalidRange {
                name: "AIHACK_LLM_MODEL".to_string(),
                min: 1,
                max: 128,
            });
        }

        let connect_timeout_ms = parse_range(
            "AIHACK_LLM_CONNECT_TIMEOUT_MS",
            DEFAULT_CONNECT_TIMEOUT_MS,
            100,
            5_000,
        )?;
        let narrative_timeout_ms = parse_range(
            "AIHACK_LLM_NARRATIVE_TIMEOUT_MS",
            DEFAULT_NARRATIVE_TIMEOUT_MS,
            100,
            10_000,
        )?;
        let decision_timeout_ms = parse_range(
            "AIHACK_LLM_DECISION_TIMEOUT_MS",
            DEFAULT_DECISION_TIMEOUT_MS,
            100,
            10_000,
        )?;
        let max_output_chars = parse_range(
            "AIHACK_LLM_MAX_CHARS",
            DEFAULT_MAX_OUTPUT_CHARS as u64,
            1,
            240,
        )? as usize;

        Ok(Self {
            enabled,
            base_url,
            model,
            connect_timeout_ms,
            narrative_timeout_ms,
            decision_timeout_ms,
            max_output_chars,
        })
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn endpoint(&self) -> &str {
        &self.base_url
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn connect_timeout_ms(&self) -> u64 {
        self.connect_timeout_ms
    }

    pub fn request_timeout_ms(&self, kind: &LlmRequestKind) -> u64 {
        match kind {
            LlmRequestKind::Narrative => self.narrative_timeout_ms,
            LlmRequestKind::Decision | LlmRequestKind::SoftAdjudication { .. } => {
                self.decision_timeout_ms
            }
        }
    }

    pub fn max_output_chars(&self) -> usize {
        self.max_output_chars
    }
}

pub(crate) fn validate_endpoint(endpoint: &str) -> Result<reqwest::Url, LlmConfigError> {
    let url = reqwest::Url::parse(endpoint).map_err(|_| LlmConfigError::InvalidEndpoint)?;
    let host_is_allowed = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    let has_credentials = !url.username().is_empty() || url.password().is_some();
    let has_explicit_port = url.port().is_some();

    if url.scheme() != "http"
        || !host_is_allowed
        || !has_explicit_port
        || has_credentials
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(LlmConfigError::InvalidEndpoint);
    }
    Ok(url)
}

fn parse_bool(name: &str, default: bool) -> Result<bool, LlmConfigError> {
    match env::var(name) {
        Ok(value) if value == "true" => Ok(true),
        Ok(value) if value == "false" => Ok(false),
        Ok(_) => Err(LlmConfigError::InvalidBoolean {
            name: name.to_string(),
        }),
        Err(_) => Ok(default),
    }
}

fn parse_range(name: &str, default: u64, min: u64, max: u64) -> Result<u64, LlmConfigError> {
    let value = match env::var(name) {
        Ok(value) => value
            .parse::<u64>()
            .map_err(|_| LlmConfigError::InvalidRange {
                name: name.to_string(),
                min,
                max,
            })?,
        Err(_) => default,
    };
    if !(min..=max).contains(&value) {
        return Err(LlmConfigError::InvalidRange {
            name: name.to_string(),
            min,
            max,
        });
    }
    Ok(value)
}
