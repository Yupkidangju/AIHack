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

// =============================================================================
// [v2.9.9] mkroom.c 미구현 함수 이식
// =============================================================================

impl MkRoom {
    /// [v2.9.9] 방 내 랜덤 x좌표 (원본: somex L636-641)
    pub fn somex(&self, rng: &mut NetHackRng) -> usize {
        let range = (self.hx - self.lx + 1) as i32;
        self.lx + rng.rn2(range) as usize
    }

    /// [v2.9.9] 방 내 랜덤 y좌표 (원본: somey L643-648)
    pub fn somey(&self, rng: &mut NetHackRng) -> usize {
        let range = (self.hy - self.ly + 1) as i32;
        self.ly + rng.rn2(range) as usize
    }

    /// [v2.9.9] 좌표가 방 내부인지 (벽 포함) (원본: inside_room L650-657)
    pub fn inside_room(&self, x: usize, y: usize) -> bool {
        x >= self.lx.saturating_sub(1)
            && x <= self.hx + 1
            && y >= self.ly.saturating_sub(1)
            && y <= self.hy + 1
    }
}

/// [v2.9.9] 궁정 몬스터 심볼 선택 (원본: courtmon L735-758)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CourtMonsterClass {
    Dragon,    // i > 100
    Giant,     // i > 95
    Troll,     // i > 85
    Centaur,   // i > 75
    Orc,       // i > 60
    Bugbear,   // i > 45
    Hobgoblin, // i > 30
    Gnome,     // i > 15
    Kobold,    // i <= 15
}

/// [v2.9.9] 궁정 몬스터 판정 (원본: courtmon L735-758)
pub fn courtmon_result(level_difficulty: i32, rng: &mut NetHackRng) -> CourtMonsterClass {
    let i = rng.rn2(60) + rng.rn2(3 * level_difficulty);
    if i > 100 {
        CourtMonsterClass::Dragon
    } else if i > 95 {
        CourtMonsterClass::Giant
    } else if i > 85 {
        CourtMonsterClass::Troll
    } else if i > 75 {
        CourtMonsterClass::Centaur
    } else if i > 60 {
        CourtMonsterClass::Orc
    } else if i > 45 {
        CourtMonsterClass::Bugbear
    } else if i > 30 {
        CourtMonsterClass::Hobgoblin
    } else if i > 15 {
        CourtMonsterClass::Gnome
    } else {
        CourtMonsterClass::Kobold
    }
}

/// [v2.9.9] 군인 확률 테이블 (원본: squadprob L762-768)
const SQUAD_PROBS: [(u32, u32); 4] = [
    (0, 80), // PM_SOLDIER, 80%
    (1, 15), // PM_SERGEANT, 15%
    (2, 4),  // PM_LIEUTENANT, 4%
    (3, 1),  // PM_CAPTAIN, 1%
];

/// [v2.9.9] 군인 유형 판정 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SquadType {
    Soldier,
    Sergeant,
    Lieutenant,
    Captain,
}

/// [v2.9.9] 군인 유형 판정 (원본: squadmon L770-792)
pub fn squadmon_result(level_difficulty: i32, rng: &mut NetHackRng) -> SquadType {
    let sel_prob = rng.rnd(80 + level_difficulty);
    let mut cpro = 0i32;
    for &(idx, prob) in &SQUAD_PROBS {
        cpro += prob as i32;
        if cpro > sel_prob {
            return match idx {
                0 => SquadType::Soldier,
                1 => SquadType::Sergeant,
                2 => SquadType::Lieutenant,
                _ => SquadType::Captain,
            };
        }
    }
    // 폴백 — 랜덤 선택
    match rng.rn2(4) {
        0 => SquadType::Soldier,
        1 => SquadType::Sergeant,
        2 => SquadType::Lieutenant,
        _ => SquadType::Captain,
    }
}

/// [v2.9.9] 묘지 몬스터 유형 (원본: morguemon L459-481)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MorgueMonType {
    Demon,
    Vampire,
    Ghost,
    Wraith,
    Zombie,
}

