// ============================================================================
// [v2.27.0 Phase 91-2] 함정 시스템 확장 (trap_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/trap.c L2000-4000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 함정 발동 효과 — dotrap_extended (trap.c L2000-3000)
// =============================================================================

/// [v2.27.0 91-2] 함정 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    ArrowTrap,
    DartTrap,
    RockTrap,
    SqueakyBoard,
    BearTrap,
    LandMine,
    RollingBoulder,
    SleepingGas,
    RustTrap,
    PitTrap,
    SpikedPit,
    HoleTrap,
    TrapDoor,
    TeleportTrap,
    LevelTeleporter,
    FireTrap,
    MagicTrap,
    WebTrap,
    AntiMagicTrap,
    PolymorphTrap,
}

/// [v2.27.0 91-2] 함정 발동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapResult {
    /// 데미지
    Damage { amount: i32, source: String },
    /// 상태 이상
    StatusEffect { effect: String, turns: i32 },
    /// 장비 손상
    EquipDamage { slot: String, damage_type: String },
    /// 텔레포트
    Teleport { random: bool },
    /// 레벨 변경
    LevelChange { direction: i32 },
    /// 던전 구멍 (낙하)
    Fall { depth: i32, damage: i32 },
    /// 변이
    Polymorph,
    /// 경보 (몬스터 각성)
    Alarm { radius: i32 },
    /// 마법 취소
    AntiMagic { drain_amount: i32 },
    /// 걸림 (이동 불가)
    Stuck { turns: i32 },
    /// 회피 성공
    Evaded { message: String },
    /// 부동/비행으로 회피
    Immune { reason: String },
}

/// [v2.27.0 91-2] 함정 발동 입력
#[derive(Debug, Clone)]
pub struct TrapTriggerInput {
    pub trap_type: TrapType,
    pub player_dex: i32,
    pub is_levitating: bool,
    pub is_flying: bool,
    pub has_fire_resist: bool,
    pub has_sleep_resist: bool,
    pub has_poly_resist: bool,
    pub is_wearing_boots: bool,
    pub weight: i32, // 체중 (곰함정/지뢰용)
}

/// [v2.27.0 91-2] 함정 발동 효과 판정
/// 원본: trap.c dotrap() 전체 분기
pub fn trap_trigger(input: &TrapTriggerInput, rng: &mut NetHackRng) -> TrapResult {
    match input.trap_type {
        TrapType::ArrowTrap => {
            // DEX 기반 회피 (1/3 + DEX 보너스)
            if rng.rn2(3) == 0 || (input.player_dex > 14 && rng.rn2(4) == 0) {
                return TrapResult::Evaded {
                    message: "화살을 피했다!".to_string(),
                };
            }
            TrapResult::Damage {
                amount: rng.rn2(6) + 1,
                source: "화살 함정".to_string(),
            }
        }
        TrapType::DartTrap => {
            if rng.rn2(4) == 0 {
                return TrapResult::Evaded {
                    message: "다트를 피했다!".to_string(),
                };
            }
            TrapResult::Damage {
                amount: rng.rn2(4) + 1,
                source: "독 다트 함정".to_string(),
            }
        }
        TrapType::RockTrap => TrapResult::Damage {
            amount: rng.rn2(20) + 10,
            source: "떨어지는 바위".to_string(),
        },
        TrapType::SqueakyBoard => TrapResult::Alarm { radius: 20 },
        TrapType::BearTrap => {
            if input.is_levitating || input.is_flying {
                return TrapResult::Immune {
                    reason: "공중에 떠 있어 회피".to_string(),
                };
            }
            TrapResult::Stuck {
                turns: rng.rn2(4) + 3,
            }
        }
        TrapType::LandMine => {
            if input.is_levitating || input.is_flying {
                return TrapResult::Immune {
                    reason: "부유로 지뢰 회피".to_string(),
                };
            }
            TrapResult::Damage {
                amount: rng.rn2(16) + 16 + input.weight / 100,
                source: "지뢰".to_string(),
            }
        }
        TrapType::RollingBoulder => {
            if rng.rn2(3) == 0 {
                return TrapResult::Evaded {
                    message: "굴러오는 바위를 피했다!".to_string(),
                };
            }
            TrapResult::Damage {
                amount: rng.rn2(20) + 10,
                source: "굴러오는 바위".to_string(),
            }
        }
        TrapType::SleepingGas => {
            if input.has_sleep_resist {
                return TrapResult::Immune {
                    reason: "수면 저항".to_string(),
                };
            }
            TrapResult::StatusEffect {
                effect: "수면".to_string(),
                turns: rng.rn2(20) + 5,
            }
        }
        TrapType::RustTrap => TrapResult::EquipDamage {
            slot: "armor".to_string(),
            damage_type: "녹".to_string(),
        },
        TrapType::PitTrap => {
            if input.is_levitating || input.is_flying {
                return TrapResult::Immune {
                    reason: "부유로 구덩이 회피".to_string(),
                };
            }
            TrapResult::Damage {
                amount: rng.rn2(6) + 2,
                source: "구덩이".to_string(),
            }
        }
        TrapType::SpikedPit => {
            if input.is_levitating || input.is_flying {
                return TrapResult::Immune {
                    reason: "부유로 가시 구덩이 회피".to_string(),
                };
            }
            let spike_damage = rng.rn2(10) + 1;
            let fall_damage = rng.rn2(6) + 2;
            TrapResult::Damage {
                amount: spike_damage + fall_damage,
                source: "가시 구덩이".to_string(),
            }
        }
        TrapType::HoleTrap | TrapType::TrapDoor => {
            if input.is_levitating || input.is_flying {
                return TrapResult::Immune {
                    reason: "부유로 구멍 회피".to_string(),
                };
            }
            TrapResult::Fall {
                depth: 1,
                damage: rng.rn2(8) + 4,
            }
        }
        TrapType::TeleportTrap => TrapResult::Teleport { random: true },
        TrapType::LevelTeleporter => TrapResult::LevelChange {
            direction: if rng.rn2(2) == 0 { 1 } else { -1 },
        },
        TrapType::FireTrap => {
            if input.has_fire_resist {
                return TrapResult::Immune {
                    reason: "화염 저항".to_string(),
                };
            }
            TrapResult::Damage {
                amount: rng.rn2(12) + 8,
                source: "화염 함정".to_string(),
            }
        }
        TrapType::MagicTrap => {
            // 무작위 마법 효과 중 하나
            let effect = rng.rn2(4);
            match effect {
                0 => TrapResult::StatusEffect {
                    effect: "혼란".to_string(),
                    turns: rng.rn2(10) + 5,
                },
                1 => TrapResult::StatusEffect {
                    effect: "실명".to_string(),
                    turns: rng.rn2(15) + 5,
                },
                2 => TrapResult::Teleport { random: true },
                _ => TrapResult::Damage {
                    amount: rng.rn2(8) + 1,
                    source: "마법 폭발".to_string(),
                },
            }
        }
        TrapType::WebTrap => TrapResult::Stuck {
            turns: rng.rn2(6) + 2,
        },
        TrapType::AntiMagicTrap => TrapResult::AntiMagic {
            drain_amount: rng.rn2(10) + 5,
        },
        TrapType::PolymorphTrap => {
            if input.has_poly_resist {
                return TrapResult::Immune {
                    reason: "변이 저항".to_string(),
                };
            }
            TrapResult::Polymorph
        }
    }
}

