// ============================================================================
// [v2.40.0 Phase 104-2] 함정/위험 통합 (hazard_phase104_ext.rs)
// 원본: NetHack 3.6.7 src/trap.c 미이식 함정 유형 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 15종 함정 유형 및 효과
//   - 함정 감지/해제 판정
//   - 함정 피해 계산
//   - 함정 회피 (민첩/레벨 기반)
//   - 던전 함정 배치 계산
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.40.0 104-2] 함정 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HazardType {
    ArrowTrap,       // 화살 함정
    DartTrap,        // 다트 함정
    FallingRock,     // 낙석 함정
    Pit,             // 구덩이
    SpikedPit,       // 가시 구덩이
    BearTrap,        // 곰 함정
    LandMine,        // 지뢰
    SqueakyBoard,    // 삐걱거리는 바닥
    MagicPortal,     // 마법 포탈
    Teleporter,      // 텔레포터
    LevelTeleporter, // 레벨 텔레포터
    FireTrap,        // 불 함정
    SleepGas,        // 수면 가스
    RustTrap,        // 녹 함정
    PolymorphTrap,   // 변이 함정
}

/// [v2.40.0 104-2] 함정 작동 결과
#[derive(Debug, Clone)]
pub struct HazardResult {
    pub hazard: HazardType,
    pub damage: i32,
    pub status_effect: Option<String>,
    pub message: String,
    pub avoided: bool,
}

/// [v2.40.0 104-2] 함정 작동
pub fn trigger_hazard(
    hazard: HazardType,
    player_dex: i32,
    player_level: i32,
    has_levitation: bool,
    rng: &mut NetHackRng,
) -> HazardResult {
    // 회피 판정: (민첩 + 레벨) / 2 > 난수(20)
    let avoid_check = (player_dex + player_level) / 2;
    let roll = rng.rn2(20) + 1;
    let base_avoided = avoid_check > roll;

    match hazard {
        HazardType::Pit | HazardType::SpikedPit => {
            if has_levitation {
                return HazardResult {
                    hazard,
                    damage: 0,
                    status_effect: None,
                    message: "공중에 떠 있어 구덩이를 피했다!".to_string(),
                    avoided: true,
                };
            }
            let dmg = if hazard == HazardType::SpikedPit {
                rng.rn2(6) + 8
            } else {
                rng.rn2(6) + 2
            };
            HazardResult {
                hazard,
                damage: dmg,
                status_effect: None,
                message: format!("구덩이에 빠졌다! ({} 데미지)", dmg),
                avoided: false,
            }
        }
        HazardType::ArrowTrap => {
            if base_avoided {
                HazardResult {
                    hazard,
                    damage: 0,
                    status_effect: None,
                    message: "화살을 피했다!".to_string(),
                    avoided: true,
                }
            } else {
                let dmg = rng.rn2(6) + 3;
                HazardResult {
                    hazard,
                    damage: dmg,
                    status_effect: None,
                    message: format!("화살에 맞았다! ({} 데미지)", dmg),
                    avoided: false,
                }
            }
        }
        HazardType::FireTrap => {
            let dmg = rng.rn2(10) + 5;
            HazardResult {
                hazard,
                damage: dmg,
                status_effect: Some("화상".to_string()),
                message: format!("불길이 솟아올랐다! ({} 데미지)", dmg),
                avoided: false,
            }
        }
        HazardType::SleepGas => {
            if base_avoided {
                HazardResult {
                    hazard,
                    damage: 0,
                    status_effect: None,
                    message: "가스를 피했다!".to_string(),
                    avoided: true,
                }
            } else {
                HazardResult {
                    hazard,
                    damage: 0,
                    status_effect: Some("수면".to_string()),
                    message: "수면 가스에 쓰러진다!".to_string(),
                    avoided: false,
                }
            }
        }
        HazardType::Teleporter => HazardResult {
            hazard,
            damage: 0,
            status_effect: Some("텔레포트".to_string()),
            message: "공간이 비틀린다!".to_string(),
            avoided: false,
        },
        HazardType::PolymorphTrap => HazardResult {
            hazard,
            damage: 0,
            status_effect: Some("변이".to_string()),
            message: "몸이 변하기 시작한다!".to_string(),
            avoided: false,
        },
        HazardType::BearTrap => {
            if has_levitation {
                return HazardResult {
                    hazard,
                    damage: 0,
                    status_effect: None,
                    message: "공중에서 곰 함정을 피했다.".to_string(),
                    avoided: true,
                };
            }
            HazardResult {
                hazard,
                damage: rng.rn2(4) + 2,
                status_effect: Some("속박".to_string()),
                message: "곰 함정에 걸렸다!".to_string(),
                avoided: false,
            }
        }
        HazardType::LandMine => {
            if has_levitation {
                return HazardResult {
                    hazard,
                    damage: 0,
                    status_effect: None,
                    message: "지뢰 위를 떠다닌다.".to_string(),
                    avoided: true,
                };
            }
            let dmg = rng.rn2(16) + 10;
            HazardResult {
                hazard,
                damage: dmg,
                status_effect: None,
                message: format!("💥 지뢰가 폭발했다! ({} 데미지)", dmg),
                avoided: false,
            }
        }
        _ => {
            let dmg = rng.rn2(4) + 1;
            HazardResult {
                hazard,
                damage: dmg,
                status_effect: None,
                message: format!("{:?} 함정 작동! ({} 데미지)", hazard, dmg),
                avoided: false,
            }
        }
    }
}

