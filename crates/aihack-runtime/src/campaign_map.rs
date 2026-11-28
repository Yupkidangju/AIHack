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

pub(crate) fn p12_level_ids() -> Vec<LevelId> {
    let mut out = Vec::with_capacity(8);
    for depth in 1..=6 {
        out.push(LevelId::main(depth));
    }
    for depth in 1..=2 {
        out.push(LevelId::new(BranchId::Mines, depth));
    }
    out
}

pub(crate) fn p13_branch_level_ids() -> Vec<LevelId> {
    let p12 = p12_level_ids();
    level_ids()
        .into_iter()
        .filter(|id| !p12.contains(id))
        .collect()
}

pub(crate) fn main6_gehennom_stairs_pos(seed: u64) -> Result<Pos, ContentError> {
    let (_, centers) = generate(seed, LevelId::main(6))?;
    Ok(centers[3])
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
        BranchId::Planes => 0x4b72_9d41_e310_829f,
    };
    seed ^ (id.depth as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ branch_salt
}

/// Combat RNG와 독립인 Generator v3 (`80x21`).
/// 가변 6개 방, 스패닝 트리 + 루프 통로, 어둠/조명, 특수방, 문·비밀문·함정을 배치하고 BFS 도달성을 보장한다.
pub(crate) fn generate(seed: u64, id: LevelId) -> Result<(GameLevel, [Pos; 6]), ContentError> {
    match id.branch {
        BranchId::Sokoban => return generate_sokoban(id),
        BranchId::Quest => return generate_quest(seed, id),
        BranchId::Gehennom => return generate_gehennom(seed, id),
        BranchId::Planes => return generate_planes(seed, id),
        BranchId::Main | BranchId::Mines => {}
    }
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
    let graph = BranchGraph::campaign_v3();
    let has_down = graph.descend_target(id).is_some()
        || graph.has_branch_entry(id, aihack_core::domain::level::BranchEntryKind::StairsDown);
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

fn branch_tile_idx(pos: Pos) -> usize {
    (pos.y as usize * CAMPAIGN_MAP_WIDTH as usize) + pos.x as usize
}

fn carve_floor_rect(
    grid: &mut [TileKind],
    lit_tiles: &mut [bool],
    x1: i16,
    y1: i16,
    x2: i16,
    y2: i16,
    lit: bool,
) {
    for y in y1..=y2 {
        for x in x1..=x2 {
            let p = Pos { x, y };
            let idx = branch_tile_idx(p);
            grid[idx] = TileKind::Floor;
            if lit {
                lit_tiles[idx] = true;
            }
        }
    }
    if lit {
        for y in (y1 - 1).max(0)..=(y2 + 1).min(CAMPAIGN_MAP_HEIGHT - 1) {
            for x in (x1 - 1).max(0)..=(x2 + 1).min(CAMPAIGN_MAP_WIDTH - 1) {
                lit_tiles[branch_tile_idx(Pos { x, y })] = true;
            }
        }
    }
}

fn carve_l_corridor(grid: &mut [TileKind], lit_tiles: &mut [bool], a: Pos, b: Pos, lit: bool) {
    for x in a.x.min(b.x)..=a.x.max(b.x) {
        let idx = branch_tile_idx(Pos { x, y: a.y });
        if grid[idx] == TileKind::Wall {
            grid[idx] = TileKind::Floor;
        }
        if lit {
            lit_tiles[idx] = true;
        }
    }
    for y in a.y.min(b.y)..=a.y.max(b.y) {
        let idx = branch_tile_idx(Pos { x: b.x, y });
        if grid[idx] == TileKind::Wall {
            grid[idx] = TileKind::Floor;
        }
        if lit {
            lit_tiles[idx] = true;
        }
    }
}

fn build_branch_level(
    id: LevelId,
    grid: Vec<TileKind>,
    lit_tiles: Vec<bool>,
    special_room: Option<SpecialRoomKind>,
    special_level_id: Option<String>,
    centers: [Pos; 6],
) -> Result<(GameLevel, [Pos; 6]), ContentError> {
    let mut overrides = Vec::with_capacity(grid.len());
    for y in 0..CAMPAIGN_MAP_HEIGHT {
        for x in 0..CAMPAIGN_MAP_WIDTH {
            let p = Pos { x, y };
            overrides.push((p, grid[branch_tile_idx(p)]));
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
            special_level_id,
        },
        centers,
    ))
}

