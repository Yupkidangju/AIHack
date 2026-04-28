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

// =============================================================================
// [v2.9.6] wizard.c 미구???�수 ?�식 ???�략/?�술/?�환/?�발/?�???�스??
// =============================================================================

/// [v2.9.6] 코베?�스 몬스?��? ?�하???�이???�형 (?�본: M3_WANTS* 마스??
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WantsFlag {
    Amulet,
    Bell,
    Candelabrum,
    Book,
    QuestArtifact,
}

/// [v2.9.6] which_arti ??마스?�→?�이??종류 매핑 (?�본: wizard.c L143-160)
pub fn which_artifact(flag: WantsFlag) -> i32 {
    match flag {
        WantsFlag::Amulet => 1,
        WantsFlag::Bell => 2,
        WantsFlag::Candelabrum => 3,
        WantsFlag::Book => 4,
        WantsFlag::QuestArtifact => 0,
    }
}

/// [v2.9.6] ?��???보유 ?�인 (?�본: mon_has_amulet L102-112)
pub fn mon_has_amulet_result(inventory_item_types: &[i32]) -> bool {
    inventory_item_types.iter().any(|&otyp| otyp == 1)
}

/// [v2.9.6] ?�수 ?�이??보유 ?�인 (?�본: mon_has_special L114-128)
pub fn mon_has_special_result(inventory_item_types: &[i32], has_quest_artifact: bool) -> bool {
    for &otyp in inventory_item_types {
        if otyp == 1 || otyp == 2 || otyp == 3 || otyp == 4 {
            return true;
        }
    }
    has_quest_artifact
}

/// [v2.9.6] ?�레?�어 ?�수 ?�이??보유 ?�태 (?�본: you_have L215-234)
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerSpecialItems {
    pub has_amulet: bool,
    pub has_bell: bool,
    pub has_menorah: bool,
    pub has_book: bool,
    pub has_quest_artifact: bool,
}

/// [v2.9.6] ?�레?�어 ?�이??보유 ?�인
pub fn player_has_item(items: &PlayerSpecialItems, flag: WantsFlag) -> bool {
    match flag {
        WantsFlag::Amulet => items.has_amulet,
        WantsFlag::Bell => items.has_bell,
        WantsFlag::Candelabrum => items.has_menorah,
        WantsFlag::Book => items.has_book,
        WantsFlag::QuestArtifact => items.has_quest_artifact,
    }
}

/// [v2.9.6] ?�략 목표 (?�본: STRAT_* 매크�?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyGoal {
    None,
    Heal,
    Player { x: i32, y: i32, wants: WantsFlag },
    Ground { x: i32, y: i32, wants: WantsFlag },
    Monster { x: i32, y: i32, wants: WantsFlag },
}

/// [v2.9.6] target_on 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOnResult {
    None,
    Player { x: i32, y: i32 },
    Ground { x: i32, y: i32 },
    Monster { x: i32, y: i32 },
}

impl Default for TargetOnResult {
    fn default() -> Self { TargetOnResult::None }
}

/// [v2.9.6] ?�략 ?�정 ?�력 (?�본: strategy L264-323)
#[derive(Debug, Clone)]
pub struct StrategyInput {
    pub is_covetous: bool,
    pub is_shopkeeper_in_shop: bool,
    pub is_priest_in_temple: bool,
    pub hp_ratio: i32,
    pub is_wizard_of_yendor: bool,
    pub amulet_made: bool,
    pub gate_invoked: bool,
    pub wants_amulet: bool,
    pub wants_bell: bool,
    pub wants_candelabrum: bool,
    pub wants_book: bool,
    pub wants_quest_artifact: bool,
    pub target_results: [TargetOnResult; 5],
}

