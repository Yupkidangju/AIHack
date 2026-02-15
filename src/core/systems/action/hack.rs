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
// [v2.4.0
//
//
//
//
//
//
//
//
//
//
//
//
//
// =============================================================================

//
//
//

///
pub const DO_MOVE: i32 = 0;
pub const TEST_MOVE: i32 = 1;
pub const TEST_TRAV: i32 = 2;
pub const TEST_TRAP: i32 = 3;

//
//
//

///
pub const UNENCUMBERED: i32 = 0;
pub const SLT_ENCUMBER: i32 = 1; // Burdened
pub const MOD_ENCUMBER: i32 = 2; // Stressed
pub const HVY_ENCUMBER: i32 = 3; // Strained
pub const EXT_ENCUMBER: i32 = 4; // Overtaxed
pub const OVERLOADED: i32 = 5;

///
pub const MAX_CARR_CAP: i32 = 1000;

//
//
//

///
///
pub fn may_dig(is_stwall_or_tree: bool, is_nondiggable: bool) -> bool {
    !(is_stwall_or_tree && is_nondiggable)
}

///
pub fn may_passwall(is_stwall: bool, is_nonpasswall: bool) -> bool {
    !(is_stwall && is_nonpasswall)
}

///
///
///
///
pub fn bad_rock(
    is_sokoban_boulder: bool,
    is_rock: bool,
    can_tunnel: bool,
    needs_pick: bool,
    passes_walls: bool,
    passwall_ok: bool,
    dig_ok: bool,
) -> bool {
    is_sokoban_boulder
        || (is_rock && (!can_tunnel || needs_pick || !dig_ok) && !(passes_walls && passwall_ok))
}

//
//
//

///
///
pub fn cant_squeeze_thru(
    is_big: bool,
    is_amorphous: bool,
    is_whirly: bool,
    is_noncorporeal: bool,
    is_slithy: bool,
    can_fog: bool,
    current_load: i32,
    is_player_in_sokoban: bool,
    is_player: bool,
) -> i32 {
    //
    if is_big && !(is_amorphous || is_whirly || is_noncorporeal || is_slithy || can_fog) {
        return 1;
    }
    //
    if current_load > 600 {
        return 2;
    }
    //
    if is_player && is_player_in_sokoban {
        return 3;
    }
    0
}

//
//
//

///
///
///
///
///
///
pub fn weight_cap(
    str_val: i32,
    con_val: i32,
    is_polyd: bool,
    poly_cwt: i32,
    poly_msize: i32,
    poly_is_nymph: bool,
    poly_is_strong: bool,
    is_levitating: bool,
    is_airlevel: bool,
    has_strong_steed: bool,
    left_leg_wounded: bool,
    right_leg_wounded: bool,
    is_flying: bool,
) -> i32 {
    //
    let mut carrcap: i32 = 25 * (str_val + con_val) + 50;

    //
    if is_polyd {
        if poly_is_nymph {
            carrcap = MAX_CARR_CAP;
        } else if poly_cwt == 0 {
            //
            carrcap = carrcap * poly_msize / 2;
        } else if !poly_is_strong || (poly_is_strong && poly_cwt > 1450) {
            // WT_HUMAN = 1450
            carrcap = carrcap * poly_cwt / 1450;
        }
    }

    //
    if is_levitating || is_airlevel || has_strong_steed {
        carrcap = MAX_CARR_CAP;
    } else {
        if carrcap > MAX_CARR_CAP {
            carrcap = MAX_CARR_CAP;
        }
        //
        if !is_flying {
            if left_leg_wounded {
                carrcap -= 100;
            }
            if right_leg_wounded {
                carrcap -= 100;
            }
        }
        if carrcap < 0 {
            carrcap = 0;
        }
    }

    carrcap
}

//
//
//

///
///
///
pub fn calc_capacity(excess_weight: i32, capacity: i32) -> i32 {
    if excess_weight <= 0 {
        return UNENCUMBERED;
    }
    if capacity <= 1 {
        return OVERLOADED;
    }
    let cap = (excess_weight * 2 / capacity) + 1;
    cap.min(OVERLOADED)
}

///
pub fn near_capacity(excess_weight: i32, capacity: i32) -> i32 {
    calc_capacity(excess_weight, capacity)
}

///
pub fn max_capacity(excess_weight: i32, capacity: i32) -> i32 {
    excess_weight - (2 * capacity)
}

