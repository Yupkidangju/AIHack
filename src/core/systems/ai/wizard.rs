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

use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone)]
pub struct WizardState {
    ///
    pub death_count: i32,
    ///
    pub alive: bool,
    ///
    pub on_level: bool,
    ///
    pub last_harass_turn: u64,
    ///
    pub knows_amulet: bool,
    ///
    pub next_action_delay: i32,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            death_count: 0,
            alive: false,
            on_level: false,
            last_harass_turn: 0,
            knows_amulet: false,
            next_action_delay: 0,
        }
    }
}

impl WizardState {
    pub fn new() -> Self {
        Self::default()
    }

    ///
    pub fn on_death(&mut self) {
        self.death_count += 1;
        self.alive = false;
        self.on_level = false;
        //
    }

    ///
    pub fn should_resurrect(&self, current_turn: u64) -> bool {
        if self.alive {
            return false;
        }
        //
        //
        let delay = (150 - self.death_count * 20).max(30);
        current_turn - self.last_harass_turn > delay as u64
    }

    ///
    pub fn resurrect(&mut self, current_turn: u64, log: &mut GameLog) {
        self.alive = true;
        self.last_harass_turn = current_turn;
        self.next_action_delay = 10;

        let messages = [
            "A voice booms out: \"So thou thought thou couldst kill me, fool!\"",
            "\"I am the Wizard of Yendor! I shall never truly die!\"",
            "The Wizard of Yendor returns from the dead!",
        ];
        let msg_idx = (self.death_count as usize).min(messages.len() - 1);
        log.add_colored(messages[msg_idx], [255, 0, 255], current_turn);
    }
}

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardAttack {
    SummonMonsters,
    CurseItems,
    StealAmulet,
    CreateTraps,
    DoubleUp,
    Aggravate,
    DisableItem,
    MakeItDark,
    Destroy,
}

///
pub fn choose_wizard_action(
    state: &WizardState,
    player_has_amulet: bool,
    rng: &mut NetHackRng,
) -> WizardAttack {
    //
    if player_has_amulet && rng.rn2(3) == 0 {
        return WizardAttack::StealAmulet;
    }

    //
    let actions = [
        WizardAttack::SummonMonsters,
        WizardAttack::CurseItems,
        WizardAttack::CreateTraps,
        WizardAttack::DoubleUp,
        WizardAttack::Aggravate,
        WizardAttack::DisableItem,
        WizardAttack::MakeItDark,
        WizardAttack::Destroy,
    ];

    //
    let max_idx = if state.death_count >= 3 {
        actions.len()
    } else if state.death_count >= 1 {
        6
    } else {
        4
    };

    let idx = rng.rn2(max_idx as i32) as usize;
    actions[idx]
}

///
pub fn execute_wizard_action(
    action: WizardAttack,
    rng: &mut NetHackRng,
    log: &mut GameLog,
    turn: u64,
) -> WizardActionResult {
    match action {
        WizardAttack::SummonMonsters => {
            let count = rng.rn2(5) + 3;
            log.add_colored(
                &format!("The Wizard of Yendor summons {} nasties!", count),
                [255, 0, 255],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: count,
                items_cursed: 0,
                traps_created: 0,
                message: format!("{} monsters summoned", count),
            }
        }
        WizardAttack::CurseItems => {
            let cursed = rng.rn2(3) + 1;
            log.add_colored(
                "The Wizard of Yendor curses your possessions!",
                [255, 0, 100],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: 0,
                items_cursed: cursed,
                traps_created: 0,
                message: format!("{} items cursed", cursed),
            }
        }
        WizardAttack::StealAmulet => {
            log.add_colored(
                "The Wizard of Yendor reaches for the Amulet!",
                [255, 0, 0],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: 0,
                items_cursed: 0,
                traps_created: 0,
                message: "Amulet steal attempt".to_string(),
            }
        }
        WizardAttack::CreateTraps => {
            let traps = rng.rn2(4) + 2;
            log.add_colored(
                "The Wizard of Yendor creates traps around you!",
                [255, 100, 0],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: 0,
                items_cursed: 0,
                traps_created: traps,
                message: format!("{} traps created", traps),
            }
        }
        WizardAttack::DoubleUp => {
            log.add_colored(
                "The Wizard of Yendor creates a double!",
                [200, 0, 200],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: 1,
                items_cursed: 0,
                traps_created: 0,
                message: "Double created".to_string(),
            }
        }
        WizardAttack::Aggravate => {
            log.add_colored(
                "The Wizard of Yendor wakes all monsters on this level!",
                [255, 200, 0],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: 0,
                items_cursed: 0,
                traps_created: 0,
                message: "All monsters aggravated".to_string(),
            }
        }
        WizardAttack::DisableItem => {
            log.add_colored(
                "A magical vortex disables one of your items!",
                [100, 100, 255],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: 0,
                items_cursed: 1,
                traps_created: 0,
                message: "Item disabled".to_string(),
            }
        }
        WizardAttack::MakeItDark => {
            log.add_colored(
                "The Wizard of Yendor plunges the level into darkness!",
                [100, 100, 100],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: 0,
                items_cursed: 0,
                traps_created: 0,
                message: "Darkness!".to_string(),
            }
        }
        WizardAttack::Destroy => {
            log.add_colored(
                "The Wizard of Yendor hurls a magical bolt at you!",
                [255, 255, 0],
                turn,
            );
            WizardActionResult {
                action,
                monster_count: 0,
                items_cursed: 0,
                traps_created: 0,
                message: "Direct magical attack".to_string(),
            }
        }
    }
}

///
#[derive(Debug, Clone)]
pub struct WizardActionResult {
    pub action: WizardAttack,
    pub monster_count: i32,
    pub items_cursed: i32,
    pub traps_created: i32,
    pub message: String,
}

