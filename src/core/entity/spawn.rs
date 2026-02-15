use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::{Grid, COLNO, ROWNO};
use crate::core::entity::monster::MonsterTemplate;
use crate::core::entity::{
    CombatStats, Health, LightSource, Monster, MonsterTag, Position, Renderable, Species, Talkative,
};
use crate::util::rng::NetHackRng;
use legion::{Entity, World};

/// NetHack makemon flags
pub const NO_MM_FLAGS: u32 = 0x00;
pub const MM_NOGRP: u32 = 0x01;
pub const MM_IGNLEVELCHECK: u32 = 0x02;
pub const MM_PEACEFUL: u32 = 0x04;

use crate::core::entity::object::ItemManager;
use crate::core::entity::{Equipment, Inventory, Item, ItemTag};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnRequest {
    pub x: i32,
    pub y: i32,
    pub template: String,
}

pub struct Spawner;

impl Spawner {
    ///
    pub fn spawn_monsters(
        world: &mut World,
        grid: &Grid,
        rng: &mut NetHackRng,
        templates: &Vec<&MonsterTemplate>,
        items: &ItemManager,
        level_id: crate::core::dungeon::LevelID,
        count: usize,
    ) {
        let mut spawned = 0;
        let mut attempts = 0;

        while spawned < count && attempts < count * 10 {
            attempts += 1;
            let x = rng.rn2(COLNO as i32) as usize;
            let y = rng.rn2(ROWNO as i32) as usize;

            if let Some(tile) = grid.get_tile(x, y) {
                if tile.typ == TileType::Room {
                    if let Some(_ent) = Self::makemon(
                        None,
                        x,
                        y,
                        NO_MM_FLAGS,
                        world,
                        grid,
                        templates,
                        items,
                        rng,
                        level_id,
                    ) {
                        spawned += 1;
                    }
                }
            }
        }
    }

    ///
    pub fn spawn_level_structures(
        world: &mut World,
        grid: &Grid,
        rng: &mut NetHackRng,
        level_id: crate::core::dungeon::LevelID,
    ) {
        if level_id.depth < 3 {
            return;
        }

        //
        let mut attempts = 0;
        while attempts < 20 {
            attempts += 1;
            let x = rng.rn2(COLNO as i32) as usize;
            let y = rng.rn2(ROWNO as i32) as usize;
            if grid
                .get_tile(x, y)
                .map(|t| t.typ == TileType::Room)
                .unwrap_or(false)
            {
                Self::spawn_structure(
                    world,
                    x,
                    y,
                    crate::core::entity::StructureType::CommBase,
                    level_id,
                );
                break;
            }
        }

        //
        for _ in 0..rng.d(1, 2) {
            let mut att = 0;
            while att < 20 {
                att += 1;
                let x = rng.rn2(COLNO as i32) as usize;
                let y = rng.rn2(ROWNO as i32) as usize;
                if grid
                    .get_tile(x, y)
                    .map(|t| t.typ == TileType::Room)
                    .unwrap_or(false)
                {
                    Self::spawn_structure(
                        world,
                        x,
                        y,
                        crate::core::entity::StructureType::SupplyDepot,
                        level_id,
                    );
                    break;
                }
            }
        }

        //
        if level_id.depth >= 5 && rng.rn2(3) == 0 {
            let spawn_pos = {
                let mut pos = None;
                let mut att = 0;
                while att < 20 {
                    att += 1;
                    let tx = rng.rn2(COLNO as i32) as usize;
                    let ty = rng.rn2(ROWNO as i32) as usize;
                    if grid
                        .get_tile(tx, ty)
                        .map(|t| t.typ == TileType::Room)
                        .unwrap_or(false)
                    {
                        pos = Some((tx, ty));
                        break;
                    }
                }
                pos
            };

            if let Some((x, y)) = spawn_pos {
                Self::spawn_beast_horde(world, x, y, level_id, rng);
            }
        }
    }

