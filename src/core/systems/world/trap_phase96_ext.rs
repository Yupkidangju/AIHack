// ============================================================================
// [v2.32.0 Phase 96-2] 함정 확장 (trap_phase96_ext.rs)
// 원본: NetHack 3.6.7 src/trap.c L1500-3000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 함정 생성 — trap_creation (trap.c L1500-2000)
// =============================================================================

/// [v2.32.0 96-2] 함정 유형 (확장)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtTrapType {
    ArrowTrap,
    DartTrap,
    RockTrap,
    SqueakyBoard,
    BearTrap,
    LandMine,
    RollingBoulder,
    SleepingGas,
    RustTrap,
    FireTrap,
    Pit,
    SpikedPit,
    Hole,
    TrapDoor,
    TeleportTrap,
    LevelTeleport,
    MagicPortal,
    Web,
    StatueTrap,
    MagicTrap,
    AntiMagicTrap,
    PolymorphTrap,
}

/// [v2.32.0 96-2] 함정 발동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapTriggerResult {
    Damage { amount: i32, trap_type: String },
    StatusEffect { effect: String, turns: i32 },
    FallDown { levels: i32 },
    Teleport { random: bool },
    Polymorph,
    Immobilize { turns: i32 },
    Alert { noise_radius: i32 },
    ItemDamage { item_effect: String },
    NoEffect { reason: String },
}

/// [v2.32.0 96-2] 함정 발동
/// 원본: trap.c dotrap()
pub fn trigger_trap(
    trap: ExtTrapType,
    victim_dex: i32,
    has_levitation: bool,
    has_flying: bool,
    rng: &mut NetHackRng,
) -> TrapTriggerResult {
    // 공중 → 바닥 함정 무시
    let airborne = has_levitation || has_flying;
    if airborne {
        match trap {
            ExtTrapType::Pit
            | ExtTrapType::SpikedPit
            | ExtTrapType::Hole
            | ExtTrapType::TrapDoor
            | ExtTrapType::BearTrap
            | ExtTrapType::Web
            | ExtTrapType::SqueakyBoard
            | ExtTrapType::LandMine => {
                return TrapTriggerResult::NoEffect {
                    reason: "공중에 떠 있어 피했다.".to_string(),
                };
            }
            _ => {}
        }
    }

    // DEX 회피
    let dodge = rng.rn2(20) + 1;
    let dodge_target = victim_dex / 2;

    match trap {
        ExtTrapType::ArrowTrap => {
            if dodge > dodge_target {
                TrapTriggerResult::Damage {
                    amount: rng.rn2(6) + 1,
                    trap_type: "화살 함정".to_string(),
                }
            } else {
                TrapTriggerResult::NoEffect {
                    reason: "화살을 피했다!".to_string(),
                }
            }
        }
        ExtTrapType::DartTrap => {
            if dodge > dodge_target {
                TrapTriggerResult::Damage {
                    amount: rng.rn2(4) + 1,
                    trap_type: "독침 함정".to_string(),
                }
            } else {
                TrapTriggerResult::NoEffect {
                    reason: "독침을 피했다!".to_string(),
                }
            }
        }
        ExtTrapType::RockTrap => TrapTriggerResult::Damage {
            amount: rng.rn2(10) + 5,
            trap_type: "낙석 함정".to_string(),
        },
        ExtTrapType::SqueakyBoard => TrapTriggerResult::Alert { noise_radius: 15 },
        ExtTrapType::BearTrap => TrapTriggerResult::Immobilize {
            turns: rng.rn2(5) + 3,
        },
        ExtTrapType::LandMine => TrapTriggerResult::Damage {
            amount: rng.rn2(16) + 10,
            trap_type: "지뢰".to_string(),
        },
        ExtTrapType::RollingBoulder => TrapTriggerResult::Damage {
            amount: rng.rn2(20) + 10,
            trap_type: "구르는 바위".to_string(),
        },
        ExtTrapType::SleepingGas => TrapTriggerResult::StatusEffect {
            effect: "수면".to_string(),
            turns: rng.rn2(15) + 5,
        },
        ExtTrapType::RustTrap => TrapTriggerResult::ItemDamage {
            item_effect: "장비 녹슬음".to_string(),
        },
        ExtTrapType::FireTrap => TrapTriggerResult::Damage {
            amount: rng.rn2(12) + 4,
            trap_type: "화염 함정".to_string(),
        },
        ExtTrapType::Pit => TrapTriggerResult::Damage {
            amount: rng.rn2(6) + 2,
            trap_type: "함정 구덩이".to_string(),
        },
        ExtTrapType::SpikedPit => TrapTriggerResult::Damage {
            amount: rng.rn2(10) + 5,
            trap_type: "가시 구덩이".to_string(),
        },
        ExtTrapType::Hole | ExtTrapType::TrapDoor => TrapTriggerResult::FallDown { levels: 1 },
        ExtTrapType::TeleportTrap => TrapTriggerResult::Teleport { random: true },
        ExtTrapType::LevelTeleport => TrapTriggerResult::FallDown {
            levels: rng.rn2(5) + 1,
        },
        ExtTrapType::MagicPortal => TrapTriggerResult::Teleport { random: false },
        ExtTrapType::Web => TrapTriggerResult::Immobilize {
            turns: rng.rn2(3) + 2,
        },
        ExtTrapType::StatueTrap => TrapTriggerResult::Alert { noise_radius: 5 },
        ExtTrapType::MagicTrap => {
            let effects = ["가속", "감속", "투명", "혼란"];
            let idx = rng.rn2(effects.len() as i32) as usize;
            TrapTriggerResult::StatusEffect {
                effect: effects[idx].to_string(),
                turns: rng.rn2(20) + 5,
            }
        }
        ExtTrapType::AntiMagicTrap => TrapTriggerResult::StatusEffect {
            effect: "마법 봉인".to_string(),
            turns: rng.rn2(10) + 10,
        },
        ExtTrapType::PolymorphTrap => TrapTriggerResult::Polymorph,
    }
}

