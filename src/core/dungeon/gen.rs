// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
use crate::core::dungeon::mkroom::{MkRoom, RoomManager};
use crate::core::dungeon::rect::{NhRect, RectManager, MAXNROFROOMS};
use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::{Grid, COLNO, ROWNO};
use crate::util::rng::NetHackRng;
use legion::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelType {
    Ordinary,
    Bigroom,
    Mines,
    Maze,
    Sokoban,
    Minetown,
}

impl LevelType {
    pub fn for_depth(id: crate::core::dungeon::LevelID, rng: &mut NetHackRng) -> Self {
        use crate::core::dungeon::DungeonBranch;
        match id.branch {
            DungeonBranch::Mines => {
                if id.depth == 7 {
                    return LevelType::Minetown;
                }
                return LevelType::Mines;
            }
            DungeonBranch::Sokoban => {
                return LevelType::Sokoban;
            }
            _ => {}
        }

        if id.depth <= 1 {
            return LevelType::Ordinary;
        }

        let depth = id.depth;
        // NetHack 3.6.7: Bigroom has a chance after depth 10
        if depth > 10 && rng.rn2(20) == 0 {
            return LevelType::Bigroom;
        }
        if depth > 3 && depth < 20 && rng.rn2(15) == 0 {
            // Main Dungeon mines layout flavor
            return LevelType::Mines;
        }
        if depth > 15 && rng.rn2(10) == 0 {
            return LevelType::Maze;
        }
        LevelType::Ordinary
    }
}

///
struct LevelGen<'a> {
    grid: Grid,
    rects: RectManager,
    rooms: RoomManager,
    smeq: Vec<usize>,
    rng: &'a mut NetHackRng,
    id: crate::core::dungeon::LevelID,
}

impl<'a> LevelGen<'a> {
    fn new(rng: &'a mut NetHackRng, id: crate::core::dungeon::LevelID) -> Self {
        Self {
            grid: Grid::new(),
            rects: RectManager::new(),
            rooms: RoomManager::new(),
            smeq: (0..MAXNROFROOMS).collect(),
            rng,
            id,
        }
    }

    ///
    fn makerooms(&mut self) {
        let mut n_rooms = 0;
        //
        for _ in 0..MAXNROFROOMS {
            if n_rooms >= MAXNROFROOMS {
                break;
            }

            // Borrow checker: Copy rect to release borrow on self.rects
            let rect_opt = self.rects.rnd_rect(self.rng).copied();

            if let Some(rect) = rect_opt {
                //
                let width = rect.hx - rect.lx + 1;
                let height = rect.hy - rect.ly + 1;

                if width < 5 || height < 4 {
                    continue;
                }

                let r_w = self.rng.rnz(4, width.min(15) as i32) as usize;
                let r_h = self.rng.rnz(3, height.min(10) as i32) as usize;

                let rx = rect.lx + self.rng.rn2((width - r_w + 1) as i32) as usize;
                let ry = rect.ly + self.rng.rn2((height - r_h + 1) as i32) as usize;

                let hx = rx + r_w - 1;
                let hy = ry + r_h - 1;

                let mut room = MkRoom::new(rx, ry, hx, hy);
                // [v2.0.0
                //
                //
                let depth = self.id.depth;
                room.rlit = self.rng.rn2(1 + depth.abs()) < 11;

                //
                let used_rect = NhRect {
                    lx: rx,
                    ly: ry,
                    hx,
                    hy,
                };
                self.rects.split_rects(rect, &used_rect);

                //
                let room_id = (self.rooms.rooms.len() as u8) + 1;
                self.apply_room(&room, room_id);

                //
                if self.rng.rn2(10) == 0 {
                    room.rtype = crate::core::dungeon::mkroom::RoomType::ShopBase;
                } else if self.rng.rn2(20) == 0 {
                    room.rtype = crate::core::dungeon::mkroom::RoomType::Zoo;
                } else if self.rng.rn2(20) == 0 {
                    room.rtype = crate::core::dungeon::mkroom::RoomType::Morgue;
                }

                //
                if self.rng.rn2(10) == 0 && r_w > 6 && r_h > 5 {
                    //
                    let sw = self.rng.rnz(3, (r_w - 2) as i32) as usize;
                    let sh = self.rng.rnz(2, (r_h - 2) as i32) as usize;
                    let sx = rx + 1 + self.rng.rn2((r_w - sw - 1) as i32) as usize;
                    let sy = ry + 1 + self.rng.rn2((r_h - sh - 1) as i32) as usize;

                    let mut sub = MkRoom::new(sx, sy, sx + sw - 1, sy + sh - 1);
                    sub.parent = Some(self.rooms.rooms.len());
                    self.rooms.rooms.push(sub);
                }

                self.rooms.rooms.push(room);
                n_rooms += 1;
            }
        }
    }