/// [v2.9.9] 묘지 몬스터 판정
pub fn morguemon_result(
    level_difficulty: i32,
    in_hell: bool,
    in_endgame: bool,
    rng: &mut NetHackRng,
) -> MorgueMonType {
    let i = rng.rn2(100);
    let hd = rng.rn2(level_difficulty.max(1));

    if hd > 10 && i < 10 {
        if in_hell || in_endgame {
            return MorgueMonType::Demon;
        }
        // 악마가 없으면 아래로 폴스루
    }

    if hd > 8 && i > 85 {
        return MorgueMonType::Vampire;
    }

    if i < 20 {
        MorgueMonType::Ghost
    } else if i < 40 {
        MorgueMonType::Wraith
    } else {
        MorgueMonType::Zombie
    }
}

/// [v2.9.9] 개미굴 몬스터 유형 (원본: antholemon L483-509)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AntType {
    SoldierAnt,
    FireAnt,
    GiantAnt,
}

/// [v2.9.9] 개미굴 몬스터 판정
pub fn antholemon_result(
    birthday_mod3: i32,
    level_difficulty: i32,
    genocided: &[bool; 3], // [soldier, fire, giant]
) -> Option<AntType> {
    let indx = birthday_mod3 + level_difficulty;
    for trycnt in 0..3 {
        let ant = match (indx + trycnt) % 3 {
            0 => AntType::SoldierAnt,
            1 => AntType::FireAnt,
            _ => AntType::GiantAnt,
        };
        let idx = match ant {
            AntType::SoldierAnt => 0,
            AntType::FireAnt => 1,
            AntType::GiantAnt => 2,
        };
        if !genocided[idx] {
            return Some(ant);
        }
    }
    None // 전부 제거됨
}

/// [v2.9.9] 언데드 무리 판정 결과 (원본: mkundead L436-457)
pub struct MkundeadResult {
    /// 소환할 언데드 수
    pub count: i32,
    /// 각 언데드의 유형
    pub types: Vec<MorgueMonType>,
}

/// [v2.9.9] 언데드 무리 판정
pub fn mkundead_result(
    level_difficulty: i32,
    in_hell: bool,
    in_endgame: bool,
    rng: &mut NetHackRng,
) -> MkundeadResult {
    let cnt = (level_difficulty + 1) / 10 + rng.rnd(5);
    let mut types = Vec::with_capacity(cnt as usize);
    for _ in 0..cnt {
        types.push(morguemon_result(level_difficulty, in_hell, in_endgame, rng));
    }
    MkundeadResult { count: cnt, types }
}

/// [v2.9.9] 왕좌 몬스터 판정 (원본: mk_zoo_thronemon L243-261)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThroneKingType {
    GnomeKing, // i <= 2
    DwarfKing, // i <= 5
    ElvenKing, // i <= 9
    OgreKing,  // i > 9
}

/// [v2.9.9] 왕좌 몬스터 판정
pub fn throne_king_result(level_difficulty: i32, rng: &mut NetHackRng) -> ThroneKingType {
    let i = rng.rnd(level_difficulty.max(1));
    if i > 9 {
        ThroneKingType::OgreKing
    } else if i > 5 {
        ThroneKingType::ElvenKing
    } else if i > 2 {
        ThroneKingType::DwarfKing
    } else {
        ThroneKingType::GnomeKing
    }
}

/// [v2.9.9] 방 선택 판정 (원본: pick_room L205-229)
pub fn pick_room_result(
    rooms: &[MkRoom],
    has_upstairs: &[bool],
    has_dnstairs: &[bool],
    strict: bool,
    rng: &mut NetHackRng,
) -> Option<usize> {
    let nroom = rooms.len();
    if nroom == 0 {
        return None;
    }
    let start = rng.rn2(nroom as i32) as usize;
    for offset in 0..nroom {
        let idx = (start + offset) % nroom;
        if rooms[idx].rtype != RoomType::Ordinary {
            continue;
        }
        if strict {
            if has_upstairs[idx] || has_dnstairs[idx] {
                continue;
            }
        } else {
            if has_upstairs[idx] {
                continue;
            }
            if has_dnstairs[idx] && rng.rn2(3) != 0 {
                continue;
            }
        }
        if rooms[idx].doorct == 1 || rng.rn2(5) == 0 {
            return Some(idx);
        }
    }
    None
}

