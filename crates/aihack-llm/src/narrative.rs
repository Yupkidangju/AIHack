use std::{sync::Arc, time::Duration};

use aihack_ai_contract::{ClientRevision, NarrativeTopic, Observation};

use crate::is_forbidden_control;

pub use crate::config::DEFAULT_NARRATIVE_TIMEOUT_MS as NARRATIVE_TIMEOUT_MS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrativeRequest {
    pub revision: ClientRevision,
    pub topic: NarrativeTopic,
    pub observation: Observation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NarrativeSource {
    Provider,
    Fallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrativeResponse {
    pub text: String,
    pub source: NarrativeSource,
    pub timed_out: bool,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NarrativeError {
    Provider(String),
    Empty,
    Timeout,
    Invalid(String),
}

pub trait NarrativeProvider: Send + Sync {
    fn generate(
        &self,
        request: &NarrativeRequest,
        timeout: Duration,
    ) -> Result<String, NarrativeError>;
}

pub fn request_narrative(
    provider: Option<Arc<dyn NarrativeProvider>>,
    request: NarrativeRequest,
) -> NarrativeResponse {
    request_narrative_with_timeout(
        provider,
        request,
        Duration::from_millis(NARRATIVE_TIMEOUT_MS),
    )
}

pub fn request_narrative_with_timeout(
    provider: Option<Arc<dyn NarrativeProvider>>,
    request: NarrativeRequest,
    timeout: Duration,
) -> NarrativeResponse {
    let Some(provider) = provider else {
        return fallback_response(&request, false);
    };

    match provider.generate(&request, timeout) {
        Ok(text) => match sanitize_text(&text) {
            Some(text) => NarrativeResponse {
                text,
                source: NarrativeSource::Provider,
                timed_out: false,
            },
            None => fallback_response(&request, false),
        },
        Err(NarrativeError::Timeout) => fallback_response(&request, true),
        Err(_) => fallback_response(&request, false),
    }
}

fn sanitize_text(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 240 {
        return None;
    }
    if trimmed.chars().any(is_forbidden_control) {
        return None;
    }
    Some(trimmed.to_string())
}

pub fn fallback_response(request: &NarrativeRequest, timed_out: bool) -> NarrativeResponse {
    let observation = &request.observation;
    let text = match request.topic {
        NarrativeTopic::SituationSummary => {
            if let Some(campaign) = &observation.campaign {
                let loc = campaign
                    .special_level_id
                    .as_deref()
                    .unwrap_or_else(|| observation.current_level.branch.display_name());
                let amulet = if campaign.has_amulet { " [Amulet]" } else { "" };
                let feature = if let Some(align) = campaign.on_high_altar {
                    format!(" [High Altar {}]", align.short_label())
                } else if campaign.on_magic_portal {
                    " [Portal]".to_string()
                } else {
                    String::new()
                };
                format!(
                    "Turn {}: {} L{} at {}:{} ({}), HP {}/{}{}{}, {} visible.",
                    observation.turn,
                    campaign.role.display_name(),
                    campaign.level,
                    observation.current_level.branch.display_name(),
                    observation.current_level.depth,
                    loc,
                    observation.player.hp,
                    observation.player.max_hp,
                    amulet,
                    feature,
                    observation.visible_entities.len()
                )
            } else {
                format!(
                    "Turn {}: depth {}, HP {}/{}, observing {} visible entities.",
                    observation.turn,
                    observation.current_level.depth,
                    observation.player.hp,
                    observation.player.max_hp,
                    observation.visible_entities.len()
                )
            }
        }
    };
    NarrativeResponse {
        text,
        source: NarrativeSource::Fallback,
        timed_out,
    }
}

pub fn narrative_log_lines(response: &NarrativeResponse) -> Vec<String> {
    vec![
        match response.source {
            NarrativeSource::Provider => "Narrative(provider)".to_string(),
            NarrativeSource::Fallback if response.timed_out => {
                "Narrative(fallback-timeout)".to_string()
            }
            NarrativeSource::Fallback => "Narrative(fallback)".to_string(),
        },
        response.text.clone(),
    ]
}