    ///
    pub fn spawn_beast_horde(
        world: &mut World,
        spawn_x: usize,
        spawn_y: usize,
        level_id: crate::core::dungeon::LevelID,
        rng: &mut NetHackRng,
    ) {
        //
        let n_minions = rng.d(3, 4);
        let mut target_poses = Vec::new();
        for _ in 0..n_minions {
            let ox = rng.d(1, 3) - 2;
            let oy = rng.d(1, 3) - 2;
            target_poses.push((
                (spawn_x as i32 + ox).clamp(0, COLNO as i32 - 1),
                (spawn_y as i32 + oy).clamp(0, ROWNO as i32 - 1),
            ));
        }

        //
        let b_ent = world.push((
            Position {
                x: spawn_x as i32,
                y: spawn_y as i32,
            },
            Renderable {
                glyph: 'F',
                color: 9,
            },
            crate::core::entity::Monster {
                kind: crate::generated::MonsterKind::from_str("orc warrior"),
                hostile: true,
                mon_name: Some("Red Flag Bearer".to_string()),
            },
            crate::core::entity::Level(level_id),
            MonsterTag,
            crate::core::entity::monster::MonsterState::default(),
            crate::core::entity::StandardBearer { aura_range: 5 },
        ));

        //
        for (nx, ny) in target_poses {
            let leader_val = b_ent;
            world.push((
                Position { x: nx, y: ny },
                Renderable {
                    glyph: 'o',
                    color: 10,
                },
                crate::core::entity::Monster {
                    kind: crate::generated::MonsterKind::from_str("orc"),
                    hostile: true,
                    mon_name: None,
                },
                crate::core::entity::monster::MonsterFaction {
                    faction: crate::core::entity::monster::Faction::Orc,
                    leader: Some(leader_val),
                },
                crate::core::entity::Level(level_id),
                MonsterTag,
                crate::core::entity::monster::MonsterState::default(),
                crate::core::entity::Health {
                    current: 15,
                    max: 15,
                },
            ));
        }
    }

    ///
    pub fn spawn_structure(
        world: &mut World,
        x: usize,
        y: usize,
        typ: crate::core::entity::StructureType,
        level_id: crate::core::dungeon::LevelID,
    ) -> Entity {
        let (glyph, color, integrity) = match typ {
            crate::core::entity::StructureType::CommBase => ('Y', 14, 30),
            crate::core::entity::StructureType::SupplyDepot => ('#', 10, 50),
        };
        world.push((
            Position {
                x: x as i32,
                y: y as i32,
            },
            Renderable { glyph, color },
            crate::core::entity::Structure {
                typ,
                integrity,
                max_integrity: integrity,
            },
            crate::core::entity::StructureTag,
            crate::core::entity::Level(level_id),
        ))
    }