// =============================================================================
// [2] 함정 해체 — trap_disarm (trap.c L2500-3000)
// =============================================================================

/// [v2.32.0 96-2] 해체 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisarmResult {
    Success,
    Failed { triggered: bool },
    NeedTool { tool: String },
    CannotDisarm,
}

/// [v2.32.0 96-2] 함정 해체 시도
pub fn disarm_trap(
    trap: ExtTrapType,
    player_dex: i32,
    player_level: i32,
    has_tools: bool,
    rng: &mut NetHackRng,
) -> DisarmResult {
    // 해체 불가 함정
    match trap {
        ExtTrapType::MagicPortal | ExtTrapType::LevelTeleport => {
            return DisarmResult::CannotDisarm;
        }
        _ => {}
    }

    // 도구 필요
    if matches!(trap, ExtTrapType::LandMine | ExtTrapType::BearTrap) && !has_tools {
        return DisarmResult::NeedTool {
            tool: "해체 도구".to_string(),
        };
    }

    let difficulty = match trap {
        ExtTrapType::LandMine => 15,
        ExtTrapType::RollingBoulder => 12,
        ExtTrapType::FireTrap => 10,
        _ => 8,
    };

    let skill = player_dex + player_level;
    if rng.rn2(20) + skill / 2 > difficulty {
        DisarmResult::Success
    } else {
        DisarmResult::Failed {
            triggered: rng.rn2(3) == 0,
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
    fn test_arrow_trap() {
        let mut rng = test_rng();
        let result = trigger_trap(ExtTrapType::ArrowTrap, 10, false, false, &mut rng);
        assert!(matches!(
            result,
            TrapTriggerResult::Damage { .. } | TrapTriggerResult::NoEffect { .. }
        ));
    }

    #[test]
    fn test_bear_trap() {
        let mut rng = test_rng();
        let result = trigger_trap(ExtTrapType::BearTrap, 10, false, false, &mut rng);
        assert!(matches!(result, TrapTriggerResult::Immobilize { .. }));
    }

    #[test]
    fn test_levitation_dodge() {
        let mut rng = test_rng();
        let result = trigger_trap(ExtTrapType::Pit, 10, true, false, &mut rng);
        assert!(matches!(result, TrapTriggerResult::NoEffect { .. }));
    }

    #[test]
    fn test_teleport_trap() {
        let mut rng = test_rng();
        let result = trigger_trap(ExtTrapType::TeleportTrap, 10, false, false, &mut rng);
        assert!(matches!(result, TrapTriggerResult::Teleport { .. }));
    }

    #[test]
    fn test_polymorph_trap() {
        let mut rng = test_rng();
        let result = trigger_trap(ExtTrapType::PolymorphTrap, 10, false, false, &mut rng);
        assert!(matches!(result, TrapTriggerResult::Polymorph));
    }

    #[test]
    fn test_disarm_success() {
        let mut success = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = disarm_trap(ExtTrapType::ArrowTrap, 16, 10, true, &mut rng);
            if matches!(result, DisarmResult::Success) {
                success = true;
                break;
            }
        }
        assert!(success);
    }

    #[test]
    fn test_disarm_no_tool() {
        let mut rng = test_rng();
        let result = disarm_trap(ExtTrapType::LandMine, 16, 10, false, &mut rng);
        assert!(matches!(result, DisarmResult::NeedTool { .. }));
    }

    #[test]
    fn test_disarm_cannot() {
        let mut rng = test_rng();
        let result = disarm_trap(ExtTrapType::MagicPortal, 16, 10, true, &mut rng);
        assert!(matches!(result, DisarmResult::CannotDisarm));
    }
}
