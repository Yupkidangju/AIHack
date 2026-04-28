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
// [v2.25.0 R13-1] 레벨 생성 파이프라인 (mklev_ext.rs)
//
// 원본 참조: NetHack 3.6.7 mklev.c (1,348줄)
//
// 구현 내용:
//   1. 방 배치 알고리즘
//   2. 복도 연결
//   3. 문/계단 배치
//   4. 특수 방 유형
//   5. 레벨 생성 파라미터
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 방 구조 (원본: mklev.c mkroom)
// =============================================================================

/// [v2.25.0 R13-1] 방 좌표
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoomRect {
    pub lx: i32,
    pub ly: i32,
    pub hx: i32,
    pub hy: i32,
}

impl RoomRect {
    /// 방 너비
    pub fn width(&self) -> i32 {
        self.hx - self.lx + 1
    }
    /// 방 높이
    pub fn height(&self) -> i32 {
        self.hy - self.ly + 1
    }
    /// 면적
    pub fn area(&self) -> i32 {
        self.width() * self.height()
    }
    /// 겹침 검사
    pub fn overlaps(&self, other: &RoomRect) -> bool {
        self.lx <= other.hx && self.hx >= other.lx && self.ly <= other.hy && self.hy >= other.ly
    }
    /// 마진 포함 겹침 (원본: 방 간 최소 거리 2)
    pub fn overlaps_with_margin(&self, other: &RoomRect, margin: i32) -> bool {
        let expanded = RoomRect {
            lx: self.lx - margin,
            ly: self.ly - margin,
            hx: self.hx + margin,
            hy: self.hy + margin,
        };
        expanded.overlaps(other)
    }
    /// 중심 좌표
    pub fn center(&self) -> (i32, i32) {
        ((self.lx + self.hx) / 2, (self.ly + self.hy) / 2)
    }
}

// =============================================================================
// [2] 특수 방 유형 (원본: mklev.c mkroom_type)
// =============================================================================

/// [v2.25.0 R13-1] 방 유형 (원본: SHOPBASE ~ MORGUE)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    /// 일반 빈 방
    Ordinary,
    /// 상점
    Shop,
    /// 동물원
    Zoo,
    /// 보물 방
    Treasury,
    /// 무법 지대
    Barracks,
    /// 벌집
    BeeHive,
    /// 모르그 (언데드)
    Morgue,
    /// 왕좌 방
    Throne,
    /// 창고
    Vault,
    /// 신전
    Temple,
    /// 소용돌이
    Swamp,
}

/// [v2.25.0 R13-1] 깊이별 특수 방 확률 (원본: mklev.c mkspecialroom)
pub fn special_room_chance(depth: i32, rng: &mut NetHackRng) -> Option<RoomType> {
    if depth < 3 {
        return None; // 얕은 층은 특수 방 없음
    }
    let roll = rng.rn2(100);
    match roll {
        0..=8 => Some(RoomType::Shop),                     // 9%
        9..=13 => Some(RoomType::Zoo),                     // 5%
        14..=16 => Some(RoomType::Treasury),               // 3%
        17..=20 if depth >= 8 => Some(RoomType::Barracks), // 4% (깊은 곳)
        21..=23 if depth >= 10 => Some(RoomType::BeeHive), // 3%
        24..=26 if depth >= 12 => Some(RoomType::Morgue),  // 3%
        27..=28 if depth >= 10 => Some(RoomType::Throne),  // 2%
        29 if depth >= 15 => Some(RoomType::Temple),       // 1%
        _ => None,
    }
}

// =============================================================================
// [3] 방 배치 (원본: mklev.c makerooms)
// =============================================================================

/// [v2.25.0 R13-1] 레벨 생성 파라미터
#[derive(Debug, Clone)]
pub struct LevelGenParams {
    /// 맵 너비
    pub width: i32,
    /// 맵 높이
    pub height: i32,
    /// 최소 방 수
    pub min_rooms: i32,
    /// 최대 방 수
    pub max_rooms: i32,
    /// 최소 방 크기
    pub min_room_size: i32,
    /// 최대 방 크기
    pub max_room_size: i32,
    /// 던전 깊이
    pub depth: i32,
}

impl Default for LevelGenParams {
    fn default() -> Self {
        Self {
            width: 80,
            height: 21,
            min_rooms: 3,
            max_rooms: 7,
            min_room_size: 3,
            max_room_size: 10,
            depth: 1,
        }
    }
}

/// [v2.25.0 R13-1] 방 배치 결과
#[derive(Debug, Clone)]
pub struct RoomPlan {
    pub rect: RoomRect,
    pub room_type: RoomType,
    pub lit: bool,
    pub door_positions: Vec<(i32, i32)>,
}

