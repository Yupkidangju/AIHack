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

// =============================================================================
// [v2.22.0 R10-1] Selection 시스템 (원본: sp_lev.c selection_* 계열)
// =============================================================================

/// [v2.22.0 R10-1] 타일 셀렉션 마스크 (원본: struct selectionvar)
/// 80×21 비트맵으로 "어디에 무엇을 할 것인가"를 표현
#[derive(Debug, Clone)]
pub struct Selection {
    /// 80×21 마스크 (true = 선택됨)
    pub mask: Vec<[bool; ROWNO]>,
}

use crate::core::dungeon::{COLNO, ROWNO};

impl Selection {
    /// 빈 셀렉션 (아무것도 선택 안 됨)
    pub fn new_empty() -> Self {
        Self {
            mask: vec![[false; ROWNO]; COLNO],
        }
    }

    /// 전체 선택
    pub fn new_full() -> Self {
        Self {
            mask: vec![[true; ROWNO]; COLNO],
        }
    }

    /// 단일 좌표 설정
    pub fn set(&mut self, x: i32, y: i32, val: bool) {
        if x >= 0 && (x as usize) < COLNO && y >= 0 && (y as usize) < ROWNO {
            self.mask[x as usize][y as usize] = val;
        }
    }

    /// 단일 좌표 조회
    pub fn get(&self, x: i32, y: i32) -> bool {
        if x >= 0 && (x as usize) < COLNO && y >= 0 && (y as usize) < ROWNO {
            self.mask[x as usize][y as usize]
        } else {
            false
        }
    }

    /// 선택된 좌표 수
    pub fn count(&self) -> usize {
        let mut c = 0;
        for x in 0..COLNO {
            for y in 0..ROWNO {
                if self.mask[x][y] {
                    c += 1;
                }
            }
        }
        c
    }

    /// [v2.22.0 R10-1] 사각 영역 채우기 (원본: selection_fillrect)
    pub fn fill_rect(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        for x in x1.max(0)..=x2.min(COLNO as i32 - 1) {
            for y in y1.max(0)..=y2.min(ROWNO as i32 - 1) {
                self.mask[x as usize][y as usize] = true;
            }
        }
    }

