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
use crate::core::dungeon::{Grid, LevelChange};
use crate::core::entity::{PlayerTag, Position};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeleportAction {
    pub target: Entity,
    pub is_level_tele: bool,
}

#[legion::system]
#[read_component(PlayerTag)]
#[read_component(TeleportAction)]
#[write_component(Position)]
pub fn teleport(
    world: &mut SubWorld,
    #[resource] tele_action_res: &mut Option<TeleportAction>,
    #[resource] log: &mut GameLog,
    #[resource] grid: &Grid,
    #[resource] rng: &mut NetHackRng,
    #[resource] level_req: &mut Option<LevelChange>,
    command_buffer: &mut legion::systems::CommandBuffer,
) {
    //
    let action_opt = tele_action_res.take();

    //
    let action = if let Some(a) = action_opt {
        Some(a)
    } else {
        let mut query = <(Entity, &TeleportAction)>::query();
        let found = query.iter(world).next().map(|(&e, &a)| (e, a));
        if let Some((e, a)) = found {
            command_buffer.remove_component::<TeleportAction>(e);
            Some(a)
        } else {
            None
        }
    };

    let action = match action {
        Some(a) => a,
        None => return,
    };

    let is_player = if let Ok(entry) = world.entry_ref(action.target) {
        entry.get_component::<PlayerTag>().is_ok()
    } else {
        false
    };

    if action.is_level_tele && is_player {
        //
        let current_id = if let Ok(entry) = world.entry_ref(action.target) {
            entry
                .get_component::<crate::core::entity::Level>()
                .map(|l| l.0)
                .unwrap_or(crate::core::dungeon::LevelID::new(
                    crate::core::dungeon::DungeonBranch::Main,
                    1,
                ))
        } else {
            crate::core::dungeon::LevelID::new(crate::core::dungeon::DungeonBranch::Main, 1)
        };

        let delta = rng.rn1(5, 1);
        let next_depth = if rng.rn2(2) == 0 {
            (current_id.depth + delta).min(50)
        } else {
            (current_id.depth - delta).max(1)
        };
        let target_id = crate::core::dungeon::LevelID::new(current_id.branch, next_depth);

        log.add("You are swept away to another level!", log.current_turn);
        *level_req = Some(LevelChange::Teleport {
            target: target_id,
            landing: crate::core::dungeon::LandingType::Random,
        });
    } else {
        //
        let mut success = false;
        let mut target_pos = (0, 0);

        for _ in 0..200 {
            let tx = rng.rn2(crate::core::dungeon::COLNO as i32) as usize;
            let ty = rng.rn2(crate::core::dungeon::ROWNO as i32) as usize;

            if let Some(tile) = grid.get_tile(tx, ty) {
                //
                if !tile.typ.is_wall() {
                    target_pos = (tx as i32, ty as i32);
                    success = true;
                    break;
                }
            }
        }

        if success {
            if let Ok(mut entry) = world.entry_mut(action.target) {
                if let Ok(pos) = entry.get_component_mut::<Position>() {
                    pos.x = target_pos.0;
                    pos.y = target_pos.1;
                }
            }

            if is_player {
                log.add("You teleport!", log.current_turn);
            } else {
                log.add("The monster teleports away!", log.current_turn);
            }
        } else {
            if is_player {
                log.add("You feel a shudder, but stay put.", log.current_turn);
            }
        }
    }
}