/// [v2.25.0 R13-1] 랜덤 방 생성 (원본: makerooms)
pub fn generate_rooms(params: &LevelGenParams, rng: &mut NetHackRng) -> Vec<RoomPlan> {
    let num_rooms = rng.rn1(params.max_rooms - params.min_rooms + 1, params.min_rooms);

    let mut rooms: Vec<RoomPlan> = Vec::new();
    let mut attempts = 0;
    let max_attempts = num_rooms * 20;

    while (rooms.len() as i32) < num_rooms && attempts < max_attempts {
        attempts += 1;
        let w = rng.rn1(
            params.max_room_size - params.min_room_size + 1,
            params.min_room_size,
        );
        let h = rng
            .rn1(
                (params.max_room_size / 2) - params.min_room_size + 1,
                params.min_room_size,
            )
            .min(params.height - 4);
        let x = rng.rn1(params.width - w - 2, 1);
        let y = rng.rn1(params.height - h - 2, 1);

        let rect = RoomRect {
            lx: x,
            ly: y,
            hx: x + w - 1,
            hy: y + h - 1,
        };

        // 기존 방과 겹침 검사
        if rooms.iter().any(|r| r.rect.overlaps_with_margin(&rect, 2)) {
            continue;
        }

        // 맵 범위 검사
        if rect.hx >= params.width - 1 || rect.hy >= params.height - 1 {
            continue;
        }

        // 특수 방 결정
        let room_type = if rooms.is_empty() {
            RoomType::Ordinary // 첫 방은 항상 일반
        } else {
            special_room_chance(params.depth, rng).unwrap_or(RoomType::Ordinary)
        };

        // 조명 (얕은 층은 밝음)
        let lit = params.depth <= 5 || rng.rn2(3) == 0;

        rooms.push(RoomPlan {
            rect,
            room_type,
            lit,
            door_positions: Vec::new(),
        });
    }

    rooms
}

// =============================================================================
// [4] 복도 연결 (원본: mklev.c makecorridors)
// =============================================================================

