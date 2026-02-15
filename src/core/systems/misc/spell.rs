// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::Grid;
use crate::core::entity::{
    player::Player, Health, MonsterTag, PlayerTag, Position, SpellKnowledge,
};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

///
#[derive(Debug, Clone)]
pub struct CastAction {
    pub spell_key: char,
    pub direction: Option<crate::core::game_state::Direction>,
}

///
#[legion::system]
#[read_component(PlayerTag)]
#[write_component(Player)]
#[read_component(SpellKnowledge)]
#[read_component(Position)]
#[write_component(Health)]
#[read_component(MonsterTag)]
pub fn spell_cast(
    world: &mut SubWorld,
    #[resource] cast_action: &mut Option<CastAction>,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] grid: &Grid,
) {
    let action = match cast_action.take() {
        Some(a) => a,
        None => return,
    };

    let spell_key = action.spell_key;

    //
    let mut spell_to_cast = None;
    let mut p_pos_copy = None;

    let mut p_query =
        <(&Player, &SpellKnowledge, &Position)>::query().filter(component::<PlayerTag>());
    for (p_stats, p_knowledge, p_pos) in p_query.iter(world) {
        if let Some(spell) = p_knowledge.spells.get(&spell_key) {
            let cost = spell.level * 5;
            if p_stats.energy >= cost {
                spell_to_cast = Some((spell.name.clone(), cost));
                p_pos_copy = Some(*p_pos);
            } else {
                log.add("You don't have enough energy to cast that spell.", *turn);
                return;
            }
        } else {
            log.add(
                format!("You don't know any spell assigned to '{}'.", spell_key),
                *turn,
            );
            return;
        }
    }

    //
    if let Some((spell_name, cost)) = spell_to_cast {
        //
        let mut p_stats_query = <&mut Player>::query().filter(component::<PlayerTag>());
        for p_stats in p_stats_query.iter_mut(world) {
            p_stats.energy -= cost;
        }

        log.add(format!("You cast {}.", spell_name), *turn);

        //
        if let Some(p_pos) = p_pos_copy {
            execute_spell_effect(
                &spell_name,
                &p_pos,
                world,
                log,
                *turn,
                action.direction,
                grid,
            );
        }
    }
}

