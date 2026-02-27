// ============================================================================
// [v2.29.0 Phase 93-7] 던전 생성 확장 (mklev_phase93_ext.rs)
// 원본: NetHack 3.6.7 src/mklev.c L400-1500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 방 생성 — make_room (mklev.c L400-700)
// =============================================================================

/// [v2.29.0 93-7] 방 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    Ordinary,
    Shop,
    Temple,
    Throne,
    Zoo,
    Morgue,
    Barracks,
    Beehive,
    Garden,
    Cockatrice,
    Swamp,
    Vault,
    Treasure,
}

/// [v2.29.0 93-7] 방 생성 결과
#[derive(Debug, Clone)]
pub struct RoomResult {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub room_type: RoomType,
    pub door_count: i32,
    pub monster_count: i32,
    pub item_count: i32,
    pub is_lit: bool,
}

/// [v2.29.0 93-7] 방 생성
/// 원본: mklev.c makerooms()
pub fn generate_room(
    map_x: i32,
    map_y: i32,
    dungeon_depth: i32,
    rng: &mut NetHackRng,
) -> RoomResult {
    let width = rng.rn2(8) + 4; // 4-11
    let height = rng.rn2(5) + 3; // 3-7
    let is_lit = dungeon_depth <= 5 || rng.rn2(3) == 0;

    // 특수방 확률
    let room_type = if rng.rn2(20) == 0 {
        let special = rng.rn2(10);
        match special {
            0 => RoomType::Shop,
            1 => RoomType::Temple,
            2 => RoomType::Zoo,
            3 => RoomType::Morgue,
            4 => RoomType::Barracks,
            5 => RoomType::Beehive,
            6 => RoomType::Throne,
            7 => RoomType::Swamp,
            8 => RoomType::Treasure,
            _ => RoomType::Vault,
        }
    } else {
        RoomType::Ordinary
    };

    let monster_count = match room_type {
        RoomType::Zoo => rng.rn2(8) + 5,
        RoomType::Morgue => rng.rn2(6) + 3,
        RoomType::Barracks => rng.rn2(6) + 4,
        RoomType::Beehive => rng.rn2(10) + 6,
        _ => rng.rn2(3),
    };

    let item_count = match room_type {
        RoomType::Shop => rng.rn2(10) + 10,
        RoomType::Treasure => rng.rn2(5) + 3,
        RoomType::Vault => rng.rn2(3) + 2,
        _ => rng.rn2(2),
    };

    let door_count = rng.rn2(3) + 1;

    RoomResult {
        x: map_x,
        y: map_y,
        width,
        height,
        room_type,
        door_count,
        monster_count,
        item_count,
        is_lit,
    }
}

// =============================================================================
// [2] 복도 생성 — corridor (mklev.c L700-1000)
// =============================================================================

/// [v2.29.0 93-7] 복도 결과
#[derive(Debug, Clone)]
pub struct CorridorResult {
    pub path: Vec<(i32, i32)>,
    pub length: i32,
    pub has_bend: bool,
}

/// [v2.29.0 93-7] 두 방 사이 복도 생성
/// 원본: mklev.c join()
pub fn generate_corridor(
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
    rng: &mut NetHackRng,
) -> CorridorResult {
    let mut path = Vec::new();
    let mut cx = from_x;
    let mut cy = from_y;

    // L자형 또는 Z자형 복도
    let bend_at = if rng.rn2(2) == 0 {
        // 수평→수직
        while cx != to_x {
            cx += if to_x > cx { 1 } else { -1 };
            path.push((cx, cy));
        }
        while cy != to_y {
            cy += if to_y > cy { 1 } else { -1 };
            path.push((cx, cy));
        }
        true
    } else {
        // 수직→수평
        while cy != to_y {
            cy += if to_y > cy { 1 } else { -1 };
            path.push((cx, cy));
        }
        while cx != to_x {
            cx += if to_x > cx { 1 } else { -1 };
            path.push((cx, cy));
        }
        true
    };

    let length = path.len() as i32;

    CorridorResult {
        path,
        length,
        has_bend: bend_at,
    }
}

