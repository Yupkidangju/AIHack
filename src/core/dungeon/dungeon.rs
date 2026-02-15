// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// =============================================================================
//
//
// [v2.3.1
//
//
//
//
// =============================================================================

use crate::core::dungeon::{DungeonBranch, Grid, LevelID};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    ///
    pub branch: DungeonBranch,
    ///
    pub depth_start: i32,
    ///
    pub depth_end: i32,
    ///
    pub name: String,
    ///
    pub entry_level: Option<LevelID>,
    ///
    pub has_special_levels: bool,
    ///
    pub is_hellish: bool,
    ///
    pub is_maze: bool,
    ///
    pub is_tower: bool,
}

impl BranchInfo {
    ///
    pub fn new(branch: DungeonBranch, depth_start: i32, depth_end: i32, name: &str) -> Self {
        Self {
            branch,
            depth_start,
            depth_end,
            name: name.to_string(),
            entry_level: None,
            has_special_levels: false,
            is_hellish: false,
            is_maze: false,
            is_tower: false,
        }
    }

    ///
    pub fn num_levels(&self) -> i32 {
        (self.depth_end - self.depth_start + 1).max(1)
    }

    ///
    pub fn absolute_depth(&self, relative_level: i32) -> i32 {
        self.depth_start + relative_level - 1
    }

    ///
    pub fn relative_level(&self, absolute_depth: i32) -> i32 {
        absolute_depth - self.depth_start + 1
    }

    ///
    pub fn contains_depth(&self, absolute_depth: i32) -> bool {
        absolute_depth >= self.depth_start && absolute_depth <= self.depth_end
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Clone, Serialize, Deserialize)]
pub struct Dungeon {
    pub levels: HashMap<LevelID, Grid>,
    pub current_level: LevelID,
    ///
    pub branches: Vec<BranchInfo>,
    ///
    pub difficulty_offset: i32,
    ///
    pub deepest_reached: i32,
    ///
    pub amulet_obtained: bool,
}

impl Dungeon {
    pub fn new() -> Self {
        let mut d = Self {
            levels: HashMap::new(),
            current_level: LevelID::new(DungeonBranch::Main, 1),
            branches: Vec::new(),
            difficulty_offset: 0,
            deepest_reached: 1,
            amulet_obtained: false,
        };
        d.init_branches();
        d
    }

    ///
    fn init_branches(&mut self) {
        //
        let mut main_branch = BranchInfo::new(
            DungeonBranch::Main,
            1,
            30,
            "The Dungeons of Doom",
        );
        main_branch.has_special_levels = true;
        self.branches.push(main_branch);

        //
        let mut mines = BranchInfo::new(
            DungeonBranch::Mines,
            3,
            15,
            "The Gnomish Mines",
        );
        mines.entry_level = Some(LevelID::new(DungeonBranch::Main, 3));
        self.branches.push(mines);

        //
        let mut sokoban = BranchInfo::new(
            DungeonBranch::Sokoban,
            6,
            10,
            "Sokoban",
        );
        sokoban.entry_level = Some(LevelID::new(DungeonBranch::Main, 6));
        sokoban.is_maze = true;
        self.branches.push(sokoban);

        //
        let mut gehennom = BranchInfo::new(
            DungeonBranch::Gehennom,
            25,
            50,
            "Gehennom",
        );
        gehennom.entry_level = Some(LevelID::new(DungeonBranch::Main, 25));
        gehennom.is_hellish = true;
        gehennom.has_special_levels = true;
        self.branches.push(gehennom);

        //
        let mut quest = BranchInfo::new(
            DungeonBranch::Quest,
            16,
            22,
            "The Quest",
        );
        quest.entry_level = Some(LevelID::new(DungeonBranch::Main, 14));
        quest.has_special_levels = true;
        self.branches.push(quest);

        //
        let mut vlad = BranchInfo::new(
            DungeonBranch::VladTower,
            40,
            43,
            "Vlad's Tower",
        );
        vlad.entry_level = Some(LevelID::new(DungeonBranch::Gehennom, 40));
        vlad.is_tower = true;
        self.branches.push(vlad);

        //
        let astral = BranchInfo::new(DungeonBranch::Astral, 51, 51, "The Astral Plane");
        self.branches.push(astral);
    }