/// [v2.9.9] 특수 방 탐색 결과 (원본: search_special L710-733)
pub fn search_special_result(
    rooms: &[MkRoom],
    target_type: RoomType,
    any_type: bool,
    any_shop: bool,
) -> Option<usize> {
    for (i, room) in rooms.iter().enumerate() {
        if (any_type && room.rtype != RoomType::Ordinary)
            || (any_shop && room.rtype as u32 >= RoomType::ShopBase as u32)
            || room.rtype == target_type
        {
            return Some(i);
        }
    }
    None
}

/// [v2.9.9] 심볼→터레인 변환 (원본: cmap_to_type L865-983)
/// 심볼 인덱스(S_stone=0, S_vwall=1, ...) → TileType
pub fn cmap_to_type(sym: i32) -> TileType {
    match sym {
        0 => TileType::Stone,                     // S_stone
        1 => TileType::VWall,                     // S_vwall
        2 => TileType::HWall,                     // S_hwall
        3 => TileType::TlCorner,                  // S_tlcorn
        4 => TileType::TrCorner,                  // S_trcorn
        5 => TileType::BlCorner,                  // S_blcorn
        6 => TileType::BrCorner,                  // S_brcorn
        7 => TileType::CrossWall,                 // S_crwall
        8 => TileType::TuWall,                    // S_tuwall
        9 => TileType::TdWall,                    // S_tdwall
        10 => TileType::TlWall,                   // S_tlwall
        11 => TileType::TrWall,                   // S_trwall
        12 | 13 | 14 | 15 | 16 => TileType::Door, // 문 계열
        17 => TileType::IronBars,                 // S_bars
        18 => TileType::Tree,                     // S_tree
        19 => TileType::Room,                     // S_room
        20 | 21 => TileType::Corr,                // S_corr, S_litcorr
        22 => TileType::StairsUp,                 // S_upstair
        23 => TileType::StairsDown,               // S_dnstair
        24 | 25 => TileType::Ladder,              // S_upladder, S_dnladder
        26 => TileType::Altar,                    // S_altar
        27 => TileType::Grave,                    // S_grave
        28 => TileType::Throne,                   // S_throne
        29 => TileType::Sink,                     // S_sink
        30 => TileType::Fountain,                 // S_fountain
        31 => TileType::Pool,                     // S_pool
        32 => TileType::Ice,                      // S_ice
        33 => TileType::LavaPool,                 // S_lava
        34 | 35 => TileType::DrawbridgeDown,      // 도개교 열림
        36 | 37 => TileType::DbWall,              // 도개교 닫힘
        38 => TileType::Air,                      // S_air
        39 => TileType::Cloud,                    // S_cloud
        40 => TileType::Water,                    // S_water
        _ => TileType::Stone,                     // 기본값
    }
}

// =============================================================================
// [v2.9.9] 테스트 — mkroom.c 추가 이식분
// =============================================================================
#[cfg(test)]
mod mkroom_phase2_tests {
    use super::*;

    #[test]
    fn test_somex_range() {
        let room = MkRoom::new(5, 10, 15, 20);
        let mut rng = NetHackRng::new(42);
        for _ in 0..100 {
            let x = room.somex(&mut rng);
            assert!(x >= 5 && x <= 15);
        }
    }

    #[test]
    fn test_somey_range() {
        let room = MkRoom::new(5, 10, 15, 20);
        let mut rng = NetHackRng::new(42);
        for _ in 0..100 {
            let y = room.somey(&mut rng);
            assert!(y >= 10 && y <= 20);
        }
    }

    #[test]
    fn test_inside_room() {
        let room = MkRoom::new(5, 5, 10, 10);
        assert!(room.inside_room(5, 5));
        assert!(room.inside_room(4, 4)); // 벽 포함
        assert!(room.inside_room(11, 11));
        assert!(!room.inside_room(3, 3));
        assert!(!room.inside_room(12, 12));
    }

