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

// =============================================================================
// [v2.4.0] hack.c 대량 이식
// 원본: nethack-3.6.7/src/hack.c (3,177줄)
// 함정 탈출, 얼음 미끄러짐, 경로 탐색, spoteffects, 방 판정 등
// =============================================================================

// ---------------------------------------------------------------------------
// 함정 탈출 시스템 (원본: hack.c:1196-1335 trapmove)
// ---------------------------------------------------------------------------

/// [v2.4.0] 함정 종류 (원본: TT_ 상수)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapType {
    /// 곰 덫 (원본: TT_BEARTRAP)
    BearTrap,
    /// 구덩이 (원본: TT_PIT)
    Pit,
    /// 거미줄 (원본: TT_WEB)
    Web,
    /// 용암 (원본: TT_LAVA)
    Lava,
    /// 바닥에 끼임 (원본: TT_INFLOOR)
    InFloor,
    /// 매장된 공 (원본: TT_BURIEDBALL)
    BuriedBall,
}

/// [v2.4.0] 함정 탈출 시도 결과 (원본: trapmove 반환값)
#[derive(Debug, Clone)]
pub struct TrapMoveResult {
    /// 이동 가능 여부
    pub can_move: bool,
    /// 함정에서 탈출 여부
    pub escaped: bool,
    /// 함정 카운터 감소 후 잔여값
    pub remaining_trap: i32,
    /// 메시지
    pub message: String,
}

/// [v2.4.0] 함정 탈출 시도 (원본: hack.c:1196-1335 trapmove)
pub fn trapmove(
    trap_type: TrapType,
    utrap: i32,
    dx: i32,
    dy: i32,
    dest_is_pit: bool,
    dest_is_lava: bool,
    has_sting: bool,
    rng_val: i32,
) -> TrapMoveResult {
    match trap_type {
        TrapType::BearTrap => {
            // 원본: 대각선 이동 또는 1/5 확률로 탈출 진행
            let new_utrap = if (dx != 0 && dy != 0) || rng_val % 5 == 0 {
                utrap - 1
            } else {
                utrap
            };
            if new_utrap <= 0 {
                TrapMoveResult {
                    can_move: false,
                    escaped: true,
                    remaining_trap: 0,
                    message: "You finally wriggle free.".into(),
                }
            } else {
                TrapMoveResult {
                    can_move: false,
                    escaped: false,
                    remaining_trap: new_utrap,
                    message: "You are caught in a bear trap.".into(),
                }
            }
        }
        TrapType::Pit => {
            // 원본: 인접 구덩이로 이동 가능
            if dest_is_pit {
                TrapMoveResult {
                    can_move: true,
                    escaped: false,
                    remaining_trap: utrap,
                    message: "You move into the adjacent pit.".into(),
                }
            } else {
                TrapMoveResult {
                    can_move: false,
                    escaped: false,
                    remaining_trap: utrap,
                    message: "You try to climb out of the pit.".into(),
                }
            }
        }
        TrapType::Web => {
            // 원본: Sting 무기로 즉시 탈출
            if has_sting {
                TrapMoveResult {
                    can_move: false,
                    escaped: true,
                    remaining_trap: 0,
                    message: "Sting cuts through the web!".into(),
                }
            } else {
                let new_utrap = utrap - 1;
                if new_utrap <= 0 {
                    TrapMoveResult {
                        can_move: false,
                        escaped: true,
                        remaining_trap: 0,
                        message: "You disentangle yourself.".into(),
                    }
                } else {
                    TrapMoveResult {
                        can_move: false,
                        escaped: false,
                        remaining_trap: new_utrap,
                        message: "You are stuck to the web.".into(),
                    }
                }
            }
        }
        TrapType::Lava => {
            // 원본: 용암이 아닌 곳으로 이동 시도
            let new_utrap = if !dest_is_lava { utrap - 1 } else { utrap };
            if new_utrap <= 0 && !dest_is_lava {
                TrapMoveResult {
                    can_move: false,
                    escaped: true,
                    remaining_trap: 0,
                    message: "You pull yourself to the edge of the lava.".into(),
                }
            } else {
                TrapMoveResult {
                    can_move: false,
                    escaped: false,
                    remaining_trap: new_utrap,
                    message: "You are stuck in the lava.".into(),
                }
            }
        }
        TrapType::InFloor => {
            let new_utrap = utrap - 1;
            if new_utrap <= 0 {
                TrapMoveResult {
                    can_move: false,
                    escaped: true,
                    remaining_trap: 0,
                    message: "You finally wriggle free.".into(),
                }
            } else {
                TrapMoveResult {
                    can_move: false,
                    escaped: false,
                    remaining_trap: new_utrap,
                    message: "You are stuck in the floor.".into(),
                }
            }
        }
        TrapType::BuriedBall => {
            let new_utrap = utrap - 1;
            if new_utrap <= 0 {
                TrapMoveResult {
                    can_move: false,
                    escaped: true,
                    remaining_trap: 0,
                    message: "You finally wrench the ball free.".into(),
                }
            } else {
                TrapMoveResult {
                    can_move: false,
                    escaped: false,
                    remaining_trap: new_utrap,
                    message: "You are chained to the buried ball.".into(),
                }
            }
        }
    }
}