/// [v2.9.6] ?�략 ?�정 (?�본: strategy L264-323)
pub fn strategy_result(input: &StrategyInput) -> StrategyGoal {
    if !input.is_covetous || input.is_shopkeeper_in_shop || input.is_priest_in_temple {
        return StrategyGoal::None;
    }
    let default_strat = match input.hp_ratio {
        0 => return StrategyGoal::Heal,
        1 => {
            if !input.is_wizard_of_yendor { return StrategyGoal::Heal; }
            StrategyGoal::Heal
        }
        2 => StrategyGoal::Heal,
        _ => StrategyGoal::None,
    };
    let flags_order = if input.gate_invoked { [0usize, 4, 3, 1, 2] } else { [0usize, 3, 1, 2, 4] };
    let wants_flags = [WantsFlag::Amulet, WantsFlag::Bell, WantsFlag::Candelabrum, WantsFlag::Book, WantsFlag::QuestArtifact];
    let wants_active = [
        input.wants_amulet && input.amulet_made,
        input.wants_bell, input.wants_candelabrum, input.wants_book, input.wants_quest_artifact,
    ];
    for &idx in &flags_order {
        if !wants_active[idx] { continue; }
        match input.target_results[idx] {
            TargetOnResult::Player { x, y } => return StrategyGoal::Player { x, y, wants: wants_flags[idx] },
            TargetOnResult::Ground { x, y } => return StrategyGoal::Ground { x, y, wants: wants_flags[idx] },
            TargetOnResult::Monster { x, y } => return StrategyGoal::Monster { x, y, wants: wants_flags[idx] },
            TargetOnResult::None => {}
        }
    }
    default_strat
}

/// [v2.9.6] 계단 ?�보 (?�본: choose_stairs L325-359)
#[derive(Debug, Clone, Copy, Default)]
pub struct StairsInfo {
    pub builds_up: bool,
    pub dn_stair: Option<(i32, i32)>,
    pub dn_ladder: Option<(i32, i32)>,
    pub up_stair: Option<(i32, i32)>,
    pub up_ladder: Option<(i32, i32)>,
    pub special_stair: Option<(i32, i32)>,
}

/// [v2.9.6] 계단 ?�택 (?�본: choose_stairs L325-359)
pub fn choose_stairs_result(info: &StairsInfo) -> Option<(i32, i32)> {
    let primary = if info.builds_up { info.dn_stair.or(info.dn_ladder) } else { info.up_stair.or(info.up_ladder) };
    primary.or(info.special_stair)
}

/// [v2.9.6] ?�술 ?�력 (?�본: tactics L361-451)
#[derive(Debug, Clone)]
pub struct TacticsInput {
    pub strategy: StrategyGoal,
    pub mon_x: i32,
    pub mon_y: i32,
    pub in_w_tower: bool,
    pub is_wizard: bool,
    pub has_amulet: bool,
    pub stairs_coord: Option<(i32, i32)>,
    pub hp: i32,
    pub maxhp: i32,
    pub is_fleeing: bool,
    pub dist_to_player_sq: i32,
    pub bolt_lim_sq: i32,
}

/// [v2.9.6] ?�술 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TacticsResult {
    DoNothing,
    RandomTeleport,
    MoveToStairs { x: i32, y: i32 },
    SelfHeal { amount: i32 },
    TeleportNearPlayer,
    TeleportAndPickup { x: i32, y: i32 },
    MoveBesideTarget { x: i32, y: i32 },
}

/// [v2.9.6] ?�술 ?�정 (?�본: tactics L361-451)
pub fn tactics_result(input: &TacticsInput, rng: &mut NetHackRng) -> TacticsResult {
    match input.strategy {
        StrategyGoal::Heal => {
            if input.in_w_tower || (input.is_wizard && input.stairs_coord.is_none() && !input.has_amulet) {
                if rng.rn2(3 + input.hp / 10) == 0 { return TacticsResult::RandomTeleport; }
            } else if let Some((sx, sy)) = input.stairs_coord {
                if input.mon_x != sx || input.mon_y != sy { return TacticsResult::MoveToStairs { x: sx, y: sy }; }
            }
            if input.dist_to_player_sq > input.bolt_lim_sq && input.hp <= input.maxhp - 8 {
                return TacticsResult::SelfHeal { amount: rng.rnd(8) };
            }
            if rng.rn2(if !input.is_fleeing { 5 } else { 33 }) == 0 { return TacticsResult::TeleportNearPlayer; }
            TacticsResult::DoNothing
        }
        StrategyGoal::None => {
            if rng.rn2(if !input.is_fleeing { 5 } else { 33 }) == 0 { return TacticsResult::TeleportNearPlayer; }
            TacticsResult::DoNothing
        }
        StrategyGoal::Player { .. } => TacticsResult::TeleportNearPlayer,
        StrategyGoal::Ground { x, y, .. } => TacticsResult::TeleportAndPickup { x, y },
        StrategyGoal::Monster { x, y, .. } => TacticsResult::MoveBesideTarget { x, y },
    }
}