    ///
    ///
    fn make_bigroom(
        &mut self,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        id: crate::core::dungeon::LevelID,
    ) {
        use crate::core::entity::spawn::{Spawner, NO_MM_FLAGS};

        let mut room = MkRoom::new(1, 1, COLNO - 2, ROWNO - 2);
        room.rlit = true;
        self.apply_room(&room, 1);
        self.rooms.rooms.push(room.clone());

        //
        let monster_count = 10 + self.rng.rn2(10) as usize;
        for _ in 0..monster_count {
            let (sx, sy) = room.somexy(self.rng);
            Spawner::makemon(
                None,
                sx,
                sy,
                NO_MM_FLAGS,
                world,
                &self.grid,
                templates,
                items,
                self.rng,
                id,
            );
        }

        //
        let item_count = 5 + self.rng.rn2(5) as usize;
        for _ in 0..item_count {
            let (sx, sy) = room.somexy(self.rng);
            Spawner::mkobj_of_class(
                crate::core::entity::object::ItemClass::Gem,
                items,
                world,
                self.rng,
            )
            .map(|e| {
                if let Some(mut entry) = world.entry(e) {
                    entry.add_component(crate::core::entity::Position {
                        x: sx as i32,
                        y: sy as i32,
                    });
                    entry.add_component(crate::core::entity::Level(id));
                }
            });
        }

        //
        for _ in 0..self.rng.rn2(2) + 1 {
            let (fx, fy) = room.somexy(self.rng);
            if let Some(tile) = self.grid.get_tile_mut(fx, fy) {
                if tile.typ == TileType::Room {
                    tile.typ = TileType::Fountain;
                }
            }
        }
    }