// =============================================================================
// [v2.3.1
//
//
//
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeleportControl {
    ///
    None,
    ///
    Directional,
    ///
    Full,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeleportCause {
    ///
    Trap,
    ///
    Magic,
    ///
    Item,
    ///
    Intrinsic,
    ///
    Forced,
    ///
    LevelTrap,
}

///
pub fn can_teleport(
    has_teleport_resist: bool,
    has_amulet: bool,
    in_no_tele_zone: bool,
    is_dead: bool,
) -> bool {
    if is_dead {
        return false;
    }
    if has_amulet {
        return false;
    }
    if in_no_tele_zone {
        return false;
    }
    !has_teleport_resist
}

///
pub fn can_level_teleport(has_amulet: bool, in_quest: bool, in_endgame: bool) -> bool {
    if has_amulet {
        return false;
    }
    if in_endgame {
        return false;
    }
    //
    !in_quest
}

///
///
pub fn find_safe_landing(
    width: usize,
    height: usize,
    is_passable_fn: &dyn Fn(usize, usize) -> bool,
    rng: &mut NetHackRng,
    max_tries: i32,
) -> Option<(i32, i32)> {
    for _ in 0..max_tries {
        let x = rng.rn2((width - 2) as i32) as usize + 1;
        let y = rng.rn2((height - 2) as i32) as usize + 1;
        if is_passable_fn(x, y) {
            return Some((x as i32, y as i32));
        }
    }
    None
}

///
pub fn is_valid_teleport_dest(
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    is_passable_fn: &dyn Fn(i32, i32) -> bool,
) -> bool {
    x >= 0 && y >= 0 && x < width && y < height && is_passable_fn(x, y)
}

///
///
pub fn level_teleport_target(
    current_depth: i32,
    max_depth: i32,
    control: TeleportControl,
    desired_depth: Option<i32>,
    rng: &mut NetHackRng,
) -> i32 {
    match control {
        TeleportControl::Full => {
            //
            let target = desired_depth.unwrap_or(current_depth);
            target.max(1).min(max_depth)
        }
        TeleportControl::Directional => {
            //
            let delta = rng.rn1(5, 1);
            let target = if desired_depth.unwrap_or(1) > current_depth {
                current_depth + delta
            } else {
                current_depth - delta
            };
            target.max(1).min(max_depth)
        }
        TeleportControl::None => {
            //
            rng.rn2(max_depth) + 1
        }
    }
}

///
pub fn teleport_message(cause: TeleportCause, is_level: bool) -> &'static str {
    if is_level {
        match cause {
            TeleportCause::Trap => "You activate a level teleport trap!",
            TeleportCause::Magic => "You are whisked away to another level!",
            TeleportCause::LevelTrap => "You are swept to another level!",
            _ => "You are transported to a different level!",
        }
    } else {
        match cause {
            TeleportCause::Trap => "A strange energy envelops you!",
            TeleportCause::Magic => "You teleport!",
            TeleportCause::Item => "Your ring of teleportation activates!",
            TeleportCause::Intrinsic => "You feel a sudden urge to be elsewhere!",
            _ => "You teleport!",
        }
    }
}

///
///
pub fn monster_should_teleport(
    hp_ratio: f32,
    has_tele_ability: bool,
    is_fleeing: bool,
    rng: &mut NetHackRng,
) -> bool {
    if !has_tele_ability {
        return false;
    }
    if is_fleeing && hp_ratio < 0.25 {
        return rng.rn2(4) == 0;
    }
    if hp_ratio < 0.1 {
        return rng.rn2(2) == 0;
    }
    false
}

///
pub fn teleport_cooldown_remaining(last_tele_turn: u64, current_turn: u64) -> u64 {
    let cooldown = 5;
    if current_turn >= last_tele_turn + cooldown {
        0
    } else {
        last_tele_turn + cooldown - current_turn
    }
}

///
pub fn teleport_failure_message(
    has_amulet: bool,
    in_no_tele_zone: bool,
    on_cooldown: bool,
) -> &'static str {
    if has_amulet {
        "You feel a momentary shimmer, but the Amulet prevents it!"
    } else if in_no_tele_zone {
        "A mysterious force prevents your teleportation!"
    } else if on_cooldown {
        "You feel a shudder, but nothing happens."
    } else {
        "Your teleport attempt fails."
    }
}

///
pub fn can_cross_branch_teleport(
    has_amulet: bool,
    source_branch: crate::core::dungeon::DungeonBranch,
    target_branch: crate::core::dungeon::DungeonBranch,
) -> bool {
    if has_amulet {
        return false;
    }
    //
    source_branch == target_branch
}

///
pub struct TeleportResult {
    ///
    pub success: bool,
    ///
    pub new_pos: (i32, i32),
    /// 硫붿떆吏
    pub message: String,
    ///
    pub level_change: bool,
}

