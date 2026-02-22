// sp_lev.rs — 특수 레벨(Special Level) 생성기 및 DES 파서 데이터 엔진
// [v2.21.0 R9-1] 신규 생성 (원본 sp_lev.c 5,441줄 매핑 준비)

use crate::core::dungeon::Grid;
use crate::util::rng::NetHackRng;
use serde::{Deserialize, Serialize};

/// 특수 레벨 설계 명세 (원본의 .des 파일 구조를 Rust 모델로 1:1 이식)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialLevelSpec {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub rooms: Vec<SpRoomSpec>,
    #[serde(default)]
    pub objects: Vec<SpObjectSpec>,
    #[serde(default)]
    pub monsters: Vec<SpMonsterSpec>,
    #[serde(default)]
    pub traps: Vec<SpTrapSpec>,
    #[serde(default)]
    pub doors: Vec<SpDoorSpec>,
}

/// 특수 레벨 내 특정 방 명세
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpRoomSpec {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub flags: u32,    // 밝음(LIT), 미로형(IS_AMAZEMAP) 등 비트마스크
    pub room_type: u8, // OROOM, COURT, SHOP 등 방 타입 상수
}

/// 특수 레벨 내 특정 아이템 명세
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpObjectSpec {
    pub object_class: char,
    pub object_name: String,
    pub x: i32,
    pub y: i32,
    pub amount: i32,
    pub status: i32, // BUC 등 상태
}

/// 특수 레벨 내 특정 몬스터 명세
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpMonsterSpec {
    pub monster_class: char,
    pub monster_name: String,
    pub x: i32,
    pub y: i32,
    pub peaceful: i32,
    pub asleep: i32,
}

/// 특수 레벨 내 특정 트랩 명세
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpTrapSpec {
    pub trap_type: u8,
    pub x: i32,
    pub y: i32,
}

/// 특수 레벨 내 특정 문 명세
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpDoorSpec {
    pub x: i32,
    pub y: i32,
    pub mask: i8, // D_ISOPEN, D_LOCKED 등 플래그
}

/// 특수 레벨 파서 및 지형 변환 어셈블러 클래스 (원본 sp_lev.c 전역 변수/로직 캡슐화)
pub struct SpecialLevelParser;

impl SpecialLevelParser {
    /// 원본: sp_lev.c load_special()
    /// 특정 이름(예: "soko1-1")의 특수 레벨 스펙을 로드하고 파싱을 준비함.
    pub fn load_special(level_name: &str) -> Option<SpecialLevelSpec> {
        // AIHack에서는 원본 lex/yacc 대신 serde를 통한 데이터 드라이브 아키텍처 사용.
        // TODO: assets 폴더의 JSON/YAML에서 DES 직렬화 변환본 로드
        None
    }

    /// 원본: sp_lev.c create_room()
    /// 스크립트에 명시된 위치와 크기에 기반하여 룸(방) 하나를 바닥재로 깎아냄.
    pub fn create_room(
        grid: &mut Grid,
        spec: &SpRoomSpec,
        x_offset: i32,
        y_offset: i32,
        room_id: u8,
    ) {
        let start_x = spec.x + x_offset;
        let start_y = spec.y + y_offset;

        for x in start_x..(start_x + spec.w) {
            for y in start_y..(start_y + spec.h) {
                if let Some(tile) = grid.get_tile_mut(x as usize, y as usize) {
                    tile.typ = crate::core::dungeon::tile::TileType::Room;
                    tile.roomno = room_id;
                    if (spec.flags & 1) != 0 {
                        // LIT flag example
                        tile.flags |= crate::core::dungeon::tile::TileFlags::LIT;
                    }
                }
            }
        }
    }

    /// 원본: sp_lev.c wallify()
    /// 지정된 사각 영역 내 바닥 주변의 돌(Stone)을 벽(Wall) 타일로 변환함.
    pub fn wallify(grid: &mut Grid, x1: i32, y1: i32, x2: i32, y2: i32) {
        use crate::core::dungeon::tile::TileType;

        // AIHack: 원본 wallify의 간략화된 1차 벽 세우기 (Stone -> HWall / VWall)
        let w = crate::core::dungeon::COLNO as i32;
        let h = crate::core::dungeon::ROWNO as i32;

        for x in x1.max(1)..=x2.min(w - 2) {
            for y in y1.max(1)..=y2.min(h - 2) {
                let is_stone = grid
                    .get_tile(x as usize, y as usize)
                    .map(|t| t.typ == TileType::Stone)
                    .unwrap_or(false);

                if is_stone {
                    // Check neighbors to decide if it borders a Room
                    let mut borders_room = false;
                    for dx in -1..=1 {
                        for dy in -1..=1 {
                            if let Some(nt) = grid.get_tile((x + dx) as usize, (y + dy) as usize) {
                                if nt.typ == TileType::Room {
                                    borders_room = true;
                                }
                            }
                        }
                    }
                    if borders_room {
                        // Very simplified wall decision (can be expanded to TlCorner, etc)
                        if let Some(tile) = grid.get_tile_mut(x as usize, y as usize) {
                            tile.typ = TileType::CrossWall; // Placeholder for wall matrix calc
                        }
                    }
                }
            }
        }
    }

