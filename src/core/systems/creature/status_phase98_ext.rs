// ============================================================================
// [v2.34.0 Phase 98-3] 상태이상 통합 확장 (status_phase98_ext.rs)
// 원본: NetHack 3.6.7 src/timeout.c + potion.c 상태이상 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상태이상 시스템 — status_effects (timeout.c 핵심)
// =============================================================================

/// [v2.34.0 98-3] 상태이상 유형 (전체)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusType {
    Confusion,
    Blindness,
    Stunned,
    Hallucination,
    Sleeping,
    Paralysis,
    Levitation,
    Invisibility,
    SeeInvisible,
    Telepathy,
    Speed,
    Slow,
    Poison,
    Petrifying,
    Lycanthropy,
    Polymorph,
    Hunger,
    Wounded,
    Aggravation,
    Protection,
    Stealth,
    Regeneration,
    Conflict,
    Charm,
    Fear,
    Berserk,
    Silence,
}

/// [v2.34.0 98-3] 상태이상 항목
#[derive(Debug, Clone)]
pub struct StatusEffect {
    pub status_type: StatusType,
    pub turns_remaining: i32,
    pub source: String,
    pub is_permanent: bool,
    pub severity: i32,
}

/// [v2.34.0 98-3] 상태이상 적용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusApplyResult {
    Applied { message: String },
    Extended { new_turns: i32 },
    AlreadyActive,
    Resisted { reason: String },
    Cured { message: String },
}

/// [v2.34.0 98-3] 상태이상 적용
pub fn apply_status(
    current_statuses: &[StatusEffect],
    new_status: StatusType,
    turns: i32,
    source: &str,
    has_resistance: bool,
) -> StatusApplyResult {
    // 저항 체크
    if has_resistance {
        return StatusApplyResult::Resisted {
            reason: format!("{:?}에 저항이 있다.", new_status),
        };
    }

    // 이미 활성 중인지
    if let Some(existing) = current_statuses.iter().find(|s| s.status_type == new_status) {
        if existing.is_permanent {
            return StatusApplyResult::AlreadyActive;
        }
        return StatusApplyResult::Extended {
            new_turns: existing.turns_remaining + turns,
        };
    }

    let message = match new_status {
        StatusType::Confusion => "혼란스러워졌다!".to_string(),
        StatusType::Blindness => "앞이 보이지 않는다!".to_string(),
        StatusType::Stunned => "비틀거린다!".to_string(),
        StatusType::Hallucination => "환각이 시작된다!".to_string(),
        StatusType::Sleeping => "잠이 든다...".to_string(),
        StatusType::Paralysis => "움직일 수 없다!".to_string(),
        StatusType::Levitation => "몸이 떠오른다!".to_string(),
        StatusType::Invisibility => "몸이 보이지 않게 된다.".to_string(),
        StatusType::Speed => "빨라졌다!".to_string(),
        StatusType::Slow => "느려졌다...".to_string(),
        StatusType::Poison => "독이 퍼지고 있다!".to_string(),
        StatusType::Petrifying => "몸이 굳어가고 있다!".to_string(),
        StatusType::Fear => "공포에 떨고 있다!".to_string(),
        StatusType::Berserk => "광전사 모드!".to_string(),
        _ => format!("{:?} 상태 적용", new_status),
    };

    StatusApplyResult::Applied { message }
}

/// [v2.34.0 98-3] 상태이상 턴 갱신
pub fn tick_status(effect: &StatusEffect) -> Option<StatusEffect> {
    if effect.is_permanent {
        return Some(effect.clone());
    }
    let remaining = effect.turns_remaining - 1;
    if remaining <= 0 {
        return None; // 만료
    }
    let mut updated = effect.clone();
    updated.turns_remaining = remaining;
    Some(updated)
}

