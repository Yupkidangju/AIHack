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
use crate::core::dungeon::Grid;
use crate::core::dungeon::{COLNO, ROWNO};
use crate::core::entity::monster::Pet;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetBehavior {
    Following,
    Guarding,
    Hunting,
    Eating,
    Fleeing,
    Wandering,
}

///
pub struct PetContext {
    pub pet_x: i32,
    pub pet_y: i32,
    pub owner_x: i32,
    pub owner_y: i32,
    pub hunger: i32,
    pub loyalty: i32,
    pub apport: i32,
    pub is_confused: bool,
    pub is_stunned: bool,
}

// =============================================================================
//
// =============================================================================

///
///
pub fn dog_move(
    ctx: &PetContext,
    grid: &Grid,
    rng: &mut NetHackRng,
    nearby_food: &[(i32, i32)],
    nearby_enemies: &[(i32, i32)],
) -> Option<(i32, i32)> {
    //
    if ctx.is_confused || ctx.is_stunned {
        return random_pet_move(ctx, grid, rng);
    }

    //
    let behavior = determine_behavior(ctx, nearby_food, nearby_enemies);

    match behavior {
        PetBehavior::Following => follow_owner(ctx, grid, rng),
        PetBehavior::Guarding => guard_owner(ctx, grid, rng),
        PetBehavior::Hunting => hunt_target(ctx, grid, rng, nearby_enemies),
        PetBehavior::Eating => {
            if let Some(&(fx, fy)) = nearby_food.first() {
                move_towards_pos(ctx.pet_x, ctx.pet_y, fx, fy, grid, rng)
            } else {
                follow_owner(ctx, grid, rng)
            }
        }
        PetBehavior::Fleeing => flee_from_enemies(ctx, grid, rng, nearby_enemies),
        PetBehavior::Wandering => random_pet_move(ctx, grid, rng),
    }
}

///
fn determine_behavior(
    ctx: &PetContext,
    nearby_food: &[(i32, i32)],
    nearby_enemies: &[(i32, i32)],
) -> PetBehavior {
    //
    if ctx.hunger > 300 && !nearby_food.is_empty() {
        return PetBehavior::Eating;
    }

    //
    if ctx.loyalty < 3 {
        return PetBehavior::Wandering;
    }

    //
    if !nearby_enemies.is_empty() {
        let dist_to_owner = dist2(ctx.pet_x, ctx.pet_y, ctx.owner_x, ctx.owner_y);
        if dist_to_owner < 36 {
            return PetBehavior::Hunting;
        }
    }

    //
    if ctx.hunger > 150 && !nearby_food.is_empty() {
        return PetBehavior::Eating;
    }

    //
    let dist_to_owner = dist2(ctx.pet_x, ctx.pet_y, ctx.owner_x, ctx.owner_y);
    if dist_to_owner > 100 {
        PetBehavior::Following
    } else if dist_to_owner < 9 {
        PetBehavior::Guarding
    } else {
        PetBehavior::Following
    }
}

///
fn dist2(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    (x1 - x2).pow(2) + (y1 - y2).pow(2)
}

// =============================================================================
//
// =============================================================================

///
fn follow_owner(ctx: &PetContext, grid: &Grid, rng: &mut NetHackRng) -> Option<(i32, i32)> {
    move_towards_pos(ctx.pet_x, ctx.pet_y, ctx.owner_x, ctx.owner_y, grid, rng)
}

///
fn guard_owner(ctx: &PetContext, grid: &Grid, rng: &mut NetHackRng) -> Option<(i32, i32)> {
    let offset_x = rng.rn2(5) - 2;
    let offset_y = rng.rn2(5) - 2;
    let target_x = ctx.owner_x + offset_x;
    let target_y = ctx.owner_y + offset_y;
    move_towards_pos(ctx.pet_x, ctx.pet_y, target_x, target_y, grid, rng)
}

///
fn hunt_target(
    ctx: &PetContext,
    grid: &Grid,
    rng: &mut NetHackRng,
    enemies: &[(i32, i32)],
) -> Option<(i32, i32)> {
    let closest = enemies
        .iter()
        .min_by_key(|&&(ex, ey)| dist2(ctx.pet_x, ctx.pet_y, ex, ey));

    if let Some(&(ex, ey)) = closest {
        move_towards_pos(ctx.pet_x, ctx.pet_y, ex, ey, grid, rng)
    } else {
        follow_owner(ctx, grid, rng)
    }
}

