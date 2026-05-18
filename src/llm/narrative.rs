use std::{sync::Arc, time::Duration};

use crate::core::{NarrativeTopic, Observation};

pub const NARRATIVE_TIMEOUT_MS: u64 = 2_000;

/// [v0.1.0] Phase 12 narrative request envelope다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrativeRequest {
    pub topic: NarrativeTopic,
    pub observation: Observation,
}

/// [v0.1.0] narrative 응답 출처를 구분한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NarrativeSource {
    Provider,
    Fallback,
}

/// [v0.1.0] Phase 12 narrative 응답 envelope다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrativeResponse {
    pub text: String,
    pub source: NarrativeSource,
    pub timed_out: bool,
}

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
    if trimmed.is_empty() || trimmed.len() > 240 {
        return None;
    }
    if trimmed
        .chars()
        .any(|ch| ch.is_control() && ch != '\n' && ch != '\t')
    {
        return None;
    }
    Some(trimmed.to_string())
}

pub fn fallback_response(request: &NarrativeRequest, timed_out: bool) -> NarrativeResponse {
    let observation = &request.observation;
    let text = match request.topic {
        NarrativeTopic::SituationSummary => format!(
            "턴 {}: {}층에서 HP {}/{} 상태이며 보이는 대상 {}개를 관찰 중이다.",
            observation.turn,
            observation.current_level.depth,
            observation.player.hp,
            observation.player.max_hp,
            observation.visible_entities.len()
        ),
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