    ///
    fn make_mines(
        &mut self,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        id: crate::core::dungeon::LevelID,
    ) {
        use crate::core::entity::spawn::{Spawner, NO_MM_FLAGS};

        let n_blobs = 15 + self.rng.rn2(10);
        for _ in 0..n_blobs {
            let mut x = (self.rng.rn2((COLNO - 10) as i32) + 5) as usize;
            let mut y = (self.rng.rn2((ROWNO - 8) as i32) + 4) as usize;
            let size = 5 + self.rng.rn2(15);

            for _ in 0..size {
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        let nx = (x as i32 + dx) as usize;
                        let ny = (y as i32 + dy) as usize;
                        if nx > 0 && nx < COLNO - 1 && ny > 0 && ny < ROWNO - 1 {
                            self.grid.locations[nx][ny].typ = TileType::Corr;
                            // Mines are dark by default (no LIT flag added)
                            // Usually rooms have LIT, but Corr doesn't have LIT by default in tile.rs?
                            // Let's explicitly ensure it's unlit or lit based on design.
                            // NetHack mines are dark.
                        }
                    }
                }
                x = (x as i32 + self.rng.rn2(3) - 1) as usize;
                y = (y as i32 + self.rng.rn2(3) - 1) as usize;
                x = x.clamp(2, COLNO - 3);
                y = y.clamp(2, ROWNO - 3);
            }
        }

        //
        let mut start_room =
            MkRoom::new(COLNO / 2 - 2, ROWNO / 2 - 2, COLNO / 2 + 2, ROWNO / 2 + 2);
        start_room.rlit = false; // Mines are dark
        self.apply_room(&start_room, 1);
        self.rooms.rooms.push(start_room);

        //
        for _ in 0..20 {
            let x = self.rng.rn2(COLNO as i32) as usize;
            let y = self.rng.rn2(ROWNO as i32) as usize;
            if self.grid.locations[x][y].typ == TileType::Corr {
                Spawner::makemon_of_class(
                    if self.rng.rn2(3) == 0 { 'h' } else { 'G' }, // dwarf or gnome
                    x,
                    y,
                    NO_MM_FLAGS,
                    world,
                    &self.grid,
                    templates,
                    items,
                    self.rng,
                    id,
                );
            }
        }

        // Gems, Tools (Pick-axe)
        for _ in 0..10 {
            let x = self.rng.rn2(COLNO as i32) as usize;
            let y = self.rng.rn2(ROWNO as i32) as usize;
            if self.grid.locations[x][y].typ == TileType::Corr {
                if self.rng.rn2(2) == 0 {
                    Spawner::mkobj_of_class(
                        crate::core::entity::object::ItemClass::Gem,
                        items,
                        world,
                        self.rng,
                    )
                    .map(|e| {
                        if let Some(mut entry) = world.entry(e) {
                            entry.add_component(crate::core::entity::Position {
                                x: x as i32,
                                y: y as i32,
                            });
                            entry.add_component(crate::core::entity::Level(id));
                        }
                    });
                } else {
                    Spawner::mksobj_at("pick-axe", x, y, items, world, id);
                }
            }
        }
    }

    ///
    fn make_minetown(
        &mut self,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        id: crate::core::dungeon::LevelID,
    ) {
        use crate::core::entity::spawn::{Spawner, NO_MM_FLAGS};

        //
        for x in 10..COLNO - 10 {
            for y in 5..ROWNO - 5 {
                self.grid.locations[x][y].typ = TileType::Room;
                self.grid.locations[x][y].flags = crate::core::dungeon::tile::TileFlags::LIT;
            }
        }

        //
        //
        let shop_coords = [(15, 7, 22, 11), (25, 7, 32, 11), (35, 7, 42, 11)];
        for (_idx, (lx, ly, hx, hy)) in shop_coords.iter().enumerate() {
            let room = MkRoom::new(*lx, *ly, *hx, *hy);
            let room_id = (self.rooms.rooms.len() as u8) + 1;
            self.apply_room(&room, room_id);
            self.rooms.rooms.push(room);
            //
            self.rooms.mkshop(
                self.rooms.rooms.len() - 1,
                &mut self.grid,
                world,
                items,
                templates,
                self.rng,
                id,
            );
            //
            self.dodoor((*lx + *hx) / 2, *hy, room_id);
        }

        //
        let temple = MkRoom::new(55, 7, 65, 13);
        let temple_id = (self.rooms.rooms.len() as u8) + 1;
        self.apply_room(&temple, temple_id);
        self.rooms.rooms.push(temple);
        self.rooms
            .mktemple(self.rooms.rooms.len() - 1, &mut self.grid, self.rng);
        self.dodoor(55, 10, temple_id);

        //
        let priest_template = templates.iter().find(|t| t.name == "priest").copied();
        let (px, py) = RoomManager::shrine_pos(self.rooms.rooms.last().unwrap(), self.rng);
        if let Some(priest_ent) = Spawner::makemon(
            priest_template,
            px,
            py,
            NO_MM_FLAGS,
            world,
            &self.grid,
            templates,
            items,
            self.rng,
            id,
        ) {
            if let Some(mut entry) = world.entry(priest_ent) {
                if let Ok(m) = entry.get_component_mut::<crate::core::entity::Monster>() {
                    m.hostile = false;
                }
            }
        }

        //
        let guardhouse = MkRoom::new(10, 15, 18, 19);
        let guard_id = (self.rooms.rooms.len() as u8) + 1;
        self.apply_room(&guardhouse, guard_id);
        self.rooms.rooms.push(guardhouse);
        self.dodoor(18, 17, guard_id);

        //
        let watchman_template = templates.iter().find(|t| t.name == "watchman").copied();
        for _ in 0..3 {
            Spawner::makemon(
                watchman_template,
                15,
                17,
                NO_MM_FLAGS,
                world,
                &self.grid,
                templates,
                items,
                self.rng,
                id,
            );
        }

        //
        for _ in 0..100 {
            let rx = self.rng.rn2(COLNO as i32) as usize;
            let ry = self.rng.rn2(ROWNO as i32) as usize;
            if rx < 10 || rx > COLNO - 10 || ry < 5 || ry > ROWNO - 5 {
                continue;
            }
            if self.grid.locations[rx][ry].roomno == 0 {
                if self.rng.rn2(2) == 0 {
                    self.grid.locations[rx][ry].typ = TileType::Stone;
                }
            }
        }
    }

    ///
    ///
    fn make_maze(
        &mut self,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        id: crate::core::dungeon::LevelID,
    ) {
        use crate::core::entity::spawn::{Spawner, NO_MM_FLAGS};

        //
        for x in 0..COLNO {
            for y in 0..ROWNO {
                self.grid.locations[x][y].typ = TileType::Stone;
            }
        }

        let mut stack = Vec::new();
        let start_x = 2;
        let start_y = 2;
        self.grid.locations[start_x][start_y].typ = TileType::Corr;
        stack.push((start_x, start_y));

        while let Some((cx, cy)) = stack.pop() {
            let mut neighbors = Vec::new();
            for (dx, dy) in &[(0, 2), (0, -2), (2, 0), (-2, 0)] {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx > 1 && nx < COLNO as i32 - 2 && ny > 1 && ny < ROWNO as i32 - 2 {
                    if self.grid.locations[nx as usize][ny as usize].typ == TileType::Stone {
                        neighbors.push((nx as usize, ny as usize));
                    }
                }
            }

            if !neighbors.is_empty() {
                stack.push((cx, cy));
                let idx = self.rng.rn2(neighbors.len() as i32) as usize;
                let (nx, ny) = neighbors[idx];

                //
                let mx = (cx + nx) / 2;
                let my = (cy + ny) / 2;
                self.grid.locations[mx][my].typ = TileType::Corr;
                self.grid.locations[nx][ny].typ = TileType::Corr;

                stack.push((nx, ny));
            }
        }

        //
        let start_room = MkRoom::new(start_x, start_y, start_x + 2, start_y + 2);
        self.apply_room(&start_room, 1);
        self.rooms.rooms.push(start_room);
        let end_room = MkRoom::new(COLNO - 5, ROWNO - 5, COLNO - 3, ROWNO - 3);
        self.apply_room(&end_room, 2);
        self.rooms.rooms.push(end_room);

        // [v2.0.0
        let monster_count = 8 + self.rng.rn2(8) as usize;
        for _ in 0..monster_count {
            let x = self.rng.rn2(COLNO as i32) as usize;
            let y = self.rng.rn2(ROWNO as i32) as usize;
            if self.grid.locations[x][y].typ == TileType::Corr
                || self.grid.locations[x][y].typ == TileType::Room
            {
                Spawner::makemon(
                    None,
                    x,
                    y,
                    NO_MM_FLAGS,
                    world,
                    &self.grid,
                    templates,
                    items,
                    self.rng,
                    id,
                );
            }
        }

        //
        let item_count = 3 + self.rng.rn2(5) as usize;
        for _ in 0..item_count {
            let x = self.rng.rn2(COLNO as i32) as usize;
            let y = self.rng.rn2(ROWNO as i32) as usize;
            if self.grid.locations[x][y].typ == TileType::Corr {
                Spawner::mkobj_of_class(
                    crate::core::entity::object::ItemClass::Gem,
                    items,
                    world,
                    self.rng,
                )
                .map(|e| {
                    if let Some(mut entry) = world.entry(e) {
                        entry.add_component(crate::core::entity::Position {
                            x: x as i32,
                            y: y as i32,
                        });
                        entry.add_component(crate::core::entity::Level(id));
                    }
                });
            }
        }
    }
    ///
    fn makecorridors(&mut self) {
        let nroom = self.rooms.rooms.len();
        if nroom <= 1 {
            return;
        }

        //
        self.smeq = (0..nroom).collect();

        //
        for i in 0..nroom.saturating_sub(1) {
            //
            if self.rooms.rooms[i].parent.is_some() || self.rooms.rooms[i + 1].parent.is_some() {
                continue;
            }
            if self.rng.rn2(2) == 0 {
                self.join(i, i + 1, false);
            }
        }

        //
        for i in 0..nroom.saturating_sub(2) {
            if self.smeq[i] != self.smeq[i + 2] {
                self.join(i, i + 2, false);
            }
        }

        //
        //
        for _ in 0..100 {
            //
            let a = self.rng.rn2(nroom as i32) as usize;
            let b = self.rng.rn2(nroom as i32) as usize;
            if a != b && self.smeq[a] != self.smeq[b] {
                self.join(a, b, false);
            }

            //
            let first = self.smeq[0];
            if self.smeq.iter().all(|&id| id == first) {
                break;
            }
        }

        //
        if nroom > 4 {
            let extra = self.rng.rn2(3) + 1;
            for _ in 0..extra {
                let a = self.rng.rn2(nroom as i32) as usize;
                let b = self.rng.rn2(nroom as i32) as usize;
                if a != b {
                    self.join(a, b, true);
                }
            }
        }
    }

    ///
    fn join(&mut self, a: usize, b: usize, _nxcor: bool) {
        if a >= self.rooms.rooms.len() || b >= self.rooms.rooms.len() {
            return;
        }

        //
        if self.smeq[a] == self.smeq[b] {
            return;
        }

        let old_id = self.smeq[b];
        let new_id = self.smeq[a];
        for i in 0..self.smeq.len() {
            if self.smeq[i] == old_id {
                self.smeq[i] = new_id;
            }
        }

        let r1 = self.rooms.rooms[a].clone();
        let r2 = self.rooms.rooms[b].clone();

        //
        let (d1x, d1y) = self.finddpos(&r1, &r2);
        let (d2x, d2y) = self.finddpos(&r2, &r1);

        //
        if self.rng.rn2(2) == 0 {
            //
            self.dig_corridor(d1x, d2x, d1y, true);
            self.dig_corridor(d1y, d2y, d2x, false);
        } else {
            //
            self.dig_corridor(d1y, d2y, d1x, false);
            self.dig_corridor(d1x, d2x, d2y, true);
        }

        //
        self.dodoor(d1x, d1y, (a as u8) + 1);
        self.dodoor(d2x, d2y, (b as u8) + 1);
    }

    ///
    fn finddpos(&mut self, rm: &MkRoom, target: &MkRoom) -> (usize, usize) {
        let mut x: usize;
        let mut y: usize;

        //
        let dx = (target.lx + target.hx) as i32 - (rm.lx + rm.hx) as i32;
        let dy = (target.ly + target.hy) as i32 - (rm.ly + rm.hy) as i32;

        if dx.abs() > dy.abs() {
            //
            x = if dx > 0 { rm.hx } else { rm.lx };
            //
            y = rm.ly + 1;
            if rm.hy > rm.ly + 2 {
                y += self.rng.rn2((rm.hy - rm.ly - 1) as i32) as usize;
            }
        } else {
            //
            y = if dy > 0 { rm.hy } else { rm.ly };
            x = rm.lx + 1;
            if rm.hx > rm.lx + 2 {
                x += self.rng.rn2((rm.hx - rm.lx - 1) as i32) as usize;
            }
        }

        (x, y)
    }

    ///
    fn dodoor(&mut self, x: usize, y: usize, room_id: u8) {
        if x >= COLNO || y >= ROWNO {
            return;
        }

        let tile = &mut self.grid.locations[x][y];
        //
        if tile.roomno == room_id {
            tile.typ = TileType::Door;
            tile.doormas = 0; // Closed
        }
    }

    fn apply_room(&mut self, r: &MkRoom, room_id: u8) {
        for x in r.lx..=r.hx {
            for y in r.ly..=r.hy {
                if x >= COLNO || y >= ROWNO {
                    continue;
                }
                self.grid.locations[x][y].roomno = room_id;

                if x == r.lx || x == r.hx || y == r.ly || y == r.hy {
                    self.create_wall(x, y, r);
                } else {
                    self.grid.locations[x][y].typ = TileType::Room;
                    self.grid.locations[x][y].flags = if r.rlit {
                        crate::core::dungeon::tile::TileFlags::LIT
                    } else {
                        crate::core::dungeon::tile::TileFlags::empty()
                    };
                }
            }
        }
    }

    fn create_wall(&mut self, x: usize, y: usize, r: &MkRoom) {
        let typ = if x == r.lx || x == r.hx {
            TileType::VWall
        } else {
            TileType::HWall
        };
        self.grid.locations[x][y].typ = typ;

        //
        if x == r.lx && y == r.ly {
            self.grid.locations[x][y].typ = TileType::TlCorner;
        }
        if x == r.hx && y == r.ly {
            self.grid.locations[x][y].typ = TileType::TrCorner;
        }
        if x == r.lx && y == r.hy {
            self.grid.locations[x][y].typ = TileType::BlCorner;
        }
        if x == r.hx && y == r.hy {
            self.grid.locations[x][y].typ = TileType::BrCorner;
        }
    }

    fn dig_corridor(&mut self, start: usize, end: usize, constant: usize, horizontal: bool) {
        let (min, max) = if start < end {
            (start, end)
        } else {
            (end, start)
        };

        for pos in min..=max {
            let (x, y) = if horizontal {
                (pos, constant)
            } else {
                (constant, pos)
            };
            if x >= COLNO || y >= ROWNO {
                continue;
            }

            let tile = &mut self.grid.locations[x][y];

            //
            if tile.typ.is_wall() {
                if tile.roomno > 0 {
                    //
                    tile.typ = TileType::Door;
                    tile.doormas = 0; // Closed
                } else {
                    tile.typ = TileType::Corr;
                }
            } else if tile.typ == TileType::Stone {
                tile.typ = TileType::Corr;
            }
        }
    }

    /// Sokoban Level Generation
    fn make_sokoban(
        &mut self,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
    ) {
        // Simple Sokoban Map (Usually loaded from file, here hardcoded)
        // 5x5 room with puzzle
        let map_data = [
            ".......", ".WWWWW.", ".W...W.", ".W.0.W.", ".W...W.", ".WW^WW.", ".......",
        ];

        let offset_x = 30;
        let offset_y = 8;
        let map_w = 7;
        let map_h = 7;

        use crate::core::entity::spawn::Spawner;

        for (y, row) in map_data.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let gx = offset_x + x;
                let gy = offset_y + y;

                let tile = &mut self.grid.locations[gx][gy];

                match ch {
                    '.' => {
                        tile.typ = TileType::Room;
                        tile.flags = crate::core::dungeon::tile::TileFlags::LIT;
                    }
                    'W' => tile.typ = TileType::VWall, // Simplified wall
                    '0' => {
                        tile.typ = TileType::Room;
                        tile.flags = crate::core::dungeon::tile::TileFlags::LIT;
                        // Spawn Boulder (Item)
                        Spawner::mksobj_at("boulder", gx, gy, items, world, self.id);
                    }
                    '^' => {
                        tile.typ = TileType::Room; // Hole is on the floor (usually logic handles trap separately)
                        tile.flags = crate::core::dungeon::tile::TileFlags::LIT;
                        // Spawn Hole (Trap)
                        // Trap system is needed here. For now, assume trap system handles entities with Trap component.
                        // Implemented as placeholder.
                        let _trap = world.push((
                            crate::core::entity::Trap {
                                typ: crate::core::entity::TrapType::Hole,
                                discovered: false,
                            },
                            crate::core::entity::Position {
                                x: gx as i32,
                                y: gy as i32,
                            },
                            crate::core::entity::Level(self.id),
                        ));
                    }
                    _ => {}
                }

                // Set room number for proper wall logic if needed, but here simple map.
                if tile.typ == TileType::Room || tile.typ == TileType::Fountain {
                    tile.roomno = 1;
                }
            }
        }

        // Register as a room so player can spawn
        let room = MkRoom::new(
            offset_x,
            offset_y,
            offset_x + map_w - 1,
            offset_y + map_h - 1,
        );
        self.apply_room(&room, 1);
        self.rooms.rooms.push(room);
    }

    ///
    ///
    ///
    fn fill_level_monsters(
        &mut self,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        id: crate::core::dungeon::LevelID,
        level_type: LevelType,
    ) {
        use crate::core::entity::spawn::{Spawner, NO_MM_FLAGS};

        //
        if level_type == LevelType::Sokoban {
            return;
        }

        let nroom = self.rooms.rooms.len().max(1);
        let depth = id.depth;

        //
        //
        let extra_monsters = nroom * (1 + self.rng.rn2(3) as usize);
        let mut spawned = 0;
        let mut attempts = 0;
        while spawned < extra_monsters && attempts < extra_monsters * 10 {
            attempts += 1;
            let x = self.rng.rn2(COLNO as i32) as usize;
            let y = self.rng.rn2(ROWNO as i32) as usize;
            let tile_typ = self.grid.locations[x][y].typ;
            //
            if tile_typ == TileType::Room || tile_typ == TileType::Corr {
                if let Some(_ent) = Spawner::makemon(
                    None,
                    x,
                    y,
                    NO_MM_FLAGS,
                    world,
                    &self.grid,
                    templates,
                    items,
                    self.rng,
                    id,
                ) {
                    spawned += 1;
                }
            }
        }

        //
        //
        let item_count = (self.rng.rn2(5) + 1 + depth.min(10)) as usize;
        let mut item_spawned = 0;
        let mut item_attempts = 0;
        while item_spawned < item_count && item_attempts < item_count * 10 {
            item_attempts += 1;
            let x = self.rng.rn2(COLNO as i32) as usize;
            let y = self.rng.rn2(ROWNO as i32) as usize;
            let tile_typ = self.grid.locations[x][y].typ;
            if tile_typ == TileType::Room || tile_typ == TileType::Corr {
                //
                let class = match self.rng.rn2(8) {
                    0 => crate::core::entity::object::ItemClass::Weapon,
                    1 => crate::core::entity::object::ItemClass::Armor,
                    2 => crate::core::entity::object::ItemClass::Potion,
                    3 => crate::core::entity::object::ItemClass::Scroll,
                    4 => crate::core::entity::object::ItemClass::Food,
                    5 => crate::core::entity::object::ItemClass::Gem,
                    6 => crate::core::entity::object::ItemClass::Tool,
                    _ => crate::core::entity::object::ItemClass::Ring,
                };
                if let Some(e) = Spawner::mkobj_of_class(class, items, world, self.rng) {
                    if let Some(mut entry) = world.entry(e) {
                        entry.add_component(crate::core::entity::Position {
                            x: x as i32,
                            y: y as i32,
                        });
                        entry.add_component(crate::core::entity::Level(id));
                    }
                    item_spawned += 1;
                }
            }
        }

        //
        //
        let gold_stacks = self.rng.rn2(3) as usize + 1;
        for _ in 0..gold_stacks {
            let x = self.rng.rn2(COLNO as i32) as usize;
            let y = self.rng.rn2(ROWNO as i32) as usize;
            let tile_typ = self.grid.locations[x][y].typ;
            if tile_typ == TileType::Room || tile_typ == TileType::Corr {
                let amount = self.rng.rnz(10, 50 + depth * 10);
                Spawner::mkgold(amount, x, y, world, id);
            }
        }
    }

    // =========================================================================
    // [v2.0.0
    //
    //
    // =========================================================================
    fn place_traps(
        &mut self,
        world: &mut World,
        id: crate::core::dungeon::LevelID,
        level_type: LevelType,
    ) {
        use crate::core::entity::{Position, Trap, TrapTag};

        //
        if level_type == LevelType::Sokoban {
            return;
        }

        let depth = id.depth.abs().max(1);
        //
        let num_traps = (depth / 4 + 2 + self.rng.rn2(5)) as usize;

        for _ in 0..num_traps {
            //
            let mut placed = false;
            for _ in 0..50 {
                let x = self.rng.rn2(COLNO as i32) as usize;
                let y = self.rng.rn2(ROWNO as i32) as usize;
                let tile_typ = self.grid.locations[x][y].typ;

                //
                if tile_typ != TileType::Room && tile_typ != TileType::Corr {
                    continue;
                }

                //
                let trap_type = self.select_trap_type(depth);

                //
                world.push((
                    TrapTag,
                    Position {
                        x: x as i32,
                        y: y as i32,
                    },
                    Trap {
                        typ: trap_type,
                        discovered: false,
                    },
                    crate::core::entity::Level(id),
                    crate::core::entity::Renderable {
                        glyph: '^',
                        color: 1,
                    },
                ));

                placed = true;
                break;
            }

            if !placed {
                break;
            }
        }
    }

    ///
    fn select_trap_type(&mut self, depth: i32) -> crate::core::entity::TrapType {
        use crate::core::entity::TrapType;

        //
        let roll = self.rng.rn2(100);

        if depth <= 3 {
            //
            match roll {
                0..=25 => TrapType::Arrow,
                26..=45 => TrapType::Dart,
                46..=65 => TrapType::SqueakyBoard,
                66..=80 => TrapType::BearTrap,
                81..=90 => TrapType::Pit,
                _ => TrapType::Rock,
            }
        } else if depth <= 8 {
            //
            match roll {
                0..=15 => TrapType::Arrow,
                16..=25 => TrapType::Dart,
                26..=35 => TrapType::SqueakyBoard,
                36..=50 => TrapType::BearTrap,
                51..=60 => TrapType::Pit,
                61..=70 => TrapType::SpikedPit,
                71..=80 => TrapType::SleepGas,
                81..=85 => TrapType::Landmine,
                86..=90 => TrapType::Rust,
                91..=95 => TrapType::Web,
                _ => TrapType::Fire,
            }
        } else if depth <= 15 {
            //
            match roll {
                0..=10 => TrapType::Arrow,
                11..=18 => TrapType::SpikedPit,
                19..=28 => TrapType::BearTrap,
                29..=38 => TrapType::Landmine,
                39..=48 => TrapType::SleepGas,
                49..=55 => TrapType::Rust,
                56..=62 => TrapType::Fire,
                63..=68 => TrapType::Web,
                69..=74 => TrapType::Hole,
                75..=80 => TrapType::TrapDoor,
                81..=85 => TrapType::Teleport,
                86..=90 => TrapType::Magic,
                91..=95 => TrapType::AntiMagic,
                _ => TrapType::RollingBoulder,
            }
        } else {
            //
            match roll {
                0..=8 => TrapType::SpikedPit,
                9..=16 => TrapType::Landmine,
                17..=24 => TrapType::Fire,
                25..=32 => TrapType::SleepGas,
                33..=38 => TrapType::Hole,
                39..=44 => TrapType::TrapDoor,
                45..=52 => TrapType::Teleport,
                53..=58 => TrapType::LevelTeleport,
                59..=64 => TrapType::Magic,
                65..=70 => TrapType::AntiMagic,
                71..=76 => TrapType::Web,
                77..=82 => TrapType::RollingBoulder,
                83..=88 => TrapType::Polymorph,
                89..=94 => TrapType::Statue,
                _ => TrapType::Rust,
            }
        }
    }
}

