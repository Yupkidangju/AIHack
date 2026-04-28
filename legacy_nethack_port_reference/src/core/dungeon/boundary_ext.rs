// ============================================================================
// [v2.31.0 R19-4] 레벨 경계 (boundary_ext.rs)
// 원본: NetHack 3.6.7 mklev.c/dungeon.c 경계 로직
// 맵 경계 판정, 유효 좌표, 벽 검사, 통과 가능성
// ============================================================================

use crate::core::dungeon::COLNO;
use crate::core::dungeon::ROWNO;

/// [v2.31.0 R19-4] 좌표 유효성 (원본: isok)
pub fn is_valid_pos(x: i32, y: i32) -> bool {
    x >= 0 && (x as usize) < COLNO && y >= 0 && (y as usize) < ROWNO
}

/// [v2.31.0 R19-4] 맵 가장자리 여부
pub fn is_edge(x: i32, y: i32) -> bool {
    x == 0 || x as usize == COLNO - 1 || y == 0 || y as usize == ROWNO - 1
}

/// [v2.31.0 R19-4] 타일 통과 가능성
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Passability {
    Open,    // 통과 가능
    Blocked, // 벽/바위
    Door,    // 문 (열면 통과)
    Water,   // 수영/비행 필요
    Lava,    // 내화/비행 필요
    Bars,    // 철창 (작은 몬스터만)
}

/// [v2.31.0 R19-4] 타일 통과 판정 (능력 기반)
pub fn can_pass(
    tile: Passability,
    can_fly: bool,
    can_swim: bool,
    fire_resistant: bool,
    is_small: bool,
) -> bool {
    match tile {
        Passability::Open => true,
        Passability::Blocked => false,
        Passability::Door => true, // 문은 열 수 있다고 가정
        Passability::Water => can_fly || can_swim,
        Passability::Lava => can_fly || fire_resistant,
        Passability::Bars => is_small,
    }
}

/// [v2.31.0 R19-4] 인접 타일 (8방향)
pub fn adjacent_positions(x: i32, y: i32) -> Vec<(i32, i32)> {
    let dirs = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    dirs.iter()
        .map(|(dx, dy)| (x + dx, y + dy))
        .filter(|&(nx, ny)| is_valid_pos(nx, ny))
        .collect()
}

/// [v2.31.0 R19-4] 맨해튼 거리
pub fn manhattan_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x1 - x2).abs() + (y1 - y2).abs()
}

/// [v2.31.0 R19-4] 체비셰프 거리 (NetHack 기본)
pub fn chebyshev_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x1 - x2).abs().max((y1 - y2).abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_pos() {
        assert!(is_valid_pos(1, 1));
        assert!(!is_valid_pos(-1, 0));
        assert!(!is_valid_pos(COLNO as i32, 0));
    }

    #[test]
    fn test_edge() {
        assert!(is_edge(0, 5));
        assert!(!is_edge(5, 5));
    }

    #[test]
    fn test_passability() {
        assert!(can_pass(Passability::Open, false, false, false, false));
        assert!(!can_pass(Passability::Blocked, true, true, true, true));
        assert!(can_pass(Passability::Water, true, false, false, false));
        assert!(can_pass(Passability::Lava, false, false, true, false));
    }

    #[test]
    fn test_adjacent() {
        let adj = adjacent_positions(5, 5);
        assert_eq!(adj.len(), 8);
    }

    #[test]
    fn test_adjacent_corner() {
        let adj = adjacent_positions(0, 0);
        assert_eq!(adj.len(), 3); // (1,0), (0,1), (1,1)
    }

    #[test]
    fn test_chebyshev() {
        assert_eq!(chebyshev_distance(0, 0, 3, 4), 4);
        assert_eq!(manhattan_distance(0, 0, 3, 4), 7);
    }
}
