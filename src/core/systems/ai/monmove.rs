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
// =============================================================================
//
//
// [v1.9.0
// =============================================================================
//
//
//
//

use crate::core::dungeon::tile::TileType;
use crate::core::entity::capability::MonsterCapability;
use crate::core::entity::monster::MonsterTemplate;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Moved(i32, i32),
    Attacked,
    Stayed,
    Fled,
    Teleported,
    UsedDoor,
    Died,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterGoal {
    SeekPlayer,
    SeekItem,
    SeekStairs,
    Wander,
    Guard,
    Flee,
    HideAndAmbush,
}

// =============================================================================
//
// =============================================================================

///
pub fn calculate_move_direction(
    mx: i32,
    my: i32,
    tx: i32,
    ty: i32,
) -> (i32, i32) {
    let dx = (tx - mx).signum();
    let dy = (ty - my).signum();
    (dx, dy)
}

///
pub fn can_move_to(tile: TileType, template: &MonsterTemplate) -> bool {
    match tile {
        //
        t if t.is_wall() => template.has_capability(MonsterCapability::WallWalk),
        //
        TileType::Pool | TileType::Moat | TileType::Water => {
            template.has_capability(MonsterCapability::Swim)
                || template.has_capability(MonsterCapability::Fly)
                || template.has_capability(MonsterCapability::Amphibious)
        }
        //
        TileType::LavaPool => {
            template.has_capability(MonsterCapability::Fly) || template.resists_fire()
        }
        //
        TileType::Tree => {
            template.has_capability(MonsterCapability::WallWalk)
                || template.has_capability(MonsterCapability::Amorphous)
        }
        //
        TileType::Air => template.has_capability(MonsterCapability::Fly),
        //
        TileType::IronBars => template.has_capability(MonsterCapability::Amorphous),
        //
        TileType::Door => false,
        //
        TileType::Stone => {
            template.has_capability(MonsterCapability::Tunnel)
                || template.has_capability(MonsterCapability::WallWalk)
        }
        TileType::SDoor | TileType::SCorr => {
            template.has_capability(MonsterCapability::Tunnel)
                || template.has_capability(MonsterCapability::WallWalk)
        }
        //
        _ => true,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn determine_goal(
    hp_pct: i32,
    sees_player: bool,
    is_hostile: bool,
    is_peaceful: bool,
    is_fleeing: bool,
    has_target_item: bool,
    is_guardian: bool,
    _rng: &mut NetHackRng,
) -> MonsterGoal {
    //
    if is_fleeing || hp_pct < 15 {
        return MonsterGoal::Flee;
    }

    //
    if is_guardian {
        return MonsterGoal::Guard;
    }

    //
    if is_hostile && sees_player {
        return MonsterGoal::SeekPlayer;
    }

    //
    if has_target_item {
        return MonsterGoal::SeekItem;
    }

    //
    MonsterGoal::Wander
}

///
pub fn wander_direction(rng: &mut NetHackRng) -> (i32, i32) {
    let dirs = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let idx = rng.rn2(8) as usize;
    dirs[idx]
}

///
pub fn flee_direction(mx: i32, my: i32, px: i32, py: i32) -> (i32, i32) {
    //
    let dx = (mx - px).signum();
    let dy = (my - py).signum();
    (dx, dy)
}

// =============================================================================
//
// =============================================================================

///
pub fn can_open_door(template: &MonsterTemplate) -> bool {
    //
    let nohands = template.has_capability(MonsterCapability::NoHands);
    let mindless = template.has_capability(MonsterCapability::Mindless);
    !nohands && !mindless
}

///
pub fn can_break_door(template: &MonsterTemplate, strength_bonus: i32) -> bool {
    //
    let flags2 = template.flags2;
    let strong = flags2 & 0x04000000 != 0; // STRONG
    let giant = flags2 & 0x00002000 != 0; // GIANT
    strong || giant || strength_bonus > 15
}

// =============================================================================
//
// =============================================================================

///
///
pub fn mcalcmove(speed: i32) -> i32 {
    const NORMAL_SPEED: i32 = 12;
    if speed <= 0 {
        return 0;
    }

    //
    //
    speed / NORMAL_SPEED
}

///
pub fn can_act_this_turn(speed: i32, turn: u64) -> bool {
    const NORMAL_SPEED: i32 = 12;
    if speed >= NORMAL_SPEED {
        true
    } else if speed <= 0 {
        false
    } else {
        //
        (turn as i32 * speed) % NORMAL_SPEED < speed
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn patrol_direction(
    mx: i32,
    my: i32,
    guard_x: i32,
    guard_y: i32,
    rng: &mut NetHackRng,
) -> (i32, i32) {
    let dist = (mx - guard_x).abs() + (my - guard_y).abs();
    if dist <= 1 {
        //
        if rng.rn2(4) == 0 {
            wander_direction(rng)
        } else {
            (0, 0)
        }
    } else {
        //
        calculate_move_direction(mx, my, guard_x, guard_y)
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn can_see_player(
    mx: i32,
    my: i32,
    px: i32,
    py: i32,
    template: &MonsterTemplate,
    player_invisible: bool,
    player_displaced: bool,
    rng: &mut NetHackRng,
) -> bool {
    let dist = (mx - px).abs().max((my - py).abs());

    //
    if dist > 15 {
        return false;
    }

    //
    if player_invisible && !template.has_capability(MonsterCapability::SeeInvis) {
        //
        return dist <= 1;
    }

    //
    if player_displaced && dist > 1 {
        return rng.rn2(3) == 0;
    }

    //
    dist <= 10
}

///
pub fn monster_distance(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x1 - x2).abs().max((y1 - y2).abs())
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_calc() {
        assert_eq!(calculate_move_direction(5, 5, 10, 5), (1, 0));
        assert_eq!(calculate_move_direction(5, 5, 5, 3), (0, -1));
        assert_eq!(calculate_move_direction(5, 5, 8, 8), (1, 1));
    }

    #[test]
    fn test_flee_direction() {
        assert_eq!(flee_direction(5, 5, 3, 3), (1, 1));
        assert_eq!(flee_direction(5, 5, 7, 5), (-1, 0));
    }

    #[test]
    fn test_goal_flee() {
        let mut rng = NetHackRng::new(42);
        let goal = determine_goal(10, true, true, false, false, false, false, &mut rng);
        assert_eq!(goal, MonsterGoal::Flee);
    }

    #[test]
    fn test_goal_seek() {
        let mut rng = NetHackRng::new(42);
        let goal = determine_goal(80, true, true, false, false, false, false, &mut rng);
        assert_eq!(goal, MonsterGoal::SeekPlayer);
    }

    #[test]
    fn test_mcalcmove() {
        assert_eq!(mcalcmove(12), 1);
        assert_eq!(mcalcmove(24), 2);
        assert_eq!(mcalcmove(6), 0);
    }

    #[test]
    fn test_distance() {
        assert_eq!(monster_distance(0, 0, 3, 4), 4);
        assert_eq!(monster_distance(5, 5, 5, 5), 0);
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialMovement {
    Normal,
    Tunnel,
    Phase,
    Swim,
    Fly,
    Burrow,
    Climb,
}

///
pub fn determine_special_movement(template: &MonsterTemplate) -> SpecialMovement {
    use crate::core::entity::capability::MonsterCapability;
    if template.has_capability(MonsterCapability::WallWalk) {
        SpecialMovement::Phase
    } else if template.has_capability(MonsterCapability::Tunnel) {
        SpecialMovement::Tunnel
    } else if template.has_capability(MonsterCapability::Fly) {
        SpecialMovement::Fly
    } else if template.has_capability(MonsterCapability::Swim) {
        SpecialMovement::Swim
    } else if template.has_capability(MonsterCapability::Cling) {
        SpecialMovement::Climb
    } else {
        SpecialMovement::Normal
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn should_hide(
    template: &MonsterTemplate,
    hp_pct: i32,
    sees_player: bool,
    distance: i32,
    rng: &mut NetHackRng,
) -> bool {
    use crate::core::entity::capability::MonsterCapability;
    //
    if !template.has_capability(MonsterCapability::Hide) {
        return false;
    }

    //
    if hp_pct < 30 {
        return true;
    }

    //
    if sees_player && distance <= 2 {
        return rng.rn2(3) == 0;
    }

    //
    if !sees_player {
        return rng.rn2(5) != 0;
    }

    false
}

///
pub fn ambush_attack_bonus(hiding: bool, distance: i32) -> i32 {
    if hiding && distance <= 1 {
        4
    } else {
        0
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn monster_desires_item(
    template: &MonsterTemplate,
    item_is_food: bool,
    item_is_gold: bool,
    item_is_gem: bool,
    item_is_weapon: bool,
) -> bool {
    //
    let is_dragon = template.symbol == 'D';
    if is_dragon && (item_is_gold || item_is_gem) {
        return true;
    }

    //
    let is_leprechaun = template.name.contains("leprechaun");
    if is_leprechaun && item_is_gold {
        return true;
    }

    //
    let is_nymph = template.name.contains("nymph");
    if is_nymph && (item_is_gem || item_is_weapon) {
        return true;
    }

    //
    if item_is_food && !template.name.contains("golem") {
        return true;
    }

    false
}

///
pub fn seek_nearest_item(mx: i32, my: i32, item_positions: &[(i32, i32)]) -> Option<(i32, i32)> {
    if item_positions.is_empty() {
        return None;
    }

    let mut best = item_positions[0];
    let mut best_dist = monster_distance(mx, my, best.0, best.1);

    for &(ix, iy) in &item_positions[1..] {
        let d = monster_distance(mx, my, ix, iy);
        if d < best_dist {
            best = (ix, iy);
            best_dist = d;
        }
    }

    Some(calculate_move_direction(mx, my, best.0, best.1))
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn should_teleport_flee(
    hp_pct: i32,
    has_teleport: bool,
    can_tele_control: bool,
    _rng: &mut NetHackRng,
) -> bool {
    if !has_teleport {
        return false;
    }

    //
    if hp_pct <= 20 {
        return true;
    }

    //
    if can_tele_control && hp_pct <= 30 {
        return true;
    }

    false
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionPriority {
    SpecialAbility,
    MeleeAttack,
    RangedAttack,
    StealAttack,
    Move,
    UseItem,
    Heal,
    Flee,
    Wait,
}

///
pub fn determine_action_priority(
    hp_pct: i32,
    distance_to_player: i32,
    has_ranged: bool,
    has_special: bool,
    is_fleeing: bool,
    can_steal: bool,
    rng: &mut NetHackRng,
) -> ActionPriority {
    //
    if is_fleeing {
        return ActionPriority::Flee;
    }

    //
    if distance_to_player <= 1 {
        //
        if can_steal && rng.rn2(3) == 0 {
            return ActionPriority::StealAttack;
        }
        //
        if hp_pct < 25 {
            return ActionPriority::Heal;
        }
        return ActionPriority::MeleeAttack;
    }

    //
    if distance_to_player <= 8 {
        if has_special && rng.rn2(3) == 0 {
            return ActionPriority::SpecialAbility;
        }
        if has_ranged && rng.rn2(2) == 0 {
            return ActionPriority::RangedAttack;
        }
        return ActionPriority::Move;
    }

    ActionPriority::Move
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn biased_wander_direction(
    mx: i32,
    my: i32,
    corridor_dirs: &[(i32, i32)],
    rng: &mut NetHackRng,
) -> (i32, i32) {
    //
    if !corridor_dirs.is_empty() && rng.rn2(2) == 0 {
        let idx = rng.rn2(corridor_dirs.len() as i32) as usize;
        return corridor_dirs[idx];
    }
    wander_direction(rng)
}

///
pub fn tracking_move(
    mx: i32,
    my: i32,
    last_known_px: i32,
    last_known_py: i32,
    turns_since_seen: i32,
    rng: &mut NetHackRng,
) -> (i32, i32) {
    //
    if turns_since_seen > 20 {
        return wander_direction(rng);
    }

    //
    let (dx, dy) = calculate_move_direction(mx, my, last_known_px, last_known_py);

    //
    if rng.rn2(10) == 0 {
        wander_direction(rng)
    } else {
        (dx, dy)
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod monmove_extended_tests {
    use super::*;

    #[test]
    fn test_should_teleport_flee() {
        let mut rng = NetHackRng::new(42);
        assert!(should_teleport_flee(15, true, false, &mut rng));
        assert!(!should_teleport_flee(50, true, false, &mut rng));
        assert!(!should_teleport_flee(15, false, false, &mut rng));
    }

    #[test]
    fn test_action_priority_flee() {
        let mut rng = NetHackRng::new(42);
        let p = determine_action_priority(50, 5, true, true, true, false, &mut rng);
        assert_eq!(p, ActionPriority::Flee);
    }

    #[test]
    fn test_action_priority_melee() {
        let mut rng = NetHackRng::new(42);
        let p = determine_action_priority(80, 1, false, false, false, false, &mut rng);
        assert_eq!(p, ActionPriority::MeleeAttack);
    }

    #[test]
    fn test_seek_nearest_item() {
        let items = vec![(10, 10), (3, 3)];
        let dir = seek_nearest_item(5, 5, &items);
        assert!(dir.is_some());
        let (dx, dy) = dir.unwrap();
        assert_eq!((dx, dy), (-1, -1));
    }

    #[test]
    fn test_tracking_move_stale() {
        let mut rng = NetHackRng::new(42);
        let (dx, dy) = tracking_move(5, 5, 10, 10, 25, &mut rng);
        //
        assert!(dx >= -1 && dx <= 1 && dy >= -1 && dy <= 1);
    }

    #[test]
    fn test_ambush_bonus() {
        assert_eq!(ambush_attack_bonus(true, 1), 4);
        assert_eq!(ambush_attack_bonus(false, 1), 0);
        assert_eq!(ambush_attack_bonus(true, 3), 0);
    }
}
