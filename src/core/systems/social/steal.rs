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

use crate::core::entity::object::ItemClass;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StealType {
    Gold,
    Item,
    Armor,
    Seduce,
    All,
}

///
#[derive(Debug, Clone)]
pub struct StealResult {
    pub success: bool,
    pub stolen_item: Option<String>,
    pub stolen_gold: i32,
    pub armor_removed: bool,
    pub message: String,
}

impl StealResult {
    pub fn fail(msg: &str) -> Self {
        Self {
            success: false,
            stolen_item: None,
            stolen_gold: 0,
            armor_removed: false,
            message: msg.to_string(),
        }
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn steal_gold(
    player_gold: &mut i32,
    monster_name: &str,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> StealResult {
    if *player_gold <= 0 {
        let msg = format!(
            "The {} searches your pockets but finds nothing.",
            monster_name
        );
        log.add(&msg, turn);
        return StealResult::fail(&msg);
    }

    //
    let max_steal = (*player_gold / 4).max(10).min(*player_gold);
    let amount = rng.rn2(max_steal) + 1;

    *player_gold -= amount;
    let msg = format!(
        "The {} steals {} gold piece{}!",
        monster_name,
        amount,
        if amount != 1 { "s" } else { "" }
    );
    log.add_colored(&msg, [255, 200, 0], turn);

    StealResult {
        success: true,
        stolen_item: None,
        stolen_gold: amount,
        armor_removed: false,
        message: msg,
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn steal_item(
    items: &[(usize, String, ItemClass)],
    monster_name: &str,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> StealResult {
    if items.is_empty() {
        let msg = format!(
            "The {} pretends to search you but finds nothing interesting.",
            monster_name
        );
        log.add(&msg, turn);
        return StealResult::fail(&msg);
    }

    //
    let seduction_msgs = [
        format!("The {} smiles seductively...", monster_name),
        format!("The {} whispers sweet nothings...", monster_name),
        format!("The {} touches you gently...", monster_name),
    ];
    let sed_idx = rng.rn2(seduction_msgs.len() as i32) as usize;
    log.add(&seduction_msgs[sed_idx], turn);

    //
    let idx = rng.rn2(items.len() as i32) as usize;
    let (slot, ref name, _class) = items[idx];

    let msg = format!("The {} stole your {}!", monster_name, name);
    log.add_colored(&msg, [255, 100, 100], turn);

    StealResult {
        success: true,
        stolen_item: Some(name.clone()),
        stolen_gold: 0,
        armor_removed: false,
        message: msg,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn steal_amulet(
    has_amulet: bool,
    player_ac: i32,
    monster_name: &str,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> StealResult {
    //
    let difficulty = 10 + player_ac;
    if rng.rn2(difficulty.max(1)) != 0 {
        let msg = format!("The {} tries to steal something but fails.", monster_name);
        log.add(&msg, turn);
        return StealResult::fail(&msg);
    }

    if has_amulet {
        let msg = format!("The {} stole your Amulet of Yendor!", monster_name);
        log.add_colored(&msg, [255, 0, 0], turn);
        return StealResult {
            success: true,
            stolen_item: Some("Amulet of Yendor".to_string()),
            stolen_gold: 0,
            armor_removed: false,
            message: msg,
        };
    }

    StealResult::fail("The thief finds nothing of interest.")
}

///
///
pub fn steal_armor(
    worn_armor: &[(usize, String)],
    monster_name: &str,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> StealResult {
    if worn_armor.is_empty() {
        let msg = format!(
            "The {} grabs at you but you aren't wearing anything to steal.",
            monster_name
        );
        log.add(&msg, turn);
        return StealResult::fail(&msg);
    }

    //
    let idx = rng.rn2(worn_armor.len() as i32) as usize;
    let (slot, ref name) = worn_armor[idx];

    let msg = format!("The {} rips off your {}!", monster_name, name);
    log.add_colored(&msg, [255, 100, 100], turn);

    StealResult {
        success: true,
        stolen_item: Some(name.clone()),
        stolen_gold: 0,
        armor_removed: true,
        message: msg,
    }
}

// =============================================================================
//
// =============================================================================

///
///
pub fn seduce_attack(
    player_charisma: i32,
    player_wisdom: i32,
    monster_name: &str,
    worn_count: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> SeduceResult {
    //
    let resist_chance = player_wisdom / 3 + 5;
    if rng.rn2(resist_chance) > 3 {
        let msg = format!("You resist the {} seductive advances.", monster_name);
        log.add(&msg, turn);
        return SeduceResult {
            resisted: true,
            items_removed: 0,
            charm_stat_change: 0,
            message: msg,
        };
    }

    //
    let items_to_remove = (rng.rn2(3) + 1).min(worn_count);

    let seduction_dialogue = [
        format!(
            "\"Let me help you with that...\" says the {}.",
            monster_name
        ),
        format!("The {} gently unlatches your cloak.", monster_name),
        format!("\"You won't be needing that anymore...\""),
    ];

    for i in 0..items_to_remove.min(seduction_dialogue.len() as i32) {
        log.add(&seduction_dialogue[i as usize], turn);
    }

    //
    let charm_change = if rng.rn2(3) == 0 { 1 } else { 0 };

    let msg = format!("The {} finishes with you.", monster_name);
    log.add(&msg, turn);

    SeduceResult {
        resisted: false,
        items_removed: items_to_remove,
        charm_stat_change: charm_change,
        message: msg,
    }
}

///
#[derive(Debug, Clone)]
pub struct SeduceResult {
    pub resisted: bool,
    pub items_removed: i32,
    pub charm_stat_change: i32,
    pub message: String,
}

// =============================================================================
//
// =============================================================================

///
pub fn theft_protection_chance(
    player_dexterity: i32,
    player_level: i32,
    has_locking_device: bool,
) -> i32 {
    let mut chance = player_dexterity / 3 + player_level / 2;
    if has_locking_device {
        chance += 5;
    }
    chance
}

///
pub fn should_attempt_steal(
    monster_name: &str,
    dist_sq: i32,
    rng: &mut NetHackRng,
) -> Option<StealType> {
    //
    if dist_sq > 2 {
        return None;
    }

    //
    if monster_name.contains("leprechaun") {
        Some(StealType::Gold)
    } else if monster_name.contains("nymph") {
        Some(StealType::Item)
    } else if monster_name.contains("monkey") || monster_name.contains("ape") {
        if rng.rn2(3) == 0 {
            Some(StealType::Item)
        } else {
            None
        }
    } else if monster_name.contains("incubus") || monster_name.contains("succubus") {
        Some(StealType::Seduce)
    } else if monster_name.contains("Keystone Kop") {
        if rng.rn2(5) == 0 {
            Some(StealType::Gold)
        } else {
            None
        }
    } else {
        None
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn thief_death_drops(
    monster_name: &str,
    stolen_gold: i32,
    stolen_items: &[String],
) -> Vec<String> {
    let mut drops = Vec::new();

    //
    if stolen_gold > 0 {
        drops.push(format!(
            "{} gold piece{}",
            stolen_gold,
            if stolen_gold != 1 { "s" } else { "" }
        ));
    }

    //
    for item in stolen_items {
        drops.push(item.clone());
    }

    drops
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steal_gold() {
        let mut gold = 100;
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = steal_gold(&mut gold, "leprechaun", &mut rng, &mut log, 1);
        assert!(result.success);
        assert!(result.stolen_gold > 0);
        assert!(gold < 100);
    }

    #[test]
    fn test_steal_gold_no_gold() {
        let mut gold = 0;
        let mut rng = NetHackRng::new(42);
        let mut log = GameLog::new(100);
        let result = steal_gold(&mut gold, "leprechaun", &mut rng, &mut log, 1);
        assert!(!result.success);
    }

    #[test]
    fn test_should_attempt_steal() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            should_attempt_steal("leprechaun", 1, &mut rng),
            Some(StealType::Gold)
        );
        assert_eq!(
            should_attempt_steal("nymph", 1, &mut rng),
            Some(StealType::Item)
        );
        assert_eq!(should_attempt_steal("goblin", 1, &mut rng), None);
        assert_eq!(should_attempt_steal("leprechaun", 10, &mut rng), None);
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn steal_gold_amount(player_gold: u64, thief_level: i32, rng: &mut NetHackRng) -> u64 {
    if player_gold == 0 {
        return 0;
    }
    //
    let max_steal = (thief_level as u64 * 25).min(player_gold);
    let stolen = rng.rn2(max_steal as i32 + 1).max(1) as u64;
    stolen.min(player_gold)
}

///
pub fn leprechaun_steal(
    player_gold: u64,
    player_level: i32,
    thief_level: i32,
    rng: &mut NetHackRng,
) -> u64 {
    //
    let level_diff = (thief_level - player_level).max(0);
    let base = steal_gold_amount(player_gold, thief_level, rng);
    let bonus = if level_diff > 0 {
        rng.rn2(level_diff * 10 + 1) as u64
    } else {
        0
    };
    (base + bonus).min(player_gold)
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StealTarget {
    Weapon,
    Armor,
    Ring,
    Amulet,
    Tool,
    Food,
    RandomItem,
}

///
#[derive(Debug, Clone)]
pub struct StealItemResult {
    pub success: bool,
    pub item_class: StealTarget,
    pub cursed_block: bool,
    pub message: String,
}

///
pub fn nymph_target_selection(
    has_weapon: bool,
    has_armor: bool,
    has_ring: bool,
    has_amulet: bool,
    rng: &mut NetHackRng,
) -> StealTarget {
    //
    let mut candidates = Vec::new();
    if has_ring {
        candidates.push((StealTarget::Ring, 30));
    }
    if has_amulet {
        candidates.push((StealTarget::Amulet, 25));
    }
    if has_armor {
        candidates.push((StealTarget::Armor, 20));
    }
    if has_weapon {
        candidates.push((StealTarget::Weapon, 15));
    }
    candidates.push((StealTarget::RandomItem, 10));

    let total: i32 = candidates.iter().map(|(_, w)| w).sum();
    let mut roll = rng.rn2(total);
    for (target, weight) in &candidates {
        roll -= weight;
        if roll < 0 {
            return *target;
        }
    }
    StealTarget::RandomItem
}

///
pub fn monkey_target_selection(
    has_food: bool,
    has_tool: bool,
    has_weapon: bool,
    rng: &mut NetHackRng,
) -> StealTarget {
    let mut candidates = Vec::new();
    if has_food {
        candidates.push((StealTarget::Food, 40));
    }
    if has_tool {
        candidates.push((StealTarget::Tool, 30));
    }
    if has_weapon {
        candidates.push((StealTarget::Weapon, 20));
    }
    candidates.push((StealTarget::RandomItem, 10));

    let total: i32 = candidates.iter().map(|(_, w)| w).sum();
    let mut roll = rng.rn2(total);
    for (target, weight) in &candidates {
        roll -= weight;
        if roll < 0 {
            return *target;
        }
    }
    StealTarget::RandomItem
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn steal_resist_chance(
    player_dex: i32,
    player_level: i32,
    thief_level: i32,
    wearing_gloves: bool,
    has_holding: bool,
) -> i32 {
    //
    let mut resist = player_dex / 3 + player_level / 2;

    //
    if wearing_gloves {
        resist += 5;
    }

    //
    if has_holding {
        resist += 10;
    }

    //
    resist -= thief_level;

    resist.clamp(5, 95)
}

///
pub fn check_steal_resist(resist_chance: i32, rng: &mut NetHackRng) -> bool {
    rng.rn2(100) < resist_chance
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharmEffect {
    FullCharm,
    PartialCharm,
    Resisted,
    Immune,
}

///
pub fn nymph_charm_check(
    player_wis: i32,
    player_level: i32,
    is_blind: bool,
    free_action: bool,
    magic_res: bool,
    rng: &mut NetHackRng,
) -> CharmEffect {
    //
    if free_action {
        return CharmEffect::Immune;
    }

    //
    if magic_res && rng.rn2(2) == 0 {
        return CharmEffect::Immune;
    }

    //
    if is_blind {
        return CharmEffect::Resisted;
    }

    //
    let resist = player_wis / 2 + player_level / 3;
    let roll = rng.rn2(20) + 1;

    if resist > roll + 5 {
        CharmEffect::Resisted
    } else if resist > roll {
        CharmEffect::PartialCharm
    } else {
        CharmEffect::FullCharm
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn theft_detection_chance(
    stealth: bool,
    invisible: bool,
    shop_keeper_wis: i32,
    distance: i32,
) -> i32 {
    let mut detect = 50 + shop_keeper_wis * 2;

    if stealth {
        detect -= 20;
    }
    if invisible {
        detect -= 30;
    }

    //
    detect -= distance * 3;

    detect.clamp(5, 99)
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct TheftStatistics {
    pub gold_stolen_total: u64,
    pub items_stolen: u32,
    pub steal_attempts: u32,
    pub steal_resisted: u32,
    pub caught_count: u32,
}

impl TheftStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_gold_theft(&mut self, amount: u64) {
        self.gold_stolen_total += amount;
        self.steal_attempts += 1;
    }

    pub fn record_item_theft(&mut self) {
        self.items_stolen += 1;
        self.steal_attempts += 1;
    }

    pub fn record_resist(&mut self) {
        self.steal_resisted += 1;
        self.steal_attempts += 1;
    }

    pub fn record_caught(&mut self) {
        self.caught_count += 1;
    }
}

///
pub fn select_by_value(
    item_values: &[(usize, i32)],
    rng: &mut NetHackRng,
) -> Option<usize> {
    if item_values.is_empty() {
        return None;
    }

    //
    let total: i32 = item_values.iter().map(|(_, v)| v.max(&1)).sum();
    if total <= 0 {
        return Some(item_values[0].0);
    }

    let mut roll = rng.rn2(total);
    for (idx, value) in item_values {
        roll -= value.max(&1);
        if roll < 0 {
            return Some(*idx);
        }
    }
    Some(item_values[0].0)
}

///
pub fn cursed_blocks_theft(is_cursed: bool, is_equipped: bool) -> bool {
    is_cursed && is_equipped
}

///
pub fn theft_message(
    thief_name: &str,
    target: StealTarget,
    success: bool,
    cursed_block: bool,
) -> String {
    if cursed_block {
        return format!(
            "The {} tries to steal your equipment, but it's stuck!",
            thief_name
        );
    }

    if !success {
        return format!(
            "The {} reaches for your belongings, but you fend it off!",
            thief_name
        );
    }

    match target {
        StealTarget::Weapon => format!("The {} steals your weapon!", thief_name),
        StealTarget::Armor => format!("The {} steals your armor!", thief_name),
        StealTarget::Ring => format!("The {} steals your ring!", thief_name),
        StealTarget::Amulet => format!("The {} steals your amulet!", thief_name),
        StealTarget::Tool => format!("The {} steals one of your tools!", thief_name),
        StealTarget::Food => format!("The {} steals some of your food!", thief_name),
        StealTarget::RandomItem => format!("The {} steals something from you!", thief_name),
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod steal_extended_tests {
    use super::*;

    #[test]
    fn test_steal_gold_amount() {
        let mut rng = NetHackRng::new(42);
        let stolen = steal_gold_amount(1000, 5, &mut rng);
        assert!(stolen > 0 && stolen <= 1000);
    }

    #[test]
    fn test_steal_gold_zero() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(steal_gold_amount(0, 5, &mut rng), 0);
    }

    #[test]
    fn test_leprechaun_steal() {
        let mut rng = NetHackRng::new(42);
        let stolen = leprechaun_steal(500, 5, 8, &mut rng);
        assert!(stolen > 0 && stolen <= 500);
    }

    #[test]
    fn test_nymph_target() {
        let mut rng = NetHackRng::new(42);
        let target = nymph_target_selection(true, true, true, true, &mut rng);
        //
        assert!(matches!(
            target,
            StealTarget::Ring
                | StealTarget::Amulet
                | StealTarget::Armor
                | StealTarget::Weapon
                | StealTarget::RandomItem
        ));
    }

    #[test]
    fn test_steal_resist() {
        let chance = steal_resist_chance(18, 10, 5, true, false);
        assert!(chance > 0 && chance <= 95);
    }

    #[test]
    fn test_nymph_charm_free_action() {
        let mut rng = NetHackRng::new(42);
        let result = nymph_charm_check(10, 5, false, true, false, &mut rng);
        assert_eq!(result, CharmEffect::Immune);
    }

    #[test]
    fn test_nymph_charm_blind() {
        let mut rng = NetHackRng::new(42);
        let result = nymph_charm_check(10, 5, true, false, false, &mut rng);
        assert_eq!(result, CharmEffect::Resisted);
    }

    #[test]
    fn test_cursed_blocks() {
        assert!(cursed_blocks_theft(true, true));
        assert!(!cursed_blocks_theft(false, true));
        assert!(!cursed_blocks_theft(true, false));
    }

    #[test]
    fn test_theft_statistics() {
        let mut stats = TheftStatistics::new();
        stats.record_gold_theft(100);
        stats.record_item_theft();
        stats.record_resist();
        assert_eq!(stats.steal_attempts, 3);
        assert_eq!(stats.gold_stolen_total, 100);
        assert_eq!(stats.items_stolen, 1);
    }

    #[test]
    fn test_select_by_value() {
        let mut rng = NetHackRng::new(42);
        let items = vec![(0, 100), (1, 50), (2, 10)];
        let selected = select_by_value(&items, &mut rng);
        assert!(selected.is_some());
    }
}

// =============================================================================
// [v2.3.4
// =============================================================================

///
pub fn player_steal_chance(
    player_dex: i32,
    player_luck: i32,
    monster_level: i32,
    monster_awake: bool,
    rng: &mut NetHackRng,
) -> bool {
    let base_chance = player_dex * 3 + player_luck * 2;
    let difficulty = monster_level * 5 + if monster_awake { 20 } else { 0 };
    let roll = rng.rn2(base_chance + difficulty);
    roll < base_chance
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThiefAttraction {
    Gold,
    MagicItems,
    Food,
    Gems,
    Weapons,
    Amulet,
    Nothing,
}

///
pub fn thief_attraction(monster_symbol: char) -> ThiefAttraction {
    match monster_symbol {
        'l' => ThiefAttraction::Gold,
        'n' => ThiefAttraction::MagicItems,
        'Y' => ThiefAttraction::Food,
        'N' => ThiefAttraction::Gems,
        'O' => ThiefAttraction::Weapons,
        'h' => ThiefAttraction::Food,
        _ => ThiefAttraction::Nothing,
    }
}

///
pub fn is_item_concealable(item_class: &str) -> bool {
    matches!(
        item_class,
        "ring" | "amulet" | "gem" | "coin" | "scroll" | "potion"
    )
}

///
pub fn concealment_bonus(item_class: &str, carrying_bag: bool) -> i32 {
    let base = if is_item_concealable(item_class) {
        5
    } else {
        0
    };
    let bag_bonus = if carrying_bag { 10 } else { 0 };
    base + bag_bonus
}

///
pub fn scatter_gold(total_gold: i32, rng: &mut NetHackRng) -> Vec<(i32, i32, i32)> {
    //
    let mut scattered = Vec::new();
    let mut remaining = total_gold;

    for dx in -1..=1 {
        for dy in -1..=1 {
            if remaining <= 0 {
                break;
            }
            if dx == 0 && dy == 0 {
                continue;
            }
            let amount = rng.rn2(remaining / 3 + 1).max(1);
            scattered.push((dx, dy, amount));
            remaining -= amount;
        }
    }

    //
    if remaining > 0 {
        scattered.push((0, 0, remaining));
    }

    scattered
}

///
pub fn anti_theft_cooldown(thief_level: i32) -> u32 {
    //
    let base = 30u32;
    base.saturating_sub(thief_level as u32 * 2).max(5)
}

///
pub fn theft_noise_chance(steal_type: StealType, is_invisible: bool, rng: &mut NetHackRng) -> bool {
    let base_noise = match steal_type {
        StealType::Gold => 10,
        StealType::Item => 20,
        StealType::Armor => 40,
        StealType::Seduce => 5,
        StealType::All => 60,
    };
    let modifier = if is_invisible {
        base_noise / 2
    } else {
        base_noise
    };
    rng.rn2(100) < modifier
}

///
#[derive(Debug, Clone, Default)]
pub struct CrimeRecord {
    pub times_robbed: u32,
    pub gold_lost: i32,
    pub items_lost: u32,
    pub successful_counter_thefts: u32,
    pub shopkeepers_angered: u32,
}

impl CrimeRecord {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_robbery(&mut self, gold: i32, items: u32) {
        self.times_robbed += 1;
        self.gold_lost += gold;
        self.items_lost += items;
    }
    pub fn record_counter_theft(&mut self) {
        self.successful_counter_thefts += 1;
    }
}

///
pub fn stolen_item_recovery_chance(thief_dead: bool, distance: i32, turns_since_theft: u32) -> i32 {
    if thief_dead {
        return 100;
    }
    let base = 50;
    let dist_penalty = distance * 3;
    let time_penalty = turns_since_theft as i32;
    (base - dist_penalty - time_penalty).max(0).min(100)
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheftAlertType {
    Silent,
    Noticed,
    CaughtRedHanded,
    ShopkeeperAlerted,
}

///
pub fn theft_alert_message(alert: &TheftAlertType, thief_name: &str) -> String {
    match alert {
        TheftAlertType::Silent => String::new(),
        TheftAlertType::Noticed => {
            format!("You notice {} trying to steal from you!", thief_name)
        }
        TheftAlertType::CaughtRedHanded => {
            format!("You catch {} red-handed!", thief_name)
        }
        TheftAlertType::ShopkeeperAlerted => {
            format!("\"Stop, {}!\" cries the shopkeeper.", thief_name)
        }
    }
}

#[cfg(test)]
mod steal_advanced_tests {
    use super::*;

    #[test]
    fn test_player_steal() {
        let mut rng = NetHackRng::new(42);
        //
        let mut success = 0;
        for _ in 0..50 {
            if player_steal_chance(18, 10, 1, false, &mut rng) {
                success += 1;
            }
        }
        assert!(success > 10);
    }

    #[test]
    fn test_attraction() {
        assert_eq!(thief_attraction('l'), ThiefAttraction::Gold);
        assert_eq!(thief_attraction('n'), ThiefAttraction::MagicItems);
    }

    #[test]
    fn test_concealment() {
        assert!(is_item_concealable("ring"));
        assert!(!is_item_concealable("weapon"));
    }

    #[test]
    fn test_scatter_gold() {
        let mut rng = NetHackRng::new(42);
        let scattered = scatter_gold(100, &mut rng);
        let total: i32 = scattered.iter().map(|(_, _, a)| *a).sum();
        assert_eq!(total, 100);
    }

    #[test]
    fn test_anti_theft_cooldown() {
        assert!(anti_theft_cooldown(1) > anti_theft_cooldown(10));
    }

    #[test]
    fn test_recovery() {
        assert_eq!(stolen_item_recovery_chance(true, 0, 0), 100);
        assert!(stolen_item_recovery_chance(false, 0, 0) < 100);
    }

    #[test]
    fn test_crime_record() {
        let mut c = CrimeRecord::new();
        c.record_robbery(50, 2);
        assert_eq!(c.times_robbed, 1);
        assert_eq!(c.gold_lost, 50);
    }

    #[test]
    fn test_alert_message() {
        let m = theft_alert_message(&TheftAlertType::CaughtRedHanded, "nymph");
        assert!(m.contains("nymph"));
    }
}