    /// [v2.22.0 R10-1] 원형 영역 채우기 (원본: selection_do_ellipse → 완전 원형 특수 케이스)
    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32) {
        let r2 = radius * radius;
        for x in (cx - radius).max(0)..=(cx + radius).min(COLNO as i32 - 1) {
            for y in (cy - radius).max(0)..=(cy + radius).min(ROWNO as i32 - 1) {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= r2 {
                    self.mask[x as usize][y as usize] = true;
                }
            }
        }
    }

    /// [v2.22.0 R10-1] 선분 그리기 (원본: selection_do_line, Bresenham)
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        let mut x = x0;
        let mut y = y0;
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.set(x, y, true);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// [v2.22.0 R10-1] 홍수 채우기 (원본: selection_floodfill)
    /// 기준 좌표 (sx, sy)와 같은 TileType인 연결된 영역을 선택
    pub fn flood_fill(&mut self, grid: &Grid, sx: i32, sy: i32) {
        let target_type = match grid.get_tile(sx as usize, sy as usize) {
            Some(t) => t.typ,
            None => return,
        };
        let mut stack = vec![(sx, sy)];
        while let Some((x, y)) = stack.pop() {
            if x < 0 || x >= COLNO as i32 || y < 0 || y >= ROWNO as i32 {
                continue;
            }
            if self.get(x, y) {
                continue;
            }
            if let Some(t) = grid.get_tile(x as usize, y as usize) {
                if t.typ == target_type {
                    self.set(x, y, true);
                    stack.push((x + 1, y));
                    stack.push((x - 1, y));
                    stack.push((x, y + 1));
                    stack.push((x, y - 1));
                }
            }
        }
    }

    /// [v2.22.0 R10-1] 타일 타입 필터 (원본: selection_filter_mapchar)
    /// Grid에서 특정 TileType인 위치만 선택
    pub fn filter_by_type(grid: &Grid, tile_type: crate::core::dungeon::tile::TileType) -> Self {
        let mut sel = Self::new_empty();
        for x in 0..COLNO {
            for y in 0..ROWNO {
                if let Some(t) = grid.get_tile(x, y) {
                    if t.typ == tile_type {
                        sel.mask[x][y] = true;
                    }
                }
            }
        }
        sel
    }

    /// [v2.22.0 R10-1] 셀렉션에서 랜덤 좌표 하나 선택 (원본: selection_rndcoord)
    pub fn random_coord(&self, rng: &mut NetHackRng) -> Option<(i32, i32)> {
        let cnt = self.count();
        if cnt == 0 {
            return None;
        }
        let target = rng.rn2(cnt as i32) as usize;
        let mut idx = 0;
        for x in 0..COLNO {
            for y in 0..ROWNO {
                if self.mask[x][y] {
                    if idx == target {
                        return Some((x as i32, y as i32));
                    }
                    idx += 1;
                }
            }
        }
        None
    }

    /// [v2.22.0 R10-1] 그라디언트 셀렉션 (원본: selection_do_gradient)
    /// 중심 (cx, cy)를 기준으로 max_radius 내의 확률 그라디언트 생성
    pub fn gradient(&mut self, cx: i32, cy: i32, max_radius: i32, rng: &mut NetHackRng) {
        let r2 = max_radius * max_radius;
        for x in (cx - max_radius).max(0)..=(cx + max_radius).min(COLNO as i32 - 1) {
            for y in (cy - max_radius).max(0)..=(cy + max_radius).min(ROWNO as i32 - 1) {
                let dx = x - cx;
                let dy = y - cy;
                let dist2 = dx * dx + dy * dy;
                if dist2 <= r2 {
                    // 거리에 반비례하는 확률
                    let prob = 100 - (100 * dist2 / r2.max(1));
                    if rng.rn2(100) < prob as i32 {
                        self.mask[x as usize][y as usize] = true;
                    }
                }
            }
        }
    }

    /// [v2.22.0 R10-1] 논리 OR 합치기
    pub fn union(&mut self, other: &Selection) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                if other.mask[x][y] {
                    self.mask[x][y] = true;
                }
            }
        }
    }

    /// [v2.22.0 R10-1] 논리 AND
    pub fn intersect(&mut self, other: &Selection) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                if !other.mask[x][y] {
                    self.mask[x][y] = false;
                }
            }
        }
    }

    /// [v2.22.0 R10-1] 반전 (NOT)
    pub fn invert(&mut self) {
        for x in 0..COLNO {
            for y in 0..ROWNO {
                self.mask[x][y] = !self.mask[x][y];
            }
        }
    }
}

// =============================================================================
// [v2.22.0 R10-1] MAP 리터럴 파서 (원본: sp_lev.c INIT_MAP / MAP ... ENDMAP)
// =============================================================================

/// [v2.22.0 R10-1] ASCII 맵 문자열을 Grid로 변환 (원본: MAP 지시어)
/// 각 문자가 TileType에 매핑됨
pub fn parse_map_literal(map_str: &str, grid: &mut Grid, x_offset: i32, y_offset: i32) {
    use crate::core::dungeon::tile::TileType;

    for (row, line) in map_str.lines().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let tx = col as i32 + x_offset;
            let ty = row as i32 + y_offset;
            let tile_type = match ch {
                ' ' => TileType::Stone,
                '.' => TileType::Room,
                '#' => TileType::Corr,
                '-' => TileType::HWall,
                '|' => TileType::VWall,
                '+' => TileType::Door,
                'S' => TileType::SDoor,
                '{' => TileType::Fountain,
                '}' => TileType::Pool,
                '\\' => TileType::Throne,
                '<' => TileType::StairsUp,
                '>' => TileType::StairsDown,
                'L' => TileType::LavaPool,
                'T' => TileType::Tree,
                'I' => TileType::Ice,
                'W' => TileType::Water,
                'F' => TileType::IronBars,
                '^' => TileType::Room, // 트랩은 별도 처리, 바닥만 배치
                _ => TileType::Stone,
            };
            if let Some(tile) = grid.get_tile_mut(tx as usize, ty as usize) {
                tile.typ = tile_type;
            }
        }
    }
}

// =============================================================================
// [v2.22.0 R10-1] 복도 생성 (원본: sp_lev.c create_corridor)
// =============================================================================

