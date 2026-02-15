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
//

use crate::core::entity::player::{Alignment, Player, PlayerClass, Race};
use crate::core::systems::role::rank_of;
use crate::ui::log::GameLog;
use serde::{Deserialize, Serialize};

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathType {
    Killed,
    Poisoned,      // POISONING
    Starved,       // STARVING
    Drowned,       // DROWNING
    Burned,
    Crushed,
    StonedToDeath,
    Slimed,        // SLIMING
    Strangled,     // STRANGULATION
    Suffocated,
    FoodPoisoning, // FOOD_POISONING
    Illness,       // ILLNESS
    Genocide,      // GENOCIDED
    Disintegrated, // DISINTEGRATED
    Escaped,
    Ascended,
    Quit,
    Tricked,
    Panicked,
    Turned,        // TURNED_TO_STONE
    FellInLava,
    FellInWater,
    CaughtInTrap,
    ZappedSelf,
    SuicideByWand,
    Petrified,
}

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeathInfo {
    pub death_type: DeathType,
    pub cause: String,
    pub monster_name: Option<String>,
    pub turn: u64,
    pub depth: i32,
}

impl DeathInfo {
    pub fn new(death_type: DeathType, cause: &str, turn: u64, depth: i32) -> Self {
        Self {
            death_type,
            cause: cause.to_string(),
            monster_name: None,
            turn,
            depth,
        }
    }

