// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::Grid;
use crate::util::rng::NetHackRng;
use legion::World;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    Ordinary = 0,
    Court = 1,
    Zoo = 2,
    Beehive = 3,
    Morgue = 4,
    Barracks = 5,
    Swamp = 6,
    Temple = 7,
    LepreHall = 8,
    CockNest = 9,
    AntHole = 10,
    ShopBase = 11,
    Oracle = 12,
}

pub const MAX_SUBROOMS: usize = 10;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MkRoom {
    pub lx: usize,
    pub ly: usize,
    pub hx: usize,
    pub hy: usize,
    pub rtype: RoomType,
    pub rlit: bool,
    pub doorct: usize,
    pub fdoor: usize, // index into doors list
    pub nsubrooms: usize,
    pub irregular: bool,
    pub subrooms: Vec<usize>,  // indices into rooms list
    pub parent: Option<usize>, // index into rooms list
}

impl MkRoom {
    pub fn new(lx: usize, ly: usize, hx: usize, hy: usize) -> Self {
        Self {
            lx,
            ly,
            hx,
            hy,
            rtype: RoomType::Ordinary,
            rlit: false,
            doorct: 0,
            fdoor: 0,
            nsubrooms: 0,
            irregular: false,
            subrooms: Vec::with_capacity(MAX_SUBROOMS),
            parent: None,
        }
    }

    pub fn area(&self) -> usize {
        // [v1.9.0
        (self.hx.saturating_sub(self.lx) + 1) * (self.hy.saturating_sub(self.ly) + 1)
    }

    ///
    pub fn is_big(&self) -> bool {
        self.area() > 20
    }

    ///
    ///
    pub fn somexy(&self, rng: &mut NetHackRng) -> (usize, usize) {
        //
        let dx = self.hx.saturating_sub(self.lx);
        let dy = self.hy.saturating_sub(self.ly);

        let x = if dx <= 2 {
            (self.lx + self.hx) / 2
        } else {
            self.lx + 1 + rng.rn2((dx - 1) as i32) as usize
        };
        let y = if dy <= 2 {
            (self.ly + self.hy) / 2
        } else {
            self.ly + 1 + rng.rn2((dy - 1) as i32) as usize
        };
        (x, y)
    }
}