///
fn flee_from_enemies(
    ctx: &PetContext,
    grid: &Grid,
    rng: &mut NetHackRng,
    enemies: &[(i32, i32)],
) -> Option<(i32, i32)> {
    if enemies.is_empty() {
        return follow_owner(ctx, grid, rng);
    }

    let closest = enemies
        .iter()
        .min_by_key(|&&(ex, ey)| dist2(ctx.pet_x, ctx.pet_y, ex, ey));

    if let Some(&(ex, ey)) = closest {
        let dx = ctx.pet_x - ex;
        let dy = ctx.pet_y - ey;
        let flee_x = ctx.pet_x + dx.signum();
        let flee_y = ctx.pet_y + dy.signum();
        if is_walkable(flee_x, flee_y, grid) {
            Some((flee_x, flee_y))
        } else {
            random_pet_move(ctx, grid, rng)
        }
    } else {
        random_pet_move(ctx, grid, rng)
    }
}

///
fn random_pet_move(ctx: &PetContext, grid: &Grid, rng: &mut NetHackRng) -> Option<(i32, i32)> {
    let deltas: [(i32, i32); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let start = rng.rn2(8) as usize;
    for i in 0..8 {
        let idx = (start + i) % 8;
        let (dx, dy) = deltas[idx];
        let nx = ctx.pet_x + dx;
        let ny = ctx.pet_y + dy;
        if is_walkable(nx, ny, grid) {
            return Some((nx, ny));
        }
    }
    None
}

///
fn move_towards_pos(
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
    grid: &Grid,
    rng: &mut NetHackRng,
) -> Option<(i32, i32)> {
    let dx = (to_x - from_x).signum();
    let dy = (to_y - from_y).signum();

    let nx = from_x + dx;
    let ny = from_y + dy;
    if is_walkable(nx, ny, grid) {
        return Some((nx, ny));
    }

    if dx != 0 && dy != 0 {
        if is_walkable(from_x + dx, from_y, grid) {
            return Some((from_x + dx, from_y));
        }
        if is_walkable(from_x, from_y + dy, grid) {
            return Some((from_x, from_y + dy));
        }
    }

    //
    let tmp_ctx = PetContext {
        pet_x: from_x,
        pet_y: from_y,
        owner_x: to_x,
        owner_y: to_y,
        hunger: 0,
        loyalty: 10,
        apport: 0,
        is_confused: false,
        is_stunned: false,
    };
    random_pet_move(&tmp_ctx, grid, rng)
}

///
fn is_walkable(x: i32, y: i32, grid: &Grid) -> bool {
    if x < 0 || y < 0 || x as usize >= COLNO || y as usize >= ROWNO {
        return false;
    }
    if let Some(tile) = grid.get_tile(x as usize, y as usize) {
        matches!(
            tile.typ,
            TileType::Room
                | TileType::Corr
                | TileType::OpenDoor
                | TileType::StairsUp
                | TileType::StairsDown
        )
    } else {
        false
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DogFood {
    Apport = 0,
    Manfood = 1,
    Accfood = 2,
    Dogfood = 3,
    Cadaver = 4,
    Poison = 5,
    Undef = 6,
    Tabu = 7,
}

///
pub fn classify_food(item_name: &str, pet_symbol: char, _rng: &mut NetHackRng) -> DogFood {
    if item_name.contains("corpse") {
        if item_name.contains("green") || item_name.contains("kobold") {
            return DogFood::Poison;
        }
        return DogFood::Cadaver;
    }

    if item_name.contains("tripe ration") {
        if pet_symbol == 'd' || pet_symbol == 'f' {
            return DogFood::Dogfood;
        }
        return DogFood::Accfood;
    }

    let foods = [
        "apple",
        "orange",
        "pear",
        "melon",
        "banana",
        "cream pie",
        "candy bar",
        "fortune cookie",
        "pancake",
        "lembas wafer",
        "cram ration",
        "food ration",
        "K-ration",
        "C-ration",
    ];
    for food in &foods {
        if item_name.contains(food) {
            return DogFood::Manfood;
        }
    }

    if item_name.contains("meat")
        || item_name.contains("meatball")
        || item_name.contains("huge chunk")
    {
        if pet_symbol == 'd' || pet_symbol == 'f' {
            return DogFood::Dogfood;
        }
        return DogFood::Manfood;
    }

    DogFood::Undef
}

// =============================================================================
//
// =============================================================================

///
pub fn try_tame(
    monster_level: i32,
    player_charisma: i32,
    food_bonus: i32,
    rng: &mut NetHackRng,
) -> bool {
    let difficulty = monster_level * 3;
    let charm = player_charisma + food_bonus;

    if monster_level <= 3 {
        return rng.rn2(20) < charm;
    }
    rng.rn2(difficulty) < charm / 2
}

///
pub fn abuse_pet(loyalty: &mut i32, rng: &mut NetHackRng) -> bool {
    *loyalty -= rng.rn2(3) + 1;
    if *loyalty <= 0 {
        *loyalty = 0;
        true
    } else {
        false
    }
}

///
pub fn feed_pet(loyalty: &mut i32, food_quality: DogFood) {
    let bonus = match food_quality {
        DogFood::Dogfood => 5,
        DogFood::Cadaver => 3,
        DogFood::Accfood => 2,
        DogFood::Manfood => 1,
        _ => 0,
    };
    *loyalty = (*loyalty + bonus).min(20);
}

// =============================================================================
//
// =============================================================================

///
pub fn pet_turn_update(pet: &mut Pet, turn: u64, _rng: &mut NetHackRng) {
    if turn % 10 == 0 {
        pet.hunger += 1;
    }
    if pet.hunger > 500 && turn % 50 == 0 {
        if pet.loyalty > 0 {
            pet.loyalty -= 1;
        }
    }
}

///
pub fn pet_eat(pet: &mut Pet, nutrition: i32) {
    pet.hunger = (pet.hunger - nutrition).max(0);
    if nutrition > 50 {
        pet.loyalty = (pet.loyalty + 1).min(20);
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn christen_pet(current_name: &mut Option<String>, new_name: &str) {
    if new_name.is_empty() {
        *current_name = None;
    } else {
        *current_name = Some(new_name.to_string());
    }
}

///
pub fn pet_display_name(custom_name: &Option<String>, template_name: &str) -> String {
    match custom_name {
        Some(name) => name.clone(),
        None => format!("your {}", template_name),
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn should_attack_monster(
    pet_level: i32,
    target_level: i32,
    loyalty: i32,
    rng: &mut NetHackRng,
) -> bool {
    let aggression = loyalty / 3;
    let level_diff = pet_level - target_level;

    if level_diff >= 3 {
        true
    } else if level_diff >= 0 {
        rng.rn2(10) < aggression + 3
    } else {
        rng.rn2(20) < aggression
    }
}

///
pub fn should_pick_up(
    item_name: &str,
    pet_symbol: char,
    apport: i32,
    rng: &mut NetHackRng,
) -> bool {
    if apport <= 0 {
        return false;
    }

    let food_class = classify_food(item_name, pet_symbol, rng);
    match food_class {
        DogFood::Dogfood | DogFood::Cadaver => true,
        DogFood::Manfood | DogFood::Accfood => rng.rn2(10) < apport,
        _ => rng.rn2(20) < apport,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn pet_follows_player(
    pet_x: i32,
    pet_y: i32,
    player_x: i32,
    player_y: i32,
    loyalty: i32,
) -> bool {
    let dist = dist2(pet_x, pet_y, player_x, player_y);
    dist <= 2 && loyalty >= 5
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_food() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            classify_food("tripe ration", 'd', &mut rng),
            DogFood::Dogfood
        );
        assert_eq!(
            classify_food("tripe ration", '@', &mut rng),
            DogFood::Accfood
        );
        assert_eq!(classify_food("orc corpse", 'd', &mut rng), DogFood::Cadaver);
        assert_eq!(classify_food("dagger", 'd', &mut rng), DogFood::Undef);
    }

    #[test]
    fn test_pet_follows_player() {
        assert!(pet_follows_player(5, 5, 6, 5, 10));
        assert!(!pet_follows_player(5, 5, 20, 20, 10));
        assert!(!pet_follows_player(5, 5, 6, 5, 2));
    }

    #[test]
    fn test_determine_behavior() {
        let ctx = PetContext {
            pet_x: 5,
            pet_y: 5,
            owner_x: 50,
            owner_y: 50,
            hunger: 0,
            loyalty: 10,
            apport: 5,
            is_confused: false,
            is_stunned: false,
        };
        assert_eq!(determine_behavior(&ctx, &[], &[]), PetBehavior::Following);
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn pet_refuses_food(food_quality: DogFood, hunger: i32, loyalty: i32) -> bool {
    match food_quality {
        DogFood::Tabu => true,
        DogFood::Poison => loyalty > 5,
        DogFood::Undef => hunger < 300,
        _ => false,
    }
}

///
pub fn pet_combat_rating(pet_level: i32, pet_hp: i32, pet_ac: i32, pet_loyalty: i32) -> i32 {
    let base = pet_level * 3 + pet_hp / 2;
    let ac_bonus = (10 - pet_ac).max(0);
    let loyalty_bonus = pet_loyalty / 5;
    base + ac_bonus + loyalty_bonus
}

///
pub fn pet_can_defeat(pet_rating: i32, enemy_level: i32, rng: &mut NetHackRng) -> bool {
    let enemy_rating = enemy_level * 5;
    let roll = rng.rn2(pet_rating + enemy_rating);
    roll < pet_rating
}

///
pub fn pet_should_level_up(kills: u32, current_level: i32) -> bool {
    let threshold = (current_level as u32) * 5;
    kills >= threshold
}

///
pub fn pet_level_up_effect(
    current_level: &mut i32,
    current_hp: &mut i32,
    max_hp: &mut i32,
    rng: &mut NetHackRng,
) {
    *current_level += 1;
    let hp_gain = rng.rn1(8, 3);
    *max_hp += hp_gain;
    *current_hp = *max_hp;
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PetDeathCause {
    Killed,
    Starved,
    Petrified,
    Drowned,
    Abandoned,
    FriendlyFire,
}

///
pub fn pet_death_message(name: &str, cause: &PetDeathCause) -> String {
    match cause {
        PetDeathCause::Killed => format!("{} is killed!", name),
        PetDeathCause::Starved => format!("{} has starved to death!", name),
        PetDeathCause::Petrified => format!("{} turns to stone!", name),
        PetDeathCause::Drowned => format!("{} drowns!", name),
        PetDeathCause::Abandoned => format!("{} looks around for its master...", name),
        PetDeathCause::FriendlyFire => format!("You accidentally kill {}!", name),
    }
}

///
pub fn pet_teleport_follow_distance(loyalty: i32) -> i32 {
    //
    15 + loyalty / 2
}

///
pub fn pet_habitat_compatible(pet_symbol: char, tile_name: &str) -> bool {
    match pet_symbol {
        'f' | 'd' | 'C' => tile_name != "water" && tile_name != "lava",
        ';' => {
            //
            tile_name == "water" || tile_name == "pool"
        }
        _ => tile_name != "lava",
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct PetStatistics {
    pub total_pets_tamed: u32,
    pub pets_died: u32,
    pub pets_leveled_up: u32,
    pub pet_kills: u32,
    pub foods_eaten: u32,
    pub foods_refused: u32,
    pub loyalty_abuses: u32,
}

impl PetStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_tame(&mut self) {
        self.total_pets_tamed += 1;
    }
    pub fn record_death(&mut self) {
        self.pets_died += 1;
    }
    pub fn record_kill(&mut self) {
        self.pet_kills += 1;
    }
    pub fn record_feed(&mut self, refused: bool) {
        if refused {
            self.foods_refused += 1;
        } else {
            self.foods_eaten += 1;
        }
    }
}

#[cfg(test)]
mod dog_extended_tests {
    use super::*;

    #[test]
    fn test_refuses_food() {
        assert!(pet_refuses_food(DogFood::Tabu, 0, 10));
        assert!(!pet_refuses_food(DogFood::Dogfood, 0, 10));
    }

    #[test]
    fn test_combat_rating() {
        let r = pet_combat_rating(5, 30, 5, 10);
        assert!(r > 0);
    }

    #[test]
    fn test_level_up_check() {
        assert!(pet_should_level_up(5, 1));
        assert!(!pet_should_level_up(3, 1));
    }

    #[test]
    fn test_level_up() {
        let mut rng = NetHackRng::new(42);
        let mut level = 3;
        let mut hp = 20;
        let mut max_hp = 20;
        pet_level_up_effect(&mut level, &mut hp, &mut max_hp, &mut rng);
        assert_eq!(level, 4);
        assert!(max_hp > 20);
        assert_eq!(hp, max_hp);
    }

    #[test]
    fn test_death_message() {
        let m = pet_death_message("Rex", &PetDeathCause::Starved);
        assert!(m.contains("Rex"));
        assert!(m.contains("starved"));
    }

    #[test]
    fn test_habitat() {
        assert!(pet_habitat_compatible('d', "floor"));
        assert!(!pet_habitat_compatible('d', "lava"));
        assert!(pet_habitat_compatible(';', "water"));
    }

    #[test]
    fn test_pet_stats() {
        let mut stats = PetStatistics::new();
        stats.record_tame();
        stats.record_kill();
        stats.record_feed(false);
        assert_eq!(stats.total_pets_tamed, 1);
        assert_eq!(stats.pet_kills, 1);
        assert_eq!(stats.foods_eaten, 1);
    }
}
