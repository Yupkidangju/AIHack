// ============================================================================
// [v2.36.0 Phase 100-4] 맵 생성 통합 (mapgen_phase100_ext.rs)
// 원본: NetHack 3.6.7 src/mklev.c + mkroom.c 핵심 미이식 함수 통합
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 맵 생성 — map_generation (mklev.c 핵심)
// =============================================================================

/// [v2.36.0 100-4] 방 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomType {
    Ordinary,
    Shop,
    Vault,
    Throne,
    Beehive,
    Zoo,
    Temple,
    Swamp,
    Morgue,
    Barracks,
    Garden,
    Cocknest,
    Anthole,
    Leprechaun,
}

/// [v2.36.0 100-4] 방 데이터
#[derive(Debug, Clone)]
pub struct Room {
    pub room_type: RoomType,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub door_count: i32,
    pub lit: bool,
    pub special_features: Vec<String>,
}

/// [v2.36.0 100-4] 복도 데이터
#[derive(Debug, Clone)]
pub struct Corridor {
    pub from_room: usize,
    pub to_room: usize,
    pub length: i32,
    pub is_lit: bool,
}

/// [v2.36.0 100-4] 레벨 레이아웃
#[derive(Debug, Clone)]
pub struct LevelLayout {
    pub depth: i32,
    pub width: i32,
    pub height: i32,
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub up_stairs: (i32, i32),
    pub down_stairs: (i32, i32),
    pub features: Vec<String>,
}

/// [v2.36.0 100-4] 레벨 생성
pub fn generate_level(depth: i32, branch: &str, rng: &mut NetHackRng) -> LevelLayout {
    let width = 79;
    let height = 21;

    // 방 수 결정 (깊이에 따라)
    let room_count = (rng.rn2(4) + 3).min(8) as usize;
    let mut rooms = Vec::new();

    for i in 0..room_count {
        let rw = rng.rn2(8) + 4; // 4~11
        let rh = rng.rn2(5) + 3; // 3~7
        let rx = rng.rn2(width - rw - 2) + 1;
        let ry = rng.rn2(height - rh - 2) + 1;

        let room_type = if i == 0 && depth > 5 && rng.rn2(5) == 0 {
            match rng.rn2(6) {
                0 => RoomType::Shop,
                1 => RoomType::Zoo,
                2 => RoomType::Temple,
                3 => RoomType::Throne,
                4 => RoomType::Vault,
                _ => RoomType::Morgue,
            }
        } else {
            RoomType::Ordinary
        };

        let features = match room_type {
            RoomType::Shop => vec!["상점 주인".to_string(), "물건 전시".to_string()],
            RoomType::Zoo => vec!["다수의 동물".to_string()],
            RoomType::Temple => vec!["제단".to_string(), "사제".to_string()],
            RoomType::Throne => vec!["왕좌".to_string()],
            RoomType::Vault => vec!["금화 더미".to_string()],
            RoomType::Morgue => vec!["언데드".to_string(), "관".to_string()],
            _ => vec![],
        };

        rooms.push(Room {
            room_type,
            x: rx,
            y: ry,
            width: rw,
            height: rh,
            door_count: rng.rn2(3) + 1,
            lit: depth < 10 || rng.rn2(3) > 0,
            special_features: features,
        });
    }

    // 복도 생성 (인접 방 연결)
    let mut corridors = Vec::new();
    for i in 0..rooms.len().saturating_sub(1) {
        corridors.push(Corridor {
            from_room: i,
            to_room: i + 1,
            length: rng.rn2(10) + 3,
            is_lit: rng.rn2(3) == 0,
        });
    }

    // 계단 배치
    let up_x = rooms.first().map(|r| r.x + r.width / 2).unwrap_or(10);
    let up_y = rooms.first().map(|r| r.y + r.height / 2).unwrap_or(5);
    let dn_x = rooms.last().map(|r| r.x + r.width / 2).unwrap_or(60);
    let dn_y = rooms.last().map(|r| r.y + r.height / 2).unwrap_or(15);

    let features = match branch {
        "게헨놈" => vec!["용암 웅덩이".to_string()],
        "광산" => vec!["보석 광맥".to_string()],
        _ => vec![],
    };

    LevelLayout {
        depth,
        width,
        height,
        rooms,
        corridors,
        up_stairs: (up_x, up_y),
        down_stairs: (dn_x, dn_y),
        features,
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
    fn test_basic_generation() {
        let mut rng = test_rng();
        let level = generate_level(5, "Main", &mut rng);
        assert!(level.rooms.len() >= 3);
        assert!(!level.corridors.is_empty());
    }

    #[test]
    fn test_stairs() {
        let mut rng = test_rng();
        let level = generate_level(10, "Main", &mut rng);
        assert_ne!(level.up_stairs, level.down_stairs);
    }

    #[test]
    fn test_deep_level() {
        let mut rng = test_rng();
        let level = generate_level(25, "Main", &mut rng);
        assert!(level.rooms.len() >= 3);
    }

    #[test]
    fn test_gehennom_features() {
        let mut rng = test_rng();
        let level = generate_level(25, "게헨놈", &mut rng);
        assert!(level.features.iter().any(|f| f.contains("용암")));
    }

    #[test]
    fn test_mine_features() {
        let mut rng = test_rng();
        let level = generate_level(5, "광산", &mut rng);
        assert!(level.features.iter().any(|f| f.contains("보석")));
    }

    #[test]
    fn test_room_size() {
        let mut rng = test_rng();
        let level = generate_level(5, "Main", &mut rng);
        for room in &level.rooms {
            assert!(room.width >= 4 && room.width <= 11);
            assert!(room.height >= 3 && room.height <= 7);
        }
    }

    #[test]
    fn test_corridors_connect() {
        let mut rng = test_rng();
        let level = generate_level(10, "Main", &mut rng);
        assert_eq!(level.corridors.len(), level.rooms.len() - 1);
    }

    #[test]
    fn test_lit_rooms() {
        let mut rng = test_rng();
        let level = generate_level(3, "Main", &mut rng);
        // 얕은 층은 대부분 밝아야 함
        let lit_count = level.rooms.iter().filter(|r| r.lit).count();
        assert!(lit_count > 0);
    }
}
