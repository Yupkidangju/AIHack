use crate::core::GameEvent;

use super::config::UiRuntimeConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEffectKind {
    DamageFlash,
    DeathPulse,
    TrapFlash,
    ProjectileTrail,
    ScrollPulse,
    /// [v0.2.0] Phase 19: 새로 보인 entity의 자동 라벨 effect.
    NewEntityLabel,
    Info,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiEffectEvent {
    pub id: u64,
    pub kind: UiEffectKind,
    pub ttl_ms: u16,
}

pub fn project_event(event: &GameEvent, next_id: u64) -> Option<UiEffectEvent> {
    project_event_with_config(event, next_id, &UiRuntimeConfig::default())
}

pub fn project_event_with_config(
    event: &GameEvent,
    next_id: u64,
    config: &UiRuntimeConfig,
) -> Option<UiEffectEvent> {
    let kind = match event {
        GameEvent::AttackResolved { .. } => UiEffectKind::DamageFlash,
        GameEvent::EntityDied { .. } => UiEffectKind::DeathPulse,
        GameEvent::TrapTriggered { .. } => UiEffectKind::TrapFlash,
        GameEvent::ItemThrown { .. } | GameEvent::WandZapped { .. } => {
            UiEffectKind::ProjectileTrail
        }
        GameEvent::ScrollRead { .. } => UiEffectKind::ScrollPulse,
        _ => UiEffectKind::Info,
    };
    let ttl_ms = match kind {
        UiEffectKind::DamageFlash => 120,
        UiEffectKind::DeathPulse => 180,
        UiEffectKind::TrapFlash => 140,
        UiEffectKind::ProjectileTrail => 100,
        UiEffectKind::ScrollPulse => 160,
        // [v0.2.0] Phase 19: 자동 라벨 effect 지속 시간은 1200ms.
        UiEffectKind::NewEntityLabel => 1200,
        UiEffectKind::Info => 80,
    };
    let ttl_ms = if config.reduced_motion {
        ttl_ms.min(40)
    } else {
        ttl_ms
    };
    Some(UiEffectEvent {
        id: next_id,
        kind,
        ttl_ms,
    })
}