fn execute_spell_effect(
    name: &str,
    p_pos: &Position,
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
    direction: Option<crate::core::game_state::Direction>,
    grid: &Grid,
) {
    match name {
        "Force Bolt" | "force bolt" | "Spellbook of force bolt" => {
            if let Some(dir) = direction {
                let (dx, dy) = dir.to_delta();
                let mut curr_x = p_pos.x;
                let mut curr_y = p_pos.y;
                let mut hit_something = false;

                for _ in 0..10 {
                    //
                    curr_x += dx;
                    curr_y += dy;

                    //
                    if curr_x < 0 || curr_x >= 80 || curr_y < 0 || curr_y >= 21 {
                        break;
                    }

                    //
                    if let Some(tile) = grid.get_tile(curr_x as usize, curr_y as usize) {
                        if tile.typ.is_wall() {
                            log.add("The force bolt hits a wall.", turn);
                            hit_something = true;
                            break;
                        }
                    }

                    //
                    let mut monster_query =
                        <(&Position, &mut Health)>::query().filter(component::<MonsterTag>());
                    let mut hit_monster = false;

                    for (m_pos, m_health) in monster_query.iter_mut(world) {
                        if m_pos.x == curr_x && m_pos.y == curr_y {
                            let dmg = 10;
                            m_health.current -= dmg;
                            log.add(
                                format!("The force bolt hits the monster for {} damage!", dmg),
                                turn,
                            );
                            hit_monster = true;
                            break;
                        }
                    }

                    if hit_monster {
                        hit_something = true;
                        break;
                    }
                }

                if !hit_something {
                    log.add("The force bolt dissipates into the air.", turn);
                }
            } else {
                log.add("You must choose a direction to cast this spell!", turn);
            }
        }
        _ => {
            log.add("The spell has no effect yet.", turn);
        }
    }
}
//
pub fn cast_monster_spell(
    m_ent: Entity,
    m_name: &str,
    p_ent: Entity,
    world: &mut SubWorld,
    assets: &crate::assets::AssetManager,
    rng: &mut crate::util::rng::NetHackRng,
    log: &mut GameLog,
    turn: u64,
    _command_buffer: &mut CommandBuffer,
) {
    if let Ok(_m_entry) = world.entry_ref(m_ent) {
        if let Some(template) = assets.monsters.templates.get(m_name) {
            //
            //
            let is_mage = template.msound == 8; // MS_MAGE
            let is_cleric = template.msound == 9; // MS_CLERIC

            if !is_mage && !is_cleric {
                return;
            }

            let _m_lev = template.level as i32;
            let _p_mr = 0; // TODO: Calculate player Magic Resistance

            //
            let spell_num = rng.rn2(if is_mage { 8 } else { 5 });

            if is_mage {
                match spell_num {
                    0..=1 => {
                        // Magic Missile
                        log.add(format!("The {} zaps a magic missile at you!", m_name), turn);
                        if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                            if let Ok(h) = p_entry.get_component_mut::<Health>() {
                                h.current -= rng.d(2, 6);
                            }
                        }
                    }
                    2 => {
                        // Haste Self
                        log.add(format!("The {} speeds up!", m_name), turn);
                        if let Ok(mut m_entry) = world.entry_mut(m_ent) {
                            if let Ok(ps) = m_entry
                                .get_component_mut::<crate::core::entity::status::StatusBundle>()
                            {
                                ps.add(crate::core::entity::status::StatusFlags::FAST, 50);
                            }
                        }
                    }
                    3 => {
                        // Cure Self
                        log.add(format!("The {} looks better.", m_name), turn);
                        if let Ok(mut m_entry) = world.entry_mut(m_ent) {
                            if let Ok(h) = m_entry.get_component_mut::<Health>() {
                                h.current = (h.current + rng.d(2, 8)).min(h.max);
                            }
                        }
                    }
                    4 => {
                        // Teleport Away
                        log.add(format!("The {} suddenly disappears!", m_name), turn);
                        // TODO: Implement teleport logic
                    }
                    5 => {
                        // Create Monster
                        log.add(format!("The {} summons help!", m_name), turn);
                        // TODO: Implement summon logic (spawn system call)
                    }
                    _ => {
                        log.add(format!("The {}'s spell dissipates.", m_name), turn);
                    }
                }
            } else {
                // Cleric Spells
                match spell_num {
                    0 => {
                        // Blindness
                        log.add(format!("The {} blinds you!", m_name), turn);
                        if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                            if let Ok(ps) = p_entry
                                .get_component_mut::<crate::core::entity::status::StatusBundle>()
                            {
                                ps.add(crate::core::entity::status::StatusFlags::BLIND, 20);
                            }
                        }
                    }
                    1 => {
                        // Confusion
                        log.add(format!("The {} confuses you!", m_name), turn);
                        if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                            if let Ok(ps) = p_entry
                                .get_component_mut::<crate::core::entity::status::StatusBundle>()
                            {
                                ps.add(crate::core::entity::status::StatusFlags::CONFUSED, 20);
                            }
                        }
                    }
                    2 => {
                        // Cause Wound
                        log.add(
                            format!("The {} strikes you with divine power!", m_name),
                            turn,
                        );
                        if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                            if let Ok(h) = p_entry.get_component_mut::<Health>() {
                                h.current -= rng.d(2, 6);
                            }
                        }
                    }
                    _ => {
                        log.add(format!("The {} prays.", m_name), turn);
                    }
                }
            }
        }
    }
}

