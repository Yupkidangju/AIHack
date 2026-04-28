// ============================================================================
// [v2.22.0 R34-P2-1] 상태 타이머 브릿지 (status_bridge.rs)
// StatusBundle.tick_effects + timeout_ext → TurnEngine StatusTimers 페이즈 연결
// ============================================================================

use crate::core::entity::status::{StatusEffect, StatusFlags};
use crate::core::events::{EventQueue, GameEvent};
use crate::core::systems::turn_engine::TurnContext;
use crate::core::systems::world::timeout_ext;

/// [v2.22.0 R34-P2-1] StatusEffect → 만료 메시지 매핑
fn expiry_message(effect: StatusEffect) -> &'static str {
    match effect {
        StatusEffect::Blind => "시야가 돌아왔다!",
        StatusEffect::Confused => "혼란이 걷혔다.",
        StatusEffect::Stunned => "기절에서 회복했다.",
        StatusEffect::Hallucinating => "환각이 사라졌다.",
        StatusEffect::Paralyzed => "다시 움직일 수 있다!",
        StatusEffect::Sleeping => "잠에서 깨어났다.",
        StatusEffect::Stoning => "석화가 완료되었다!",
        StatusEffect::Slimed => "슬라임화가 완료되었다!",
        StatusEffect::Strangled => "숨을 쉴 수 있게 되었다!",
        StatusEffect::Levitating => "부유가 끝났다.",
        StatusEffect::Fast => "가속이 끝났다.",
        StatusEffect::Slow => "감속이 풀렸다.",
        StatusEffect::Sick => "병이 나았다.",
        StatusEffect::Poisoned => "독이 해소되었다.",
        StatusEffect::Vomiting => "구역질이 멈췄다.",
        StatusEffect::Choking => "숨통이 풀렸다.",
        StatusEffect::FoodPoisoning => "식중독이 나았다.",
        StatusEffect::Lycanthropy => "수인병이 나았다.",
        StatusEffect::Flying => "비행이 끝났다.",
        StatusEffect::Phasing => "통벽이 끝났다.",
        _ => "상태이상이 해제되었다.",
    }
}

/// [v2.22.0 R34-P2-1] 상태 타이머 턴 처리 (TurnEngine StatusTimers 페이즈)
///
/// StatusBundle을 사용하여 상태 타이머를 감소시키고,
/// 만료된 효과에 대해 이벤트를 발행합니다.
pub fn tick_status_timers(ctx: &mut TurnContext) {
    // [1] 석화 진행 중인 경우 → 단계별 메시지 (tick 전에 확인)
    if ctx.player.status_bundle.has_effect(StatusEffect::Stoning) {
        // 석화 타이머에서 남은 턴 수 조회
        if let Some(remaining) = ctx
            .player
            .status_bundle
            .active
            .iter()
            .find(|s| s.flag == StatusFlags::STONING)
            .map(|s| s.remaining_turns as i32)
        {
            let stage = timeout_ext::stoned_stage(remaining);
            if stage != timeout_ext::StonedStage::None {
                ctx.event_queue.push(GameEvent::Message {
                    text: format!("석화 진행: {:?}", stage),
                    priority: true,
                });
            }
        }
    }

    // [2] 질식 진행 중인 경우 → 단계별 메시지
    if ctx.player.status_bundle.has_effect(StatusEffect::Strangled) {
        if let Some(remaining) = ctx
            .player
            .status_bundle
            .active
            .iter()
            .find(|s| s.flag == StatusFlags::STRANGLED)
            .map(|s| s.remaining_turns as i32)
        {
            let stage = timeout_ext::choke_stage(remaining);
            if stage != timeout_ext::ChokeStage::None {
                ctx.event_queue.push(GameEvent::Message {
                    text: format!("질식 진행: {:?}", stage),
                    priority: true,
                });
            }
        }
    }

    // [3] StatusBundle.tick_effects() 호출 → 만료된 효과 목록
    let expired_effects = ctx.player.status_bundle.tick_effects();

    // [4] 만료된 효과에 대해 이벤트 발행
    for effect in &expired_effects {
        // StatusExpired 이벤트 (기존 형식: target + StatusFlags)
        ctx.event_queue.push(GameEvent::StatusExpired {
            target: "Player".to_string(),
            status: effect.to_flag(),
        });

        // 만료 메시지
        ctx.event_queue.push(GameEvent::Message {
            text: expiry_message(*effect).to_string(),
            priority: effect.is_dangerous(),
        });
    }

    // [5] 치명적 상태 만료 = 사망 처리 (DeathCheck 페이즈에서 감지)
    for effect in &expired_effects {
        match effect {
            StatusEffect::Stoning => {
                ctx.player.hp = -1; // 석화 사망
            }
            StatusEffect::Slimed => {
                ctx.player.hp = -1; // 슬라임화 사망
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::player::Player;
    use crate::core::entity::status::StatusEffect as SE;
    use crate::core::events::EventQueue;

    #[test]
    fn test_tick_expires_and_emits_event() {
        let mut p = Player::new();
        p.status_bundle.add_effect(SE::Blind, 1); // 1턴 남음
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };

        tick_status_timers(&mut ctx);

        // 실명 만료됨
        assert!(!p.status_bundle.has_effect(SE::Blind));
        // StatusExpired 이벤트 발행
        let has_expired = q.iter().any(|e| {
            matches!(e, GameEvent::StatusExpired { status, .. } if status.contains(StatusFlags::BLIND))
        });
        assert!(has_expired);
    }

    #[test]
    fn test_stoning_death() {
        let mut p = Player::new();
        p.status_bundle.add_effect(SE::Stoning, 1); // 1턴 후 석화
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };

        tick_status_timers(&mut ctx);

        // 석화 만료 → 사망
        assert!(p.hp <= 0);
    }

    #[test]
    fn test_multiple_effects_partial_expire() {
        let mut p = Player::new();
        p.status_bundle.add_effect(SE::Confused, 3); // 3턴 남음
        p.status_bundle.add_effect(SE::Fast, 1); // 1턴 남음
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };

        tick_status_timers(&mut ctx);

        // 혼란은 아직 활성, 가속은 만료
        assert!(p.status_bundle.has_effect(SE::Confused));
        assert!(!p.status_bundle.has_effect(SE::Fast));
    }

    #[test]
    fn test_no_timers_no_events() {
        let mut p = Player::new();
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };

        tick_status_timers(&mut ctx);

        assert!(q.is_empty());
    }
}
