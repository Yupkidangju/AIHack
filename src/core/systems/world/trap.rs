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
use crate::core::entity::{Health, MonsterTag, PlayerTag, Position, Trap, TrapType};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

///
#[legion::system]
#[read_component(Position)]
#[read_component(PlayerTag)]
#[read_component(MonsterTag)]
#[read_component(crate::core::entity::Level)]
#[read_component(crate::core::entity::Inventory)]
#[read_component(crate::core::entity::Equipment)]
#[write_component(Trap)]
#[write_component(Health)]
#[write_component(crate::core::entity::Item)]
pub fn trap_trigger(
    world: &mut SubWorld,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] rng: &mut NetHackRng,
    #[resource] item_mgr: &crate::core::entity::object::ItemManager,
    #[resource] tele_action: &mut Option<crate::core::systems::teleport::TeleportAction>,
    #[resource] level_change: &mut Option<crate::core::dungeon::LevelChange>,
    command_buffer: &mut CommandBuffer,
) {
    //
    let mut targets = Vec::new();

    //
    let mut p_query = <(Entity, &Position, &crate::core::entity::Level)>::query()
        .filter(component::<PlayerTag>());
    for (ent, pos, level) in p_query.iter(world) {
        targets.push((*ent, *pos, true, *level)); // (entity, position, is_player, level)
    }

    //
    let mut m_query = <(Entity, &Position, &crate::core::entity::Level)>::query()
        .filter(component::<MonsterTag>());
    for (ent, pos, level) in m_query.iter(world) {
        targets.push((*ent, *pos, false, *level));
    }

    //
    let mut traps = Vec::new();
    let mut t_query = <(Entity, &Position, &Trap, &crate::core::entity::Level)>::query();
    for (ent, pos, trap, lvl) in t_query.iter(world) {
        traps.push((*ent, *pos, *trap, *lvl));
    }

    //
    for (t_ent, t_pos, t_data, t_lvl) in traps {
        for (target_ent, target_pos, is_player, level) in &targets {
            //
            if t_pos.x == target_pos.x && t_pos.y == target_pos.y && t_lvl.0 == level.0 {
                trigger_trap(
                    t_ent,
                    t_data,
                    *target_ent,
                    *is_player,
                    level.0, // current level id
                    world,
                    log,
                    *turn,
                    rng,
                    item_mgr,
                    tele_action,
                    level_change,
                    command_buffer,
                );
            }
        }
    }
}

fn trigger_trap(
    t_ent: Entity,
    trap: Trap,
    target: Entity,
    is_player: bool,
    current_level: crate::core::dungeon::LevelID,
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
    item_mgr: &crate::core::entity::object::ItemManager,
    tele_action: &mut Option<crate::core::systems::teleport::TeleportAction>,
    level_change: &mut Option<crate::core::dungeon::LevelChange>,
    command_buffer: &mut CommandBuffer,
) {
    //
    if !trap.discovered {
        seetrap(t_ent, world);
    }

    match trap.typ {
        TrapType::NoTrap => {}
        TrapType::Arrow => {
            if is_player {
                log.add("An arrow shoots out at you!", turn);
            } else {
                log.add("An arrow shoots out at the monster!", turn);
            }

            //
            if rng.rn2(2) == 0 {
                let dmg = rng.rn1(6, 1); // 1d6
                apply_damage(target, dmg, world, log, turn, is_player);
            } else {
                if is_player {
                    log.add("The arrow misses you.", turn);
                }
            }
        }
        TrapType::Dart => {
            if is_player {
                log.add("A little dart shoots out at you!", turn);
            }
            //
            if rng.rn2(2) == 0 {
                let dmg = rng.rn1(3, 1); // 1d3
                apply_damage(target, dmg, world, log, turn, is_player);
                //
                if rng.rn2(6) == 0 {
                    if is_player {
                        log.add_colored("The dart was poisoned!", [255, 100, 100], turn);
                    }
                }
            } else {
                if is_player {
                    log.add("The dart misses you.", turn);
                }
            }
        }
        TrapType::Rock => {
            let dmg = rng.d(2, 6);
            if is_player {
                log.add("A rock falls from the ceiling and hits you!", turn);
            }
            apply_damage(target, dmg, world, log, turn, is_player);
        }
        TrapType::SqueakyBoard => {
            seetrap(t_ent, world);
            if is_player {
                log.add("A board beneath you squeaks loudly.", turn);
            } else {
                log.add("You hear a squeak.", turn);
            }
            //
        }
        TrapType::BearTrap => {
            if is_player {
                log.add("A bear trap snaps shut on your leg!", turn);
            }
            let dmg = rng.rn1(4, 1);
            apply_damage(target, dmg, world, log, turn, is_player);
            //
        }
        TrapType::Landmine => {
            if is_player {
                log.add_colored(
                    "KAABLAMM!!! You stepped on a land mine!",
                    [255, 100, 0],
                    turn,
                );
            }
            let dmg = rng.d(4, 8);
            apply_damage(target, dmg, world, log, turn, is_player);
            //
            command_buffer.remove(t_ent);
        }
        TrapType::RollingBoulder => {
            if is_player {
                log.add_colored("A large boulder rolls towards you!", [200, 200, 200], turn);
            }
            let dmg = rng.d(3, 10);
            apply_damage(target, dmg, world, log, turn, is_player);
        }
        TrapType::SleepGas => {
            if is_player {
                log.add("A cloud of gas puts you to sleep!", turn);
                if let Ok(mut entry) = world.entry_mut(target) {
                    if let Ok(status) =
                        entry.get_component_mut::<crate::core::entity::status::StatusBundle>()
                    {
                        status.add(
                            crate::core::entity::status::StatusFlags::SLEEPING,
                            rng.rn1(50, 10) as u32,
                        );
                    }
                }
            }
        }
        TrapType::Rust => {
            if is_player {
                log.add("A gush of water hits you!", turn);
                damage_random_equipment(target, world, item_mgr, log, turn, true, false, rng);
            }
        }
        TrapType::Fire => {
            if is_player {
                log.add("A burst of flame hits you!", turn);
            }
            let dmg = rng.rn1(10, 5);
            apply_damage(target, dmg, world, log, turn, is_player);
            if is_player {
                damage_random_equipment(target, world, item_mgr, log, turn, false, true, rng);
            }
        }
        TrapType::Pit | TrapType::SpikedPit => {
            if is_player {
                log.add(
                    format!(
                        "You fall into a {}pit!",
                        if trap.typ == TrapType::SpikedPit {
                            "spiked "
                        } else {
                            ""
                        }
                    ),
                    turn,
                );
            } else {
                log.add("The monster falls into a pit!", turn);
            }
            let dice = if trap.typ == TrapType::SpikedPit {
                (2, 10)
            } else {
                (2, 6)
            };
            let dmg = rng.d(dice.0, dice.1);
            apply_damage(target, dmg, world, log, turn, is_player);

            if trap.typ == TrapType::SpikedPit && rng.rn2(6) == 0 {
                if is_player {
                    log.add_colored("The spikes were poisoned!", [255, 100, 100], turn);
                }
            }
        }
        TrapType::Hole | TrapType::TrapDoor => {
            let msg = if trap.typ == TrapType::TrapDoor {
                "A trap door opens under your feet!"
            } else {
                "You fall through a hole in the floor!"
            };

            if is_player {
                log.add_colored(msg, [255, 0, 0], turn);
                let next_depth = current_level.depth + 1;
                if next_depth <= 50 {
                    *level_change = Some(crate::core::dungeon::LevelChange::Teleport {
                        target: crate::core::dungeon::LevelID {
                            branch: current_level.branch,
                            depth: next_depth,
                        },
                        landing: crate::core::dungeon::LandingType::Random,
                    });
                }
            } else {
                if trap.typ == TrapType::TrapDoor {
                    log.add("A trap door opens under the monster!", turn);
                } else {
                    log.add("The monster falls through a hole!", turn);
                }
            }
        }
        TrapType::Teleport => {
            if is_player {
                log.add("A strange energy envelops you!", turn);
            }
            *tele_action = Some(crate::core::systems::teleport::TeleportAction {
                target,
                is_level_tele: false,
            });
        }
        TrapType::LevelTeleport => {
            if is_player {
                log.add("You feel a wrenching sensation!", turn);
            }
            *tele_action = Some(crate::core::systems::teleport::TeleportAction {
                target,
                is_level_tele: true,
            });
        }
        TrapType::MagicPortal | TrapType::VibratingSquare => {
            if is_player {
                log.add("Nothing happens.", turn);
            }
        }
        TrapType::Web => {
            if is_player {
                log.add("You are caught in a spider web!", turn);
            }
            //
        }
        TrapType::Statue => {
            // Statue trap activation logic (animate_statue in C)
            if is_player {
                log.add("The statue comes to life!", turn);
            }
        }
        TrapType::Magic => {
            if is_player {
                log.add("You are caught in a magical explosion!", turn);
                let dmg = rng.d(1, 10);
                apply_damage(target, dmg, world, log, turn, is_player);
            }
        }
        TrapType::AntiMagic => {
            if is_player {
                log.add("You feel a sudden loss of energy!", turn);
                //
            }
        }
        TrapType::Polymorph => {
            if is_player {
                log.add("You feel a change coming over you!", turn);
                // TODO: Polymorph ?ㅽ뻾
            }
        }
    }
}