// =============================================================================
// [v2.3.1
//
//
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellSchool {
    ///
    Attack,
    ///
    Healing,
    ///
    Divination,
    ///
    Enchantment,
    ///
    Conjuration,
    ///
    Clerical,
    ///
    Escape,
}

///
#[derive(Debug, Clone)]
pub struct SpellData {
    ///
    pub name: &'static str,
    ///
    pub school: SpellSchool,
    ///
    pub level: i32,
    ///
    pub mana_cost: i32,
    ///
    pub directional: bool,
    ///
    pub damage_type: Option<crate::core::entity::monster::DamageType>,
    ///
    pub cast_time: i32,
}

///
pub fn spell_data_table() -> Vec<SpellData> {
    use crate::core::entity::monster::DamageType;
    vec![
        // ===
        SpellData {
            name: "force bolt",
            school: SpellSchool::Attack,
            level: 1,
            mana_cost: 5,
            directional: true,
            damage_type: Some(DamageType::Phys),
            cast_time: 1,
        },
        SpellData {
            name: "magic missile",
            school: SpellSchool::Attack,
            level: 2,
            mana_cost: 10,
            directional: true,
            damage_type: Some(DamageType::Magm),
            cast_time: 1,
        },
        SpellData {
            name: "fireball",
            school: SpellSchool::Attack,
            level: 4,
            mana_cost: 20,
            directional: true,
            damage_type: Some(DamageType::Fire),
            cast_time: 2,
        },
        SpellData {
            name: "cone of cold",
            school: SpellSchool::Attack,
            level: 4,
            mana_cost: 20,
            directional: true,
            damage_type: Some(DamageType::Cold),
            cast_time: 2,
        },
        SpellData {
            name: "finger of death",
            school: SpellSchool::Attack,
            level: 7,
            mana_cost: 35,
            directional: true,
            damage_type: Some(DamageType::Deth),
            cast_time: 3,
        },
        SpellData {
            name: "drain life",
            school: SpellSchool::Attack,
            level: 2,
            mana_cost: 10,
            directional: true,
            damage_type: Some(DamageType::Drli),
            cast_time: 1,
        },
        SpellData {
            name: "sleep",
            school: SpellSchool::Enchantment,
            level: 1,
            mana_cost: 5,
            directional: true,
            damage_type: Some(DamageType::Slee),
            cast_time: 1,
        },
        // ===
        SpellData {
            name: "healing",
            school: SpellSchool::Healing,
            level: 1,
            mana_cost: 5,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "extra healing",
            school: SpellSchool::Healing,
            level: 3,
            mana_cost: 15,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "cure blindness",
            school: SpellSchool::Healing,
            level: 2,
            mana_cost: 10,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "cure sickness",
            school: SpellSchool::Healing,
            level: 3,
            mana_cost: 15,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "restore ability",
            school: SpellSchool::Healing,
            level: 4,
            mana_cost: 20,
            directional: false,
            damage_type: None,
            cast_time: 2,
        },
        //
        SpellData {
            name: "detect monsters",
            school: SpellSchool::Divination,
            level: 1,
            mana_cost: 5,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "detect food",
            school: SpellSchool::Divination,
            level: 2,
            mana_cost: 10,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "clairvoyance",
            school: SpellSchool::Divination,
            level: 3,
            mana_cost: 15,
            directional: false,
            damage_type: None,
            cast_time: 2,
        },
        SpellData {
            name: "detect unseen",
            school: SpellSchool::Divination,
            level: 3,
            mana_cost: 15,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "identify",
            school: SpellSchool::Divination,
            level: 5,
            mana_cost: 25,
            directional: false,
            damage_type: None,
            cast_time: 2,
        },
        SpellData {
            name: "magic mapping",
            school: SpellSchool::Divination,
            level: 5,
            mana_cost: 25,
            directional: false,
            damage_type: None,
            cast_time: 2,
        },
        //
        SpellData {
            name: "confuse monster",
            school: SpellSchool::Enchantment,
            level: 2,
            mana_cost: 10,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "slow monster",
            school: SpellSchool::Enchantment,
            level: 2,
            mana_cost: 10,
            directional: true,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "cause fear",
            school: SpellSchool::Enchantment,
            level: 3,
            mana_cost: 15,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "charm monster",
            school: SpellSchool::Enchantment,
            level: 3,
            mana_cost: 15,
            directional: false,
            damage_type: None,
            cast_time: 2,
        },
        //
        SpellData {
            name: "create monster",
            school: SpellSchool::Conjuration,
            level: 2,
            mana_cost: 10,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "create familiar",
            school: SpellSchool::Conjuration,
            level: 6,
            mana_cost: 30,
            directional: false,
            damage_type: None,
            cast_time: 3,
        },
        //
        SpellData {
            name: "protection",
            school: SpellSchool::Clerical,
            level: 1,
            mana_cost: 5,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "remove curse",
            school: SpellSchool::Clerical,
            level: 3,
            mana_cost: 15,
            directional: false,
            damage_type: None,
            cast_time: 2,
        },
        SpellData {
            name: "turn undead",
            school: SpellSchool::Clerical,
            level: 5,
            mana_cost: 25,
            directional: false,
            damage_type: None,
            cast_time: 2,
        },
        SpellData {
            name: "cancellation",
            school: SpellSchool::Clerical,
            level: 7,
            mana_cost: 35,
            directional: true,
            damage_type: None,
            cast_time: 3,
        },
        //
        SpellData {
            name: "jumping",
            school: SpellSchool::Escape,
            level: 1,
            mana_cost: 5,
            directional: true,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "haste self",
            school: SpellSchool::Escape,
            level: 3,
            mana_cost: 15,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "teleport away",
            school: SpellSchool::Escape,
            level: 6,
            mana_cost: 30,
            directional: true,
            damage_type: None,
            cast_time: 2,
        },
        SpellData {
            name: "levitation",
            school: SpellSchool::Escape,
            level: 4,
            mana_cost: 20,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "invisibility",
            school: SpellSchool::Escape,
            level: 4,
            mana_cost: 20,
            directional: false,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "dig",
            school: SpellSchool::Escape,
            level: 5,
            mana_cost: 25,
            directional: true,
            damage_type: None,
            cast_time: 2,
        },
        SpellData {
            name: "knock",
            school: SpellSchool::Escape,
            level: 1,
            mana_cost: 5,
            directional: true,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "wizard lock",
            school: SpellSchool::Escape,
            level: 2,
            mana_cost: 10,
            directional: true,
            damage_type: None,
            cast_time: 1,
        },
        SpellData {
            name: "polymorph",
            school: SpellSchool::Escape,
            level: 6,
            mana_cost: 30,
            directional: true,
            damage_type: Some(crate::core::entity::monster::DamageType::Poly),
            cast_time: 2,
        },
    ]
}