/// [v2.4.0] 뿌리 박힌 상태 판정 (원본: hack.c:1337-1349 u_rooted)
pub fn u_rooted(
    monster_move_speed: i32,
    is_levitating: bool,
    is_air_level: bool,
) -> Option<&'static str> {
    if monster_move_speed == 0 {
        let location = if is_levitating || is_air_level {
            "in place"
        } else {
            "to the ground"
        };
        Some(if location == "in place" {
            "You are rooted in place."
        } else {
            "You are rooted to the ground."
        })
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// 얼음 미끄러짐 (원본: hack.c:1421-1439)
// ---------------------------------------------------------------------------

/// [v2.4.0] 얼음 위 미끄러짐 판정 (원본: domove_core 내 on_ice 처리)
pub fn check_ice_slip(
    is_ice: bool,
    is_levitating: bool,
    has_skates: bool,
    cold_resistant: bool,
    is_flying: bool,
    is_clinger: bool,
    rng_val: i32,
) -> bool {
    if !is_ice || is_levitating || has_skates || is_flying || is_clinger {
        return false;
    }
    // 원본: Cold_resistance ? 1/3 : 1/2 확률로 미끄러짐
    let chance = if cold_resistant { 3 } else { 2 };
    rng_val % chance == 0
}

// ---------------------------------------------------------------------------
// 혼란/기절 방향 변경 (원본: hack.c:1443-1455)
// ---------------------------------------------------------------------------

/// [v2.4.0] 혼란 시 방향 변경 결과
#[derive(Debug, Clone)]
pub struct ConfusedDirection {
    pub dx: i32,
    pub dy: i32,
}

/// [v2.4.0] 혼란/기절 상태에서 방향 무작위화 (원본: confdir)
pub fn confused_direction(rng_val: i32) -> ConfusedDirection {
    // 원본: xdir/ydir 배열 기반 — 8방향 무작위
    let dirs: [(i32, i32); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let idx = (rng_val.unsigned_abs() as usize) % 8;
    ConfusedDirection {
        dx: dirs[idx].0,
        dy: dirs[idx].1,
    }
}

// ---------------------------------------------------------------------------
// 이동 판정 보조 (원본: hack.c:700-923 test_move 지원)
// ---------------------------------------------------------------------------

/// [v2.4.0] 이동 모드 (원본: DO_MOVE, TEST_MOVE, TEST_TRAV, TEST_TRAP)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMode {
    DoMove,
    TestMove,
    TestTravel,
    TestTrap,
}

/// [v2.4.0] 이동 판정 결과
#[derive(Debug, Clone)]
pub struct TestMoveResult {
    /// 이동 가능 여부
    pub can_move: bool,
    /// 문이 열렸는지 여부
    pub door_opened: bool,
    /// 메시지
    pub message: Option<String>,
}

/// [v2.4.0] 이동 판정 (원본: hack.c:700-923 test_move 간소화)
pub fn test_move(
    is_rock: bool,
    is_ironbars: bool,
    is_closed_door: bool,
    is_pool: bool,
    is_lava: bool,
    passes_walls: bool,
    can_tunnel: bool,
    can_ooze: bool,
    is_underwater: bool,
    mode: MoveMode,
    mention_walls: bool,
) -> TestMoveResult {
    // 바위/철창 판정
    if is_rock || is_ironbars {
        if passes_walls && !is_ironbars {
            return TestMoveResult {
                can_move: true,
                door_opened: false,
                message: None,
            };
        }
        if is_ironbars && !passes_walls {
            return TestMoveResult {
                can_move: false,
                door_opened: false,
                message: if mode == MoveMode::DoMove && mention_walls {
                    Some("You cannot pass through the bars.".into())
                } else {
                    None
                },
            };
        }
        if can_tunnel {
            // 터널링 몬스터 — 씹어서 통과 (still_chewing 체크 필요)
            return TestMoveResult {
                can_move: false,
                door_opened: false,
                message: Some("You start chewing through the rock.".into()),
            };
        }
        return TestMoveResult {
            can_move: false,
            door_opened: false,
            message: if mode == MoveMode::DoMove && mention_walls {
                Some("It's a wall.".into())
            } else {
                None
            },
        };
    }

    // 닫힌 문 판정
    if is_closed_door {
        if passes_walls {
            return TestMoveResult {
                can_move: true,
                door_opened: false,
                message: None,
            };
        }
        if can_ooze {
            return TestMoveResult {
                can_move: true,
                door_opened: false,
                message: if mode == MoveMode::DoMove {
                    Some("You ooze under the door.".into())
                } else {
                    None
                },
            };
        }
        if is_underwater {
            return TestMoveResult {
                can_move: false,
                door_opened: false,
                message: Some("There is an obstacle there.".into()),
            };
        }
        if can_tunnel {
            return TestMoveResult {
                can_move: false,
                door_opened: false,
                message: Some("You start chewing through the door.".into()),
            };
        }
        return TestMoveResult {
            can_move: false,
            door_opened: false,
            message: if mode == MoveMode::DoMove {
                Some("That door is closed.".into())
            } else {
                None
            },
        };
    }

    // 물/용암 — 기본적으로 통과 가능 (추가 효과는 spoteffects에서 처리)
    TestMoveResult {
        can_move: true,
        door_opened: false,
        message: None,
    }
}

// ---------------------------------------------------------------------------
// 경로 탐색 (원본: hack.c:925-1169 findtravelpath 간소화)
// ---------------------------------------------------------------------------

/// [v2.4.0] 최단 경로 단계 결과
#[derive(Debug, Clone)]
pub struct TravelStep {
    pub dx: i32,
    pub dy: i32,
    pub found: bool,
}

/// [v2.4.0] BFS 기반 이동 경로 탐색 (원본: findtravelpath 간소화)
/// 실제 맵 데이터는 호출 측에서 관리, 여기서는 순수 알고리즘만 이식
pub fn find_travel_step(
    ux: i32,
    uy: i32,
    tx: i32,
    ty: i32,
    passable: &dyn Fn(i32, i32) -> bool,
    cols: i32,
    rows: i32,
) -> TravelStep {
    // 인접 이동 직접 판정
    let ddx = tx - ux;
    let ddy = ty - uy;
    if ddx.abs() <= 1 && ddy.abs() <= 1 && passable(tx, ty) {
        return TravelStep {
            dx: ddx,
            dy: ddy,
            found: true,
        };
    }

    // BFS (원본: travel[][] 배열 기반)
    let w = cols as usize;
    let h = rows as usize;
    let mut visited = vec![vec![false; h]; w];
    let mut parent = vec![vec![(-1i32, -1i32); h]; w];
    let mut queue = std::collections::VecDeque::new();

    let ui = ux as usize;
    let uj = uy as usize;
    visited[ui][uj] = true;
    queue.push_back((ux, uy));

    let dirs: [(i32, i32); 8] = [
        (0, -1),
        (0, 1),
        (-1, 0),
        (1, 0),
        (-1, -1),
        (1, -1),
        (-1, 1),
        (1, 1),
    ];

    while let Some((cx, cy)) = queue.pop_front() {
        if cx == tx && cy == ty {
            // 경로 역추적 — 첫 번째 단계 찾기
            let mut px = tx;
            let mut py = ty;
            loop {
                let (ppx, ppy) = parent[px as usize][py as usize];
                if ppx == ux && ppy == uy {
                    return TravelStep {
                        dx: px - ux,
                        dy: py - uy,
                        found: true,
                    };
                }
                if ppx < 0 {
                    break;
                }
                px = ppx;
                py = ppy;
            }
        }
        for &(ddx, ddy) in &dirs {
            let nx = cx + ddx;
            let ny = cy + ddy;
            if nx >= 0
                && nx < cols
                && ny >= 0
                && ny < rows
                && !visited[nx as usize][ny as usize]
                && passable(nx, ny)
            {
                visited[nx as usize][ny as usize] = true;
                parent[nx as usize][ny as usize] = (cx, cy);
                queue.push_back((nx, ny));
            }
        }
    }

    TravelStep {
        dx: 0,
        dy: 0,
        found: false,
    }
}

// ---------------------------------------------------------------------------
// 방 판정 (원본: hack.c:2310-2400 in_rooms/in_town)
// ---------------------------------------------------------------------------

/// [v2.4.0] 좌표가 방 내부인지 판정 (원본: in_rooms 간소화)
pub fn in_room(x: i32, y: i32, room_lx: i32, room_ly: i32, room_hx: i32, room_hy: i32) -> bool {
    x >= room_lx && x <= room_hx && y >= room_ly && y <= room_hy
}

/// [v2.4.0] 마을 레벨 판정 (원본: in_town)
pub fn in_town(x: i32, y: i32, is_town_level: bool, rooms: &[(i32, i32, i32, i32, usize)]) -> bool {
    if !is_town_level {
        return false;
    }
    let has_subrooms = rooms.iter().any(|r| r.4 > 0);
    if !has_subrooms {
        return true;
    } // 서브방 없으면 전체가 마을
    rooms
        .iter()
        .any(|r| r.4 > 0 && in_room(x, y, r.0, r.1, r.2, r.3))
}

// ---------------------------------------------------------------------------
// Spoteffects 보조 (원본: hack.c:2145-2290)
// ---------------------------------------------------------------------------

/// [v2.4.0] 위치 효과 판정 결과
#[derive(Debug, Clone)]
pub struct SpotEffect {
    /// 함정 발동
    pub trigger_trap: bool,
    /// 지형 전환 필요
    pub terrain_switch: bool,
    /// 물/용암 효과
    pub pool_effect: bool,
    /// 아이템 줍기
    pub pickup: bool,
    /// 특수 방 체크
    pub check_special_room: bool,
    /// 싱크홀 낙하
    pub sink_fall: bool,
    /// 얼음 경고
    pub ice_warning: Option<&'static str>,
    /// 천장 몬스터 낙하
    pub ceiling_fall: bool,
}

/// [v2.4.0] 위치 효과 판정 (원본: spoteffects 간소화)
pub fn spot_effects(
    is_new_spot: bool,
    has_trap: bool,
    trap_is_pit: bool,
    is_ice: bool,
    ice_time_left: i32,
    is_sink: bool,
    is_levitating: bool,
    has_ceiling_monster: bool,
    terrain_changed: bool,
) -> SpotEffect {
    SpotEffect {
        trigger_trap: has_trap,
        terrain_switch: terrain_changed,
        pool_effect: is_new_spot, // 호출 측에서 pooleffects() 별도 호출
        pickup: is_new_spot && !trap_is_pit, // 구덩이면 함정 먼저
        check_special_room: is_new_spot,
        sink_fall: is_sink && is_levitating,
        ice_warning: if is_ice && ice_time_left > 0 {
            if ice_time_left < 5 {
                Some("The ice, is gonna BREAK!")
            } else if ice_time_left < 10 {
                Some("You feel the ice shift beneath you!")
            } else if ice_time_left < 15 {
                Some("The ice seems very soft and slushy.")
            } else {
                None
            }
        } else {
            None
        },
        ceiling_fall: has_ceiling_monster,
    }
}

// ---------------------------------------------------------------------------
// 반려동물 교환 (원본: hack.c:1782-1893)
// ---------------------------------------------------------------------------

/// [v2.4.0] 반려동물 교환 결과
#[derive(Debug, Clone)]
pub struct PetSwapResult {
    /// 교환 성공 여부
    pub swapped: bool,
    /// 반려동물이 야생화 여부
    pub went_wild: bool,
    /// 메시지
    pub message: String,
}

/// [v2.4.0] 반려동물 교환 판정 (원본: domove_core 내 safepet 교환)
pub fn try_pet_swap(
    pet_is_trapped: bool,
    pet_tameness: i32,
    pet_can_diagonal: bool,
    is_diagonal_move: bool,
    pet_fits: bool,
    rng_val: i32,
) -> PetSwapResult {
    if pet_is_trapped {
        if rng_val % pet_tameness.max(1) == 0 {
            return PetSwapResult {
                swapped: false,
                went_wild: true,
                message: "Your pet goes wild!".into(),
            };
        }
        return PetSwapResult {
            swapped: false,
            went_wild: false,
            message: "Your pet yelps!".into(),
        };
    }
    if is_diagonal_move && !pet_can_diagonal {
        return PetSwapResult {
            swapped: false,
            went_wild: false,
            message: "Your pet can't move diagonally.".into(),
        };
    }
    if !pet_fits {
        return PetSwapResult {
            swapped: false,
            went_wild: false,
            message: "Your pet won't fit through.".into(),
        };
    }
    PetSwapResult {
        swapped: true,
        went_wild: false,
        message: "You swap places with your pet.".into(),
    }
}

// ---------------------------------------------------------------------------
// 수중 이동 (원본: hack.c:1457-1478)
// ---------------------------------------------------------------------------

/// [v2.4.0] 수중 이동 판정 (원본: domove_core 내 water_friction)
pub fn water_move_check(
    is_underwater: bool,
    dest_is_pool: bool,
    encumbrance: i32,
    can_swim: bool,
) -> (bool, Option<&'static str>) {
    if !is_underwater {
        return (true, None);
    }
    if !dest_is_pool {
        let max_enc = if can_swim { MOD_ENCUMBER } else { SLT_ENCUMBER };
        if encumbrance > max_enc {
            return (
                false,
                Some("You are carrying too much to climb out of the water."),
            );
        }
    }
    (true, None)
}

/// [v2.4.0] 공중 레벨 이동 판정 (원본: hack.c:1404-1418)
pub fn air_level_move(rng_val: i32) -> Option<&'static str> {
    let msg = match rng_val % 3 {
        0 => "You tumble in place.",
        1 => "You can't control your movements very well.",
        _ => "It's hard to walk in thin air.",
    };
    Some(msg)
}