// =============================================================================
// [2] 함정 탐지 확률 — find_trap (trap.c L500-600)
// =============================================================================

/// [v2.27.0 91-2] 함정 탐지 확률 계산
pub fn trap_detection_chance(
    player_level: i32,
    player_luck: i32,
    is_searching: bool,
    has_trap_detection: bool,
) -> i32 {
    if has_trap_detection {
        return 100; // 자동 탐지
    }

    let base = if is_searching { 20 } else { 5 };
    let level_bonus = player_level * 2;
    let luck_bonus = player_luck.max(0) * 3;

    (base + level_bonus + luck_bonus).min(95)
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

    fn base_input(trap_type: TrapType) -> TrapTriggerInput {
        TrapTriggerInput {
            trap_type,
            player_dex: 14,
            is_levitating: false,
            is_flying: false,
            has_fire_resist: false,
            has_sleep_resist: false,
            has_poly_resist: false,
            is_wearing_boots: true,
            weight: 1500,
        }
    }

    #[test]
    fn test_arrow_trap_damage() {
        // 여러 시드로 최소 1번은 데미지
        let mut got_damage = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = trap_trigger(&base_input(TrapType::ArrowTrap), &mut rng);
            if matches!(result, TrapResult::Damage { .. }) {
                got_damage = true;
                break;
            }
        }
        assert!(got_damage);
    }

    #[test]
    fn test_bear_trap_levitating() {
        let mut rng = test_rng();
        let mut input = base_input(TrapType::BearTrap);
        input.is_levitating = true;
        let result = trap_trigger(&input, &mut rng);
        assert!(matches!(result, TrapResult::Immune { .. }));
    }

    #[test]
    fn test_fire_trap_resistant() {
        let mut rng = test_rng();
        let mut input = base_input(TrapType::FireTrap);
        input.has_fire_resist = true;
        let result = trap_trigger(&input, &mut rng);
        assert!(matches!(result, TrapResult::Immune { .. }));
    }

    #[test]
    fn test_sleeping_gas_resistant() {
        let mut rng = test_rng();
        let mut input = base_input(TrapType::SleepingGas);
        input.has_sleep_resist = true;
        let result = trap_trigger(&input, &mut rng);
        assert!(matches!(result, TrapResult::Immune { .. }));
    }

    #[test]
    fn test_teleport_trap() {
        let mut rng = test_rng();
        let result = trap_trigger(&base_input(TrapType::TeleportTrap), &mut rng);
        assert!(matches!(result, TrapResult::Teleport { random: true }));
    }

    #[test]
    fn test_polymorph_trap() {
        let mut rng = test_rng();
        let result = trap_trigger(&base_input(TrapType::PolymorphTrap), &mut rng);
        assert!(matches!(result, TrapResult::Polymorph));
    }

    #[test]
    fn test_squeaky_board() {
        let mut rng = test_rng();
        let result = trap_trigger(&base_input(TrapType::SqueakyBoard), &mut rng);
        assert!(matches!(result, TrapResult::Alarm { .. }));
    }

    #[test]
    fn test_web_trap() {
        let mut rng = test_rng();
        let result = trap_trigger(&base_input(TrapType::WebTrap), &mut rng);
        assert!(matches!(result, TrapResult::Stuck { .. }));
    }

    #[test]
    fn test_pit_levitating() {
        let mut rng = test_rng();
        let mut input = base_input(TrapType::PitTrap);
        input.is_flying = true;
        let result = trap_trigger(&input, &mut rng);
        assert!(matches!(result, TrapResult::Immune { .. }));
    }

    #[test]
    fn test_detection_with_ability() {
        assert_eq!(trap_detection_chance(10, 0, false, true), 100);
    }

    #[test]
    fn test_detection_searching() {
        let chance = trap_detection_chance(10, 3, true, false);
        assert!(chance > 30);
    }
}
