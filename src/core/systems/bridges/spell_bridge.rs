// ============================================================================
// [v2.22.0 R34-P2-3] 주문 브릿지 (spell_bridge.rs)
// spell_ext2 → 주문 시전 통합
// ============================================================================

use crate::core::entity::status::StatusFlags;
use crate::core::events::GameEvent;
use crate::core::systems::magic::spell_ext2::{self, SpellBackfireEffect, SpellSuccessInput};
use crate::core::systems::turn_engine::TurnContext;
use crate::util::rng::NetHackRng;

/// [v2.22.0 R34-P2-3] 주문 시전 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastResult {
    /// 성공
    Success { energy_cost: i32, hunger_cost: i32 },
    /// 실패 (역발)
    Backfire {
        effect: SpellBackfireEffect,
        energy_cost: i32,
    },
    /// 에너지 부족
    InsufficientEnergy { required: i32, available: i32 },
}

/// [v2.22.0 R34-P2-3] 주문 시전 처리
/// spell_ext2의 순수 함수들을 조합하여 주문 시전 전 과정 처리
pub fn cast_spell(
    ctx: &mut TurnContext,
    spell_level: i32,
    skill_level: i32,
    is_role_spell: bool,
    rng: &mut NetHackRng,
) -> CastResult {
    // [1] 에너지 비용 계산
    let energy_cost = spell_ext2::calc_spell_energy(spell_level, skill_level, is_role_spell);

    // [2] 에너지 부족 체크
    if ctx.player.energy < energy_cost {
        ctx.event_queue.push(GameEvent::Message {
            text: format!(
                "에너지가 부족하다! (필요: {}, 보유: {})",
                energy_cost, ctx.player.energy
            ),
            priority: true,
        });
        return CastResult::InsufficientEnergy {
            required: energy_cost,
            available: ctx.player.energy,
        };
    }

    // [3] 성공률 계산 (간소화된 입력 — 장비 정보는 TODO)
    let input = SpellSuccessInput {
        role_spelbase: 1,
        role_spelheal: 2,
        role_spelarmr: 10,
        role_spelshld: 1,
        role_spelsbon: -4,
        role_spelspec_id: -1, // 기본: 특수 주문 아님
        has_metallic_body_armor: false,
        has_robe: false,
        has_shield: false,
        has_heavy_shield: false,
        has_metallic_helmet: false,
        has_metallic_gloves: false,
        has_metallic_boots: false,
        spell_id: 0,
        spell_level,
        is_healing_spell: false,
        magic_stat: ctx.player.int.base.max(ctx.player.wis.base),
        player_level: ctx.player.level,
        skill_level,
    };
    let success_pct = spell_ext2::calc_spell_success(&input);

    // [4] 혼란 시 성공률 반감
    let effective_pct = if ctx.player.status_bundle.has(StatusFlags::CONFUSED) {
        success_pct / 2
    } else {
        success_pct
    };

    // [5] 성공/실패 판정
    let roll = rng.rn2(100);
    if roll < effective_pct {
        // 성공!
        let hunger_cost = spell_ext2::calc_spell_hunger(spell_level, skill_level);
        ctx.player.energy -= energy_cost;
        ctx.player.nutrition -= hunger_cost;

        ctx.event_queue.push(GameEvent::ItemUsed {
            item_name: format!("레벨 {} 주문", spell_level),
            use_type: "cast".to_string(),
        });
        ctx.event_queue.push(GameEvent::Message {
            text: format!("주문을 성공적으로 시전했다! (성공률: {}%)", effective_pct),
            priority: false,
        });

        CastResult::Success {
            energy_cost,
            hunger_cost,
        }
    } else {
        // 실패 → 역발
        let backfire = spell_ext2::determine_spell_backfire(spell_level, rng);
        ctx.player.energy -= energy_cost / 2; // 실패 시 절반

        // 역발 효과 적용
        match &backfire {
            SpellBackfireEffect::Confusion { duration } => {
                ctx.player.status_bundle.make_confused(*duration as u32);
                ctx.event_queue.push(GameEvent::Message {
                    text: format!("주문이 역발했다! {}턴간 혼란.", duration),
                    priority: true,
                });
            }
            SpellBackfireEffect::Damage { amount } => {
                ctx.player.hp -= amount;
                ctx.event_queue.push(GameEvent::DamageDealt {
                    attacker: "역발 주문".to_string(),
                    defender: "Player".to_string(),
                    amount: *amount,
                    source: "spell_backfire".to_string(),
                });
            }
            SpellBackfireEffect::EnergyLoss { amount } => {
                ctx.player.energy = (ctx.player.energy - amount).max(0);
                ctx.event_queue.push(GameEvent::Message {
                    text: format!("에너지가 {}만큼 추가 소실!", amount),
                    priority: true,
                });
            }
        }

        CastResult::Backfire {
            effect: backfire,
            energy_cost: energy_cost / 2,
        }
    }
}

/// [v2.22.0 R34-P2-3] 주문 기억 잔존율 조회
pub fn check_spell_memory(turns_left: i64, skill_level: i32) -> Option<(i64, i64)> {
    spell_ext2::calc_spell_retention(turns_left, skill_level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::entity::player::Player;
    use crate::core::events::EventQueue;

    #[test]
    fn test_cast_insufficient_energy() {
        let mut p = Player::new();
        p.energy = 1;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(0);

        let result = cast_spell(&mut ctx, 5, 0, false, &mut rng);
        assert!(matches!(result, CastResult::InsufficientEnergy { .. }));
    }

    #[test]
    fn test_cast_reduces_energy() {
        let mut p = Player::new();
        p.energy = 50;
        p.energy_max = 50;
        let initial_energy = p.energy;
        let mut q = EventQueue::new();
        let mut ctx = TurnContext {
            player: &mut p,
            turn_number: 1,
            event_queue: &mut q,
        };
        let mut rng = NetHackRng::new(42);

        let _result = cast_spell(&mut ctx, 1, 3, false, &mut rng);
        // 성공 또는 실패 모두 에너지 감소
        assert!(p.energy < initial_energy);
    }

    #[test]
    fn test_spell_memory_full() {
        let result = check_spell_memory(20000, 2);
        assert_eq!(result, Some((100, 100)));
    }

    #[test]
    fn test_spell_memory_expired() {
        let result = check_spell_memory(0, 1);
        assert_eq!(result, None);
    }
}
