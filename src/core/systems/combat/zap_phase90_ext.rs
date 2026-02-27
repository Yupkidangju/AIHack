// ============================================================================
// [v2.26.0 Phase 90-5] 마법 지팡이 확장 (zap_phase90_ext.rs)
// 원본: NetHack 3.6.7 src/zap.c L2000-4000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 빔 효과 — beam_effect (zap.c L2000-2800)
// =============================================================================

/// [v2.26.0 90-5] 빔 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamType {
    Fire,
    Cold,
    Lightning,
    Sleep,
    Death,
    Disintegrate,
    Poison,
    Acid,
    MagicMissile,
    Petrification,
    Polymorph,
    Teleport,
    Digging,
    Light,
}

/// [v2.26.0 90-5] 빔이 대상에 미치는 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeamHitEffect {
    /// 데미지
    Damage { amount: i32, element: String },
    /// 즉사
    InstantDeath { message: String },
    /// 소멸 (분해)
    Disintegrate,
    /// 수면
    Sleep { turns: i32 },
    /// 석화
    Petrify,
    /// 변이
    Polymorph,
    /// 텔레포트
    Teleport,
    /// 저항으로 무효화
    Resisted { beam: BeamType },
    /// 반사
    Reflected { beam: BeamType },
}

/// [v2.26.0 90-5] 빔 효과 판정
/// 원본: zap.c buzz() + zhitm()
pub fn beam_hit_effect(
    beam: BeamType,
    beam_damage_dice: i32,
    target_has_resistance: bool,
    target_has_reflection: bool,
    target_magic_resistance: bool,
    rng: &mut NetHackRng,
) -> BeamHitEffect {
    // [1] 반사 검사
    if target_has_reflection
        && matches!(
            beam,
            BeamType::Fire
                | BeamType::Cold
                | BeamType::Lightning
                | BeamType::Death
                | BeamType::MagicMissile
        )
    {
        return BeamHitEffect::Reflected { beam };
    }

    // [2] 빔별 효과
    match beam {
        BeamType::Fire => {
            if target_has_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                let damage = roll_dice(beam_damage_dice, 6, rng);
                BeamHitEffect::Damage {
                    amount: damage,
                    element: "화염".to_string(),
                }
            }
        }
        BeamType::Cold => {
            if target_has_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                let damage = roll_dice(beam_damage_dice, 6, rng);
                BeamHitEffect::Damage {
                    amount: damage,
                    element: "냉기".to_string(),
                }
            }
        }
        BeamType::Lightning => {
            if target_has_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                let damage = roll_dice(beam_damage_dice, 6, rng);
                BeamHitEffect::Damage {
                    amount: damage,
                    element: "전격".to_string(),
                }
            }
        }
        BeamType::Sleep => {
            if target_has_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                BeamHitEffect::Sleep {
                    turns: rng.rn2(25) + 5,
                }
            }
        }
        BeamType::Death => {
            if target_magic_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                BeamHitEffect::InstantDeath {
                    message: "죽음의 빔에 맞았다!".to_string(),
                }
            }
        }
        BeamType::Disintegrate => {
            if target_has_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                BeamHitEffect::Disintegrate
            }
        }
        BeamType::Poison => {
            if target_has_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                let damage = roll_dice(beam_damage_dice, 4, rng);
                BeamHitEffect::Damage {
                    amount: damage,
                    element: "독".to_string(),
                }
            }
        }
        BeamType::Acid => {
            if target_has_resistance {
                let half_damage = roll_dice(beam_damage_dice, 6, rng) / 2;
                BeamHitEffect::Damage {
                    amount: half_damage.max(1),
                    element: "산성(저항)".to_string(),
                }
            } else {
                let damage = roll_dice(beam_damage_dice, 6, rng);
                BeamHitEffect::Damage {
                    amount: damage,
                    element: "산성".to_string(),
                }
            }
        }
        BeamType::MagicMissile => {
            if target_magic_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                let damage = roll_dice(beam_damage_dice, 6, rng) + 1;
                BeamHitEffect::Damage {
                    amount: damage,
                    element: "마법".to_string(),
                }
            }
        }
        BeamType::Petrification => {
            if target_has_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                BeamHitEffect::Petrify
            }
        }
        BeamType::Polymorph => {
            if target_has_resistance {
                BeamHitEffect::Resisted { beam }
            } else {
                BeamHitEffect::Polymorph
            }
        }
        BeamType::Teleport => BeamHitEffect::Teleport,
        BeamType::Digging => BeamHitEffect::Damage {
            amount: 0,
            element: "채굴".to_string(),
        },
        BeamType::Light => BeamHitEffect::Damage {
            amount: roll_dice(beam_damage_dice, 4, rng),
            element: "빛".to_string(),
        },
    }
}

/// 주사위 굴림 (NdM)
fn roll_dice(num: i32, sides: i32, rng: &mut NetHackRng) -> i32 {
    (0..num).map(|_| rng.rn2(sides) + 1).sum()
}

// =============================================================================
// [2] 지팡이 충전 소모 — wand_charge (zap.c L3000-3200)
// =============================================================================

/// [v2.26.0 90-5] 지팡이 충전 소모 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WandChargeResult {
    /// 정상 사용
    Used { remaining: i32 },
    /// 마지막 충전
    LastCharge,
    /// 충전 없음 (비어있음)
    Empty,
    /// 폭발 (과충전/저주)
    Explode { damage: i32 },
}

