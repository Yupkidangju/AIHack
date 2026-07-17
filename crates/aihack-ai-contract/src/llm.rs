use aihack_core::action::ActionIntent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionPayload {
    pub action: ActionIntent,
    pub rationale: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrativePayload {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SoftVerdict {
    Favorable,
    Neutral,
    Unfavorable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftAdjudicationPayload {
    pub verdict: SoftVerdict,
    pub reason_code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LlmPayload {
    Narrative(NarrativePayload),
    Decision(DecisionPayload),
    SoftAdjudication(SoftAdjudicationPayload),
}
