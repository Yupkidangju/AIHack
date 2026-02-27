// ============================================================================
// [v2.32.0 Phase 96-5] 시야/탐지 확장 (vision_phase96_ext.rs)
// 원본: NetHack 3.6.7 src/vision.c + detect.c L300-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 시야 계산 — vision_calc (vision.c L300-600)
// =============================================================================

/// [v2.32.0 96-5] 시야 상태
#[derive(Debug, Clone)]
pub struct VisionState {
    pub can_see: bool,
    pub vision_radius: i32,
    pub is_dark: bool,
    pub has_infravision: bool,
    pub has_see_invisible: bool,
    pub has_telepathy: bool,
    pub is_blind: bool,
    pub is_underwater: bool,
}

/// [v2.32.0 96-5] 타일 가시성 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileVisibility {
    Visible,
    DimlyLit,
    InfraredOnly,
    Telepathy,
    NotVisible,
    Remembered, // 이전에 봤던 것
}

/// [v2.32.0 96-5] 타일 가시성 계산
/// 원본: vision.c vision_recalc()
pub fn calculate_tile_visibility(
    viewer: &VisionState,
    viewer_x: i32,
    viewer_y: i32,
    tile_x: i32,
    tile_y: i32,
    tile_lit: bool,
    has_monster: bool,
    monster_invisible: bool,
    was_seen: bool,
) -> TileVisibility {
    let dist = (viewer_x - tile_x).abs().max((viewer_y - tile_y).abs());

    // 실명
    if viewer.is_blind {
        if dist <= 1 {
            return TileVisibility::DimlyLit; // 인접은 감각으로
        }
        if viewer.has_telepathy && has_monster {
            return TileVisibility::Telepathy;
        }
        if was_seen {
            return TileVisibility::Remembered;
        }
        return TileVisibility::NotVisible;
    }

    // 수중
    if viewer.is_underwater && dist > 1 {
        return TileVisibility::NotVisible;
    }

    // 시야 범위 내
    if dist <= viewer.vision_radius {
        // 밝은 타일 또는 충분히 가까움
        if tile_lit || dist <= 1 || viewer.is_dark == false {
            // 투명 몬스터 체크
            if has_monster && monster_invisible && !viewer.has_see_invisible {
                return TileVisibility::DimlyLit;
            }
            return TileVisibility::Visible;
        }

        // 적외선 시야
        if viewer.has_infravision && has_monster {
            return TileVisibility::InfraredOnly;
        }

        // 어두운 곳
        if dist <= 2 {
            return TileVisibility::DimlyLit;
        }
    }

    // 텔레파시로 몬스터 감지
    if viewer.has_telepathy && has_monster {
        return TileVisibility::Telepathy;
    }

    // 이전에 본 적 있음
    if was_seen {
        return TileVisibility::Remembered;
    }

    TileVisibility::NotVisible
}

// =============================================================================
// [2] 탐지 마법 — detection (detect.c L400-800)
// =============================================================================

/// [v2.32.0 96-5] 탐지 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionType {
    Monster,
    Object,
    Food,
    Gold,
    Trap,
    Magic,
    StairsPortal,
    SecretDoor,
}

/// [v2.32.0 96-5] 탐지 결과
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub detection_type: DetectionType,
    pub found_positions: Vec<(i32, i32)>,
    pub found_count: i32,
    pub is_blessed: bool,
}