///
pub fn check_capacity(excess_weight: i32, capacity: i32) -> bool {
    near_capacity(excess_weight, capacity) >= EXT_ENCUMBER
}

//
//
//

///
#[derive(Debug, Clone)]
pub struct LoseHpResult {
    /// ??HP
    pub new_hp: i32,
    ///
    pub new_maxhp: i32,
    ///
    pub died: bool,
    ///
    pub need_wail: bool,
    ///
    pub need_stop_running: bool,
}

///
///
///
pub fn losehp(n: i32, hp: i32, maxhp: i32, is_polyd: bool) -> LoseHpResult {
    let new_hp = hp - n;

    if is_polyd {
        //
        let new_maxhp = if maxhp < new_hp { new_hp } else { maxhp };
        LoseHpResult {
            new_hp,
            new_maxhp,
            died: new_hp < 1,
            //
            need_wail: n > 0 && new_hp * 10 < new_maxhp,
            need_stop_running: false,
        }
    } else {
        let (new_maxhp, stop_run) = if new_hp > maxhp {
            //
            (new_hp, false)
        } else {
            //
            (maxhp, n > 0)
        };

        LoseHpResult {
            new_hp,
            new_maxhp,
            died: new_hp < 1,
            need_wail: n > 0 && new_hp > 0 && new_hp * 10 < new_maxhp,
            need_stop_running: stop_run,
        }
    }
}

//
//
//

///
#[derive(Debug, Clone)]
pub struct OverexertionResult {
    ///
    pub hp_loss: i32,
    ///
    pub fainted: bool,
}

///
///
pub fn overexertion(turn: u64, encumbrance: i32, hp: i32) -> OverexertionResult {
    //
    if (turn % 3) != 0 && encumbrance >= HVY_ENCUMBER {
        if hp > 1 {
            OverexertionResult {
                hp_loss: 1,
                fainted: false,
            }
        } else {
            //
            OverexertionResult {
                hp_loss: 0,
                fainted: true,
            }
        }
    } else {
        OverexertionResult {
            hp_loss: 0,
            fainted: false,
        }
    }
}

//
//
//

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WailRole {
    Wizard,
    Valkyrie,
    Elf,
    Other,
}

///
///
///
pub fn maybe_wail(
    hp: i32,
    role: WailRole,
    power_count: i32,
    last_wail_turn: u64,
    current_turn: u64,
) -> Option<String> {
    //
    if current_turn <= last_wail_turn + 50 {
        return None;
    }

    match role {
        WailRole::Wizard | WailRole::Valkyrie | WailRole::Elf => {
            let who = match role {
                WailRole::Wizard => "Wizard",
                WailRole::Valkyrie => "Valkyrie",
                WailRole::Elf => "Elf",
                _ => "Hero",
            };
            if hp == 1 {
                Some(format!("{} is about to die.", who))
            } else if power_count >= 4 {
                Some(format!("{}, all your powers will be lost...", who))
            } else {
                Some(format!("{}, your life force is running out.", who))
            }
        }
        WailRole::Other => {
            if hp == 1 {
                Some("You hear the wailing of the Banshee...".to_string())
            } else {
                Some("You hear the howling of the CwnAnnwn...".to_string())
            }
        }
    }
}

//
//
//

///
#[derive(Debug, Clone, Default)]
pub struct MultiTurnState {
    ///
    pub multi: i32,
    ///
    pub multi_reason: Option<String>,
    ///
    pub is_running: bool,
    ///
    pub is_traveling: bool,
    ///
    pub is_sleeping: bool,
    ///
    pub invulnerable: bool,
}

///
pub fn nomul(state: &mut MultiTurnState, nval: i32) {
    if state.multi < nval {
        return;
    }
    state.invulnerable = false;
    state.is_sleeping = false;
    state.multi = nval;
    if nval == 0 {
        state.multi_reason = None;
    }
    state.is_traveling = false;
    state.is_running = false;
}

///
pub fn unmul(state: &mut MultiTurnState, msg_override: Option<&str>) -> Option<String> {
    state.multi = 0;
    state.is_sleeping = false;
    state.multi_reason = None;

    let msg = msg_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| "You can move again.".to_string());

    Some(msg)
}

//
//
//

///
#[derive(Debug, Clone)]
pub struct NearbyMonsterInfo {
    ///
    pub dx: i32,
    pub dy: i32,
    ///
    pub is_hostile: bool,
    ///
    pub is_visible: bool,
}