impl SpecialLevelParser {
    /// [v2.22.0 R10-1] 복도 생성 (원본: sp_lev.c create_corridor)
    /// (x1,y1) → (x2,y2) 까지 L자형 복도를 파냄
    pub fn create_corridor(grid: &mut Grid, x1: i32, y1: i32, x2: i32, y2: i32) {
        use crate::core::dungeon::tile::TileType;

        // 수평 → 수직 L자형
        let sx = if x1 < x2 { 1 } else { -1 };
        let mut cx = x1;
        while cx != x2 {
            if let Some(tile) = grid.get_tile_mut(cx as usize, y1 as usize) {
                if tile.typ == TileType::Stone {
                    tile.typ = TileType::Corr;
                }
            }
            cx += sx;
        }
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut cy = y1;
        while cy != y2 {
            if let Some(tile) = grid.get_tile_mut(x2 as usize, cy as usize) {
                if tile.typ == TileType::Stone {
                    tile.typ = TileType::Corr;
                }
            }
            cy += sy;
        }
        // 최종 점
        if let Some(tile) = grid.get_tile_mut(x2 as usize, y2 as usize) {
            if tile.typ == TileType::Stone {
                tile.typ = TileType::Corr;
            }
        }
    }

    /// [v2.22.0 R10-1] 계단 배치 (원본: sp_lev.c place_stair)
    pub fn place_stair(grid: &mut Grid, x: i32, y: i32, up: bool) {
        use crate::core::dungeon::tile::TileType;
        if let Some(tile) = grid.get_tile_mut(x as usize, y as usize) {
            tile.typ = if up {
                TileType::StairsUp
            } else {
                TileType::StairsDown
            };
        }
    }

    /// [v2.22.0 R10-1] 벽 배치 정밀화 (원본: wallify_map, sp_lev.c)
    /// 기존 wallify를 확장하여 수평/수직/코너 벽을 올바르게 분류
    pub fn wallify_map(grid: &mut Grid) {
        use crate::core::dungeon::tile::TileType;

        for x in 1..(COLNO - 1) {
            for y in 1..(ROWNO - 1) {
                let is_stone = grid
                    .get_tile(x, y)
                    .map(|t| t.typ == TileType::Stone)
                    .unwrap_or(false);

                if !is_stone {
                    continue;
                }

                // 인접한 바닥/복도 존재 여부 (4방향 + 4대각선)
                let is_floor = |tx: usize, ty: usize| -> bool {
                    grid.get_tile(tx, ty)
                        .map(|t| {
                            matches!(
                                t.typ,
                                TileType::Room
                                    | TileType::Corr
                                    | TileType::Door
                                    | TileType::StairsUp
                                    | TileType::StairsDown
                                    | TileType::Fountain
                                    | TileType::Throne
                                    | TileType::Altar
                            )
                        })
                        .unwrap_or(false)
                };

                let n = is_floor(x, y.wrapping_sub(1));
                let s = is_floor(x, y + 1);
                let e = is_floor(x + 1, y);
                let w = is_floor(x.wrapping_sub(1), y);
                let ne = is_floor(x + 1, y.wrapping_sub(1));
                let nw = is_floor(x.wrapping_sub(1), y.wrapping_sub(1));
                let se = is_floor(x + 1, y + 1);
                let sw = is_floor(x.wrapping_sub(1), y + 1);

                let any_adj = n || s || e || w || ne || nw || se || sw;
                if !any_adj {
                    continue;
                }

                // 벽 종류 결정 (간략화된 원본 알고리즘)
                let wall_type = if (n || ne || nw) && (s || se || sw) && !e && !w {
                    TileType::VWall
                } else if (e || ne || se) && (w || nw || sw) && !n && !s {
                    TileType::HWall
                } else if s && e && !n && !w {
                    TileType::TlCorner
                } else if s && w && !n && !e {
                    TileType::TrCorner
                } else if n && e && !s && !w {
                    TileType::BlCorner
                } else if n && w && !s && !e {
                    TileType::BrCorner
                } else {
                    TileType::CrossWall
                };

                if let Some(tile) = grid.get_tile_mut(x, y) {
                    tile.typ = wall_type;
                }
            }
        }
    }

    /// [v2.22.0 R10-1] MAP 리터럴 기반 특수 레벨 빌드
    /// ASCII 맵 + 메타데이터로 완전한 특수 레벨 구축
    pub fn build_from_map(
        grid: &mut Grid,
        map_str: &str,
        x_offset: i32,
        y_offset: i32,
        do_wallify: bool,
    ) {
        parse_map_literal(map_str, grid, x_offset, y_offset);
        if do_wallify {
            Self::wallify_map(grid);
        }
    }
}