///
pub fn get_spell_data(name: &str) -> Option<SpellData> {
    let lower = name.to_lowercase();
    spell_data_table()
        .into_iter()
        .find(|s| lower.contains(s.name))
}

///
///
pub fn spell_memory_decay(initial_knowledge: i32, turns_since_study: u64) -> i32 {
    //
    let decay_rate = (turns_since_study / 10000) as i32;
    (initial_knowledge - decay_rate * 5).max(0)
}

///
pub fn spell_still_known(knowledge: i32) -> bool {
    knowledge > 0
}

///
pub fn spell_failure_chance(
    spell_level: i32,
    player_exp_level: i32,
    role_bonus: i32,
    wearing_body_armor: bool,
    wearing_shield: bool,
) -> i32 {
    let mut chance = (spell_level * 15) - (player_exp_level * 3) - role_bonus;

    //
    if wearing_body_armor {
        chance += 20;
    }
    if wearing_shield {
        chance += 10;
    }

    chance.clamp(0, 100)
}

///
pub fn role_spell_bonus(role: &str, school: SpellSchool) -> i32 {
    match role {
        "Wizard" => match school {
            SpellSchool::Attack | SpellSchool::Enchantment | SpellSchool::Escape => 20,
            SpellSchool::Divination => 15,
            _ => 10,
        },
        "Priest" | "Priestess" => match school {
            SpellSchool::Clerical | SpellSchool::Healing => 20,
            SpellSchool::Divination => 15,
            _ => 5,
        },
        "Monk" => match school {
            SpellSchool::Healing | SpellSchool::Clerical => 15,
            SpellSchool::Escape => 10,
            _ => 5,
        },
        "Valkyrie" | "Knight" | "Samurai" => -10,
        "Barbarian" => -20,
        _ => 0,
    }
}