///
///
pub fn monster_nearby(neighbors: &[NearbyMonsterInfo]) -> bool {
    neighbors.iter().any(|m| m.is_hostile && m.is_visible)
}

//
//
//

///
#[derive(Debug, Clone)]
pub struct PoolEffectResult {
    ///
    pub left_water: bool,
    ///
    pub entered_water: bool,
    ///
    pub entered_lava: bool,
    /// 硫붿떆吏
    pub message: Option<String>,
    ///
    pub need_vision_recalc: bool,
}

///
pub fn leaving_water_message(
    is_on_pool: bool,
    is_on_lava: bool,
    is_on_ice: bool,
    is_levitating: bool,
    is_flying: bool,
    is_water_walking: bool,
) -> Option<String> {
    if !is_on_pool {
        if is_on_lava {
            Some("You leave the water...".to_string())
        } else if is_on_ice {
            Some("You are on solid ice again.".to_string())
        } else {
            Some("You are on solid land again.".to_string())
        }
    } else if is_levitating {
        Some("You pop out of the water like a cork!".to_string())
    } else if is_flying {
        Some("You fly out of the water.".to_string())
    } else if is_water_walking {
        Some("You slowly rise above the surface.".to_string())
    } else {
        None
    }
}

//
//
//

///
#[derive(Debug, Clone)]
pub struct TerrainSwitchResult {
    ///
    pub levitation_blocked: bool,
    ///
    pub flying_blocked: bool,
    ///
    pub levitation_restored: bool,
    ///
    pub flying_restored: bool,
    /// 硫붿떆吏
    pub messages: Vec<String>,
}

///
///
///
///
pub fn switch_terrain(
    is_rock_or_closed_door: bool,
    was_levitating: bool,
    was_flying: bool,
    currently_levitating: bool,
    currently_flying: bool,
) -> TerrainSwitchResult {
    let mut result = TerrainSwitchResult {
        levitation_blocked: false,
        flying_blocked: false,
        levitation_restored: false,
        flying_restored: false,
        messages: Vec::new(),
    };

    if is_rock_or_closed_door {
        //
        if currently_levitating {
            result.levitation_blocked = true;
            result
                .messages
                .push("You can't levitate in here.".to_string());
        }
        if currently_flying {
            result.flying_blocked = true;
            result.messages.push("You can't fly in here.".to_string());
        }
    } else {
        //
        if was_levitating && !currently_levitating {
            //
        }
        if !was_flying && currently_flying {
            result.flying_restored = true;
            result.messages.push("You start flying.".to_string());
        }
    }

    result
}

//
//
//

///
pub fn invocation_pos(x: i32, y: i32, inv_x: i32, inv_y: i32, is_invocation_level: bool) -> bool {
    is_invocation_level && x == inv_x && y == inv_y
}

//
//
//

///
pub const D_NODOOR: u8 = 0x01;
pub const D_BROKEN: u8 = 0x02;
pub const D_ISOPEN: u8 = 0x04;
pub const D_CLOSED: u8 = 0x08;
pub const D_LOCKED: u8 = 0x10;
pub const D_TRAPPED: u8 = 0x20;

///
pub fn doorless_door(is_door: bool, doormask: u8, is_rogue_level: bool) -> bool {
    if !is_door {
        return false;
    }
    //
    if is_rogue_level {
        return false;
    }
    (doormask & !(D_NODOOR | D_BROKEN)) == 0
}

//
//
//

///
///
///
///
pub fn crawl_destination(
    is_good_pos: bool,
    ux: i32,
    uy: i32,
    x: i32,
    y: i32,
    no_diagonal: bool,
    passes_walls: bool,
    is_door_blocked: bool,
    is_rock_blocked: bool,
) -> bool {
    if !is_good_pos {
        return false;
    }
    //
    if x == ux || y == uy {
        return true;
    }
    //
    if no_diagonal {
        return false;
    }
    if passes_walls {
        return true;
    }
    if is_door_blocked {
        return false;
    }
    !is_rock_blocked
}

//
//
//

///
#[derive(Debug, Clone)]
pub struct ChewState {
    ///
    pub pos_x: i32,
    pub pos_y: i32,
    ///
    pub effort: i32,
    ///
    pub chewing_down: bool,
    ///
    pub is_chewing: bool,
}

impl ChewState {
    pub fn new() -> Self {
        Self {
            pos_x: 0,
            pos_y: 0,
            effort: 0,
            chewing_down: false,
            is_chewing: false,
        }
    }
}

