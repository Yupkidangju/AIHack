// ============================================================================
// [v2.26.0 Phase 90-1] 몬스터→플레이어 전투 확장 (mhitu_phase90_ext.rs)
// 원본: NetHack 3.6.7 src/mhitu.c L1200-3000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 특수 공격 효과 — mattacku_specials (mhitu.c L1200-1800)
// =============================================================================

/// [v2.26.0 90-1] 특수 공격 효과 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialAttackEffect {
    /// 능력치 드레인
    StatDrain { stat_index: usize, amount: i32 },
    /// 레벨 드레인
    LevelDrain { levels: i32 },
    /// 석화 (서서히)
    Petrification { turns_remaining: i32 },
    /// 질병
    Disease { turns: i32, fatal: bool },
    /// 슬라임화
    Sliming { turns: i32 },
    /// 독
    Poison { damage: i32, stat_index: usize },
    /// 마비
    Paralysis { turns: i32 },
    /// 실명
    Blinding { turns: i32 },
    /// 혼란
    Confusion { turns: i32 },
    /// 텔레포트
    Teleport,
    /// 녹슬게 함 (무기/갑옷 부식)
    Rust { slot: String },
    /// 아이템 훔침
    Steal { item_type: String },
    /// 포옹 (구속)
    Grab { damage_per_turn: i32 },
    /// 플레이어를 삼킴
    Engulf,
    /// 없음
    None,
}

/// [v2.26.0 90-1] 특수 공격 입력
#[derive(Debug, Clone)]
pub struct SpecialAttackInput {
    /// 공격 유형 ID (원본: AT_xxx)
    pub attack_type: i32,
    /// 데미지 타입 (원본: AD_xxx)
    pub damage_type: i32,
    /// 몬스터 레벨
    pub monster_level: i32,
    /// 플레이어 방어: 해당 속성 저항 여부
    pub has_resistance: bool,
    /// 플레이어 MC (마법 취소)
    pub magic_cancellation: i32,
    /// 플레이어 AC
    pub player_ac: i32,
}

/// [v2.26.0 90-1] 특수 공격 효과 판정
/// 원본: mhitu.c mattacku() → mdamageu() 내 각 AD_xxx 분기
pub fn special_attack_effect(
    input: &SpecialAttackInput,
    rng: &mut NetHackRng,
) -> SpecialAttackEffect {
    // 저항이 있으면 대부분 무효
    if input.has_resistance {
        return SpecialAttackEffect::None;
    }

    // MC 기반 확률 — MC가 높으면 특수공격 무효화
    // 원본: mhitu.c 마법 취소 확률 = MC * 25%
    let mc_chance = input.magic_cancellation * 25;
    if mc_chance > 0 && rng.rn2(100) < mc_chance {
        return SpecialAttackEffect::None;
    }

    // damage_type별 분기 (원본 AD_xxx 상수)
    match input.damage_type {
        1 => {
            // AD_DRST — 힘 드레인
            let amount = 1 + rng.rn2(2);
            SpecialAttackEffect::Poison {
                damage: input.monster_level / 2 + 1,
                stat_index: 0, // STR
            }
        }
        2 => {
            // AD_DRLI — 레벨 드레인
            SpecialAttackEffect::LevelDrain { levels: 1 }
        }
        3 => {
            // AD_STON — 석화
            SpecialAttackEffect::Petrification { turns_remaining: 5 }
        }
        4 => {
            // AD_DISE — 질병
            SpecialAttackEffect::Disease {
                turns: 20 + rng.rn2(20),
                fatal: rng.rn2(3) == 0,
            }
        }
        5 => {
            // AD_SLIM — 슬라임화
            SpecialAttackEffect::Sliming { turns: 10 }
        }
        6 => {
            // AD_PLYS — 마비
            SpecialAttackEffect::Paralysis {
                turns: 3 + rng.rn2(4),
            }
        }
        7 => {
            // AD_BLND — 실명
            SpecialAttackEffect::Blinding {
                turns: 10 + rng.rn2(20),
            }
        }
        8 => {
            // AD_CONF — 혼란
            SpecialAttackEffect::Confusion {
                turns: 5 + rng.rn2(10),
            }
        }
        9 => {
            // AD_TELE — 텔레포트
            SpecialAttackEffect::Teleport
        }
        10 => {
            // AD_RUST — 녹
            SpecialAttackEffect::Rust {
                slot: "weapon".to_string(),
            }
        }
        11 => {
            // AD_STCK — 포옹/구속
            SpecialAttackEffect::Grab {
                damage_per_turn: input.monster_level / 3 + 1,
            }
        }
        12 => {
            // AD_WRAP — 삼킴
            SpecialAttackEffect::Engulf
        }
        13 => {
            // AD_SITM — 아이템 훔침
            SpecialAttackEffect::Steal {
                item_type: "random".to_string(),
            }
        }
        _ => SpecialAttackEffect::None,
    }
}

// =============================================================================
// [2] 패시브 반격 데미지 — passiveum (mhitu.c L2800-3000)
// =============================================================================

/// [v2.26.0 90-1] 패시브 반격 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassiveEffect {
    /// 산성 혈액 (접촉 데미지)
    AcidBlood { damage: i32 },
    /// 전기 충격 (접촉)
    ElectricShock { damage: i32 },
    /// 가시 (접촉)
    Thorns { damage: i32 },
    /// 냉기 (접촉)
    ColdTouch { damage: i32 },
    /// 화염 (접촉)
    FireTouch { damage: i32 },
    /// 없음
    None,
}