    pub fn makemon(
        mdat: Option<&MonsterTemplate>,
        x: usize,
        y: usize,
        mmflags: u32,
        world: &mut World,
        grid: &Grid,
        templates: &Vec<&MonsterTemplate>,
        items: &ItemManager,
        rng: &mut NetHackRng,
        level_id: crate::core::dungeon::LevelID,
    ) -> Option<Entity> {
        let template = match mdat {
            Some(t) => t,
            None => match Self::select_monster_by_difficulty(templates, level_id.depth, rng) {
                Some(t) => t,
                None => return None,
            },
        };
        if (template.level as i32) > (level_id.depth + 5) && (mmflags & MM_IGNLEVELCHECK == 0) {
            return None;
        }
        let mut m_lev = template.level as i32;
        if level_id.depth > m_lev + 2 {
            m_lev += (level_id.depth - m_lev) / 2;
        }
        m_lev = m_lev.max(1);
        let hp = if m_lev <= 0 {
            rng.rnz(1, 4)
        } else {
            let mut h = 0;
            for _ in 0..m_lev {
                h += rng.rnz(1, 8);
            }
            h
        }
        .max(1);

        let faction = if template
            .has_capability(crate::core::entity::capability::MonsterCapability::Orc)
        {
            crate::core::entity::monster::Faction::Orc
        } else if template.has_capability(crate::core::entity::capability::MonsterCapability::Elf) {
            crate::core::entity::monster::Faction::Elf
        } else if template.has_capability(crate::core::entity::capability::MonsterCapability::Dwarf)
        {
            crate::core::entity::monster::Faction::Dwarf
        } else if template.has_capability(crate::core::entity::capability::MonsterCapability::Gnome)
        {
            crate::core::entity::monster::Faction::Gnome
        } else if template.has_capability(crate::core::entity::capability::MonsterCapability::Demon)
        {
            crate::core::entity::monster::Faction::Demon
        } else if template
            .has_capability(crate::core::entity::capability::MonsterCapability::Undead)
        {
            crate::core::entity::monster::Faction::Undead
        } else if template
            .has_capability(crate::core::entity::capability::MonsterCapability::Animal)
        {
            crate::core::entity::monster::Faction::Animal
        } else if template.symbol == '@' {
            if template.name.contains("soldier") || template.name.contains("watchman") {
                crate::core::entity::monster::Faction::Soldier
            } else {
                crate::core::entity::monster::Faction::Human
            }
        } else {
            crate::core::entity::monster::Faction::None
        };

        let entity = world.push((
            MonsterTag,
            Species {
                current: template.name.clone(),
                original: template.name.clone(),
                timer: None,
            },
            Talkative,
            Monster {
                kind: crate::generated::MonsterKind::from_str(&template.name),
                hostile: !template
                    .has_capability(crate::core::entity::capability::MonsterCapability::Peaceful),
                mon_name: None,
            },
            Position {
                x: x as i32,
                y: y as i32,
            },
            Renderable {
                glyph: template.symbol,
                color: template.color,
            },
            Health {
                current: hp,
                max: hp,
            },
            CombatStats {
                ac: template.ac as i32,
                level: m_lev,
            },
        ));

        if let Some(mut entry) = world.entry(entity) {
            entry.add_component(Inventory::new());
            entry.add_component(Equipment::new());
            entry.add_component(crate::core::entity::monster::MonsterState::new());
            entry.add_component(crate::core::entity::Level(level_id));
            entry.add_component(crate::core::entity::monster::MonsterFaction {
                faction,
                leader: None,
            });
        }
        Self::m_initweap(entity, template, items, world, rng);
        Self::m_initinv(entity, template, items, world, rng);
        if mmflags & MM_NOGRP == 0 {
            let geno = template.geno;
            if (geno & crate::core::entity::monster::G_SGROUP) != 0
                || (geno & crate::core::entity::monster::G_LGROUP) != 0
            {
                let n = if (geno & crate::core::entity::monster::G_LGROUP) != 0 {
                    6
                } else {
                    3
                };
                Self::m_initgrp(
                    entity, template, x, y, n, mmflags, world, grid, templates, items, rng,
                    level_id,
                );
            }
        }
        Some(entity)
    }

