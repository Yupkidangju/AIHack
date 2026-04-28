// ============================================================================
// [v2.27.0 Phase 91-9] 도적/훔치기 확장 (steal_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/steal.c L400-900 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 아이템 훔치기 판정 — steal_check (steal.c L400-600)
// =============================================================================

/// [v2.27.0 91-9] 훔치기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StealResult {
    /// 훔치기 성공
    Stolen { item_name: String, slot: String },
    /// 훔칠 아이템 없음
    NothingToSteal,
    /// 플레이어가 회피
    Evaded { message: String },
    /// 저주받은 장비라 못 훔침
    CursedBlock { slot: String },
    /// 인벤토리에서 금화 훔침
    GoldStolen { amount: i32 },
}

/// [v2.27.0 91-9] 훔치기 판정
/// 원본: steal.c steal()
pub fn steal_attempt(
    stealer_level: i32,
    player_dex: i32,
    player_luck: i32,
    has_items: bool,
    has_gold: i32,
    equipped_slots: &[(String, bool)], // (슬롯명, 저주여부)
    rng: &mut NetHackRng,
) -> StealResult {
    if !has_items && has_gold <= 0 && equipped_slots.is_empty() {
        return StealResult::NothingToSteal;
    }

    // 회피 확률: DEX + luck vs stealer_level
    let evade_chance = player_dex + player_luck * 2 - stealer_level;
    if evade_chance > 0 && rng.rn2(20) < evade_chance.min(15) {
        return StealResult::Evaded {
            message: "도둑의 손길을 피했다!".to_string(),
        };
    }

    // 금화 우선 (50% 확률)
    if has_gold > 0 && rng.rn2(2) == 0 {
        let stolen_amount = (has_gold / 3).max(1).min(has_gold);
        return StealResult::GoldStolen {
            amount: stolen_amount,
        };
    }

    // 장비 훔치기
    if !equipped_slots.is_empty() {
        let target = rng.rn2(equipped_slots.len() as i32) as usize;
        let (ref slot, cursed) = equipped_slots[target];
        if cursed {
            return StealResult::CursedBlock { slot: slot.clone() };
        }
        return StealResult::Stolen {
            item_name: format!("장비 ({})", slot),
            slot: slot.clone(),
        };
    }

    // 인벤토리 아이템
    StealResult::Stolen {
        item_name: "인벤토리 아이템".to_string(),
        slot: "inventory".to_string(),
    }
}

// =============================================================================
// [2] 님프 매혹 — nymph_charm (steal.c L700-800)
// =============================================================================

/// [v2.27.0 91-9] 님프 매혹 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NymphCharmResult {
    /// 매혹 성공 → 아이템 탈취
    Charmed {
        stolen_item: String,
        message: String,
    },
    /// 매혹 실패
    Resisted { message: String },
    /// 플레이어 기절
    Stunned { turns: i32 },
}

/// [v2.27.0 91-9] 님프 매혹 판정
pub fn nymph_charm(
    player_wis: i32,
    has_free_action: bool,
    is_blind: bool,
    rng: &mut NetHackRng,
) -> NymphCharmResult {
    if has_free_action {
        return NymphCharmResult::Resisted {
            message: "자유 행동이 매혹을 막았다.".to_string(),
        };
    }

    if is_blind {
        return NymphCharmResult::Resisted {
            message: "눈이 보이지 않아 매혹되지 않았다.".to_string(),
        };
    }

    // WIS 기반 저항
    if rng.rn2(20) < player_wis - 8 {
        return NymphCharmResult::Resisted {
            message: "지혜로써 유혹을 물리쳤다.".to_string(),
        };
    }

    // 매혹 성공
    if rng.rn2(3) == 0 {
        NymphCharmResult::Stunned {
            turns: rng.rn2(3) + 1,
        }
    } else {
        NymphCharmResult::Charmed {
            stolen_item: "아이템".to_string(),
            message: "님프의 매력에 빠져 아이템을 빼앗겼다!".to_string(),
        }
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_steal_nothing() {
        let mut rng = test_rng();
        let result = steal_attempt(10, 14, 0, false, 0, &[], &mut rng);
        assert!(matches!(result, StealResult::NothingToSteal));
    }

    #[test]
    fn test_steal_gold() {
        let mut got_gold = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = steal_attempt(10, 10, 0, false, 300, &[], &mut rng);
            if matches!(result, StealResult::GoldStolen { .. }) {
                got_gold = true;
                break;
            }
        }
        assert!(got_gold);
    }

    #[test]
    fn test_steal_cursed_block() {
        let mut got_block = false;
        let slots = vec![("armor".to_string(), true)];
        for seed in 0..30u64 {
            let mut rng = NetHackRng::new(seed);
            let result = steal_attempt(20, 10, 0, false, 0, &slots, &mut rng);
            if matches!(result, StealResult::CursedBlock { .. }) {
                got_block = true;
                break;
            }
        }
        assert!(got_block);
    }

    #[test]
    fn test_nymph_free_action() {
        let mut rng = test_rng();
        let result = nymph_charm(14, true, false, &mut rng);
        assert!(matches!(result, NymphCharmResult::Resisted { .. }));
    }

    #[test]
    fn test_nymph_blind() {
        let mut rng = test_rng();
        let result = nymph_charm(14, false, true, &mut rng);
        assert!(matches!(result, NymphCharmResult::Resisted { .. }));
    }
}