///
#[derive(Debug, Clone)]
pub enum ChewResult {
    ///
    StillChewing(String),
    ///
    Completed {
        nutrition_gain: i32,
        message: String,
    },
    ///
    Blocked(String),
}

///
///
pub fn advance_chew(
    state: &mut ChewState,
    x: i32,
    y: i32,
    damage_inc: i32,
    is_nondiggable: bool,
    is_same_target: bool,
) -> ChewResult {
    //
    if is_nondiggable {
        return ChewResult::Blocked("You hurt your teeth on the hard stone.".to_string());
    }

    if !is_same_target || !state.is_chewing {
        //
        state.pos_x = x;
        state.pos_y = y;
        state.is_chewing = true;
        state.effort = 30 + damage_inc;
        return ChewResult::StillChewing("You start chewing.".to_string());
    }

    //
    state.effort += 30 + damage_inc;
    if state.effort <= 100 {
        return ChewResult::StillChewing("You continue chewing.".to_string());
    }

    //
    state.is_chewing = false;
    state.effort = 0;
    ChewResult::Completed {
        nutrition_gain: 20,
        message: "You chew through!".to_string(),
    }
}

//
//
//

///
pub fn inv_cnt(item_count: usize, gold_count: usize, include_gold: bool) -> usize {
    if include_gold {
        item_count
    } else {
        item_count - gold_count.min(item_count)
    }
}

///
///
pub fn money_cnt(items: &[(bool, i64)]) -> i64 {
    for (is_coin, qty) in items {
        if *is_coin {
            return *qty;
        }
    }
    0
}

//
//
//

