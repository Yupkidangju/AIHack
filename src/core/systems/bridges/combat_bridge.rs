// ============================================================================
// [v2.44.0 R32-4] 전투 브릿지 (combat_bridge.rs)
// hit_calc_ext + elemental_ext + death_check_ext → GameEvent 연결
// ============================================================================

use crate::core::entity::player::Player;
use crate::core::entity::{CombatStats, Health, Monster};
use crate::core::events::{EventQueue, GameEvent};
use crate::core::systems::combat::elemental_ext::{elemental_damage, Element};
use crate::core::systems::combat::hit_calc_ext::{roll_damage, roll_to_hit};
use crate::core::systems::creature::death_check_ext::{check_death, DeathCause, DeathResult};
use crate::core::systems::turn_engine::{TurnContext, TurnResult};
use crate::util::rng::NetHackRng;

/// [v2.44.0 R32-4] 플레이어→몬스터 공격 처리
pub fn player_attacks_monster(
    player: &Player,
    monster_name: &str,
    monster_ac: i32,
    monster_health: &mut i32,
    weapon_dice: i32,
    weapon_sides: i32,
    weapon_bonus: i32,
    rng: &mut NetHackRng,
    events: &mut EventQueue,
) {
    let attack_bonus = player.level + (player.str.base - 10) / 2;
    if roll_to_hit(attack_bonus, monster_ac, player.luck, rng) {
        let damage = roll_damage(weapon_dice, weapon_sides, weapon_bonus, rng);
        *monster_health -= damage;
        events.push(GameEvent::DamageDealt {
            attacker: "Player".to_string(),
            defender: monster_name.to_string(),
            amount: damage,
            source: "melee".to_string(),
        });
    } else {
        events.push(GameEvent::AttackMissed {
            attacker: "Player".to_string(),
            defender: monster_name.to_string(),
        });
    }
}

/// [v2.44.0 R32-4] 몬스터→플레이어 공격 처리
pub fn monster_attacks_player(
    monster_name: &str,
    monster_level: i32,
    player: &mut Player,
    dice: i32,
    sides: i32,
    rng: &mut NetHackRng,
    events: &mut EventQueue,
) {
    if roll_to_hit(monster_level, player.ac, 0, rng) {
        let damage = roll_damage(dice, sides, 0, rng);
        player.hp -= damage;
        events.push(GameEvent::DamageDealt {
            attacker: monster_name.to_string(),
            defender: "Player".to_string(),
            amount: damage,
            source: "monster_melee".to_string(),
        });
    } else {
        events.push(GameEvent::AttackMissed {
            attacker: monster_name.to_string(),
            defender: "Player".to_string(),
        });
    }
}

/// [v2.44.0 R32-4] 원소 공격 처리
pub fn elemental_attack(
    element: Element,
    base_damage: i32,
    target_resists: bool,
    target_hp: &mut i32,
    target_name: &str,
    events: &mut EventQueue,
) {
    let damage = elemental_damage(element, base_damage, target_resists, false);
    if damage > 0 {
        *target_hp -= damage;
        events.push(GameEvent::DamageDealt {
            attacker: format!("{:?}", element),
            defender: target_name.to_string(),
            amount: damage,
            source: format!("{:?}", element),
        });
    }
}

/// [v2.44.0 R32-4] 플레이어 사망 판정 (TurnEngine DeathCheck phase용)
pub fn check_player_death(ctx: &mut TurnContext) -> Option<TurnResult> {
    let p = &ctx.player;
    let result = check_death(
        p.hp,
        DeathCause::Monster("unknown".to_string()),
        false, // TODO: 라이프세이빙 장비 체크 연결
        false,
    );
    match result {
        DeathResult::Dead { epitaph, .. } => {
            ctx.event_queue.push(GameEvent::PlayerDied {
                cause: epitaph.clone(),
            });
            Some(TurnResult::PlayerDied { cause: epitaph })
        }
        DeathResult::LifeSaved => {
            ctx.player.hp = 1;
            ctx.event_queue.push(GameEvent::Message {
                text: "라이프세이빙이 당신을 구했다!".to_string(),
                priority: true,
            });
            None
        }
        DeathResult::Resurrected { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::EventQueue;
    use crate::core::systems::turn_engine::TurnContext;

    #[test]
    fn test_player_attacks() {
        let p = Player::new();
        let mut mon_hp = 20;
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(42);
        player_attacks_monster(&p, "goblin", 5, &mut mon_hp, 1, 6, 2, &mut rng, &mut q);
        // 명중 or 미스 모두 이벤트 발생
        assert!(!q.is_empty());
    }

    #[test]
    fn test_monster_attacks() {
        let mut p = Player::new();
        let hp_before = p.hp;
        let mut q = EventQueue::new();
        let mut rng = NetHackRng::new(1);
        monster_attacks_player("orc", 3, &mut p, 1, 6, &mut rng, &mut q);
        assert!(!q.is_empty());
    }

    #[test]
    fn test_death_check_alive() {
        let mut p = Player::new(); // hp=15
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        assert!(check_player_death(&mut ctx).is_none());
    }

    #[test]
    fn test_death_check_dead() {
        let mut p = Player::new();
        p.hp = -5;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        assert!(matches!(
            check_player_death(&mut ctx),
            Some(TurnResult::PlayerDied { .. })
        ));
    }

    #[test]
    fn test_elemental_resist() {
        let mut hp = 20;
        let mut q = EventQueue::new();
        elemental_attack(Element::Fire, 10, true, &mut hp, "player", &mut q);
        assert_eq!(hp, 20); // 저항 시 데미지 0
    }
}
