// ============================================================================
// [v2.38.0 R26-1] 위치 기억 (memory_map_ext.rs)
// 원본: NetHack 3.6.7 display.c/mapglyph.c 기억 관리
// 탐색한 타일 기억, 안개 전쟁, 마지막 본 상태
// ============================================================================

use crate::core::dungeon::COLNO;
use crate::core::dungeon::ROWNO;

/// [v2.38.0 R26-1] 기억 엔트리
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryTile {
    pub glyph: char,
    pub seen_turn: u64,
    pub known: bool,
}

/// [v2.38.0 R26-1] 레벨 기억 맵
pub type MemoryMap = [[MemoryTile; ROWNO]; COLNO];

/// [v2.38.0 R26-1] 빈 기억 맵
pub fn new_memory_map() -> MemoryMap {
    [[MemoryTile::default(); ROWNO]; COLNO]
}

/// [v2.38.0 R26-1] 타일 기억 갱신
pub fn update_memory(map: &mut MemoryMap, x: usize, y: usize, glyph: char, turn: u64) {
    if x < COLNO && y < ROWNO {
        map[x][y] = MemoryTile {
            glyph,
            seen_turn: turn,
            known: true,
        };
    }
}

/// [v2.38.0 R26-1] 최근 기억인지
pub fn is_recent(map: &MemoryMap, x: usize, y: usize, current_turn: u64, threshold: u64) -> bool {
    if x >= COLNO || y >= ROWNO {
        return false;
    }
    let tile = &map[x][y];
    tile.known && current_turn.saturating_sub(tile.seen_turn) <= threshold
}

/// [v2.38.0 R26-1] 탐색률
pub fn exploration_pct(map: &MemoryMap) -> f64 {
    let total = COLNO * ROWNO;
    let known = map
        .iter()
        .flat_map(|col| col.iter())
        .filter(|t| t.known)
        .count();
    known as f64 / total as f64 * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_map() {
        let m = new_memory_map();
        assert!(!m[0][0].known);
    }

    #[test]
    fn test_update() {
        let mut m = new_memory_map();
        update_memory(&mut m, 5, 5, '.', 100);
        assert!(m[5][5].known);
        assert_eq!(m[5][5].glyph, '.');
    }

    #[test]
    fn test_recent() {
        let mut m = new_memory_map();
        update_memory(&mut m, 3, 3, '#', 90);
        assert!(is_recent(&m, 3, 3, 100, 20));
        assert!(!is_recent(&m, 3, 3, 200, 20));
    }

    #[test]
    fn test_exploration() {
        let mut m = new_memory_map();
        update_memory(&mut m, 0, 0, '.', 1);
        assert!(exploration_pct(&m) > 0.0);
    }

    #[test]
    fn test_bounds() {
        let mut m = new_memory_map();
        update_memory(&mut m, 999, 999, '.', 1); // 범위 밖 — 무시
        assert!(!is_recent(&m, 999, 999, 1, 10));
    }
}