pub struct MapGenerator;

impl MapGenerator {
    ///
    pub fn generate_improved(
        rng: &mut NetHackRng,
        id: crate::core::dungeon::LevelID,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        level_type: LevelType,
    ) -> (Grid, (i32, i32), (i32, i32), Vec<MkRoom>) {
        let mut gen = LevelGen::new(rng, id);

        match level_type {
            LevelType::Ordinary => {
                //
                gen.makerooms();

                //
                gen.rooms.finalize_special_rooms(
                    id,
                    &mut gen.grid,
                    world,
                    items,
                    templates,
                    gen.rng,
                );

                //
                gen.makecorridors();
            }
            LevelType::Bigroom => {
                gen.make_bigroom(world, items, templates, id);
            }
            LevelType::Mines => {
                gen.make_mines(world, items, templates, id);
            }
            LevelType::Maze => {
                gen.make_maze(world, items, templates, id);
            }
            LevelType::Sokoban => {
                gen.make_sokoban(world, items);
            }
            LevelType::Minetown => {
                gen.make_minetown(world, items, templates, id);
            }
        }

        // [v2.0.0
        //
        gen.fill_level_monsters(world, items, templates, id, level_type);

        // [v2.0.0
        gen.place_traps(world, id, level_type);

        //
        crate::core::entity::spawn::Spawner::spawn_level_structures(world, &gen.grid, gen.rng, id);

        //
        let mut start_pos = (10, 10);
        let mut down_stairs = (10, 10);

        use crate::core::dungeon::{DungeonBranch, LevelID};

        //
        if let Some(first_room) = gen.rooms.rooms.first() {
            let sx = (first_room.lx + (first_room.hx - first_room.lx) / 2) as i32;
            let sy = (first_room.ly + (first_room.hy - first_room.ly) / 2) as i32;
            start_pos = (sx, sy);

            if id.branch == DungeonBranch::Sokoban {
                //
                gen.grid.locations[sx as usize][sy as usize].typ = TileType::StairsDown;

                let target = if id.depth == 1 {
                    LevelID::new(DungeonBranch::Main, 6)
                } else {
                    LevelID::new(DungeonBranch::Sokoban, id.depth - 1)
                };
                gen.grid.portals.insert((sx, sy), target);
            } else {
                //
                gen.grid.locations[sx as usize][sy as usize].typ = TileType::StairsUp;

                let target = if id.branch == DungeonBranch::Mines && id.depth == 1 {
                    LevelID::new(DungeonBranch::Main, 3)
                } else if id.depth > 1 {
                    LevelID::new(id.branch, id.depth - 1)
                } else {
                    // id.depth == 1 at Main: Exit dungeon (no portal target means standard prev level logic handles it or exit)
                    // We can explicitly set target to a special "Surface" level if we had one,
                    // or let stairs system handle "no portal" as "PrevLevel" -> potentially exit.
                    // For now, let's insert a dummy target to avoid confusion if needed, or skip.
                    // Skipping portal insertion means stairs system will default to PrevLevel.
                    LevelID::new(DungeonBranch::Main, 0) // Placeholder for surface
                };

                if target.depth > 0 {
                    gen.grid.portals.insert((sx, sy), target);
                }
            }
        }

        //
        if let Some(last_room) = gen.rooms.rooms.last() {
            let dx = (last_room.lx + (last_room.hx - last_room.lx) / 2) as i32;
            let dy = (last_room.ly + (last_room.hy - last_room.ly) / 2) as i32;

            if (dx, dy) != start_pos {
                down_stairs = (dx, dy);

                if id.branch == DungeonBranch::Sokoban {
                    // Sokoban: Climb UP to next level
                    gen.grid.locations[dx as usize][dy as usize].typ = TileType::StairsUp;
                    gen.grid
                        .portals
                        .insert((dx, dy), LevelID::new(id.branch, id.depth + 1));
                } else {
                    // Standard: Go DOWN to next level
                    gen.grid.locations[dx as usize][dy as usize].typ = TileType::StairsDown;
                    gen.grid
                        .portals
                        .insert((dx, dy), LevelID::new(id.branch, id.depth + 1));
                }
            }
        }

        //
        if id.branch == DungeonBranch::Main {
            if id.depth == 3 {
                // Mines Entrance (Target: Mines 1)
                let len = gen.rooms.rooms.len();
                if len > 2 {
                    let room_idx = gen.rng.rn2(len as i32 - 2) + 1;
                    let room = &gen.rooms.rooms[room_idx as usize];
                    let mx = (room.lx + (room.hx - room.lx) / 2) as i32;
                    let my = (room.ly + (room.hy - room.ly) / 2) as i32;

                    if gen.grid.locations[mx as usize][my as usize].typ == TileType::Room {
                        gen.grid.locations[mx as usize][my as usize].typ = TileType::StairsDown;
                        gen.grid
                            .portals
                            .insert((mx, my), LevelID::new(DungeonBranch::Mines, 1));
                        println!("[Debug] Created Mines Entrance at {},{}", mx, my);
                    }
                }
            } else if id.depth == 6 {
                // Sokoban Entrance (Target: Sokoban 1) - UP stairs
                let len = gen.rooms.rooms.len();
                if len > 2 {
                    let room_idx = gen.rng.rn2(len as i32 - 2) + 1;
                    let room = &gen.rooms.rooms[room_idx as usize];
                    let sx = (room.lx + (room.hx - room.lx) / 2) as i32;
                    let sy = (room.ly + (room.hy - room.ly) / 2) as i32;

                    if gen.grid.locations[sx as usize][sy as usize].typ == TileType::Room {
                        gen.grid.locations[sx as usize][sy as usize].typ = TileType::StairsUp;
                        gen.grid
                            .portals
                            .insert((sx, sy), LevelID::new(DungeonBranch::Sokoban, 1));
                        println!("[Debug] Created Sokoban Entrance at {},{}", sx, sy);
                    }
                }
            }
        }

        (gen.grid, start_pos, down_stairs, gen.rooms.rooms)
    }
}
