// ============================================================================
// [v2.28.0 Phase 92-1] 포션 효과 확장 (potion_phase92_ext.rs)
// 원본: NetHack 3.6.7 src/potion.c L400-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 포션 효과 — potion_effect (potion.c L400-900)
// =============================================================================

/// [v2.28.0 92-1] 포션 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionType {
    Healing,
    ExtraHealing,
    FullHealing,
    Gain,
    Speed,
    Invisibility,
    SeeInvisible,
    Levitation,
    Confusion,
    Blindness,
    Hallucination,
    Sleeping,
    Paralysis,
    Poison,
    Acid,
    Oil,
    Water,
    Booze,
    Sickness,
    Polymorph,
    GainAbility,
    GainLevel,
    Enlightenment,
    MonsterDetection,
    ObjectDetection,
}

/// [v2.28.0 92-1] 포션 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PotionEffectResult {
    /// HP 회복
    Heal { amount: i32, max_increase: i32 },
    /// 속도 변화
    SpeedChange { bonus: i32, turns: i32 },
    /// 상태 부여
    StatusGrant { status: String, turns: i32 },
    /// 상태 해제
    StatusCure { cured: Vec<String> },
    /// 능력치 변화
    StatChange { stat: String, amount: i32 },
    /// 레벨 변화
    LevelChange { amount: i32 },
    /// 데미지
    Damage { amount: i32, element: String },
    /// 변이
    Polymorph,
    /// 탐지
    Detect { detect_type: String, radius: i32 },
    /// 깨달음 (스탯 표시)
    Enlightenment,
    /// 없음
    NoEffect { message: String },
}