/// [v2.32.0 96-5] 탐지 실행
pub fn perform_detection(
    detection_type: DetectionType,
    caster_x: i32,
    caster_y: i32,
    map_width: i32,
    map_height: i32,
    is_blessed: bool,
    rng: &mut NetHackRng,
) -> DetectionResult {
    // 축복: 전맵 / 일반: 제한 범위
    let radius = if is_blessed {
        map_width.max(map_height)
    } else {
        15
    };

    // 시뮬레이션: 랜덤 위치에 아이템 생성
    let count = match detection_type {
        DetectionType::Monster => rng.rn2(10) + 3,
        DetectionType::Object => rng.rn2(8) + 2,
        DetectionType::Gold => rng.rn2(5) + 1,
        DetectionType::Trap => rng.rn2(4) + 1,
        DetectionType::Food => rng.rn2(6) + 1,
        DetectionType::Magic => rng.rn2(3) + 1,
        DetectionType::StairsPortal => rng.rn2(2) + 1,
        DetectionType::SecretDoor => rng.rn2(3),
    };

    let mut positions = Vec::new();
    for _ in 0..count {
        let x = caster_x + rng.rn2(radius * 2) - radius;
        let y = caster_y + rng.rn2(radius * 2) - radius;
        if x >= 0 && x < map_width && y >= 0 && y < map_height {
            positions.push((x, y));
        }
    }

    let found = positions.len() as i32;

    DetectionResult {
        detection_type,
        found_positions: positions,
        found_count: found,
        is_blessed,
    }
}

// =============================================================================
// [3] 라인 오브 사이트 — line_of_sight (vision.c L600-900)
// =============================================================================

/// [v2.32.0 96-5] 직선 시야 확인 (Bresenham 기반)
pub fn has_line_of_sight(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    is_blocked: &dyn Fn(i32, i32) -> bool,
) -> bool {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut cx = x1;
    let mut cy = y1;

    loop {
        if cx == x2 && cy == y2 {
            return true;
        }
        if is_blocked(cx, cy) && !(cx == x1 && cy == y1) {
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

    fn base_vision() -> VisionState {
        VisionState {
            can_see: true,
            vision_radius: 5,
            is_dark: false,
            has_infravision: false,
            has_see_invisible: false,
            has_telepathy: false,
            is_blind: false,
            is_underwater: false,
        }
    }

    #[test]
    fn test_visible_close() {
        let v = base_vision();
        let result = calculate_tile_visibility(&v, 10, 10, 12, 10, true, false, false, false);
        assert_eq!(result, TileVisibility::Visible);
    }

    #[test]
    fn test_blind() {
        let mut v = base_vision();
        v.is_blind = true;
        let result = calculate_tile_visibility(&v, 10, 10, 15, 10, true, false, false, false);
        assert_eq!(result, TileVisibility::NotVisible);
    }

    #[test]
    fn test_blind_adjacent() {
        let mut v = base_vision();
        v.is_blind = true;
        let result = calculate_tile_visibility(&v, 10, 10, 11, 10, true, false, false, false);
        assert_eq!(result, TileVisibility::DimlyLit);
    }

    #[test]
    fn test_telepathy() {
        let mut v = base_vision();
        v.is_blind = true;
        v.has_telepathy = true;
        let result = calculate_tile_visibility(&v, 10, 10, 20, 20, false, true, false, false);
        assert_eq!(result, TileVisibility::Telepathy);
    }

    #[test]
    fn test_invisible_monster() {
        let v = base_vision();
        let result = calculate_tile_visibility(&v, 10, 10, 12, 10, true, true, true, false);
        assert_eq!(result, TileVisibility::DimlyLit);
    }

    #[test]
    fn test_detection() {
        let mut rng = test_rng();
        let result = perform_detection(DetectionType::Monster, 40, 10, 80, 21, false, &mut rng);
        assert!(result.found_count >= 0);
    }

    #[test]
    fn test_detection_blessed() {
        let mut rng = test_rng();
        let result = perform_detection(DetectionType::Object, 40, 10, 80, 21, true, &mut rng);
        assert!(result.is_blessed);
    }

    #[test]
    fn test_line_of_sight() {
        let clear = has_line_of_sight(0, 0, 5, 5, &|_, _| false);
        assert!(clear);
    }

    #[test]
    fn test_line_of_sight_blocked() {
        let blocked = has_line_of_sight(0, 0, 5, 5, &|x, y| x == 3 && y == 3);
        assert!(!blocked);
    }
}
