use std::collections::{HashSet, VecDeque};

use aihack_core::{
    domain::{
        level::{BranchGraph, GameLevel, SpecialRoomKind},
        map::{GameMap, MapLayout},
        tile::{DoorState, TileKind, TrapKind},
    },
    error::ContentError,
    ids::{BranchId, LevelId},
    position::{Direction, Pos},
};

use crate::save::MAX_PERSISTED_TEXT_BYTES;

pub const CAMPAIGN_MAP_WIDTH: i16 = 80;
pub const CAMPAIGN_MAP_HEIGHT: i16 = 21;
const MAX_ENGRAVINGS_PER_LEVEL: usize = 32;
const MAX_ENGRAVING_TEXT_CHARS: usize = 80;

pub(crate) fn level_ids() -> Vec<LevelId> {
    BranchGraph::campaign_v3().active_level_ids()
}

struct Layout(Vec<(Pos, TileKind)>);
impl MapLayout for Layout {
    fn level_id(&self) -> &str {
        "campaign generated"
    }
    fn depth(&self) -> i16 {
        1
    }
    fn dimensions(&self) -> (i16, i16) {
        (CAMPAIGN_MAP_WIDTH, CAMPAIGN_MAP_HEIGHT)
    }
    fn tile_overrides(&self) -> Result<Vec<(Pos, TileKind)>, ContentError> {
        Ok(self.0.clone())
    }
}

#[derive(Clone, Copy)]
struct RoomRect {
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
    center: Pos,
    lit: bool,
}