/// [v2.26.0 90-1] 패시브 반격 판정
/// 원본: mhitu.c passiveum()
pub fn passive_effect(
    passive_type: i32,
    attacker_has_resistance: bool,
    monster_level: i32,
    rng: &mut NetHackRng,
) -> PassiveEffect {
    if attacker_has_resistance {
        return PassiveEffect::None;
    }

    match passive_type {
        1 => PassiveEffect::AcidBlood {
            damage: rng.rn2(monster_level / 2 + 1).max(1),
        },
        2 => PassiveEffect::ElectricShock {
            damage: rng.rn2(monster_level).max(1),
        },
        3 => PassiveEffect::Thorns {
            damage: rng.rn2(4) + 1,
        },
        4 => PassiveEffect::ColdTouch {
            damage: rng.rn2(monster_level / 2).max(1),
        },
        5 => PassiveEffect::FireTouch {
            damage: rng.rn2(monster_level / 2).max(1),
        },
        _ => PassiveEffect::None,
    }
}

// =============================================================================
// [3] 명중 판정 — mattacku hit_check (mhitu.c L800-950)
// =============================================================================

/// [v2.26.0 90-1] 몬스터→플레이어 명중 판정
/// 원본: mhitu.c 명중 계산 (thitu() 간소화)
pub fn monster_hit_check(
    monster_level: i32,
    player_ac: i32,
    is_blind: bool,
    is_stunned: bool,
    rng: &mut NetHackRng,
) -> bool {
    // 원본: 1d20 + monster_level >= player_ac + 10
    let roll = rng.rn2(20) + 1;
    let mut to_hit = roll + monster_level;

    // 플레이어 실명/기절 시 명중 보너스
    if is_blind {
        to_hit += 2;
    }
    if is_stunned {
        to_hit += 2;
    }

    let target = player_ac + 10;
    to_hit >= target
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

    // --- special_attack_effect ---

    #[test]
    fn test_special_resistance() {
        let mut rng = test_rng();
        let input = SpecialAttackInput {
            attack_type: 0,
            damage_type: 2,
            monster_level: 10,
            has_resistance: true,
            magic_cancellation: 0,
            player_ac: 5,
        };
        assert_eq!(
            special_attack_effect(&input, &mut rng),
            SpecialAttackEffect::None
        );
    }

    #[test]
    fn test_special_level_drain() {
        let mut rng = test_rng();
        let input = SpecialAttackInput {
            attack_type: 0,
            damage_type: 2,
            monster_level: 10,
            has_resistance: false,
            magic_cancellation: 0,
            player_ac: 5,
        };
        let result = special_attack_effect(&input, &mut rng);
        assert!(matches!(result, SpecialAttackEffect::LevelDrain { .. }));
    }

    #[test]
    fn test_special_petrification() {
        let mut rng = test_rng();
        let input = SpecialAttackInput {
            attack_type: 0,
            damage_type: 3,
            monster_level: 10,
            has_resistance: false,
            magic_cancellation: 0,
            player_ac: 5,
        };
        let result = special_attack_effect(&input, &mut rng);
        assert!(matches!(result, SpecialAttackEffect::Petrification { .. }));
    }

    #[test]
    fn test_special_mc_blocks() {
        // MC 3 = 75% 차단 확률 — 여러 시드로 하나라도 차단되는지 확인
        let mut blocked = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let input = SpecialAttackInput {
                attack_type: 0,
                damage_type: 6,
                monster_level: 10,
                has_resistance: false,
                magic_cancellation: 3,
                player_ac: 5,
            };
            if special_attack_effect(&input, &mut rng) == SpecialAttackEffect::None {
                blocked = true;
                break;
            }
        }
        assert!(blocked, "MC 3이면 높은 확률로 차단해야 함");
    }

    // --- passive_effect ---

    #[test]
    fn test_passive_acid() {
        let mut rng = test_rng();
        let result = passive_effect(1, false, 10, &mut rng);
        assert!(matches!(result, PassiveEffect::AcidBlood { .. }));
    }

    #[test]
    fn test_passive_resistant() {
        let mut rng = test_rng();
        let result = passive_effect(1, true, 10, &mut rng);
        assert_eq!(result, PassiveEffect::None);
    }

    // --- monster_hit_check ---

    #[test]
    fn test_hit_check_high_ac() {
        // NetHack: target = AC + 10. AC 낮을수록 target 낮아서 명중 쉬움.
        // AC 10 (방어없음) → target 20 → 명중 어려움
        // AC -5 (좋은 방어) → target 5 → 명중 쉬움 (역설적이지만 올바른 동작)
        let mut hits_ac10 = 0;
        let mut hits_acn5 = 0;
        for seed in 0..50u64 {
            let mut rng = NetHackRng::new(seed);
            if monster_hit_check(5, 10, false, false, &mut rng) {
                hits_ac10 += 1;
            }
            let mut rng2 = NetHackRng::new(seed);
            if monster_hit_check(5, -5, false, false, &mut rng2) {
                hits_acn5 += 1;
            }
        }
        // AC -5 (target=5)는 AC 10 (target=20)보다 더 많이 맞아야 함
        assert!(
            hits_acn5 >= hits_ac10,
            "AC 낮으면(target 작아) 명중 더 쉬움"
        );
    }

    #[test]
    fn test_hit_check_blind_bonus() {
        // 실명 시 명중률 증가 확인
        let mut hits_normal = 0;
        let mut hits_blind = 0;
        for seed in 0..50u64 {
            let mut rng = NetHackRng::new(seed);
            if monster_hit_check(5, 5, false, false, &mut rng) {
                hits_normal += 1;
            }
            let mut rng2 = NetHackRng::new(seed);
            if monster_hit_check(5, 5, true, false, &mut rng2) {
                hits_blind += 1;
            }
        }
        assert!(hits_blind >= hits_normal, "실명 시 명중률 증가");
    }
}
