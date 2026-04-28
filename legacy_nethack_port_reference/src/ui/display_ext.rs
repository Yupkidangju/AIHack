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
// [v2.24.0 R12-2] 디스플레이 심볼 엔진 (display_ext.rs)
//
// 원본 참조: NetHack 3.6.7 display.c (2,091줄)
//
// 구현 내용:
//   1. 타일→심볼/색상 매핑
//   2. 기억(Memory) 맵
//   3. 오버레이 우선순위
//   4. 환각 심볼 치환
//   5. 심볼 세트 관리
// ============================================================================

use crate::core::dungeon::COLNO;
use crate::core::dungeon::ROWNO;
use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 심볼 매핑 (원본: display.c mapglyph)
// =============================================================================

/// [v2.24.0 R12-2] 타일 심볼 정보
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayGlyph {
    /// 표시 문자
    pub ch: char,
    /// 색상 (0~15 ANSI)
    pub color: u8,
    /// 강조 (bold)
    pub bold: bool,
}

/// [v2.24.0 R12-2] 표시 색상 상수
pub mod colors {
    pub const BLACK: u8 = 0;
    pub const RED: u8 = 1;
    pub const GREEN: u8 = 2;
    pub const BROWN: u8 = 3;
    pub const BLUE: u8 = 4;
    pub const MAGENTA: u8 = 5;
    pub const CYAN: u8 = 6;
    pub const GRAY: u8 = 7;
    pub const NO_COLOR: u8 = 8;
    pub const ORANGE: u8 = 9;
    pub const BRIGHT_GREEN: u8 = 10;
    pub const YELLOW: u8 = 11;
    pub const BRIGHT_BLUE: u8 = 12;
    pub const BRIGHT_MAGENTA: u8 = 13;
    pub const BRIGHT_CYAN: u8 = 14;
    pub const WHITE: u8 = 15;
}

/// [v2.24.0 R12-2] 타일 타입→심볼 매핑 (원본: mapglyph 디폴트)
pub fn tile_to_glyph(tile_type: u8) -> DisplayGlyph {
    match tile_type {
        0 => DisplayGlyph {
            ch: ' ',
            color: colors::NO_COLOR,
            bold: false,
        }, // 빈 공간
        1 => DisplayGlyph {
            ch: '#',
            color: colors::GRAY,
            bold: false,
        }, // 벽 (수직)
        2 => DisplayGlyph {
            ch: '#',
            color: colors::GRAY,
            bold: false,
        }, // 벽 (수평)
        3 => DisplayGlyph {
            ch: '.',
            color: colors::GRAY,
            bold: false,
        }, // 바닥 (돌)
        4 => DisplayGlyph {
            ch: '.',
            color: colors::GRAY,
            bold: false,
        }, // 바닥 (방)
        5 => DisplayGlyph {
            ch: '+',
            color: colors::BROWN,
            bold: false,
        }, // 닫힌 문
        6 => DisplayGlyph {
            ch: '|',
            color: colors::BROWN,
            bold: false,
        }, // 열린 문 (수직)
        7 => DisplayGlyph {
            ch: '-',
            color: colors::BROWN,
            bold: false,
        }, // 열린 문 (수평)
        8 => DisplayGlyph {
            ch: '<',
            color: colors::GRAY,
            bold: false,
        }, // 올라가는 계단
        9 => DisplayGlyph {
            ch: '>',
            color: colors::GRAY,
            bold: false,
        }, // 내려가는 계단
        10 => DisplayGlyph {
            ch: '^',
            color: colors::MAGENTA,
            bold: false,
        }, // 함정
        11 => DisplayGlyph {
            ch: '{',
            color: colors::BLUE,
            bold: true,
        }, // 샘
        12 => DisplayGlyph {
            ch: '}',
            color: colors::BLUE,
            bold: false,
        }, // 물
        13 => DisplayGlyph {
            ch: '}',
            color: colors::RED,
            bold: true,
        }, // 용암
        14 => DisplayGlyph {
            ch: '\\',
            color: colors::YELLOW,
            bold: true,
        }, // 왕좌
        15 => DisplayGlyph {
            ch: '_',
            color: colors::GRAY,
            bold: false,
        }, // 제단
        16 => DisplayGlyph {
            ch: '#',
            color: colors::CYAN,
            bold: false,
        }, // 얼음
        17 => DisplayGlyph {
            ch: '}',
            color: colors::CYAN,
            bold: true,
        }, // 공기
        18 => DisplayGlyph {
            ch: '}',
            color: colors::GRAY,
            bold: false,
        }, // 구름
        _ => DisplayGlyph {
            ch: '?',
            color: colors::WHITE,
            bold: false,
        }, // 미지
    }
}