/// [v2.4.0] 방향 이름 (원본: directionname)
pub fn direction_name(dx: i32, dy: i32) -> &'static str {
    match (dx.signum(), dy.signum()) {
        (0, -1) => "north",
        (1, -1) => "northeast",
        (1, 0) => "east",
        (1, 1) => "southeast",
        (0, 1) => "south",
        (-1, 1) => "southwest",
        (-1, 0) => "west",
        (-1, -1) => "northwest",
        _ => "here",
    }
}

/// [v2.4.0] 과적 시 이동 불가 판정 (원본: hack.c:1385-1397)
pub fn movement_overloaded(
    encumbrance: i32,
    hp: i32,
    maxhp: i32,
    is_polyd: bool,
    is_air_level: bool,
) -> Option<&'static str> {
    if is_air_level {
        return None;
    }
    if encumbrance >= OVERLOADED {
        return Some("You collapse under your load.");
    }
    if encumbrance > SLT_ENCUMBER {
        let (h, mh) = if is_polyd { (hp, maxhp) } else { (hp, maxhp) };
        if h < 10 && h != mh {
            return Some("You don't have enough stamina to move.");
        }
    }
    None
}

// ---------------------------------------------------------------------------
// 테스트
// ---------------------------------------------------------------------------
#[cfg(test)]
mod hack_v240_tests {
    use super::*;