fn next_rng(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn seed_for_level(seed: u64, id: LevelId) -> u64 {
    let branch_salt = match id.branch {
        BranchId::Main => 0,
        BranchId::Mines => 0xa076_1d64_78bd_642f,
        BranchId::Sokoban => 0xe703_7ed1_a0b4_28db,
        BranchId::Quest => 0x8ebc_6af0_9c88_c6e3,
        BranchId::Gehennom => 0x5899_65cc_7537_4cc3,
    };
    seed ^ (id.depth as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ branch_salt
}

/// Combat RNG와 독립인 Generator v3 (`80x21`).
/// 가변 6개 방, 스패닝 트리 + 루프 통로, 어둠/조명, 특수방, 문·비밀문·함정을 배치하고 BFS 도달성을 보장한다.
pub(crate) fn generate(seed: u64, id: LevelId) -> Result<(GameLevel, [Pos; 6]), ContentError> {
    let mut state = seed_for_level(seed, id);

    // 3 columns x 2 rows 구역에 6개 가변 방을 비중첩 배치한다.
    // Room order: 0(top-left), 1(top-mid), 2(top-right), 3(bottom-right), 4(bottom-mid), 5(bottom-left)
    let zones: [(i16, i16, i16, i16); 6] = [
        (2, 22, 2, 8),    // 0
        (28, 49, 2, 8),   // 1
        (54, 76, 2, 8),   // 2
        (54, 76, 12, 18), // 3
        (28, 49, 12, 18), // 4
        (2, 22, 12, 18),  // 5
    ];

    let mut rooms = [RoomRect {
        x1: 0,
        y1: 0,
        x2: 0,
        y2: 0,
        center: Pos { x: 0, y: 0 },
        lit: true,
    }; 6];

    for (idx, &(zx1, zx2, zy1, zy2)) in zones.iter().enumerate() {
        let max_w = (zx2 - zx1 + 1).min(13);
        let w = 7 + (next_rng(&mut state) % u64::from((max_w - 6).max(1) as u16)) as i16;
        let max_h = (zy2 - zy1 + 1).min(7);
        let h = 5 + (next_rng(&mut state) % u64::from((max_h - 4).max(1) as u16)) as i16;
        let x_slack = (zx2 - zx1 + 1 - w).max(1);
        let y_slack = (zy2 - zy1 + 1 - h).max(1);
        let x1 = zx1 + (next_rng(&mut state) % u64::from(x_slack as u16)) as i16;
        let y1 = zy1 + (next_rng(&mut state) % u64::from(y_slack as u16)) as i16;
        let x2 = x1 + w - 1;
        let y2 = y1 + h - 1;
        let center = Pos {
            x: x1 + w / 2,
            y: y1 + h / 2,
        };
        let lit_roll = next_rng(&mut state);
        let lit = if id == LevelId::main(1) && idx == 0 {
            true
        } else if idx == 5 && id != LevelId::main(1) {
            false
        } else {
            (lit_roll & 1) == 0
        };
        rooms[idx] = RoomRect {
            x1,
            y1,
            x2,
            y2,
            center,
            lit,
        };
    }

    let mut grid = vec![TileKind::Wall; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let mut lit_tiles = vec![false; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let mut in_room = vec![false; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];

    let tile_idx =
        |pos: Pos| -> usize { (pos.y as usize * CAMPAIGN_MAP_WIDTH as usize) + pos.x as usize };

    for room in &rooms {
        for y in room.y1..=room.y2 {
            for x in room.x1..=room.x2 {
                let p = Pos { x, y };
                let idx = tile_idx(p);
                grid[idx] = TileKind::Floor;
                in_room[idx] = true;
            }
        }
        if room.lit {
            for y in (room.y1 - 1)..=(room.y2 + 1) {
                for x in (room.x1 - 1)..=(room.x2 + 1) {
                    let idx = tile_idx(Pos { x, y });
                    lit_tiles[idx] = true;
                }
            }
        }
    }

    // 스패닝 트리 + 루프 통로 연결
    let mut edges = vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5), (5, 0)];
    if (next_rng(&mut state) & 1) == 0 {
        edges.push((1, 4));
    }
    let horizontal_first_seed = next_rng(&mut state);
    for (edge_idx, (a_idx, b_idx)) in edges.into_iter().enumerate() {
        let a = rooms[a_idx].center;
        let b = rooms[b_idx].center;
        if ((horizontal_first_seed >> (edge_idx % 8)) & 1) == 0 {
            for x in a.x.min(b.x)..=a.x.max(b.x) {
                grid[tile_idx(Pos { x, y: a.y })] = TileKind::Floor;
            }
            for y in a.y.min(b.y)..=a.y.max(b.y) {
                grid[tile_idx(Pos { x: b.x, y })] = TileKind::Floor;
            }
        } else {
            for y in a.y.min(b.y)..=a.y.max(b.y) {
                grid[tile_idx(Pos { x: a.x, y })] = TileKind::Floor;
            }
            for x in a.x.min(b.x)..=a.x.max(b.x) {
                grid[tile_idx(Pos { x, y: b.y })] = TileKind::Floor;
            }
        }
    }

    // 통로-방 경계의 1칸 폭 출입구 후보 수집
    let mut doorway_candidates = Vec::new();
    for room in &rooms {
        for x in room.x1..=room.x2 {
            for y in [room.y1 - 1, room.y2 + 1] {
                let p = Pos { x, y };
                if grid[tile_idx(p)] == TileKind::Floor
                    && !in_room[tile_idx(p)]
                    && is_clean_choke_doorway(&grid, p)
                {
                    doorway_candidates.push(p);
                }
            }
        }
        for y in room.y1..=room.y2 {
            for x in [room.x1 - 1, room.x2 + 1] {
                let p = Pos { x, y };
                if grid[tile_idx(p)] == TileKind::Floor
                    && !in_room[tile_idx(p)]
                    && is_clean_choke_doorway(&grid, p)
                {
                    doorway_candidates.push(p);
                }
            }
        }
    }
    doorway_candidates.sort_by_key(|p| (p.y, p.x));
    doorway_candidates.dedup();
    // 서로 인접하지 않은 출입구 우선 필터링
    let mut chosen_doors: Vec<Pos> = Vec::new();
    for candidate in doorway_candidates {
        if chosen_doors
            .iter()
            .all(|d| d.chebyshev_distance(candidate) > 2)
        {
            chosen_doors.push(candidate);
        }
    }
    if !chosen_doors.is_empty() {
        let first_door = if id
            == (LevelId {
                branch: BranchId::Mines,
                depth: 2,
            }) {
            TileKind::Door(DoorState::Locked)
        } else {
            TileKind::Door(DoorState::Closed)
        };
        grid[tile_idx(chosen_doors[0])] = first_door;
    }
    if chosen_doors.len() >= 2 {
        let second_door = if id.depth % 2 == 1 {
            TileKind::Door(DoorState::Closed)
        } else {
            TileKind::HiddenDoor
        };
        grid[tile_idx(chosen_doors[1])] = second_door;
    }
    if chosen_doors.len() >= 3 && (next_rng(&mut state) & 1) == 0 {
        grid[tile_idx(chosen_doors[2])] = TileKind::Door(DoorState::Closed);
    }

    // 방 1·방 2 함정 (아이템/계단 스폰 좌표와 겹치지 않도록 center +/- 2 주변에 배치)
    let room1_trap = if id.depth % 2 == 1 {
        TrapKind::Pit
    } else {
        TrapKind::Dart
    };
    let trap1_pos = Pos {
        x: rooms[1].center.x - 2,
        y: rooms[1].center.y,
    };
    if grid[tile_idx(trap1_pos)] == TileKind::Floor {
        grid[tile_idx(trap1_pos)] = TileKind::HiddenTrap(room1_trap);
    }
    if matches!(
        (id.branch, id.depth),
        (BranchId::Main, 3 | 5 | 6) | (BranchId::Mines, 2)
    ) {
        let trap2_pos = Pos {
            x: rooms[2].center.x - 2,
            y: rooms[2].center.y,
        };
        if grid[tile_idx(trap2_pos)] == TileKind::Floor {
            grid[tile_idx(trap2_pos)] = TileKind::HiddenTrap(TrapKind::Bear);
        }
    }

    // 특수방 배치: Shop(Main 2, Mines 1), Temple/Altar(Main 3, Mines 2), Barracks(Main 5), Vault(Main 6)
    let mut special_room = None;
    if id == LevelId::main(2)
        || id
            == (LevelId {
                branch: BranchId::Mines,
                depth: 1,
            })
    {
        special_room = Some(SpecialRoomKind::Shop);
        let shop_pos = Pos {
            x: rooms[1].center.x,
            y: rooms[1].center.y - 1,
        };
        grid[tile_idx(shop_pos)] = TileKind::Shop;
    } else if id == LevelId::main(3)
        || id
            == (LevelId {
                branch: BranchId::Mines,
                depth: 2,
            })
    {
        special_room = Some(SpecialRoomKind::Temple);
        let altar_pos = Pos {
            x: rooms[1].center.x,
            y: rooms[1].center.y - 1,
        };
        grid[tile_idx(altar_pos)] = TileKind::Altar;
    } else if id == LevelId::main(5) {
        special_room = Some(SpecialRoomKind::Barracks);
    } else if id == LevelId::main(6) {
        special_room = Some(SpecialRoomKind::Vault);
    }

    // P9 지형(분수/싱크/물/용암) 및 신규 함정 9종은 Room 4·5 모서리에 결정론적으로 배치한다.
    let mut set_if_floor = |pos: Pos, tile: TileKind| {
        if grid[tile_idx(pos)] == TileKind::Floor {
            grid[tile_idx(pos)] = tile;
        }
    };
    match (id.branch, id.depth) {
        (BranchId::Main, 1) => {
            set_if_floor(
                Pos {
                    x: rooms[4].x1,
                    y: rooms[4].y1,
                },
                TileKind::Fountain,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y1,
                },
                TileKind::HiddenTrap(TrapKind::SleepingGas),
            );
        }
        (BranchId::Main, 2) => {
            set_if_floor(
                Pos {
                    x: rooms[4].x2,
                    y: rooms[4].y1,
                },
                TileKind::Sink,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y1,
                },
                TileKind::HiddenTrap(TrapKind::Rust),
            );
        }
        (BranchId::Main, 3) => {
            set_if_floor(
                Pos {
                    x: rooms[4].x1,
                    y: rooms[4].y1,
                },
                TileKind::Fountain,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y2,
                },
                TileKind::Water,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1 + 1,
                    y: rooms[5].y2,
                },
                TileKind::Water,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y1,
                },
                TileKind::HiddenTrap(TrapKind::Teleport),
            );
            set_if_floor(
                Pos {
                    x: rooms[4].x2,
                    y: rooms[4].y2,
                },
                TileKind::HiddenTrap(TrapKind::AntiMagic),
            );
        }
        (BranchId::Main, 4) => {
            set_if_floor(
                Pos {
                    x: rooms[4].x2,
                    y: rooms[4].y1,
                },
                TileKind::Sink,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y2,
                },
                TileKind::Water,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1 + 1,
                    y: rooms[5].y2,
                },
                TileKind::Water,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y1,
                },
                TileKind::HiddenTrap(TrapKind::FallingRock),
            );
            set_if_floor(
                Pos {
                    x: rooms[4].x1,
                    y: rooms[4].y2,
                },
                TileKind::HiddenTrap(TrapKind::Hole),
            );
        }
        (BranchId::Main, 5) => {
            set_if_floor(
                Pos {
                    x: rooms[5].x2,
                    y: rooms[5].y2,
                },
                TileKind::Lava,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x2 - 1,
                    y: rooms[5].y2,
                },
                TileKind::Lava,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y1,
                },
                TileKind::HiddenTrap(TrapKind::Fire),
            );
            set_if_floor(
                Pos {
                    x: rooms[4].x2,
                    y: rooms[4].y2,
                },
                TileKind::HiddenTrap(TrapKind::Polymorph),
            );
        }
        (BranchId::Main, 6) => {
            set_if_floor(
                Pos {
                    x: rooms[5].x2,
                    y: rooms[5].y2,
                },
                TileKind::Lava,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x2 - 1,
                    y: rooms[5].y2,
                },
                TileKind::Lava,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y1,
                },
                TileKind::HiddenTrap(TrapKind::Magic),
            );
        }
        (BranchId::Mines, 1) => {
            set_if_floor(
                Pos {
                    x: rooms[4].x1,
                    y: rooms[4].y1,
                },
                TileKind::Fountain,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y2,
                },
                TileKind::Water,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1 + 1,
                    y: rooms[5].y2,
                },
                TileKind::Water,
            );
            set_if_floor(
                Pos {
                    x: rooms[4].x2,
                    y: rooms[4].y2,
                },
                TileKind::HiddenTrap(TrapKind::FallingRock),
            );
        }
        (BranchId::Mines, 2) => {
            set_if_floor(
                Pos {
                    x: rooms[4].x2,
                    y: rooms[4].y1,
                },
                TileKind::Sink,
            );
            set_if_floor(
                Pos {
                    x: rooms[5].x1,
                    y: rooms[5].y1,
                },
                TileKind::HiddenTrap(TrapKind::Rust),
            );
        }
        _ => {}
    }

    // 계단 배치
    grid[tile_idx(rooms[0].center)] = TileKind::StairsUp;
    let has_down = BranchGraph::campaign_v3().descend_target(id).is_some();
    if has_down {
        grid[tile_idx(rooms[3].center)] = TileKind::StairsDown;
    }

    // 필수 BFS 연결성 검증 (단절 시 직접 통로 보강)
    let centers = rooms.map(|r| r.center);
    if !all_targets_reachable(&grid, centers[0], &centers[1..]) {
        for pair in centers.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            for x in a.x.min(b.x)..=a.x.max(b.x) {
                let idx = tile_idx(Pos { x, y: a.y });
                if grid[idx] == TileKind::Wall {
                    grid[idx] = TileKind::Floor;
                }
            }
            for y in a.y.min(b.y)..=a.y.max(b.y) {
                let idx = tile_idx(Pos { x: b.x, y });
                if grid[idx] == TileKind::Wall {
                    grid[idx] = TileKind::Floor;
                }
            }
        }
    }

    let mut overrides = Vec::with_capacity(grid.len());
    for y in 0..CAMPAIGN_MAP_HEIGHT {
        for x in 0..CAMPAIGN_MAP_WIDTH {
            let p = Pos { x, y };
            overrides.push((p, grid[tile_idx(p)]));
        }
    }

    Ok((
        GameLevel {
            id,
            map: GameMap::from_level_data(&Layout(overrides))?,
            explored: Vec::new(),
            lit_tiles,
            special_room,
            engravings: Vec::new(),
            altar_alignments: Vec::new(),
            special_level_id: None,
        },
        centers,
    ))
}