///
///
pub fn should_revive_nasty(
    x: i32,
    y: i32,
    rider_corpse_count: i32,
    has_amulet_of_yendor: bool,
) -> bool {
    //
    rider_corpse_count > 0 || has_amulet_of_yendor
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_may_dig() {
        assert!(may_dig(true, false));
        assert!(!may_dig(true, true));
        assert!(may_dig(false, true));
    }

    #[test]
    fn test_may_passwall() {
        assert!(may_passwall(false, false));
        assert!(!may_passwall(true, true));
        assert!(may_passwall(true, false));
    }

    #[test]
    fn test_bad_rock() {
        //
        assert!(bad_rock(true, false, false, false, false, false, false));
        //
        assert!(!bad_rock(false, true, true, false, false, false, true));
        //
        assert!(!bad_rock(false, true, false, false, true, true, false));
        //
        assert!(bad_rock(false, true, false, false, false, false, false));
    }

    #[test]
    fn test_cant_squeeze_thru() {
        //
        assert_eq!(
            cant_squeeze_thru(false, false, false, false, false, false, 0, false, false),
            0
        );
        //
        assert_eq!(
            cant_squeeze_thru(true, true, false, false, false, false, 0, false, false),
            0
        );
        //
        assert_eq!(
            cant_squeeze_thru(true, false, false, false, false, false, 0, false, false),
            1
        );
        //
        assert_eq!(
            cant_squeeze_thru(false, false, false, false, false, false, 700, false, false),
            2
        );
        //
        assert_eq!(
            cant_squeeze_thru(false, false, false, false, false, false, 0, true, true),
            3
        );
    }

    #[test]
    fn test_weight_cap_basic() {
        //
        let cap = weight_cap(
            18, 18, false, 0, 0, false, false, false, false, false, false, false, false,
        );
        assert_eq!(cap, 950);

        //
        let cap = weight_cap(
            10, 10, false, 0, 0, false, false, true, false, false, false, false, false,
        );
        assert_eq!(cap, MAX_CARR_CAP);

        //
        let cap = weight_cap(
            18, 18, false, 0, 0, false, false, false, false, false, true, true, false,
        );
        assert_eq!(cap, 750); // 950-200
    }

    #[test]
    fn test_calc_capacity() {
        //
        assert_eq!(calc_capacity(-100, 500), UNENCUMBERED);
        //
        assert!(calc_capacity(100, 500) >= SLT_ENCUMBER);
        //
        assert_eq!(calc_capacity(5000, 1), OVERLOADED);
    }

    #[test]
    fn test_losehp() {
        //
        let r = losehp(5, 20, 30, false);
        assert_eq!(r.new_hp, 15);
        assert!(!r.died);
        assert!(r.need_stop_running);

        //
        let r = losehp(10, 5, 30, false);
        assert!(r.died);

        //
        let r = losehp(-5, 30, 30, false);
        assert_eq!(r.new_hp, 35);
        assert_eq!(r.new_maxhp, 35);
        assert!(!r.need_stop_running);

        //
        let r = losehp(9, 10, 100, false);
        assert!(r.need_wail);
    }

    #[test]
    fn test_overexertion() {
        //
        let r = overexertion(3, HVY_ENCUMBER, 10);
        assert_eq!(r.hp_loss, 0);

        //
        let r = overexertion(4, HVY_ENCUMBER, 10);
        assert_eq!(r.hp_loss, 1);

        //
        let r = overexertion(4, HVY_ENCUMBER, 1);
        assert!(r.fainted);
    }

    #[test]
    fn test_maybe_wail() {
        //
        assert!(maybe_wail(5, WailRole::Wizard, 0, 100, 140).is_none());

        //
        let msg = maybe_wail(1, WailRole::Wizard, 0, 0, 100);
        assert!(msg.unwrap().contains("about to die"));

        //
        let msg = maybe_wail(5, WailRole::Wizard, 5, 0, 100);
        assert!(msg.unwrap().contains("powers will be lost"));

        //
        let msg = maybe_wail(1, WailRole::Other, 0, 0, 100);
        assert!(msg.unwrap().contains("Banshee"));
    }

    #[test]
    fn test_nomul_unmul() {
        let mut state = MultiTurnState::default();
        state.multi = 5;
        state.is_running = true;
        state.is_traveling = true;

        nomul(&mut state, 0);
        assert_eq!(state.multi, 0);
        assert!(!state.is_running);
        assert!(!state.is_traveling);

        let msg = unmul(&mut state, Some("Done eating."));
        assert_eq!(msg.unwrap(), "Done eating.");
    }

    #[test]
    fn test_monster_nearby() {
        let neighbors = vec![NearbyMonsterInfo {
            dx: 1,
            dy: 0,
            is_hostile: true,
            is_visible: true,
        }];
        assert!(monster_nearby(&neighbors));

        let neighbors = vec![NearbyMonsterInfo {
            dx: 1,
            dy: 0,
            is_hostile: false,
            is_visible: true,
        }];
        assert!(!monster_nearby(&neighbors));
    }

    #[test]
    fn test_doorless_door() {
        assert!(doorless_door(true, D_NODOOR, false));
        assert!(doorless_door(true, D_BROKEN, false));
        assert!(!doorless_door(true, D_CLOSED, false));
        assert!(!doorless_door(true, D_NODOOR, true));
        assert!(!doorless_door(false, D_NODOOR, false));
    }

    #[test]
    fn test_crawl_destination() {
        //
        assert!(crawl_destination(
            true, 5, 5, 6, 5, false, false, false, false
        ));
        //
        assert!(crawl_destination(
            true, 5, 5, 6, 6, false, false, false, false
        ));
        //
        assert!(!crawl_destination(
            true, 5, 5, 6, 6, true, false, false, false
        ));
        //
        assert!(!crawl_destination(
            true, 5, 5, 6, 6, false, false, true, false
        ));
    }

    #[test]
    fn test_chew_progress() {
        let mut state = ChewState::new();

        //
        let r = advance_chew(&mut state, 5, 5, 0, false, false);
        assert!(matches!(r, ChewResult::StillChewing(_)));

        //
        let r = advance_chew(&mut state, 5, 5, 0, false, true);
        assert!(matches!(r, ChewResult::StillChewing(_)));

        //
        let r = advance_chew(&mut state, 5, 5, 0, true, true);
        assert!(matches!(r, ChewResult::Blocked(_)));
    }

    #[test]
    fn test_money_cnt() {
        let items = vec![(false, 100), (true, 500), (false, 50)];
        assert_eq!(money_cnt(&items), 500);
        let items: Vec<(bool, i64)> = vec![(false, 100)];
        assert_eq!(money_cnt(&items), 0);
    }

    #[test]
    fn test_inv_cnt() {
        assert_eq!(inv_cnt(10, 2, true), 10);
        assert_eq!(inv_cnt(10, 2, false), 8);
    }

    #[test]
    fn test_invocation_pos() {
        assert!(invocation_pos(5, 10, 5, 10, true));
        assert!(!invocation_pos(5, 10, 5, 10, false));
        assert!(!invocation_pos(5, 10, 6, 10, true));
    }
}
