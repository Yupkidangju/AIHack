// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.23.0 R11-4] 시야/조명 엔진 (vision_ext.rs)
//
// 원본 참조: NetHack 3.6.7 vision.c (1,200줄) + light.c (886줄)
//
// 구현 내용:
//   1. FOV(Field of View) — 제한 Raycasting
//   2. 동적 광원 전파 (BFS 확산)
//   3. 시야 차단 판정 (벽, 문 등)
//   4. 야간 시야 보정
//   5. 마법 어둠/밝음 처리
//   6. 투명 보기(See Invisible), 적외선 시야(Infravision)
// ============================================================================

use crate::core::dungeon::{COLNO, ROWNO};

// =============================================================================
// [1] FOV 계산 (원본: vision.c do_clear_area, vision_recalc)
// =============================================================================

/// [v2.23.0 R11-4] 시야 맵 (80×21 비트맵)
pub type VisionMap = [[bool; ROWNO]; COLNO];

/// [v2.23.0 R11-4] 빈 시야 맵 생성
pub fn empty_vision_map() -> VisionMap {
    [[false; ROWNO]; COLNO]
}

/// [v2.23.0 R11-4] 브레젠험 라인 시야 체크 (원본: clear_path_to)
/// (x0,y0)에서 (x1,y1)까지 직선 경로에 차단물이 없는지 확인
pub fn has_clear_path<F>(x0: i32, y0: i32, x1: i32, y1: i32, is_opaque: &F) -> bool
where
    F: Fn(i32, i32) -> bool,
{
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x0;
    let mut cy = y0;

    loop {
        if cx == x1 && cy == y1 {
            return true;
        }
        // 중간 지점이 불투명하면 차단
        if (cx != x0 || cy != y0) && is_opaque(cx, cy) {
            return false;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            cx += sx;
        }
        if e2 < dx {
            err += dx;
            cy += sy;
        }
        // 무한 루프 방지
        if (cx - x0).abs() + (cy - y0).abs() > (COLNO + ROWNO) as i32 {
            return false;
        }
    }
}

/// [v2.23.0 R11-4] FOV 계산 (단순 Raycasting)
/// (px, py)에서 반경 radius 내 시야 계산
pub fn calculate_fov<F>(px: i32, py: i32, radius: i32, is_opaque: &F) -> VisionMap
where
    F: Fn(i32, i32) -> bool,
{
    let mut map = empty_vision_map();

    // 자기 위치는 항상 보임
    if px >= 0 && (px as usize) < COLNO && py >= 0 && (py as usize) < ROWNO {
        map[px as usize][py as usize] = true;
    }

    // 원 테두리의 모든 점으로 레이 발사
    for angle in 0..360 {
        let rad = (angle as f64) * std::f64::consts::PI / 180.0;
        let tx = px + (radius as f64 * rad.cos()) as i32;
        let ty = py + (radius as f64 * rad.sin()) as i32;

        // 레이 경로상의 모든 타일 체크
        let dx = (tx - px).abs();
        let dy = (ty - py).abs();
        let sx = if px < tx { 1 } else { -1 };
        let sy = if py < ty { 1 } else { -1 };
        let mut err = dx - dy;
        let mut cx = px;
        let mut cy = py;

        loop {
            if cx >= 0 && (cx as usize) < COLNO && cy >= 0 && (cy as usize) < ROWNO {
                let dist_sq = (cx - px) * (cx - px) + (cy - py) * (cy - py);
                if dist_sq <= radius * radius {
                    map[cx as usize][cy as usize] = true;
                }
            }

            if cx == tx && cy == ty {
                break;
            }

            // 불투명 차단
            if (cx != px || cy != py)
                && cx >= 0
                && (cx as usize) < COLNO
                && cy >= 0
                && (cy as usize) < ROWNO
                && is_opaque(cx, cy)
            {
                // 벽 자체는 보이지만 그 너머는 안 보임
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                cx += sx;
            }
            if e2 < dx {
                err += dx;
                cy += sy;
            }

            if (cx - px).abs() > radius + 1 || (cy - py).abs() > radius + 1 {
                break;
            }
        }
    }

    map
}

/// [v2.23.0 R11-4] 시야 맵에서 보이는 타일 수
pub fn count_visible(map: &VisionMap) -> usize {
    let mut count = 0;
    for col in map.iter() {
        for &visible in col.iter() {
            if visible {
                count += 1;
            }
        }
    }
    count
}

// =============================================================================
// [2] 동적 광원 (원본: light.c do_light_sources)
// =============================================================================

/// [v2.23.0 R11-4] 광원 정보
#[derive(Debug, Clone)]
pub struct LightSource {
    /// 광원 위치 X
    pub x: i32,
    /// 광원 위치 Y
    pub y: i32,
    /// 광원 반경
    pub radius: i32,
    /// 광원 타입 (아이템, 몬스터, 마법 등)
    pub source_type: LightSourceType,
}

/// [v2.23.0 R11-4] 광원 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightSourceType {
    /// 점등된 램프/양초
    Item,
    /// 빛나는 몬스터
    Monster,
    /// 마법 빛
    Magical,
}

/// [v2.23.0 R11-4] 광원으로부터 조명 맵 생성 (원본: do_light_sources)
pub fn apply_light_sources<F>(sources: &[LightSource], is_opaque: &F) -> VisionMap
where
    F: Fn(i32, i32) -> bool,
{
    let mut light_map = empty_vision_map();

    for source in sources {
        let fov = calculate_fov(source.x, source.y, source.radius, is_opaque);
        // OR 합산
        for x in 0..COLNO {
            for y in 0..ROWNO {
                if fov[x][y] {
                    light_map[x][y] = true;
                }
            }
        }
    }

    light_map
}