fn is_clean_choke_doorway(grid: &[TileKind], pos: Pos) -> bool {
    if pos.x <= 1
        || pos.x >= CAMPAIGN_MAP_WIDTH - 2
        || pos.y <= 1
        || pos.y >= CAMPAIGN_MAP_HEIGHT - 2
    {
        return false;
    }
    let at = |x: i16, y: i16| grid[(y as usize * CAMPAIGN_MAP_WIDTH as usize) + x as usize];
    let ew_pass = at(pos.x - 1, pos.y) == TileKind::Floor
        && at(pos.x + 1, pos.y) == TileKind::Floor
        && at(pos.x, pos.y - 1) == TileKind::Wall
        && at(pos.x, pos.y + 1) == TileKind::Wall;
    let ns_pass = at(pos.x, pos.y - 1) == TileKind::Floor
        && at(pos.x, pos.y + 1) == TileKind::Floor
        && at(pos.x - 1, pos.y) == TileKind::Wall
        && at(pos.x + 1, pos.y) == TileKind::Wall;
    ew_pass || ns_pass
}

fn all_targets_reachable(grid: &[TileKind], start: Pos, targets: &[Pos]) -> bool {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(start);
    queue.push_back(start);
    while let Some(pos) = queue.pop_front() {
        for dir in Direction::ALL {
            let next = pos.offset(dir.delta());
            if next.x < 1
                || next.x >= CAMPAIGN_MAP_WIDTH - 1
                || next.y < 1
                || next.y >= CAMPAIGN_MAP_HEIGHT - 1
                || visited.contains(&next)
            {
                continue;
            }
            let tile = grid[(next.y as usize * CAMPAIGN_MAP_WIDTH as usize) + next.x as usize];
            if !matches!(tile, TileKind::Wall | TileKind::Water | TileKind::Lava) {
                visited.insert(next);
                queue.push_back(next);
            }
        }
    }
    targets.iter().all(|target| visited.contains(target))
}