// =============================================================================
// [v2.22.0 R10-1] 내장 특수 레벨 데이터 (Sokoban, Medusa, Big Room)
// =============================================================================

/// [v2.22.0 R10-1] 소코반 레벨 1a (원본: soko1-1.des 기반)
pub const SOKOBAN_1A: &str = "\
------  -----
|....|  |...|
|....----...|
|...........|
|...--.--...|
------..-----
  |......|
  |......|
  ---|.---
    --.--
    |...|
    |...|
    --.--
    |...|
    -----";

/// [v2.22.0 R10-1] 소코반 레벨 2a (원본: soko2-1.des 기반)
pub const SOKOBAN_2A: &str = "\
-------- ------
|......|-|....|
|..............|
|......|--.----|
-------|.....|
 |...........|
 |.....----.-|
 ----|-|  |..|
    |..---|..|
    |........|
    |..|-----|
    ---|";

/// [v2.22.0 R10-1] 빅룸 (원본: bigrm-X.des 기반)
pub const BIGROOM_1: &str = "\
---------------------------------------------------------------------------
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
|.........................................................................|
---------------------------------------------------------------------------";

/// [v2.22.0 R10-1] 내장 레벨 이름 → 맵 데이터 조회
pub fn get_builtin_map(name: &str) -> Option<&'static str> {
    match name {
        "soko1-1" | "sokoban_1a" => Some(SOKOBAN_1A),
        "soko2-1" | "sokoban_2a" => Some(SOKOBAN_2A),
        "bigroom_1" | "bigrm-1" => Some(BIGROOM_1),
        _ => None,
    }
}