/// [v2.34.0 98-3] 상태이상 치유
pub fn cure_status(
    current_statuses: &[StatusEffect],
    target: StatusType,
) -> StatusApplyResult {
    if current_statuses.iter().any(|s| s.status_type == target) {
        let message = match target {
            StatusType::Confusion => "머리가 맑아졌다.".to_string(),
            StatusType::Blindness => "다시 보인다!".to_string(),
            StatusType::Poison => "독이 빠져나갔다.".to_string(),
            StatusType::Petrifying => "몸이 다시 부드러워졌다.".to_string(),
            _ => format!("{:?} 상태가 해제되었다.", target),
        };
        StatusApplyResult::Cured { message }
    } else {
        StatusApplyResult::AlreadyActive // 이미 없음
    }
}

// =============================================================================
// [2] 포션 효과 통합 — potion_effects (potion.c 핵심)
// =============================================================================

/// [v2.34.0 98-3] 포션 종류 (전체)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtPotionType {
    Healing,
    ExtraHealing,
    FullHealing,
    GainLevel,
    GainEnergy,
    Speed,
    Invisibility,
    SeeInvisible,
    Blindness,
    Confusion,
    Hallucination,
    Sleeping,
    Paralysis,
    Poison,
    Polymorph,
    Levitation,
    OilLamp,
    FruitJuice,
    Water,
    Booze,
    Sickness,
    Restore,
    ObjectDetect,
    MonsterDetect,
}

/// [v2.34.0 98-3] 포션 음용 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PotionDrinkResult {
    HealHP { amount: i32 },
    GainLevel,
    GainMana { amount: i32 },
    StatusGained { status: StatusType, turns: i32 },
    StatusCured { cured: Vec<StatusType> },
    Polymorph,
    Detection { detect_type: String, count: i32 },
    Nutrition { amount: i32 },
    NoEffect,
}