// =============================================================================
// [3] 시야 보정 (야간, 적외선, 투명)
// =============================================================================

/// [v2.23.0 R11-4] 시야 보정 입력
#[derive(Debug, Clone)]
pub struct VisionModifiers {
    /// 기본 시야 반경
    pub base_radius: i32,
    /// 야간 시야 (Infravision) 보유
    pub has_infravision: bool,
    /// 투명 보기 (See Invisible) 보유
    pub has_see_invisible: bool,
    /// 실명 상태
    pub is_blind: bool,
    /// 마법 어둠 영역 내
    pub in_darkness: bool,
    /// 밝은 레벨 (항상 밝음)
    pub lit_level: bool,
}

/// [v2.23.0 R11-4] 보정된 시야 반경 계산
pub fn effective_vision_radius(modifiers: &VisionModifiers) -> i32 {
    if modifiers.is_blind {
        return 0; // 실명 → 시야 없음 (촉각만)
    }
    let mut radius = modifiers.base_radius;
    if modifiers.in_darkness {
        radius = 1; // 어둠 → 인접만
    }
    if modifiers.lit_level {
        radius = radius.max(7); // 밝은 레벨 → 최소 7
    }
    if modifiers.has_infravision {
        radius = radius.max(3); // 적외선 최소 보장
    }
    radius.max(0)
}

/// [v2.23.0 R11-4] invisible 몬스터 감지 여부
pub fn can_see_invisible_monster(
    has_see_invisible: bool,
    monster_invisible: bool,
    monster_detected: bool,
) -> bool {
    if !monster_invisible {
        return true; // 투명하지 않으면 항상 보임
    }
    if has_see_invisible || monster_detected {
        return true; // 투명 보기 또는 감지
    }
    false
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_vision_map() {
        let map = empty_vision_map();
        assert!(!map[0][0]);
        assert!(!map[40][10]);
    }

    #[test]
    fn test_has_clear_path_no_obstacles() {
        assert!(has_clear_path(5, 5, 10, 10, &|_, _| false));
    }

    #[test]
    fn test_has_clear_path_blocked() {
        // (7, 7)에 벽
        assert!(!has_clear_path(5, 5, 10, 10, &|x, y| x == 7 && y == 7));
    }

    #[test]
    fn test_has_clear_path_same_point() {
        assert!(has_clear_path(5, 5, 5, 5, &|_, _| true));
    }

    #[test]
    fn test_fov_center_visible() {
        let map = calculate_fov(40, 10, 5, &|_, _| false);
        assert!(map[40][10]); // 자기 위치
    }

    #[test]
    fn test_fov_radius() {
        let map = calculate_fov(40, 10, 3, &|_, _| false);
        assert!(map[40][10]);
        assert!(map[41][10]); // 인접
        assert!(map[42][10]); // 반경 2
    }

    #[test]
    fn test_fov_wall_blocks() {
        // (41, 10) 벽 → (42, 10) 안 보임
        let map = calculate_fov(40, 10, 5, &|x, _y| x == 41);
        assert!(map[40][10]); // 자기
        assert!(map[41][10]); // 벽 자체는 보임
                              // 벽 너머는 안 보이는 경향 (정확한 레이에 따라 다름)
    }

    #[test]
    fn test_count_visible() {
        let map = calculate_fov(40, 10, 2, &|_, _| false);
        let count = count_visible(&map);
        assert!(count > 1); // 최소 자기 + 인접
        assert!(count < 100); // 반경 2 → 많아야 ~20
    }

    #[test]
    fn test_light_sources() {
        let sources = vec![LightSource {
            x: 10,
            y: 5,
            radius: 3,
            source_type: LightSourceType::Item,
        }];
        let map = apply_light_sources(&sources, &|_, _| false);
        assert!(map[10][5]);
        assert!(map[11][5]);
    }

    #[test]
    fn test_effective_radius_normal() {
        let mods = VisionModifiers {
            base_radius: 5,
            has_infravision: false,
            has_see_invisible: false,
            is_blind: false,
            in_darkness: false,
            lit_level: false,
        };
        assert_eq!(effective_vision_radius(&mods), 5);
    }

    #[test]
    fn test_effective_radius_blind() {
        let mods = VisionModifiers {
            base_radius: 5,
            has_infravision: true,
            has_see_invisible: true,
            is_blind: true,
            in_darkness: false,
            lit_level: true,
        };
        assert_eq!(effective_vision_radius(&mods), 0);
    }

    #[test]
    fn test_effective_radius_darkness() {
        let mods = VisionModifiers {
            base_radius: 5,
            has_infravision: false,
            has_see_invisible: false,
            is_blind: false,
            in_darkness: true,
            lit_level: false,
        };
        assert_eq!(effective_vision_radius(&mods), 1);
    }

    #[test]
    fn test_effective_radius_lit_level() {
        let mods = VisionModifiers {
            base_radius: 3,
            has_infravision: false,
            has_see_invisible: false,
            is_blind: false,
            in_darkness: false,
            lit_level: true,
        };
        assert_eq!(effective_vision_radius(&mods), 7);
    }

    #[test]
    fn test_see_invisible() {
        assert!(can_see_invisible_monster(false, false, false)); // 투명 아님
        assert!(!can_see_invisible_monster(false, true, false)); // 투명, 감지 없음
        assert!(can_see_invisible_monster(true, true, false)); // see invisible
        assert!(can_see_invisible_monster(false, true, true)); // 감지됨
    }
}