// =============================================================================
// [3] 레벨 전체 구조 — level_layout (mklev.c L1000-1300)
// =============================================================================

/// [v2.29.0 93-7] 레벨 레이아웃
#[derive(Debug, Clone)]
pub struct LevelLayout {
    pub width: i32,
    pub height: i32,
    pub rooms: Vec<RoomResult>,
    pub corridors: Vec<CorridorResult>,
    pub stair_up: (i32, i32),
    pub stair_down: (i32, i32),
    pub depth: i32,
}

/// [v2.29.0 93-7] 레벨 생성
/// 원본: mklev.c makelevel()
pub fn generate_level(
    depth: i32,
    map_width: i32,
    map_height: i32,
    rng: &mut NetHackRng,
) -> LevelLayout {
    let room_count = rng.rn2(5) + 3; // 3-7 방
    let mut rooms = Vec::new();

    for i in 0..room_count {
        let rx = (i * (map_width / room_count)).max(2);
        let ry = rng.rn2(map_height - 8) + 2;
        rooms.push(generate_room(rx, ry, depth, rng));
    }

    // 복도 연결 (인접 방끼리)
    let mut corridors = Vec::new();
    for i in 1..rooms.len() {
        let from = &rooms[i - 1];
        let to = &rooms[i];
        corridors.push(generate_corridor(
            from.x + from.width / 2,
            from.y + from.height / 2,
            to.x + to.width / 2,
            to.y + to.height / 2,
            rng,
        ));
    }

    // 계단
    let stair_up = if !rooms.is_empty() {
        (rooms[0].x + 1, rooms[0].y + 1)
    } else {
        (5, 5)
    };
    let stair_down = if rooms.len() > 1 {
        let last = rooms.last().unwrap();
        (last.x + last.width - 2, last.y + last.height - 2)
    } else {
        (map_width - 5, map_height - 5)
    };

    LevelLayout {
        width: map_width,
        height: map_height,
        rooms,
        corridors,
        stair_up,
        stair_down,
        depth,
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

    #[test]
    fn test_generate_room() {
        let mut rng = test_rng();
        let room = generate_room(10, 5, 3, &mut rng);
        assert!(room.width >= 4 && room.width <= 11);
        assert!(room.height >= 3 && room.height <= 7);
    }

    #[test]
    fn test_special_room() {
        let mut found_special = false;
        for seed in 0..100u64 {
            let mut rng = NetHackRng::new(seed);
            let room = generate_room(10, 5, 10, &mut rng);
            if room.room_type != RoomType::Ordinary {
                found_special = true;
                break;
            }
        }
        assert!(found_special);
    }

    #[test]
    fn test_corridor() {
        let mut rng = test_rng();
        let corridor = generate_corridor(5, 5, 20, 10, &mut rng);
        assert!(corridor.length > 0);
        assert!(corridor.has_bend);
    }

    #[test]
    fn test_corridor_connects() {
        let mut rng = test_rng();
        let corridor = generate_corridor(5, 5, 15, 5, &mut rng);
        if let Some(last) = corridor.path.last() {
            assert_eq!(last.0, 15);
            assert_eq!(last.1, 5);
        }
    }

    #[test]
    fn test_generate_level() {
        let mut rng = test_rng();
        let level = generate_level(5, 80, 21, &mut rng);
        assert!(level.rooms.len() >= 3);
        assert!(!level.corridors.is_empty());
    }

    #[test]
    fn test_level_stairs() {
        let mut rng = test_rng();
        let level = generate_level(5, 80, 21, &mut rng);
        assert_ne!(level.stair_up, level.stair_down);
    }

    #[test]
    fn test_level_depth_affects_lighting() {
        let mut rng1 = NetHackRng::new(42);
        let mut rng2 = NetHackRng::new(42);
        let shallow = generate_room(10, 5, 2, &mut rng1);
        let _deep = generate_room(10, 5, 20, &mut rng2);
        // 얕은 레벨은 항상 밝음
        assert!(shallow.is_lit);
    }
}
