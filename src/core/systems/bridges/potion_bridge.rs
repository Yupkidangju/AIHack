// ============================================================================
// [v2.45.0 R33-1] 포션 사용 브릿지 (potion_bridge.rs)
// potion_quaff_ext (순수 로직) → Player + GameEvent 연결
// ============================================================================

use crate::core::entity::player::Player;
use crate::core::events::{EventQueue, GameEvent};
use crate::core::systems::item::potion_quaff_ext::{quaff_potion, QuaffEffect};

/// [v2.45.0 R33-1] 포션 마시기 — 효과를 Player에 직접 적용
pub fn drink_potion(
    player: &mut Player,
    potion_name: &str,
    blessed: bool,
    cursed: bool,
    events: &mut EventQueue,
) {
    let effect = quaff_potion(potion_name, blessed, cursed);
    apply_quaff_effect(player, effect, events);
}

/// [v2.45.0 R33-1] QuaffEffect → Player 상태 적용
fn apply_quaff_effect(player: &mut Player, effect: QuaffEffect, events: &mut EventQueue) {
    match effect {
        QuaffEffect::Heal(amount) => {
            let old = player.hp;
            player.hp = (player.hp + amount).min(player.hp_max);
            events.push(GameEvent::HealthChanged {
                target: "Player".into(),
                old_hp: old,
                new_hp: player.hp,
                max_hp: player.hp_max,
            });
            events.push(GameEvent::Message {
                text: format!("{}HP 회복됐다.", player.hp - old),
                priority: false,
            });
        }
        QuaffEffect::FullHeal => {
            let old = player.hp;
            player.hp = player.hp_max;
            events.push(GameEvent::HealthChanged {
                target: "Player".into(),
                old_hp: old,
                new_hp: player.hp,
                max_hp: player.hp_max,
            });
        }
        QuaffEffect::GainLevel => {
            player.exp_level = (player.exp_level + 1).min(20);
            player.hp_max += 5;
            player.hp = player.hp_max;
            events.push(GameEvent::LevelUp {
                new_level: player.exp_level,
            });
        }
        QuaffEffect::Confusion(turns) => {
            events.push(GameEvent::StatusApplied {
                target: "Player".into(),
                status: crate::core::entity::status::StatusFlags::CONFUSED,
                turns: turns as u32,
            });
            events.push(GameEvent::Message {
                text: "머리가 빙글빙글 돈다!".into(),
                priority: false,
            });
        }
        QuaffEffect::Paralysis(turns) => {
            events.push(GameEvent::StatusApplied {
                target: "Player".into(),
                status: crate::core::entity::status::StatusFlags::PARALYZED,
                turns: turns as u32,
            });
            events.push(GameEvent::Message {
                text: "몸이 굳어버렸다!".into(),
                priority: true,
            });
        }
        QuaffEffect::Speed(turns) => {
            events.push(GameEvent::StatusApplied {
                target: "Player".into(),
                status: crate::core::entity::status::StatusFlags::FAST,
                turns: turns as u32,
            });
            events.push(GameEvent::Message {
                text: "몸이 매우 빠르다!".into(),
                priority: false,
            });
        }
        QuaffEffect::Blindness(turns) => {
            events.push(GameEvent::StatusApplied {
                target: "Player".into(),
                status: crate::core::entity::status::StatusFlags::BLIND,
                turns: turns as u32,
            });
        }
        QuaffEffect::Polymorph => {
            events.push(GameEvent::Message {
                text: "몸이 변한다!".into(),
                priority: true,
            });
        }
        QuaffEffect::Restore => {
            events.push(GameEvent::Message {
                text: "능력치가 회복됐다.".into(),
                priority: false,
            });
        }
        QuaffEffect::Nothing => {
            events.push(GameEvent::Message {
                text: "아무 일도 없었다.".into(),
                priority: false,
            });
        }
        _ => {
            events.push(GameEvent::Message {
                text: "효과가 발생했다.".into(),
                priority: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heal_potion() {
        let mut p = Player::new();
        p.hp = 5;
        let mut q = EventQueue::new();
        drink_potion(&mut p, "healing", false, false, &mut q);
        assert!(p.hp > 5);
    }

    #[test]
    fn test_full_heal() {
        let mut p = Player::new();
        p.hp = 1;
        let mut q = EventQueue::new();
        drink_potion(&mut p, "full healing", true, false, &mut q);
        assert_eq!(p.hp, p.hp_max);
    }

    #[test]
    fn test_gain_level() {
        let mut p = Player::new();
        let lvl = p.exp_level;
        let mut q = EventQueue::new();
        drink_potion(&mut p, "gain level", false, false, &mut q);
        assert_eq!(p.exp_level, lvl + 1);
        assert!(q.iter().any(|e| matches!(e, GameEvent::LevelUp { .. })));
    }

    #[test]
    fn test_health_event() {
        let mut p = Player::new();
        p.hp = 5;
        let mut q = EventQueue::new();
        drink_potion(&mut p, "healing", false, false, &mut q);
        assert!(q
            .iter()
            .any(|e| matches!(e, GameEvent::HealthChanged { .. })));
    }

    #[test]
    fn test_nothing() {
        let mut p = Player::new();
        let hp = p.hp;
        let mut q = EventQueue::new();
        drink_potion(&mut p, "unknown_xyz", false, false, &mut q);
        assert_eq!(p.hp, hp); // HP 변화 없음
    }
}