/// [v2.40.0 104-2] 함정 감지
pub fn detect_hazard(perception: i32, trap_difficulty: i32, rng: &mut NetHackRng) -> bool {
    let chance = (perception * 5 - trap_difficulty * 3).max(5).min(95);
    rng.rn2(100) < chance
}

/// [v2.40.0 104-2] 함정 해제
pub fn disarm_hazard(
    dex: i32,
    level: i32,
    trap_difficulty: i32,
    rng: &mut NetHackRng,
) -> (bool, String) {
    let chance = ((dex + level) * 3 - trap_difficulty * 5).max(5).min(90);
    if rng.rn2(100) < chance {
        (true, "함정을 해제했다!".to_string())
    } else {
        (false, "함정 해제에 실패했다! 함정이 작동한다!".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_pit_levitate() {
        let mut rng = test_rng();
        let r = trigger_hazard(HazardType::Pit, 14, 10, true, &mut rng);
        assert!(r.avoided);
        assert_eq!(r.damage, 0);
    }

    #[test]
    fn test_fire_trap() {
        let mut rng = test_rng();
        let r = trigger_hazard(HazardType::FireTrap, 14, 10, false, &mut rng);
        assert!(r.damage > 0);
        assert!(r.status_effect.is_some());
    }

    #[test]
    fn test_teleporter() {
        let mut rng = test_rng();
        let r = trigger_hazard(HazardType::Teleporter, 14, 10, false, &mut rng);
        assert!(r.status_effect.unwrap().contains("텔레포트"));
    }

    #[test]
    fn test_landmine() {
        let mut rng = test_rng();
        let r = trigger_hazard(HazardType::LandMine, 14, 10, false, &mut rng);
        assert!(r.damage >= 10);
    }

    #[test]
    fn test_landmine_levitate() {
        let mut rng = test_rng();
        let r = trigger_hazard(HazardType::LandMine, 14, 10, true, &mut rng);
        assert!(r.avoided);
    }

    #[test]
    fn test_disarm_success() {
        let mut rng = NetHackRng::new(1);
        let (ok, _) = disarm_hazard(18, 15, 5, &mut rng);
        // 높은 민첩+레벨, 낮은 난이도
        assert!(ok);
    }

    #[test]
    fn test_detect() {
        let mut rng = NetHackRng::new(1);
        let detected = detect_hazard(18, 3, &mut rng);
        assert!(detected);
    }
}
