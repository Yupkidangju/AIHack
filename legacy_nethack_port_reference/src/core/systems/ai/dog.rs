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

// =============================================================================
// [v2.9.4] dog.c + dogmove.c 2차 이식 — 핵심 미구현 로직
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// initedog 결과 (원본: dog.c L33-54)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 초기 길들이기 결과 (원본: initedog L37-53)
#[derive(Debug, Clone)]
pub struct InitEdogResult {
    /// 길들이기 수치: is_domestic? 10 : 5
    pub tame: i32,
    /// 평화 상태
    pub peaceful: bool,
    /// 복수 없음
    pub mavenge: bool,
    /// 목줄 없음
    pub leashed: bool,
    /// 식사 중 아님
    pub eating: i32,
    /// 드롭 거리 (초기 10000)
    pub dropdist: i32,
    /// apport = 카리스마
    pub apport: i32,
    /// 배고픔 시간 = 1000 + current_turn
    pub hungrytime: i64,
    /// 학대 0
    pub abuse: i32,
    /// 부활 0
    pub revivals: i32,
    /// HP 감소 패널티 0
    pub mhpmax_penalty: i32,
    /// 플레이어에 의한 사망 여부 0
    pub killed_by_u: i32,
}

/// [v2.9.4] 초기 길들이기 결과 계산 (원본: initedog L33-54)
pub fn init_edog_result(is_domestic: bool, player_cha: i32, current_turn: i64) -> InitEdogResult {
    InitEdogResult {
        tame: if is_domestic { 10 } else { 5 },
        peaceful: true,
        mavenge: false,
        leashed: false,
        eating: 0,
        dropdist: 10000,
        apport: player_cha,
        hungrytime: 1000 + current_turn,
        abuse: 0,
        revivals: 0,
        mhpmax_penalty: 0,
        killed_by_u: 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// dogfood 확장 판정 (원본: dog.c L733-858)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 확장 음식 판정 입력 (원본: dogfood L740-857)
#[derive(Debug, Clone)]
pub struct DogFoodInput {
    /// 육식
    pub is_carnivore: bool,
    /// 초식
    pub is_herbivore: bool,
    /// 굶주림 (tame && !minion && mhpmax_penalty>0)
    pub is_starving: bool,
    /// 일시적 실명
    pub is_blind: bool,
    /// 구울
    pub is_ghoul: bool,
    /// 언데드
    pub is_undead: bool,
    /// 뱀파이어시프터
    pub is_vampshifter: bool,
    /// 젤라틴 큐브
    pub is_gelatinous_cube: bool,
    /// 금속 식사
    pub is_metallivorous: bool,
    /// 러스트 몬스터
    pub is_rust_monster: bool,
    /// 유인원/예티 계열
    pub is_yeti_family: bool,
    /// 인간형
    pub is_humanoid: bool,
    /// 곰팡이 계열
    pub is_fungus: bool,
    /// 은 혐오
    pub hates_silver: bool,
    /// 석화 내성
    pub resists_ston: bool,
    /// 산성 내성
    pub resists_acid: bool,
    /// 독 내성
    pub resists_poison: bool,
    /// 슬라임 내성
    pub slimeproof: bool,
    /// 엘프
    pub is_elf: bool,
    /// 같은 종족
    pub same_race: bool,
}

/// [v2.9.4] 확장 음식 판정 입력 — 아이템 정보
#[derive(Debug, Clone)]
pub struct FoodItemInfo {
    /// FOOD_CLASS 여부
    pub is_food: bool,
    /// 아이템 이름 (식별용)
    pub item_type: FoodType,
    /// 저주
    pub is_cursed: bool,
    /// 축복
    pub is_blessed: bool,
    /// 시체인 경우: 라이더
    pub corpse_is_rider: bool,
    /// 시체인 경우: 석화
    pub corpse_touch_petrifies: bool,
    /// 시체인 경우: 비건
    pub corpse_is_vegan: bool,
    /// 시체인 경우: 산성
    pub corpse_is_acidic: bool,
    /// 시체인 경우: 독
    pub corpse_is_poisonous: bool,
    /// 시체인 경우: 그린 슬라임
    pub corpse_is_green_slime: bool,
    /// 시체인 경우: 도마뱀/이끼
    pub corpse_is_lizard_lichen: bool,
    /// 시체나이: 오래됨 (age+50 <= monstermoves)
    pub corpse_is_old: bool,
    /// 유기물
    pub is_organic: bool,
    /// 금속
    pub is_metallic: bool,
    /// 녹 취약
    pub is_rustprone: bool,
    /// 방부 처리
    pub is_erodeproof: bool,
    /// 은 재질
    pub is_silver: bool,
    /// ROCK_CLASS
    pub is_rock_class: bool,
    /// 퀘스트 아티팩트
    pub is_quest_artifact: bool,
    /// 저항 (obj_resists 95%)
    pub obj_resists95: bool,
}

/// [v2.9.4] 음식 유형 enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodType {
    Tripe,
    Meatball,
    MeatRing,
    MeatStick,
    HugeChunk,
    Egg,
    Corpse,
    Garlic,
    Tin,
    Apple,
    Carrot,
    Banana,
    OtherFood,
    StrangulationAmulet,
    SlowDigestionRing,
    Coin,
    Other,
}

/// [v2.9.4] 확장 dogfood 판정 (원본: dogfood L733-858)
pub fn dogfood_extended(mon: &DogFoodInput, food: &FoodItemInfo) -> DogFood {
    // 퀘스트 아티팩트 또는 저항
    if food.is_quest_artifact || food.obj_resists95 {
        return if food.is_cursed {
            DogFood::Tabu
        } else {
            DogFood::Apport
        };
    }

    if food.is_food {
        // 라이더 시체
        if matches!(food.item_type, FoodType::Corpse) && food.corpse_is_rider {
            return DogFood::Tabu;
        }
        // 석화 시체/알
        if (matches!(food.item_type, FoodType::Corpse | FoodType::Egg))
            && food.corpse_touch_petrifies
            && !mon.resists_ston
        {
            return DogFood::Poison;
        }
        // 육/초식 둘 다 아닌 경우
        if !mon.is_carnivore && !mon.is_herbivore {
            return if food.is_cursed {
                DogFood::Undef
            } else {
                DogFood::Apport
            };
        }

        // 구울 (원본: L769-781)
        if mon.is_ghoul {
            return match food.item_type {
                FoodType::Corpse => {
                    if food.corpse_is_old && !food.corpse_is_lizard_lichen {
                        DogFood::Dogfood
                    } else if mon.is_starving && !food.corpse_is_vegan {
                        DogFood::Accfood
                    } else {
                        DogFood::Poison
                    }
                }
                FoodType::Egg => {
                    if food.corpse_is_old {
                        DogFood::Cadaver
                    } else if mon.is_starving {
                        DogFood::Accfood
                    } else {
                        DogFood::Poison
                    }
                }
                _ => DogFood::Tabu,
            };
        }

        match food.item_type {
            FoodType::Tripe
            | FoodType::Meatball
            | FoodType::MeatRing
            | FoodType::MeatStick
            | FoodType::HugeChunk => {
                if mon.is_carnivore {
                    DogFood::Dogfood
                } else {
                    DogFood::Manfood
                }
            }
            FoodType::Egg => {
                if mon.is_carnivore {
                    DogFood::Cadaver
                } else {
                    DogFood::Manfood
                }
            }
            FoodType::Corpse => {
                // 오래된/산성/독 시체 (원본: L793-798)
                if (food.corpse_is_old && !food.corpse_is_lizard_lichen && !mon.is_fungus)
                    || (food.corpse_is_acidic && !mon.resists_acid)
                    || (food.corpse_is_poisonous && !mon.resists_poison)
                {
                    DogFood::Poison
                } else if food.corpse_is_green_slime && !mon.slimeproof {
                    if mon.is_starving {
                        DogFood::Accfood
                    } else {
                        DogFood::Poison
                    }
                } else if food.corpse_is_vegan {
                    if mon.is_herbivore {
                        DogFood::Cadaver
                    } else {
                        DogFood::Manfood
                    }
                } else if mon.is_humanoid && mon.same_race && !mon.is_undead {
                    // 식인 피하기 (원본: L806-809)
                    if mon.is_starving && mon.is_carnivore && !mon.is_elf {
                        DogFood::Accfood
                    } else {
                        DogFood::Tabu
                    }
                } else if mon.is_carnivore {
                    DogFood::Cadaver
                } else {
                    DogFood::Manfood
                }
            }
            FoodType::Garlic => {
                if mon.is_undead || mon.is_vampshifter {
                    DogFood::Tabu
                } else if mon.is_herbivore || mon.is_starving {
                    DogFood::Accfood
                } else {
                    DogFood::Manfood
                }
            }
            FoodType::Tin => {
                if mon.is_metallivorous {
                    DogFood::Accfood
                } else {
                    DogFood::Manfood
                }
            }
            FoodType::Apple => {
                if mon.is_herbivore {
                    DogFood::Dogfood
                } else if mon.is_starving {
                    DogFood::Accfood
                } else {
                    DogFood::Manfood
                }
            }
            FoodType::Carrot => {
                if mon.is_herbivore || mon.is_blind {
                    DogFood::Dogfood
                } else if mon.is_starving {
                    DogFood::Accfood
                } else {
                    DogFood::Manfood
                }
            }
            FoodType::Banana => {
                if mon.is_yeti_family && mon.is_herbivore {
                    DogFood::Dogfood
                } else if mon.is_herbivore || mon.is_starving {
                    DogFood::Accfood
                } else {
                    DogFood::Manfood
                }
            }
            FoodType::OtherFood => {
                if mon.is_starving {
                    DogFood::Accfood
                } else if mon.is_carnivore {
                    DogFood::Accfood
                } else {
                    DogFood::Manfood
                }
            }
            _ => DogFood::Undef,
        }
    } else {
        // 비음식 클래스 (원본: L836-857)
        if matches!(
            food.item_type,
            FoodType::StrangulationAmulet | FoodType::SlowDigestionRing
        ) {
            return DogFood::Tabu;
        }
        if mon.hates_silver && food.is_silver {
            return DogFood::Tabu;
        }
        if mon.is_gelatinous_cube && food.is_organic {
            return DogFood::Accfood;
        }
        if mon.is_metallivorous && food.is_metallic {
            if food.is_rustprone && !mon.is_rust_monster {
                return if !food.is_erodeproof {
                    DogFood::Dogfood
                } else {
                    DogFood::Accfood
                };
            }
            if mon.is_rust_monster && food.is_rustprone && !food.is_erodeproof {
                return DogFood::Dogfood;
            }
            return DogFood::Accfood;
        }
        if !food.is_cursed && !food.is_rock_class {
            return DogFood::Apport;
        }
        DogFood::Undef
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// dog_nutrition 결과 (원본: dogmove.c L141-202)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 영양가 크기 배수 (원본: dog_nutrition L160-180)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonSize {
    Tiny,     // ×8
    Small,    // ×6
    Medium,   // ×5
    Large,    // ×4
    Huge,     // ×3
    Gigantic, // ×2
}

/// [v2.9.4] 영양가 계산 결과 (원본: dog_nutrition L141-202)
#[derive(Debug, Clone)]
pub struct DogNutritionResult {
    /// 식사 시간 (턴)
    pub eating_time: i32,
    /// 영양가
    pub nutrition: i32,
}

/// [v2.9.4] 영양가 계산 (원본: dog_nutrition L141-202)
pub fn dog_nutrition_result(
    is_food_class: bool,
    is_corpse: bool,
    corpse_weight: i32,    // cwt >> 6
    corpse_nutrition: i32, // cnutrit
    food_delay: i32,       // oc_delay
    food_nutrition: i32,   // oc_nutrition
    is_coin: bool,
    coin_quantity: i64,
    obj_weight: i32,    // owt
    obj_nutrition: i32, // oc_nutrition (비음식)
    mon_size: MonSize,
    partially_eaten: bool,
    eaten_fraction: i32, // 0-100(%)
) -> DogNutritionResult {
    let (mut eat_time, mut nutrit) = if is_food_class {
        if is_corpse {
            (3 + corpse_weight, corpse_nutrition)
        } else {
            (food_delay, food_nutrition)
        }
    } else if is_coin {
        let et = ((coin_quantity / 2000) + 1).max(1) as i32;
        let nt = ((coin_quantity / 20).max(0)) as i32;
        (et, nt)
    } else {
        (obj_weight / 20 + 1, 5 * obj_nutrition)
    };

    // 크기 보정 (원본: L160-180)
    if is_food_class {
        let mult = match mon_size {
            MonSize::Tiny => 8,
            MonSize::Small => 6,
            MonSize::Medium => 5,
            MonSize::Large => 4,
            MonSize::Huge => 3,
            MonSize::Gigantic => 2,
        };
        nutrit *= mult;
    }

    // 부분 섭취 (원본: L181-184)
    if partially_eaten && eaten_fraction > 0 {
        eat_time = eat_time * eaten_fraction / 100;
        nutrit = nutrit * eaten_fraction / 100;
    }

    DogNutritionResult {
        eating_time: eat_time.max(1),
        nutrition: nutrit.max(0),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// dog_eat 결과 (원본: dogmove.c L204-353)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 식사 결과 (원본: dog_eat L204-353)
#[derive(Debug, Clone)]
pub struct DogEatResult {
    /// 영양 추가량
    pub nutrition_gained: i32,
    /// 혼란 해제
    pub clear_confusion: bool,
    /// HP 패널티 복구
    pub restore_hp_penalty: bool,
    /// 도주 시간 절반
    pub halve_flee: bool,
    /// 길들이기 +1
    pub tame_increase: bool,
    /// 위치 달라짐 → newsym 필요
    pub moved_to_eat: bool,
    /// 다형 변이 필요 (polymorph/slime)
    pub polymorph: bool,
    /// 슬라임화
    pub slime: bool,
    /// 성장 (mlevelgain)
    pub grow: bool,
    /// 치유 (mhealup)
    pub heal: bool,
    /// 실명 치료 (당근/치유)
    pub cure_blindness: bool,
    /// 미믹 흉내 (deadmimic)
    pub quickmimic: bool,
    /// devour 모드 시 이식 절반
    pub devour: bool,
    /// 러스트 몬스터 + erodeproof → 기절
    pub rust_stun: bool,
}

/// [v2.9.4] 식사 결과 계산 (원본: dog_eat L204-353)
pub fn dog_eat_result(
    nutrition: i32,
    devour: bool,
    is_fleeing: bool,
    flee_time: i32,
    tame: i32,
    hp_penalty: i32,
    moved: bool,
    poly_food: bool,
    level_gain_food: bool,
    heal_food: bool,
    is_carrot: bool,
    is_mimicorpse: bool,
    is_slime_corpse: bool,
    is_rust_monster: bool,
    is_erodeproof: bool,
    is_blind: bool,
) -> DogEatResult {
    let actual_nutrit = if devour { nutrition * 3 / 4 } else { nutrition };

    DogEatResult {
        nutrition_gained: actual_nutrit,
        clear_confusion: true,
        restore_hp_penalty: hp_penalty > 0,
        halve_flee: is_fleeing && flee_time > 1,
        tame_increase: tame < 20,
        moved_to_eat: moved,
        polymorph: poly_food,
        slime: is_slime_corpse,
        grow: level_gain_food,
        heal: heal_food,
        cure_blindness: (is_carrot || heal_food) && is_blind,
        quickmimic: is_mimicorpse,
        devour,
        rust_stun: is_rust_monster && is_erodeproof,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// dog_hunger 결과 (원본: dogmove.c L355-397)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 배고픔 결과 (원본: dog_hunger L355-397)
#[derive(Debug, Clone, PartialEq)]
pub enum HungerResult {
    /// 배고프지 않음
    Ok,
    /// 혼란 시작 (hp 1/3 감소)
    ConfusedFromHunger { new_maxhp: i32, penalty: i32 },
    /// 기아 사망
    Starved,
}

/// [v2.9.4] 배고픔 판정 (원본: dog_hunger L355-397)
pub fn dog_hunger_result(
    current_turn: i64,
    hungrytime: i64,
    maxhp: i32,
    hp: i32,
    current_penalty: i32,
    is_carnivore: bool,
    is_herbivore: bool,
) -> HungerResult {
    if current_turn <= hungrytime + 500 {
        return HungerResult::Ok;
    }

    if !is_carnivore && !is_herbivore {
        return HungerResult::Ok; // 비식사 몬스터 — 시간 리셋
    }

    if current_penalty == 0 {
        // 처음 굶주림 (원본: L365-381)
        let new_max = maxhp / 3;
        let penalty = maxhp - new_max;
        if new_max <= 0 || hp <= 0 {
            return HungerResult::Starved;
        }
        return HungerResult::ConfusedFromHunger {
            new_maxhp: new_max,
            penalty,
        };
    }

    // 이미 패널티 적용 — 750턴 이후 사망 (원본: L382-393)
    if current_turn > hungrytime + 750 || hp <= 0 {
        return HungerResult::Starved;
    }

    HungerResult::Ok
}

// ─────────────────────────────────────────────────────────────────────────────
// tamedog 결과 (원본: dog.c L860-952)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 길들이기 결과 (원본: tamedog L865-952)
#[derive(Debug, Clone, PartialEq)]
pub enum TameDogResult {
    /// 길들이기 불가 (위즈/메두사/퀘스트 등)
    Refused,
    /// 평화만 달성 (보름달 + 밤 + 개)
    PeacefulOnly,
    /// 먹이 던져서 먹음 (기존 펫)
    AteFood,
    /// 먹이 던져 안 먹음 (기존 펫)
    RefusedFood,
    /// 구조 충돌 (상점/경비/성직/인간 등)
    StructureConflict,
    /// 성공
    Tamed,
}

/// [v2.9.4] 길들이기 판정 (원본: tamedog L865-952)
pub fn tamedog_result(
    is_wiz: bool,
    is_medusa: bool,
    wants_arti: bool,
    is_dog_family: bool,
    full_moon_night: bool,
    already_tame: bool,
    can_move: bool,
    is_confused: bool,
    is_eating: bool,
    has_food: bool,
    food_quality: DogFood,
    is_hungry: bool,
    is_shk: bool,
    is_guard: bool,
    is_priest: bool,
    is_minion: bool,
    is_covetous: bool,
    is_human: bool,
    is_demon: bool,
    player_is_demon: bool,
    is_quest_leader: bool,
    rng: &mut NetHackRng,
) -> TameDogResult {
    // 절대 불가 (원본: L871-873)
    if is_wiz || is_medusa || wants_arti {
        return TameDogResult::Refused;
    }

    // 보름달 + 밤 + 개 (원본: L878-880)
    if full_moon_night && is_dog_family && has_food && rng.rn2(6) != 0 {
        return TameDogResult::PeacefulOnly;
    }

    // 기존 펫 + 먹이 (원본: L895-920)
    if already_tame && has_food {
        if can_move
            && !is_confused
            && !is_eating
            && (food_quality == DogFood::Dogfood || (food_quality <= DogFood::Accfood && is_hungry))
        {
            return TameDogResult::AteFood;
        }
        return TameDogResult::RefusedFood;
    }

    // 구조 충돌 (원본: L922-928)
    if already_tame
        || !can_move
        || is_shk
        || is_guard
        || is_priest
        || is_minion
        || is_covetous
        || is_human
        || (is_demon && !player_is_demon)
    {
        return TameDogResult::StructureConflict;
    }

    // 퀘스트 리더 (원본: L930-931)
    if is_quest_leader {
        return TameDogResult::StructureConflict;
    }

    // 음식 품질 확인 (원본: L927)
    if has_food && food_quality >= DogFood::Manfood {
        return TameDogResult::Refused;
    }

    TameDogResult::Tamed
}

// ─────────────────────────────────────────────────────────────────────────────
// wary_dog 결과 (원본: dog.c L954-1030)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 부활 후 성격 판정 결과 (원본: wary_dog L961-1030)
#[derive(Debug, Clone)]
pub struct WaryDogResult {
    /// 길들이기 수치
    pub new_tame: i32,
    /// 평화 여부
    pub new_peaceful: bool,
    /// 부활 카운터 증가
    pub increment_revivals: bool,
    /// 학대/사망 카운터 리셋
    pub reset_abuse: bool,
    /// HP 패널티 복구
    pub restore_hp_penalty: bool,
    /// 목줄 해제 필요
    pub unleash: bool,
    /// 기수 하마 필요
    pub dismount: bool,
}

/// [v2.9.4] 부활 후 성격 판정 (원본: wary_dog L961-1030)
pub fn wary_dog_result(
    tame: i32,
    was_dead: bool,
    killed_by_u: bool,
    abuse: i32,
    hp_penalty: i32,
    is_minion: bool,
    is_leashed: bool,
    is_steed: bool,
    rng: &mut NetHackRng,
) -> WaryDogResult {
    let mut r = WaryDogResult {
        new_tame: tame,
        new_peaceful: true,
        increment_revivals: false,
        reset_abuse: false,
        restore_hp_penalty: hp_penalty > 0,
        unleash: false,
        dismount: false,
    };

    if tame == 0 {
        return r;
    }

    // 심한 학대나 플레이어가 죽인 경우 (원본: L982-996)
    if !is_minion && (killed_by_u || abuse > 2) {
        r.new_tame = 0;
        r.new_peaceful = if abuse >= 0 && abuse < 10 {
            rng.rn2(abuse + 1) == 0
        } else {
            false
        };
    } else {
        // Pet Sematary 확률 (원본: L998-1001)
        r.new_tame = rng.rn2(tame + 1);
        if r.new_tame == 0 {
            r.new_peaceful = rng.rn2(2) != 0;
        }
    }

    // 길들이기 실패 시 (원본: L1004-1014)
    if r.new_tame == 0 {
        if is_leashed {
            r.unleash = true;
        }
        if is_steed {
            r.dismount = true;
        }
    } else if !is_minion {
        // 성공: 슬레이트 초기화 (원본: L1015-1029)
        r.increment_revivals = true;
        r.reset_abuse = true;
    }

    r
}

// ─────────────────────────────────────────────────────────────────────────────
// abuse_dog 결과 (원본: dog.c L1032-1061)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 학대 결과 (원본: abuse_dog L1032-1061)
#[derive(Debug, Clone)]
pub struct AbuseDogResult {
    /// 새 길들이기 수치
    pub new_tame: i32,
    /// 학대 카운터 증가
    pub abuse_increment: bool,
    /// 목줄 해제 필요
    pub unleash: bool,
    /// 화난 소리 (false=으르렁, true=낑낑)
    pub sound: Option<bool>,
}

/// [v2.9.4] 학대 결과 계산 (원본: abuse_dog L1032-1061)
pub fn abuse_dog_result(
    tame: i32,
    aggravate_or_conflict: bool,
    is_minion: bool,
    is_leashed: bool,
    on_map: bool,
    rng: &mut NetHackRng,
) -> AbuseDogResult {
    if tame == 0 {
        return AbuseDogResult {
            new_tame: 0,
            abuse_increment: false,
            unleash: false,
            sound: None,
        };
    }

    // 감소 (원본: L1039-1042)
    let new_tame = if aggravate_or_conflict {
        tame / 2
    } else {
        tame - 1
    };

    let abuse_inc = new_tame > 0 && !is_minion;
    let unleash = new_tame == 0 && is_leashed;

    let sound = if on_map {
        if new_tame > 0 && rng.rn2(new_tame) != 0 {
            Some(true) // yelp (낑낑)
        } else {
            Some(false) // growl (으르렁)
        }
    } else {
        None
    };

    AbuseDogResult {
        new_tame,
        abuse_increment: abuse_inc,
        unleash,
        sound,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// mon_catchup_elapsed_time 결과 (원본: dog.c L463-561)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 시간 경과 따라잡기 결과 (원본: mon_catchup_elapsed_time L463-561)
#[derive(Debug, Clone)]
pub struct CatchupResult {
    /// 실명 남은 턴
    pub blinded: i32,
    /// 동결 남은 턴
    pub frozen: i32,
    /// 도주 남은 턴
    pub flee_time: i32,
    /// 트랩 탈출 여부
    pub freed_from_trap: bool,
    /// 혼란 해제 여부
    pub confusion_cleared: bool,
    /// 기절 해제 여부
    pub stun_cleared: bool,
    /// 식사 완료 여부
    pub finish_eating: bool,
    /// 남은 식사 시간
    pub eating_time: i32,
    /// 특수능력 쿨다운 남은
    pub spec_used: i32,
    /// 길들이기 감소량
    pub tame_reduction: i32,
    /// 완전 야생화
    pub gone_wild: bool,
    /// 적대화
    pub gone_hostile: bool,
    /// HP 회복량
    pub hp_regen: i32,
}

/// [v2.9.4] 시간 경과 따라잡기 (원본: mon_catchup_elapsed_time L463-561)
pub fn catchup_elapsed_time(
    nmv: i64,
    blinded: i32,
    frozen: i32,
    flee_time: i32,
    is_trapped: bool,
    is_confused: bool,
    is_stunned: bool,
    eating: i32,
    spec_used: i32,
    tame: i32,
    hp: i32,
    maxhp: i32,
    regenerates: bool,
    rng: &mut NetHackRng,
) -> CatchupResult {
    let imv = nmv.min(i32::MAX as i64 - 1) as i32;

    // 실명/동결/도주 감소 (원본: L488-505)
    let new_blind = if blinded > 0 {
        if imv >= blinded {
            1
        } else {
            blinded - imv
        }
    } else {
        0
    };
    let new_frozen = if frozen > 0 {
        if imv >= frozen {
            1
        } else {
            frozen - imv
        }
    } else {
        0
    };
    let new_flee = if flee_time > 0 {
        if imv >= flee_time {
            1
        } else {
            flee_time - imv
        }
    } else {
        0
    };

    // 트랩/혼란/기절 (원본: L508-513)
    let freed = is_trapped && rng.rn2(imv + 1) > 20;
    let conf_clear = is_confused && rng.rn2(imv + 1) > 25;
    let stun_clear = is_stunned && rng.rn2(imv + 1) > 5;

    // 식사/특수 (원본: L516-523)
    let (finish_eat, eat_t) = if eating > 0 {
        if imv > eating {
            (true, 0)
        } else {
            (false, eating - imv)
        }
    } else {
        (false, 0)
    };
    let new_spec = if spec_used > 0 {
        if imv > spec_used {
            0
        } else {
            spec_used - imv
        }
    } else {
        0
    };

    // 길들이기 감소 (원본: L526-534)
    let wilder = (imv + 75) / 150;
    let (tame_red, wild, hostile) = if tame > 0 && wilder > 0 {
        if tame > wilder {
            (wilder, false, false)
        } else if tame > rng.rn2(wilder) {
            (tame, true, false) // untame
        } else {
            (tame, true, true) // hostile
        }
    } else {
        (0, false, false)
    };

    // HP 회복 (원본: L554-560)
    let hp_factor = if regenerates { imv } else { imv / 20 };
    let hp_gain = (maxhp - hp).min(hp_factor).max(0);

    CatchupResult {
        blinded: new_blind,
        frozen: new_frozen,
        flee_time: new_flee,
        freed_from_trap: freed,
        confusion_cleared: conf_clear,
        stun_cleared: stun_clear,
        finish_eating: finish_eat,
        eating_time: eat_t,
        spec_used: new_spec,
        tame_reduction: tame_red,
        gone_wild: wild,
        gone_hostile: hostile,
        hp_regen: hp_gain,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// could_reach_item / can_reach_location (원본: dogmove.c L1268-1321)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 아이템 접근 가능 여부 (원본: could_reach_item L1268-1279)
pub fn could_reach_item(
    is_pool: bool,
    is_swimmer: bool,
    is_lava: bool,
    likes_lava: bool,
    has_boulder: bool,
    throws_rocks: bool,
) -> bool {
    (is_pool == false || is_swimmer)
        && (is_lava == false || likes_lava)
        && (has_boulder == false || throws_rocks)
}

/// [v2.9.4] 도달 가능성 (원본: can_reach_location L1288-1321 — 간소화)
pub fn can_reach_location_simple(mx: i32, my: i32, fx: i32, fy: i32, max_depth: i32) -> bool {
    if mx == fx && my == fy {
        return true;
    }
    if max_depth <= 0 {
        return false;
    }

    let dist = (mx - fx) * (mx - fx) + (my - fy) * (my - fy);
    // 체비셰프 1칸 이내면 도달
    if (mx - fx).abs() <= 1 && (my - fy).abs() <= 1 {
        return true;
    }
    // 거리 감소 방향으로 진행
    let dx = (fx - mx).signum();
    let dy = (fy - my).signum();
    let nx = mx + dx;
    let ny = my + dy;
    let new_dist = (nx - fx) * (nx - fx) + (ny - fy) * (ny - fy);
    new_dist < dist && can_reach_location_simple(nx, ny, fx, fy, max_depth - 1)
}

// ─────────────────────────────────────────────────────────────────────────────
// score_targ 결과 (원본: dogmove.c L708-807)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 타겟 점수 (원본: score_targ L708-807)
pub fn score_target(
    pet_level: i32,
    target_level: i32,
    target_hp: i32,
    target_is_pet: bool,
    target_is_peaceful: bool,
    target_is_passive: bool,
    target_is_leader: bool,
    target_is_guardian: bool,
    is_adjacent: bool,
    friends_behind: bool,
    pet_confused: bool,
    pet_vampshifter_level: Option<i32>, // 뱀파이어시프터 원래 레벨
    player_level: i32,
    rng: &mut NetHackRng,
) -> i64 {
    // 혼란 — 1/3 안전 (원본: L720)
    if pet_confused && rng.rn2(3) != 0 {
        return -(1000 + rng.rnd(5) as i64);
    }

    // 절대 불가 대상 (원본: L738-761)
    if target_is_leader || target_is_guardian {
        return -5000;
    }
    if target_is_pet {
        return -3000;
    }
    if is_adjacent {
        return -3000;
    }
    if friends_behind {
        return -3000;
    }

    let mut score: i64 = 0;

    // 적대 선호 (원본: L764-765)
    if !target_is_peaceful {
        score += 10;
    }

    // 수동 공격 (원본: L767-768)
    if target_is_passive {
        score -= 1000;
    }

    // 약한 적 무시 (원본: L771-774)
    let pet_lv = pet_vampshifter_level.unwrap_or(pet_level);
    if (target_level < 2 && pet_lv > 5)
        || (pet_lv > 12
            && target_level < pet_lv - 9
            && player_level > 8
            && target_level < player_level - 7)
    {
        score -= 25;
    }

    // 강한 적 회피 (원본: L793-794)
    if target_level > pet_lv + 4 {
        score -= (target_level - pet_lv) as i64 * 20;
    }

    // 비프 보너스 (원본: L797-798)
    score += target_level as i64 * 2 + target_hp as i64 / 3;

    // 퍼즈 (원본: L802)
    score += rng.rnd(5) as i64;

    // 혼란 페널티 (원본: L804-805)
    if pet_confused && rng.rn2(3) == 0 {
        score -= 1000;
    }

    score
}

// =============================================================================
// [v2.9.4] dog.c + dogmove.c 2차 이식 테스트
// =============================================================================
#[cfg(test)]
mod dog_phase2_tests {
    use super::*;

    #[test]
    fn test_init_edog_domestic() {
        let r = init_edog_result(true, 12, 1000);
        assert_eq!(r.tame, 10);
        assert_eq!(r.apport, 12);
        assert_eq!(r.hungrytime, 2000);
    }

    #[test]
    fn test_init_edog_wild() {
        let r = init_edog_result(false, 8, 500);
        assert_eq!(r.tame, 5);
        assert_eq!(r.hungrytime, 1500);
    }

    #[test]
    fn test_dogfood_tripe_carnivore() {
        let mon = DogFoodInput {
            is_carnivore: true,
            is_herbivore: false,
            is_starving: false,
            is_blind: false,
            is_ghoul: false,
            is_undead: false,
            is_vampshifter: false,
            is_gelatinous_cube: false,
            is_metallivorous: false,
            is_rust_monster: false,
            is_yeti_family: false,
            is_humanoid: false,
            is_fungus: false,
            hates_silver: false,
            resists_ston: false,
            resists_acid: false,
            resists_poison: false,
            slimeproof: false,
            is_elf: false,
            same_race: false,
        };
        let food = FoodItemInfo {
            is_food: true,
            item_type: FoodType::Tripe,
            is_cursed: false,
            is_blessed: false,
            corpse_is_rider: false,
            corpse_touch_petrifies: false,
            corpse_is_vegan: false,
            corpse_is_acidic: false,
            corpse_is_poisonous: false,
            corpse_is_green_slime: false,
            corpse_is_lizard_lichen: false,
            corpse_is_old: false,
            is_organic: false,
            is_metallic: false,
            is_rustprone: false,
            is_erodeproof: false,
            is_silver: false,
            is_rock_class: false,
            is_quest_artifact: false,
            obj_resists95: false,
        };
        assert_eq!(dogfood_extended(&mon, &food), DogFood::Dogfood);
    }

    #[test]
    fn test_dogfood_garlic_undead() {
        let mon = DogFoodInput {
            is_carnivore: false,
            is_herbivore: true,
            is_starving: false,
            is_blind: false,
            is_ghoul: false,
            is_undead: true,
            is_vampshifter: false,
            is_gelatinous_cube: false,
            is_metallivorous: false,
            is_rust_monster: false,
            is_yeti_family: false,
            is_humanoid: false,
            is_fungus: false,
            hates_silver: false,
            resists_ston: false,
            resists_acid: false,
            resists_poison: false,
            slimeproof: false,
            is_elf: false,
            same_race: false,
        };
        let food = FoodItemInfo {
            is_food: true,
            item_type: FoodType::Garlic,
            is_cursed: false,
            is_blessed: false,
            corpse_is_rider: false,
            corpse_touch_petrifies: false,
            corpse_is_vegan: false,
            corpse_is_acidic: false,
            corpse_is_poisonous: false,
            corpse_is_green_slime: false,
            corpse_is_lizard_lichen: false,
            corpse_is_old: false,
            is_organic: false,
            is_metallic: false,
            is_rustprone: false,
            is_erodeproof: false,
            is_silver: false,
            is_rock_class: false,
            is_quest_artifact: false,
            obj_resists95: false,
        };
        assert_eq!(dogfood_extended(&mon, &food), DogFood::Tabu);
    }

    #[test]
    fn test_dogfood_apple_herbi() {
        let mon = DogFoodInput {
            is_carnivore: false,
            is_herbivore: true,
            is_starving: false,
            is_blind: false,
            is_ghoul: false,
            is_undead: false,
            is_vampshifter: false,
            is_gelatinous_cube: false,
            is_metallivorous: false,
            is_rust_monster: false,
            is_yeti_family: false,
            is_humanoid: false,
            is_fungus: false,
            hates_silver: false,
            resists_ston: false,
            resists_acid: false,
            resists_poison: false,
            slimeproof: false,
            is_elf: false,
            same_race: false,
        };
        let food = FoodItemInfo {
            is_food: true,
            item_type: FoodType::Apple,
            is_cursed: false,
            is_blessed: false,
            corpse_is_rider: false,
            corpse_touch_petrifies: false,
            corpse_is_vegan: false,
            corpse_is_acidic: false,
            corpse_is_poisonous: false,
            corpse_is_green_slime: false,
            corpse_is_lizard_lichen: false,
            corpse_is_old: false,
            is_organic: false,
            is_metallic: false,
            is_rustprone: false,
            is_erodeproof: false,
            is_silver: false,
            is_rock_class: false,
            is_quest_artifact: false,
            obj_resists95: false,
        };
        assert_eq!(dogfood_extended(&mon, &food), DogFood::Dogfood);
    }

    #[test]
    fn test_dog_nutrition_corpse() {
        let r = dog_nutrition_result(
            true,
            true,
            2,
            100,
            0,
            0,
            false,
            0,
            0,
            0,
            MonSize::Medium,
            false,
            0,
        );
        assert_eq!(r.eating_time, 5); // 3+2
        assert_eq!(r.nutrition, 500); // 100*5
    }

    #[test]
    fn test_dog_nutrition_tiny() {
        let r = dog_nutrition_result(
            true,
            false,
            0,
            0,
            3,
            50,
            false,
            0,
            0,
            0,
            MonSize::Tiny,
            false,
            0,
        );
        assert_eq!(r.nutrition, 400); // 50*8
    }

    #[test]
    fn test_dog_eat_devour() {
        let r = dog_eat_result(
            100, true, false, 0, 15, 0, false, false, false, false, false, false, false, false,
            false, false,
        );
        assert_eq!(r.nutrition_gained, 75); // 100*3/4
        assert!(r.devour);
    }

    #[test]
    fn test_dog_eat_carrot_blindness() {
        let r = dog_eat_result(
            50, false, false, 0, 10, 0, false, false, false, false, true, false, false, false,
            false, true,
        );
        assert!(r.cure_blindness);
    }

    #[test]
    fn test_hunger_ok() {
        let r = dog_hunger_result(1000, 800, 30, 20, 0, true, false);
        assert_eq!(r, HungerResult::Ok);
    }

    #[test]
    fn test_hunger_confused() {
        let r = dog_hunger_result(1600, 1000, 30, 20, 0, true, false);
        match r {
            HungerResult::ConfusedFromHunger { new_maxhp, penalty } => {
                assert_eq!(new_maxhp, 10);
                assert_eq!(penalty, 20);
            }
            _ => panic!("기아 혼란 예상"),
        }
    }

    #[test]
    fn test_hunger_starved() {
        let r = dog_hunger_result(1800, 1000, 30, 20, 10, true, false);
        assert_eq!(r, HungerResult::Starved);
    }

    #[test]
    fn test_tamedog_wiz() {
        let mut rng = NetHackRng::new(42);
        let r = tamedog_result(
            true,
            false,
            false,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            DogFood::Undef,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            &mut rng,
        );
        assert_eq!(r, TameDogResult::Refused);
    }

    #[test]
    fn test_tamedog_success() {
        let mut rng = NetHackRng::new(42);
        let r = tamedog_result(
            false,
            false,
            false,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            DogFood::Undef,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            &mut rng,
        );
        assert_eq!(r, TameDogResult::Tamed);
    }

    #[test]
    fn test_tamedog_ate() {
        let mut rng = NetHackRng::new(42);
        let r = tamedog_result(
            false,
            false,
            false,
            false,
            false,
            true,
            true,
            false,
            false,
            true,
            DogFood::Dogfood,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            &mut rng,
        );
        assert_eq!(r, TameDogResult::AteFood);
    }

    #[test]
    fn test_wary_dog_killed_by_u() {
        let mut rng = NetHackRng::new(42);
        let r = wary_dog_result(10, true, true, 0, 5, false, false, false, &mut rng);
        assert_eq!(r.new_tame, 0);
        assert!(r.restore_hp_penalty);
    }

    #[test]
    fn test_wary_dog_normal() {
        let mut rng = NetHackRng::new(99);
        let r = wary_dog_result(10, true, false, 0, 0, false, false, false, &mut rng);
        // Pet Sematary: rn2(11) → 0-10
        assert!(r.new_tame >= 0);
    }

    #[test]
    fn test_abuse_dog_aggravate() {
        let mut rng = NetHackRng::new(42);
        let r = abuse_dog_result(10, true, false, false, true, &mut rng);
        assert_eq!(r.new_tame, 5); // 10/2
    }

    #[test]
    fn test_abuse_dog_normal() {
        let mut rng = NetHackRng::new(42);
        let r = abuse_dog_result(5, false, false, false, true, &mut rng);
        assert_eq!(r.new_tame, 4); // 5-1
        assert!(r.abuse_increment);
    }

    #[test]
    fn test_abuse_dog_untame() {
        let mut rng = NetHackRng::new(42);
        let r = abuse_dog_result(1, false, false, true, true, &mut rng);
        assert_eq!(r.new_tame, 0);
        assert!(r.unleash);
    }

    #[test]
    fn test_catchup_basic() {
        let mut rng = NetHackRng::new(42);
        let r = catchup_elapsed_time(
            100, 50, 30, 20, true, true, true, 10, 5, 10, 20, 30, false, &mut rng,
        );
        assert!(r.blinded < 50);
        assert!(r.frozen < 30);
        assert!(r.flee_time < 20);
        assert_eq!(r.spec_used, 0); // 100 > 5
        assert!(r.hp_regen >= 0);
    }

    #[test]
    fn test_catchup_regenerates() {
        let mut rng = NetHackRng::new(42);
        let r = catchup_elapsed_time(
            100, 0, 0, 0, false, false, false, 0, 0, 10, 10, 50, true, &mut rng,
        );
        assert_eq!(r.hp_regen, 40); // min(50-10, 100)
    }

    #[test]
    fn test_could_reach_item_pool() {
        assert!(!could_reach_item(true, false, false, false, false, false));
        assert!(could_reach_item(true, true, false, false, false, false));
    }

    #[test]
    fn test_could_reach_item_boulder() {
        assert!(!could_reach_item(false, false, false, false, true, false));
        assert!(could_reach_item(false, false, false, false, true, true));
    }

    #[test]
    fn test_can_reach_simple() {
        assert!(can_reach_location_simple(5, 5, 5, 5, 5));
        assert!(can_reach_location_simple(5, 5, 6, 6, 5));
        assert!(can_reach_location_simple(5, 5, 7, 7, 5));
    }

    #[test]
    fn test_score_target_pet() {
        let mut rng = NetHackRng::new(42);
        let s = score_target(
            5, 3, 20, true, false, false, false, false, false, false, false, None, 10, &mut rng,
        );
        assert_eq!(s, -3000);
    }

    #[test]
    fn test_score_target_leader() {
        let mut rng = NetHackRng::new(42);
        let s = score_target(
            5, 3, 20, false, false, false, true, false, false, false, false, None, 10, &mut rng,
        );
        assert_eq!(s, -5000);
    }

    #[test]
    fn test_score_target_hostile() {
        let mut rng = NetHackRng::new(42);
        let s = score_target(
            5, 3, 20, false, false, false, false, false, false, false, false, None, 10, &mut rng,
        );
        assert!(s > 0); // 적대 + 비프 보너스
    }

    #[test]
    fn test_score_target_vastly_stronger() {
        let mut rng = NetHackRng::new(42);
        let s = score_target(
            3, 15, 100, false, false, false, false, false, false, false, false, None, 10, &mut rng,
        );
        // 강한 적 → 큰 감점
        assert!(s < 0);
    }

    #[test]
    fn test_dogfood_metal_rust() {
        let mon = DogFoodInput {
            is_carnivore: false,
            is_herbivore: false,
            is_starving: false,
            is_blind: false,
            is_ghoul: false,
            is_undead: false,
            is_vampshifter: false,
            is_gelatinous_cube: false,
            is_metallivorous: true,
            is_rust_monster: true,
            is_yeti_family: false,
            is_humanoid: false,
            is_fungus: false,
            hates_silver: false,
            resists_ston: false,
            resists_acid: false,
            resists_poison: false,
            slimeproof: false,
            is_elf: false,
            same_race: false,
        };
        let food = FoodItemInfo {
            is_food: false,
            item_type: FoodType::Other,
            is_cursed: false,
            is_blessed: false,
            corpse_is_rider: false,
            corpse_touch_petrifies: false,
            corpse_is_vegan: false,
            corpse_is_acidic: false,
            corpse_is_poisonous: false,
            corpse_is_green_slime: false,
            corpse_is_lizard_lichen: false,
            corpse_is_old: false,
            is_organic: false,
            is_metallic: true,
            is_rustprone: true,
            is_erodeproof: false,
            is_silver: false,
            is_rock_class: false,
            is_quest_artifact: false,
            obj_resists95: false,
        };
        assert_eq!(dogfood_extended(&mon, &food), DogFood::Dogfood);
    }
}

// =============================================================================
// [v2.9.6] dog.c + dogmove.c 3차 이식 — 미구현 함수 완전 이식
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// droppables — 펫이 버릴 아이템 선택 (원본: dogmove.c L25-120)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 펫 인벤토리 아이템 분류 — droppables 판정에 사용
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PetItemType {
    /// 곡괭이류 (PICK_AXE, DWARVISH_MATTOCK)
    Pickaxe,
    /// 유니콘 뿔 (UNICORN_HORN)
    UnicornHorn,
    /// 열쇠류 (SKELETON_KEY, LOCK_PICK, CREDIT_CARD)
    Key,
    /// 매트록 (DWARVISH_MATTOCK — 이중 무기 방패 검사)
    Mattock,
    /// 기타 아이템
    Other,
}

/// [v2.9.6] 펫 인벤토리 아이템 정보
#[derive(Debug, Clone)]
pub struct PetInventoryItem {
    /// 아이템 인덱스 (인벤토리 내 순서)
    pub index: usize,
    /// 아이템 유형
    pub item_type: PetItemType,
    /// 아티팩트 여부
    pub is_artifact: bool,
    /// 저주 여부
    pub is_cursed: bool,
    /// 장착 중인지
    pub is_worn: bool,
    /// 현재 무기인지
    pub is_wielded: bool,
}

/// [v2.9.6] 펫 드롭 대상 판정 입력 (원본: droppables L25-120)
#[derive(Debug, Clone)]
pub struct DroppablesInput {
    /// 동물이거나 지성 없는 몬스터인지 (is_animal || mindless)
    pub is_mindless_or_animal: bool,
    /// 터널링 가능 + 곡괭이 필요 (tunnels && needspick)
    pub needs_pickaxe: bool,
    /// 문 열기 가능 (nohands || verysmall → 불가)
    pub can_use_key: bool,
    /// 방패 착용 중 (which_armor(W_ARMS) — 매트록 거부 조건)
    pub has_shield: bool,
    /// 인벤토리 아이템 목록
    pub inventory: Vec<PetInventoryItem>,
}

/// [v2.9.6] 버릴 아이템 인덱스 반환 (원본: droppables L25-120)
/// 반환값: Some(index) = 해당 아이템을 버림, None = 아무것도 버리지 않음
pub fn droppables(input: &DroppablesInput) -> Option<usize> {
    // 보유 도구 추적 (원본: pickaxe, unihorn, key 변수)
    let mut best_pick: Option<usize> = None;
    let mut best_horn: Option<usize> = None;
    let mut best_key: Option<usize> = None;

    // 동물/지성없는 몬스터는 도구를 유지하지 않음 (원본: L37-39)
    let keep_pickaxe = !input.is_mindless_or_animal && input.needs_pickaxe;
    let keep_key = !input.is_mindless_or_animal && input.can_use_key;

    // 장착 무기에서 기존 도구 확인 (원본: L48-54)
    for item in &input.inventory {
        if item.is_wielded {
            match item.item_type {
                PetItemType::Pickaxe | PetItemType::Mattock => {
                    if keep_pickaxe {
                        best_pick = Some(item.index);
                    }
                }
                PetItemType::UnicornHorn => {
                    best_horn = Some(item.index);
                }
                _ => {}
            }
        }
    }

    // 인벤토리 순회 — 최적 도구 유지, 나머지 버리기 (원본: L56-117)
    for item in &input.inventory {
        match item.item_type {
            PetItemType::Mattock => {
                // 방패 착용 중이면 매트록 거부 (원본: L60-61)
                if input.has_shield {
                    // 기타 아이템으로 폴스루
                } else if keep_pickaxe {
                    // 기존 곡괭이보다 매트록 선호 (원본: L64-66)
                    if let Some(prev) = best_pick {
                        let prev_item = input.inventory.iter().find(|i| i.index == prev);
                        if let Some(pi) = prev_item {
                            if pi.item_type == PetItemType::Pickaxe
                                && !pi.is_wielded
                                && (!pi.is_artifact || item.is_artifact)
                            {
                                return Some(prev); // 이전 곡괭이를 버림
                            }
                        }
                    }
                    // 더 좋은 곡괭이 유지 (원본: L69-74)
                    if best_pick.is_none()
                        || (item.is_artifact
                            && best_pick.map_or(false, |p| {
                                !input
                                    .inventory
                                    .iter()
                                    .find(|i| i.index == p)
                                    .map_or(false, |i| i.is_artifact)
                            }))
                    {
                        if let Some(old) = best_pick {
                            return Some(old);
                        }
                        best_pick = Some(item.index);
                        continue;
                    }
                }
            }
            PetItemType::Pickaxe => {
                if keep_pickaxe {
                    if best_pick.is_none()
                        || (item.is_artifact
                            && best_pick.map_or(false, |p| {
                                !input
                                    .inventory
                                    .iter()
                                    .find(|i| i.index == p)
                                    .map_or(false, |i| i.is_artifact)
                            }))
                    {
                        if let Some(old) = best_pick {
                            return Some(old);
                        }
                        best_pick = Some(item.index);
                        continue;
                    }
                }
            }
            PetItemType::UnicornHorn => {
                // 저주받은 유니콘 뿔은 거부 (원본: L79-80)
                if item.is_cursed {
                    // 폴스루
                } else {
                    if best_horn.is_none()
                        || (item.is_artifact
                            && best_horn.map_or(false, |h| {
                                !input
                                    .inventory
                                    .iter()
                                    .find(|i| i.index == h)
                                    .map_or(false, |i| i.is_artifact)
                            }))
                    {
                        if let Some(old) = best_horn {
                            return Some(old);
                        }
                        best_horn = Some(item.index);
                        continue;
                    }
                }
            }
            PetItemType::Key => {
                if keep_key {
                    // 열쇠류 우선순위: SKELETON_KEY > LOCK_PICK > CREDIT_CARD
                    // 간소화: 아티팩트 우선, 기존보다 나으면 교체 (원본: L90-108)
                    if best_key.is_none()
                        || (item.is_artifact
                            && best_key.map_or(false, |k| {
                                !input
                                    .inventory
                                    .iter()
                                    .find(|i| i.index == k)
                                    .map_or(false, |i| i.is_artifact)
                            }))
                    {
                        if let Some(old) = best_key {
                            return Some(old);
                        }
                        best_key = Some(item.index);
                        continue;
                    }
                }
            }
            PetItemType::Other => {}
        }

        // 장착/무기가 아닌 일반 아이템은 버림 (원본: L115-116)
        if !item.is_worn && !item.is_wielded {
            return Some(item.index);
        }
    }

    None // 버릴 것 없음 (원본: L119)
}

// ─────────────────────────────────────────────────────────────────────────────
// cursed_object_at — 해당 위치에 저주받은 아이템이 있는지 (원본: dogmove.c L129-139)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 해당 위치의 아이템 목록 중 저주받은 것이 있는지 확인
/// (원본: cursed_object_at L129-139)
pub fn cursed_object_at(items_cursed: &[bool]) -> bool {
    items_cursed.iter().any(|&c| c)
}

// ─────────────────────────────────────────────────────────────────────────────
// dog_invent — 펫 인벤토리 관리 (원본: dogmove.c L399-471)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 펫 인벤토리 관리 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DogInventResult {
    /// 아무 행동 없음 (수면/이동불가, 또는 인벤토리 관련 행동 없음)
    Nothing,
    /// 아이템을 버림 (droppables 결과 + apport 감소)
    DroppedItem {
        apport_decreased: bool,
        new_dropdist: i32,
        new_droptime: i64,
    },
    /// 바닥 음식을 먹었음 (dog_eat 호출 필요)
    AteFood,
    /// 바닥 아이템을 주웠음
    PickedUp,
}

/// [v2.9.6] 펫 인벤토리 관리 판정 입력
#[derive(Debug, Clone)]
pub struct DogInventInput {
    /// 수면 중인지
    pub is_sleeping: bool,
    /// 이동 가능한지
    pub can_move: bool,
    /// 현재 소지 아이템이 있는지 (droppables 결과)
    pub has_droppable: bool,
    /// 플레이어까지 거리
    pub udist: i32,
    /// apport 값
    pub apport: i32,
    /// 현재 위치 바닥 아이템의 음식 등급 (DogFood 순서)
    pub floor_food_quality: Option<DogFood>,
    /// 바닥 아이템이 접근 가능한지 (could_reach_item)
    pub floor_item_reachable: bool,
    /// 기아 상태인지 (mhpmax_penalty > 0)
    pub is_starving: bool,
    /// 바닥 아이템을 들 수 있는 용량
    pub can_carry_amount: i32,
    /// 바닥 아이템이 저주받았는지
    pub floor_item_cursed: bool,
    /// 바닥 아이템이 nofetch 클래스인지 (BALL/CHAIN/ROCK)
    pub floor_item_nofetch: bool,
    /// 현재 턴
    pub current_turn: i64,
}

/// [v2.9.6] 펫 인벤토리 관리 판정 (원본: dog_invent L399-471)
pub fn dog_invent_result(input: &DogInventInput, rng: &mut NetHackRng) -> DogInventResult {
    // 수면/이동불가 시 행동 없음 (원본: L411-412)
    if input.is_sleeping || !input.can_move {
        return DogInventResult::Nothing;
    }

    // 소지품 버리기 판정 (원본: L421-429)
    if input.has_droppable {
        // 판정: 1/udist+1 확률 또는 1/apport 확률 (원본: L422)
        let drop_chance1 = rng.rn2(input.udist + 1) == 0;
        let drop_chance2 = input.apport > 0 && rng.rn2(input.apport) == 0;
        if drop_chance1 || drop_chance2 {
            // rn2(10) < apport 면 실제 버리기 (원본: L423)
            if rng.rn2(10) < input.apport {
                return DogInventResult::DroppedItem {
                    apport_decreased: input.apport > 1,
                    new_dropdist: input.udist,
                    new_droptime: input.current_turn,
                };
            }
        }
        return DogInventResult::Nothing;
    }

    // 바닥 아이템 처리 — nofetch 제외 (원본: L431-469)
    if input.floor_item_nofetch {
        return DogInventResult::Nothing;
    }

    if let Some(food_quality) = &input.floor_food_quality {
        // 먹을 수 있는 음식인지 판정 (원본: L439-443)
        let edible_enough = *food_quality <= DogFood::Cadaver
            || (input.is_starving && *food_quality == DogFood::Accfood);
        if edible_enough && input.floor_item_reachable {
            return DogInventResult::AteFood;
        }
    }

    // 음식이 아니거나 먹지 않은 경우 — 줍기 시도 (원본: L445-467)
    if input.can_carry_amount > 0 && !input.floor_item_cursed && input.floor_item_reachable {
        // apport+3 vs rn2(20) 판정 (원본: L448)
        if rng.rn2(20) < input.apport + 3 {
            // udist vs rn2(udist) || !rn2(apport) (원본: L449)
            let pick_chance = rng.rn2(input.udist.max(1)) != 0
                || (input.apport > 0 && rng.rn2(input.apport) == 0);
            if pick_chance {
                return DogInventResult::PickedUp;
            }
        }
    }

    DogInventResult::Nothing
}

// ─────────────────────────────────────────────────────────────────────────────
// dog_goal — 펫 목표 설정 (원본: dogmove.c L473-617)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 펫 목표 설정 결과
#[derive(Debug, Clone, PartialEq)]
pub struct DogGoalResult {
    /// 목표 좌표 (gx, gy)
    pub goal_x: i32,
    pub goal_y: i32,
    /// 목표 유형 (DogFood enum 재활용 — 음식/운반/미정)
    pub goal_type: DogFood,
    /// 접근 의사 결정 (-1=회피, 0=중립, 1=접근) 또는 -2(이동 중단)
    pub approach: i32,
}

/// [v2.9.6] 펫 목표 설정 입력
#[derive(Debug, Clone)]
pub struct DogGoalInput {
    /// 탈것인지 (usteed) — true이면 -2 반환
    pub is_steed: bool,
    /// 펫 위치
    pub pet_x: i32,
    pub pet_y: i32,
    /// 플레이어 위치
    pub player_x: i32,
    pub player_y: i32,
    /// 목줄 착용 여부
    pub is_leashed: bool,
    /// edog 존재 여부 (미니언은 없음)
    pub has_edog: bool,
    /// 플레이어 시야 내인지 (couldsee)
    pub in_masters_sight: bool,
    /// 소지품 있는지 (droppables != null)
    pub has_droppable: bool,
    /// after 플래그 (빠른 이동)
    pub after: bool,
    /// 플레이어까지 거리 (distu)
    pub udist: i32,
    /// 휘파람 직후인지 (whappr)
    pub whappr: bool,
    /// 도주 중인지
    pub is_fleeing: bool,
    /// 혼란 상태인지
    pub is_confused: bool,
    /// 배고프지 않은지 (hungrytime > monstermoves)
    pub not_hungry: bool,
    /// 플레이어가 개밥을 갖고 있는지
    pub player_has_dogfood: bool,
    /// 주변 음식 목록: (x, y, food_quality, reachable)
    pub nearby_food: Vec<(i32, i32, DogFood, bool)>,
    /// 주변 운반 가능 아이템: (x, y, can_carry, can_see)
    pub nearby_items: Vec<(i32, i32, bool, bool)>,
    /// apport 값
    pub apport: i32,
    /// 플레이어 위치가 방 안인지
    pub player_in_room: bool,
}

/// [v2.9.6] 펫 목표 설정 판정 (원본: dog_goal L473-617)
pub fn dog_goal_result(input: &DogGoalInput, rng: &mut NetHackRng) -> DogGoalResult {
    // 탈것은 독자 이동 안 함 (원본: L488-489)
    if input.is_steed {
        return DogGoalResult {
            goal_x: input.player_x,
            goal_y: input.player_y,
            goal_type: DogFood::Apport,
            approach: -2,
        };
    }

    let mut gx;
    let mut gy;
    let mut gtyp;

    // 목줄 또는 edog 없음 → 플레이어 따라감 (원본: L497-500)
    if !input.has_edog || input.is_leashed {
        gx = input.player_x;
        gy = input.player_y;
        gtyp = DogFood::Apport;
    } else {
        // 주변 음식/아이템 탐색 (원본: L507-553)
        gtyp = DogFood::Undef;
        gx = 0;
        gy = 0;

        // 음식 우선 탐색 — SQSRCHRADIUS=5 이내 (원본: L520-553)
        for &(nx, ny, ref otyp, reachable) in &input.nearby_food {
            // 현재 목표보다 열등하면 스킵 (원본: L526-527)
            if *otyp > gtyp || *otyp == DogFood::Undef {
                continue;
            }
            // 접근 불가면 스킵 (원본: L533-535)
            if !reachable {
                continue;
            }

            if *otyp < DogFood::Manfood {
                // 더 가까운/더 좋은 음식 (원본: L537-541)
                let new_dist = (nx - input.pet_x).pow(2) + (ny - input.pet_y).pow(2);
                let old_dist = (gx - input.pet_x).pow(2) + (gy - input.pet_y).pow(2);
                if *otyp < gtyp || new_dist < old_dist {
                    gx = nx;
                    gy = ny;
                    gtyp = *otyp;
                }
            }
        }

        // 음식 없으면 운반 아이템 탐색 (원본: L542-551)
        if gtyp == DogFood::Undef && input.in_masters_sight && !input.has_droppable {
            for &(nx, ny, can_carry, can_see) in &input.nearby_items {
                if !can_carry || !can_see {
                    continue;
                }
                if input.apport > rng.rn2(8) {
                    gx = nx;
                    gy = ny;
                    gtyp = DogFood::Apport;
                    break;
                }
            }
        }
    }

    // 플레이어 따라가기 판정 (원본: L556-577)
    let appr;
    if gtyp == DogFood::Undef
        || (gtyp != DogFood::Dogfood && gtyp != DogFood::Apport && input.not_hungry)
    {
        gx = input.player_x;
        gy = input.player_y;

        // after && udist <= 4 이면 이동 중단 (원본: L561-562)
        if input.after && input.udist <= 4 {
            return DogGoalResult {
                goal_x: gx,
                goal_y: gy,
                goal_type: gtyp,
                approach: -2,
            };
        }

        // 접근 의사 결정 (원본: L563-575)
        appr = if input.udist >= 9 {
            1
        } else if input.is_fleeing {
            -1
        } else {
            let mut a = 0;
            if input.udist > 1 {
                if !input.player_in_room
                    || rng.rn2(4) == 0
                    || input.whappr
                    || (input.has_droppable && input.apport > 0 && rng.rn2(input.apport) != 0)
                {
                    a = 1;
                }
            }
            // 플레이어가 개밥 소지 시 접근 (원본: L570-575)
            if a == 0 && input.player_has_dogfood {
                a = 1;
            }
            a
        };
    } else {
        appr = 1; // 목표 있음 (원본: L577)
    }

    // 혼란 시 중립 (원본: L578-579)
    let final_appr = if input.is_confused { 0 } else { appr };

    DogGoalResult {
        goal_x: gx,
        goal_y: gy,
        goal_type: gtyp,
        approach: final_appr,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// find_targ — 방향별 타겟 탐색 (원본: dogmove.c L619-660)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 라인 위의 몬스터 정보
#[derive(Debug, Clone)]
pub struct LineSightTarget {
    /// 타겟의 인덱스 또는 식별자
    pub target_id: usize,
    /// 플레이어인지
    pub is_player: bool,
    /// 탐지까지 거리
    pub distance: i32,
}

/// [v2.9.6] 직선 탐색 입력 — 한 방향의 탐색 정보
#[derive(Debug, Clone)]
pub struct LineSightQuery {
    /// 방향 (dx, dy)
    pub dx: i32,
    pub dy: i32,
    /// 최대 거리
    pub max_dist: i32,
    /// 각 칸의 정보: (valid, can_see, has_player, monster_id, monster_visible)
    /// monster_id=None이면 몬스터 없음
    pub cells: Vec<LineSightCell>,
}

/// [v2.9.6] 직선상 한 칸 정보
#[derive(Debug, Clone)]
pub struct LineSightCell {
    /// 유효한 좌표인지 (isok)
    pub is_valid: bool,
    /// 펫이 해당 칸을 볼 수 있는지 (m_cansee)
    pub pet_can_see: bool,
    /// 플레이어가 해당 위치에 있는지
    pub is_player_pos: bool,
    /// 몬스터 ID (있으면 Some)
    pub monster_id: Option<usize>,
    /// 몬스터가 펫에게 보이는지 (투명 아님 또는 perceives)
    pub monster_visible: bool,
}

/// [v2.9.6] 방향별 타겟 탐색 (원본: find_targ L619-660)
pub fn find_targ_result(query: &LineSightQuery) -> Option<LineSightTarget> {
    for (dist, cell) in query.cells.iter().enumerate() {
        if !cell.is_valid {
            break;
        }
        // 시야 밖이면 중단 (원본: L644-645)
        if !cell.pet_can_see {
            break;
        }
        // 플레이어 위치이면 플레이어 반환 (원본: L647-648)
        if cell.is_player_pos {
            return Some(LineSightTarget {
                target_id: 0,
                is_player: true,
                distance: dist as i32 + 1,
            });
        }
        // 몬스터가 있고 보이면 반환 (원본: L650-657)
        if let Some(mid) = cell.monster_id {
            if cell.monster_visible {
                return Some(LineSightTarget {
                    target_id: mid,
                    is_player: false,
                    distance: dist as i32 + 1,
                });
            }
            // 안 보이면 없는 것으로 취급 (원본: L656)
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────────
// find_friends — 타겟 뒤의 아군 검색 (원본: dogmove.c L662-706)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 타겟 뒤 아군 검색 입력
#[derive(Debug, Clone)]
pub struct FindFriendsQuery {
    /// 타겟 뒤쪽 방향의 칸 정보
    /// (valid, pet_can_see, is_player_pos, is_tame, is_visible, is_leader_or_guardian)
    pub cells_behind: Vec<FriendCell>,
}

/// [v2.9.6] 타겟 뒤 한 칸 정보
#[derive(Debug, Clone)]
pub struct FriendCell {
    pub is_valid: bool,
    pub pet_can_see: bool,
    pub is_player_pos: bool,
    pub has_monster: bool,
    pub is_tame: bool,
    pub is_visible: bool,
    pub is_leader_or_guardian: bool,
}

/// [v2.9.6] 타겟 뒤에 아군이 있는지 검사 (원본: find_friends L662-706)
pub fn find_friends_result(query: &FindFriendsQuery) -> bool {
    for cell in &query.cells_behind {
        if !cell.is_valid {
            return false;
        }
        // 시야 밖이면 중단 (원본: L683-684)
        if !cell.pet_can_see {
            return false;
        }
        // 플레이어 위치이면 아군 (원본: L687-688)
        if cell.is_player_pos {
            return true;
        }
        // 몬스터가 있는 경우 (원본: L690-703)
        if cell.has_monster {
            if cell.is_tame {
                // 보이는 아군 → 공격 금지 (원본: L694-696)
                if cell.is_visible {
                    return true;
                }
            } else if cell.is_leader_or_guardian {
                // 퀘스트 리더/가디언은 항상 아군 취급 (원본: L699-701)
                return true;
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// best_target — 최적 공격 대상 선택 (원본: dogmove.c L809-858)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 최적 타겟 후보
#[derive(Debug, Clone)]
pub struct TargetCandidate {
    /// 타겟 식별자
    pub target_id: usize,
    /// 플레이어인지
    pub is_player: bool,
    /// 해당 타겟의 점수
    pub score: i64,
}

/// [v2.9.6] 최적 타겟 선택 (원본: best_target L809-858)
/// 입력: 8방향 탐색 결과 (find_targ + score_targ 결과 합산)
/// 반환: 점수 > 0인 최고 점수 타겟, 없으면 None
pub fn best_target_result(pet_can_see: bool, candidates: &[TargetCandidate]) -> Option<usize> {
    // 시야 없으면 타겟 못 쏨 (원본: L822-823)
    if !pet_can_see {
        return None;
    }

    let mut best_score = -40000i64;
    let mut best_id: Option<usize> = None;

    for c in candidates {
        if c.score > best_score {
            best_score = c.score;
            best_id = Some(c.target_id);
        }
    }

    // 점수 < 0 이면 필터링 (원본: L854-855)
    if best_score < 0 {
        return None;
    }

    best_id
}

// ─────────────────────────────────────────────────────────────────────────────
// wantdoor — 근처 문 찾기 (원본: dogmove.c L1324-1336)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 근처 문 찾기 — 가장 가까운 문 좌표 반환 (원본: wantdoor L1324-1336)
/// doors: (x, y, player_distance_sq) 목록
pub fn wantdoor_result(doors: &[(i32, i32, i32)]) -> Option<(i32, i32)> {
    let mut best_dist = i32::MAX;
    let mut best_pos: Option<(i32, i32)> = None;

    for &(x, y, dist) in doors {
        if dist < best_dist {
            best_dist = dist;
            best_pos = Some((x, y));
        }
    }

    best_pos
}

// ─────────────────────────────────────────────────────────────────────────────
// finish_meating — 식사 완료 처리 (원본: dogmove.c L1357-1368)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 식사 완료 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishMeatingResult {
    /// 식사 시간 = 0으로 설정
    pub meating_cleared: bool,
    /// 외형 리셋 필요 (미믹 시체 먹기 후 복원)
    pub appearance_reset: bool,
}

/// [v2.9.6] 식사 완료 처리 (원본: finish_meating L1357-1368)
pub fn finish_meating_result(
    meating: i32,
    has_appearance: bool,
    has_mappearance: bool,
    cham_is_non_pm: bool,
) -> FinishMeatingResult {
    if meating <= 0 {
        return FinishMeatingResult {
            meating_cleared: false,
            appearance_reset: false,
        };
    }

    // 미믹 시체를 먹은 후 외형을 리셋해야 하는지 (원본: L1362-1367)
    let reset = has_appearance && has_mappearance && cham_is_non_pm;

    FinishMeatingResult {
        meating_cleared: true,
        appearance_reset: reset,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// quickmimic — 미믹 시체 먹은 후 변장 (원본: dogmove.c L1370-1432)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 미믹 변장 외형 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MimicAppearance {
    /// 몬스터로 변장 (M_AP_MONSTER)
    Monster(i32),
    /// 가구로 변장 (M_AP_FURNITURE — 예: SINK)
    Furniture(i32),
    /// 아이템으로 변장 (M_AP_OBJECT — 예: TRIPE_RATION)
    Object(i32),
}

/// [v2.9.6] 미믹 변장 후보
#[derive(Debug, Clone)]
pub struct QuickMimicChoice {
    /// 특정 몬스터 인덱스 (0이면 아무거나)
    pub mndx: i32,
    /// 특정 심볼 (0이면 아무거나)
    pub mlet: char,
    /// 변장 결과
    pub appearance: MimicAppearance,
}

/// [v2.9.6] 미믹 시체 먹은 후 변장 판정 (원본: quickmimic L1370-1432)
/// 반환: 변장할 외형, 불가능하면 None
pub fn quickmimic_result(
    protection_from_shapechangers: bool,
    meating: i32,
    monster_index: i32,
    monster_symbol: char,
    is_steed: bool,
    rng: &mut NetHackRng,
) -> Option<MimicAppearance> {
    // 형태변경 보호 또는 식사 안 함이면 불가 (원본: L1377-1378)
    if protection_from_shapechangers || meating <= 0 {
        return None;
    }

    // 펫별 변장 후보 테이블 (원본: qm[] L1338-1355)
    // 간소화: 핵심 패턴만 반영
    let candidates: Vec<(i32, char, MimicAppearance)> = vec![
        // 강아지 → 고양이 (원본: PM_LITTLE_DOG → PM_KITTEN)
        (1, '\0', MimicAppearance::Monster(2)),   // 개 계열
        (3, '\0', MimicAppearance::Monster(4)),   // 고양이 계열
        (0, 'd', MimicAppearance::Furniture(42)), // 개 → 세면대 (SINK)
        (0, '\0', MimicAppearance::Object(28)),   // 기본: TRIPE_RATION
    ];

    let mut trycnt = 5;
    let mut idx = candidates.len() - 1; // 기본값: 마지막 후보

    while trycnt > 0 {
        let trial = rng.rn2(candidates.len() as i32) as usize;
        let (mndx, mlet, _) = &candidates[trial];

        if *mndx != 0 && monster_index == *mndx {
            idx = trial;
            break;
        }
        if *mlet != '\0' && monster_symbol == *mlet {
            idx = trial;
            break;
        }
        if *mndx == 0 && *mlet == '\0' {
            idx = trial;
            break;
        }
        trycnt -= 1;
    }

    // 탈것이면 dismount 필요 (원본: L1386-1387) — 여기서는 판정만
    let _ = is_steed;

    Some(candidates[idx].2.clone())
}

// ─────────────────────────────────────────────────────────────────────────────
// make_familiar / makedog / keepdogs / losedogs / mon_arrive / migrate_to_level
// 이 함수들은 ECS 월드 조작이 필요하여 순수 함수로 이식 불가.
// ECS 시스템 레벨에서 구현되어야 하며, 여기서는 판정 로직만 제공.
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.6] 소환수 생성 판정 (원본: make_familiar L69-149 — 판정부만)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamiliarResult {
    /// 소환 성공 — 길들이기
    Tamed,
    /// 소환 성공 — 평화적 (축복되지 않은 피규어)
    Peaceful,
    /// 소환 성공 — 적대적 (저주받은 피규어)
    Hostile,
    /// 소환 실패 (멸종 등)
    Failed,
}

/// [v2.9.6] 소환수 성격 판정 (원본: make_familiar L122-135)
pub fn familiar_disposition(
    is_figurine: bool,
    is_blessed: bool,
    is_cursed: bool,
    rng: &mut NetHackRng,
) -> FamiliarResult {
    if !is_figurine {
        return FamiliarResult::Tamed; // 주문 소환은 항상 길들이기
    }

    // 피규어: 0=tame, 1=peaceful, 2=hostile (원본: L123-125)
    let mut chance = rng.rn2(10);
    if chance > 2 {
        chance = if is_blessed {
            0
        } else if !is_cursed {
            1
        } else {
            2
        };
    }

    match chance {
        0 => FamiliarResult::Tamed,
        1 => FamiliarResult::Peaceful,
        _ => FamiliarResult::Hostile,
    }
}

/// [v2.9.6] 초기 펫 타입 결정 (원본: pet_type L56-67)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartingPetType {
    LittleDog,
    Kitten,
    Pony,
}

/// [v2.9.6] 초기 펫 타입 판정
pub fn starting_pet_type(
    role_pet: Option<StartingPetType>,
    preferred: Option<char>,
    rng: &mut NetHackRng,
) -> StartingPetType {
    // 역할 지정 펫 우선 (원본: L59-60)
    if let Some(rp) = role_pet {
        return rp;
    }
    // 사용자 선호 (원본: L61-64)
    match preferred {
        Some('c') => StartingPetType::Kitten,
        Some('d') => StartingPetType::LittleDog,
        _ => {
            // 50/50 (원본: L66)
            if rng.rn2(2) != 0 {
                StartingPetType::Kitten
            } else {
                StartingPetType::LittleDog
            }
        }
    }
}

// =============================================================================
// [v2.9.6] 3차 이식 테스트
// =============================================================================
#[cfg(test)]
mod dog_phase3_tests {
    use super::*;

    #[test]
    fn test_droppables_no_items() {
        let input = DroppablesInput {
            is_mindless_or_animal: false,
            needs_pickaxe: false,
            can_use_key: true,
            has_shield: false,
            inventory: vec![],
        };
        assert_eq!(droppables(&input), None);
    }

    #[test]
    fn test_droppables_drops_unworn() {
        let input = DroppablesInput {
            is_mindless_or_animal: false,
            needs_pickaxe: false,
            can_use_key: true,
            has_shield: false,
            inventory: vec![PetInventoryItem {
                index: 0,
                item_type: PetItemType::Other,
                is_artifact: false,
                is_cursed: false,
                is_worn: false,
                is_wielded: false,
            }],
        };
        assert_eq!(droppables(&input), Some(0));
    }

    #[test]
    fn test_droppables_keeps_pickaxe() {
        let input = DroppablesInput {
            is_mindless_or_animal: false,
            needs_pickaxe: true,
            can_use_key: true,
            has_shield: false,
            inventory: vec![
                PetInventoryItem {
                    index: 0,
                    item_type: PetItemType::Pickaxe,
                    is_artifact: false,
                    is_cursed: false,
                    is_worn: false,
                    is_wielded: false,
                },
                PetInventoryItem {
                    index: 1,
                    item_type: PetItemType::Other,
                    is_artifact: false,
                    is_cursed: false,
                    is_worn: false,
                    is_wielded: false,
                },
            ],
        };
        // 곡괭이는 유지하고 기타 아이템을 버림
        assert_eq!(droppables(&input), Some(1));
    }

    #[test]
    fn test_cursed_object_at_true() {
        assert!(cursed_object_at(&[false, true, false]));
    }

    #[test]
    fn test_cursed_object_at_false() {
        assert!(!cursed_object_at(&[false, false, false]));
    }

    #[test]
    fn test_dog_invent_sleeping() {
        let mut rng = NetHackRng::new(42);
        let input = DogInventInput {
            is_sleeping: true,
            can_move: true,
            has_droppable: false,
            udist: 5,
            apport: 3,
            floor_food_quality: None,
            floor_item_reachable: false,
            is_starving: false,
            can_carry_amount: 0,
            floor_item_cursed: false,
            floor_item_nofetch: false,
            current_turn: 1000,
        };
        assert_eq!(
            dog_invent_result(&input, &mut rng),
            DogInventResult::Nothing
        );
    }

    #[test]
    fn test_dog_invent_eats_food() {
        let mut rng = NetHackRng::new(42);
        let input = DogInventInput {
            is_sleeping: false,
            can_move: true,
            has_droppable: false,
            udist: 5,
            apport: 3,
            floor_food_quality: Some(DogFood::Dogfood),
            floor_item_reachable: true,
            is_starving: false,
            can_carry_amount: 0,
            floor_item_cursed: false,
            floor_item_nofetch: false,
            current_turn: 1000,
        };
        assert_eq!(
            dog_invent_result(&input, &mut rng),
            DogInventResult::AteFood
        );
    }

    #[test]
    fn test_dog_goal_steed() {
        let mut rng = NetHackRng::new(42);
        let input = DogGoalInput {
            is_steed: true,
            pet_x: 5,
            pet_y: 5,
            player_x: 10,
            player_y: 10,
            is_leashed: false,
            has_edog: true,
            in_masters_sight: true,
            has_droppable: false,
            after: false,
            udist: 50,
            whappr: false,
            is_fleeing: false,
            is_confused: false,
            not_hungry: true,
            player_has_dogfood: false,
            nearby_food: vec![],
            nearby_items: vec![],
            apport: 3,
            player_in_room: true,
        };
        let r = dog_goal_result(&input, &mut rng);
        assert_eq!(r.approach, -2);
    }

    #[test]
    fn test_dog_goal_follows_player() {
        let mut rng = NetHackRng::new(42);
        let input = DogGoalInput {
            is_steed: false,
            pet_x: 5,
            pet_y: 5,
            player_x: 15,
            player_y: 15,
            is_leashed: false,
            has_edog: true,
            in_masters_sight: true,
            has_droppable: false,
            after: false,
            udist: 50,
            whappr: false,
            is_fleeing: false,
            is_confused: false,
            not_hungry: true,
            player_has_dogfood: false,
            nearby_food: vec![],
            nearby_items: vec![],
            apport: 3,
            player_in_room: true,
        };
        let r = dog_goal_result(&input, &mut rng);
        assert_eq!(r.goal_x, 15);
        assert_eq!(r.goal_y, 15);
        assert_eq!(r.approach, 1); // udist >= 9 → 접근
    }

    #[test]
    fn test_find_targ_player() {
        let query = LineSightQuery {
            dx: 1,
            dy: 0,
            max_dist: 7,
            cells: vec![
                LineSightCell {
                    is_valid: true,
                    pet_can_see: true,
                    is_player_pos: false,
                    monster_id: None,
                    monster_visible: false,
                },
                LineSightCell {
                    is_valid: true,
                    pet_can_see: true,
                    is_player_pos: true,
                    monster_id: None,
                    monster_visible: false,
                },
            ],
        };
        let r = find_targ_result(&query);
        assert!(r.is_some());
        assert!(r.unwrap().is_player);
    }

    #[test]
    fn test_find_targ_blind_stops() {
        let query = LineSightQuery {
            dx: 0,
            dy: 1,
            max_dist: 7,
            cells: vec![LineSightCell {
                is_valid: true,
                pet_can_see: false,
                is_player_pos: false,
                monster_id: Some(5),
                monster_visible: true,
            }],
        };
        assert!(find_targ_result(&query).is_none());
    }

    #[test]
    fn test_find_friends_player_behind() {
        let query = FindFriendsQuery {
            cells_behind: vec![FriendCell {
                is_valid: true,
                pet_can_see: true,
                is_player_pos: true,
                has_monster: false,
                is_tame: false,
                is_visible: false,
                is_leader_or_guardian: false,
            }],
        };
        assert!(find_friends_result(&query));
    }

    #[test]
    fn test_find_friends_no_one() {
        let query = FindFriendsQuery {
            cells_behind: vec![FriendCell {
                is_valid: true,
                pet_can_see: true,
                is_player_pos: false,
                has_monster: false,
                is_tame: false,
                is_visible: false,
                is_leader_or_guardian: false,
            }],
        };
        assert!(!find_friends_result(&query));
    }

    #[test]
    fn test_best_target_blind() {
        assert_eq!(best_target_result(false, &[]), None);
    }

    #[test]
    fn test_best_target_positive_score() {
        let candidates = vec![
            TargetCandidate {
                target_id: 1,
                is_player: false,
                score: -100,
            },
            TargetCandidate {
                target_id: 2,
                is_player: false,
                score: 50,
            },
            TargetCandidate {
                target_id: 3,
                is_player: false,
                score: 30,
            },
        ];
        assert_eq!(best_target_result(true, &candidates), Some(2));
    }

    #[test]
    fn test_best_target_all_negative() {
        let candidates = vec![
            TargetCandidate {
                target_id: 1,
                is_player: false,
                score: -100,
            },
            TargetCandidate {
                target_id: 2,
                is_player: false,
                score: -50,
            },
        ];
        assert_eq!(best_target_result(true, &candidates), None);
    }

    #[test]
    fn test_wantdoor_finds_closest() {
        let doors = vec![(5, 5, 10), (3, 3, 2), (8, 8, 20)];
        assert_eq!(wantdoor_result(&doors), Some((3, 3)));
    }

    #[test]
    fn test_finish_meating_no_eating() {
        let r = finish_meating_result(0, false, false, false);
        assert!(!r.meating_cleared);
    }

    #[test]
    fn test_finish_meating_with_reset() {
        let r = finish_meating_result(5, true, true, true);
        assert!(r.meating_cleared);
        assert!(r.appearance_reset);
    }

    #[test]
    fn test_quickmimic_protected() {
        let mut rng = NetHackRng::new(42);
        assert!(quickmimic_result(true, 5, 1, 'd', false, &mut rng).is_none());
    }

    #[test]
    fn test_quickmimic_normal() {
        let mut rng = NetHackRng::new(42);
        let r = quickmimic_result(false, 5, 1, 'd', false, &mut rng);
        assert!(r.is_some());
    }

    #[test]
    fn test_familiar_spell() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            familiar_disposition(false, false, false, &mut rng),
            FamiliarResult::Tamed
        );
    }

    #[test]
    fn test_familiar_cursed_figurine() {
        // 저주 피규어: 높은 확률 적대적 (80%)
        let mut rng = NetHackRng::new(42);
        let mut hostile_count = 0;
        for seed in 0..100 {
            let mut r = NetHackRng::new(seed);
            if familiar_disposition(true, false, true, &mut r) == FamiliarResult::Hostile {
                hostile_count += 1;
            }
        }
        // 대략 70-90% 적대적이어야 함
        assert!(
            hostile_count > 60,
            "적대 비율이 너무 낮음: {}",
            hostile_count
        );
    }

    #[test]
    fn test_starting_pet_role() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            starting_pet_type(Some(StartingPetType::Pony), None, &mut rng),
            StartingPetType::Pony
        );
    }

    #[test]
    fn test_starting_pet_preference() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            starting_pet_type(None, Some('c'), &mut rng),
            StartingPetType::Kitten
        );
        assert_eq!(
            starting_pet_type(None, Some('d'), &mut rng),
            StartingPetType::LittleDog
        );
    }
}