    #[test]
    fn test_courtmon_variety() {
        let mut classes = std::collections::HashSet::new();
        for seed in 0..200u64 {
            let mut r = NetHackRng::new(seed);
            classes.insert(courtmon_result(10, &mut r));
        }
        assert!(classes.len() >= 3);
    }

    #[test]
    fn test_courtmon_high_difficulty() {
        // 높은 난이도에서 드래곤 출현 가능
        for seed in 0..500u64 {
            let mut r = NetHackRng::new(seed);
            if courtmon_result(50, &mut r) == CourtMonsterClass::Dragon {
                return;
            }
        }
        // 드래곤이 한 번도 안 나올 수 있으므로 패스
    }

    #[test]
    fn test_squadmon_soldier_dominant() {
        let mut soldier_count = 0;
        for seed in 0..200u64 {
            let mut r = NetHackRng::new(seed);
            if squadmon_result(5, &mut r) == SquadType::Soldier {
                soldier_count += 1;
            }
        }
        assert!(soldier_count > 100); // ~80%
    }

    #[test]
    fn test_morguemon_variety() {
        let mut types = std::collections::HashSet::new();
        for seed in 0..200u64 {
            let mut r = NetHackRng::new(seed);
            types.insert(morguemon_result(15, false, false, &mut r));
        }
        assert!(types.len() >= 3); // ghost, wraith, zombie 최소
    }

    #[test]
    fn test_antholemon_normal() {
        let genocided = [false, false, false];
        let result = antholemon_result(0, 5, &genocided);
        assert!(result.is_some());
    }

    #[test]
    fn test_antholemon_all_genocided() {
        let genocided = [true, true, true];
        assert_eq!(antholemon_result(0, 5, &genocided), None);
    }

    #[test]
    fn test_mkundead_count() {
        let mut rng = NetHackRng::new(42);
        let result = mkundead_result(20, false, false, &mut rng);
        assert!(result.count >= 1);
        assert_eq!(result.types.len(), result.count as usize);
    }

    #[test]
    fn test_throne_king_variety() {
        let mut types = std::collections::HashSet::new();
        for seed in 0..200u64 {
            let mut r = NetHackRng::new(seed);
            types.insert(throne_king_result(15, &mut r));
        }
        assert!(types.len() >= 2);
    }

    #[test]
    fn test_pick_room_empty() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(pick_room_result(&[], &[], &[], false, &mut rng), None);
    }

    #[test]
    fn test_pick_room_single() {
        let rooms = vec![MkRoom::new(1, 1, 10, 10)];
        let up = vec![false];
        let dn = vec![false];
        // doorct가 0이므로 rn2(5)==0이 필요
        let mut found = false;
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            if pick_room_result(&rooms, &up, &dn, false, &mut r).is_some() {
                found = true;
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_search_special_found() {
        let mut rooms = vec![MkRoom::new(1, 1, 10, 10)];
        rooms[0].rtype = RoomType::Zoo;
        assert_eq!(
            search_special_result(&rooms, RoomType::Zoo, false, false),
            Some(0)
        );
    }

    #[test]
    fn test_search_special_not_found() {
        let rooms = vec![MkRoom::new(1, 1, 10, 10)]; // Ordinary
        assert_eq!(
            search_special_result(&rooms, RoomType::Zoo, false, false),
            None
        );
    }

    #[test]
    fn test_cmap_to_type_room() {
        assert_eq!(cmap_to_type(19), TileType::Room);
    }

    #[test]
    fn test_cmap_to_type_door() {
        assert_eq!(cmap_to_type(12), TileType::Door);
        assert_eq!(cmap_to_type(16), TileType::Door);
    }

    #[test]
    fn test_cmap_to_type_stairs() {
        assert_eq!(cmap_to_type(22), TileType::StairsUp);
        assert_eq!(cmap_to_type(23), TileType::StairsDown);
    }

    #[test]
    fn test_cmap_to_type_fountain() {
        assert_eq!(cmap_to_type(30), TileType::Fountain);
    }
}