    /// 원본: sp_lev.c fill_room()
    /// 특정 방 내부 전체를 지정된 물질(물, 용암, 얼음 등)로 채움.
    pub fn fill_room(
        grid: &mut Grid,
        spec: &SpRoomSpec,
        x_offset: i32,
        y_offset: i32,
        tile_type: crate::core::dungeon::tile::TileType,
    ) {
        let start_x = spec.x + x_offset;
        let start_y = spec.y + y_offset;

        for x in start_x..(start_x + spec.w) {
            for y in start_y..(start_y + spec.h) {
                if let Some(tile) = grid.get_tile_mut(x as usize, y as usize) {
                    tile.typ = tile_type;
                }
            }
        }
    }

    /// 원본: sp_lev.c 스크립트 실행 루프
    /// 실제 스펙들을 Grid 대상의 물리적 세계로 인스턴스화 함.
    pub fn evaluate_special_level(
        grid: &mut Grid,
        spec: &SpecialLevelSpec,
        mut command_buffer: Option<&mut legion::systems::CommandBuffer>,
        _rng: &mut NetHackRng,
    ) -> bool {
        // [1] 방앗간 (Rooms) 생성
        for (i, room_spec) in spec.rooms.iter().enumerate() {
            Self::create_room(grid, room_spec, 0, 0, i as u8 + 1);
        }

        // [2] 통로 및 벽 생성 (단순화: 전체 맵기준)
        Self::wallify(
            grid,
            0,
            0,
            crate::core::dungeon::COLNO as i32,
            crate::core::dungeon::ROWNO as i32,
        );

        // [3] 문 (Doors) 설정
        for door in &spec.doors {
            if let Some(tile) = grid.get_tile_mut(door.x as usize, door.y as usize) {
                tile.typ = crate::core::dungeon::tile::TileType::Door;
                tile.doormas = door.mask;
            }
        }

        // [4] 함정 (Traps), 몬스터 (Monsters), 아이템 (Objects) 엔티티 대기열 생성
        // AIHack에서는 CommandBuffer를 통해 ECS 엔티티로 승급시킴.
        if let Some(ref mut cb) = command_buffer {
            for trap in &spec.traps {
                // TODO: cb.push()
                // cb.push((Position { x: trap.x, y: trap.y }, Trap { kind: trap.trap_type }));
                let _ = trap;
            }
            for mon in &spec.monsters {
                let _ = mon; // TODO: spawn monster
            }
            for obj in &spec.objects {
                let _ = obj; // TODO: spawn object
            }
        }

        true
    }
}

// =========================================================================
// [v2.21.0 R9-1] 특수 레벨 작동 테스트
// =========================================================================

#[cfg(test)]
mod sp_lev_tests {
    use super::*;
    use crate::core::dungeon::tile::TileType;

    fn make_test_grid() -> Grid {
        Grid::new()
    }

    #[test]
    fn test_create_and_wallify() {
        let mut grid = make_test_grid();
        let spec = SpRoomSpec {
            x: 5,
            y: 5,
            w: 4,
            h: 4,
            flags: 1,
            room_type: 1,
        };

        // 방 깎기
        SpecialLevelParser::create_room(&mut grid, &spec, 0, 0, 1);
        assert_eq!(grid.get_tile(6, 6).unwrap().typ, TileType::Room);
        assert_eq!(grid.get_tile(6, 6).unwrap().roomno, 1);

        // 4x4 크기이므로 (5..9, 5..9)까지 room
        // 4, 4 / 4, 5 / 9, 9 등은 여전히 Stone임
        assert_eq!(grid.get_tile(4, 5).unwrap().typ, TileType::Stone);

        // wallify 적용
        SpecialLevelParser::wallify(&mut grid, 0, 0, 20, 20);

        // Stone이며 Room 주변(4, 5)는 벽으로 변환됨
        assert_eq!(grid.get_tile(4, 5).unwrap().typ, TileType::CrossWall);
        // Room과 관계 없는 (2, 2)는 그대로 돌
        assert_eq!(grid.get_tile(2, 2).unwrap().typ, TileType::Stone);
    }

    #[test]
    fn test_fill_room() {
        let mut grid = make_test_grid();
        let spec = SpRoomSpec {
            x: 1,
            y: 1,
            w: 2,
            h: 2,
            flags: 0,
            room_type: 1,
        };

        SpecialLevelParser::fill_room(&mut grid, &spec, 0, 0, TileType::Water);

        assert_eq!(grid.get_tile(1, 1).unwrap().typ, TileType::Water);
        assert_eq!(grid.get_tile(2, 2).unwrap().typ, TileType::Water);
    }

    #[test]
    fn test_evaluate() {
        let mut grid = make_test_grid();
        let spec = SpecialLevelSpec {
            name: "test_sp".to_string(),
            width: 80,
            height: 21,
            rooms: vec![SpRoomSpec {
                x: 4,
                y: 4,
                w: 4,
                h: 4,
                flags: 1,
                room_type: 1,
            }],
            objects: vec![],
            monsters: vec![],
            traps: vec![],
            doors: vec![SpDoorSpec {
                x: 4,
                y: 4,
                mask: 1,
            }],
        };
        let mut rng = crate::util::rng::NetHackRng::new(0);

        let result = SpecialLevelParser::evaluate_special_level(&mut grid, &spec, None, &mut rng);
        assert!(result);

        // 문 자리
        assert_eq!(grid.get_tile(4, 4).unwrap().typ, TileType::Door);
        assert_eq!(grid.get_tile(4, 4).unwrap().doormas, 1);

        // 방 자리
        assert_eq!(grid.get_tile(5, 5).unwrap().typ, TileType::Room);
    }
}