pub struct RoomManager {
    pub rooms: Vec<MkRoom>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self { rooms: Vec::new() }
    }

    ///
    ///
    pub fn shrine_pos(room: &MkRoom, rng: &mut NetHackRng) -> (usize, usize) {
        let delta_x = room.hx.saturating_sub(room.lx);
        let mut sx = room.lx + delta_x / 2;
        if (delta_x % 2 != 0) && rng.rn2(2) == 0 {
            sx += 1;
        }

        let delta_y = room.hy.saturating_sub(room.ly);
        let mut sy = room.ly + delta_y / 2;
        if (delta_y % 2 != 0) && rng.rn2(2) == 0 {
            sy += 1;
        }

        (sx, sy)
    }

    ///
    pub fn mktemple(&mut self, room_idx: usize, grid: &mut Grid, rng: &mut NetHackRng) {
        if let Some(room) = self.rooms.get_mut(room_idx) {
            room.rtype = RoomType::Temple;
            let (sx, sy) = Self::shrine_pos(room, rng);
            if let Some(tile) = grid.get_tile_mut(sx, sy) {
                tile.typ = TileType::Altar;
                // altarmask bits (rm.h): 1=Lawful, 2=Neutral, 4=Chaotic
                tile.altarmask = match rng.rn2(3) {
                    0 => 1, // Lawful
                    1 => 2, // Neutral
                    _ => 4, // Chaotic
                };
            }
        }
    }

    ///
    pub fn finalize_special_rooms(
        &mut self,
        id: crate::core::dungeon::LevelID,
        grid: &mut Grid,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        rng: &mut NetHackRng,
    ) {
        if self.rooms.is_empty() {
            return;
        }

        //
        let nroom = self.rooms.len();
        let room_threshold = 3;
        let depth = id.depth;

        for i in 0..nroom {
            if self.rooms[i].rtype != RoomType::Ordinary {
                continue;
            }

            let rtype = if depth > 1 && depth < 20 && nroom >= room_threshold && rng.rn2(depth) < 3
            {
                Some(RoomType::ShopBase)
            } else if depth > 4 && rng.rn2(25) == 0 {
                Some(RoomType::Court)
            } else if depth > 6 && rng.rn2(30) == 0 {
                Some(RoomType::Zoo)
            } else if depth > 8 && rng.rn2(35) == 0 {
                Some(RoomType::Temple)
            } else if depth > 10 && rng.rn2(40) == 0 {
                Some(RoomType::Morgue)
            } else if depth > 12 && rng.rn2(45) == 0 {
                Some(RoomType::Barracks)
            } else if depth > 5 && rng.rn2(50) == 0 {
                Some(RoomType::Swamp)
            } else if depth > 7 && rng.rn2(55) == 0 {
                Some(RoomType::Beehive)
            } else if depth > 9 && rng.rn2(60) == 0 {
                Some(RoomType::LepreHall)
            } else if depth > 13 && rng.rn2(65) == 0 {
                Some(RoomType::CockNest)
            } else if depth > 3 && rng.rn2(70) == 0 {
                Some(RoomType::AntHole)
            } else if (5..=9).contains(&depth) && rng.rn2(5) == 0 {
                Some(RoomType::Oracle)
            } else {
                None
            };

            if let Some(rt) = rtype {
                if rt == RoomType::Temple {
                    self.mktemple(i, grid, rng);
                } else if rt == RoomType::ShopBase {
                    self.rooms[i].rtype = RoomType::ShopBase;
                    self.mkshop(i, grid, world, items, templates, rng, id);
                } else if rt == RoomType::Oracle {
                    self.mkoracle(i, grid, world, items, templates, rng, id);
                } else {
                    self.mkspecial(i, rt, world, grid, items, templates, rng, id);
                }
            } else {
                //
                self.fill_ordinary_room(i, world, items, templates, rng, id, grid);
            }
        }
    }

    ///
    ///
    ///
    pub fn fill_ordinary_room(
        &self,
        room_idx: usize,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        rng: &mut NetHackRng,
        id: crate::core::dungeon::LevelID,
        grid: &mut Grid,
    ) {
        let room = &self.rooms[room_idx];
        let area = room.area();

        //
        //
        //
        let monster_count = if room.is_big() {
            1 + rng.rn2(3) as usize // 1~3留덈━
        } else if area > 15 {
            rng.rn2(2) as usize + 1 // 1~2留덈━
        } else if rng.rn2(2) == 0 {
            1
        } else {
            0
        };

        for _ in 0..monster_count {
            let (sx, sy) = room.somexy(rng);
            crate::core::entity::spawn::Spawner::makemon(
                None,
                sx,
                sy,
                crate::core::entity::spawn::NO_MM_FLAGS,
                world,
                grid,
                templates,
                items,
                rng,
                id,
            );
        }

        //
        //
        let item_count = if room.is_big() {
            rng.rn2(3) as usize
        } else if rng.rn2(3) == 0 {
            1
        } else {
            0
        };

        for _ in 0..item_count {
            let (sx, sy) = room.somexy(rng);
            //
            let class = match rng.rn2(6) {
                0 => crate::core::entity::object::ItemClass::Weapon,
                1 => crate::core::entity::object::ItemClass::Armor,
                2 => crate::core::entity::object::ItemClass::Potion,
                3 => crate::core::entity::object::ItemClass::Scroll,
                4 => crate::core::entity::object::ItemClass::Food,
                _ => crate::core::entity::object::ItemClass::Gem,
            };
            if let Some(e) =
                crate::core::entity::spawn::Spawner::mkobj_of_class(class, items, world, rng)
            {
                if let Some(mut entry) = world.entry(e) {
                    entry.add_component(crate::core::entity::Position {
                        x: sx as i32,
                        y: sy as i32,
                    });
                    entry.add_component(crate::core::entity::Level(id));
                }
            }
        }

        //
        //
        if rng.rn2(10) == 0 {
            let (sx, sy) = room.somexy(rng);
            let amount = rng.rnz(10, 30 + id.depth * 5);
            crate::core::entity::spawn::Spawner::mkgold(amount, sx, sy, world, id);
        }

        //
        //

        //
        if id.depth > 2 && rng.rn2(10) == 0 {
            let (fx, fy) = room.somexy(rng);
            if let Some(tile) = grid.get_tile_mut(fx, fy) {
                if tile.typ == TileType::Room {
                    tile.typ = TileType::Fountain;
                }
            }
        }

        //
        if id.depth > 5 && rng.rn2(20) == 0 {
            let (sx, sy) = room.somexy(rng);
            if let Some(tile) = grid.get_tile_mut(sx, sy) {
                if tile.typ == TileType::Room {
                    tile.typ = TileType::Sink;
                }
            }
        }

        //
        if rng.rn2(20) == 0 {
            let (ax, ay) = room.somexy(rng);
            if let Some(tile) = grid.get_tile_mut(ax, ay) {
                if tile.typ == TileType::Room {
                    tile.typ = TileType::Altar;
                    tile.altarmask = match rng.rn2(3) {
                        0 => 1, // Lawful
                        1 => 2, // Neutral
                        _ => 4, // Chaotic
                    };
                }
            }
        }

        //
        if id.depth > 7 && rng.rn2(20) == 0 {
            let (gx, gy) = room.somexy(rng);
            if let Some(tile) = grid.get_tile_mut(gx, gy) {
                if tile.typ == TileType::Room {
                    tile.typ = TileType::Grave;
                }
            }
        }
    }

    pub fn mkspecial(
        &mut self,
        room_idx: usize,
        rtype: RoomType,
        world: &mut World,
        grid: &mut Grid,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        rng: &mut NetHackRng,
        id: crate::core::dungeon::LevelID,
    ) {
        use crate::core::entity::spawn::{Spawner, NO_MM_FLAGS};

        // 1. Update Room Type first (Mutable borrow)
        {
            if let Some(room) = self.rooms.get_mut(room_idx) {
                room.rtype = rtype;
            }
        }

        // 2. Clone room for population (Release borrow on self.rooms)
        let room = self.rooms[room_idx].clone();

        match rtype {
            RoomType::Court => {
                let (tx, ty) = Self::shrine_pos(&room, rng);
                if let Some(tile) = grid.get_tile_mut(tx, ty) {
                    tile.typ = TileType::Throne;
                }
                self.fill_room_with_monsters(&room, '@', 2, world, grid, templates, items, rng, id);
                // King or Queen (Soldiers)
                Spawner::makemon_of_class(
                    'H',
                    tx,
                    ty,
                    NO_MM_FLAGS,
                    world,
                    grid,
                    templates,
                    items,
                    rng,
                    id,
                );
                Spawner::mkgold(500, tx, ty, world, id);
            }
            RoomType::Zoo => {
                self.fill_room_with_monsters_random(
                    &room, 5, world, grid, templates, items, rng, id,
                );
                Spawner::mkgold(1000, room.lx + 1, room.ly + 1, world, id);
            }
            RoomType::Morgue => {
                self.fill_room_with_monsters(&room, 'Z', 2, world, grid, templates, items, rng, id);
                // Guaranteed Wraith or Ghost
                Spawner::makemon_of_class(
                    'W',
                    room.lx + 1,
                    room.ly + 1,
                    NO_MM_FLAGS,
                    world,
                    grid,
                    templates,
                    items,
                    rng,
                    id,
                );
                let (bx, by) = room.somexy(rng);
                Spawner::mksobj_at("large box", bx, by, items, world, id);
            }
            RoomType::Swamp => {
                for x in room.lx + 1..room.hx {
                    for y in room.ly + 1..room.hy {
                        if (x + y) % 2 == 0 && rng.rn2(2) == 0 {
                            if let Some(tile) = grid.get_tile_mut(x, y) {
                                tile.typ = TileType::Pool;
                            }
                            if rng.rn2(3) == 0 {
                                Spawner::makemon_of_class(
                                    if rng.rn2(2) == 0 { ';' } else { 'F' }, // Eels or Fungus
                                    x,
                                    y,
                                    NO_MM_FLAGS,
                                    world,
                                    grid,
                                    templates,
                                    items,
                                    rng,
                                    id,
                                );
                            }
                        }
                    }
                }
            }
            RoomType::Beehive => {
                self.fill_room_with_monsters(&room, 'a', 5, world, grid, templates, items, rng, id);
                // Queen Bee possible
                Spawner::makemon_of_class(
                    'a',
                    room.hx - 1,
                    room.hy - 1,
                    NO_MM_FLAGS,
                    world,
                    grid,
                    templates,
                    items,
                    rng,
                    id,
                );
                let (bx, by) = room.somexy(rng);
                Spawner::mksobj_at("food ration", bx, by, items, world, id);
            }
            RoomType::Barracks => {
                self.fill_room_with_monsters(&room, '@', 4, world, grid, templates, items, rng, id);
                // Guaranteed Sergeant or Captain
                Spawner::makemon_of_class(
                    '@',
                    room.lx + 1,
                    room.ly + 1,
                    NO_MM_FLAGS,
                    world,
                    grid,
                    templates,
                    items,
                    rng,
                    id,
                );
                let (bx, by) = room.somexy(rng);
                Spawner::mksobj_at("chest", bx, by, items, world, id);
            }
            RoomType::LepreHall => {
                self.fill_room_with_monsters(&room, 'l', 4, world, grid, templates, items, rng, id);
                // Scatter lots of gold
                for _ in 0..8 {
                    let (gx, gy) = room.somexy(rng);
                    Spawner::mkgold(rng.rnz(10, 100), gx, gy, world, id);
                }
            }
            RoomType::CockNest => {
                self.fill_room_with_monsters(&room, 'c', 4, world, grid, templates, items, rng, id);
            }
            RoomType::AntHole => {
                self.fill_room_with_monsters(&room, 'a', 6, world, grid, templates, items, rng, id);
            }
            RoomType::ShopBase => {
                //
                //
                self.mkshop(room_idx, grid, world, items, templates, rng, id);
            }
            _ => {}
        }
    }

    fn fill_room_with_monsters(
        &self,
        room: &MkRoom,
        symbol: char,
        density_factor: usize, // Lower is more dense? No, simpler: number of attempts/monsters
        world: &mut World,
        grid: &Grid,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        items: &crate::core::entity::object::ItemManager,
        rng: &mut NetHackRng,
        id: crate::core::dungeon::LevelID,
    ) {
        // Simple iteration to fill room
        let area = room.area();
        let count = (area / 10).max(1) * density_factor;

        for _ in 0..count {
            let (x, y) = room.somexy(rng);
            crate::core::entity::spawn::Spawner::makemon_of_class(
                symbol,
                x,
                y,
                crate::core::entity::spawn::NO_MM_FLAGS,
                world,
                grid,
                templates,
                items,
                rng,
                id,
            );
        }
    }

    fn fill_room_with_monsters_random(
        &self,
        room: &MkRoom,
        density_factor: usize,
        world: &mut World,
        grid: &Grid,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        items: &crate::core::entity::object::ItemManager,
        rng: &mut NetHackRng,
        id: crate::core::dungeon::LevelID,
    ) {
        let area = room.area();
        let count = (area / 10).max(1) * density_factor;

        for _ in 0..count {
            let (x, y) = room.somexy(rng);
            crate::core::entity::spawn::Spawner::makemon(
                None, // Random
                x,
                y,
                crate::core::entity::spawn::NO_MM_FLAGS,
                world,
                grid,
                templates,
                items,
                rng,
                id,
            );
        }
    }

    ///
    pub fn mkshop(
        &mut self,
        room_idx: usize,
        grid: &mut Grid,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        rng: &mut NetHackRng,
        id: crate::core::dungeon::LevelID,
    ) {
        use crate::core::entity::object::ItemClass;
        use crate::core::entity::spawn::{Spawner, NO_MM_FLAGS};
        use crate::core::entity::Position;

        //
        let room = self.rooms[room_idx].clone();

        //
        let shop_type = rng.rn2(6);
        let item_class = match shop_type {
            1 => ItemClass::Weapon,
            2 => ItemClass::Armor,
            3 => ItemClass::Food,
            4 => ItemClass::Potion,
            5 => ItemClass::Scroll,
            _ => ItemClass::Random, // General Store
        };

        //
        for x in room.lx..=room.hx {
            for y in room.ly..=room.hy {
                if let Some(tile) = grid.get_tile_mut(x, y) {
                    tile.shop_type = (shop_type + 1) as u8;
                }
            }
        }

        //
        //
        let (sx, sy) = room.somexy(rng);
        if let Some(shk_ent) = Spawner::makemon_of_class(
            '@',
            sx,
            sy,
            NO_MM_FLAGS,
            world,
            grid,
            templates,
            items,
            rng,
            id,
        ) {
            if let Some(mut entry) = world.entry(shk_ent) {
                entry.add_component(crate::core::entity::ShopkeeperTag);
                entry.add_component(crate::core::entity::Shopkeeper {
                    name: "Abigail".to_string(), // TODO: shknam.c:shkname()
                    shoproom: (room_idx as u8) + 1,
                });
                //
                if let Ok(monster) = entry.get_component_mut::<crate::core::entity::Monster>() {
                    monster.hostile = false;
                }
            }
        }

        //
        //
        for x in room.lx + 1..room.hx {
            for y in room.ly + 1..room.hy {
                if rng.rn2(4) == 0 {
                    if let Some(item_entity) =
                        Spawner::mkobj_of_class(item_class, items, world, rng)
                    {
                        if let Some(mut entry) = world.entry(item_entity) {
                            entry.add_component(Position {
                                x: x as i32,
                                y: y as i32,
                            });
                            entry.add_component(crate::core::entity::Level(id));
                            //
                            if let Ok(item) = entry.get_component_mut::<crate::core::entity::Item>()
                            {
                                item.unpaid = true;
                            }
                        }
                    }
                }
            }
        }
    }

    ///
    pub fn mkoracle(
        &mut self,
        room_idx: usize,
        grid: &mut Grid,
        world: &mut World,
        items: &crate::core::entity::object::ItemManager,
        templates: &Vec<&crate::core::entity::monster::MonsterTemplate>,
        rng: &mut NetHackRng,
        id: crate::core::dungeon::LevelID,
    ) {
        use crate::core::entity::spawn::{Spawner, NO_MM_FLAGS};

        //
        let room = self.rooms[room_idx].clone();

        //
        if let Some(r) = self.rooms.get_mut(room_idx) {
            r.rtype = RoomType::Oracle;
        }

        //
        let cx = (room.lx + room.hx) / 2;
        let cy = (room.ly + room.hy) / 2;

        //
        if let Some(tile) = grid.get_tile_mut(cx, cy) {
            tile.typ = TileType::Pool;
        }

        //
        let offsets = [(-2, -2), (2, -2), (-2, 2), (2, 2)];
        for (dx, dy) in offsets {
            let fx = (cx as i32 + dx) as usize;
            let fy = (cy as i32 + dy) as usize;
            if fx > room.lx && fx < room.hx && fy > room.ly && fy < room.hy {
                if let Some(tile) = grid.get_tile_mut(fx, fy) {
                    tile.typ = TileType::Fountain;
                }
            }
        }

        //
        let oracle_template = templates.iter().find(|t| t.name == "oracle").copied();
        Spawner::makemon(
            oracle_template,
            cx + 1,
            cy,
            NO_MM_FLAGS,
            world,
            grid,
            templates,
            items,
            rng,
            id,
        );
    }
}
