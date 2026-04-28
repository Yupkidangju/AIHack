// ============================================================================
// [v2.44.0 R32-2] 배고픔 브릿지 (hunger_bridge.rs)
// hunger_ext (순수 로직) ↔ Player 구조체 연결
// ============================================================================

use crate::core::entity::player::{HungerState, Player};
use crate::core::events::{EventQueue, GameEvent};
use crate::core::systems::creature::hunger_ext::{
    hunger_from_nutrition, nutrition_burn_rate, HungerLevel,
};
use crate::core::systems::turn_engine::TurnContext;

/// [v2.44.0 R32-2] HungerLevel → HungerState 변환
fn to_hunger_state(level: HungerLevel) -> HungerState {
    match level {
        HungerLevel::Satiated => HungerState::Satiated,
        HungerLevel::NotHungry => HungerState::NotHungry,
        HungerLevel::Hungry => HungerState::Hungry,
        HungerLevel::Weak => HungerState::Weak,
        HungerLevel::Fainting | HungerLevel::Fainted => HungerState::Fainting,
        HungerLevel::Starved => HungerState::Starved,
    }
}

/// [v2.44.0 R32-2] 매턴 영양 소모 + 상태 갱신 (원본: newuhs / gethungry)
pub fn tick_nutrition(ctx: &mut TurnContext) {
    let p = &mut ctx.player;

    // 짐 무게에 따른 소모율 (단순 근사: 5단계)
    let encumbrance = 0; // TODO: 실제 짐 무게 계산 연결
    let is_regen = false; // TODO: 재생 반지 체크 연결
    let ring_hunger = p.equip_hunger_bonus > 0;

    let burn = nutrition_burn_rate(encumbrance, is_regen, ring_hunger);
    p.nutrition = (p.nutrition - burn).max(-100);

    let prev_state = p.hunger;
    let new_level = hunger_from_nutrition(p.nutrition);
    let new_state = to_hunger_state(new_level);

    if new_state != prev_state {
        p.hunger = new_state;
        // 배고픔 상태 변화 이벤트
        let message = match new_state {
            HungerState::Hungry => "배가 고프기 시작한다.",
            HungerState::Weak => "몸이 허약해진다.",
            HungerState::Fainting => "쓰러질 것 같다...",
            HungerState::Starved => "굶어 죽었다!",
            _ => "",
        };
        if !message.is_empty() {
            ctx.event_queue.push(GameEvent::Message {
                text: message.to_string(),
                priority: matches!(new_state, HungerState::Starved | HungerState::Fainting),
            });
        }
    }
}

/// [v2.44.0 R32-2] HP/에너지 재생 (원본: regen)
pub fn tick_regeneration(ctx: &mut TurnContext) {
    let p = &mut ctx.player;

    // HP 재생: 레벨 5마다 1
    if ctx.turn_number % (20 / p.level.max(1) as u64).max(1) == 0 {
        if p.hp < p.hp_max {
            p.hp = (p.hp + 1).min(p.hp_max);
        }
    }

    // 에너지 재생: 지혜 기반
    let wis_bonus = (p.wis.base - 10).max(0) / 4;
    let regen_rate = 10 - wis_bonus;
    if ctx.turn_number % regen_rate.max(1) as u64 == 0 {
        if p.energy < p.energy_max {
            p.energy = (p.energy + 1).min(p.energy_max);
        }
    }
}

/// [v2.44.0 R32-2] 음식 섭취 (원본: doeat / eatcorpse)
pub fn eat_food(player: &mut Player, nutrition: i32, events: &mut EventQueue) {
    let will_overeat = player.nutrition + nutrition > 2500;
    player.nutrition = (player.nutrition + nutrition).min(2500);
    player.hunger = to_hunger_state(hunger_from_nutrition(player.nutrition));

    if will_overeat {
        events.push(GameEvent::Message {
            text: "너무 많이 먹었다!".to_string(),
            priority: false,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::events::EventQueue;
    use crate::core::systems::turn_engine::TurnContext;

    fn make_ctx(nutrition: i32) -> (Player, EventQueue) {
        let mut p = Player::new();
        p.nutrition = nutrition;
        let q = EventQueue::new();
        (p, q)
    }

    #[test]
    fn test_tick_reduces_nutrition() {
        let (mut p, mut q) = make_ctx(900);
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        tick_nutrition(&mut ctx);
        assert!(p.nutrition < 900);
    }

    #[test]
    fn test_hunger_state_change() {
        let (mut p, mut q) = make_ctx(60); // Hungry 경계
        p.nutrition = 40; // Weak 시작
        p.hunger = HungerState::Hungry;
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        tick_nutrition(&mut ctx);
        assert_eq!(p.hunger, HungerState::Weak);
    }

    #[test]
    fn test_eat_food() {
        let (mut p, mut q) = make_ctx(100);
        eat_food(&mut p, 800, &mut q);
        assert!(p.nutrition >= 900);
    }

    #[test]
    fn test_regen() {
        let (mut p, mut q) = make_ctx(900);
        p.hp = 10;
        p.hp_max = 20;
        p.level = 20;
        for t in 0..10 {
            let mut ctx = TurnContext {
                player: &mut p,
                turn_number: t,
                event_queue: &mut q,
            };
            tick_regeneration(&mut ctx);
        }
        assert!(p.hp > 10);
    }
}