/// [v2.34.0 98-3] 포션 음용
pub fn drink_potion(
    potion: ExtPotionType,
    is_blessed: bool,
    is_cursed: bool,
    rng: &mut NetHackRng,
) -> PotionDrinkResult {
    match potion {
        ExtPotionType::Healing => {
            let base = if is_blessed { 12 } else if is_cursed { 4 } else { 8 };
            PotionDrinkResult::HealHP { amount: rng.rn2(base) + base }
        }
        ExtPotionType::ExtraHealing => {
            let base = if is_blessed { 24 } else { 16 };
            PotionDrinkResult::HealHP { amount: rng.rn2(base) + base }
        }
        ExtPotionType::FullHealing => {
            PotionDrinkResult::HealHP { amount: 999 }
        }
        ExtPotionType::GainLevel => PotionDrinkResult::GainLevel,
        ExtPotionType::GainEnergy => {
            PotionDrinkResult::GainMana { amount: rng.rn2(20) + 10 }
        }
        ExtPotionType::Speed => PotionDrinkResult::StatusGained {
            status: StatusType::Speed,
            turns: rng.rn2(100) + 50,
        },
        ExtPotionType::Invisibility => PotionDrinkResult::StatusGained {
            status: StatusType::Invisibility,
            turns: rng.rn2(100) + 50,
        },
        ExtPotionType::SeeInvisible => PotionDrinkResult::StatusGained {
            status: StatusType::SeeInvisible,
            turns: if is_blessed { 999 } else { rng.rn2(100) + 50 },
        },
        ExtPotionType::Blindness => PotionDrinkResult::StatusGained {
            status: StatusType::Blindness,
            turns: rng.rn2(100) + 50,
        },
        ExtPotionType::Confusion => PotionDrinkResult::StatusGained {
            status: StatusType::Confusion,
            turns: rng.rn2(20) + 10,
        },
        ExtPotionType::Hallucination => PotionDrinkResult::StatusGained {
            status: StatusType::Hallucination,
            turns: rng.rn2(200) + 100,
        },
        ExtPotionType::Sleeping => PotionDrinkResult::StatusGained {
            status: StatusType::Sleeping,
            turns: rng.rn2(20) + 10,
        },
        ExtPotionType::Paralysis => PotionDrinkResult::StatusGained {
            status: StatusType::Paralysis,
            turns: rng.rn2(10) + 5,
        },
        ExtPotionType::Poison => PotionDrinkResult::StatusGained {
            status: StatusType::Poison,
            turns: rng.rn2(10) + 5,
        },
        ExtPotionType::Polymorph => PotionDrinkResult::Polymorph,
        ExtPotionType::Levitation => PotionDrinkResult::StatusGained {
            status: StatusType::Levitation,
            turns: rng.rn2(50) + 20,
        },
        ExtPotionType::Restore => PotionDrinkResult::StatusCured {
            cured: vec![StatusType::Blindness, StatusType::Confusion, StatusType::Stunned, StatusType::Hallucination],
        },
        ExtPotionType::ObjectDetect => PotionDrinkResult::Detection {
            detect_type: "아이템".to_string(),
            count: rng.rn2(10) + 3,
        },
        ExtPotionType::MonsterDetect => PotionDrinkResult::Detection {
            detect_type: "몬스터".to_string(),
            count: rng.rn2(15) + 5,
        },
        ExtPotionType::FruitJuice => PotionDrinkResult::Nutrition { amount: 50 },
        ExtPotionType::Water => {
            if is_blessed {
                PotionDrinkResult::StatusCured { cured: vec![StatusType::Lycanthropy] }
            } else {
                PotionDrinkResult::Nutrition { amount: 10 }
            }
        }
        ExtPotionType::Booze => PotionDrinkResult::StatusGained {
            status: StatusType::Confusion,
            turns: rng.rn2(15) + 5,
        },
        ExtPotionType::OilLamp => PotionDrinkResult::NoEffect,
        ExtPotionType::Sickness => PotionDrinkResult::StatusGained {
            status: StatusType::Poison,
            turns: rng.rn2(20) + 10,
        },
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
    fn test_apply_new_status() {
        let result = apply_status(&[], StatusType::Confusion, 10, "마법", false);
        assert!(matches!(result, StatusApplyResult::Applied { .. }));
    }

    #[test]
    fn test_apply_resisted() {
        let result = apply_status(&[], StatusType::Poison, 10, "독침", true);
        assert!(matches!(result, StatusApplyResult::Resisted { .. }));
    }

    #[test]
    fn test_tick_expire() {
        let effect = StatusEffect {
            status_type: StatusType::Confusion,
            turns_remaining: 1, source: "마법".to_string(),
            is_permanent: false, severity: 1,
        };
        let result = tick_status(&effect);
        assert!(result.is_none()); // 만료
    }

    #[test]
    fn test_tick_continue() {
        let effect = StatusEffect {
            status_type: StatusType::Speed,
            turns_remaining: 5, source: "포션".to_string(),
            is_permanent: false, severity: 1,
        };
        let result = tick_status(&effect);
        assert!(result.is_some());
        assert_eq!(result.unwrap().turns_remaining, 4);
    }

    #[test]
    fn test_cure() {
        let statuses = vec![StatusEffect {
            status_type: StatusType::Blindness,
            turns_remaining: 10, source: "함정".to_string(),
            is_permanent: false, severity: 1,
        }];
        let result = cure_status(&statuses, StatusType::Blindness);
        assert!(matches!(result, StatusApplyResult::Cured { .. }));
    }

    #[test]
    fn test_healing_potion() {
        let mut rng = test_rng();
        let result = drink_potion(ExtPotionType::Healing, false, false, &mut rng);
        if let PotionDrinkResult::HealHP { amount } = result {
            assert!(amount >= 8);
        }
    }

    #[test]
    fn test_full_healing() {
        let mut rng = test_rng();
        let result = drink_potion(ExtPotionType::FullHealing, false, false, &mut rng);
        assert!(matches!(result, PotionDrinkResult::HealHP { amount: 999 }));
    }

    #[test]
    fn test_restore_potion() {
        let mut rng = test_rng();
        let result = drink_potion(ExtPotionType::Restore, false, false, &mut rng);
        if let PotionDrinkResult::StatusCured { cured } = result {
            assert!(cured.len() >= 4);
        }
    }

    #[test]
    fn test_polymorph_potion() {
        let mut rng = test_rng();
        let result = drink_potion(ExtPotionType::Polymorph, false, false, &mut rng);
        assert!(matches!(result, PotionDrinkResult::Polymorph));
    }
}