    #[test]
    fn test_trapmove_beartrap() {
        let r = trapmove(TrapType::BearTrap, 1, 1, 1, false, false, false, 0);
        assert!(r.escaped);
    }

    #[test]
    fn test_trapmove_web_sting() {
        let r = trapmove(TrapType::Web, 5, 0, 1, false, false, true, 0);
        assert!(r.escaped);
        assert_eq!(r.remaining_trap, 0);
    }

    #[test]
    fn test_trapmove_pit_adjacent() {
        let r = trapmove(TrapType::Pit, 3, 0, 1, true, false, false, 0);
        assert!(r.can_move);
    }

    #[test]
    fn test_ice_slip() {
        assert!(check_ice_slip(true, false, false, false, false, false, 0));
        assert!(!check_ice_slip(true, false, true, false, false, false, 0));
        assert!(!check_ice_slip(false, false, false, false, false, false, 0));
    }

    #[test]
    fn test_confused_dir() {
        let d = confused_direction(3);
        assert!(d.dx.abs() <= 1 && d.dy.abs() <= 1);
    }

    #[test]
    fn test_test_move_wall() {
        let r = test_move(
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            MoveMode::DoMove,
            true,
        );
        assert!(!r.can_move);
        assert!(r.message.is_some());
    }

    #[test]
    fn test_test_move_passwall() {
        let r = test_move(
            true,
            false,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            MoveMode::DoMove,
            true,
        );
        assert!(r.can_move);
    }