// =============================================================================
// [2] 몬스터/아이템 심볼 (원본: display.c mon_to_glyph, obj_to_glyph)
// =============================================================================

/// [v2.24.0 R12-2] 아이템 클래스별 심볼
pub fn item_class_glyph(class: char) -> DisplayGlyph {
    match class {
        ')' => DisplayGlyph {
            ch: ')',
            color: colors::CYAN,
            bold: false,
        }, // 무기
        '[' => DisplayGlyph {
            ch: '[',
            color: colors::CYAN,
            bold: false,
        }, // 갑옷
        '!' => DisplayGlyph {
            ch: '!',
            color: colors::MAGENTA,
            bold: false,
        }, // 포션
        '?' => DisplayGlyph {
            ch: '?',
            color: colors::WHITE,
            bold: false,
        }, // 스크롤
        '+' => DisplayGlyph {
            ch: '+',
            color: colors::MAGENTA,
            bold: true,
        }, // 마법서
        '/' => DisplayGlyph {
            ch: '/',
            color: colors::BLUE,
            bold: true,
        }, // 지팡이
        '=' => DisplayGlyph {
            ch: '=',
            color: colors::ORANGE,
            bold: false,
        }, // 반지
        '"' => DisplayGlyph {
            ch: '"',
            color: colors::ORANGE,
            bold: false,
        }, // 아뮬렛
        '(' => DisplayGlyph {
            ch: '(',
            color: colors::GRAY,
            bold: false,
        }, // 도구
        '%' => DisplayGlyph {
            ch: '%',
            color: colors::GREEN,
            bold: false,
        }, // 음식
        '*' => DisplayGlyph {
            ch: '*',
            color: colors::YELLOW,
            bold: true,
        }, // 보석
        '$' => DisplayGlyph {
            ch: '$',
            color: colors::YELLOW,
            bold: true,
        }, // 금화
        _ => DisplayGlyph {
            ch: class,
            color: colors::WHITE,
            bold: false,
        },
    }
}

// =============================================================================
// [3] 기억(Memory) 맵 (원본: display.c show_glyph, remember)
// =============================================================================

/// [v2.24.0 R12-2] 기억 맵 셀
#[derive(Debug, Clone, Copy, Default)]
pub struct MemoryCell {
    /// 기억된 타일 타입
    pub tile_type: u8,
    /// 기억된 아이템 클래스 (없으면 0)
    pub item_class: u8,
    /// 방문 횟수
    pub visit_count: u16,
    /// 마지막 방문 턴
    pub last_seen_turn: u32,
}

/// [v2.24.0 R12-2] 기억 맵 타입
pub type MemoryMap = [[MemoryCell; ROWNO]; COLNO];

/// [v2.24.0 R12-2] 빈 기억 맵
pub fn empty_memory_map() -> MemoryMap {
    [[MemoryCell::default(); ROWNO]; COLNO]
}

/// [v2.24.0 R12-2] 기억 맵 업데이트 (원본: show_glyph)
pub fn update_memory(
    memory: &mut MemoryMap,
    x: usize,
    y: usize,
    tile_type: u8,
    item_class: u8,
    turn: u32,
) {
    if x < COLNO && y < ROWNO {
        let cell = &mut memory[x][y];
        cell.tile_type = tile_type;
        cell.item_class = item_class;
        cell.visit_count = cell.visit_count.saturating_add(1);
        cell.last_seen_turn = turn;
    }
}

// =============================================================================
// [4] 오버레이 우선순위 (원본: display.c newsym)
// =============================================================================

/// [v2.24.0 R12-2] 표시 레이어 (낮은 번호 = 배경)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DisplayLayer {
    /// 바닥 타일
    Terrain = 0,
    /// 함정
    Trap = 1,
    /// 아이템
    Object = 2,
    /// 몬스터
    Monster = 3,
    /// 플레이어
    Player = 4,
    /// 마법 효과
    Effect = 5,
}

/// [v2.24.0 R12-2] 오버레이 엔트리
#[derive(Debug, Clone)]
pub struct OverlayEntry {
    /// 레이어
    pub layer: DisplayLayer,
    /// 표시 심볼
    pub glyph: DisplayGlyph,
}

/// [v2.24.0 R12-2] 오버레이 합성 (가장 높은 레이어 우선)
pub fn resolve_overlay(entries: &mut [OverlayEntry]) -> Option<DisplayGlyph> {
    if entries.is_empty() {
        return None;
    }
    entries.sort_by(|a, b| b.layer.cmp(&a.layer));
    Some(entries[0].glyph)
}