/// [v2.9.6] ?�발 ?�???�보 (?�본: has_aggravatables L453-474)
#[derive(Debug, Clone, Copy)]
pub struct AggravatableMonster {
    pub is_dead: bool,
    pub in_same_area: bool,
    pub is_waiting: bool,
    pub is_sleeping: bool,
    pub cannot_move: bool,
}

/// [v2.9.6] ?�발 ?�??존재 ?�인
pub fn has_aggravatables_result(caster_same_area: bool, monsters: &[AggravatableMonster]) -> bool {
    if !caster_same_area { return false; }
    monsters.iter().any(|m| !m.is_dead && m.in_same_area && (m.is_waiting || m.is_sleeping || m.cannot_move))
}

/// [v2.9.6] ?�발 ?�과
#[derive(Debug, Clone)]
pub struct AggravateEffect { pub woken_count: i32, pub unfrozen_count: i32 }

/// [v2.9.6] ?�발 ?�과 계산 (?�본: aggravate L476-494)
pub fn aggravate_effect_result(monsters: &[AggravatableMonster], rng: &mut NetHackRng) -> AggravateEffect {
    let (mut woken, mut unfrozen) = (0, 0);
    for m in monsters {
        if m.is_dead || !m.in_same_area { continue; }
        if m.is_sleeping || m.is_waiting { woken += 1; }
        if m.cannot_move && rng.rn2(5) == 0 { unfrozen += 1; }
    }
    AggravateEffect { woken_count: woken, unfrozen_count: unfrozen }
}

/// [v2.9.6] nasty 몬스???�이�??�기
pub const NASTY_COUNT: usize = 42;

/// [v2.9.6] pick_nasty (?�본: pick_nasty L514-531)
pub fn pick_nasty_result(is_rogue: bool, rng: &mut NetHackRng) -> usize {
    let mut idx = rng.rn2(NASTY_COUNT as i32) as usize;
    if is_rogue { idx = rng.rn2(NASTY_COUNT as i32) as usize; }
    idx
}

/// [v2.9.6] nasty ?�환 ?�력
#[derive(Debug, Clone)]
pub struct NastySummonInput { pub caster_align: i32, pub player_level: i32, pub in_hell: bool }

/// [v2.9.6] nasty ?�환 결과
#[derive(Debug, Clone)]
pub struct NastySummonResult { pub monster_indices: Vec<usize>, pub demon_summon: bool }

/// [v2.9.6] nasty ?�환 ?�정 (?�본: nasty L533-625)
pub fn nasty_summon_result(input: &NastySummonInput, is_rogue: bool, rng: &mut NetHackRng) -> NastySummonResult {
    if rng.rn2(10) == 0 && input.in_hell {
        return NastySummonResult { monster_indices: vec![], demon_summon: true };
    }
    let tmp = if input.player_level > 3 { input.player_level / 3 } else { 1 };
    let mut indices = Vec::new();
    let outer = rng.rnd(tmp);
    for _ in 0..outer {
        if indices.len() >= 10 { break; }
        for _ in 0..20 {
            let idx = pick_nasty_result(is_rogue, rng);
            indices.push(idx);
            let mon_align = if idx < 18 { 0 } else if idx < 32 { -1 } else { 1 };
            if indices.len() >= 10 || idx < 18 || mon_align == input.caster_align { break; }
        }
    }
    NastySummonResult { monster_indices: indices, demon_summon: false }
}

/// [v2.9.6] ?�후 개입 ?�동 (?�본: intervene L681-708)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterveneAction { VagueNervousness, Curse, Aggravate, SummonNasties, Resurrect }

/// [v2.9.6] ?�후 개입 ?�정
pub fn intervene_result(is_astral: bool, rng: &mut NetHackRng) -> InterveneAction {
    let which = if is_astral { rng.rnd(4) } else { rng.rn2(6) };
    match which { 0 | 1 => InterveneAction::VagueNervousness, 2 => InterveneAction::Curse, 3 => InterveneAction::Aggravate, 4 => InterveneAction::SummonNasties, 5 => InterveneAction::Resurrect, _ => InterveneAction::VagueNervousness }
}