    #[test]
    fn test_travel_step_adjacent() {
        let step = find_travel_step(5, 5, 6, 5, &|_x, _y| true, 20, 20);
        assert!(step.found);
        assert_eq!(step.dx, 1);
        assert_eq!(step.dy, 0);
    }

    #[test]
    fn test_travel_step_blocked() {
        let step = find_travel_step(0, 0, 5, 5, &|_x, _y| false, 10, 10);
        assert!(!step.found);
    }

    #[test]
    fn test_in_room() {
        assert!(in_room(5, 5, 3, 3, 7, 7));
        assert!(!in_room(2, 5, 3, 3, 7, 7));
    }

    #[test]
    fn test_in_town() {
        let rooms = vec![(2, 2, 8, 8, 1)];
        assert!(in_town(5, 5, true, &rooms));
        assert!(!in_town(5, 5, false, &rooms));
    }

    #[test]
    fn test_pet_swap_trapped() {
        let r = try_pet_swap(true, 1, true, false, true, 0);
        assert!(r.went_wild);
    }

    #[test]
    fn test_pet_swap_diagonal() {
        let r = try_pet_swap(false, 5, false, true, true, 0);
        assert!(!r.swapped);
    }

    #[test]
    fn test_pet_swap_success() {
        let r = try_pet_swap(false, 5, true, false, true, 0);
        assert!(r.swapped);
    }

    #[test]
    fn test_direction_name() {
        assert_eq!(direction_name(1, 0), "east");
        assert_eq!(direction_name(-1, 1), "southwest");
        assert_eq!(direction_name(0, 0), "here");
    }

    #[test]
    fn test_movement_overloaded() {
        assert!(movement_overloaded(OVERLOADED, 10, 20, false, false).is_some());
        assert!(movement_overloaded(OVERLOADED, 10, 20, false, true).is_none());
    }

    #[test]
    fn test_spot_effects_ice() {
        let e = spot_effects(true, false, false, true, 3, false, false, false, false);
        assert!(e.ice_warning.unwrap().contains("BREAK"));
    }
}