impl TeleportResult {
    pub fn success(x: i32, y: i32, msg: &str) -> Self {
        Self {
            success: true,
            new_pos: (x, y),
            message: msg.to_string(),
            level_change: false,
        }
    }
    pub fn level_tele(depth: i32, msg: &str) -> Self {
        Self {
            success: true,
            new_pos: (0, depth),
            message: msg.to_string(),
            level_change: true,
        }
    }
    pub fn failure(msg: &str) -> Self {
        Self {
            success: false,
            new_pos: (0, 0),
            message: msg.to_string(),
            level_change: false,
        }
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn is_safe_landing(
    x: i32,
    y: i32,
    map_width: i32,
    map_height: i32,
    is_wall: bool,
    is_water: bool,
    is_lava: bool,
    occupied_by_monster: bool,
    has_trap: bool,
) -> bool {
    //
    if x < 1 || x >= map_width - 1 || y < 0 || y >= map_height {
        return false;
    }

    //
    if is_wall {
        return false;
    }

    //
    //
    if occupied_by_monster {
        return false;
    }

    true
}

///
pub fn is_noteleport_zone(depth: i32, is_quest: bool, is_sanctum: bool) -> bool {
    //
    if is_sanctum {
        return true;
    }

    //
    if is_quest && depth > 0 {
        return true;
    }

    false
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn random_teleport_location(
    map_width: i32,
    map_height: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> (i32, i32) {
    //
    let x = rng.rn2(map_width - 2) + 1;
    let y = rng.rn2(map_height - 2) + 1;
    (x, y)
}

///
pub fn monster_can_teleport(
    has_tele_ability: bool,
    on_noteleport_level: bool,
    carrying_amulet: bool,
) -> bool {
    if on_noteleport_level {
        return false;
    }
    if carrying_amulet {
        return false;
    }
    has_tele_ability
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeleResistResult {
    NoResist,
    PartialResist,
    FullResist,
}

///
pub fn check_teleport_resist(
    has_tele_resist: bool,
    magic_resistance: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> TeleResistResult {
    if has_tele_resist {
        return TeleResistResult::FullResist;
    }

    if magic_resistance > 0 {
        let roll = rng.rn2(100);
        if roll < magic_resistance {
            return TeleResistResult::FullResist;
        }
        if roll < magic_resistance * 2 {
            return TeleResistResult::PartialResist;
        }
    }

    TeleResistResult::NoResist
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeleTrapType {
    RandomSameLevel,
    LevelTeleport,
    ControlledTeleport,
}

///
pub fn teleport_trap_effect(
    is_level_tele_trap: bool,
    has_tele_control: bool,
    rng: &mut crate::util::rng::NetHackRng,
) -> TeleTrapType {
    if is_level_tele_trap {
        if has_tele_control {
            return TeleTrapType::ControlledTeleport;
        }
        return TeleTrapType::LevelTeleport;
    }

    if has_tele_control {
        TeleTrapType::ControlledTeleport
    } else {
        TeleTrapType::RandomSameLevel
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn short_teleport_range(
    x: i32,
    y: i32,
    max_distance: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> (i32, i32) {
    let dx = rng.rn2(max_distance * 2 + 1) - max_distance;
    let dy = rng.rn2(max_distance * 2 + 1) - max_distance;
    (x + dx, y + dy)
}

///
pub fn teleport_on_cooldown(last_tele_turn: u64, current_turn: u64, cooldown: u64) -> bool {
    current_turn < last_tele_turn + cooldown
}

///
pub fn teleport_trap_message(tele_type: TeleTrapType, controlled: bool) -> &'static str {
    match tele_type {
        TeleTrapType::RandomSameLevel => "You feel disoriented...",
        TeleTrapType::LevelTeleport => "You feel yourself yanked upward!",
        TeleTrapType::ControlledTeleport => {
            if controlled {
                "You concentrate and teleport!"
            } else {
                "You feel in control of your destination."
            }
        }
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct TeleportStatistics {
    pub times_teleported: u32,
    pub controlled_teleports: u32,
    pub level_teleports: u32,
    pub teleport_traps_triggered: u32,
    pub teleport_resisted: u32,
}

impl TeleportStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_teleport(&mut self, controlled: bool, level_tele: bool) {
        self.times_teleported += 1;
        if controlled {
            self.controlled_teleports += 1;
        }
        if level_tele {
            self.level_teleports += 1;
        }
    }

    pub fn record_trap(&mut self) {
        self.teleport_traps_triggered += 1;
    }

    pub fn record_resist(&mut self) {
        self.teleport_resisted += 1;
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod teleport_extended_tests {
    use super::*;

    #[test]
    fn test_safe_landing() {
        assert!(is_safe_landing(
            5, 5, 80, 25, false, false, false, false, false
        ));
        assert!(!is_safe_landing(
            0, 0, 80, 25, false, false, false, false, false
        ));
        assert!(!is_safe_landing(
            5, 5, 80, 25, true, false, false, false, false
        ));
        assert!(!is_safe_landing(
            5, 5, 80, 25, false, false, false, true, false
        ));
    }

    #[test]
    fn test_noteleport_zone() {
        assert!(is_noteleport_zone(5, false, true));
        assert!(is_noteleport_zone(3, true, false));
        assert!(!is_noteleport_zone(5, false, false));
    }

    #[test]
    fn test_monster_can_teleport() {
        assert!(monster_can_teleport(true, false, false));
        assert!(!monster_can_teleport(true, true, false));
        assert!(!monster_can_teleport(true, false, true));
    }

    #[test]
    fn test_teleport_resist() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let result = check_teleport_resist(true, 0, &mut rng);
        assert_eq!(result, TeleResistResult::FullResist);
    }

    #[test]
    fn test_teleport_trap_effect() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let result = teleport_trap_effect(true, true, &mut rng);
        assert_eq!(result, TeleTrapType::ControlledTeleport);
    }

    #[test]
    fn test_cooldown() {
        assert!(teleport_on_cooldown(100, 105, 10));
        assert!(!teleport_on_cooldown(100, 115, 10));
    }

    #[test]
    fn test_teleport_stats() {
        let mut stats = TeleportStatistics::new();
        stats.record_teleport(true, false);
        stats.record_teleport(false, true);
        stats.record_trap();
        assert_eq!(stats.times_teleported, 2);
        assert_eq!(stats.controlled_teleports, 1);
        assert_eq!(stats.level_teleports, 1);
        assert_eq!(stats.teleport_traps_triggered, 1);
    }
}