// =============================================================================
//
// =============================================================================

///
///
pub fn should_harass(
    state: &WizardState,
    current_turn: u64,
    player_has_amulet: bool,
    rng: &mut NetHackRng,
) -> bool {
    if !state.alive {
        return false;
    }
    if !player_has_amulet {
        return false;
    }

    //
    let interval = if state.on_level { 50 } else { 200 };
    if current_turn - state.last_harass_turn < interval as u64 {
        return false;
    }

    //
    rng.rn2(10) < 3
}

// =============================================================================
//
// =============================================================================

///
pub fn wizard_dialogue(death_count: i32, rng: &mut NetHackRng) -> &'static str {
    let taunts = if death_count == 0 {
        &[
            "\"I am the Wizard of Yendor! Surrender the Amulet!\"",
            "\"Thou shalt pay for thine impudence!\"",
            "\"My power is beyond thy comprehension!\"",
        ][..]
    } else {
        &[
            "\"Again thou hast dared to oppose me!\"",
            "\"Thou canst not truly defeat me, mortal!\"",
            "\"I shall have my revenge!\"",
            "\"Each death only makes me stronger!\"",
        ][..]
    };

    let idx = rng.rn2(taunts.len() as i32) as usize;
    taunts[idx]
}

///
pub fn wizard_spawn_message(death_count: i32) -> &'static str {
    if death_count == 0 {
        "The Wizard of Yendor appears before you!"
    } else {
        "The Wizard of Yendor has returned!"
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn wizard_ai_turn(
    state: &mut WizardState,
    current_turn: u64,
    player_has_amulet: bool,
    dist_to_player: i32,
    rng: &mut NetHackRng,
    log: &mut GameLog,
) -> Option<WizardActionResult> {
    if !state.alive {
        //
        if state.should_resurrect(current_turn) {
            state.resurrect(current_turn, log);
            return None;
        }
        return None;
    }

    //
    if state.next_action_delay > 0 {
        state.next_action_delay -= 1;
        return None;
    }

    //
    if dist_to_player > 100 {
        return None;
    }

    //
    if rng.rn2(5) == 0 {
        let action = choose_wizard_action(state, player_has_amulet, rng);
        let result = execute_wizard_action(action, rng, log, current_turn);
        state.last_harass_turn = current_turn;
        state.next_action_delay = rng.rn2(10) + 5;
        return Some(result);
    }

    None
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_death_and_resurrect() {
        let mut state = WizardState::new();
        state.alive = true;
        state.on_death();
        assert!(!state.alive);
        assert_eq!(state.death_count, 1);
    }

    #[test]
    fn test_wizard_dialogue() {
        let mut rng = NetHackRng::new(42);
        let d = wizard_dialogue(0, &mut rng);
        assert!(!d.is_empty());
    }

    #[test]
    fn test_wizard_spawn_message() {
        assert_eq!(
            wizard_spawn_message(0),
            "The Wizard of Yendor appears before you!"
        );
        assert_eq!(
            wizard_spawn_message(1),
            "The Wizard of Yendor has returned!"
        );
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn wizard_detects_amulet(
    wizard_level: i32,
    distance: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> bool {
    let detect_range = wizard_level * 3;
    distance <= detect_range && rng.rn2(3) < 2
}

///
pub fn wizard_respawn_interval(defeat_count: u32) -> u32 {
    let base = 500u32;
    let reduction = defeat_count.saturating_mul(50);
    base.saturating_sub(reduction).max(100)
}

///
pub fn wizard_double_trouble(defeat_count: u32) -> u32 {
    if defeat_count >= 3 {
        (defeat_count - 2).min(3)
    } else {
        0
    }
}

///
pub fn curse_intensity(wizard_level: i32, player_luck: i32) -> i32 {
    let base = wizard_level / 3;
    let reduction = player_luck.max(0);
    (base - reduction).max(1).min(10)
}

///
#[derive(Debug, Clone, Default)]
pub struct WizardStatistics {
    pub total_defeats: u32,
    pub amulet_steal_attempts: u32,
    pub curses_applied: u32,
    pub monsters_summoned: u32,
    pub traps_created: u32,
    pub doubles_spawned: u32,
}

impl WizardStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_attack(&mut self, attack: &WizardAttack) {
        match attack {
            WizardAttack::StealAmulet => self.amulet_steal_attempts += 1,
            WizardAttack::CurseItems => self.curses_applied += 1,
            WizardAttack::SummonMonsters => self.monsters_summoned += 1,
            WizardAttack::CreateTraps => self.traps_created += 1,
            WizardAttack::DoubleUp => self.doubles_spawned += 1,
            _ => {}
        }
    }
}

#[cfg(test)]
mod wizard_extended_tests {
    use super::*;

    #[test]
    fn test_wizard_respawn() {
        assert!(wizard_respawn_interval(0) > wizard_respawn_interval(5));
        assert!(wizard_respawn_interval(100) >= 100);
    }

    #[test]
    fn test_double_trouble() {
        assert_eq!(wizard_double_trouble(0), 0);
        assert_eq!(wizard_double_trouble(3), 1);
        assert_eq!(wizard_double_trouble(10), 3);
    }

    #[test]
    fn test_curse_intensity() {
        assert!(curse_intensity(30, 5) >= 1);
        assert!(curse_intensity(30, 5) <= 10);
    }

    #[test]
    fn test_wizard_stats() {
        let mut stats = WizardStatistics::new();
        stats.record_attack(&WizardAttack::StealAmulet);
        stats.record_attack(&WizardAttack::CurseItems);
        assert_eq!(stats.amulet_steal_attempts, 1);
        assert_eq!(stats.curses_applied, 1);
    }
}
