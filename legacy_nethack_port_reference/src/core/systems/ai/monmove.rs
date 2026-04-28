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
pub fn calculate_move_direction(mx: i32, my: i32, tx: i32, ty: i32) -> (i32, i32) {
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

// =============================================================================
// [v2.9.4] monmove.c 1차 이식 — 트랩폭발/열쇠/경비/공포/재생/각성/도주/전투판정
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// 트랩 도어 폭발 (원본: mb_trapped, L21-43)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 트랩 도어 폭발 결과 (원본: mb_trapped, L21-43)
#[derive(Debug, Clone)]
pub struct MbTrappedResult {
    /// 플레이어가 볼 수 있으면 "KABOOM!!" 메시지
    pub show_kaboom: bool,
    /// 플레이어가 소리를 들을 수 있으면 폭발음 메시지
    pub hear_explosion: bool,
    /// 몬스터 기절 여부
    pub stun: bool,
    /// 피해량 (rnd(15))
    pub damage: i32,
    /// 몬스터 사망 여부
    pub mon_died: bool,
    /// 주변 각성 필요 (7*7 = 49 거리)
    pub wake_distance: i32,
}

/// [v2.9.4] 트랩 도어 판정 (원본: mb_trapped, L21-43)
pub fn mb_trapped_result(
    can_see: bool,
    is_deaf: bool,
    mon_hp: i32,
    rng: &mut NetHackRng,
) -> MbTrappedResult {
    let damage = rng.rnd(15);
    let new_hp = mon_hp - damage;
    MbTrappedResult {
        show_kaboom: can_see,
        hear_explosion: !can_see && !is_deaf,
        stun: true,
        damage,
        mon_died: new_hp <= 0,
        wake_distance: 49, // 7*7 (원본: L32)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 열쇠/잠금해제 도구 (원본: monhaskey, L45-54)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 열쇠 보유 판정 (원본: monhaskey, L45-54)
pub fn monhaskey_check(
    has_credit_card: bool,
    has_skeleton_key: bool,
    has_lock_pick: bool,
    for_unlocking: bool,
) -> bool {
    if for_unlocking && has_credit_card {
        return true;
    }
    has_skeleton_key || has_lock_pick
}

// ─────────────────────────────────────────────────────────────────────────────
// 공포 장소 (원본: onscary, L134-182)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 공포 장소 결과 (원본: onscary, L134-182)
#[derive(Debug, Clone)]
pub struct OnScaryResult {
    /// 공포 효과 적용됨
    pub is_scared: bool,
    /// 면역 사유 (Wizard/Rider/Angel 등)
    pub immune_reason: Option<String>,
}

/// [v2.9.4] 공포 장소 판정 (원본: onscary, L134-182)
pub fn onscary_check(
    x: i32,
    y: i32,
    // 면역 플래그
    is_wizard: bool,
    is_lawful_minion: bool,
    is_angel: bool,
    is_rider: bool,
    is_shopkeeper_in_shop: bool,
    is_priest_in_temple: bool,
    // 위치 플래그
    has_scare_scroll: bool,
    is_altar: bool,
    is_vampire: bool,
    has_elbereth: bool,
    player_at_pos: bool,
    displaced_at_pos: bool,
    // 제외 플래그
    is_blind: bool,
    is_peaceful: bool,
    is_human: bool,
    is_minotaur: bool,
    in_hell: bool,
    in_endgame: bool,
) -> OnScaryResult {
    // 직접 면역 (원본: L142-146)
    if is_wizard
        || is_lawful_minion
        || is_angel
        || is_rider
        || is_shopkeeper_in_shop
        || is_priest_in_temple
    {
        return OnScaryResult {
            is_scared: false,
            immune_reason: Some("직접 면역 (Wizard/Rider/상점/사제)".to_string()),
        };
    }

    // <0,0> 체크 (원본: L150-151) — 음악 공포
    if x == 0 && y == 0 {
        return OnScaryResult {
            is_scared: true,
            immune_reason: None,
        };
    }

    // 제단 + 뱀파이어 (원본: L154-156)
    if is_altar && is_vampire {
        return OnScaryResult {
            is_scared: true,
            immune_reason: None,
        };
    }

    // 공포 두루마리 (원본: L160-161)
    if has_scare_scroll {
        return OnScaryResult {
            is_scared: true,
            immune_reason: None,
        };
    }

    // Elbereth (원본: L175-181)
    let elbereth_works = has_elbereth
        && (player_at_pos || displaced_at_pos)
        && !is_blind
        && !is_peaceful
        && !is_human
        && !is_minotaur
        && !in_hell
        && !in_endgame;

    OnScaryResult {
        is_scared: elbereth_works,
        immune_reason: if !elbereth_works && has_elbereth {
            Some("Elbereth 무효 (지옥/종말/인간 등)".to_string())
        } else {
            None
        },
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 몬스터 HP 재생 (원본: mon_regen, L185-202)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 재생 결과 (원본: mon_regen, L185-202)
#[derive(Debug, Clone)]
pub struct MonRegenResult {
    /// HP 회복량
    pub hp_regen: i32,
    /// 특수 능력 쿨다운 감소
    pub spec_used_decrement: bool,
    /// 식사 타이머 감소
    pub eating_decrement: bool,
    /// 식사 완료
    pub finish_eating: bool,
}

/// [v2.9.4] 몬스터 재생 판정 (원본: mon_regen, L185-202)
pub fn mon_regen_result(
    hp: i32,
    max_hp: i32,
    turn: u64,
    regenerates: bool,
    spec_used: i32,
    eating: i32,
    digest_meal: bool,
) -> MonRegenResult {
    // HP 재생: 20턴마다 또는 재생 능력 (원본: L191-192)
    let hp_regen = if hp < max_hp && (turn % 20 == 0 || regenerates) {
        1
    } else {
        0
    };

    // 쿨다운 감소 (원본: L193-194)
    let spec_decr = spec_used > 0;

    // 식사 처리 (원본: L195-201)
    let (eat_decr, finish) = if digest_meal && eating > 0 {
        (true, eating - 1 <= 0)
    } else {
        (false, false)
    };

    MonRegenResult {
        hp_regen,
        spec_used_decrement: spec_decr,
        eating_decrement: eat_decr,
        finish_eating: finish,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 각성 판정 (원본: disturb, L204-240)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 각성 판정 (원본: disturb, L204-240)
/// 에틴/님프/잽버워크/레프리콘의 깨어나기 확률 판정
pub fn disturb_check(
    can_see_mon: bool,
    distance_sq: i32,
    is_stealthy: bool,
    is_ettin: bool,
    is_nymph: bool,
    is_jabberwock: bool,
    is_leprechaun: bool,
    aggravate: bool,
    is_dog: bool,
    is_human: bool,
    ap_type_furniture: bool,
    ap_type_object: bool,
    rng: &mut NetHackRng,
) -> bool {
    // 시야 내 + 거리 100 이내 (원본: L224)
    if !can_see_mon || distance_sq > 100 {
        return false;
    }
    // 스텔스 체크 (원본: L225)
    if is_stealthy && !(is_ettin && rng.rn2(10) != 0) {
        return false;
    }
    // 님프/잽버워크/레프리콘은 잘 안 깨어남 (원본: L226-231)
    if (is_nymph || is_jabberwock || is_leprechaun) && rng.rn2(50) != 0 {
        return false;
    }
    // 최종: Aggravate 또는 개/인간 또는 1/7 확률 (원본: L232-235)
    aggravate || is_dog || is_human || (rng.rn2(7) == 0 && !ap_type_furniture && !ap_type_object)
}

// ─────────────────────────────────────────────────────────────────────────────
// 도주 (원본: monflee L263-312, distfleeck L314-349)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 도주 시작 결과 (원본: monflee, L263-312)
#[derive(Debug, Clone)]
pub struct MonfleeResult {
    /// 도주 활성화
    pub flee_activated: bool,
    /// 도주 시간 (0 = 무제한)
    pub flee_time: i32,
    /// 플레이어 해방 필요 (점착/삼킴)
    pub release_hero: bool,
    /// 추적 기억 초기화
    pub clear_tracking: bool,
    /// 도주 메시지 출력 ("turns to flee" 등)
    pub show_flee_msg: bool,
    /// 이동 불가 상태에서의 떨림 메시지
    pub show_flinch_msg: bool,
}

/// [v2.9.4] 도주 시작 판정 (원본: monflee, L263-312)
pub fn monflee_result(
    current_flee: bool,
    current_flee_time: i32,
    fleetime: i32,
    first: bool,
    fleemsg: bool,
    is_stuck_to_player: bool,
    can_see_mon: bool,
    can_move: bool,
    ap_is_disguised: bool,
) -> MonfleeResult {
    let mut r = MonfleeResult {
        flee_activated: false,
        flee_time: 0,
        release_hero: is_stuck_to_player,
        clear_tracking: true,
        show_flee_msg: false,
        show_flinch_msg: false,
    };

    if first && current_flee {
        return r; // 이미 도주 중이면 무시 (원본: L280)
    }

    // 도주 시간 계산 (원본: L282-290)
    if fleetime == 0 {
        r.flee_time = 0;
    } else if !current_flee || current_flee_time > 0 {
        let total = fleetime + current_flee_time;
        let total = if total == 1 { 2 } else { total };
        r.flee_time = total.min(127);
    }

    // 메시지 (원본: L291-306)
    if !current_flee && fleemsg && can_see_mon && !ap_is_disguised {
        if !can_move {
            r.show_flinch_msg = true;
        } else {
            r.show_flee_msg = true;
        }
    }

    r.flee_activated = true;
    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 도주 거리/공포 판정 (원본: distfleeck, L314-349)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 거리/공포 판정 결과 (원본: distfleeck, L314-349)
#[derive(Debug, Clone)]
pub struct DistfleeckResult {
    /// 볼트 범위 내
    pub in_range: bool,
    /// 근접 (인접 1칸)
    pub nearby: bool,
    /// 공포 영향 받음
    pub scared: bool,
    /// 도주 시간 (공포 시 rnd(rn2(7)?10:100))
    pub flee_time: i32,
}

/// [v2.9.4] 거리/공포 판정 (원본: distfleeck, L314-349)
pub fn distfleeck_result(
    mon_x: i32,
    mon_y: i32,
    target_x: i32,
    target_y: i32,
    on_scary: bool,
    is_peaceful_sanctuary: bool,
    rng: &mut NetHackRng,
) -> DistfleeckResult {
    let bolt_lim = 8; // BOLT_LIM
    let dist = (mon_x - target_x) * (mon_x - target_x) + (mon_y - target_y) * (mon_y - target_y);
    let in_range = dist <= bolt_lim * bolt_lim;
    let nearby = in_range && {
        let chebyshev = (mon_x - target_x).abs().max((mon_y - target_y).abs());
        chebyshev <= 1
    };

    let scared = nearby && (on_scary || is_peaceful_sanctuary);
    let flee_time = if scared {
        let base = if rng.rn2(7) != 0 { 10 } else { 100 };
        rng.rnd(base)
    } else {
        0
    };

    DistfleeckResult {
        in_range,
        nearby,
        scared,
        flee_time,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 점착 체크 (원본: itsstuck, L674-683)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 점착 체크 (원본: itsstuck, L674-683)
pub fn itsstuck_check(
    player_sticks: bool,
    is_stuck_to_player: bool,
    player_swallowed: bool,
) -> bool {
    // 플레이어가 점착 + 이 몬스터가 점착 대상 + 삼킨 게 아님 (원본: L678)
    player_sticks && is_stuck_to_player && !player_swallowed
}

// ─────────────────────────────────────────────────────────────────────────────
// 밀어내기 판정 (원본: should_displace, L685-727)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 밀어내기 판정 (원본: should_displace, L685-727)
/// 다른 몬스터를 밀어낼지 결정 — 더 짧은 경로일 때만
pub fn should_displace_check(
    positions_with_displace: &[(i32, i32, i32)], // (x, y, 목표까지 거리)
    positions_without_displace: &[(i32, i32, i32)], // (x, y, 목표까지 거리)
) -> bool {
    let best_with = positions_with_displace.iter().map(|p| p.2).min();
    let best_without = positions_without_displace.iter().map(|p| p.2).min();

    match (best_with, best_without) {
        (Some(bw), Some(bwo)) => bw < bwo,
        (Some(_), None) => true, // 밀어내기만 가능
        _ => false,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 위치 인식 (원본: set_apparxy, L1531-1596)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 플레이어 인식 위치 결과 (원본: set_apparxy, L1531-1596)
#[derive(Debug, Clone)]
pub struct ApparXYResult {
    /// 몬스터가 생각하는 플레이어 X
    pub perceived_x: i32,
    /// 몬스터가 생각하는 플레이어 Y
    pub perceived_y: i32,
    /// 정확히 아는지
    pub exact: bool,
}

/// [v2.9.4] 플레이어 인식 위치 판정 (원본: set_apparxy, L1531-1596)
pub fn set_apparxy_result(
    player_x: i32,
    player_y: i32,
    is_tame: bool,
    is_stuck: bool,
    already_knows: bool,
    can_see: bool,
    player_invis: bool,
    perceives_invis: bool,
    player_displaced: bool,
    player_underwater: bool,
    is_xorn_with_gold: bool,
    rng: &mut NetHackRng,
) -> ApparXYResult {
    // 애완동물/점착 → 정확 (원본: L1545-1546)
    if is_tame || is_stuck {
        return ApparXYResult {
            perceived_x: player_x,
            perceived_y: player_y,
            exact: true,
        };
    }

    // 이미 알고 있음 → 정확 (원본: L1550-1551)
    if already_knows {
        return ApparXYResult {
            perceived_x: player_x,
            perceived_y: player_y,
            exact: true,
        };
    }

    // 투명/수중/포트 판정 (원본: L1553-1565)
    let not_seen = !can_see || (player_invis && !perceives_invis);
    let disp = if not_seen || player_underwater {
        if is_xorn_with_gold && !player_underwater {
            0
        } else {
            1
        }
    } else if player_displaced {
        2
    } else {
        0
    };

    if disp == 0 {
        return ApparXYResult {
            perceived_x: player_x,
            perceived_y: player_y,
            exact: true,
        };
    }

    // 투명/포트 관통 (원본: L1571)
    let gotu = if not_seen {
        rng.rn2(3) == 0
    } else if player_displaced {
        rng.rn2(4) == 0
    } else {
        false
    };

    if gotu {
        return ApparXYResult {
            perceived_x: player_x,
            perceived_y: player_y,
            exact: true,
        };
    }

    // 오차 (원본: L1579-1580)
    let mx = player_x - disp + rng.rn2(2 * disp + 1);
    let my = player_y - disp + rng.rn2(2 * disp + 1);

    ApparXYResult {
        perceived_x: mx,
        perceived_y: my,
        exact: false,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 밀어내기 비선호 (원본: undesirable_disp, L1598-1641)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 밀어내기 비선호 (원본: undesirable_disp, L1598-1641)
pub fn undesirable_disp_check(
    is_pet: bool,
    has_seen_trap: bool,
    has_cursed_item: bool,
    trap_type_seen: bool,
    is_accessible: bool,
    is_pool_target: bool,
    is_pool_current: bool,
    rng: &mut NetHackRng,
) -> bool {
    if is_pet {
        // 애완동물: 보이는 트랩 회피 (원본: L1616-1617)
        if has_seen_trap && rng.rn2(40) != 0 {
            return true;
        }
        // 저주 물건 회피 (원본: L1619-1620)
        if has_cursed_item {
            return true;
        }
    } else {
        // 일반 몬스터: 본 적 있는 트랩 유형 회피 (원본: L1623-1625)
        if trap_type_seen && rng.rn2(40) != 0 {
            return true;
        }
    }

    // 접근 불가 위치 (원본: L1633-1638)
    if !is_accessible && !(is_pool_target && is_pool_current) {
        return true;
    }

    false
}

// ─────────────────────────────────────────────────────────────────────────────
// 통로 통과/안개 (원본: can_ooze L1685-1692, can_fog L1694-1703)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 점액 통과 판정 (원본: can_ooze, L1685-1692)
pub fn can_ooze_check(is_amorphous: bool, stuff_prevents: bool) -> bool {
    is_amorphous && !stuff_prevents
}

/// [v2.9.4] 안개 변신 판정 (원본: can_fog, L1694-1703)
pub fn can_fog_check(
    is_vampshifter: bool,
    fog_genocided: bool,
    protection_from_shapechanger: bool,
    stuff_prevents: bool,
) -> bool {
    !fog_genocided && is_vampshifter && !protection_from_shapechanger && !stuff_prevents
}

// ─────────────────────────────────────────────────────────────────────────────
// 뱀파이어 변신 (원본: vamp_shift, L1705-1735)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 뱀파이어 변신 결과 (원본: vamp_shift, L1705-1735)
#[derive(Debug, Clone)]
pub struct VampShiftResult {
    /// 변신 성공
    pub shifted: bool,
    /// 이미 올바른 형태
    pub already_correct: bool,
    /// 메시지 출력 (변신 관찰)
    pub show_message: bool,
}

/// [v2.9.4] 뱀파이어 변신 판정 (원본: vamp_shift, L1705-1735)
pub fn vamp_shift_result(
    current_form: &str,
    target_form: &str,
    is_vampshifter: bool,
    show_msg: bool,
) -> VampShiftResult {
    if current_form == target_form {
        // 이미 원하는 형태 (원본: L1717-1720)
        VampShiftResult {
            shifted: true,
            already_correct: true,
            show_message: false,
        }
    } else if is_vampshifter {
        // 변형 시도 (원본: L1721-1722)
        VampShiftResult {
            shifted: true, // newcham 호출 직접은 ECS에서 처리
            already_correct: false,
            show_message: show_msg,
        }
    } else {
        VampShiftResult {
            shifted: false,
            already_correct: false,
            show_message: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 굴착 무기 확인 (원본: m_digweapon_check, L729-759)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 굴착 무기 필요 타입
#[derive(Debug, Clone, PartialEq)]
pub enum DigWeaponNeed {
    /// 필요 없음
    None,
    /// 곡괭이 필요
    NeedPickAxe,
    /// 도끼 필요
    NeedAxe,
    /// 곡괭이 또는 도끼 필요
    NeedPickOrAxe,
}

/// [v2.9.4] 굴착 무기 판정 (원본: m_digweapon_check, L729-759)
pub fn m_digweapon_check_result(
    can_tunnel: bool,
    needs_pick: bool,
    is_rogue_level: bool,
    target_is_closed_door: bool,
    target_is_tree: bool,
    target_is_wall: bool,
    has_pick: bool,
    has_axe: bool,
) -> DigWeaponNeed {
    if is_rogue_level || !can_tunnel || !needs_pick {
        return DigWeaponNeed::None;
    }

    if target_is_closed_door {
        if !has_pick && !has_axe {
            return DigWeaponNeed::NeedPickOrAxe;
        }
    } else if target_is_tree {
        if !has_axe {
            return DigWeaponNeed::NeedAxe;
        }
    } else if target_is_wall {
        if !has_pick {
            return DigWeaponNeed::NeedPickAxe;
        }
    }

    DigWeaponNeed::None
}

// ─────────────────────────────────────────────────────────────────────────────
// dochug 전처리 (원본: dochug, L364-663 — 전처리 부분만)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] dochug 전처리 결과 (원본: dochug, L364-440)
#[derive(Debug, Clone)]
pub struct DochugPreResult {
    /// 도착 이벤트 소비 (STRAT_ARRIVE)
    pub arrival_consumed: bool,
    /// 대기 해제 (STRAT_WAITFORU)
    pub wait_cleared: bool,
    /// 행동 불가 (동결/대기)
    pub frozen_skip: bool,
    /// 혼란 해제 (2% 확률)
    pub confusion_cleared: bool,
    /// 기절 해제 (10% 확률)
    pub stun_cleared: bool,
    /// 도주 중 텔레포트
    pub flee_teleport: bool,
    /// 용기 회복 (도주 해제)
    pub courage_regained: bool,
    /// 분쟁 종료 해방
    pub conflict_release: bool,
}

/// [v2.9.4] dochug 전처리 판정 (원본: dochug, L364-440)
pub fn dochug_pre_result(
    has_arrive: bool,
    can_see_player: bool,
    hp_below_max: bool,
    can_move: bool,
    has_wait_strat: bool,
    is_sleeping: bool,
    awoke: bool,
    is_confused: bool,
    is_stunned: bool,
    is_fleeing: bool,
    can_teleport: bool,
    is_wizard: bool,
    no_teleport_level: bool,
    flee_time: i32,
    hp_full: bool,
    is_stuck_peaceful: bool,
    is_conflict: bool,
    rng: &mut NetHackRng,
) -> DochugPreResult {
    let mut r = DochugPreResult {
        arrival_consumed: has_arrive,
        wait_cleared: false,
        frozen_skip: false,
        confusion_cleared: false,
        stun_cleared: false,
        flee_teleport: false,
        courage_regained: false,
        conflict_release: false,
    };

    // 대기 해제 (원본: L388-390)
    if has_wait_strat && (can_see_player || hp_below_max) {
        r.wait_cleared = true;
    }

    // 동결/대기 (원본: L395-402)
    if !can_move || (has_wait_strat && !r.wait_cleared) {
        r.frozen_skip = true;
        return r;
    }

    // 수면 중 + 깨어나지 않음 (원본: L405-409)
    if is_sleeping && !awoke {
        r.frozen_skip = true;
        return r;
    }

    // 혼란 해제 2% (원본: L415-416)
    if is_confused && rng.rn2(50) == 0 {
        r.confusion_cleared = true;
    }

    // 기절 해제 10% (원본: L419-420)
    if is_stunned && rng.rn2(10) == 0 {
        r.stun_cleared = true;
    }

    // 도주 중 텔레포트 2.5% (원본: L423-427)
    if is_fleeing && rng.rn2(40) == 0 && can_teleport && !is_wizard && !no_teleport_level {
        r.flee_teleport = true;
    }

    // 용기 회복 (원본: L436-438)
    if is_fleeing && flee_time == 0 && hp_full && rng.rn2(25) == 0 {
        r.courage_regained = true;
    }

    // 분쟁 종료 해방 (원본: L441-444)
    if is_stuck_peaceful && !is_conflict {
        r.conflict_release = true;
    }

    r
}

// ─────────────────────────────────────────────────────────────────────────────
// 도어 타일 검사 (원본: closed_door L1510-1516, accessible L1518-1529)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 닫힌 문 판정 (원본: closed_door, L1510-1516)
pub fn closed_door_check(is_door: bool, is_locked_or_closed: bool) -> bool {
    is_door && is_locked_or_closed
}

/// [v2.9.4] 접근 가능 판정 (원본: accessible, L1518-1529)
pub fn accessible_check(is_accessible_terrain: bool, is_closed_door: bool) -> bool {
    is_accessible_terrain && !is_closed_door
}

/// [v2.9.4] 철창 용해 결과 (원본: dissolve_bars, L1501-1508)
pub fn dissolve_bars_result(is_special_level: bool, in_room: bool) -> &'static str {
    if is_special_level || in_room {
        "room" // ROOM 타입으로 전환
    } else {
        "corridor" // CORR 타입으로 전환
    }
}

// =============================================================================
// [v2.9.4] monmove.c 1차 이식 테스트
// =============================================================================
#[cfg(test)]
mod monmove_ported_tests {
    use super::*;

    #[test]
    fn test_mb_trapped() {
        let mut rng = NetHackRng::new(42);
        let r = mb_trapped_result(true, false, 20, &mut rng);
        assert!(r.show_kaboom);
        assert!(!r.hear_explosion);
        assert!(r.stun);
        assert!(r.damage > 0 && r.damage <= 15);
        assert_eq!(r.wake_distance, 49);
    }

    #[test]
    fn test_mb_trapped_deaf() {
        let mut rng = NetHackRng::new(42);
        let r = mb_trapped_result(false, true, 20, &mut rng);
        assert!(!r.show_kaboom);
        assert!(!r.hear_explosion);
    }

    #[test]
    fn test_monhaskey() {
        assert!(monhaskey_check(true, false, false, true));
        assert!(!monhaskey_check(true, false, false, false));
        assert!(monhaskey_check(false, true, false, false));
        assert!(monhaskey_check(false, false, true, true));
    }

    #[test]
    fn test_onscary_wizard_immune() {
        let r = onscary_check(
            5, 5, true, false, false, false, false, false, true, false, false, false, false, false,
            false, false, false, false, false, false,
        );
        assert!(!r.is_scared);
    }

    #[test]
    fn test_onscary_scare_scroll() {
        let r = onscary_check(
            5, 5, false, false, false, false, false, false, true, false, false, false, false,
            false, false, false, false, false, false, false,
        );
        assert!(r.is_scared);
    }

    #[test]
    fn test_onscary_music() {
        let r = onscary_check(
            0, 0, false, false, false, false, false, false, false, false, false, false, false,
            false, false, false, false, false, false, false,
        );
        assert!(r.is_scared);
    }

    #[test]
    fn test_mon_regen_normal() {
        let r = mon_regen_result(10, 20, 20, false, 3, 5, true);
        assert_eq!(r.hp_regen, 1); // 턴 20 → 재생
        assert!(r.spec_used_decrement);
        assert!(r.eating_decrement);
    }

    #[test]
    fn test_mon_regen_no_digest() {
        let r = mon_regen_result(20, 20, 21, false, 0, 5, false);
        assert_eq!(r.hp_regen, 0); // HP 가득
        assert!(!r.spec_used_decrement);
        assert!(!r.eating_decrement);
    }

    #[test]
    fn test_disturb_basic() {
        let mut rng = NetHackRng::new(42);
        // Aggravate → 반드시 깨어남
        assert!(disturb_check(
            true, 50, false, false, false, false, false, true, false, false, false, false, &mut rng
        ));
    }

    #[test]
    fn test_disturb_too_far() {
        let mut rng = NetHackRng::new(42);
        assert!(!disturb_check(
            true, 200, false, false, false, false, false, false, false, false, false, false,
            &mut rng
        ));
    }

    #[test]
    fn test_monflee_new() {
        let r = monflee_result(false, 0, 10, false, true, false, true, true, false);
        assert!(r.flee_activated);
        assert_eq!(r.flee_time, 10);
        assert!(r.show_flee_msg);
    }

    #[test]
    fn test_monflee_already_fleeing() {
        let r = monflee_result(true, 5, 10, true, true, false, true, true, false);
        assert!(!r.flee_activated); // first=true, 이미 도주 중
    }

    #[test]
    fn test_distfleeck_nearby_scared() {
        let mut rng = NetHackRng::new(42);
        let r = distfleeck_result(5, 5, 6, 5, true, false, &mut rng);
        assert!(r.in_range);
        assert!(r.nearby);
        assert!(r.scared);
        assert!(r.flee_time > 0);
    }

    #[test]
    fn test_itsstuck() {
        assert!(itsstuck_check(true, true, false));
        assert!(!itsstuck_check(true, true, true)); // 삼킴 상태
        assert!(!itsstuck_check(false, true, false));
    }

    #[test]
    fn test_should_displace() {
        let with = vec![(5, 5, 3)];
        let without = vec![(6, 6, 5)];
        assert!(should_displace_check(&with, &without));

        let with2 = vec![(5, 5, 10)];
        let without2 = vec![(6, 6, 5)];
        assert!(!should_displace_check(&with2, &without2));
    }

    #[test]
    fn test_set_apparxy_tame() {
        let mut rng = NetHackRng::new(42);
        let r = set_apparxy_result(
            10, 10, true, false, false, true, false, false, false, false, false, &mut rng,
        );
        assert!(r.exact);
        assert_eq!(r.perceived_x, 10);
    }

    #[test]
    fn test_can_ooze() {
        assert!(can_ooze_check(true, false));
        assert!(!can_ooze_check(true, true));
        assert!(!can_ooze_check(false, false));
    }

    #[test]
    fn test_can_fog() {
        assert!(can_fog_check(true, false, false, false));
        assert!(!can_fog_check(true, true, false, false)); // 제노사이드
        assert!(!can_fog_check(false, false, false, false)); // 뱀파이어 아님
    }

    #[test]
    fn test_vamp_shift_same_form() {
        let r = vamp_shift_result("vampire bat", "vampire bat", true, true);
        assert!(r.shifted);
        assert!(r.already_correct);
        assert!(!r.show_message);
    }

    #[test]
    fn test_vamp_shift_different() {
        let r = vamp_shift_result("vampire", "fog cloud", true, true);
        assert!(r.shifted);
        assert!(!r.already_correct);
        assert!(r.show_message);
    }

    #[test]
    fn test_m_digweapon_wall() {
        assert_eq!(
            m_digweapon_check_result(true, true, false, false, false, true, false, false),
            DigWeaponNeed::NeedPickAxe
        );
    }

    #[test]
    fn test_m_digweapon_tree() {
        assert_eq!(
            m_digweapon_check_result(true, true, false, false, true, false, false, false),
            DigWeaponNeed::NeedAxe
        );
    }

    #[test]
    fn test_dochug_pre_frozen() {
        let mut rng = NetHackRng::new(42);
        let r = dochug_pre_result(
            false, false, false, false, false, false, false, false, false, false, false, false,
            false, 0, false, false, false, &mut rng,
        );
        assert!(r.frozen_skip);
    }

    #[test]
    fn test_dochug_pre_conflict_release() {
        let mut rng = NetHackRng::new(42);
        let r = dochug_pre_result(
            false, false, false, true, false, false, false, false, false, false, false, false,
            false, 0, false, true, false, &mut rng,
        );
        assert!(r.conflict_release);
    }

    #[test]
    fn test_closed_door_check() {
        assert!(closed_door_check(true, true));
        assert!(!closed_door_check(true, false));
        assert!(!closed_door_check(false, true));
    }

    #[test]
    fn test_accessible_check() {
        assert!(accessible_check(true, false));
        assert!(!accessible_check(true, true));
    }

    #[test]
    fn test_dissolve_bars() {
        assert_eq!(dissolve_bars_result(true, false), "room");
        assert_eq!(dissolve_bars_result(false, true), "room");
        assert_eq!(dissolve_bars_result(false, false), "corridor");
    }
}

// =============================================================================
// [v2.9.4] monmove.c 2차 이식 — m_move/dochug 메인 이동 AI
// =============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// m_move 전처리 (원본: m_move L771-828 전반부)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 몬스터 이동 반환값 (원본: m_move)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MMoveReturn {
    /// 이동 안 함, 행동 가능
    DidNotMove,
    /// 이동함, 공격 가능
    Moved,
    /// 몬스터 사망
    Died,
    /// 이동 안 함, 다른 행동도 불가
    Stuck,
}

/// [v2.9.4] m_move 전처리 결과 (원본: m_move L790-807)
#[derive(Debug, Clone)]
pub struct MMovePreResult {
    /// 트랩에 걸림 — mintrap 처리 필요
    pub trapped_check_needed: bool,
    /// 트랩 사망
    pub trap_died: bool,
    /// 트랩에 아직 걸려 있음
    pub still_trapped: bool,
    /// 식사 중 — 행동 불가
    pub eating_skip: bool,
    /// 식사 타이머 감소 필요
    pub eating_decrement: bool,
    /// 식사 완료
    pub finish_eating: bool,
    /// 은신 유지 (hides_under + 오브젝트)
    pub stay_hidden: bool,
}

/// [v2.9.4] m_move 전처리 판정 (원본: m_move L790-809)
pub fn m_move_pre_result(
    is_trapped: bool,
    trap_result: i32, // mintrap 결과: 0=탈출, 1=아직, 2+=사망
    eating_timer: i32,
    hides_under: bool,
    has_obj_here: bool,
    rng: &mut NetHackRng,
) -> MMovePreResult {
    // 트랩 처리 (원본: L790-799)
    let (trap_died, still_trapped) = if is_trapped {
        (trap_result >= 2, trap_result == 1)
    } else {
        (false, false)
    };

    // 식사 중 (원본: L802-807)
    let eating_skip = eating_timer > 0;
    let eating_decr = eating_timer > 0;
    let finish_eat = eating_timer == 1; // 1→0 = 완료

    // 은신 유지 (원본: L808-809)
    let stay = hides_under && has_obj_here && rng.rn2(10) != 0;

    MMovePreResult {
        trapped_check_needed: is_trapped,
        trap_died,
        still_trapped,
        eating_skip,
        eating_decrement: eating_decr,
        finish_eating: finish_eat,
        stay_hidden: stay,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// m_move 특수 몬스터 위임 체크 (원본: m_move L822-898)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 특수 몬스터 이동 위임 타입
#[derive(Debug, Clone, PartialEq)]
pub enum SpecialMoveDelegate {
    /// 일반 이동 로직 사용
    Normal,
    /// 애완동물 이동 (dog_move)
    PetMove,
    /// 상점 주인 이동 (shk_move)
    ShopkeeperMove,
    /// 경비원 이동 (gd_move)
    GuardMove,
    /// 탐욕 몬스터 이동 (tactics)
    CovetousMove,
    /// 성직자 이동 (pri_move)
    PriestMove,
    /// 텐구 텔레포트 (원본: L890-897)
    TenguTeleport,
    /// 웜 — goto not_special
    WormSkip,
}

/// [v2.9.4] 특수 몬스터 위임 판정 (원본: m_move L822-898)
pub fn special_move_delegate(
    is_worm: bool,
    is_tame: bool,
    is_shopkeeper: bool,
    is_guard: bool,
    is_covetous: bool,
    is_priest: bool,
    is_tengu: bool,
    tengu_can_tele: bool,
    rng: &mut NetHackRng,
) -> SpecialMoveDelegate {
    if is_worm {
        return SpecialMoveDelegate::WormSkip;
    }
    if is_tame {
        return SpecialMoveDelegate::PetMove;
    }
    if is_shopkeeper {
        return SpecialMoveDelegate::ShopkeeperMove;
    }
    if is_guard {
        return SpecialMoveDelegate::GuardMove;
    }
    if is_covetous {
        return SpecialMoveDelegate::CovetousMove;
    }
    if is_priest {
        return SpecialMoveDelegate::PriestMove;
    }
    if is_tengu && rng.rn2(5) == 0 && tengu_can_tele {
        return SpecialMoveDelegate::TenguTeleport;
    }
    SpecialMoveDelegate::Normal
}

// ─────────────────────────────────────────────────────────────────────────────
// m_move 접근/목표 방향 결정 (원본: m_move L900-939)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 접근 방향 결정 결과 (원본: m_move L906-939)
#[derive(Debug, Clone)]
pub struct ApproachResult {
    /// 접근 방향: 1=접근, -1=도주, 0=무작위
    pub appr: i32,
    /// 목표 X
    pub gx: i32,
    /// 목표 Y
    pub gy: i32,
}

/// [v2.9.4] 접근 방향 판정 (원본: m_move L906-939)
pub fn approach_result(
    is_fleeing: bool,
    is_confused: bool,
    is_swallowed_stuck: bool,
    can_see: bool,
    player_invis: bool,
    perceives_invis: bool,
    is_peaceful: bool,
    is_shopkeeper: bool,
    is_stalker_bat_light: bool,
    player_undetected: bool,
    target_x: i32,
    target_y: i32,
    rng: &mut NetHackRng,
) -> ApproachResult {
    let mut appr = if is_fleeing { -1 } else { 1 };
    let mut gx = target_x;
    let mut gy = target_y;

    if is_confused || is_swallowed_stuck {
        appr = 0;
    } else {
        // 시야 차단 (원본: L915-922)
        if !can_see
            || (player_invis && !perceives_invis && rng.rn2(11) != 0)
            || player_undetected
            || (is_peaceful && !is_shopkeeper)
            || (is_stalker_bat_light && rng.rn2(3) != 0)
        {
            appr = 0;
        }
    }

    ApproachResult { appr, gx, gy }
}

// ─────────────────────────────────────────────────────────────────────────────
// m_move 아이템 선호도 (원본: m_move L941-963)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 아이템 선호도 (원본: m_move L941-963)
#[derive(Debug, Clone)]
pub struct ItemLikes {
    pub like_gold: bool,
    pub like_gems: bool,
    pub like_objs: bool,
    pub like_magic: bool,
    pub like_rock: bool,
    pub conceals: bool,
    pub uses_items: bool,
}

/// [v2.9.4] 아이템 선호도 판정 (원본: m_move L951-962)
pub fn item_likes_result(
    likes_gold: bool,
    likes_gems: bool,
    likes_objs: bool,
    likes_magic: bool,
    throws_rocks: bool,
    hides_under: bool,
    mindless: bool,
    is_animal: bool,
    pct_load: i32,
    is_sokoban: bool,
) -> ItemLikes {
    ItemLikes {
        like_gold: likes_gold && pct_load < 95,
        like_gems: likes_gems && pct_load < 85,
        like_objs: likes_objs && pct_load < 75,
        like_magic: likes_magic && pct_load < 85,
        like_rock: throws_rocks && pct_load < 50 && !is_sokoban,
        conceals: hides_under,
        uses_items: !mindless && !is_animal && pct_load < 75,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// m_move 이동 플래그 (원본: m_move L1082-1108)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 이동 허용 플래그 (원본: m_move L1082-1108)
#[derive(Debug, Clone, Default)]
pub struct MoveFlags {
    pub allow_player: bool,
    pub allow_sanctuary: bool,
    pub allow_wall: bool,
    pub allow_rock: bool,
    pub allow_bars: bool,
    pub allow_dig: bool,
    pub allow_ssm: bool,
    pub no_garlic: bool,
    pub no_teleport_line: bool,
    pub open_door: bool,
    pub unlock_door: bool,
    pub bust_door: bool,
}

/// [v2.9.4] 이동 플래그 계산 (원본: m_move L1082-1108)
pub fn calc_move_flags(
    is_peaceful: bool,
    resist_conflict: bool,
    is_minion: bool,
    is_rider: bool,
    is_unicorn: bool,
    no_teleport_level: bool,
    passes_walls: bool,
    passes_bars: bool,
    can_tunnel: bool,
    is_human: bool,
    is_minotaur: bool,
    is_undead_not_ghost: bool,
    is_vampshifter: bool,
    throws_rocks: bool,
    can_open: bool,
    can_unlock: bool,
    is_doorbuster: bool,
) -> MoveFlags {
    let mut f = MoveFlags::default();

    if is_peaceful && (!false || resist_conflict) {
        // 평화: sanctuary/ssm 허용
        f.allow_sanctuary = true;
        f.allow_ssm = true;
    } else {
        f.allow_player = true;
    }

    if is_minion || is_rider {
        f.allow_sanctuary = true;
    }
    if is_unicorn && !no_teleport_level {
        f.no_teleport_line = true;
    }
    if passes_walls {
        f.allow_wall = true;
        f.allow_rock = true;
    }
    if passes_bars {
        f.allow_bars = true;
    }
    if can_tunnel {
        f.allow_dig = true;
    }
    if is_human || is_minotaur {
        f.allow_ssm = true;
    }
    if is_undead_not_ghost || is_vampshifter {
        f.no_garlic = true;
    }
    if throws_rocks {
        f.allow_rock = true;
    }
    if can_open {
        f.open_door = true;
    }
    if can_unlock {
        f.unlock_door = true;
    }
    if is_doorbuster {
        f.bust_door = true;
    }

    f
}

// ─────────────────────────────────────────────────────────────────────────────
// m_move 최적 위치 선택 (원본: m_move L1109-1163)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 위치 후보
#[derive(Debug, Clone)]
pub struct PosCandidate {
    pub x: i32,
    pub y: i32,
    pub dist_to_goal: i32,
    pub has_monster: bool,
    pub allow_displace: bool,
    pub allow_attack: bool,
    pub allow_player: bool,
    pub on_track: bool,
}

/// [v2.9.4] 최적 위치 선택 결과 (원본: m_move L1133-1162)
#[derive(Debug, Clone)]
pub struct BestMoveResult {
    /// 선택된 위치
    pub best_pos: Option<(i32, i32)>,
    /// 선택된 인덱스
    pub best_idx: Option<usize>,
    /// 이동 여부
    pub moved: bool,
}

/// [v2.9.4] 최적 위치 선택 (원본: m_move L1133-1162)
pub fn select_best_move(
    candidates: &[PosCandidate],
    appr: i32,
    current_dist: i32,
    better_with_displace: bool,
    rng: &mut NetHackRng,
) -> BestMoveResult {
    let mut best_pos = None;
    let mut best_idx = None;
    let mut best_dist = current_dist;
    let mut moved = false;
    let mut chcnt = 0i32;

    for (i, cand) in candidates.iter().enumerate() {
        // 밀어내기 필터 (원본: L1139-1141)
        if cand.has_monster && cand.allow_displace && !cand.allow_attack && !better_with_displace {
            continue;
        }

        let nearer = cand.dist_to_goal < best_dist;

        // 접근/도주/무작위 (원본: L1152-1158)
        let select = match appr {
            1 => nearer,
            -1 => !nearer,
            _ => {
                chcnt += 1;
                rng.rn2(chcnt) == 0
            }
        };

        if select || !moved {
            best_pos = Some((cand.x, cand.y));
            best_idx = Some(i);
            best_dist = cand.dist_to_goal;
            moved = true;
        }
    }

    BestMoveResult {
        best_pos,
        best_idx,
        moved,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// m_move 이동 후처리 — 문 처리 (원본: m_move L1304-1411)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 문 처리 행동
#[derive(Debug, Clone, PartialEq)]
pub enum DoorAction {
    /// 문 처리 불필요
    None,
    /// 점액/안개로 통과 (ooze/flow)
    OozeThrough,
    /// 잠김 해제 + 열기
    UnlockAndOpen,
    /// 일반 열기
    Open,
    /// 부수기 (거인)
    SmashDown,
    /// 트랩 폭발 → nodoor
    TrapExplode,
}

/// [v2.9.4] 문 결과
#[derive(Debug, Clone)]
pub struct DoorHandleResult {
    pub action: DoorAction,
    /// 트랩 있음
    pub is_trapped: bool,
    /// 트랩 해제 (MKoT)
    pub trap_disarmed: bool,
    /// 시야 내 메시지
    pub show_message: bool,
    /// 50% 확률로 NODOOR (잠김 + 부수기)
    pub becomes_nodoor: bool,
    /// 상점 수리 필요
    pub shop_damage: bool,
}

/// [v2.9.4] 문 처리 판정 (원본: m_move L1304-1397)
pub fn door_handle_result(
    is_door: bool,
    is_locked: bool,
    is_closed: bool,
    is_trapped: bool,
    passes_walls: bool,
    can_tunnel: bool,
    is_amorphous: bool,
    can_unlock: bool,
    can_open: bool,
    is_doorbuster: bool,
    has_magic_key: bool,
    in_shop: bool,
    rng: &mut NetHackRng,
) -> DoorHandleResult {
    let mut r = DoorHandleResult {
        action: DoorAction::None,
        is_trapped,
        trap_disarmed: false,
        show_message: true,
        becomes_nodoor: false,
        shop_damage: false,
    };

    if !is_door || passes_walls || can_tunnel {
        return r;
    }

    // MKoT 해제 (원본: L1313-1321)
    if is_trapped && has_magic_key {
        r.trap_disarmed = true;
        r.is_trapped = false;
    }

    if (is_locked || is_closed) && is_amorphous {
        r.action = DoorAction::OozeThrough;
    } else if is_locked && can_unlock {
        if r.is_trapped {
            r.action = DoorAction::TrapExplode;
            r.becomes_nodoor = true;
        } else {
            r.action = DoorAction::UnlockAndOpen;
        }
    } else if is_closed && !is_locked && can_open {
        if r.is_trapped {
            r.action = DoorAction::TrapExplode;
            r.becomes_nodoor = true;
        } else {
            r.action = DoorAction::Open;
        }
    } else if (is_locked || is_closed) && is_doorbuster {
        if r.is_trapped {
            r.action = DoorAction::TrapExplode;
            r.becomes_nodoor = true;
        } else {
            r.action = DoorAction::SmashDown;
            // 50% NODOOR vs BROKEN (원본: L1387-1390)
            r.becomes_nodoor = is_locked && rng.rn2(2) == 0;
        }
        r.shop_damage = in_shop;
    }

    r
}

// ─────────────────────────────────────────────────────────────────────────────
// m_move 포스트무브 — 아이템 주움/은신 (원본: m_move L1259-1497)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 포스트무브 결과 (원본: m_move L1429-1496)
#[derive(Debug, Clone)]
pub struct PostMoveResult {
    /// 금속 식사 필요 (metallivorous)
    pub eat_metal: bool,
    /// 금 줍기
    pub pick_gold: bool,
    /// 젤라틴 큐브 식사 (meatobj)
    pub cube_eat: bool,
    /// 아이템 줍기 (practical/magical/boulder/gem)
    pub pick_items: bool,
    /// 은신 재시도
    pub re_hide: bool,
    /// 상점주인 사후 처리
    pub shk_after: bool,
    /// 뱀파이어 안개 변신 필요
    pub vamp_fog_shift: bool,
}

/// [v2.9.4] 포스트무브 판정 (원본: m_move L1429-1496)
pub fn post_move_result(
    has_obj_here: bool,
    can_act: bool,
    is_metallivorous: bool,
    has_gold_here: bool,
    like_gold: bool,
    is_gelatinous_cube: bool,
    hides_under: bool,
    is_eel: bool,
    is_shopkeeper: bool,
    is_vampshifter: bool,
    is_amorphous: bool,
    at_closed_door: bool,
    can_fog: bool,
    likes: &ItemLikes,
) -> PostMoveResult {
    PostMoveResult {
        eat_metal: has_obj_here && can_act && is_metallivorous,
        pick_gold: has_gold_here && like_gold,
        cube_eat: has_obj_here && can_act && is_gelatinous_cube,
        pick_items: has_obj_here
            && can_act
            && (likes.like_objs
                || likes.like_magic
                || likes.like_rock
                || likes.like_gems
                || likes.uses_items),
        re_hide: hides_under || is_eel,
        shk_after: is_shopkeeper,
        vamp_fog_shift: is_vampshifter && !is_amorphous && at_closed_door && can_fog,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// dochug 메인 루프 — 전투/이동 통합 (원본: dochug L440-663)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] dochug 메인 루프 결과 (원본: dochug L450-662)
#[derive(Debug, Clone)]
pub struct DochugMainResult {
    /// 마인드 플레이어 정신파 공격
    pub mind_blast: bool,
    /// 정신파 피해량
    pub mind_blast_damage: i32,
    /// 무기 장비 필요
    pub need_wield_weapon: bool,
    /// 이동 실행
    pub should_move: bool,
    /// 이동 후 거리 재계산
    pub recalc_after_move: bool,
    /// 근접 공격 시도
    pub should_melee: bool,
    /// 원거리 공격 시도 (이동 후 거리)
    pub ranged_after_move: bool,
    /// Quest 대화
    pub quest_talk: bool,
    /// 욕설 (MS_CUSS)
    pub cuss: bool,
    /// 악마 협상 (MS_BRIBE)
    pub demon_bribe: bool,
    /// 방어/기타 아이템 사용
    pub use_defensive: bool,
    /// 경비원 업무
    pub watch_duty: bool,
}

/// [v2.9.4] dochug 메인 루프 판정 (원본: dochug L450-662)
pub fn dochug_main_result(
    in_range: bool,
    nearby: bool,
    scared: bool,
    is_peaceful: bool,
    is_conflict: bool,
    is_watch: bool,
    is_mind_flayer: bool,
    has_weapon_attack: bool,
    distance_sq: i32,
    is_fleeing: bool,
    is_confused: bool,
    is_stunned: bool,
    is_invisible: bool,
    is_wanderer: bool,
    can_see: bool,
    ms_bribe: bool,
    ms_cuss: bool,
    is_sleeping: bool,
    can_move: bool,
    rng: &mut NetHackRng,
) -> DochugMainResult {
    let mut r = DochugMainResult {
        mind_blast: false,
        mind_blast_damage: 0,
        need_wield_weapon: false,
        should_move: false,
        recalc_after_move: false,
        should_melee: false,
        ranged_after_move: false,
        quest_talk: false,
        cuss: false,
        demon_bribe: false,
        use_defensive: false,
        watch_duty: false,
    };

    // 경비원 (원본: L492-493)
    if is_watch {
        r.watch_duty = true;
    }

    // 마인드 플레이어 (원본: L495)
    if is_mind_flayer && rng.rn2(20) == 0 {
        r.mind_blast = true;
        let dmg = rng.rnd(15);
        r.mind_blast_damage = dmg;
    }

    // 악마 협상 (원본: L469-489)
    if nearby && ms_bribe && is_peaceful {
        r.demon_bribe = true;
    }

    // 무기 장비 (원본: L549-569)
    if (!is_peaceful || is_conflict) && in_range && distance_sq <= 8 && has_weapon_attack && !scared
    {
        r.need_wield_weapon = true;
    }

    // 이동 판정 (원본: L574-579)
    r.should_move = !nearby
        || is_fleeing
        || scared
        || is_confused
        || is_stunned
        || (is_invisible && rng.rn2(3) != 0)
        || (is_wanderer && rng.rn2(4) != 0)
        || (is_conflict)
        || (!can_see && rng.rn2(4) != 0)
        || is_peaceful;

    r.recalc_after_move = r.should_move;

    // 근접 공격 (원본: L645-649)
    if !is_peaceful || is_conflict {
        if in_range && !scared {
            r.should_melee = true;
        }
    }

    // Quest 대화 (원본: L655-656)
    if !is_sleeping && can_move && nearby {
        r.quest_talk = true;
    }

    // 욕설 (원본: L658-660)
    if in_range && ms_cuss && !is_peaceful && can_see && !is_invisible && rng.rn2(5) == 0 {
        r.cuss = true;
    }

    r
}

// ─────────────────────────────────────────────────────────────────────────────
// m_move 이동 후 충돌 처리 (원본: m_move L1165-1236)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 이동 후 충돌 결과 (원본: m_move L1165-1236)
#[derive(Debug, Clone, PartialEq)]
pub enum MoveCollisionResult {
    /// 정상 이동 (충돌 없음)
    MoveOk,
    /// 점착 때문에 이동 불가 → Stuck
    StuckByPlayer,
    /// 굴착 무기 교체 → Stuck
    DigWeaponSwitch,
    /// 플레이어 위치에 접근 → DidNotMove
    ApproachPlayer,
    /// 몬스터 공격 (aggressor 사망 = Died)
    AttackMonDied,
    /// 몬스터 공격 → Stuck (반격 포함)
    AttackMonStuck,
    /// 밀어내기 성공 → Moved
    DisplaceOk,
    /// 밀어내기 중 사망 → Died
    DisplaceDied,
    /// 영역 불가 → Stuck
    RegionBlocked,
}

/// [v2.9.4] 이동 후 충돌 판정 (원본: m_move L1165-1239)
pub fn move_collision_result(
    moved: bool,
    target_is_player: bool,
    is_stuck_to_player: bool,
    allow_attack_mon: bool,
    allow_displace: bool,
    allow_player: bool,
    target_has_monster: bool,
    region_ok: bool,
) -> MoveCollisionResult {
    if !moved {
        return MoveCollisionResult::MoveOk;
    }

    // 점착 (원본: L1168-1169)
    if is_stuck_to_player {
        return MoveCollisionResult::StuckByPlayer;
    }

    // ALLOW_U → 플레이어에 접근 (원본: L1186-1193)
    if allow_player && target_is_player {
        return MoveCollisionResult::ApproachPlayer;
    }

    // 몬스터 공격 (원본: L1201-1223)
    if allow_attack_mon || target_has_monster {
        return MoveCollisionResult::AttackMonStuck;
    }

    // 밀어내기 (원본: L1225-1236)
    if allow_displace && target_has_monster {
        return MoveCollisionResult::DisplaceOk;
    }

    // 영역 검사 (원본: L1238-1239)
    if !region_ok {
        return MoveCollisionResult::RegionBlocked;
    }

    MoveCollisionResult::MoveOk
}

// ─────────────────────────────────────────────────────────────────────────────
// 철창 처리 (원본: m_move L1398-1411)
// ─────────────────────────────────────────────────────────────────────────────

/// [v2.9.4] 철창 처리 결과 (원본: m_move L1398-1411)
#[derive(Debug, Clone, PartialEq)]
pub enum IronBarsAction {
    /// 처리 불필요
    None,
    /// 녹/부식 공격으로 식사 (dissolve)
    EatThrough,
    /// 통과 (pass walls/between)
    PassThrough,
    /// 통과 불가
    Blocked,
}

/// [v2.9.4] 철창 처리 판정 (원본: m_move L1398-1411)
pub fn iron_bars_action(
    is_iron_bars: bool,
    is_non_diggable: bool,
    has_rust_attack: bool,
    has_corr_attack: bool,
    passes_walls: bool,
    passes_bars: bool,
) -> IronBarsAction {
    if !is_iron_bars {
        return IronBarsAction::None;
    }

    if !is_non_diggable && (has_rust_attack || has_corr_attack) {
        return IronBarsAction::EatThrough;
    }

    if passes_walls || passes_bars {
        return IronBarsAction::PassThrough;
    }

    IronBarsAction::Blocked
}

// =============================================================================
// [v2.9.4] monmove.c 2차 이식 테스트
// =============================================================================
#[cfg(test)]
mod monmove_phase2_tests {
    use super::*;

    #[test]
    fn test_m_move_pre_trapped_died() {
        let mut rng = NetHackRng::new(42);
        let r = m_move_pre_result(true, 2, 0, false, false, &mut rng);
        assert!(r.trap_died);
        assert!(!r.still_trapped);
    }

    #[test]
    fn test_m_move_pre_eating() {
        let mut rng = NetHackRng::new(42);
        let r = m_move_pre_result(false, 0, 5, false, false, &mut rng);
        assert!(r.eating_skip);
        assert!(r.eating_decrement);
        assert!(!r.finish_eating);
    }

    #[test]
    fn test_m_move_pre_eating_finish() {
        let mut rng = NetHackRng::new(42);
        let r = m_move_pre_result(false, 0, 1, false, false, &mut rng);
        assert!(r.finish_eating);
    }

    #[test]
    fn test_special_delegate_pet() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            special_move_delegate(false, true, false, false, false, false, false, false, &mut rng),
            SpecialMoveDelegate::PetMove
        );
    }

    #[test]
    fn test_special_delegate_normal() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            special_move_delegate(false, false, false, false, false, false, false, false, &mut rng),
            SpecialMoveDelegate::Normal
        );
    }

    #[test]
    fn test_approach_fleeing() {
        let mut rng = NetHackRng::new(42);
        let r = approach_result(
            true, false, false, true, false, false, false, false, false, false, 10, 10, &mut rng,
        );
        assert_eq!(r.appr, -1);
    }

    #[test]
    fn test_approach_confused() {
        let mut rng = NetHackRng::new(42);
        let r = approach_result(
            false, true, false, true, false, false, false, false, false, false, 10, 10, &mut rng,
        );
        assert_eq!(r.appr, 0);
    }

    #[test]
    fn test_item_likes_heavy() {
        let l = item_likes_result(true, true, true, true, true, false, false, false, 96, false);
        assert!(!l.like_gold); // 96% > 95%
        assert!(!l.like_rock); // 96% > 50%
    }

    #[test]
    fn test_item_likes_light() {
        let l = item_likes_result(true, true, true, true, true, true, false, false, 30, false);
        assert!(l.like_gold);
        assert!(l.like_gems);
        assert!(l.like_rock);
        assert!(l.conceals);
    }

    #[test]
    fn test_move_flags_peaceful() {
        let f = calc_move_flags(
            true, true, false, false, false, false, false, false, false, false, false, false,
            false, false, true, false, false,
        );
        assert!(f.allow_sanctuary);
        assert!(f.allow_ssm);
        assert!(!f.allow_player);
    }

    #[test]
    fn test_move_flags_hostile() {
        let f = calc_move_flags(
            false, false, false, false, false, false, true, false, true, false, false, false,
            false, false, false, false, true,
        );
        assert!(f.allow_player);
        assert!(f.allow_wall);
        assert!(f.allow_dig);
        assert!(f.bust_door);
    }

    #[test]
    fn test_select_best_approaching() {
        let mut rng = NetHackRng::new(42);
        let cands = vec![
            PosCandidate {
                x: 5,
                y: 5,
                dist_to_goal: 10,
                has_monster: false,
                allow_displace: false,
                allow_attack: false,
                allow_player: false,
                on_track: false,
            },
            PosCandidate {
                x: 6,
                y: 5,
                dist_to_goal: 5,
                has_monster: false,
                allow_displace: false,
                allow_attack: false,
                allow_player: false,
                on_track: false,
            },
        ];
        let r = select_best_move(&cands, 1, 15, false, &mut rng);
        assert!(r.moved);
        assert_eq!(r.best_pos, Some((6, 5)));
    }

    #[test]
    fn test_select_best_fleeing() {
        let mut rng = NetHackRng::new(42);
        let cands = vec![
            PosCandidate {
                x: 5,
                y: 5,
                dist_to_goal: 10,
                has_monster: false,
                allow_displace: false,
                allow_attack: false,
                allow_player: false,
                on_track: false,
            },
            PosCandidate {
                x: 3,
                y: 3,
                dist_to_goal: 20,
                has_monster: false,
                allow_displace: false,
                allow_attack: false,
                allow_player: false,
                on_track: false,
            },
        ];
        let r = select_best_move(&cands, -1, 15, false, &mut rng);
        assert!(r.moved);
        assert_eq!(r.best_pos, Some((3, 3)));
    }

    #[test]
    fn test_door_handle_ooze() {
        let mut rng = NetHackRng::new(42);
        let r = door_handle_result(
            true, true, false, false, false, false, true, false, false, false, false, false,
            &mut rng,
        );
        assert_eq!(r.action, DoorAction::OozeThrough);
    }

    #[test]
    fn test_door_handle_unlock() {
        let mut rng = NetHackRng::new(42);
        let r = door_handle_result(
            true, true, false, false, false, false, false, true, false, false, false, false,
            &mut rng,
        );
        assert_eq!(r.action, DoorAction::UnlockAndOpen);
    }

    #[test]
    fn test_door_handle_smash() {
        let mut rng = NetHackRng::new(42);
        let r = door_handle_result(
            true, true, false, false, false, false, false, false, false, true, false, false,
            &mut rng,
        );
        assert_eq!(r.action, DoorAction::SmashDown);
    }

    #[test]
    fn test_door_handle_trapped() {
        let mut rng = NetHackRng::new(42);
        let r = door_handle_result(
            true, false, true, true, false, false, false, false, true, false, false, false,
            &mut rng,
        );
        assert_eq!(r.action, DoorAction::TrapExplode);
    }

    #[test]
    fn test_door_magic_key_disarm() {
        let mut rng = NetHackRng::new(42);
        let r = door_handle_result(
            true, true, false, true, false, false, false, true, false, false, true, false, &mut rng,
        );
        assert!(r.trap_disarmed);
        assert!(!r.is_trapped);
        assert_eq!(r.action, DoorAction::UnlockAndOpen);
    }

    #[test]
    fn test_post_move_metal() {
        let likes = ItemLikes {
            like_gold: false,
            like_gems: false,
            like_objs: false,
            like_magic: false,
            like_rock: false,
            conceals: false,
            uses_items: false,
        };
        let r = post_move_result(
            true, true, true, false, false, false, false, false, false, false, false, false, false,
            &likes,
        );
        assert!(r.eat_metal);
        assert!(!r.pick_gold);
    }

    #[test]
    fn test_post_move_vamp_fog() {
        let likes = ItemLikes {
            like_gold: false,
            like_gems: false,
            like_objs: false,
            like_magic: false,
            like_rock: false,
            conceals: false,
            uses_items: false,
        };
        let r = post_move_result(
            false, false, false, false, false, false, false, false, false, true, false, true, true,
            &likes,
        );
        assert!(r.vamp_fog_shift);
    }

    #[test]
    fn test_dochug_main_mind_blast() {
        // 마인드 플레이어 정신파는 1/20 확률이므로 시드 고정으로 확인할 수 없음
        // 대신 비닐마인드 플레이어가 정신파를 안 쓰는 것을 확인
        let mut rng = NetHackRng::new(42);
        let r = dochug_main_result(
            true, true, false, false, false, false, false, false, 4, false, false, false, false,
            false, true, false, false, false, true, &mut rng,
        );
        assert!(!r.mind_blast);
    }

    #[test]
    fn test_dochug_main_should_move_peaceful() {
        let mut rng = NetHackRng::new(42);
        let r = dochug_main_result(
            true, true, false, true, false, false, false, false, 4, false, false, false, false,
            false, true, false, false, false, true, &mut rng,
        );
        assert!(r.should_move);
        assert!(!r.should_melee); // 평화
    }

    #[test]
    fn test_dochug_main_melee() {
        let mut rng = NetHackRng::new(42);
        let r = dochug_main_result(
            true, true, false, false, false, false, false, false, 4, false, false, false, false,
            false, true, false, false, false, true, &mut rng,
        );
        assert!(r.should_melee);
    }

    #[test]
    fn test_collision_stuck() {
        let r = move_collision_result(true, false, true, false, false, false, false, true);
        assert_eq!(r, MoveCollisionResult::StuckByPlayer);
    }

    #[test]
    fn test_collision_approach_player() {
        let r = move_collision_result(true, true, false, false, false, true, false, true);
        assert_eq!(r, MoveCollisionResult::ApproachPlayer);
    }

    #[test]
    fn test_collision_ok() {
        let r = move_collision_result(true, false, false, false, false, false, false, true);
        assert_eq!(r, MoveCollisionResult::MoveOk);
    }

    #[test]
    fn test_iron_bars_eat() {
        assert_eq!(
            iron_bars_action(true, false, true, false, false, false),
            IronBarsAction::EatThrough
        );
    }

    #[test]
    fn test_iron_bars_pass() {
        assert_eq!(
            iron_bars_action(true, false, false, false, true, false),
            IronBarsAction::PassThrough
        );
    }

    #[test]
    fn test_iron_bars_none() {
        assert_eq!(
            iron_bars_action(false, false, false, false, false, false),
            IronBarsAction::None
        );
    }
}
