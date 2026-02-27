// ============================================================================
// [v2.28.0 Phase 92-2] 던지기 시스템 확장 (dothrow_phase92_ext.rs)
// 원본: NetHack 3.6.7 src/dothrow.c L400-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 던지기 명중/데미지 — throw_calc (dothrow.c L400-700)
// =============================================================================

/// [v2.28.0 92-2] 투사체 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileType {
    Rock,
    Dart,
    Shuriken,
    Arrow,
    Crossbow,
    Javelin,
    Spear,
    Dagger,
    Axe,
    Boomerang,
    Cream,
    Egg,
    Potion,
    Other,
}

/// [v2.28.0 92-2] 투사체 비행 경로 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThrowTrajectory {
    pub path: Vec<(i32, i32)>,
    pub max_range: i32,
    pub hit_obstacle_at: Option<(i32, i32)>,
}

/// [v2.28.0 92-2] 투사체 비행 경로 계산
/// 원본: dothrow.c hurtle_step() / throwit()
pub fn calculate_throw_path(
    start_x: i32,
    start_y: i32,
    dx: i32,
    dy: i32,
    strength: i32,
    projectile: ProjectileType,
    is_wall: &dyn Fn(i32, i32) -> bool,
) -> ThrowTrajectory {
    // 사거리: STR 기반 + 투사체 보너스
    let base_range = strength / 2 + 1;
    let bonus = match projectile {
        ProjectileType::Rock => 0,
        ProjectileType::Dart | ProjectileType::Shuriken => 2,
        ProjectileType::Arrow | ProjectileType::Crossbow => 3,
        ProjectileType::Javelin | ProjectileType::Spear => 2,
        ProjectileType::Dagger => 1,
        ProjectileType::Axe => 1,
        ProjectileType::Boomerang => 4,
        ProjectileType::Cream | ProjectileType::Egg | ProjectileType::Potion => 1,
        ProjectileType::Other => 0,
    };
    let max_range = (base_range + bonus).min(20);

    let mut path = Vec::new();
    let mut hit_obstacle = None;
    let mut cx = start_x;
    let mut cy = start_y;

    for _ in 0..max_range {
        cx += dx;
        cy += dy;

        if is_wall(cx, cy) {
            hit_obstacle = Some((cx, cy));
            break;
        }
        path.push((cx, cy));
    }

    ThrowTrajectory {
        path,
        max_range,
        hit_obstacle_at: hit_obstacle,
    }
}

// =============================================================================
// [2] 투사체 명중/데미지 판정 (dothrow.c L700-1000)
// =============================================================================

/// [v2.28.0 92-2] 투사체 명중 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThrowHitResult {
    /// 명중
    Hit {
        damage: i32,
        bonus_effect: Option<String>,
    },
    /// 빗나감
    Miss,
    /// 깨짐 (포션/달걀)
    Shatter { splash_damage: i32, effect: String },
    /// 부메랑 귀환
    BoomerangReturn,
}

/// [v2.28.0 92-2] 투사체 명중 판정
/// 원본: dothrow.c thitmonst()
pub fn throw_hit_check(
    projectile: ProjectileType,
    thrower_dex: i32,
    thrower_level: i32,
    target_ac: i32,
    distance: i32,
    has_launcher: bool,
    rng: &mut NetHackRng,
) -> ThrowHitResult {
    // 부메랑 특수 처리
    if projectile == ProjectileType::Boomerang {
        if rng.rn2(10) < 7 {
            return ThrowHitResult::BoomerangReturn;
        }
    }

    // 포션/달걀 → 깨짐
    if projectile == ProjectileType::Potion || projectile == ProjectileType::Egg {
        return ThrowHitResult::Shatter {
            splash_damage: rng.rn2(4) + 1,
            effect: if projectile == ProjectileType::Potion {
                "포션 효과 적용".to_string()
            } else {
                "달걀 깨짐".to_string()
            },
        };
    }

    // 명중 판정: 1d20 + DEX_bonus + level - distance_penalty >= target_AC + 10
    let roll = rng.rn2(20) + 1;
    let dex_bonus = (thrower_dex - 10) / 2;
    let launcher_bonus = if has_launcher { 3 } else { 0 };
    let distance_penalty = distance / 3;

    let to_hit = roll + dex_bonus + thrower_level + launcher_bonus - distance_penalty;
    let target = target_ac + 10;

    if to_hit < target {
        return ThrowHitResult::Miss;
    }

    // 데미지 계산
    let base_damage = match projectile {
        ProjectileType::Rock => rng.rn2(6) + 1,
        ProjectileType::Dart => rng.rn2(3) + 1,
        ProjectileType::Shuriken => rng.rn2(6) + 1,
        ProjectileType::Arrow => rng.rn2(6) + 1,
        ProjectileType::Crossbow => rng.rn2(8) + 1,
        ProjectileType::Javelin => rng.rn2(8) + 2,
        ProjectileType::Spear => rng.rn2(8) + 1,
        ProjectileType::Dagger => rng.rn2(4) + 1,
        ProjectileType::Axe => rng.rn2(6) + 1,
        _ => rng.rn2(4) + 1,
    };

    let launcher_dmg = if has_launcher { rng.rn2(4) + 1 } else { 0 };
    let damage = (base_damage + launcher_dmg).max(1);

    let bonus = match projectile {
        ProjectileType::Dart => Some("독 효과 가능".to_string()),
        ProjectileType::Shuriken => Some("다연발 가능".to_string()),
        _ => None,
    };

    ThrowHitResult::Hit {
        damage,
        bonus_effect: bonus,
    }
}