    ///
    pub fn get_level(&self, id: LevelID) -> Option<&Grid> {
        self.levels.get(&id)
    }

    ///
    pub fn get_level_mut(&mut self, id: LevelID) -> Option<&mut Grid> {
        self.levels.get_mut(&id)
    }

    pub fn set_level(&mut self, id: LevelID, grid: Grid) {
        self.levels.insert(id, grid);
    }

    ///
    pub fn get_branch(&self, branch: DungeonBranch) -> Option<&BranchInfo> {
        self.branches.iter().find(|b| b.branch == branch)
    }

    ///
    pub fn current_depth(&self) -> i32 {
        self.level_depth(self.current_level)
    }

    ///
    pub fn level_depth(&self, level_id: LevelID) -> i32 {
        if let Some(branch) = self.get_branch(level_id.branch) {
            branch.absolute_depth(level_id.depth as i32)
        } else {
            level_id.depth as i32
        }
    }

    ///
    ///
    pub fn level_difficulty(&self, player_level: i32) -> i32 {
        let depth = self.current_depth();
        let lev_bonus = player_level / 5;
        let amulet_bonus = if self.amulet_obtained { 5 } else { 0 };
        (depth + lev_bonus + amulet_bonus + self.difficulty_offset).max(1)
    }

    ///
    pub fn in_hell(&self) -> bool {
        if let Some(branch) = self.get_branch(self.current_level.branch) {
            branch.is_hellish
        } else {
            false
        }
    }

    ///
    pub fn in_quest(&self) -> bool {
        self.current_level.branch == DungeonBranch::Quest
    }

    ///
    pub fn in_mines(&self) -> bool {
        self.current_level.branch == DungeonBranch::Mines
    }

    ///
    pub fn in_sokoban(&self) -> bool {
        self.current_level.branch == DungeonBranch::Sokoban
    }

    ///
    pub fn on_tower(&self) -> bool {
        if let Some(branch) = self.get_branch(self.current_level.branch) {
            branch.is_tower
        } else {
            false
        }
    }

    ///
    pub fn update_deepest(&mut self) {
        let depth = self.current_depth();
        if depth > self.deepest_reached {
            self.deepest_reached = depth;
        }
    }