    fn select_monster_by_difficulty<'a>(
        templates: &'a Vec<&MonsterTemplate>,
        depth: i32,
        rng: &mut NetHackRng,
    ) -> Option<&'a MonsterTemplate> {
        if templates.is_empty() {
            return None;
        }
        let mut candidates = Vec::new();
        let mut total_weight = 0;
        for t in templates {
            if (t.geno & crate::core::entity::monster::G_NOGEN) != 0
                || (t.geno & crate::core::entity::monster::G_UNIQ) != 0
            {
                continue;
            }
            if (t.level as i32) > depth + (depth / 10) + 2 {
                continue;
            }
            let weight = (t.geno & crate::core::entity::monster::G_FREQ) as i32 + 1;
            candidates.push((t, weight));
            total_weight += weight;
        }
        if candidates.is_empty() {
            //
            let idx = rng.rn2(templates.len() as i32) as usize;
            return Some(templates[idx]);
        }
        let mut pick = rng.rn2(total_weight);
        for (t, weight) in candidates {
            if pick < weight {
                return Some(t);
            }
            pick -= weight;
        }
        Some(templates[0])
    }

    fn m_initgrp(
        parent: Entity,
        template: &MonsterTemplate,
        x: usize,
        y: usize,
        n: i32,
        mmflags: u32,
        world: &mut World,
        grid: &Grid,
        templates: &Vec<&MonsterTemplate>,
        items: &ItemManager,
        rng: &mut NetHackRng,
        level_id: crate::core::dungeon::LevelID,
    ) {
        let cnt = rng.rnz(1, n);
        for _ in 0..cnt {
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = (x as i32 + dx) as usize;
                    let ny = (y as i32 + dy) as usize;
                    if nx < COLNO && ny < ROWNO {
                        if let Some(tile) = grid.get_tile(nx, ny) {
                            if tile.typ == TileType::Room {
                                if let Some(child_ent) = Self::makemon(
                                    Some(template),
                                    nx,
                                    ny,
                                    mmflags | MM_NOGRP,
                                    world,
                                    grid,
                                    templates,
                                    items,
                                    rng,
                                    level_id,
                                ) {
                                    if let Some(mut entry) = world.entry(child_ent) {
                                        if let Ok(f) = entry.get_component_mut::<crate::core::entity::monster::MonsterFaction>() { f.leader = Some(parent); }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn mkgold(
        _amount: i32,
        x: usize,
        y: usize,
        world: &mut World,
        level_id: crate::core::dungeon::LevelID,
    ) -> Entity {
        world.push((
            ItemTag,
            Item {
                kind: crate::generated::ItemKind::from_str("gold piece"),
                price: 1,
                weight: 0,
                unpaid: false,
                spe: 0,
                blessed: false,
                cursed: false,
                bknown: false,
                known: true,
                dknown: true,
                oeroded: 0,
                oeroded2: 0,
                quantity: 1,
                corpsenm: None,
                age: 0,
                oeaten: 0,
                olocked: false,
                oopened: false,
                user_name: None,
                artifact: None,
                owet: 0,
            },
            Renderable {
                glyph: '$',
                color: 14,
            },
            Position {
                x: x as i32,
                y: y as i32,
            },
            crate::core::entity::Level(level_id),
        ))
    }

    pub fn mksobj_at(
        template_name: &str,
        x: usize,
        y: usize,
        items: &ItemManager,
        world: &mut World,
        level_id: crate::core::dungeon::LevelID,
    ) -> Option<Entity> {
        if let Some(entity) = Self::mksobj(template_name, items, world) {
            if let Some(mut entry) = world.entry(entity) {
                entry.add_component(Position {
                    x: x as i32,
                    y: y as i32,
                });
                entry.add_component(crate::core::entity::Level(level_id));
            }
            return Some(entity);
        }
        None
    }

    pub fn mkobj_of_class(
        class: crate::core::entity::object::ItemClass,
        items: &ItemManager,
        world: &mut World,
        rng: &mut NetHackRng,
    ) -> Option<Entity> {
        use crate::core::entity::object::ItemClass;
        let target_class = if class == ItemClass::Random {
            match rng.rn2(10) {
                0..=1 => ItemClass::Weapon,
                2..=3 => ItemClass::Armor,
                4..=5 => ItemClass::Food,
                6 => ItemClass::Potion,
                7 => ItemClass::Scroll,
                8 => ItemClass::Wand,
                9 => ItemClass::Ring,
                _ => ItemClass::Tool,
            }
        } else {
            class
        };
        let candidates = items.get_templates_by_class(target_class);
        if candidates.is_empty() {
            return None;
        }
        let total_prob: i32 = candidates.iter().map(|t| t.prob as i32).sum();
        if total_prob <= 0 {
            let idx = rng.rn2(candidates.len() as i32) as usize;
            return Self::mksobj(&candidates[idx].name, items, world);
        }
        let mut pick = rng.rn2(total_prob);
        for t in &candidates {
            if pick < t.prob as i32 {
                return Self::mksobj(&t.name, items, world);
            }
            pick -= t.prob as i32;
        }
        Self::mksobj(&candidates[0].name, items, world)
    }

    pub fn makemon_of_class(
        symbol: char,
        x: usize,
        y: usize,
        mmflags: u32,
        world: &mut World,
        grid: &Grid,
        templates: &Vec<&MonsterTemplate>,
        items: &ItemManager,
        rng: &mut NetHackRng,
        level_id: crate::core::dungeon::LevelID,
    ) -> Option<Entity> {
        let candidates: Vec<&&MonsterTemplate> =
            templates.iter().filter(|t| t.symbol == symbol).collect();
        if candidates.is_empty() {
            return None;
        }
        let candidate_structs: Vec<&MonsterTemplate> = candidates.into_iter().map(|t| *t).collect();
        let template =
            match Self::select_monster_by_difficulty(&candidate_structs, level_id.depth, rng) {
                Some(t) => t,
                None => return None,
            };
        Self::makemon(
            Some(template),
            x,
            y,
            mmflags,
            world,
            grid,
            templates,
            items,
            rng,
            level_id,
        )
    }

    pub fn mksobj(template_name: &str, items: &ItemManager, world: &mut World) -> Option<Entity> {
        if let Some(template) = items.get_template(template_name) {
            let entity = world.push((
                ItemTag,
                Item {
                    kind: crate::generated::ItemKind::from_str(&template.name),
                    price: template.cost as u32,
                    weight: template.weight as u32,
                    unpaid: false,
                    spe: 0,
                    blessed: false,
                    cursed: false,
                    bknown: false,
                    known: false,
                    dknown: true,
                    oeroded: 0,
                    oeroded2: 0,
                    quantity: 1,
                    corpsenm: None,
                    age: 0,
                    oeaten: 0,
                    olocked: template_name == "chest" || template_name == "large box",
                    oopened: false,
                    user_name: None,
                    artifact: None,
                    owet: 0,
                },
                Renderable {
                    glyph: match template.class {
                        crate::core::entity::object::ItemClass::Weapon => ')',
                        crate::core::entity::object::ItemClass::Armor => '[',
                        crate::core::entity::object::ItemClass::Food => '%',
                        crate::core::entity::object::ItemClass::Potion => '!',
                        crate::core::entity::object::ItemClass::Scroll => '?',
                        crate::core::entity::object::ItemClass::Wand => '/',
                        crate::core::entity::object::ItemClass::Spellbook => '+',
                        crate::core::entity::object::ItemClass::Ring => '=',
                        crate::core::entity::object::ItemClass::Amulet => '"',
                        crate::core::entity::object::ItemClass::Tool => '(',
                        crate::core::entity::object::ItemClass::Gem => '*',
                        _ => '?',
                    },
                    color: template.color,
                },
            ));
            if template_name == "large box"
                || template_name == "chest"
                || template_name == "ice box"
                || template_name == "sack"
                || template_name == "bag of holding"
            {
                if let Some(mut entry) = world.entry(entity) {
                    entry.add_component(crate::core::entity::ContainerTag);
                    entry.add_component(Inventory::new());
                }
            } else if template_name == "boulder" {
                if let Some(mut entry) = world.entry(entity) {
                    entry.add_component(crate::core::entity::BoulderTag);
                }
            } else if template_name == "statue" {
                if let Some(mut entry) = world.entry(entity) {
                    entry.add_component(crate::core::entity::StatueTag);
                }
            } else if template_name == "lamp" {
                if let Some(mut entry) = world.entry(entity) {
                    entry.add_component(LightSource {
                        range: 3,
                        lit: false,
                    });
                    if let Ok(item) = entry.get_component_mut::<Item>() {
                        item.age = 500;
                    }
                }
            }
            return Some(entity);
        }
        None
    }

    pub fn mongets(
        mtmp_ent: Entity,
        otmp_name: &str,
        items: &ItemManager,
        world: &mut World,
    ) -> Option<Entity> {
        if let Some(otmp_ent) = Self::mksobj(otmp_name, items, world) {
            use legion::EntityStore;
            if let Ok(mut entry) = world.entry_mut(mtmp_ent) {
                if let Ok(inventory) = entry.get_component_mut::<Inventory>() {
                    inventory.items.push(otmp_ent);
                }
            }
            return Some(otmp_ent);
        }
        None
    }

    pub fn m_initweap(
        mtmp_ent: Entity,
        template: &MonsterTemplate,
        items: &ItemManager,
        world: &mut World,
        rng: &mut NetHackRng,
    ) {
        let mlet = template.symbol;
        let mm = &template.name;
        if mlet == 'H' {
            if rng.rn2(2) == 0 {
                Self::mongets(
                    mtmp_ent,
                    if mm == "ettin" { "club" } else { "boulder" },
                    items,
                    world,
                );
            }
        }
        if mlet == '@' {
            if mm.contains("soldier") || mm.contains("watchman") {
                if rng.rn2(3) == 0 {
                    Self::mongets(mtmp_ent, "spear", items, world);
                } else {
                    Self::mongets(mtmp_ent, "short sword", items, world);
                }
            } else if mm.contains("elf") {
                Self::mongets(mtmp_ent, "elven dagger", items, world);
                if rng.rn2(3) == 0 {
                    Self::mongets(mtmp_ent, "elven bow", items, world);
                }
            }
        }
        if mlet == 'o' {
            if mm.contains("Uruk-hai") {
                Self::mongets(mtmp_ent, "orcish short sword", items, world);
                Self::mongets(mtmp_ent, "uruk-hai shield", items, world);
            } else {
                Self::mongets(mtmp_ent, "scimitar", items, world);
                Self::mongets(mtmp_ent, "orcish shield", items, world);
            }
        }
        if mlet == '&' {
            if mm == "balrog" {
                Self::mongets(mtmp_ent, "bullwhip", items, world);
                Self::mongets(mtmp_ent, "broadsword", items, world);
            } else if mm == "horned devil" {
                Self::mongets(mtmp_ent, "trident", items, world);
            }
        }
        if mlet == 'Z' {
            if rng.rn2(4) == 0 {
                Self::mongets(mtmp_ent, "knife", items, world);
            }
        }
    }

    pub fn m_initinv(
        mtmp_ent: Entity,
        template: &MonsterTemplate,
        items: &ItemManager,
        world: &mut World,
        rng: &mut NetHackRng,
    ) {
        let mlet = template.symbol;
        let mm = &template.name;
        if mlet == '@' {
            if mm.contains("soldier") {
                Self::mongets(mtmp_ent, "leather armor", items, world);
            } else if mm.contains("sergeant") {
                Self::mongets(mtmp_ent, "chain mail", items, world);
            } else if mm.contains("captain") {
                Self::mongets(mtmp_ent, "plate mail", items, world);
            }
        }
        if mm.contains("elf") {
            if rng.rn2(2) == 0 {
                Self::mongets(mtmp_ent, "elven cloak", items, world);
            }
        }
        if mlet == '@' || mlet == 'h' {
            if rng.rn2(3) == 0 {
                Self::mongets(mtmp_ent, "food ration", items, world);
            }
        }
        if mlet == 'n' {
            if rng.rn2(2) == 0 {
                Self::mongets(mtmp_ent, "mirror", items, world);
            }
        }
    }
}