    ///
    pub fn killed_by(monster: &str, turn: u64, depth: i32) -> Self {
        Self {
            death_type: DeathType::Killed,
            cause: format!("killed by {}", an(monster)),
            monster_name: Some(monster.to_string()),
            turn,
            depth,
        }
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameScore {
    pub score: i64,
    pub player_name: String,
    pub role: PlayerClass,
    pub race: Race,
    pub alignment: Alignment,
    pub level: i32,
    pub max_hp: i32,
    pub max_depth: i32,
    pub gold: u64,
    pub death_info: DeathInfo,
    pub turns: u64,
}

///
pub fn calculate_score(
    player: &Player,
    death_info: &DeathInfo,
    max_depth: i32,
    carried_gold: u64,
    turns: u64,
) -> i64 {
    let mut score: i64 = 0;

    //
    score += carried_gold as i64;

    //
    //
    //

    //
    score += player.experience as i64;

    //
    score += (max_depth as i64) * 1000;

    //
    if death_info.death_type == DeathType::Ascended {
        score *= 2;
        //
        score += (player.exp_level as i64) * 5000;
    }

    //
    if death_info.death_type == DeathType::Ascended && turns < 30000 {
        score += ((30000 - turns) as i64) * 10;
    }

    score.max(0)
}

// =============================================================================
//
// =============================================================================

///
pub fn generate_tombstone(
    player_name: &str,
    role: PlayerClass,
    level: i32,
    gold: u64,
    death_info: &DeathInfo,
) -> Vec<String> {
    let rank = rank_of(role, level);
    let mut lines = Vec::new();

    lines.push("                       ----------".to_string());
    lines.push("                      /          \\".to_string());
    lines.push("                     /    REST    \\".to_string());
    lines.push("                    /      IN      \\".to_string());
    lines.push("                   /     PEACE      \\".to_string());
    lines.push("                  /                  \\".to_string());
    lines.push(format!(
        "                 |  {}  |",
        center_text(player_name, 16)
    ));
    lines.push(format!("                 |  {}  |", center_text(rank, 16)));
    lines.push("                 |                  |".to_string());

    //
    let cause_lines = word_wrap(&death_info.cause, 16);
    for cline in cause_lines.iter().take(3) {
        lines.push(format!("                 |  {}  |", center_text(cline, 16)));
    }

    lines.push("                 |                  |".to_string());
    lines.push(format!(
        "                 |  {}  |",
        center_text(&format!("{} Gold", gold), 16)
    ));
    lines.push(format!(
        "                 |  {}  |",
        center_text(&format!("Depth: {}", death_info.depth), 16)
    ));
    lines.push(format!(
        "                 |  {}  |",
        center_text(&format!("Turn: {}", death_info.turn), 16)
    ));
    lines.push("                 |                  |".to_string());
    lines.push("                *|     *  *  *      |*".to_string());
    lines.push("        _______)/\\\\__//(\\/(/\\)/\\//\\/|_)_______".to_string());

    lines
}

///
fn center_text(text: &str, width: usize) -> String {
    if text.len() >= width {
        text[..width].to_string()
    } else {
        let padding = (width - text.len()) / 2;
        let mut result = " ".repeat(padding);
        result.push_str(text);
        while result.len() < width {
            result.push(' ');
        }
        result
    }
}

///
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > max_width {
            if !current.is_empty() {
                lines.push(current.clone());
                current.clear();
            }
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

// =============================================================================
//
// =============================================================================

///
fn an(name: &str) -> String {
    let first = name.chars().next().unwrap_or(' ').to_ascii_lowercase();
    if "aeiou".contains(first) {
        format!("an {}", name)
    } else {
        format!("a {}", name)
    }
}

///
pub fn death_message(death_info: &DeathInfo) -> String {
    match death_info.death_type {
        DeathType::Killed => format!("You were {}.", death_info.cause),
        DeathType::Poisoned => format!(
            "You died from poisoning on dungeon level {}.",
            death_info.depth
        ),
        DeathType::Starved => "You died of starvation.".to_string(),
        DeathType::Drowned => format!("You drowned on dungeon level {}.", death_info.depth),
        DeathType::Burned => "You were burned to a crisp.".to_string(),
        DeathType::StonedToDeath => "You turned to stone.".to_string(),
        DeathType::Slimed => "You turned to slime.".to_string(),
        DeathType::Strangled => "You choked on your food.".to_string(),
        DeathType::Suffocated => "You choked to death.".to_string(),
        DeathType::FoodPoisoning => "You died of food poisoning.".to_string(),
        DeathType::Illness => "You died of illness.".to_string(),
        DeathType::Genocide => "You were genocided.".to_string(),
        DeathType::Disintegrated => "You were disintegrated.".to_string(),
        DeathType::Escaped => "You escaped the dungeon!".to_string(),
        DeathType::Ascended => "You ascended to demigodhood!".to_string(),
        DeathType::Quit => "You quit.".to_string(),
        DeathType::FellInLava => "You fell into lava.".to_string(),
        DeathType::FellInWater => "You drowned.".to_string(),
        DeathType::CaughtInTrap => format!("You died from a trap on level {}.", death_info.depth),
        DeathType::ZappedSelf => "You zapped yourself with a wand of death.".to_string(),
        DeathType::SuicideByWand => "You committed suicide.".to_string(),
        DeathType::Petrified => "You turned to stone.".to_string(),
        _ => format!("You died: {}.", death_info.cause),
    }
}

///
pub fn game_over(
    player: &Player,
    player_name: &str,
    death_info: &DeathInfo,
    max_depth: i32,
    log: &mut GameLog,
) {
    //
    let msg = death_message(death_info);
    log.add_colored(&msg, [255, 100, 100], death_info.turn);

    //
    let score = calculate_score(player, death_info, max_depth, player.gold, death_info.turn);
    log.add_colored(
        &format!("Your score: {}", score),
        [255, 255, 0],
        death_info.turn,
    );

    //
    if death_info.death_type != DeathType::Ascended && death_info.death_type != DeathType::Escaped {
        let tombstone = generate_tombstone(
            player_name,
            player.role,
            player.exp_level,
            player.gold,
            death_info,
        );
        for line in &tombstone {
            log.add(line, death_info.turn);
        }
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopTenEntry {
    pub score: i64,
    pub name: String,
    pub death_cause: String,
    pub role: PlayerClass,
    pub race: Race,
    pub alignment: Alignment,
    pub level: i32,
    pub max_depth: i32,
    pub turns: u64,
}

///
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HighScoreBoard {
    pub entries: Vec<TopTenEntry>,
}

impl HighScoreBoard {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    ///
    pub fn add_entry(&mut self, entry: TopTenEntry) -> usize {
        let score = entry.score;
        self.entries.push(entry);
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        self.entries.truncate(100);

        //
        self.entries
            .iter()
            .position(|e| e.score == score)
            .unwrap_or(self.entries.len())
    }

    ///
    pub fn top_n(&self, n: usize) -> &[TopTenEntry] {
        let end = n.min(self.entries.len());
        &self.entries[..end]
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Conduct {
    pub wishes: u32,
    pub kills: u32,
    pub atheist: bool,
    pub pacifist: bool,
    pub illiterate: bool,
    pub vegetarian: bool,       // 梨꾩떇
    pub vegan: bool,
    pub foodless: bool,
    pub weaponless: bool,
    pub genocideless: bool,
    pub polyselfless: bool,
    pub polymorphless: bool,
    pub artifacts_touched: u32,
    pub elbereth_written: u32,
}

impl Conduct {
    pub fn new() -> Self {
        Self {
            atheist: true,
            pacifist: true,
            illiterate: true,
            vegetarian: true,
            vegan: true,
            foodless: true,
            weaponless: true,
            genocideless: true,
            polyselfless: true,
            polymorphless: true,
            ..Default::default()
        }
    }

    ///
    pub fn record_kill(&mut self) {
        self.kills += 1;
        self.pacifist = false;
    }

    ///
    pub fn record_eat(&mut self, is_meat: bool) {
        self.foodless = false;
        if is_meat {
            self.vegetarian = false;
            self.vegan = false;
        }
    }

    ///
    pub fn record_read(&mut self) {
        self.illiterate = false;
    }

    ///
    pub fn record_pray(&mut self) {
        self.atheist = false;
    }

    ///
    pub fn active_conducts(&self) -> Vec<&'static str> {
        let mut conducts = Vec::new();
        if self.atheist {
            conducts.push("atheist");
        }
        if self.pacifist {
            conducts.push("pacifist");
        }
        if self.illiterate {
            conducts.push("illiterate");
        }
        if self.vegetarian {
            conducts.push("vegetarian");
        }
        if self.vegan {
            conducts.push("vegan");
        }
        if self.foodless {
            conducts.push("foodless");
        }
        if self.weaponless {
            conducts.push("weaponless");
        }
        if self.genocideless {
            conducts.push("genocideless");
        }
        if self.polyselfless {
            conducts.push("never polymorphed");
        }
        if self.wishes == 0 {
            conducts.push("wishless");
        }
        conducts
    }

    ///
    pub fn summary(&self) -> String {
        let conducts = self.active_conducts();
        if conducts.is_empty() {
            "No special conducts maintained.".to_string()
        } else {
            format!("Conducts maintained: {}", conducts.join(", "))
        }
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_death_info() {
        let info = DeathInfo::killed_by("orc", 100, 5);
        assert_eq!(info.cause, "killed by an orc");
        assert_eq!(info.depth, 5);
    }

    #[test]
    fn test_death_message() {
        let info = DeathInfo::new(DeathType::Starved, "starvation", 50, 3);
        let msg = death_message(&info);
        assert!(msg.contains("starvation"));
    }

    #[test]
    fn test_tombstone() {
        let info = DeathInfo::killed_by("dragon", 200, 10);
        let lines = generate_tombstone("TestHero", PlayerClass::Valkyrie, 10, 500, &info);
        assert!(lines.len() > 10);
    }

    #[test]
    fn test_high_score() {
        let mut board = HighScoreBoard::new();
        let entry = TopTenEntry {
            score: 1000,
            name: "Hero".to_string(),
            death_cause: "killed by an orc".to_string(),
            role: PlayerClass::Valkyrie,
            race: Race::Human,
            alignment: Alignment::Neutral,
            level: 5,
            max_depth: 10,
            turns: 500,
        };
        let rank = board.add_entry(entry);
        assert_eq!(rank, 0);
        assert_eq!(board.entries.len(), 1);
    }

    #[test]
    fn test_conduct() {
        let mut conduct = Conduct::new();
        assert!(conduct.pacifist);
        conduct.record_kill();
        assert!(!conduct.pacifist);
        assert!(conduct.atheist);
        conduct.record_pray();
        assert!(!conduct.atheist);
    }

    #[test]
    fn test_word_wrap() {
        let text = "killed by a very large and dangerous red dragon";
        let lines = word_wrap(text, 16);
        assert!(lines.len() >= 3);
        for line in &lines {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_an() {
        assert_eq!(an("orc"), "an orc");
        assert_eq!(an("dragon"), "a dragon");
        assert_eq!(an("elf"), "an elf");
    }
}