/// [v2.28.0 92-1] 포션 효과 판정
/// 원본: potion.c peffects()
pub fn potion_effect(
    potion: PotionType,
    is_blessed: bool,
    is_cursed: bool,
    player_level: i32,
    rng: &mut NetHackRng,
) -> PotionEffectResult {
    match potion {
        PotionType::Healing => {
            let base = if is_blessed {
                rng.rn2(8) + 8
            } else {
                rng.rn2(8) + 4
            };
            let max_inc = if is_blessed { 2 } else { 0 };
            PotionEffectResult::Heal {
                amount: base,
                max_increase: max_inc,
            }
        }
        PotionType::ExtraHealing => {
            let base = if is_blessed {
                rng.rn2(12) + 20
            } else {
                rng.rn2(12) + 12
            };
            PotionEffectResult::Heal {
                amount: base,
                max_increase: if is_blessed { 5 } else { 2 },
            }
        }
        PotionType::FullHealing => PotionEffectResult::Heal {
            amount: 999,
            max_increase: if is_blessed { 8 } else { 4 },
        },
        PotionType::Speed => {
            if is_cursed {
                PotionEffectResult::SpeedChange {
                    bonus: -1,
                    turns: rng.rn2(50) + 50,
                }
            } else {
                PotionEffectResult::SpeedChange {
                    bonus: if is_blessed { 2 } else { 1 },
                    turns: rng.rn2(100) + 100,
                }
            }
        }
        PotionType::Invisibility => {
            PotionEffectResult::StatusGrant {
                status: "투명".to_string(),
                turns: if is_blessed { -1 } else { rng.rn2(100) + 50 }, // -1 = 영구
            }
        }
        PotionType::SeeInvisible => {
            PotionEffectResult::StatusGrant {
                status: "투명 감지".to_string(),
                turns: -1, // 영구
            }
        }
        PotionType::Levitation => PotionEffectResult::StatusGrant {
            status: "부유".to_string(),
            turns: if is_blessed {
                rng.rn2(50) + 100
            } else {
                rng.rn2(50) + 50
            },
        },
        PotionType::Confusion => {
            if is_blessed {
                PotionEffectResult::StatusCure {
                    cured: vec!["혼란".to_string()],
                }
            } else {
                PotionEffectResult::StatusGrant {
                    status: "혼란".to_string(),
                    turns: rng.rn2(20) + 10,
                }
            }
        }
        PotionType::Blindness => {
            if is_blessed {
                PotionEffectResult::StatusCure {
                    cured: vec!["실명".to_string()],
                }
            } else {
                PotionEffectResult::StatusGrant {
                    status: "실명".to_string(),
                    turns: rng.rn2(100) + 50,
                }
            }
        }
        PotionType::Hallucination => {
            if is_blessed {
                PotionEffectResult::StatusCure {
                    cured: vec!["환각".to_string()],
                }
            } else {
                PotionEffectResult::StatusGrant {
                    status: "환각".to_string(),
                    turns: rng.rn2(100) + 50,
                }
            }
        }
        PotionType::Sleeping => PotionEffectResult::StatusGrant {
            status: "수면".to_string(),
            turns: rng.rn2(20) + 10,
        },
        PotionType::Paralysis => PotionEffectResult::StatusGrant {
            status: "마비".to_string(),
            turns: rng.rn2(10) + 5,
        },
        PotionType::Poison => {
            if is_blessed {
                PotionEffectResult::StatusCure {
                    cured: vec!["독".to_string(), "질병".to_string()],
                }
            } else {
                PotionEffectResult::Damage {
                    amount: rng.rn2(6) + 1,
                    element: "독".to_string(),
                }
            }
        }
        PotionType::Acid => {
            let dmg = if is_cursed {
                rng.rn2(12) + 6
            } else {
                rng.rn2(8) + 2
            };
            PotionEffectResult::Damage {
                amount: dmg,
                element: "산성".to_string(),
            }
        }
        PotionType::Oil => PotionEffectResult::NoEffect {
            message: "기름 맛이 난다.".to_string(),
        },
        PotionType::Water => {
            if is_blessed {
                PotionEffectResult::StatusCure {
                    cured: vec!["질병".to_string(), "실명".to_string(), "혼란".to_string()],
                }
            } else if is_cursed {
                PotionEffectResult::StatusGrant {
                    status: "혼란".to_string(),
                    turns: rng.rn2(5) + 3,
                }
            } else {
                PotionEffectResult::NoEffect {
                    message: "상쾌한 느낌이 든다.".to_string(),
                }
            }
        }
        PotionType::Booze => PotionEffectResult::StatusGrant {
            status: "혼란".to_string(),
            turns: rng.rn2(15) + 5,
        },
        PotionType::Sickness => PotionEffectResult::StatusGrant {
            status: "질병".to_string(),
            turns: rng.rn2(50) + 20,
        },
        PotionType::Polymorph => PotionEffectResult::Polymorph,
        PotionType::GainAbility => {
            let stat_names = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
            let idx = rng.rn2(6) as usize;
            PotionEffectResult::StatChange {
                stat: stat_names[idx].to_string(),
                amount: if is_blessed { 2 } else { 1 },
            }
        }
        PotionType::GainLevel => PotionEffectResult::LevelChange {
            amount: if is_cursed { -1 } else { 1 },
        },
        PotionType::Enlightenment => PotionEffectResult::Enlightenment,
        PotionType::MonsterDetection => PotionEffectResult::Detect {
            detect_type: "몬스터".to_string(),
            radius: if is_blessed { 999 } else { 20 },
        },
        PotionType::ObjectDetection => PotionEffectResult::Detect {
            detect_type: "물건".to_string(),
            radius: if is_blessed { 999 } else { 20 },
        },
        PotionType::Gain => {
            // 경험치 증가
            let exp = (player_level as i32) * 10 + rng.rn2(50);
            PotionEffectResult::StatChange {
                stat: "EXP".to_string(),
                amount: exp,
            }
        }
    }
}

// =============================================================================
// [2] 포션 딥 — dip_effect (potion.c L1000-1200)
// =============================================================================

/// [v2.28.0 92-1] 아이템을 포션에 담그기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DipResult {
    /// 무기 독 바르기
    PoisonWeapon { turns: i32 },
    /// 축복/저주 변환 (성수/저주수)
    BucChange { new_buc: String },
    /// 무기 녹 제거
    RustRemoval,
    /// 루시안 블레이드 (축복+빛)
    Glow { color: String },
    /// 불꽃 코팅 (오일)
    FlameCoat { bonus_damage: i32, turns: i32 },
    /// 폴리모프
    TransformItem { message: String },
    /// 효과 없음
    NoEffect,
}

