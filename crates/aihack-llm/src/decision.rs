use std::{sync::Arc, time::Duration};

use aihack_ai_contract::{
    ActionIntent, ActionSpace, ClientRevision, CommandIntent, Direction, Observation,
};
use serde::Deserialize;
use serde_json::{json, Value};

pub use aihack_ai_contract::llm::DecisionPayload;

use crate::{
    is_forbidden_control,
    transport::{LlmResponseError, LlmValidationCode},
    worker::RequestId,
};

pub use crate::config::DEFAULT_DECISION_TIMEOUT_MS as DECISION_TIMEOUT_MS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRequest {
    pub revision: ClientRevision,
    pub observation: Observation,
    pub action_space: ActionSpace,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionGateError {
    AlreadyOutstanding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionResponseEnvelope {
    request_id: RequestId,
    revision: ClientRevision,
    payload: DecisionPayload,
}

impl DecisionResponseEnvelope {
    pub fn new(request_id: RequestId, revision: ClientRevision, payload: DecisionPayload) -> Self {
        Self {
            request_id,
            revision,
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedDecision {
    revision: ClientRevision,
    action: ActionIntent,
    rationale: String,
    confidence: f32,
}

impl ValidatedDecision {
    pub fn revision(&self) -> &ClientRevision {
        &self.revision
    }

    pub fn action(&self) -> ActionIntent {
        self.action
    }

    pub fn rationale(&self) -> &str {
        &self.rationale
    }

    pub fn confidence(&self) -> f32 {
        self.confidence
    }
}

#[derive(Debug, Default)]
pub struct DecisionGate {
    outstanding: Option<(RequestId, ClientRevision)>,
}

impl DecisionGate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn begin(&mut self, request: &DecisionRequest) -> Result<RequestId, DecisionGateError> {
        if self.outstanding.is_some() {
            return Err(DecisionGateError::AlreadyOutstanding);
        }
        let request_id = RequestId::new();
        self.outstanding = Some((request_id.clone(), request.revision.clone()));
        Ok(request_id)
    }

    pub fn validate(
        &mut self,
        envelope: DecisionResponseEnvelope,
        current_revision: &ClientRevision,
        current_action_space: &ActionSpace,
    ) -> Result<ValidatedDecision, LlmResponseError> {
        let expected_revision = {
            let Some((expected_id, expected_revision)) = self.outstanding.as_ref() else {
                return Err(invalid(LlmValidationCode::UnknownRequestId));
            };
            if envelope.request_id != *expected_id {
                return Err(invalid(LlmValidationCode::UnknownRequestId));
            }
            expected_revision.clone()
        };

        self.outstanding.take();
        if envelope.revision != expected_revision {
            return Err(LlmResponseError::Stale);
        }
        validate_decision_payload(
            envelope.revision,
            envelope.payload,
            current_revision,
            current_action_space,
        )
    }

    pub fn complete_error(&mut self, request_id: &RequestId) -> Result<(), LlmResponseError> {
        let Some((expected_id, _)) = self.outstanding.as_ref() else {
            return Err(invalid(LlmValidationCode::UnknownRequestId));
        };
        if request_id != expected_id {
            return Err(invalid(LlmValidationCode::UnknownRequestId));
        }
        self.outstanding.take();
        Ok(())
    }
}

pub fn validate_decision_payload(
    revision: ClientRevision,
    payload: DecisionPayload,
    current_revision: &ClientRevision,
    current_action_space: &ActionSpace,
) -> Result<ValidatedDecision, LlmResponseError> {
    if revision != *current_revision {
        return Err(LlmResponseError::Stale);
    }
    if !current_action_space.commands.contains(&payload.action) {
        return Err(invalid(LlmValidationCode::InvalidAction));
    }
    if !payload.confidence.is_finite() || !(0.0..=1.0).contains(&payload.confidence) {
        return Err(invalid(LlmValidationCode::InvalidConfidence));
    }
    let rationale = payload.rationale.trim();
    if rationale.is_empty() {
        return Err(LlmResponseError::InvalidSchema {
            code: LlmValidationCode::EmptyText,
        });
    }
    if rationale.chars().count() > 160 {
        return Err(invalid(LlmValidationCode::TextTooLong));
    }
    if rationale.chars().any(is_forbidden_control) {
        return Err(invalid(LlmValidationCode::ControlCharacter));
    }
    Ok(ValidatedDecision {
        revision: current_revision.clone(),
        action: payload.action,
        rationale: rationale.to_string(),
        confidence: payload.confidence,
    })
}

fn invalid(code: LlmValidationCode) -> LlmResponseError {
    LlmResponseError::InvalidSchema { code }
}

pub fn parse_decision_payload_json(
    content: &str,
    request_action_space: &ActionSpace,
) -> Result<DecisionPayload, LlmResponseError> {
    let wire: DecisionWirePayload =
        serde_json::from_str(content).map_err(|_| invalid(LlmValidationCode::InvalidJson))?;
    if wire.kind != DecisionWireKind::Decision {
        return Err(invalid(LlmValidationCode::WrongKind));
    }
    let action = request_action_space
        .commands
        .iter()
        .copied()
        .find(|action| wire_action_value(*action) == wire.action)
        .ok_or_else(|| invalid(LlmValidationCode::InvalidAction))?;
    Ok(DecisionPayload {
        action,
        rationale: wire.rationale,
        confidence: wire.confidence,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DecisionWirePayload {
    kind: DecisionWireKind,
    action: Value,
    rationale: String,
    confidence: f32,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum DecisionWireKind {
    Narrative,
    Decision,
    SoftAdjudication,
}

fn wire_action_value(action: ActionIntent) -> Value {
    match action {
        ActionIntent::Command(command) => wire_command_value(command),
        ActionIntent::NarrativeRequest { topic } => json!({
            "type": "NARRATIVE_REQUEST",
            "topic": match topic {
                aihack_ai_contract::NarrativeTopic::SituationSummary => "SITUATION_SUMMARY",
            },
        }),
        ActionIntent::Noop => json!({ "type": "NOOP" }),
    }
}

fn wire_command_value(command: CommandIntent) -> Value {
    match command {
        CommandIntent::Wait => json!({ "type": "WAIT" }),
        CommandIntent::Quit => json!({ "type": "QUIT" }),
        CommandIntent::Move(direction) => direction_action("MOVE", direction),
        CommandIntent::Search => json!({ "type": "SEARCH" }),
        CommandIntent::Kick(direction) => direction_action("KICK", direction),
        CommandIntent::Open(direction) => direction_action("OPEN", direction),
        CommandIntent::Close(direction) => direction_action("CLOSE", direction),
        CommandIntent::Pickup => json!({ "type": "PICKUP" }),
        CommandIntent::Drop { item } => item_action("DROP", item.0),
        CommandIntent::Throw { item, direction } => json!({
            "type": "THROW",
            "item": item.0,
            "direction": direction_name(direction),
        }),
        CommandIntent::ShowInventory => json!({ "type": "SHOW_INVENTORY" }),
        CommandIntent::Wield { item } => item_action("WIELD", item.0),
        CommandIntent::Wear { item } => item_action("WEAR", item.0),
        CommandIntent::Quaff { item } => item_action("QUAFF", item.0),
        CommandIntent::Eat { item } => item_action("EAT", item.0),
        CommandIntent::Zap { item, direction } => json!({
            "type": "ZAP",
            "item": item.0,
            "direction": direction_name(direction),
        }),
        CommandIntent::Read { item } => item_action("READ", item.0),
        CommandIntent::Pray => json!({ "type": "PRAY" }),
        CommandIntent::Descend => json!({ "type": "DESCEND" }),
        CommandIntent::Ascend => json!({ "type": "ASCEND" }),
        CommandIntent::AcknowledgeMore => json!({ "type": "ACKNOWLEDGE_MORE" }),
    }
}

fn direction_action(kind: &str, direction: Direction) -> Value {
    json!({ "type": kind, "direction": direction_name(direction) })
}

fn item_action(kind: &str, item: u32) -> Value {
    json!({ "type": kind, "item": item })
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::North => "NORTH",
        Direction::NorthEast => "NORTH_EAST",
        Direction::East => "EAST",
        Direction::SouthEast => "SOUTH_EAST",
        Direction::South => "SOUTH",
        Direction::SouthWest => "SOUTH_WEST",
        Direction::West => "WEST",
        Direction::NorthWest => "NORTH_WEST",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionSource {
    Provider,
    Fallback,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestedAction {
    pub action: ActionIntent,
    pub rationale: String,
    pub source: DecisionSource,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionError {
    Provider(String),
    Timeout,
    Invalid(String),
}

pub trait DecisionProvider: Send + Sync {
    fn suggest(
        &self,
        request: &DecisionRequest,
        timeout: Duration,
    ) -> Result<SuggestedAction, DecisionError>;
}

pub fn request_decision(
    provider: Option<Arc<dyn DecisionProvider>>,
    request: DecisionRequest,
) -> SuggestedAction {
    request_decision_with_timeout(
        provider,
        request,
        Duration::from_millis(DECISION_TIMEOUT_MS),
    )
}

pub fn request_decision_with_timeout(
    provider: Option<Arc<dyn DecisionProvider>>,
    request: DecisionRequest,
    timeout: Duration,
) -> SuggestedAction {
    let Some(provider) = provider else {
        return fallback_suggestion(&request, DecisionSource::Disabled);
    };
    match provider.suggest(&request, timeout) {
        Ok(suggestion) if is_legal_suggestion(&request, &suggestion) => suggestion,
        Ok(_) | Err(_) => fallback_suggestion(&request, DecisionSource::Fallback),
    }
}

pub fn is_legal_suggestion(request: &DecisionRequest, suggestion: &SuggestedAction) -> bool {
    request.action_space.commands.contains(&suggestion.action)
}

pub fn fallback_suggestion(request: &DecisionRequest, source: DecisionSource) -> SuggestedAction {
    let action = request
        .action_space
        .commands
        .iter()
        .copied()
        .find(|action| {
            matches!(
                action,
                ActionIntent::Command(CommandIntent::Wait) | ActionIntent::Noop
            )
        })
        .or_else(|| request.action_space.commands.first().copied())
        .unwrap_or(ActionIntent::Noop);
    SuggestedAction {
        action,
        rationale: "deterministic fallback suggestion".to_string(),
        source,
    }
}

pub fn decision_log_lines(suggestion: &SuggestedAction, accepted: Option<bool>) -> Vec<String> {
    let status = match accepted {
        Some(true) => "accepted",
        Some(false) => "rejected",
        None => "suggested",
    };
    vec![
        format!("Decision({:?}, {status})", suggestion.source),
        format!("{:?} :: {}", suggestion.action, suggestion.rationale),
    ]
}