/// 저장된 캠페인 지도의 구조적 불변조건을 검사한다(내부 굴착 Wall->Floor는 허용, 외곽/계단/특수타일 위조·단절 거부).
pub(crate) fn validate_persisted_map(seed: u64, actual: &GameLevel) -> Result<(), String> {
    let id = actual.id;
    let (expected, _) = generate(seed, id).map_err(|error| error.to_string())?;
    if actual.map.width != CAMPAIGN_MAP_WIDTH || actual.map.height != CAMPAIGN_MAP_HEIGHT {
        return Err("campaign generator/map dimension mismatch".to_owned());
    }
    let expected_tile_count = (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize;
    if actual.map.tiles().len() != expected_tile_count {
        return Err("campaign generator/map tile count mismatch".to_owned());
    }
    if !actual.lit_tiles.is_empty() && actual.lit_tiles.len() != expected_tile_count {
        return Err("campaign map lit_tiles length mismatch".to_owned());
    }
    if actual.engravings.len() > MAX_ENGRAVINGS_PER_LEVEL {
        return Err("campaign map engraving count exceeds limit".to_owned());
    }
    for engraving in &actual.engravings {
        if engraving.pos.x <= 0
            || engraving.pos.x >= CAMPAIGN_MAP_WIDTH - 1
            || engraving.pos.y <= 0
            || engraving.pos.y >= CAMPAIGN_MAP_HEIGHT - 1
            || engraving.text.len() > MAX_PERSISTED_TEXT_BYTES
            || engraving.text.chars().count() > MAX_ENGRAVING_TEXT_CHARS
            || engraving.text.chars().any(char::is_control)
        {
            return Err("campaign map contains invalid engraving".to_owned());
        }
    }

    // 1. 외곽 테두리는 모두 Wall이어야 한다.
    for x in 0..CAMPAIGN_MAP_WIDTH {
        for y in [0, CAMPAIGN_MAP_HEIGHT - 1] {
            if actual.map.tile(Pos { x, y }).ok() != Some(TileKind::Wall) {
                return Err("campaign generator/map mismatch: perimeter breach detected".to_owned());
            }
        }
    }
    for y in 0..CAMPAIGN_MAP_HEIGHT {
        for x in [0, CAMPAIGN_MAP_WIDTH - 1] {
            if actual.map.tile(Pos { x, y }).ok() != Some(TileKind::Wall) {
                return Err("campaign generator/map mismatch: perimeter breach detected".to_owned());
            }
        }
    }

    // 2. 내부 타일 전이 규칙(Wall->Floor 굴착 허용, 그 외 변조 차단)을 검증한다.
    for y in 1..CAMPAIGN_MAP_HEIGHT - 1 {
        for x in 1..CAMPAIGN_MAP_WIDTH - 1 {
            let p = Pos { x, y };
            let exp_tile = expected.map.tile(p).map_err(|e| e.to_string())?;
            let act_tile = actual.map.tile(p).map_err(|e| e.to_string())?;
            let allowed = match (exp_tile, act_tile) {
                (TileKind::Wall, TileKind::Wall | TileKind::Floor) => true,
                (TileKind::Floor, TileKind::Floor) => true,
                (
                    TileKind::Door(DoorState::Closed),
                    TileKind::Door(DoorState::Closed | DoorState::Open),
                ) => true,
                (
                    TileKind::Door(DoorState::Open),
                    TileKind::Door(DoorState::Open | DoorState::Closed),
                ) => true,
                (
                    TileKind::Door(DoorState::Locked),
                    TileKind::Door(DoorState::Locked | DoorState::Open | DoorState::Closed),
                ) => true,
                (
                    TileKind::HiddenDoor,
                    TileKind::HiddenDoor | TileKind::Door(DoorState::Closed | DoorState::Open),
                ) => true,
                (TileKind::HiddenTrap(k), TileKind::HiddenTrap(k2) | TileKind::Trap(k2))
                    if k == k2 =>
                {
                    true
                }
                (TileKind::Trap(k), TileKind::Trap(k2)) if k == k2 => true,
                (TileKind::StairsUp, TileKind::StairsUp)
                | (TileKind::StairsDown, TileKind::StairsDown)
                | (TileKind::Shop, TileKind::Shop)
                | (TileKind::Altar, TileKind::Altar)
                | (TileKind::Fountain, TileKind::Fountain | TileKind::DryFountain)
                | (TileKind::DryFountain, TileKind::DryFountain)
                | (TileKind::Sink, TileKind::Sink)
                | (TileKind::Water, TileKind::Water)
                | (TileKind::Lava, TileKind::Lava) => true,
                _ => false,
            };
            if !allowed {
                return Err(format!(
                    "campaign generator/map mismatch at ({}, {})",
                    p.x, p.y
                ));
            }
        }
    }

    // 3. 필수 구조 타일(계단, 상점, 제단)의 정확한 좌표/개수 불변성을 검증한다.
    let collect_tiles = |map: &GameMap| {
        let mut stairs_up = Vec::new();
        let mut stairs_down = Vec::new();
        let mut shops = Vec::new();
        let mut altars = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                let p = Pos { x, y };
                match map.tile(p) {
                    Ok(TileKind::StairsUp) => stairs_up.push(p),
                    Ok(TileKind::StairsDown) => stairs_down.push(p),
                    Ok(TileKind::Shop) => shops.push(p),
                    Ok(TileKind::Altar) => altars.push(p),
                    _ => {}
                }
            }
        }
        (stairs_up, stairs_down, shops, altars)
    };

    let (exp_up, exp_down, exp_shops, exp_altars) = collect_tiles(&expected.map);
    let (act_up, act_down, act_shops, act_altars) = collect_tiles(&actual.map);
    if act_up != exp_up || act_down != exp_down {
        return Err("campaign generator/map mismatch: stairs tampering detected".to_owned());
    }
    if act_shops != exp_shops {
        return Err("campaign generator/map mismatch: shop tampering detected".to_owned());
    }
    if act_altars != exp_altars {
        return Err("campaign generator/map mismatch: altar tampering detected".to_owned());
    }

    // 4. 위 계단에서 아래 계단/상점/제단까지의 구조적 BFS 연결성을 확인한다.
    let Some(start) = act_up.first().copied() else {
        return Err("campaign generator/map mismatch: missing stairs up".to_owned());
    };
    let mut required_targets = Vec::new();
    required_targets.extend(act_down);
    required_targets.extend(act_shops);
    required_targets.extend(act_altars);
    if !all_targets_reachable(actual.map.tiles(), start, &required_targets) {
        return Err(
            "campaign generator/map mismatch: critical structure is disconnected".to_owned(),
        );
    }

    Ok(())
}