/// [v2.25.0 R13-1] 복도 세그먼트
#[derive(Debug, Clone)]
pub struct CorridorSegment {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

/// [v2.25.0 R13-1] 두 방을 연결하는 복도 계획 (L자형)
pub fn plan_corridor(room_a: &RoomRect, room_b: &RoomRect) -> Vec<CorridorSegment> {
    let (ax, ay) = room_a.center();
    let (bx, by) = room_b.center();

    // L자형 복도: 먼저 수평, 그다음 수직
    vec![
        CorridorSegment {
            x1: ax,
            y1: ay,
            x2: bx,
            y2: ay,
        },
        CorridorSegment {
            x1: bx,
            y1: ay,
            x2: bx,
            y2: by,
        },
    ]
}

/// [v2.25.0 R13-1] 모든 방 연결 (순차 연결)
pub fn plan_all_corridors(rooms: &[RoomPlan]) -> Vec<CorridorSegment> {
    let mut corridors = Vec::new();
    for i in 0..rooms.len().saturating_sub(1) {
        corridors.extend(plan_corridor(&rooms[i].rect, &rooms[i + 1].rect));
    }
    corridors
}

// =============================================================================
// [5] 계단 배치 (원본: mklev.c mkstairs)
// =============================================================================

/// [v2.25.0 R13-1] 계단 배치 결과
#[derive(Debug, Clone)]
pub struct StairPlan {
    pub up_x: i32,
    pub up_y: i32,
    pub down_x: i32,
    pub down_y: i32,
}

/// [v2.25.0 R13-1] 계단 배치 (첫 방에 올라가기, 마지막 방에 내려가기)
pub fn plan_stairs(rooms: &[RoomPlan], rng: &mut NetHackRng) -> Option<StairPlan> {
    if rooms.len() < 2 {
        return None;
    }
    let first = &rooms[0].rect;
    let last = &rooms[rooms.len() - 1].rect;

    let up_x = rng.rn1(first.width(), first.lx);
    let up_y = rng.rn1(first.height(), first.ly);
    let down_x = rng.rn1(last.width(), last.lx);
    let down_y = rng.rn1(last.height(), last.ly);

    Some(StairPlan {
        up_x,
        up_y,
        down_x,
        down_y,
    })
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_rect_basic() {
        let r = RoomRect {
            lx: 5,
            ly: 3,
            hx: 10,
            hy: 7,
        };
        assert_eq!(r.width(), 6);
        assert_eq!(r.height(), 5);
        assert_eq!(r.area(), 30);
        assert_eq!(r.center(), (7, 5));
    }

    #[test]
    fn test_room_overlap() {
        let a = RoomRect {
            lx: 0,
            ly: 0,
            hx: 5,
            hy: 5,
        };
        let b = RoomRect {
            lx: 3,
            ly: 3,
            hx: 8,
            hy: 8,
        };
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_room_no_overlap() {
        let a = RoomRect {
            lx: 0,
            ly: 0,
            hx: 3,
            hy: 3,
        };
        let b = RoomRect {
            lx: 10,
            ly: 10,
            hx: 15,
            hy: 15,
        };
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn test_overlap_margin() {
        let a = RoomRect {
            lx: 0,
            ly: 0,
            hx: 3,
            hy: 3,
        };
        let b = RoomRect {
            lx: 5,
            ly: 0,
            hx: 8,
            hy: 3,
        };
        assert!(!a.overlaps(&b));
        assert!(a.overlaps_with_margin(&b, 2));
    }

    #[test]
    fn test_special_room_shallow() {
        let mut rng = NetHackRng::new(42);
        assert!(special_room_chance(1, &mut rng).is_none());
    }

    #[test]
    fn test_special_room_deep() {
        let mut rng = NetHackRng::new(42);
        let mut found_special = false;
        for seed in 0..50 {
            let mut r = NetHackRng::new(seed);
            if special_room_chance(15, &mut r).is_some() {
                found_special = true;
                break;
            }
        }
        assert!(found_special);
    }

    #[test]
    fn test_generate_rooms() {
        let mut rng = NetHackRng::new(42);
        let params = LevelGenParams::default();
        let rooms = generate_rooms(&params, &mut rng);
        assert!(rooms.len() >= 2);
        assert!(rooms.len() <= 7);
        // 겹침 없음 확인
        for i in 0..rooms.len() {
            for j in (i + 1)..rooms.len() {
                assert!(!rooms[i].rect.overlaps(&rooms[j].rect));
            }
        }
    }

    #[test]
    fn test_generate_rooms_first_ordinary() {
        let mut rng = NetHackRng::new(42);
        let params = LevelGenParams {
            depth: 20,
            ..Default::default()
        };
        let rooms = generate_rooms(&params, &mut rng);
        assert_eq!(rooms[0].room_type, RoomType::Ordinary);
    }

    #[test]
    fn test_corridor_plan() {
        let a = RoomRect {
            lx: 5,
            ly: 5,
            hx: 10,
            hy: 8,
        };
        let b = RoomRect {
            lx: 30,
            ly: 12,
            hx: 35,
            hy: 16,
        };
        let corridors = plan_corridor(&a, &b);
        assert_eq!(corridors.len(), 2); // L자형
    }

    #[test]
    fn test_all_corridors() {
        let rooms = vec![
            RoomPlan {
                rect: RoomRect {
                    lx: 0,
                    ly: 0,
                    hx: 5,
                    hy: 5,
                },
                room_type: RoomType::Ordinary,
                lit: true,
                door_positions: vec![],
            },
            RoomPlan {
                rect: RoomRect {
                    lx: 20,
                    ly: 0,
                    hx: 25,
                    hy: 5,
                },
                room_type: RoomType::Ordinary,
                lit: true,
                door_positions: vec![],
            },
            RoomPlan {
                rect: RoomRect {
                    lx: 40,
                    ly: 10,
                    hx: 45,
                    hy: 15,
                },
                room_type: RoomType::Ordinary,
                lit: true,
                door_positions: vec![],
            },
        ];
        let corridors = plan_all_corridors(&rooms);
        assert_eq!(corridors.len(), 4); // 2 연결 × 2 세그먼트
    }

    #[test]
    fn test_stairs() {
        let mut rng = NetHackRng::new(42);
        let rooms = vec![
            RoomPlan {
                rect: RoomRect {
                    lx: 5,
                    ly: 5,
                    hx: 10,
                    hy: 8,
                },
                room_type: RoomType::Ordinary,
                lit: true,
                door_positions: vec![],
            },
            RoomPlan {
                rect: RoomRect {
                    lx: 30,
                    ly: 12,
                    hx: 35,
                    hy: 16,
                },
                room_type: RoomType::Ordinary,
                lit: true,
                door_positions: vec![],
            },
        ];
        let stairs = plan_stairs(&rooms, &mut rng);
        assert!(stairs.is_some());
        let s = stairs.unwrap();
        assert!(s.up_x >= 5 && s.up_x <= 10);
        assert!(s.down_x >= 30 && s.down_x <= 35);
    }
}
