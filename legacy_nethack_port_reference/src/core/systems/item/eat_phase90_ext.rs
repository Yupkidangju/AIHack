// ============================================================================
// [v2.26.0 Phase 90-3] 음식 시스템 확장 (eat_phase90_ext.rs)
// 원본: NetHack 3.6.7 src/eat.c L1200-2800 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 시체 효과 — corpse_effect (eat.c L1200-1800)
// =============================================================================

/// [v2.26.0 90-3] 시체 섭취 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpseEffect {
    /// 능력치 증가
    StatGain { stat_index: usize, amount: i32 },
    /// 저항 획득
    ResistanceGain { resist_type: String },
    /// 독
    Poisoned {
        damage: i32,
        stat_loss: Option<(usize, i32)>,
    },
    /// 산성
    AcidBurn { damage: i32 },
    /// 석화 (코카트리스 시체)
    Petrification,
    /// 슬라임화 (녹색 슬라임)
    Sliming,
    /// 텔레포트 능력 획득
    Teleportitis,
    /// 텔레포트 제어 획득
    TeleportControl,
    /// 투명 능력 획득
    Invisibility,
    /// 변이 (무작위 효과)
    Polymorph,
    /// 질병
    FoodPoisoning { turns: i32 },
    /// 환각
    Hallucination { turns: i32 },
    /// 회복 (뉴트 시체 등)
    Healing { amount: i32 },
    /// 없음
    None,
}

/// [v2.26.0 90-3] 시체 효과 판정 입력
#[derive(Debug, Clone)]
pub struct CorpseEffectInput {
    /// 몬스터 종류 ID
    pub monster_type: i32,
    /// 시체 나이 (턴)
    pub corpse_age: i32,
    /// 독 저항 여부
    pub has_poison_resist: bool,
    /// 석화 저항 여부
    pub has_stone_resist: bool,
    /// 산성 저항 여부
    pub has_acid_resist: bool,
    /// 플레이어가 변이 저항 여부
    pub has_poly_resist: bool,
}

/// [v2.26.0 90-3] 시체 효과 판정
/// 원본: eat.c cpostfx() + cprefx()
pub fn corpse_effect(input: &CorpseEffectInput, rng: &mut NetHackRng) -> CorpseEffect {
    // [1] 썩은 시체 (age > 50) → 식중독
    if input.corpse_age > 50 && rng.rn2(3) == 0 {
        return CorpseEffect::FoodPoisoning {
            turns: 20 + rng.rn2(30),
        };
    }

    // [2] 몬스터 유형별 효과 (monster_type 기반)
    match input.monster_type {
        // 코카트리스 (타입 100) → 석화
        100 => {
            if input.has_stone_resist {
                CorpseEffect::None
            } else {
                CorpseEffect::Petrification
            }
        }
        // 녹색 슬라임 (타입 101) → 슬라임화
        101 => CorpseEffect::Sliming,
        // 님프 (타입 110) → 텔레포트
        110 => {
            if rng.rn2(3) == 0 {
                CorpseEffect::Teleportitis
            } else {
                CorpseEffect::None
            }
        }
        // 스톨커 (타입 111) → 투명
        111 => {
            if rng.rn2(3) == 0 {
                CorpseEffect::Invisibility
            } else {
                CorpseEffect::None
            }
        }
        // 카멜레온 (타입 120) → 변이
        120 => {
            if input.has_poly_resist {
                CorpseEffect::None
            } else {
                CorpseEffect::Polymorph
            }
        }
        // 독 몬스터 (타입 200-210) → 독
        200..=210 => {
            if input.has_poison_resist {
                CorpseEffect::ResistanceGain {
                    resist_type: "poison".to_string(),
                }
            } else {
                CorpseEffect::Poisoned {
                    damage: rng.rn2(6) + 1,
                    stat_loss: Some((0, 1)), // STR -1
                }
            }
        }
        // 산성 몬스터 (타입 220-225) → 산
        220..=225 => {
            if input.has_acid_resist {
                CorpseEffect::None
            } else {
                CorpseEffect::AcidBurn {
                    damage: rng.rn2(8) + 1,
                }
            }
        }
        // 뉴트 (타입 300) → 회복
        300 => CorpseEffect::Healing {
            amount: rng.rn2(8) + 5,
        },
        // 버섯 (타입 310) → 환각
        310 => CorpseEffect::Hallucination {
            turns: 20 + rng.rn2(30),
        },
        // 자이언트 (타입 400-405) → 힘 증가 (희귀)
        400..=405 => {
            if rng.rn2(6) == 0 {
                CorpseEffect::StatGain {
                    stat_index: 0, // STR
                    amount: 1,
                }
            } else {
                CorpseEffect::None
            }
        }
        // 마인드 플레이어 (타입 500) → 지능 증가 (드물게)
        500 => {
            if rng.rn2(5) == 0 {
                CorpseEffect::StatGain {
                    stat_index: 3, // INT
                    amount: 1,
                }
            } else {
                CorpseEffect::None
            }
        }
        _ => CorpseEffect::None,
    }
}

// =============================================================================
// [2] 영양가 계산 — nutrition_calc (eat.c L800-1000)
// =============================================================================

/// [v2.26.0 90-3] 음식 영양가 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NutritionResult {
    pub base_nutrition: i32,
    pub actual_nutrition: i32,
    pub turns_to_eat: i32,
    pub message: String,
}

