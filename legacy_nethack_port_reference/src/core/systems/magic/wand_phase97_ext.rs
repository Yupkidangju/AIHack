// ============================================================================
// [v2.33.0 Phase 97-4] 지팡이/완드 확장 (wand_phase97_ext.rs)
// 원본: NetHack 3.6.7 src/zap.c L2000-3500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 완드 효과 — wand_effects (zap.c L2000-3000)
// =============================================================================

/// [v2.33.0 97-4] 완드 유형 (확장)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtWandType {
    Fire,
    Cold,
    Lightning,
    MagicMissile,
    Sleep,
    Death,
    Polymorph,
    Teleport,
    Cancellation,
    MakeInvisible,
    Slow,
    Speed,
    Undead,
    Opening,
    Locking,
    Probing,
    Digging,
    Light,
    Nothing,
    Wishing,
    CreateMonster,
    Striking,
}

/// [v2.33.0 97-4] 완드 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WandEffect {
    Beam {
        damage: i32,
        element: String,
        bounces: i32,
    },
    Ray {
        damage: i32,
        element: String,
    },
    StatusOnTarget {
        effect: String,
        turns: i32,
    },
    TeleportTarget,
    PolymorphTarget,
    InstantDeath {
        resisted: bool,
    },
    Cancel {
        magic_removed: bool,
    },
    OpenLock,
    CloseLock,
    Probe {
        info: String,
    },
    Dig {
        direction: String,
    },
    LightArea {
        radius: i32,
    },
    Nothing,
    Wish,
    SummonMonster {
        count: i32,
    },
    Strike {
        damage: i32,
    },
    Fizzled {
        reason: String,
    },
}

/// [v2.33.0 97-4] 완드 사용
/// 원본: zap.c dozap()
pub fn zap_wand(
    wand: ExtWandType,
    charges: i32,
    target_mr: i32,
    target_has_reflect: bool,
    rng: &mut NetHackRng,
) -> WandEffect {
    // 충전 부족
    if charges <= 0 {
        return WandEffect::Fizzled {
            reason: "충전이 남아있지 않다.".to_string(),
        };
    }

    // 반사 체크 (빔 유형)
    let is_beam = matches!(
        wand,
        ExtWandType::Fire
            | ExtWandType::Cold
            | ExtWandType::Lightning
            | ExtWandType::MagicMissile
            | ExtWandType::Death
            | ExtWandType::Sleep
    );

    if is_beam && target_has_reflect {
        return WandEffect::Beam {
            damage: 0,
            element: "반사됨".to_string(),
            bounces: 1,
        };
    }

    match wand {
        ExtWandType::Fire => WandEffect::Beam {
            damage: rng.rn2(12) + 6,
            element: "화염".to_string(),
            bounces: 0,
        },
        ExtWandType::Cold => WandEffect::Beam {
            damage: rng.rn2(12) + 6,
            element: "냉기".to_string(),
            bounces: 0,
        },
        ExtWandType::Lightning => WandEffect::Beam {
            damage: rng.rn2(12) + 6,
            element: "전기".to_string(),
            bounces: 0,
        },
        ExtWandType::MagicMissile => WandEffect::Ray {
            damage: rng.rn2(6) + 2,
            element: "마법".to_string(),
        },
        ExtWandType::Sleep => {
            if target_mr > 0 && rng.rn2(100) < target_mr {
                return WandEffect::Fizzled {
                    reason: "저항됨".to_string(),
                };
            }
            WandEffect::StatusOnTarget {
                effect: "수면".to_string(),
                turns: rng.rn2(15) + 5,
            }
        }
        ExtWandType::Death => {
            if target_mr > 50 && rng.rn2(100) < target_mr / 2 {
                return WandEffect::InstantDeath { resisted: true };
            }
            WandEffect::InstantDeath { resisted: false }
        }
        ExtWandType::Polymorph => {
            if target_mr > 0 && rng.rn2(100) < target_mr {
                return WandEffect::Fizzled {
                    reason: "저항됨".to_string(),
                };
            }
            WandEffect::PolymorphTarget
        }
        ExtWandType::Teleport => WandEffect::TeleportTarget,
        ExtWandType::Cancellation => WandEffect::Cancel {
            magic_removed: rng.rn2(100) >= target_mr / 2,
        },
        ExtWandType::MakeInvisible => WandEffect::StatusOnTarget {
            effect: "투명".to_string(),
            turns: rng.rn2(100) + 50,
        },
        ExtWandType::Slow => WandEffect::StatusOnTarget {
            effect: "감속".to_string(),
            turns: rng.rn2(20) + 10,
        },
        ExtWandType::Speed => WandEffect::StatusOnTarget {
            effect: "가속".to_string(),
            turns: rng.rn2(20) + 10,
        },
        ExtWandType::Undead => WandEffect::Ray {
            damage: rng.rn2(20) + 10,
            element: "언데드 전용".to_string(),
        },
        ExtWandType::Opening => WandEffect::OpenLock,
        ExtWandType::Locking => WandEffect::CloseLock,
        ExtWandType::Probing => WandEffect::Probe {
            info: "HP, 장비, 인벤토리 확인".to_string(),
        },
        ExtWandType::Digging => WandEffect::Dig {
            direction: "전방".to_string(),
        },
        ExtWandType::Light => WandEffect::LightArea { radius: 5 },
        ExtWandType::Nothing => WandEffect::Nothing,
        ExtWandType::Wishing => WandEffect::Wish,
        ExtWandType::CreateMonster => WandEffect::SummonMonster {
            count: rng.rn2(3) + 1,
        },
        ExtWandType::Striking => WandEffect::Strike {
            damage: rng.rn2(10) + 5,
        },
    }
}