/// [v2.28.0 92-1] 포션에 아이템 담그기
pub fn dip_item_in_potion(
    potion: PotionType,
    item_class: &str,
    is_potion_blessed: bool,
    is_potion_cursed: bool,
    rng: &mut NetHackRng,
) -> DipResult {
    match potion {
        PotionType::Poison => {
            if item_class == "weapon" {
                DipResult::PoisonWeapon {
                    turns: rng.rn2(20) + 10,
                }
            } else {
                DipResult::NoEffect
            }
        }
        PotionType::Water => {
            if is_potion_blessed {
                DipResult::BucChange {
                    new_buc: "축복".to_string(),
                }
            } else if is_potion_cursed {
                DipResult::BucChange {
                    new_buc: "저주".to_string(),
                }
            } else {
                DipResult::BucChange {
                    new_buc: "무축".to_string(),
                }
            }
        }
        PotionType::Oil => {
            if item_class == "weapon" {
                DipResult::FlameCoat {
                    bonus_damage: 2,
                    turns: rng.rn2(30) + 20,
                }
            } else {
                DipResult::RustRemoval
            }
        }
        PotionType::Polymorph => DipResult::TransformItem {
            message: "아이템이 변형되었다!".to_string(),
        },
        PotionType::Acid => {
            if item_class == "weapon" {
                DipResult::RustRemoval
            } else {
                DipResult::NoEffect
            }
        }
        _ => DipResult::NoEffect,
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

    // --- potion_effect ---

    #[test]
    fn test_healing_normal() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::Healing, false, false, 10, &mut rng);
        if let PotionEffectResult::Heal {
            amount,
            max_increase,
        } = result
        {
            assert!(amount >= 4 && amount <= 11);
            assert_eq!(max_increase, 0);
        } else {
            panic!("회복 기대");
        }
    }

    #[test]
    fn test_healing_blessed() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::Healing, true, false, 10, &mut rng);
        if let PotionEffectResult::Heal {
            amount,
            max_increase,
        } = result
        {
            assert!(amount >= 8);
            assert_eq!(max_increase, 2);
        } else {
            panic!("축복 회복 기대");
        }
    }

    #[test]
    fn test_full_healing() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::FullHealing, false, false, 10, &mut rng);
        assert!(matches!(
            result,
            PotionEffectResult::Heal { amount: 999, .. }
        ));
    }

    #[test]
    fn test_speed_cursed() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::Speed, false, true, 10, &mut rng);
        if let PotionEffectResult::SpeedChange { bonus, .. } = result {
            assert_eq!(bonus, -1);
        } else {
            panic!("속도 감소 기대");
        }
    }

    #[test]
    fn test_confusion_blessed_cures() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::Confusion, true, false, 10, &mut rng);
        assert!(matches!(result, PotionEffectResult::StatusCure { .. }));
    }

    #[test]
    fn test_polymorph() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::Polymorph, false, false, 10, &mut rng);
        assert!(matches!(result, PotionEffectResult::Polymorph));
    }

    #[test]
    fn test_gain_ability() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::GainAbility, true, false, 10, &mut rng);
        if let PotionEffectResult::StatChange { amount, .. } = result {
            assert_eq!(amount, 2); // 축복이면 +2
        }
    }

    #[test]
    fn test_gain_level_cursed() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::GainLevel, false, true, 10, &mut rng);
        assert!(matches!(
            result,
            PotionEffectResult::LevelChange { amount: -1 }
        ));
    }

    #[test]
    fn test_holy_water() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::Water, true, false, 10, &mut rng);
        assert!(matches!(result, PotionEffectResult::StatusCure { .. }));
    }

    #[test]
    fn test_monster_detection() {
        let mut rng = test_rng();
        let result = potion_effect(PotionType::MonsterDetection, true, false, 10, &mut rng);
        assert!(matches!(result, PotionEffectResult::Detect { .. }));
    }

    // --- dip ---

    #[test]
    fn test_dip_poison_weapon() {
        let mut rng = test_rng();
        let result = dip_item_in_potion(PotionType::Poison, "weapon", false, false, &mut rng);
        assert!(matches!(result, DipResult::PoisonWeapon { .. }));
    }

    #[test]
    fn test_dip_holy_water() {
        let mut rng = test_rng();
        let result = dip_item_in_potion(PotionType::Water, "armor", true, false, &mut rng);
        assert!(matches!(result, DipResult::BucChange { .. }));
    }

    #[test]
    fn test_dip_polymorph() {
        let mut rng = test_rng();
        let result = dip_item_in_potion(PotionType::Polymorph, "scroll", false, false, &mut rng);
        assert!(matches!(result, DipResult::TransformItem { .. }));
    }
}