/// [v2.9.6] wizdead 결과 (?�본: wizdead L710-718)
#[derive(Debug, Clone)]
pub struct WizdeadResult { pub first_demigod: bool, pub demigod_counter: i32 }

/// [v2.9.6] ?��????�망 처리 ?�정
pub fn wizdead_result(already_demigod: bool, rng: &mut NetHackRng) -> WizdeadResult {
    if already_demigod { WizdeadResult { first_demigod: false, demigod_counter: 0 } }
    else { WizdeadResult { first_demigod: true, demigod_counter: rng.rn1(250, 50) } }
}

/// [v2.9.6] 모욕 ?�이�?(?�본: random_insult[] L720-729)
pub const RANDOM_INSULTS: &[&str] = &[
    "antic", "blackguard", "caitiff", "chucklehead", "coistrel", "craven", "cretin", "cur",
    "dastard", "demon fodder", "dimwit", "dolt", "fool", "footpad", "imbecile", "knave",
    "maledict", "miscreant", "niddering", "poltroon", "rattlepate", "reprobate", "scapegrace",
    "varlet", "villein", "wittol", "worm", "wretch",
];

/// [v2.9.6] ?�협 ?�이�?(?�본: random_malediction[] L731-738)
pub const RANDOM_MALEDICTIONS: &[&str] = &[
    "Hell shall soon claim thy remains,", "I chortle at thee, thou pathetic",
    "Prepare to die, thou", "Resistance is useless,", "Surrender or die, thou",
    "There shall be no mercy, thou", "Thou shalt repent of thy cunning,",
    "Thou art as a flea to me,", "Thou art doomed,", "Thy fate is sealed,",
    "Verily, thou shalt be one dead",
];

