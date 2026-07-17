use aihack_ai_contract::{
    llm::{SoftAdjudicationPayload, SoftVerdict},
    ClientRevision, Observation,
};
use serde::Deserialize;

use crate::{
    is_forbidden_control,
    transport::{LlmResponseError, LlmValidationCode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftAdjudicationRequest {
    pub revision: ClientRevision,
    pub observation: Observation,
    pub user_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftAdjudicationSource {
    Provider,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoftAdjudicationResponse {
    pub payload: SoftAdjudicationPayload,
    pub source: SoftAdjudicationSource,
}

pub fn parse_soft_adjudication_payload_json(
    content: &str,
) -> Result<SoftAdjudicationPayload, LlmResponseError> {
    let wire: SoftAdjudicationWirePayload =
        serde_json::from_str(content).map_err(|_| invalid(LlmValidationCode::InvalidJson))?;
    if wire.kind != SoftAdjudicationWireKind::SoftAdjudication {
        return Err(invalid(LlmValidationCode::WrongKind));
    }

    let reason_code = wire.reason_code.as_str();
    if !valid_reason_code(reason_code) {
        return Err(invalid(LlmValidationCode::InvalidReasonCode));
    }
    let message = wire.message.trim();
    if message.is_empty() {
        return Err(invalid(LlmValidationCode::EmptyText));
    }
    if message.chars().count() > 240 {
        return Err(invalid(LlmValidationCode::TextTooLong));
    }
    if message.chars().any(is_forbidden_control) {
        return Err(invalid(LlmValidationCode::ControlCharacter));
    }

    Ok(SoftAdjudicationPayload {
        verdict: wire.verdict,
        reason_code: reason_code.to_string(),
        message: message.to_string(),
    })
}

pub fn fallback_soft_adjudication() -> SoftAdjudicationResponse {
    SoftAdjudicationResponse {
        payload: SoftAdjudicationPayload {
            verdict: SoftVerdict::Neutral,
            reason_code: "LLM_UNAVAILABLE".to_string(),
            message: "Local adjudicator unavailable; no core rule effect is applied.".to_string(),
        },
        source: SoftAdjudicationSource::Fallback,
    }
}

pub fn soft_adjudication_lines(response: &SoftAdjudicationResponse) -> Vec<String> {
    vec![
        format!(
            "Soft judgment · {} · {}",
            verdict_label(response.payload.verdict),
            response.payload.reason_code
        ),
        response.payload.message.clone(),
        "[N] Dismiss".to_string(),
    ]
}

fn valid_reason_code(reason_code: &str) -> bool {
    (1..=32).contains(&reason_code.len())
        && reason_code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn verdict_label(verdict: SoftVerdict) -> &'static str {
    match verdict {
        SoftVerdict::Favorable => "Favorable",
        SoftVerdict::Neutral => "Neutral",
        SoftVerdict::Unfavorable => "Unfavorable",
    }
}

fn invalid(code: LlmValidationCode) -> LlmResponseError {
    LlmResponseError::InvalidSchema { code }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SoftAdjudicationWirePayload {
    kind: SoftAdjudicationWireKind,
    verdict: SoftVerdict,
    reason_code: String,
    message: String,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SoftAdjudicationWireKind {
    Narrative,
    Decision,
    SoftAdjudication,
}