// =============================================================================
// [2] 완드 파괴 — wand_break (zap.c L3000-3500)
// =============================================================================

/// [v2.33.0 97-4] 완드 파괴 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WandBreakResult {
    Explosion { damage: i32, radius: i32 },
    Release { effect: String },
    Fizzle,
}

/// [v2.33.0 97-4] 완드 강제 파괴
pub fn break_wand(wand: ExtWandType, charges: i32, rng: &mut NetHackRng) -> WandBreakResult {
    if charges <= 0 {
        return WandBreakResult::Fizzle;
    }

    match wand {
        ExtWandType::Fire | ExtWandType::Lightning | ExtWandType::Cold => {
            WandBreakResult::Explosion {
                damage: charges * (rng.rn2(6) + 4),
                radius: 3,
            }
        }
        ExtWandType::Death | ExtWandType::Cancellation => WandBreakResult::Release {
            effect: format!(
                "{}의 힘이 해방된다!",
                match wand {
                    ExtWandType::Death => "죽음",
                    _ => "취소",
                }
            ),
        },
        ExtWandType::Wishing => WandBreakResult::Release {
            effect: "소원의 힘이 사라진다...".to_string(),
        },
        _ => {
            if charges > 5 {
                WandBreakResult::Explosion {
                    damage: charges * 2,
                    radius: 1,
                }
            } else {
                WandBreakResult::Fizzle
            }
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
    fn test_fire_beam() {
        let mut rng = test_rng();
        let result = zap_wand(ExtWandType::Fire, 5, 0, false, &mut rng);
        assert!(matches!(result, WandEffect::Beam { .. }));
    }

    #[test]
    fn test_reflected() {
        let mut rng = test_rng();
        let result = zap_wand(ExtWandType::Fire, 5, 0, true, &mut rng);
        if let WandEffect::Beam { element, .. } = result {
            assert!(element.contains("반사"));
        }
    }

    #[test]
    fn test_no_charges() {
        let mut rng = test_rng();
        let result = zap_wand(ExtWandType::Fire, 0, 0, false, &mut rng);
        assert!(matches!(result, WandEffect::Fizzled { .. }));
    }

    #[test]
    fn test_death_wand() {
        let mut rng = test_rng();
        let result = zap_wand(ExtWandType::Death, 3, 0, false, &mut rng);
        assert!(matches!(
            result,
            WandEffect::InstantDeath { resisted: false }
        ));
    }

    #[test]
    fn test_wishing() {
        let mut rng = test_rng();
        let result = zap_wand(ExtWandType::Wishing, 1, 0, false, &mut rng);
        assert!(matches!(result, WandEffect::Wish));
    }

    #[test]
    fn test_teleport() {
        let mut rng = test_rng();
        let result = zap_wand(ExtWandType::Teleport, 3, 0, false, &mut rng);
        assert!(matches!(result, WandEffect::TeleportTarget));
    }

    #[test]
    fn test_break_fire() {
        let mut rng = test_rng();
        let result = break_wand(ExtWandType::Fire, 5, &mut rng);
        assert!(matches!(result, WandBreakResult::Explosion { .. }));
    }

    #[test]
    fn test_break_empty() {
        let mut rng = test_rng();
        let result = break_wand(ExtWandType::Fire, 0, &mut rng);
        assert!(matches!(result, WandBreakResult::Fizzle));
    }
}