fn damage_random_equipment(
    target: Entity,
    world: &mut SubWorld,
    item_mgr: &crate::core::entity::object::ItemManager,
    log: &mut GameLog,
    turn: u64,
    do_rust: bool,
    do_fire: bool,
    rng: &mut NetHackRng,
) {
    use crate::core::entity::{Equipment, Item};
    use crate::core::systems::item_damage::ItemDamageSystem;

    //
    //
    if let Ok(entry) = world.entry_ref(target) {
        if let Ok(equip) = entry.get_component::<Equipment>() {
            let slots: Vec<_> = equip.slots.values().cloned().collect();
            if slots.is_empty() {
                return;
            }

            let item_ent = slots[rng.rn2(slots.len() as i32) as usize];
            drop(entry);

            if let Ok(mut item_entry) = world.entry_mut(item_ent) {
                if let Ok(item) = item_entry.get_component_mut::<Item>() {
                    if let Some(template) = item_mgr.get_by_kind(item.kind) {
                        if do_rust {
                            ItemDamageSystem::rust_item(item, template, log, turn, true);
                        }
                        if do_fire {
                            ItemDamageSystem::burn_item(item, template, log, turn, true);
                        }
                    }
                }
            }
        }
    }
}

fn apply_damage(
    target: Entity,
    dmg: i32,
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
    is_player: bool,
) {
    if let Ok(mut entry) = world.entry_mut(target) {
        if let Ok(health) = entry.get_component_mut::<Health>() {
            health.current -= dmg;
            if is_player {
                log.add(format!("You take {} damage!", dmg), turn);
            } else {
                log.add(format!("The monster takes {} damage!", dmg), turn);
            }
        }
    }
}