// =============================================================================
// [3] 부메랑 궤적 — boomerang (dothrow.c L1000-1200)
// =============================================================================

/// [v2.28.0 92-2] 부메랑 궤적 계산
pub fn boomerang_trajectory(
    start_x: i32,
    start_y: i32,
    dx: i32,
    dy: i32,
    range: i32,
) -> Vec<(i32, i32)> {
    let mut path = Vec::new();
    let half = range / 2;

    // 왕복 경로: 나가는 길
    for i in 1..=half {
        path.push((start_x + dx * i, start_y + dy * i));
    }
    // 돌아오는 길 (약간 옆으로 이탈)
    for i in (0..half).rev() {
        let offset = if i > 0 { 1 } else { 0 };
        path.push((start_x + dx * i + offset, start_y + dy * i));
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

    #[test]
    fn test_throw_path_open() {
        let traj = calculate_throw_path(5, 5, 1, 0, 16, ProjectileType::Arrow, &|_, _| false);
        assert!(traj.path.len() > 3);
        assert!(traj.hit_obstacle_at.is_none());
    }

    #[test]
    fn test_throw_path_wall() {
        let traj = calculate_throw_path(5, 5, 1, 0, 16, ProjectileType::Arrow, &|x, _| x >= 10);
        assert!(traj.hit_obstacle_at.is_some());
        assert_eq!(traj.hit_obstacle_at.unwrap().0, 10);
    }

    #[test]
    fn test_throw_hit_basic() {
        let mut got_hit = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = throw_hit_check(ProjectileType::Arrow, 16, 10, 5, 3, true, &mut rng);
            if matches!(result, ThrowHitResult::Hit { .. }) {
                got_hit = true;
                break;
            }
        }
        assert!(got_hit);
    }

    #[test]
    fn test_throw_potion_shatter() {
        let mut rng = test_rng();
        let result = throw_hit_check(ProjectileType::Potion, 14, 10, 5, 3, false, &mut rng);
        assert!(matches!(result, ThrowHitResult::Shatter { .. }));
    }

    #[test]
    fn test_boomerang_return() {
        let mut returns = false;
        for seed in 0..10u64 {
            let mut rng = NetHackRng::new(seed);
            let result = throw_hit_check(ProjectileType::Boomerang, 14, 10, 5, 3, false, &mut rng);
            if matches!(result, ThrowHitResult::BoomerangReturn) {
                returns = true;
                break;
            }
        }
        assert!(returns);
    }

    #[test]
    fn test_boomerang_trajectory() {
        let path = boomerang_trajectory(5, 5, 1, 0, 8);
        assert!(path.len() > 4);
        // 마지막 좌표가 시작 근처로 돌아옴
        let last = path.last().unwrap();
        assert!(last.0 <= 7);
    }

    #[test]
    fn test_throw_range_strength() {
        let weak = calculate_throw_path(0, 0, 1, 0, 8, ProjectileType::Rock, &|_, _| false);
        let strong = calculate_throw_path(0, 0, 1, 0, 18, ProjectileType::Rock, &|_, _| false);
        assert!(strong.path.len() >= weak.path.len());
    }
}