// =============================================================================
// [5] 환각 심볼 (원본: display.c random_monster, hallucination)
// =============================================================================

/// [v2.24.0 R12-2] 환각 심볼 테이블
const HALLUC_MONSTERS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// [v2.24.0 R12-2] 환각 상태 시 랜덤 심볼 반환
pub fn hallucinated_glyph(rng: &mut NetHackRng) -> DisplayGlyph {
    let idx = rng.rn2(HALLUC_MONSTERS.len() as i32) as usize;
    let color = rng.rn2(16) as u8;
    DisplayGlyph {
        ch: HALLUC_MONSTERS[idx],
        color,
        bold: rng.rn2(2) != 0,
    }
}

/// [v2.24.0 R12-2] 표시 심볼 최종 결정 (환각 고려)
pub fn final_glyph(
    base_glyph: DisplayGlyph,
    is_hallucinating: bool,
    is_monster: bool,
    rng: &mut NetHackRng,
) -> DisplayGlyph {
    if is_hallucinating && is_monster {
        hallucinated_glyph(rng)
    } else {
        base_glyph
    }
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_floor() {
        let g = tile_to_glyph(4);
        assert_eq!(g.ch, '.');
    }

    #[test]
    fn test_tile_door() {
        let g = tile_to_glyph(5);
        assert_eq!(g.ch, '+');
    }

    #[test]
    fn test_tile_stairs() {
        assert_eq!(tile_to_glyph(8).ch, '<');
        assert_eq!(tile_to_glyph(9).ch, '>');
    }

    #[test]
    fn test_tile_unknown() {
        let g = tile_to_glyph(255);
        assert_eq!(g.ch, '?');
    }

    #[test]
    fn test_item_weapon() {
        let g = item_class_glyph(')');
        assert_eq!(g.ch, ')');
    }

    #[test]
    fn test_item_gold() {
        let g = item_class_glyph('$');
        assert_eq!(g.ch, '$');
        assert_eq!(g.color, colors::YELLOW);
    }

    #[test]
    fn test_memory_update() {
        let mut mem = empty_memory_map();
        update_memory(&mut mem, 10, 5, 4, 0, 100);
        assert_eq!(mem[10][5].tile_type, 4);
        assert_eq!(mem[10][5].visit_count, 1);
        update_memory(&mut mem, 10, 5, 4, 0, 200);
        assert_eq!(mem[10][5].visit_count, 2);
        assert_eq!(mem[10][5].last_seen_turn, 200);
    }

    #[test]
    fn test_memory_bounds() {
        let mut mem = empty_memory_map();
        update_memory(&mut mem, COLNO + 1, 0, 4, 0, 100); // 초과 → 무시
                                                          // 패닉 없이 통과
    }

    #[test]
    fn test_overlay_resolve() {
        let mut entries = vec![
            OverlayEntry {
                layer: DisplayLayer::Terrain,
                glyph: tile_to_glyph(4),
            },
            OverlayEntry {
                layer: DisplayLayer::Monster,
                glyph: DisplayGlyph {
                    ch: 'D',
                    color: colors::RED,
                    bold: true,
                },
            },
            OverlayEntry {
                layer: DisplayLayer::Object,
                glyph: item_class_glyph(')'),
            },
        ];
        let result = resolve_overlay(&mut entries);
        assert!(result.is_some());
        assert_eq!(result.unwrap().ch, 'D'); // 몬스터 우선
    }

    #[test]
    fn test_overlay_empty() {
        let mut entries: Vec<OverlayEntry> = vec![];
        assert!(resolve_overlay(&mut entries).is_none());
    }

    #[test]
    fn test_hallucinated_glyph() {
        let mut rng = NetHackRng::new(42);
        let g = hallucinated_glyph(&mut rng);
        assert!(HALLUC_MONSTERS.contains(&g.ch));
    }

    #[test]
    fn test_final_glyph_normal() {
        let mut rng = NetHackRng::new(42);
        let base = tile_to_glyph(4);
        let g = final_glyph(base, false, true, &mut rng);
        assert_eq!(g.ch, '.'); // 환각 아님 → 원래 심볼
    }

    #[test]
    fn test_final_glyph_hallucinating() {
        let mut rng = NetHackRng::new(42);
        let base = DisplayGlyph {
            ch: 'D',
            color: colors::RED,
            bold: true,
        };
        let g = final_glyph(base, true, true, &mut rng);
        assert!(HALLUC_MONSTERS.contains(&g.ch)); // 환각 → 랜덤
    }
}
