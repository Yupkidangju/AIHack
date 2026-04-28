// ============================================================================
// [v2.44.0 R32-3] 운/정렬 브릿지 (luck_align_bridge.rs)
// luck_ext + alignment_ext → Player 구조체 연결
// ============================================================================

use crate::core::entity::player::Player;
use crate::core::events::{EventQueue, GameEvent};
use crate::core::systems::misc::luck_ext::{clamp_luck, luck_decay};
use crate::core::systems::social::alignment_ext::{align_delta, god_angry, AlignAction};
use crate::core::systems::turn_engine::TurnContext;

/// [v2.44.0 R32-3] 매턴 운 감쇠 (원본: exerchk 내 luck 처리)
pub fn tick_luck(ctx: &mut TurnContext) {
    let p = &mut ctx.player;
    // 럭스톤 여부는 장비 시스템이 없으므로 일단 false (TODO: 장비 연결)
    let has_luckstone = false;
    p.luck_turns -= 1;
    if p.luck_turns <= 0 {
        p.luck = luck_decay(p.luck, has_luckstone);
        p.luck_turns = 600; // 감쇠 주기 리셋 (NetHack: 600턴마다)
    }
    p.luck = clamp_luck(p.luck, has_luckstone);
}

/// [v2.44.0 R32-3] 정렬 기록 변동 적용 (원본: adjalign)
pub fn apply_align_action(player: &mut Player, action: AlignAction, events: &mut EventQueue) {
    let delta = align_delta(&action);
    player.alignment_record = (player.alignment_record + delta).clamp(-128, 127);

    // 신 분노 체크
    if god_angry(player.alignment_record) {
        events.push(GameEvent::Message {
            text: "신이 당신에게 분노하고 있다.".to_string(),
            priority: true,
        });
    }
}

/// [v2.44.0 R32-3] 기도 쿨다운 갱신
pub fn tick_prayer_cooldown(player: &mut Player) {
    if player.prayer_cooldown > 0 {
        player.prayer_cooldown -= 1;
    }
}

/// [v2.44.0 R32-3] 신앙도(piety) 봉헌 시 증가
pub fn add_piety(player: &mut Player, amount: i32) {
    player.piety = (player.piety + amount).clamp(0, 100);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::EventQueue;
    use crate::core::systems::turn_engine::TurnContext;

    #[test]
    fn test_luck_decay_after_timeout() {
        let mut p = Player::new();
        p.luck = 5;
        p.luck_turns = 1; // 즉시 감쇠
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        tick_luck(&mut ctx);
        assert_eq!(p.luck, 4); // 1 감소
    }

    #[test]
    fn test_align_steal() {
        let mut p = Player::new();
        p.alignment_record = 0;
        let mut q = EventQueue::new();
        apply_align_action(&mut p, AlignAction::StealFromShop, &mut q);
        assert_eq!(p.alignment_record, -5);
    }

    #[test]
    fn test_god_angry_event() {
        let mut p = Player::new();
        p.alignment_record = -8;
        let mut q = EventQueue::new();
        apply_align_action(&mut p, AlignAction::StealFromShop, &mut q);
        assert!(!q.is_empty()); // 분노 메시지 발생
    }

    #[test]
    fn test_prayer_cooldown() {
        let mut p = Player::new();
        p.prayer_cooldown = 300;
        tick_prayer_cooldown(&mut p);
        assert_eq!(p.prayer_cooldown, 299);
    }

    #[test]
    fn test_piety_clamp() {
        let mut p = Player::new();
        add_piety(&mut p, 200);
        assert_eq!(p.piety, 100);
    }
}