/// [v2.26.0 90-3] 영양가 계산
/// 원본: eat.c dog_nutrition() + eat_common()
pub fn nutrition_calc(
    food_type: i32, // 음식 유형 코드
    quantity: i32,
    tin_contents: i32, // 통조림 내용 (-1=통조림 아님)
    blessed: bool,
    rng: &mut NetHackRng,
) -> NutritionResult {
    let (base, turns, msg) = match food_type {
        // 일반 식량
        0 => (800, 5, "식량을 먹었다."),
        // 과일
        1 => (250, 2, "과즙이 풍부하다!"),
        // 시체 (크기별)
        2 => (200 + rng.rn2(200), 3, "시체를 뜯어먹었다."),
        // 달걀
        3 => (80, 1, "달걀을 먹었다."),
        // 통조림
        4 => {
            let n = if tin_contents >= 0 {
                50 + tin_contents * 10
            } else {
                100
            };
            (n, 2, "통조림을 먹었다.")
        }
        // 린치 시식용
        5 => (100, 1, "뭔가 먹었다."),
        // 웨이퍼
        6 => (200, 1, "웨이퍼를 먹었다!"),
        // 트라이던트
        7 => (0, 1, "이건 먹을 수 없다!"),
        _ => (100, 1, "먹었다."),
    };

    let blessing_bonus = if blessed { base / 5 } else { 0 };
    let actual = (base + blessing_bonus) * quantity;

    NutritionResult {
        base_nutrition: base,
        actual_nutrition: actual,
        turns_to_eat: turns * quantity,
        message: msg.to_string(),
    }
}

// =============================================================================
// [3] 식욕 상태 전이 — hunger_transition (eat.c L50-150)
// =============================================================================

/// [v2.26.0 90-3] 허기 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HungerState {
    Satiated = 0,
    NotHungry = 1,
    Hungry = 2,
    Weak = 3,
    Fainting = 4,
    Starving = 5,
}

/// [v2.26.0 90-3] 허기 상태 전이 판정
/// 원본: eat.c moddone_apply() 기반
pub fn hunger_transition(
    current_nutrition: i32,
    consumption_per_turn: i32,
    ring_hunger: i32, // 허기 반지 보정 (+유지/-소모)
) -> HungerState {
    let effective = current_nutrition - ring_hunger;

    match effective {
        n if n >= 1000 => HungerState::Satiated,
        n if n >= 150 => HungerState::NotHungry,
        n if n >= 50 => HungerState::Hungry,
        n if n >= 10 => HungerState::Weak,
        n if n >= 1 => HungerState::Fainting,
        _ => HungerState::Starving,
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

    // --- corpse_effect ---

    #[test]
    fn test_cockatrice_petrify() {
        let mut rng = test_rng();
        let input = CorpseEffectInput {
            monster_type: 100,
            corpse_age: 0,
            has_poison_resist: false,
            has_stone_resist: false,
            has_acid_resist: false,
            has_poly_resist: false,
        };
        assert_eq!(corpse_effect(&input, &mut rng), CorpseEffect::Petrification);
    }

    #[test]
    fn test_cockatrice_resistant() {
        let mut rng = test_rng();
        let input = CorpseEffectInput {
            monster_type: 100,
            corpse_age: 0,
            has_poison_resist: false,
            has_stone_resist: true,
            has_acid_resist: false,
            has_poly_resist: false,
        };
        assert_eq!(corpse_effect(&input, &mut rng), CorpseEffect::None);
    }

    #[test]
    fn test_newt_healing() {
        let mut rng = test_rng();
        let input = CorpseEffectInput {
            monster_type: 300,
            corpse_age: 0,
            has_poison_resist: false,
            has_stone_resist: false,
            has_acid_resist: false,
            has_poly_resist: false,
        };
        assert!(matches!(
            corpse_effect(&input, &mut rng),
            CorpseEffect::Healing { .. }
        ));
    }

    #[test]
    fn test_poison_with_resist() {
        let mut rng = test_rng();
        let input = CorpseEffectInput {
            monster_type: 200,
            corpse_age: 0,
            has_poison_resist: true,
            has_stone_resist: false,
            has_acid_resist: false,
            has_poly_resist: false,
        };
        assert!(matches!(
            corpse_effect(&input, &mut rng),
            CorpseEffect::ResistanceGain { .. }
        ));
    }

    // --- nutrition_calc ---

    #[test]
    fn test_ration_nutrition() {
        let mut rng = test_rng();
        let result = nutrition_calc(0, 1, -1, false, &mut rng);
        assert_eq!(result.base_nutrition, 800);
    }

    #[test]
    fn test_blessed_bonus() {
        let mut rng = test_rng();
        let normal = nutrition_calc(0, 1, -1, false, &mut rng).actual_nutrition;
        let mut rng2 = test_rng();
        let blessed = nutrition_calc(0, 1, -1, true, &mut rng2).actual_nutrition;
        assert!(blessed > normal, "축복 시 영양 증가");
    }

    // --- hunger_transition ---

    #[test]
    fn test_satiated() {
        assert_eq!(hunger_transition(1500, 1, 0), HungerState::Satiated);
    }

    #[test]
    fn test_starving() {
        assert_eq!(hunger_transition(0, 1, 0), HungerState::Starving);
    }

    #[test]
    fn test_hungry() {
        assert_eq!(hunger_transition(100, 1, 0), HungerState::Hungry);
    }
}