fn generate_sokoban(id: LevelId) -> Result<(GameLevel, [Pos; 6]), ContentError> {
    let mut grid = vec![TileKind::Wall; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let mut lit_tiles = vec![false; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let centers = [
        Pos { x: 8, y: 10 },
        Pos { x: 24, y: 5 },
        Pos { x: 52, y: 5 },
        Pos { x: 70, y: 10 },
        Pos { x: 52, y: 15 },
        Pos { x: 24, y: 15 },
    ];
    carve_floor_rect(&mut grid, &mut lit_tiles, 4, 6, 14, 14, true);
    carve_floor_rect(&mut grid, &mut lit_tiles, 20, 3, 28, 7, true);
    carve_floor_rect(&mut grid, &mut lit_tiles, 48, 3, 56, 7, true);
    carve_floor_rect(&mut grid, &mut lit_tiles, 66, 6, 75, 14, true);
    carve_floor_rect(&mut grid, &mut lit_tiles, 48, 13, 56, 17, true);
    carve_floor_rect(&mut grid, &mut lit_tiles, 20, 13, 28, 17, true);
    carve_floor_rect(&mut grid, &mut lit_tiles, 15, 9, 65, 11, true);
    carve_floor_rect(&mut grid, &mut lit_tiles, 34, 8, 60, 12, true);
    carve_l_corridor(
        &mut grid,
        &mut lit_tiles,
        centers[1],
        Pos { x: 24, y: 9 },
        true,
    );
    carve_l_corridor(
        &mut grid,
        &mut lit_tiles,
        centers[2],
        Pos { x: 52, y: 9 },
        true,
    );
    carve_l_corridor(
        &mut grid,
        &mut lit_tiles,
        centers[4],
        Pos { x: 52, y: 11 },
        true,
    );
    carve_l_corridor(
        &mut grid,
        &mut lit_tiles,
        centers[5],
        Pos { x: 24, y: 11 },
        true,
    );

    grid[branch_tile_idx(centers[0])] = TileKind::StairsUp;
    if id.depth < 4 {
        grid[branch_tile_idx(centers[3])] = TileKind::StairsDown;
    }

    // Vertical barrier at x = 42 forces Sokoban boulder/pit puzzle completion before reaching East exit/prize.
    for y in 8..=12_i16 {
        grid[branch_tile_idx(Pos { x: 42, y })] = TileKind::Wall;
    }

    // Solvable boulder/pit puzzle lanes aligned directly against the x = 42 pit barrier.
    let puzzle_pairs: &[(Pos, Pos)] = match id.depth {
        1 => &[
            (Pos { x: 37, y: 9 }, Pos { x: 42, y: 9 }),
            (Pos { x: 37, y: 11 }, Pos { x: 42, y: 11 }),
        ],
        2 => &[
            (Pos { x: 37, y: 8 }, Pos { x: 42, y: 8 }),
            (Pos { x: 37, y: 11 }, Pos { x: 42, y: 11 }),
        ],
        3 => &[
            (Pos { x: 36, y: 8 }, Pos { x: 42, y: 8 }),
            (Pos { x: 38, y: 10 }, Pos { x: 42, y: 10 }),
            (Pos { x: 36, y: 12 }, Pos { x: 42, y: 12 }),
        ],
        _ => &[
            (Pos { x: 36, y: 8 }, Pos { x: 42, y: 8 }),
            (Pos { x: 37, y: 10 }, Pos { x: 42, y: 10 }),
            (Pos { x: 36, y: 12 }, Pos { x: 42, y: 12 }),
        ],
    };
    for &(boulder_pos, pit_pos) in puzzle_pairs {
        grid[branch_tile_idx(boulder_pos)] = TileKind::Boulder;
        grid[branch_tile_idx(pit_pos)] = TileKind::Trap(TrapKind::Pit);
    }
    // Extra spare boulder in side alcove of puzzle hall
    grid[branch_tile_idx(Pos { x: 56, y: 8 })] = TileKind::Boulder;

    build_branch_level(
        id,
        grid,
        lit_tiles,
        None,
        Some(format!("sokoban_{}", id.depth)),
        centers,
    )
}

fn generate_quest(seed: u64, id: LevelId) -> Result<(GameLevel, [Pos; 6]), ContentError> {
    let mut state = seed_for_level(seed, id);
    let mut grid = vec![TileKind::Wall; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let mut lit_tiles = vec![false; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let jitter_x = (next_rng(&mut state) % 3) as i16 - 1;
    let centers = [
        Pos { x: 10, y: 10 },
        Pos {
            x: 30 + jitter_x,
            y: 5,
        },
        Pos {
            x: 52 + jitter_x,
            y: 5,
        },
        Pos { x: 70, y: 10 },
        Pos {
            x: 52 + jitter_x,
            y: 15,
        },
        Pos {
            x: 30 + jitter_x,
            y: 15,
        },
    ];
    for c in centers {
        carve_floor_rect(
            &mut grid,
            &mut lit_tiles,
            c.x - 5,
            c.y - 3,
            c.x + 5,
            c.y + 3,
            true,
        );
    }
    carve_l_corridor(&mut grid, &mut lit_tiles, centers[0], centers[1], true);
    carve_l_corridor(&mut grid, &mut lit_tiles, centers[1], centers[2], true);
    carve_l_corridor(&mut grid, &mut lit_tiles, centers[2], centers[3], true);
    carve_l_corridor(&mut grid, &mut lit_tiles, centers[0], centers[5], true);
    carve_l_corridor(&mut grid, &mut lit_tiles, centers[5], centers[4], true);
    carve_l_corridor(&mut grid, &mut lit_tiles, centers[4], centers[3], true);

    grid[branch_tile_idx(centers[0])] = TileKind::StairsUp;
    let special_room = if id.depth < 3 {
        grid[branch_tile_idx(centers[3])] = TileKind::StairsDown;
        Some(SpecialRoomKind::Barracks)
    } else {
        Some(SpecialRoomKind::Vault)
    };
    build_branch_level(
        id,
        grid,
        lit_tiles,
        special_room,
        Some(format!("quest_{}", id.depth)),
        centers,
    )
}

fn generate_gehennom(seed: u64, id: LevelId) -> Result<(GameLevel, [Pos; 6]), ContentError> {
    let mut state = seed_for_level(seed, id);
    let mut grid = vec![TileKind::Wall; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let mut lit_tiles = vec![false; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let offset = (next_rng(&mut state) % 2) as i16;
    let centers = [
        Pos { x: 9, y: 10 },
        Pos {
            x: 28 + offset,
            y: 5,
        },
        Pos {
            x: 50 + offset,
            y: 5,
        },
        Pos { x: 70, y: 10 },
        Pos {
            x: 50 + offset,
            y: 15,
        },
        Pos {
            x: 28 + offset,
            y: 15,
        },
    ];
    for (idx, c) in centers.iter().copied().enumerate() {
        let (y1, y2) = match idx {
            1 | 2 => (c.y - 3, c.y + 2),
            4 | 5 => (c.y - 2, c.y + 3),
            _ => (c.y - 3, c.y + 3),
        };
        carve_floor_rect(&mut grid, &mut lit_tiles, c.x - 5, y1, c.x + 5, y2, true);
    }
    carve_floor_rect(&mut grid, &mut lit_tiles, 14, 9, 65, 11, true);
    for c in [centers[1], centers[2]] {
        carve_l_corridor(&mut grid, &mut lit_tiles, c, Pos { x: c.x, y: 9 }, true);
        grid[branch_tile_idx(Pos { x: c.x, y: 8 })] = TileKind::Door(DoorState::Closed);
    }
    for c in [centers[4], centers[5]] {
        carve_l_corridor(&mut grid, &mut lit_tiles, c, Pos { x: c.x, y: 11 }, true);
    }
    carve_l_corridor(&mut grid, &mut lit_tiles, centers[1], centers[2], true);
    carve_l_corridor(&mut grid, &mut lit_tiles, centers[5], centers[4], true);

    if id.depth == 3 {
        // Wizard's Tower moat border (keeps side floor paths open around the edges)
        for x in [46_i16, 47, 54, 55] {
            for y in [3_i16, 17] {
                grid[branch_tile_idx(Pos { x, y })] = TileKind::Water;
            }
        }
    } else if id.depth == 4 {
        // Moloch's Sanctum lava border around central bridge
        for x in 60..=76 {
            for y in [5_i16, 15] {
                grid[branch_tile_idx(Pos { x, y })] = TileKind::Lava;
            }
        }
    }

    grid[branch_tile_idx(centers[0])] = TileKind::StairsUp;
    if id.depth < 4 {
        grid[branch_tile_idx(centers[3])] = TileKind::StairsDown;
    }

    let (special_room, special_level_id) = match id.depth {
        1 => (None, Some("valley_of_the_dead".to_owned())),
        2 => (
            Some(SpecialRoomKind::Barracks),
            Some("orcus_town".to_owned()),
        ),
        3 => (None, Some("wizards_tower".to_owned())),
        _ => (
            Some(SpecialRoomKind::Vault),
            Some("molochs_sanctum".to_owned()),
        ),
    };

    build_branch_level(id, grid, lit_tiles, special_room, special_level_id, centers)
}

fn generate_planes(seed: u64, id: LevelId) -> Result<(GameLevel, [Pos; 6]), ContentError> {
    let _ = seed_for_level(seed, id);
    let mut grid = vec![TileKind::Wall; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];
    let mut lit_tiles = vec![false; (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize];

    if id.depth == 5 {
        // Astral Plane: entry west hall leading to 3 distinct High Altar chapels on east
        let centers = [
            Pos { x: 10, y: 10 },
            Pos { x: 60, y: 4 },
            Pos { x: 60, y: 10 },
            Pos { x: 60, y: 16 },
            Pos { x: 38, y: 10 },
            Pos { x: 24, y: 10 },
        ];
        carve_floor_rect(&mut grid, &mut lit_tiles, 5, 6, 46, 14, true);
        carve_floor_rect(&mut grid, &mut lit_tiles, 46, 9, 68, 11, true);
        carve_floor_rect(&mut grid, &mut lit_tiles, 52, 2, 68, 6, true);
        carve_floor_rect(&mut grid, &mut lit_tiles, 52, 8, 68, 12, true);
        carve_floor_rect(&mut grid, &mut lit_tiles, 52, 14, 68, 18, true);
        carve_floor_rect(&mut grid, &mut lit_tiles, 48, 3, 52, 17, true);

        grid[branch_tile_idx(centers[0])] = TileKind::StairsUp;
        grid[branch_tile_idx(centers[1])] = TileKind::Altar;
        grid[branch_tile_idx(centers[2])] = TileKind::Altar;
        grid[branch_tile_idx(centers[3])] = TileKind::Altar;
        return build_branch_level(
            id,
            grid,
            lit_tiles,
            Some(SpecialRoomKind::Temple),
            Some("astral_plane".to_owned()),
            centers,
        );
    }

    let centers = [
        Pos { x: 10, y: 10 },
        Pos { x: 28, y: 5 },
        Pos { x: 50, y: 5 },
        Pos { x: 70, y: 10 },
        Pos { x: 50, y: 15 },
        Pos { x: 28, y: 15 },
    ];
    match id.depth {
        1 => {
            // Plane of Earth: mostly Wall cavern with carved winding tunnels
            for c in centers {
                carve_floor_rect(
                    &mut grid,
                    &mut lit_tiles,
                    c.x - 4,
                    c.y - 2,
                    c.x + 4,
                    c.y + 2,
                    true,
                );
            }
            carve_l_corridor(&mut grid, &mut lit_tiles, centers[0], centers[1], true);
            carve_l_corridor(&mut grid, &mut lit_tiles, centers[1], centers[2], true);
            carve_l_corridor(&mut grid, &mut lit_tiles, centers[2], centers[3], true);
            carve_l_corridor(&mut grid, &mut lit_tiles, centers[0], centers[5], true);
            carve_l_corridor(&mut grid, &mut lit_tiles, centers[5], centers[4], true);
            carve_l_corridor(&mut grid, &mut lit_tiles, centers[4], centers[3], true);
        }
        2 => {
            // Plane of Air: expansive sky arena
            carve_floor_rect(&mut grid, &mut lit_tiles, 3, 2, 76, 18, true);
            for wall_x in [20_i16, 40, 58] {
                for wall_y in [4_i16, 16] {
                    grid[branch_tile_idx(Pos {
                        x: wall_x,
                        y: wall_y,
                    })] = TileKind::Wall;
                }
            }
        }
        3 | 4 => {
            // Plane of Fire (3) / Plane of Water (4): hazard fill with solid floor islands and bridges
            let hazard = if id.depth == 3 {
                TileKind::Lava
            } else {
                TileKind::Water
            };
            for y in 2..=18 {
                for x in 2..=77 {
                    let idx = branch_tile_idx(Pos { x, y });
                    grid[idx] = hazard;
                    lit_tiles[idx] = true;
                }
            }
            for c in centers {
                carve_floor_rect(
                    &mut grid,
                    &mut lit_tiles,
                    c.x - 4,
                    c.y - 2,
                    c.x + 4,
                    c.y + 2,
                    true,
                );
            }
            carve_floor_rect(&mut grid, &mut lit_tiles, 10, 9, 70, 11, true);
            carve_floor_rect(&mut grid, &mut lit_tiles, 27, 5, 29, 15, true);
            carve_floor_rect(&mut grid, &mut lit_tiles, 49, 5, 51, 15, true);
        }
        _ => {}
    }

    grid[branch_tile_idx(centers[0])] = TileKind::StairsUp;
    grid[branch_tile_idx(centers[3])] = TileKind::MagicPortal;

    let special_level_id = match id.depth {
        1 => Some("plane_of_earth".to_owned()),
        2 => Some("plane_of_air".to_owned()),
        3 => Some("plane_of_fire".to_owned()),
        _ => Some("plane_of_water".to_owned()),
    };

    build_branch_level(id, grid, lit_tiles, None, special_level_id, centers)
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
    validate_persisted_map_inner(seed, actual, false)
}

/// P8–P12 8층 캠페인 세이브(`Main 1..6`, `Mines 1..2`)의 지도를 검증한다(`Main 6` 게헤놈 하향 계단 부재 허용).
pub(crate) fn validate_persisted_p12_map(seed: u64, actual: &GameLevel) -> Result<(), String> {
    validate_persisted_map_inner(seed, actual, true)
}

fn validate_persisted_map_inner(
    seed: u64,
    actual: &GameLevel,
    allow_p12_main6_floor_stairs: bool,
) -> Result<(), String> {
    let id = actual.id;
    let p12_main6_compat = allow_p12_main6_floor_stairs && id == LevelId::main(6);
    let (expected, _) = generate(seed, id).map_err(|error| error.to_string())?;
    if actual.map.width != CAMPAIGN_MAP_WIDTH || actual.map.height != CAMPAIGN_MAP_HEIGHT {
        return Err("campaign generator/map dimension mismatch".to_owned());
    }
    let expected_tile_count = (CAMPAIGN_MAP_WIDTH * CAMPAIGN_MAP_HEIGHT) as usize;
    if actual.map.tiles().len() != expected_tile_count {
        return Err("campaign generator/map tile count mismatch".to_owned());
    }
    if expected.special_level_id.is_some() && actual.special_level_id != expected.special_level_id {
        return Err("campaign generator/map mismatch: special_level_id mismatch".to_owned());
    }
    if let Some(special_id) = &actual.special_level_id {
        if special_id.len() > MAX_PERSISTED_TEXT_BYTES || special_id.chars().any(char::is_control) {
            return Err("campaign map contains invalid special_level_id".to_owned());
        }
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

    // 2. 내부 타일 전이 규칙(Wall->Floor 굴착 허용, Sokoban/Planes 2..5 제한 및 볼더-함정 보존)을 검증한다.
    let diggable =
        id.branch != BranchId::Sokoban && !(id.branch == BranchId::Planes && id.depth > 1);
    let mut exp_boulders = 0_usize;
    let mut exp_pits = 0_usize;
    let mut act_boulders = 0_usize;
    let mut act_pits = 0_usize;
    for y in 1..CAMPAIGN_MAP_HEIGHT - 1 {
        for x in 1..CAMPAIGN_MAP_WIDTH - 1 {
            let p = Pos { x, y };
            let exp_tile = expected.map.tile(p).map_err(|e| e.to_string())?;
            let act_tile = actual.map.tile(p).map_err(|e| e.to_string())?;
            if exp_tile == TileKind::Boulder {
                exp_boulders += 1;
            }
            if matches!(
                exp_tile,
                TileKind::Trap(TrapKind::Pit) | TileKind::HiddenTrap(TrapKind::Pit)
            ) {
                exp_pits += 1;
            }
            if act_tile == TileKind::Boulder {
                act_boulders += 1;
            }
            if matches!(
                act_tile,
                TileKind::Trap(TrapKind::Pit) | TileKind::HiddenTrap(TrapKind::Pit)
            ) {
                act_pits += 1;
            }
            let allowed = match (exp_tile, act_tile) {
                (TileKind::Wall, TileKind::Wall) => true,
                (TileKind::Wall, TileKind::Floor) => diggable,
                (TileKind::Floor, TileKind::Floor) => true,
                (TileKind::Floor, TileKind::Boulder) if id.branch == BranchId::Sokoban => true,
                (TileKind::Boulder, TileKind::Boulder | TileKind::Floor)
                    if id.branch == BranchId::Sokoban =>
                {
                    true
                }
                (TileKind::Trap(TrapKind::Pit), TileKind::Floor | TileKind::Boulder)
                    if id.branch == BranchId::Sokoban =>
                {
                    true
                }
                (TileKind::HiddenTrap(TrapKind::Pit), TileKind::Floor | TileKind::Boulder)
                    if id.branch == BranchId::Sokoban =>
                {
                    true
                }
                (
                    TileKind::Door(DoorState::Closed),
                    TileKind::Door(DoorState::Closed | DoorState::Open | DoorState::Locked),
                ) => true,
                (
                    TileKind::Door(DoorState::Open),
                    TileKind::Door(DoorState::Open | DoorState::Closed | DoorState::Locked),
                ) => true,
                (
                    TileKind::Door(DoorState::Locked),
                    TileKind::Door(DoorState::Locked | DoorState::Open | DoorState::Closed),
                ) => true,
                (
                    TileKind::HiddenDoor,
                    TileKind::HiddenDoor
                    | TileKind::Door(DoorState::Closed | DoorState::Open | DoorState::Locked),
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
                | (TileKind::MagicPortal, TileKind::MagicPortal)
                | (TileKind::Fountain, TileKind::Fountain | TileKind::DryFountain)
                | (TileKind::DryFountain, TileKind::DryFountain)
                | (TileKind::Sink, TileKind::Sink)
                | (TileKind::Water, TileKind::Water)
                | (TileKind::Lava, TileKind::Lava) => true,
                (TileKind::StairsDown, TileKind::Floor) if p12_main6_compat => true,
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
    if id.branch == BranchId::Sokoban
        && (act_boulders > exp_boulders
            || act_pits > exp_pits
            || (exp_boulders - act_boulders) != (exp_pits - act_pits))
    {
        return Err("campaign generator/map mismatch: sokoban conservation violated".to_owned());
    }

    // 3. 필수 구조 타일(계단, 상점, 제단, 포털)의 정확한 좌표/개수 불변성을 검증한다.
    let collect_tiles = |map: &GameMap| {
        let mut stairs_up = Vec::new();
        let mut stairs_down = Vec::new();
        let mut shops = Vec::new();
        let mut altars = Vec::new();
        let mut portals = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                let p = Pos { x, y };
                match map.tile(p) {
                    Ok(TileKind::StairsUp) => stairs_up.push(p),
                    Ok(TileKind::StairsDown) => stairs_down.push(p),
                    Ok(TileKind::Shop) => shops.push(p),
                    Ok(TileKind::Altar) => altars.push(p),
                    Ok(TileKind::MagicPortal) => portals.push(p),
                    _ => {}
                }
            }
        }
        (stairs_up, stairs_down, shops, altars, portals)
    };

    let (exp_up, exp_down, exp_shops, exp_altars, exp_portals) = collect_tiles(&expected.map);
    let (act_up, act_down, act_shops, act_altars, act_portals) = collect_tiles(&actual.map);
    let down_matches =
        act_down == exp_down || (p12_main6_compat && act_down.is_empty() && exp_down.len() == 1);
    if act_up != exp_up || !down_matches {
        return Err("campaign generator/map mismatch: stairs tampering detected".to_owned());
    }
    if act_shops != exp_shops {
        return Err("campaign generator/map mismatch: shop tampering detected".to_owned());
    }
    if act_altars != exp_altars {
        return Err("campaign generator/map mismatch: altar tampering detected".to_owned());
    }
    if act_portals != exp_portals {
        return Err("campaign generator/map mismatch: portal tampering detected".to_owned());
    }

    // 4. 위 계단에서 아래 계단/상점/제단/포털까지의 구조적 BFS 연결성을 확인한다.
    let Some(start) = act_up.first().copied() else {
        return Err("campaign generator/map mismatch: missing stairs up".to_owned());
    };
    let mut required_targets = Vec::new();
    required_targets.extend(act_down);
    required_targets.extend(act_shops);
    required_targets.extend(act_altars);
    required_targets.extend(act_portals);
    if !all_targets_reachable(actual.map.tiles(), start, &required_targets) {
        return Err(
            "campaign generator/map mismatch: critical structure is disconnected".to_owned(),
        );
    }

    Ok(())
}