///
pub fn spell_failure_effect(spell_level: i32) -> &'static str {
    if spell_level >= 6 {
        "You feel a terrible magical backlash!"
    } else if spell_level >= 4 {
        "You feel a surge of uncontrolled energy!"
    } else if spell_level >= 2 {
        "The spell fizzles."
    } else {
        "Nothing happens."
    }
}

///
pub fn spell_school_color(school: SpellSchool) -> [u8; 3] {
    match school {
        SpellSchool::Attack => [255, 80, 80],
        SpellSchool::Healing => [80, 255, 80],
        SpellSchool::Divination => [80, 80, 255],
        SpellSchool::Enchantment => [255, 200, 80],
        SpellSchool::Conjuration => [200, 80, 255],
        SpellSchool::Clerical => [255, 255, 200],
        SpellSchool::Escape => [80, 255, 255],
    }
}

// =============================================================================
// [v2.3.4
// =============================================================================

///
pub fn spell_memory_turns(spell_level: i32, intelligence: i32) -> i32 {
    //
    let base = spell_level * 10000;
    let int_bonus = (intelligence - 10) * 500;
    (base + int_bonus).max(5000)
}

///
pub fn spell_decay_message(remaining_percent: f32) -> &'static str {
    if remaining_percent <= 10.0 {
        "You barely remember the spell!"
    } else if remaining_percent <= 25.0 {
        "Your knowledge of the spell is fading."
    } else if remaining_percent <= 50.0 {
        "The spell feels less familiar."
    } else {
        ""
    }
}

///
pub fn energy_regen_per_turn(wisdom: i32, level: i32, is_wizard: bool) -> i32 {
    let base = level / 5 + 1;
    let wis_bonus = (wisdom - 10) / 3;
    let role_bonus = if is_wizard { 2 } else { 0 };
    (base + wis_bonus + role_bonus).max(1)
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellBackfire {
    Nothing,
    EnergyDrain,
    SelfDamage,
    Confusion,
    Paralyze,    // 留덈퉬
    BlindSelf,
    Amnesia,
}

///
pub fn spell_backfire_effect(
    spell_level: i32,
    failure_margin: i32,
    rng: &mut NetHackRng,
) -> SpellBackfire {
    //
    if failure_margin < 10 {
        return SpellBackfire::Nothing;
    }
    if failure_margin < 20 {
        return SpellBackfire::EnergyDrain;
    }
    let roll = rng.rn2(100);
    if spell_level >= 7 && roll < 20 {
        return SpellBackfire::Amnesia;
    }
    if spell_level >= 5 && roll < 30 {
        return SpellBackfire::SelfDamage;
    }
    if roll < 40 {
        return SpellBackfire::Confusion;
    }
    if roll < 50 {
        return SpellBackfire::BlindSelf;
    }
    SpellBackfire::EnergyDrain
}

///
pub fn spell_levelup_benefit(school: SpellSchool) -> &'static str {
    match school {
        SpellSchool::Attack => "+1 spell damage per level",
        SpellSchool::Healing => "+2 healing per level",
        SpellSchool::Divination => "+1 detection range per level",
        SpellSchool::Enchantment => "+5% success rate per level",
        SpellSchool::Conjuration => "+1 conjured entity per 3 levels",
        SpellSchool::Clerical => "+1 holy power per level",
        SpellSchool::Escape => "+1 escape radius per level",
    }
}