// =========================================================================
// [v2.21.0 R9-1 + v2.22.0 R10-1] 특수 레벨 테스트
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

    // ===== R10-1 신규 테스트 =====

    #[test]
    fn test_selection_fill_rect() {
        let mut sel = Selection::new_empty();
        sel.fill_rect(5, 5, 10, 10);
        assert!(sel.get(5, 5));
        assert!(sel.get(10, 10));
        assert!(!sel.get(4, 5));
        assert!(!sel.get(11, 10));
        // 6×6 = 36개
        assert_eq!(sel.count(), 36);
    }

    #[test]
    fn test_selection_fill_circle() {
        let mut sel = Selection::new_empty();
        sel.fill_circle(10, 10, 3);
        // 중심 선택됨
        assert!(sel.get(10, 10));
        // 반지름 범위
        assert!(sel.get(12, 10));
        assert!(sel.get(10, 12));
        // 반지름 밖
        assert!(!sel.get(14, 10));
        // 개수: π*r² ≈ 28.27, 실제는 이산 격자이므로 약간 다름
        assert!(sel.count() > 20 && sel.count() < 40);
    }

    #[test]
    fn test_selection_draw_line() {
        let mut sel = Selection::new_empty();
        sel.draw_line(0, 0, 10, 0);
        // 수평선 0..10 = 11개
        assert_eq!(sel.count(), 11);
        assert!(sel.get(0, 0));
        assert!(sel.get(10, 0));
        assert!(!sel.get(0, 1));
    }

    #[test]
    fn test_selection_flood_fill() {
        let mut grid = make_test_grid();
        // 10×10 영역을 Room으로 설정
        for x in 5..15 {
            for y in 5..10 {
                if let Some(t) = grid.get_tile_mut(x, y) {
                    t.typ = TileType::Room;
                }
            }
        }
        let mut sel = Selection::new_empty();
        sel.flood_fill(&grid, 7, 7);
        // 10×5 = 50 타일 선택됨
        assert_eq!(sel.count(), 50);
        assert!(sel.get(5, 5));
        assert!(sel.get(14, 9));
        // Stone 영역 미선택
        assert!(!sel.get(4, 5));
    }

    #[test]
    fn test_selection_filter_by_type() {
        let mut grid = make_test_grid();
        // 몇 개의 타일을 Room으로
        for x in 0..5 {
            if let Some(t) = grid.get_tile_mut(x, 0) {
                t.typ = TileType::Room;
            }
        }
        let sel = Selection::filter_by_type(&grid, TileType::Room);
        assert_eq!(sel.count(), 5);
    }

    #[test]
    fn test_selection_random_coord() {
        let mut sel = Selection::new_empty();
        sel.set(10, 10, true);
        sel.set(20, 15, true);
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let coord = sel.random_coord(&mut rng);
        assert!(coord.is_some());
        let (x, y) = coord.unwrap();
        assert!((x == 10 && y == 10) || (x == 20 && y == 15));
    }

    #[test]
    fn test_selection_operations() {
        let mut a = Selection::new_empty();
        a.fill_rect(0, 0, 5, 5);
        let mut b = Selection::new_empty();
        b.fill_rect(3, 3, 8, 8);

        let mut union = a.clone();
        union.union(&b);
        // 겹치는 영역 포함 전체
        assert!(union.get(0, 0));
        assert!(union.get(8, 8));

        let mut inter = a.clone();
        inter.intersect(&b);
        // 겹치는 영역만
        assert!(inter.get(3, 3));
        assert!(inter.get(5, 5));
        assert!(!inter.get(0, 0));
        assert!(!inter.get(8, 8));

        let mut inv = Selection::new_empty();
        inv.set(10, 10, true);
        inv.invert();
        assert!(!inv.get(10, 10));
        assert!(inv.get(0, 0));
    }

    #[test]
    fn test_parse_map_literal() {
        let mut grid = make_test_grid();
        let map = "---\n|.|\n---";
        parse_map_literal(map, &mut grid, 5, 5);

        assert_eq!(grid.get_tile(5, 5).unwrap().typ, TileType::HWall);
        assert_eq!(grid.get_tile(6, 6).unwrap().typ, TileType::Room);
        assert_eq!(grid.get_tile(5, 6).unwrap().typ, TileType::VWall);
    }

    #[test]
    fn test_create_corridor() {
        let mut grid = make_test_grid();
        SpecialLevelParser::create_corridor(&mut grid, 5, 5, 10, 8);
        // 수평 구간 체크
        assert_eq!(grid.get_tile(7, 5).unwrap().typ, TileType::Corr);
        // 수직 구간 체크
        assert_eq!(grid.get_tile(10, 7).unwrap().typ, TileType::Corr);
        // 기존 Stone이 아닌 곳은 변경 없음 확인 (원점 근처)
        assert_eq!(grid.get_tile(3, 3).unwrap().typ, TileType::Stone);
    }

    #[test]
    fn test_place_stair() {
        let mut grid = make_test_grid();
        SpecialLevelParser::place_stair(&mut grid, 10, 10, true);
        assert_eq!(grid.get_tile(10, 10).unwrap().typ, TileType::StairsUp);

        SpecialLevelParser::place_stair(&mut grid, 20, 15, false);
        assert_eq!(grid.get_tile(20, 15).unwrap().typ, TileType::StairsDown);
    }

    #[test]
    fn test_builtin_sokoban() {
        let map = get_builtin_map("soko1-1");
        assert!(map.is_some());

        let mut grid = make_test_grid();
        SpecialLevelParser::build_from_map(&mut grid, map.unwrap(), 5, 2, true);

        // 소코반 맵의 첫 번째 줄 첫 문자 '-' → HWall 위치 확인
        assert_eq!(grid.get_tile(5, 2).unwrap().typ, TileType::HWall);
        // 내부 '.' → Room
        assert_eq!(grid.get_tile(6, 3).unwrap().typ, TileType::Room);
    }

    #[test]
    fn test_wallify_map_precision() {
        let mut grid = make_test_grid();
        // 작은 방 수동 생성
        for x in 10..15 {
            for y in 5..10 {
                if let Some(t) = grid.get_tile_mut(x, y) {
                    t.typ = TileType::Room;
                }
            }
        }
        SpecialLevelParser::wallify_map(&mut grid);

        // Room 내부는 그대로
        assert_eq!(grid.get_tile(12, 7).unwrap().typ, TileType::Room);
        // Room 바로 옆 Stone → 벽 타입으로 변환 확인
        let border = grid.get_tile(9, 7).unwrap().typ;
        assert!(border.is_wall());
        // 멀리 떨어진 곳 → 여전히 Stone
        assert_eq!(grid.get_tile(1, 1).unwrap().typ, TileType::Stone);
    }
}