fn seetrap(t_ent: Entity, world: &mut SubWorld) {
    if let Ok(mut entry) = world.entry_mut(t_ent) {
        if let Ok(t_comp) = entry.get_component_mut::<Trap>() {
            t_comp.discovered = true;
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
#[derive(Debug, Clone)]
pub struct TrapInfo {
    ///
    pub typ: TrapType,
    ///
    pub name: &'static str,
    ///
    pub symbol: char,
    ///
    pub detect_difficulty: i32,
    ///
    pub disarm_difficulty: i32,
    ///
    pub base_damage: (i32, i32),
    ///
    pub is_magical: bool,
}

///
pub fn trap_info_table() -> Vec<TrapInfo> {
    vec![
        TrapInfo {
            typ: TrapType::Arrow,
            name: "arrow trap",
            symbol: '^',
            detect_difficulty: 2,
            disarm_difficulty: 3,
            base_damage: (1, 6),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::Dart,
            name: "dart trap",
            symbol: '^',
            detect_difficulty: 2,
            disarm_difficulty: 3,
            base_damage: (1, 3),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::Rock,
            name: "falling rock trap",
            symbol: '^',
            detect_difficulty: 3,
            disarm_difficulty: 5,
            base_damage: (2, 6),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::SqueakyBoard,
            name: "squeaky board",
            symbol: '^',
            detect_difficulty: 1,
            disarm_difficulty: 2,
            base_damage: (0, 0),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::BearTrap,
            name: "bear trap",
            symbol: '^',
            detect_difficulty: 3,
            disarm_difficulty: 4,
            base_damage: (1, 4),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::Landmine,
            name: "land mine",
            symbol: '^',
            detect_difficulty: 4,
            disarm_difficulty: 7,
            base_damage: (4, 8),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::RollingBoulder,
            name: "rolling boulder trap",
            symbol: '^',
            detect_difficulty: 5,
            disarm_difficulty: 6,
            base_damage: (3, 10),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::SleepGas,
            name: "sleeping gas trap",
            symbol: '^',
            detect_difficulty: 3,
            disarm_difficulty: 4,
            base_damage: (0, 0),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::Rust,
            name: "rust trap",
            symbol: '^',
            detect_difficulty: 3,
            disarm_difficulty: 4,
            base_damage: (0, 0),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::Fire,
            name: "fire trap",
            symbol: '^',
            detect_difficulty: 4,
            disarm_difficulty: 5,
            base_damage: (2, 4),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::Pit,
            name: "pit",
            symbol: '^',
            detect_difficulty: 2,
            disarm_difficulty: 3,
            base_damage: (2, 6),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::SpikedPit,
            name: "spiked pit",
            symbol: '^',
            detect_difficulty: 3,
            disarm_difficulty: 4,
            base_damage: (2, 10),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::Hole,
            name: "hole",
            symbol: '^',
            detect_difficulty: 4,
            disarm_difficulty: 6,
            base_damage: (1, 1),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::TrapDoor,
            name: "trap door",
            symbol: '^',
            detect_difficulty: 5,
            disarm_difficulty: 7,
            base_damage: (1, 1),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::Teleport,
            name: "teleportation trap",
            symbol: '^',
            detect_difficulty: 5,
            disarm_difficulty: 8,
            base_damage: (0, 0),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::LevelTeleport,
            name: "level teleporter",
            symbol: '^',
            detect_difficulty: 7,
            disarm_difficulty: 10,
            base_damage: (0, 0),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::Web,
            name: "web",
            symbol: '"',
            detect_difficulty: 2,
            disarm_difficulty: 3,
            base_damage: (0, 0),
            is_magical: false,
        },
        TrapInfo {
            typ: TrapType::Statue,
            name: "statue trap",
            symbol: '`',
            detect_difficulty: 6,
            disarm_difficulty: 8,
            base_damage: (0, 0),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::Magic,
            name: "magic trap",
            symbol: '^',
            detect_difficulty: 5,
            disarm_difficulty: 7,
            base_damage: (1, 10),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::AntiMagic,
            name: "anti-magic field",
            symbol: '^',
            detect_difficulty: 6,
            disarm_difficulty: 9,
            base_damage: (0, 0),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::Polymorph,
            name: "polymorph trap",
            symbol: '^',
            detect_difficulty: 6,
            disarm_difficulty: 9,
            base_damage: (0, 0),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::MagicPortal,
            name: "magic portal",
            symbol: '\\',
            detect_difficulty: 10,
            disarm_difficulty: 10,
            base_damage: (0, 0),
            is_magical: true,
        },
        TrapInfo {
            typ: TrapType::VibratingSquare,
            name: "vibrating square",
            symbol: '^',
            detect_difficulty: 10,
            disarm_difficulty: 10,
            base_damage: (0, 0),
            is_magical: true,
        },
    ]
}

///
pub fn get_trap_info(typ: TrapType) -> Option<TrapInfo> {
    trap_info_table().into_iter().find(|t| t.typ == typ)
}

///
pub fn trap_name(typ: TrapType) -> &'static str {
    match typ {
        TrapType::NoTrap => "no trap",
        TrapType::Arrow => "arrow trap",
        TrapType::Dart => "dart trap",
        TrapType::Rock => "falling rock trap",
        TrapType::SqueakyBoard => "squeaky board",
        TrapType::BearTrap => "bear trap",
        TrapType::Landmine => "land mine",
        TrapType::RollingBoulder => "rolling boulder trap",
        TrapType::SleepGas => "sleeping gas trap",
        TrapType::Rust => "rust trap",
        TrapType::Fire => "fire trap",
        TrapType::Pit => "pit",
        TrapType::SpikedPit => "spiked pit",
        TrapType::Hole => "hole",
        TrapType::TrapDoor => "trap door",
        TrapType::Teleport => "teleportation trap",
        TrapType::LevelTeleport => "level teleporter",
        TrapType::Web => "web",
        TrapType::Statue => "statue trap",
        TrapType::Magic => "magic trap",
        TrapType::AntiMagic => "anti-magic field",
        TrapType::Polymorph => "polymorph trap",
        TrapType::MagicPortal => "magic portal",
        TrapType::VibratingSquare => "vibrating square",
    }
}

///
///
pub fn try_detect_trap(
    trap_typ: TrapType,
    dexterity: i32,
    perception_bonus: i32,
    is_rogue: bool,
    rng: &mut NetHackRng,
) -> bool {
    let info = match get_trap_info(trap_typ) {
        Some(i) => i,
        None => return false,
    };

    //
    let mut threshold = info.detect_difficulty;
    if is_rogue {
        threshold -= 3;
    }
    threshold -= perception_bonus;

    let roll = rng.rn2(10) + 1;
    roll + dexterity / 3 > threshold
}

///
///
pub fn try_disarm_trap(
    trap_typ: TrapType,
    dexterity: i32,
    player_level: i32,
    has_tools: bool,
    rng: &mut NetHackRng,
) -> DisarmResult {
    let info = match get_trap_info(trap_typ) {
        Some(i) => i,
        None => return DisarmResult::Failed,
    };

    let mut difficulty = info.disarm_difficulty;
    if has_tools {
        difficulty -= 3;
    }
    //
    if info.is_magical {
        difficulty += 2;
    }

    let roll = rng.rn2(20) + 1 + dexterity / 2 + player_level / 3;

    if roll >= difficulty + 10 {
        DisarmResult::Success
    } else if roll >= difficulty {
        DisarmResult::Partial
    } else if roll < difficulty - 5 {
        DisarmResult::Triggered
    } else {
        DisarmResult::Failed
    }
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisarmResult {
    ///
    Success,
    ///
    Partial,
    ///
    Failed,
    ///
    Triggered,
}

///
pub fn can_place_trap(x: i32, y: i32, is_walkable: bool, has_trap: bool, is_door: bool) -> bool {
    is_walkable && !has_trap && !is_door
}

///
pub fn random_trap_type(dungeon_depth: i32, rng: &mut NetHackRng) -> TrapType {
    //
    let pool: Vec<TrapType> = if dungeon_depth <= 5 {
        vec![
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::SqueakyBoard,
            TrapType::Pit,
            TrapType::BearTrap,
            TrapType::Web,
        ]
    } else if dungeon_depth <= 15 {
        vec![
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::Rock,
            TrapType::SqueakyBoard,
            TrapType::BearTrap,
            TrapType::Pit,
            TrapType::SpikedPit,
            TrapType::SleepGas,
            TrapType::Rust,
            TrapType::Fire,
            TrapType::Web,
            TrapType::Landmine,
        ]
    } else if dungeon_depth <= 30 {
        vec![
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::Rock,
            TrapType::BearTrap,
            TrapType::Pit,
            TrapType::SpikedPit,
            TrapType::Landmine,
            TrapType::SleepGas,
            TrapType::Rust,
            TrapType::Fire,
            TrapType::Web,
            TrapType::Teleport,
            TrapType::Hole,
            TrapType::TrapDoor,
            TrapType::RollingBoulder,
            TrapType::Magic,
            TrapType::AntiMagic,
        ]
    } else {
        vec![
            TrapType::Arrow,
            TrapType::Dart,
            TrapType::Rock,
            TrapType::BearTrap,
            TrapType::SpikedPit,
            TrapType::Landmine,
            TrapType::SleepGas,
            TrapType::Fire,
            TrapType::Teleport,
            TrapType::LevelTeleport,
            TrapType::Hole,
            TrapType::TrapDoor,
            TrapType::RollingBoulder,
            TrapType::Magic,
            TrapType::AntiMagic,
            TrapType::Polymorph,
        ]
    };

    let idx = rng.rn2(pool.len() as i32) as usize;
    pool[idx]
}

///
///
pub fn traps_per_level(depth: i32, rng: &mut NetHackRng) -> i32 {
    //
    let base = (depth / 3 + 1).max(1);
    rng.rn1(5, base)
}

///
pub fn fall_damage(trap_typ: TrapType, con_mod: i32, rng: &mut NetHackRng) -> i32 {
    let base = match trap_typ {
        TrapType::Pit => rng.d(2, 6),
        TrapType::SpikedPit => rng.d(2, 10),
        TrapType::Hole | TrapType::TrapDoor => rng.d(1, 6) + 2,
        _ => 0,
    };
    //
    (base - con_mod / 4).max(1)
}

///
///
pub fn trap_resist_check(
    trap_typ: TrapType,
    has_fire_res: bool,
    has_cold_res: bool,
    has_sleep_res: bool,
    has_poison_res: bool,
    has_magic_res: bool,
    is_flying: bool,
    is_levitating: bool,
) -> bool {
    match trap_typ {
        TrapType::Fire => has_fire_res,
        TrapType::SleepGas => has_sleep_res,
        TrapType::Dart => has_poison_res,
        TrapType::Magic => has_magic_res,
        TrapType::AntiMagic => has_magic_res,
        TrapType::Pit | TrapType::SpikedPit => is_flying || is_levitating,
        TrapType::Hole | TrapType::TrapDoor => is_flying || is_levitating,
        TrapType::BearTrap => is_flying || is_levitating,
        TrapType::Landmine => is_flying || is_levitating,
        TrapType::Web => false,
        _ => false,
    }
}

///
pub fn bear_trap_escape(strength: i32, rng: &mut NetHackRng) -> bool {
    //
    rng.rn2(20) < strength
}

///
pub fn web_escape(strength: i32, is_spider: bool, rng: &mut NetHackRng) -> bool {
    if is_spider {
        return true;
    }
    rng.rn2(15) < strength
}

///
pub fn disarm_message(trap_typ: TrapType, result: DisarmResult) -> String {
    let name = trap_name(trap_typ);
    match result {
        DisarmResult::Success => format!("You disarm the {}!", name),
        DisarmResult::Partial => format!("You partially disarm the {}, but it remains.", name),
        DisarmResult::Failed => format!("You fail to disarm the {}.", name),
        DisarmResult::Triggered => format!("You set off the {} while trying to disarm it!", name),
    }
}

///
pub fn trap_warning_message(trap_typ: TrapType) -> &'static str {
    match trap_typ {
        TrapType::Arrow => "You notice an arrow trap here!",
        TrapType::Dart => "You notice a dart trap here!",
        TrapType::Rock => "You notice a falling rock trap here!",
        TrapType::SqueakyBoard => "You notice a squeaky board here.",
        TrapType::BearTrap => "You see a bear trap here!",
        TrapType::Landmine => "You see a land mine here!",
        TrapType::RollingBoulder => "You see a rolling boulder trap!",
        TrapType::SleepGas => "You see a sleeping gas trap here!",
        TrapType::Rust => "You see a rust trap here.",
        TrapType::Fire => "You see a fire trap here!",
        TrapType::Pit => "You see a pit here!",
        TrapType::SpikedPit => "You see a spiked pit here!",
        TrapType::Hole => "You see a hole here!",
        TrapType::TrapDoor => "You see a trap door here!",
        TrapType::Teleport => "You see a teleportation trap here!",
        TrapType::LevelTeleport => "You see a level teleporter here!",
        TrapType::Web => "You see a web here.",
        TrapType::Statue => "You see a statue trap here!",
        TrapType::Magic => "You see a magic trap here!",
        TrapType::AntiMagic => "You feel an anti-magic field!",
        TrapType::Polymorph => "You see a polymorph trap here!",
        TrapType::MagicPortal => "You see a magic portal here!",
        TrapType::VibratingSquare => "You feel a strange vibration under your feet.",
        TrapType::NoTrap => "",
    }
}

///
pub fn generate_traps_for_level(
    depth: i32,
    width: i32,
    height: i32,
    rng: &mut NetHackRng,
) -> Vec<(i32, i32, TrapType)> {
    let count = traps_per_level(depth, rng);
    let mut result = Vec::new();

    for _ in 0..count {
        let x = rng.rn2(width - 2) + 1;
        let y = rng.rn2(height - 2) + 1;
        let typ = random_trap_type(depth, rng);
        result.push((x, y, typ));
    }
    result
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn trap_base_damage(trap_type: TrapType, depth: i32, rng: &mut NetHackRng) -> i32 {
    match trap_type {
        //
        TrapType::Arrow => {
            let base = rng.rn2(6) + 1;
            let depth_bonus = (depth / 4).min(5);
            base + depth_bonus
        }
        //
        TrapType::Dart => rng.rn2(3) + 1,
        //
        TrapType::Rock => {
            let d1 = rng.rn2(6) + 1;
            let d2 = rng.rn2(6) + 1;
            d1 + d2
        }
        //
        TrapType::SqueakyBoard => 0,
        //
        TrapType::BearTrap => {
            let d1 = rng.rn2(4) + 1;
            let d2 = rng.rn2(4) + 1;
            d1 + d2
        }
        //
        TrapType::Landmine => {
            let mut dmg = 0;
            for _ in 0..4 {
                dmg += rng.rn2(6) + 1;
            }
            dmg
        }
        //
        TrapType::Pit => {
            let d1 = rng.rn2(6) + 1;
            let d2 = rng.rn2(6) + 1;
            d1 + d2
        }
        //
        TrapType::SpikedPit => {
            let mut dmg = 0;
            for _ in 0..3 {
                dmg += rng.rn2(6) + 1;
            }
            dmg
        }
        //
        TrapType::Hole | TrapType::TrapDoor => rng.rn2(6) + 1,
        //
        TrapType::Fire => {
            let d1 = rng.rn2(4) + 1;
            let d2 = rng.rn2(4) + 1;
            d1 + d2 + depth / 2
        }
        //
        TrapType::Rust => 0,
        //
        TrapType::Magic => rng.rn2(10) + 5,
        //
        TrapType::SleepGas => 0,
        //
        TrapType::Teleport | TrapType::LevelTeleport => 0,
        //
        TrapType::Web => 0,
        //
        TrapType::AntiMagic => 0,
        //
        TrapType::Polymorph => 0,
        //
        TrapType::VibratingSquare => 0,
        //
        TrapType::Statue => 0,
        //
        TrapType::MagicPortal => 0,
        //
        TrapType::RollingBoulder => {
            let mut dmg = 0;
            for _ in 0..3 {
                dmg += rng.rn2(6) + 1;
            }
            dmg
        }
        //
        TrapType::NoTrap => 0,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn trap_evasion_check(
    dexterity: i32,
    luck: i32,
    trap_type: TrapType,
    is_flying: bool,
    is_levitating: bool,
    rng: &mut NetHackRng,
) -> bool {
    //
    let ground_trap = matches!(
        trap_type,
        TrapType::BearTrap
            | TrapType::Landmine
            | TrapType::Pit
            | TrapType::SpikedPit
            | TrapType::SqueakyBoard
            | TrapType::Web
            | TrapType::VibratingSquare
    );

    if ground_trap && (is_flying || is_levitating) {
        return true;
    }

    //
    let difficulty = trap_evasion_difficulty(trap_type);
    let roll = rng.rn2(20) + 1 + dexterity / 3 + luck;
    roll > difficulty
}

///
pub fn trap_evasion_difficulty(trap_type: TrapType) -> i32 {
    match trap_type {
        TrapType::Arrow => 12,
        TrapType::Dart => 10,
        TrapType::Rock => 8,
        TrapType::SqueakyBoard => 5,
        TrapType::BearTrap => 14,
        TrapType::Landmine => 16,
        TrapType::Pit => 8,
        TrapType::SpikedPit => 10,
        TrapType::Hole => 12,
        TrapType::TrapDoor => 12,
        TrapType::Fire => 14,
        TrapType::Rust => 10,
        TrapType::Magic => 18,
        TrapType::SleepGas => 12,
        TrapType::Teleport => 20,
        TrapType::LevelTeleport => 22,
        TrapType::Web => 10,
        TrapType::AntiMagic => 15,
        TrapType::Polymorph => 20,
        TrapType::VibratingSquare => 0,
        TrapType::RollingBoulder => 14,
        TrapType::Statue => 0,
        TrapType::MagicPortal => 0,
        TrapType::NoTrap => 0,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapResistance {
    Immune,
    PartialResist,
    Vulnerable,
}

///
pub fn trap_resistance_check(
    trap_type: TrapType,
    fire_res: bool,
    poison_res: bool,
    sleep_res: bool,
    shock_res: bool,
    magic_res: bool,
) -> TrapResistance {
    match trap_type {
        TrapType::Fire => {
            if fire_res {
                TrapResistance::Immune
            } else {
                TrapResistance::Vulnerable
            }
        }
        TrapType::Dart => {
            //
            if poison_res {
                TrapResistance::PartialResist
            } else {
                TrapResistance::Vulnerable
            }
        }
        TrapType::SleepGas => {
            if sleep_res {
                TrapResistance::Immune
            } else {
                TrapResistance::Vulnerable
            }
        }
        TrapType::Magic | TrapType::AntiMagic => {
            if magic_res {
                TrapResistance::PartialResist
            } else {
                TrapResistance::Vulnerable
            }
        }
        TrapType::Rust => {
            //
            TrapResistance::Vulnerable
        }
        _ => TrapResistance::Vulnerable,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn trap_chain_reaction(trap_type: TrapType) -> bool {
    //
    matches!(trap_type, TrapType::Landmine)
}

///
pub fn chain_reaction_radius(trap_type: TrapType) -> i32 {
    match trap_type {
        TrapType::Landmine => 3,
        _ => 0,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagicTrapEffect {
    ShowerOfMissiles,
    TowerOfFlame,
    PoolOfWater,
    BlindingFlash,
    FindGold,
    FindObject,
    CreateMonster,
    Teleportation,
    Nothing,
}

///
pub fn roll_magic_trap_effect(rng: &mut NetHackRng) -> MagicTrapEffect {
    match rng.rn2(20) {
        0..=2 => MagicTrapEffect::ShowerOfMissiles,
        3..=4 => MagicTrapEffect::TowerOfFlame,
        5 => MagicTrapEffect::PoolOfWater,
        6..=7 => MagicTrapEffect::BlindingFlash,
        8..=9 => MagicTrapEffect::FindGold,
        10..=11 => MagicTrapEffect::FindObject,
        12..=14 => MagicTrapEffect::CreateMonster,
        15..=16 => MagicTrapEffect::Teleportation,
        _ => MagicTrapEffect::Nothing,
    }
}

///
pub fn magic_trap_effect_damage(effect: MagicTrapEffect, depth: i32, rng: &mut NetHackRng) -> i32 {
    match effect {
        MagicTrapEffect::ShowerOfMissiles => rng.rn2(10) + 5 + depth / 3,
        MagicTrapEffect::TowerOfFlame => rng.rn2(12) + 8,
        MagicTrapEffect::BlindingFlash => 0,
        _ => 0,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn level_teleport_destination(
    current_depth: i32,
    max_depth: i32,
    has_control: bool,
    desired_depth: Option<i32>,
    rng: &mut NetHackRng,
) -> i32 {
    if has_control {
        if let Some(desired) = desired_depth {
            return desired.clamp(1, max_depth);
        }
    }
    //
    let offset = rng.rn2(max_depth.max(1)) - max_depth / 2;
    (current_depth + offset).clamp(1, max_depth)
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct TrapStatistics {
    pub traps_placed: u32,
    pub traps_triggered: u32,
    pub traps_evaded: u32,
    pub traps_disarmed: u32,
    pub total_damage_taken: i32,
    pub deaths_by_trap: u32,
}

impl TrapStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_trigger(&mut self, damage: i32) {
        self.traps_triggered += 1;
        self.total_damage_taken += damage;
    }

    pub fn record_evasion(&mut self) {
        self.traps_evaded += 1;
    }

    pub fn record_disarm(&mut self) {
        self.traps_disarmed += 1;
    }

    pub fn record_death(&mut self) {
        self.deaths_by_trap += 1;
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn trap_weight_table(depth: i32) -> Vec<(TrapType, i32)> {
    let mut table = vec![
        (TrapType::Arrow, 20),
        (TrapType::Dart, 20),
        (TrapType::Rock, 15),
        (TrapType::SqueakyBoard, 10),
        (TrapType::BearTrap, 10),
        (TrapType::Pit, 15),
        (TrapType::SpikedPit, 10),
        (TrapType::Web, 5),
    ];

    if depth >= 5 {
        table.push((TrapType::Landmine, 5));
        table.push((TrapType::Fire, 8));
        table.push((TrapType::Rust, 5));
    }
    if depth >= 8 {
        table.push((TrapType::Hole, 5));
        table.push((TrapType::TrapDoor, 5));
        table.push((TrapType::SleepGas, 8));
    }
    if depth >= 12 {
        table.push((TrapType::Teleport, 5));
        table.push((TrapType::Magic, 3));
        table.push((TrapType::AntiMagic, 3));
    }
    if depth >= 16 {
        table.push((TrapType::LevelTeleport, 3));
        table.push((TrapType::Polymorph, 2));
        table.push((TrapType::RollingBoulder, 5));
    }

    table
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct MonsterTrapResult {
    pub damage: i32,
    pub killed: bool,
    pub status_effect: Option<String>,
    pub message: String,
}

///
pub fn monster_trap_damage(
    trap_type: TrapType,
    monster_name: &str,
    monster_hp: i32,
    depth: i32,
    fire_res: bool,
    poison_res: bool,
    rng: &mut NetHackRng,
) -> MonsterTrapResult {
    let base_dmg = trap_base_damage(trap_type, depth, rng);

    //
    let adjusted_dmg = match trap_type {
        TrapType::Fire if fire_res => 0,
        TrapType::Dart if poison_res => base_dmg,
        _ => base_dmg,
    };

    let killed = adjusted_dmg >= monster_hp;
    let status = match trap_type {
        TrapType::BearTrap => Some("trapped".into()),
        TrapType::Web => Some("stuck".into()),
        TrapType::SleepGas if !poison_res => Some("asleep".into()),
        _ => None,
    };

    let msg = if killed {
        format!("{} is killed by a trap!", monster_name)
    } else if adjusted_dmg > 0 {
        format!(
            "{} triggers a trap! ({} damage)",
            monster_name, adjusted_dmg
        )
    } else {
        format!("{} triggers a trap but is unharmed.", monster_name)
    };

    MonsterTrapResult {
        damage: adjusted_dmg,
        killed,
        status_effect: status,
        message: msg,
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn trap_detection_difficulty(trap_type: TrapType) -> i32 {
    match trap_type {
        TrapType::Arrow => 15,
        TrapType::Dart => 12,
        TrapType::Rock => 10,
        TrapType::SqueakyBoard => 8,
        TrapType::BearTrap => 14,
        TrapType::Landmine => 20,
        TrapType::Pit => 10,
        TrapType::SpikedPit => 12,
        TrapType::Hole => 18,
        TrapType::TrapDoor => 18,
        TrapType::Fire => 16,
        TrapType::Rust => 14,
        TrapType::Magic => 22,
        TrapType::SleepGas => 16,
        TrapType::Teleport => 20,
        TrapType::LevelTeleport => 24,
        TrapType::Web => 8,
        TrapType::AntiMagic => 20,
        TrapType::Polymorph => 22,
        TrapType::VibratingSquare => 5,
        TrapType::RollingBoulder => 12,
        TrapType::Statue => 10,
        TrapType::MagicPortal => 25,
        TrapType::NoTrap => 0,
    }
}

///
pub fn trap_disarm_difficulty(trap_type: TrapType) -> i32 {
    match trap_type {
        TrapType::Arrow => 10,
        TrapType::Dart => 8,
        TrapType::Rock => 15,
        TrapType::SqueakyBoard => 3,
        TrapType::BearTrap => 12,
        TrapType::Landmine => 18,
        TrapType::Pit => 5,
        TrapType::SpikedPit => 8,
        TrapType::Web => 6,
        TrapType::Fire => 14,
        TrapType::Rust => 10,
        TrapType::Magic => 16,
        TrapType::SleepGas => 10,
        TrapType::Teleport => 20,
        TrapType::LevelTeleport => 20,
        TrapType::Hole => 15,
        TrapType::TrapDoor => 15,
        TrapType::AntiMagic => 22,
        TrapType::Polymorph => 22,
        TrapType::VibratingSquare => 0,
        TrapType::RollingBoulder => 12,
        TrapType::Statue => 8,
        TrapType::MagicPortal => 25,
        TrapType::NoTrap => 0,
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod trap_extended_tests {
    use super::*;

    #[test]
    fn test_trap_evasion_flying() {
        let mut rng = NetHackRng::new(42);
        //
        assert!(trap_evasion_check(
            10,
            0,
            TrapType::BearTrap,
            true,
            false,
            &mut rng
        ));
    }

    #[test]
    fn test_trap_resistance() {
        let r = trap_resistance_check(TrapType::Fire, true, false, false, false, false);
        assert_eq!(r, TrapResistance::Immune);

        let r2 = trap_resistance_check(TrapType::SleepGas, false, false, true, false, false);
        assert_eq!(r2, TrapResistance::Immune);
    }

    #[test]
    fn test_trap_damage_range() {
        let mut rng = NetHackRng::new(42);
        //
        for _ in 0..50 {
            let dmg = trap_base_damage(TrapType::Arrow, 1, &mut rng);
            assert!(dmg >= 1);
        }
        //
        assert_eq!(trap_base_damage(TrapType::SqueakyBoard, 10, &mut rng), 0);
    }

    #[test]
    fn test_chain_reaction() {
        assert!(trap_chain_reaction(TrapType::Landmine));
        assert!(!trap_chain_reaction(TrapType::Arrow));
    }

    #[test]
    fn test_magic_trap_effect() {
        let mut rng = NetHackRng::new(42);
        let effect = roll_magic_trap_effect(&mut rng);
        //
        let dmg = magic_trap_effect_damage(effect, 10, &mut rng);
        assert!(dmg >= 0);
    }

    #[test]
    fn test_trap_statistics() {
        let mut stats = TrapStatistics::new();
        stats.record_trigger(10);
        stats.record_trigger(5);
        stats.record_evasion();
        assert_eq!(stats.traps_triggered, 2);
        assert_eq!(stats.total_damage_taken, 15);
        assert_eq!(stats.traps_evaded, 1);
    }

    #[test]
    fn test_trap_weight_table() {
        let shallow = trap_weight_table(1);
        let deep = trap_weight_table(20);
        assert!(deep.len() > shallow.len());
    }

    #[test]
    fn test_monster_trap_damage() {
        let mut rng = NetHackRng::new(42);
        let result = monster_trap_damage(TrapType::BearTrap, "orc", 10, 5, false, false, &mut rng);
        assert!(result.damage > 0);
        assert_eq!(result.status_effect, Some("trapped".into()));
    }

    #[test]
    fn test_trap_detect_difficulty() {
        //
        assert!(
            trap_detection_difficulty(TrapType::Landmine)
                > trap_detection_difficulty(TrapType::SqueakyBoard)
        );
    }

    #[test]
    fn test_level_teleport() {
        let mut rng = NetHackRng::new(42);
        //
        let dest = level_teleport_destination(5, 30, true, Some(10), &mut rng);
        assert_eq!(dest, 10);
        //
        let dest2 = level_teleport_destination(5, 30, false, None, &mut rng);
        assert!(dest2 >= 1 && dest2 <= 30);
    }
}

// =============================================================================
// [v2.3.4
// =============================================================================

///
pub fn trap_trigger_message(trap_type: TrapType) -> &'static str {
    match trap_type {
        TrapType::Arrow => "An arrow shoots out at you!",
        TrapType::Dart => "A little dart shoots out at you!",
        TrapType::Rock => "A rock falls on your head!",
        TrapType::SqueakyBoard => "A board beneath you squeaks loudly!",
        TrapType::BearTrap => "A bear trap closes on your foot!",
        TrapType::Landmine => "KAABLAMM!!! You triggered a land mine!",
        TrapType::RollingBoulder => "Click! You trigger a rolling boulder trap!",
        TrapType::SleepGas => "A cloud of gas puts you to sleep!",
        TrapType::Rust => "A gush of water hits you!",
        TrapType::Fire => "A tower of flame erupts beneath you!",
        TrapType::Pit => "You fall into a pit!",
        TrapType::SpikedPit => "You fall into a pit with spikes!",
        TrapType::Hole => "You fall through a hole in the floor!",
        TrapType::TrapDoor => "A trap door opens up under you!",
        TrapType::Teleport => "You feel a wrenching sensation!",
        TrapType::LevelTeleport => "You are momentarily blinded by a flash of light!",
        TrapType::Web => "You are caught in a web!",
        TrapType::Statue => "A statue comes to life!",
        TrapType::MagicPortal => "You feel dizzy for a moment, but it passes.",
        TrapType::AntiMagic => "You feel your magical energy drain away!",
        TrapType::Polymorph => "You feel a change coming over you!",
        TrapType::VibratingSquare => "You feel a strange vibration under your feet.",
        TrapType::Magic => "You are caught in a magical explosion!",
        TrapType::NoTrap => "",
    }
}

///
pub fn trap_disarm_xp(trap_type: TrapType, player_level: i32) -> i32 {
    let base = match trap_type {
        TrapType::Landmine => 6,
        TrapType::BearTrap => 3,
        TrapType::Fire => 5,
        TrapType::SleepGas => 4,
        TrapType::Dart | TrapType::Arrow => 2,
        TrapType::SpikedPit | TrapType::Pit => 3,
        TrapType::Web => 1,
        TrapType::Magic | TrapType::AntiMagic => 8,
        TrapType::Polymorph => 10,
        _ => 1,
    };
    //
    (base - player_level / 5).max(1)
}

///
pub fn trap_disarm_parts(trap_type: TrapType) -> Vec<&'static str> {
    match trap_type {
        TrapType::Arrow => vec!["arrow"],
        TrapType::Dart => vec!["dart"],
        TrapType::BearTrap => vec!["bear trap"],
        TrapType::Landmine => vec!["land mine"],
        TrapType::RollingBoulder => vec!["boulder"],
        TrapType::Web => vec!["spider web"],
        _ => vec![],
    }
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapImmunity {
    Immune,
    Resistant,
    Vulnerable,
    Normal,
}

///
pub fn check_trap_immunity(
    trap_type: TrapType,
    is_flying: bool,
    is_levitating: bool,
    fire_resistant: bool,
    sleep_resistant: bool,
    poison_resistant: bool,
    shock_resistant: bool,
    magic_resistant: bool,
) -> TrapImmunity {
    match trap_type {
        TrapType::Pit | TrapType::SpikedPit | TrapType::Hole | TrapType::TrapDoor => {
            if is_flying || is_levitating {
                TrapImmunity::Immune
            } else {
                TrapImmunity::Normal
            }
        }
        TrapType::Fire => {
            if fire_resistant {
                TrapImmunity::Resistant
            } else {
                TrapImmunity::Normal
            }
        }
        TrapType::SleepGas => {
            if sleep_resistant {
                TrapImmunity::Immune
            } else {
                TrapImmunity::Normal
            }
        }
        TrapType::Dart => {
            if poison_resistant {
                TrapImmunity::Resistant
            } else {
                TrapImmunity::Normal
            }
        }
        TrapType::SqueakyBoard => {
            if is_levitating || is_flying {
                TrapImmunity::Immune
            } else {
                TrapImmunity::Normal
            }
        }
        TrapType::BearTrap => {
            if is_flying {
                TrapImmunity::Immune
            } else {
                TrapImmunity::Normal
            }
        }
        TrapType::Magic | TrapType::AntiMagic => {
            if magic_resistant {
                TrapImmunity::Resistant
            } else {
                TrapImmunity::Normal
            }
        }
        TrapType::Web => {
            //
            if fire_resistant {
                TrapImmunity::Resistant
            } else {
                TrapImmunity::Normal
            }
        }
        TrapType::Rust => {
            //
            TrapImmunity::Normal
        }
        TrapType::Landmine => {
            if is_flying || is_levitating {
                TrapImmunity::Immune
            } else {
                TrapImmunity::Vulnerable
            }
        }
        _ => TrapImmunity::Normal,
    }
}

///
pub fn trap_creation_cost(trap_type: TrapType) -> i32 {
    match trap_type {
        TrapType::Landmine => 150,
        TrapType::BearTrap => 80,
        TrapType::Arrow => 30,
        TrapType::Dart => 20,
        TrapType::Fire => 200,
        TrapType::SleepGas => 120,
        TrapType::Pit => 50,
        TrapType::SpikedPit => 70,
        TrapType::Web => 40,
        TrapType::RollingBoulder => 100,
        _ => 0,
    }
}

///
pub fn trap_chain_tiles(trap_type: TrapType, rng: &mut NetHackRng) -> Vec<(i32, i32)> {
    //
    match trap_type {
        TrapType::Landmine => {
            //
            let mut chain = Vec::new();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    if rng.rn2(4) == 0 {
                        chain.push((dx, dy));
                    }
                }
            }
            chain
        }
        TrapType::Fire => {
            //
            let mut chain = Vec::new();
            for dx in -1..=1 {
                for dy in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    if rng.rn2(6) == 0 {
                        chain.push((dx, dy));
                    }
                }
            }
            chain
        }
        _ => Vec::new(),
    }
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrapTransformEffect {
    None,
    Polymorph,
    LevelTeleport,
    Teleport,
    MagicDrain,
    StatChange(i32),
    ItemChange,
}

///
pub fn magic_trap_effect(rng: &mut NetHackRng) -> TrapTransformEffect {
    match rng.rn2(12) {
        0 => TrapTransformEffect::Polymorph,
        1 => TrapTransformEffect::Teleport,
        2 => TrapTransformEffect::LevelTeleport,
        3 => TrapTransformEffect::MagicDrain,
        4 => TrapTransformEffect::StatChange(1),
        5 => TrapTransformEffect::StatChange(-1),
        6 => TrapTransformEffect::ItemChange,
        _ => TrapTransformEffect::None,
    }
}

///
pub fn trap_disarm_tool_bonus(tool_name: &str) -> i32 {
    let l = tool_name.to_lowercase();
    if l.contains("tinning kit") {
        return 3;
    }
    if l.contains("pick-axe") || l.contains("mattock") {
        return 5;
    }
    if l.contains("dagger") || l.contains("knife") {
        return 2;
    }
    if l.contains("stethoscope") {
        return 4;
    }
    0
}

///
pub fn max_traps_per_level(dungeon_depth: i32) -> i32 {
    //
    let base = 20 + dungeon_depth;
    base.min(60)
}

///
#[derive(Debug, Clone, Default)]
pub struct ExtendedTrapStatistics {
    pub traps_disarmed: u32,
    pub traps_triggered: u32,
    pub chain_reactions: u32,
    pub xp_from_disarm: i32,
    pub parts_recovered: u32,
    pub immunities_used: u32,
    pub magic_effects: u32,
}

impl ExtendedTrapStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_trigger(&mut self) {
        self.traps_triggered += 1;
    }
    pub fn record_disarm(&mut self, xp: i32) {
        self.traps_disarmed += 1;
        self.xp_from_disarm += xp;
    }
    pub fn record_chain(&mut self) {
        self.chain_reactions += 1;
    }
}

#[cfg(test)]
mod trap_extended2_tests {
    use super::*;

    #[test]
    fn test_trigger_message() {
        let m = trap_trigger_message(TrapType::BearTrap);
        assert!(m.contains("bear trap"));
    }

    #[test]
    fn test_disarm_xp() {
        assert!(trap_disarm_xp(TrapType::Landmine, 1) > trap_disarm_xp(TrapType::Web, 1));
    }

    #[test]
    fn test_disarm_parts() {
        let parts = trap_disarm_parts(TrapType::Arrow);
        assert!(parts.contains(&"arrow"));
    }

    #[test]
    fn test_immunity() {
        let r = check_trap_immunity(
            TrapType::Pit,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        );
        assert_eq!(r, TrapImmunity::Immune);
    }

    #[test]
    fn test_creation_cost() {
        assert!(trap_creation_cost(TrapType::Landmine) > trap_creation_cost(TrapType::Dart));
    }

    #[test]
    fn test_chain_reaction() {
        let mut rng = NetHackRng::new(42);
        let chain = trap_chain_tiles(TrapType::Landmine, &mut rng);
        //
        assert!(chain.len() <= 8);
    }

    #[test]
    fn test_magic_effect() {
        let mut rng = NetHackRng::new(42);
        let e = magic_trap_effect(&mut rng);
        //
        let _ = e;
    }

    #[test]
    fn test_tool_bonus() {
        assert_eq!(trap_disarm_tool_bonus("pick-axe"), 5);
        assert_eq!(trap_disarm_tool_bonus("random"), 0);
    }

    #[test]
    fn test_max_traps() {
        assert!(max_traps_per_level(1) < max_traps_per_level(30));
        assert!(max_traps_per_level(100) <= 60);
    }

    #[test]
    fn test_extended_stats() {
        let mut s = ExtendedTrapStatistics::new();
        s.record_trigger();
        s.record_disarm(5);
        s.record_chain();
        assert_eq!(s.traps_triggered, 1);
        assert_eq!(s.traps_disarmed, 1);
        assert_eq!(s.xp_from_disarm, 5);
    }
}