///
pub fn school_defense_bonus(school: SpellSchool) -> i32 {
    match school {
        SpellSchool::Enchantment => 2,
        SpellSchool::Clerical => 2,
        SpellSchool::Escape => 1,
        SpellSchool::Healing => 1,
        _ => 0,
    }
}

///
pub fn mana_surge_threshold(max_energy: i32) -> i32 {
    (max_energy * 90) / 100
}

///
pub fn mana_surge_message(current: i32, max: i32) -> &'static str {
    if current >= max {
        "Your magical energy is overflowing!"
    } else if current >= mana_surge_threshold(max) {
        "You feel a surge of magical power."
    } else {
        ""
    }
}

///
pub fn spells_conflict(spell_a: SpellSchool, spell_b: SpellSchool) -> bool {
    //
    matches!(
        (spell_a, spell_b),
        (SpellSchool::Healing, SpellSchool::Attack)
            | (SpellSchool::Attack, SpellSchool::Healing)
            | (SpellSchool::Clerical, SpellSchool::Conjuration)
            | (SpellSchool::Conjuration, SpellSchool::Clerical)
    )
}

///
#[derive(Debug, Clone, Default)]
pub struct SpellStatistics {
    pub spells_cast: u32,
    pub spells_failed: u32,
    pub spells_forgotten: u32,
    pub energy_spent: i32,
    pub backfires: u32,
    pub healing_done: i32,
    pub damage_dealt: i32,
}

impl SpellStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_cast(&mut self, energy: i32) {
        self.spells_cast += 1;
        self.energy_spent += energy;
    }
    pub fn record_fail(&mut self) {
        self.spells_failed += 1;
    }
    pub fn record_forget(&mut self) {
        self.spells_forgotten += 1;
    }
    pub fn success_rate(&self) -> f32 {
        if self.spells_cast == 0 {
            0.0
        } else {
            (self.spells_cast - self.spells_failed) as f32 / self.spells_cast as f32
        }
    }
}

#[cfg(test)]
mod spell_extended_tests {
    use super::*;

    #[test]
    fn test_spell_memory() {
        assert!(spell_memory_turns(7, 18) > spell_memory_turns(1, 10));
    }

    #[test]
    fn test_decay_message() {
        assert!(spell_decay_message(5.0).contains("barely"));
        assert_eq!(spell_decay_message(80.0), "");
    }

    #[test]
    fn test_energy_regen() {
        let regen_wiz = energy_regen_per_turn(18, 15, true);
        let regen_war = energy_regen_per_turn(10, 15, false);
        assert!(regen_wiz > regen_war);
    }

    #[test]
    fn test_backfire() {
        let mut rng = NetHackRng::new(42);
        let e = spell_backfire_effect(1, 5, &mut rng);
        assert_eq!(e, SpellBackfire::Nothing);
    }

    #[test]
    fn test_spell_conflict() {
        assert!(spells_conflict(SpellSchool::Healing, SpellSchool::Attack));
        assert!(!spells_conflict(SpellSchool::Attack, SpellSchool::Escape));
    }

    #[test]
    fn test_mana_surge() {
        assert!(mana_surge_message(100, 100).contains("overflowing"));
        assert_eq!(mana_surge_message(50, 100), "");
    }

    #[test]
    fn test_spell_stats() {
        let mut s = SpellStatistics::new();
        s.record_cast(15);
        s.record_fail();
        assert_eq!(s.spells_cast, 1);
        assert_eq!(s.energy_spent, 15);
    }
}