/// [v2.26.0 90-5] 지팡이 충전 소모
pub fn use_wand_charge(
    charges: i32,
    is_cursed: bool,
    recharge_count: i32,
    rng: &mut NetHackRng,
) -> WandChargeResult {
    if charges <= 0 {
        // 빈 지팡이 — 과충전 폭발 확인
        if recharge_count >= 2 && rng.rn2(3) == 0 {
            return WandChargeResult::Explode {
                damage: rng.rn2(20) + 10,
            };
        }
        return WandChargeResult::Empty;
    }

    let new_charges = charges - 1;

    // 저주 시 추가 소모
    let extra_drain = if is_cursed && rng.rn2(3) == 0 { 1 } else { 0 };
    let final_charges = (new_charges - extra_drain).max(0);

    if final_charges == 0 {
        WandChargeResult::LastCharge
    } else {
        WandChargeResult::Used {
            remaining: final_charges,
        }
    }
}

// =============================================================================
// [3] 빔 경로 계산 — beam_path (zap.c L3500-3700)
// =============================================================================

/// [v2.26.0 90-5] 빔 경로의 한 타일
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeamPathTile {
    pub x: i32,
    pub y: i32,
    pub blocked: bool,
    pub has_monster: bool,
}

/// [v2.26.0 90-5] 빔 경로 계산 (직선)
/// 원본: zap.c buzz() 경로 계산 부분
pub fn beam_path(
    start_x: i32,
    start_y: i32,
    dx: i32,
    dy: i32,
    max_range: i32,
    map_width: i32,
    map_height: i32,
    is_wall: &dyn Fn(i32, i32) -> bool,
    has_monster: &dyn Fn(i32, i32) -> bool,
) -> Vec<BeamPathTile> {
    let mut path = Vec::new();

    for step in 1..=max_range {
        let x = start_x + dx * step;
        let y = start_y + dy * step;

        if x < 0 || x >= map_width || y < 0 || y >= map_height {
            break;
        }

        let blocked = is_wall(x, y);
        let monster = has_monster(x, y);

        path.push(BeamPathTile {
            x,
            y,
            blocked,
            has_monster: monster,
        });

        if blocked {
            break; // 빔은 벽에서 멈춤
        }
    }

    path
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

    // --- beam_hit_effect ---

    #[test]
    fn test_fire_damage() {
        let mut rng = test_rng();
        let result = beam_hit_effect(BeamType::Fire, 6, false, false, false, &mut rng);
        assert!(matches!(result, BeamHitEffect::Damage { .. }));
    }

    #[test]
    fn test_fire_resisted() {
        let mut rng = test_rng();
        let result = beam_hit_effect(BeamType::Fire, 6, true, false, false, &mut rng);
        assert!(matches!(result, BeamHitEffect::Resisted { .. }));
    }

    #[test]
    fn test_fire_reflected() {
        let mut rng = test_rng();
        let result = beam_hit_effect(BeamType::Fire, 6, false, true, false, &mut rng);
        assert!(matches!(result, BeamHitEffect::Reflected { .. }));
    }

    #[test]
    fn test_death_beam() {
        let mut rng = test_rng();
        let result = beam_hit_effect(BeamType::Death, 1, false, false, false, &mut rng);
        assert!(matches!(result, BeamHitEffect::InstantDeath { .. }));
    }

    #[test]
    fn test_death_magic_resist() {
        let mut rng = test_rng();
        let result = beam_hit_effect(BeamType::Death, 1, false, false, true, &mut rng);
        assert!(matches!(result, BeamHitEffect::Resisted { .. }));
    }

    #[test]
    fn test_sleep_beam() {
        let mut rng = test_rng();
        let result = beam_hit_effect(BeamType::Sleep, 1, false, false, false, &mut rng);
        assert!(matches!(result, BeamHitEffect::Sleep { .. }));
    }

    // --- wand_charge ---

    #[test]
    fn test_wand_use() {
        let mut rng = test_rng();
        let result = use_wand_charge(5, false, 0, &mut rng);
        assert!(matches!(result, WandChargeResult::Used { remaining: 4 }));
    }

    #[test]
    fn test_wand_last() {
        let mut rng = test_rng();
        let result = use_wand_charge(1, false, 0, &mut rng);
        assert!(matches!(result, WandChargeResult::LastCharge));
    }

    #[test]
    fn test_wand_empty() {
        let mut rng = test_rng();
        let result = use_wand_charge(0, false, 0, &mut rng);
        assert!(matches!(result, WandChargeResult::Empty));
    }

    // --- beam_path ---

    #[test]
    fn test_beam_path_straight() {
        let path = beam_path(5, 5, 1, 0, 10, 80, 21, &|_, _| false, &|_, _| false);
        assert_eq!(path.len(), 10);
        assert_eq!(path[0].x, 6);
    }

    #[test]
    fn test_beam_path_wall() {
        let path = beam_path(5, 5, 1, 0, 10, 80, 21, &|x, _| x == 8, &|_, _| false);
        assert_eq!(path.len(), 3); // 6, 7, 8(벽=멈춤)
        assert!(path.last().unwrap().blocked);
    }

    #[test]
    fn test_beam_path_edge() {
        let path = beam_path(78, 5, 1, 0, 10, 80, 21, &|_, _| false, &|_, _| false);
        assert_eq!(path.len(), 1); // 79만 유효
    }
}