    ///
    pub fn special_level_name(&self, level_id: LevelID) -> Option<&'static str> {
        match level_id.branch {
            DungeonBranch::Main => match level_id.depth {
                1 => Some("Welcome Level"),
                5 => Some("The Oracle"),
                10 => Some("Big Room"),
                14 => Some("Quest Portal"),
                20 => Some("Castle"),
                25 => Some("Valley of the Dead"),
                _ => None,
            },
            DungeonBranch::Mines => match level_id.depth {
                8 => Some("Mine's End"),
                6 => Some("Minetown"),
                _ => None,
            },
            DungeonBranch::Gehennom => match level_id.depth {
                35 => Some("Juiblex's Swamp"),
                40 => Some("Asmodeus' Lair"),
                45 => Some("Wizard's Tower"),
                50 => Some("Sanctum"),
                _ => None,
            },
            DungeonBranch::Astral => Some("The Astral Plane"),
            _ => None,
        }
    }

    ///
    pub fn on_astral_plane(&self) -> bool {
        self.current_level.branch == DungeonBranch::Astral
    }

    ///
    pub fn branch_name(&self, branch: DungeonBranch) -> &str {
        if let Some(info) = self.get_branch(branch) {
            &info.name
        } else {
            "Unknown"
        }
    }

    ///
    pub fn next_level_down(&self) -> Option<LevelID> {
        let branch_info = self.get_branch(self.current_level.branch)?;
        let next_depth = self.current_level.depth as i32 + 1;
        if next_depth <= branch_info.num_levels() {
            Some(LevelID::new(self.current_level.branch, next_depth))
        } else {
            None
        }
    }

    ///
    pub fn next_level_up(&self) -> Option<LevelID> {
        let depth = self.current_level.depth as i32 - 1;
        if depth >= 1 {
            Some(LevelID::new(self.current_level.branch, depth))
        } else {
            //
            let branch_info = self.get_branch(self.current_level.branch)?;
            branch_info.entry_level
        }
    }

    ///
    pub fn at_branch_bottom(&self) -> bool {
        if let Some(info) = self.get_branch(self.current_level.branch) {
            self.current_level.depth as i32 >= info.num_levels()
        } else {
            false
        }
    }

    ///
    pub fn at_branch_top(&self) -> bool {
        self.current_level.depth <= 1
    }

    ///
    pub fn all_level_ids(&self) -> Vec<LevelID> {
        self.levels.keys().cloned().collect()
    }

    ///
    pub fn num_explored_levels(&self) -> usize {
        self.levels.len()
    }

    ///
    pub fn level_exists(&self, id: LevelID) -> bool {
        self.levels.contains_key(&id)
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn describe_level(dungeon: &Dungeon, level_id: LevelID) -> String {
    let branch_name = dungeon.branch_name(level_id.branch);
    let depth = dungeon.level_depth(level_id);

    if let Some(special_name) = dungeon.special_level_name(level_id) {
        format!("{}: {} (Depth {})", branch_name, special_name, depth)
    } else {
        format!(
            "{}: Level {} (Depth {})",
            branch_name, level_id.depth, depth
        )
    }
}

///
pub fn short_level_name(dungeon: &Dungeon) -> String {
    let level_id = dungeon.current_level;
    match level_id.branch {
        DungeonBranch::Main => format!("Dlvl:{}", dungeon.current_depth()),
        DungeonBranch::Mines => format!("Mine:{}", level_id.depth),
        DungeonBranch::Sokoban => format!("Sok:{}", level_id.depth),
        DungeonBranch::Gehennom => format!("Geh:{}", dungeon.current_depth()),
        DungeonBranch::Quest => format!("Qst:{}", level_id.depth),
        DungeonBranch::VladTower => format!("Vlad:{}", level_id.depth),
        DungeonBranch::Astral => "Astral".to_string(),
        _ => format!("Lvl:{}", dungeon.current_depth()),
    }
}

///
pub fn max_monster_level(difficulty: i32) -> i32 {
    //
    (difficulty + 5).min(49)
}

///
pub fn item_difficulty(depth: i32) -> i32 {
    //
    (depth + 2).max(1)
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dungeon_new() {
        let d = Dungeon::new();
        assert!(!d.branches.is_empty());
        assert_eq!(d.current_level.branch, DungeonBranch::Main);
        assert_eq!(d.current_level.depth, 1);
    }

    #[test]
    fn test_branch_info() {
        let d = Dungeon::new();
        let main = d.get_branch(DungeonBranch::Main).unwrap();
        assert_eq!(main.depth_start, 1);
        assert_eq!(main.num_levels(), 30);
    }

    #[test]
    fn test_depth_calculation() {
        let d = Dungeon::new();
        let depth = d.level_depth(LevelID::new(DungeonBranch::Main, 5));
        assert_eq!(depth, 5);
    }

    #[test]
    fn test_hell_check() {
        let mut d = Dungeon::new();
        d.current_level = LevelID::new(DungeonBranch::Gehennom, 30);
        assert!(d.in_hell());
    }

    #[test]
    fn test_special_level_name() {
        let d = Dungeon::new();
        let name = d.special_level_name(LevelID::new(DungeonBranch::Main, 20));
        assert_eq!(name, Some("Castle"));
    }
}