/// [v2.9.6] cuss 결과 (?�본: cuss L740-773)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CussResult {
    FiendishLaugh,
    RelinquishAmulet { insult: &'static str },
    PanicTaunt { message: &'static str, insult: &'static str },
    ParthianShot { message: &'static str },
    Malediction { malediction: &'static str, insult: &'static str },
    AngelicMessage,
    DemonicMessage { aspersion: bool },
    Deaf,
}

/// [v2.9.6] cuss ?�정 (?�본: cuss L740-773)
pub fn cuss_result(
    is_deaf: bool, is_wizard: bool, player_has_amulet: bool,
    player_hp: i32, monster_hp: i32,
    is_lawful_minion: bool, is_renegade: bool, is_minion: bool,
    rng: &mut NetHackRng,
) -> CussResult {
    if is_deaf { return CussResult::Deaf; }
    if is_wizard {
        if rng.rn2(5) == 0 { return CussResult::FiendishLaugh; }
        if player_has_amulet && rng.rn2(RANDOM_INSULTS.len() as i32) == 0 {
            let ins = RANDOM_INSULTS[rng.rn2(RANDOM_INSULTS.len() as i32) as usize];
            return CussResult::RelinquishAmulet { insult: ins };
        }
        if player_hp < 5 && rng.rn2(2) == 0 {
            let msg = if rng.rn2(2) == 0 { "Even now thy life force ebbs, %s!" } else { "Savor thy breath, %s, it be thy last!" };
            let ins = RANDOM_INSULTS[rng.rn2(RANDOM_INSULTS.len() as i32) as usize];
            return CussResult::PanicTaunt { message: msg, insult: ins };
        }
        if monster_hp < 5 && rng.rn2(2) == 0 {
            let msg = if rng.rn2(2) == 0 { "I shall return." } else { "I'll be back." };
            return CussResult::ParthianShot { message: msg };
        }
        let mal = RANDOM_MALEDICTIONS[rng.rn2(RANDOM_MALEDICTIONS.len() as i32) as usize];
        let ins = RANDOM_INSULTS[rng.rn2(RANDOM_INSULTS.len() as i32) as usize];
        return CussResult::Malediction { malediction: mal, insult: ins };
    }
    if is_lawful_minion && !is_renegade { return CussResult::AngelicMessage; }
    let aspersion = rng.rn2(if is_minion { 100 } else { 5 }) == 0;
    CussResult::DemonicMessage { aspersion }
}

/// [v2.9.6] 변???�이�?(?�본: wizapp[] L49-53)
pub const WIZARD_APPEARANCES: &[&str] = &[
    "Human", "Water Demon", "Vampire", "Red Dragon", "Troll", "Umber Hulk",
    "Xorn", "Xan", "Cockatrice", "Floating Eye", "Guardian Naga", "Trapper",
];

/// [v2.9.6] clonewiz 결과 (?�본: clonewiz L496-512)
#[derive(Debug, Clone)]
pub struct CloneWizResult { pub give_fake_amulet: bool, pub appearance_index: usize }

/// [v2.9.6] clonewiz ?�정
pub fn clonewiz_result(player_has_amulet: bool, rng: &mut NetHackRng) -> CloneWizResult {
    CloneWizResult { give_fake_amulet: !player_has_amulet && rng.rn2(2) != 0, appearance_index: rng.rn2(WIZARD_APPEARANCES.len() as i32) as usize }
}

/// [v2.9.6] ?��????�트 (?�본: amulet L55-100)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmuletHintResult { Hot, VeryWarm, Warm, Normal, NoHint }

/// [v2.9.6] ?��????�탈 감�?
pub fn amulet_hint_result(worn: bool, portal_dist_sq: Option<i32>, rng: &mut NetHackRng) -> AmuletHintResult {
    if !worn || rng.rn2(15) != 0 { return AmuletHintResult::NoHint; }
    match portal_dist_sq {
        Some(d) if d <= 9 => AmuletHintResult::Hot,
        Some(d) if d <= 64 => AmuletHintResult::VeryWarm,
        Some(d) if d <= 144 => AmuletHintResult::Warm,
        Some(_) => AmuletHintResult::Normal,
        None => AmuletHintResult::NoHint,
    }
}

// =============================================================================
// [v2.9.6] ?�스????wizard.c 추�? ?�식�?
// =============================================================================
#[cfg(test)]
mod wizard_phase2_tests {
    use super::*;

    #[test]
    fn test_which_artifact() {
        assert_eq!(which_artifact(WantsFlag::Amulet), 1);
        assert_eq!(which_artifact(WantsFlag::QuestArtifact), 0);
    }

    #[test]
    fn test_mon_has_amulet() {
        assert!(mon_has_amulet_result(&[5, 1, 3]));
        assert!(!mon_has_amulet_result(&[5, 6]));
    }

    #[test]
    fn test_mon_has_special() {
        assert!(mon_has_special_result(&[1], false));
        assert!(mon_has_special_result(&[5], true));
        assert!(!mon_has_special_result(&[5], false));
    }

    #[test]
    fn test_player_has_item() {
        let items = PlayerSpecialItems { has_amulet: true, has_bell: false, has_menorah: true, has_book: false, has_quest_artifact: false };
        assert!(player_has_item(&items, WantsFlag::Amulet));
        assert!(!player_has_item(&items, WantsFlag::Bell));
    }

    #[test]
    fn test_strategy_non_covetous() {
        let input = StrategyInput {
            is_covetous: false, is_shopkeeper_in_shop: false, is_priest_in_temple: false,
            hp_ratio: 3, is_wizard_of_yendor: false, amulet_made: false, gate_invoked: false,
            wants_amulet: false, wants_bell: false, wants_candelabrum: false,
            wants_book: false, wants_quest_artifact: false,
            target_results: [TargetOnResult::None; 5],
        };
        assert_eq!(strategy_result(&input), StrategyGoal::None);
    }

    #[test]
    fn test_strategy_heal() {
        let input = StrategyInput {
            is_covetous: true, is_shopkeeper_in_shop: false, is_priest_in_temple: false,
            hp_ratio: 0, is_wizard_of_yendor: false, amulet_made: false, gate_invoked: false,
            wants_amulet: false, wants_bell: false, wants_candelabrum: false,
            wants_book: false, wants_quest_artifact: false,
            target_results: [TargetOnResult::None; 5],
        };
        assert_eq!(strategy_result(&input), StrategyGoal::Heal);
    }

    #[test]
    fn test_strategy_target_amulet() {
        let mut targets = [TargetOnResult::None; 5];
        targets[0] = TargetOnResult::Player { x: 10, y: 20 };
        let input = StrategyInput {
            is_covetous: true, is_shopkeeper_in_shop: false, is_priest_in_temple: false,
            hp_ratio: 3, is_wizard_of_yendor: true, amulet_made: true, gate_invoked: false,
            wants_amulet: true, wants_bell: false, wants_candelabrum: false,
            wants_book: false, wants_quest_artifact: false,
            target_results: targets,
        };
        assert_eq!(strategy_result(&input), StrategyGoal::Player { x: 10, y: 20, wants: WantsFlag::Amulet });
    }

    #[test]
    fn test_choose_stairs() {
        let info = StairsInfo { builds_up: true, dn_stair: Some((5, 10)), ..Default::default() };
        assert_eq!(choose_stairs_result(&info), Some((5, 10)));
    }

    #[test]
    fn test_tactics_self_heal() {
        let mut rng = NetHackRng::new(42);
        let input = TacticsInput {
            strategy: StrategyGoal::Heal, mon_x: 5, mon_y: 5,
            in_w_tower: false, is_wizard: false, has_amulet: false,
            stairs_coord: Some((5, 5)), hp: 10, maxhp: 50,
            is_fleeing: false, dist_to_player_sq: 200, bolt_lim_sq: 64,
        };
        assert!(matches!(tactics_result(&input, &mut rng), TacticsResult::SelfHeal { .. }));
    }

    #[test]
    fn test_tactics_ground() {
        let mut rng = NetHackRng::new(99);
        let input = TacticsInput {
            strategy: StrategyGoal::Ground { x: 15, y: 20, wants: WantsFlag::Book },
            mon_x: 5, mon_y: 5, in_w_tower: false, is_wizard: true, has_amulet: false,
            stairs_coord: None, hp: 30, maxhp: 30, is_fleeing: false,
            dist_to_player_sq: 50, bolt_lim_sq: 64,
        };
        assert_eq!(tactics_result(&input, &mut rng), TacticsResult::TeleportAndPickup { x: 15, y: 20 });
    }

    #[test]
    fn test_has_aggravatables() {
        let m = vec![AggravatableMonster { is_dead: false, in_same_area: true, is_waiting: false, is_sleeping: true, cannot_move: false }];
        assert!(has_aggravatables_result(true, &m));
        assert!(!has_aggravatables_result(false, &m));
    }

    #[test]
    fn test_aggravate() {
        let mut rng = NetHackRng::new(42);
        let m = vec![AggravatableMonster { is_dead: false, in_same_area: true, is_waiting: true, is_sleeping: false, cannot_move: false }];
        let e = aggravate_effect_result(&m, &mut rng);
        assert!(e.woken_count >= 1);
    }

    #[test]
    fn test_pick_nasty() {
        let mut rng = NetHackRng::new(42);
        assert!(pick_nasty_result(false, &mut rng) < NASTY_COUNT);
    }

    #[test]
    fn test_nasty_summon() {
        let mut rng = NetHackRng::new(42);
        let input = NastySummonInput { caster_align: 0, player_level: 15, in_hell: false };
        assert!(!nasty_summon_result(&input, false, &mut rng).monster_indices.is_empty());
    }

    #[test]
    fn test_intervene_astral() {
        let mut rng = NetHackRng::new(42);
        for _ in 0..100 { assert_ne!(intervene_result(true, &mut rng), InterveneAction::Resurrect); }
    }

    #[test]
    fn test_wizdead() {
        let mut rng = NetHackRng::new(42);
        let r = wizdead_result(false, &mut rng);
        assert!(r.first_demigod && (50..=299).contains(&r.demigod_counter));
        assert!(!wizdead_result(true, &mut rng).first_demigod);
    }

    #[test]
    fn test_cuss_deaf() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(cuss_result(true, false, false, 50, 50, false, false, false, &mut rng), CussResult::Deaf);
    }

    #[test]
    fn test_cuss_wizard() {
        let mut rng = NetHackRng::new(42);
        let r = cuss_result(false, true, false, 50, 50, false, false, false, &mut rng);
        assert!(matches!(r, CussResult::FiendishLaugh | CussResult::Malediction { .. }));
    }

    #[test]
    fn test_cuss_angelic() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(cuss_result(false, false, false, 50, 50, true, false, true, &mut rng), CussResult::AngelicMessage);
    }

    #[test]
    fn test_clonewiz() {
        let mut rng = NetHackRng::new(42);
        assert!(clonewiz_result(false, &mut rng).appearance_index < WIZARD_APPEARANCES.len());
    }

    #[test]
    fn test_amulet_hint_not_worn() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(amulet_hint_result(false, Some(5), &mut rng), AmuletHintResult::NoHint);
    }
}
