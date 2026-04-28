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
// [v2.6.0] pickup.c 핵심 시스템 대량 이식 (472→1050줄)
// =============================================================================

use crate::core::dungeon::tile::TileType;
use crate::core::entity::object::ItemClass;
use crate::ui::log::GameLog;

// =============================================================================
// 자동 줍기 설정
// =============================================================================

/// 자동 줍기 설정 (pickup.c의 autopickup 관련 flags 이식)
#[derive(Debug, Clone)]
pub struct PickupConfig {
    pub autopickup: bool,
    pub pickup_types: Vec<ItemClass>,
    pub pickup_thrown: bool,
    pub pickup_burden: BurdenLevel,
}

impl Default for PickupConfig {
    fn default() -> Self {
        Self {
            autopickup: true,
            pickup_types: vec![ItemClass::Coin, ItemClass::Gem],
            pickup_thrown: true,
            pickup_burden: BurdenLevel::Stressed,
        }
    }
}

// =============================================================================
// 짐 레벨 (BurdenLevel)
// =============================================================================

/// 짐 단계 열거형 (pickup.c의 near_capacity 반환값 이식)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BurdenLevel {
    Unencumbered = 0,
    Burdened = 1,
    Stressed = 2,
    Strained = 3,
    Overtaxed = 4,
    Overloaded = 5,
}

// =============================================================================
// 무게/짐 계산
// =============================================================================

/// 근력 기반 최대 운반 무게 (pickup.c weight_cap 대응)
pub fn weight_capacity(strength: i32) -> i32 {
    match strength {
        0..=3 => 100,
        4..=6 => 200,
        7..=9 => 350,
        10..=12 => 500,
        13..=15 => 700,
        16..=17 => 900,
        18 => 1000,
        _ => 1000 + (strength - 18) * 100,
    }
}

/// 현재 무게와 근력으로 짐 단계 계산 (pickup.c near_capacity 대응)
pub fn burden_level(current_weight: i32, strength: i32) -> BurdenLevel {
    let cap = weight_capacity(strength);
    let ratio = if cap > 0 {
        current_weight * 100 / cap
    } else {
        100
    };
    if ratio <= 50 {
        BurdenLevel::Unencumbered
    } else if ratio <= 67 {
        BurdenLevel::Burdened
    } else if ratio <= 83 {
        BurdenLevel::Stressed
    } else if ratio <= 100 {
        BurdenLevel::Strained
    } else if ratio <= 117 {
        BurdenLevel::Overtaxed
    } else {
        BurdenLevel::Overloaded
    }
}

/// 짐 단계별 속도 패널티
pub fn burden_speed_penalty(level: BurdenLevel) -> i32 {
    match level {
        BurdenLevel::Unencumbered => 0,
        BurdenLevel::Burdened => -1,
        BurdenLevel::Stressed => -3,
        BurdenLevel::Strained => -5,
        BurdenLevel::Overtaxed => -7,
        BurdenLevel::Overloaded => -9,
    }
}

/// 짐 단계 이름
pub fn burden_name(level: BurdenLevel) -> &'static str {
    match level {
        BurdenLevel::Unencumbered => "Unencumbered",
        BurdenLevel::Burdened => "Burdened",
        BurdenLevel::Stressed => "Stressed",
        BurdenLevel::Strained => "Strained",
        BurdenLevel::Overtaxed => "Overtaxed",
        BurdenLevel::Overloaded => "Overloaded",
    }
}

// =============================================================================
// 줍기/내려놓기 결과
// =============================================================================

/// 줍기 결과 구조체
#[derive(Debug, Clone)]
pub struct PickupResult {
    pub picked_up: bool,
    pub item_name: String,
    pub item_count: u32,
    pub weight_added: i32,
    pub message: String,
    pub new_burden: BurdenLevel,
}

impl PickupResult {
    pub fn fail(msg: &str) -> Self {
        Self {
            picked_up: false,
            item_name: String::new(),
            item_count: 0,
            weight_added: 0,
            message: msg.to_string(),
            new_burden: BurdenLevel::Unencumbered,
        }
    }
}

/// 아이템 줍기 시도
pub fn try_pickup_item(
    item_name: &str,
    item_weight: i32,
    item_count: u32,
    _item_class: ItemClass,
    current_weight: i32,
    strength: i32,
    is_levitating: bool,
    log: &mut GameLog,
    turn: u64,
) -> PickupResult {
    if is_levitating {
        let msg = "You cannot reach the floor.";
        log.add(msg, turn);
        return PickupResult::fail(msg);
    }
    let total_weight = item_weight * item_count as i32;
    let new_weight = current_weight + total_weight;
    let new_burden = burden_level(new_weight, strength);
    if new_burden >= BurdenLevel::Overloaded {
        let msg = format!("{} is too heavy to pick up!", item_name);
        log.add(&msg, turn);
        return PickupResult::fail(&msg);
    }
    let msg = if item_count > 1 {
        format!("You pick up {} {}.", item_count, item_name)
    } else {
        format!("You pick up {}.", item_name)
    };
    log.add(&msg, turn);
    let old_burden = burden_level(current_weight, strength);
    if new_burden > old_burden {
        log.add(&format!("You are now {}.", burden_name(new_burden)), turn);
    }
    PickupResult {
        picked_up: true,
        item_name: item_name.to_string(),
        item_count,
        weight_added: total_weight,
        message: msg,
        new_burden,
    }
}

/// 금화 줍기
pub fn pickup_gold(amount: i32, is_levitating: bool, log: &mut GameLog, turn: u64) -> PickupResult {
    if is_levitating {
        let msg = "You cannot reach the floor.";
        log.add(msg, turn);
        return PickupResult::fail(msg);
    }
    if amount <= 0 {
        return PickupResult::fail("No gold here.");
    }
    let msg = format!(
        "You pick up {} gold piece{}.",
        amount,
        if amount != 1 { "s" } else { "" }
    );
    log.add_colored(&msg, [255, 215, 0], turn);
    PickupResult {
        picked_up: true,
        item_name: "gold".to_string(),
        item_count: amount as u32,
        weight_added: 1,
        message: msg,
        new_burden: BurdenLevel::Unencumbered,
    }
}

// =============================================================================
// 자동 줍기 판정
// =============================================================================

/// 기본 자동 줍기 판정 (pickup.c should_autopickup 대응)
pub fn should_autopickup(item_class: ItemClass, item_name: &str, config: &PickupConfig) -> bool {
    if !config.autopickup {
        return false;
    }
    if config.pickup_types.contains(&item_class) {
        return true;
    }
    if config.pickup_thrown {
        if item_name.contains("arrow")
            || item_name.contains("bolt")
            || item_name.contains("shuriken")
            || item_name.contains("dart")
        {
            return true;
        }
    }
    false
}

// =============================================================================
// 내려놓기
// =============================================================================

/// 내려놓기 결과
#[derive(Debug, Clone)]
pub struct DropResult {
    pub dropped: bool,
    pub item_name: String,
    pub item_count: u32,
    pub weight_removed: i32,
    pub message: String,
}

/// 아이템 내려놓기 시도
pub fn try_drop_item(
    item_name: &str,
    item_weight: i32,
    item_count: u32,
    is_cursed: bool,
    is_equipped: bool,
    tile_type: TileType,
    log: &mut GameLog,
    turn: u64,
) -> DropResult {
    if is_cursed && is_equipped {
        let msg = format!("Your {} is welded to you!", item_name);
        log.add_colored(&msg, [200, 50, 50], turn);
        return DropResult {
            dropped: false,
            item_name: item_name.to_string(),
            item_count: 0,
            weight_removed: 0,
            message: msg,
        };
    }
    let destroyed = matches!(tile_type, TileType::LavaPool | TileType::Pool);
    let msg = if destroyed {
        if tile_type == TileType::LavaPool {
            format!("Your {} is destroyed by the lava!", item_name)
        } else {
            format!("Your {} sinks into the water.", item_name)
        }
    } else if item_count > 1 {
        format!("You drop {} {}.", item_count, item_name)
    } else {
        format!("You drop {}.", item_name)
    };
    if destroyed {
        log.add_colored(&msg, [255, 100, 0], turn);
    } else {
        log.add(&msg, turn);
    }
    let total_weight = item_weight * item_count as i32;
    DropResult {
        dropped: true,
        item_name: item_name.to_string(),
        item_count,
        weight_removed: total_weight,
        message: msg,
    }
}

/// 전체 내려놓기 프롬프트
pub fn drop_all_prompt() -> &'static str {
    "Drop all items? [y/n]"
}

// =============================================================================
// 바닥 아이템 메시지
// =============================================================================

/// 바닥 아이템 현황 메시지
pub fn floor_items_message(item_count: usize, has_gold: bool, gold_amount: i32) -> String {
    if item_count == 0 && !has_gold {
        "There is nothing here.".to_string()
    } else if item_count == 0 && has_gold {
        format!(
            "There {} {} gold piece{} here.",
            if gold_amount == 1 { "is" } else { "are" },
            gold_amount,
            if gold_amount != 1 { "s" } else { "" }
        )
    } else if item_count == 1 && !has_gold {
        "There is an item here.".to_string()
    } else {
        let mut msg = format!("There are {} things here", item_count);
        if has_gold {
            msg.push_str(&format!(", plus {} gold", gold_amount));
        }
        msg.push('.');
        msg
    }
}

// =============================================================================
// 시체/바위 특수 줍기
// =============================================================================

/// 시체 줍기 가능 여부
pub fn can_pickup_corpse(
    corpse_age: u64,
    current_turn: u64,
    is_tinned: bool,
) -> (bool, &'static str) {
    if is_tinned {
        return (true, "The corpse is preserved in a tin.");
    }
    let age = current_turn.saturating_sub(corpse_age);
    if age > 50 {
        (false, "This corpse is too old and has rotted away!")
    } else if age > 30 {
        (true, "The corpse smells terrible.")
    } else {
        (true, "")
    }
}

/// 바위 줍기 가능 여부
pub fn can_pickup_boulder(is_giant: bool, player_str: i32) -> bool {
    is_giant || player_str >= 25
}

/// 바닥 트랩 확인
pub fn check_floor_for_trap(
    has_trap: bool,
    trap_discovered: bool,
    log: &mut GameLog,
    turn: u64,
) -> bool {
    if has_trap && !trap_discovered {
        log.add("Wait! There's a trap here.", turn);
        return true;
    }
    false
}

// =============================================================================
// [v2.6.0] pickup.c 핵심 시스템 대량 이식 — 신규 함수들
// =============================================================================

/// 금화 무게 계산 (GOLD_WT 매크로: (n+50)/100)
pub fn gold_weight(n: i64) -> i64 {
    (n + 50) / 100
}

/// 금화 운반 한도 (GOLD_CAPACITY 매크로)
pub fn gold_capacity(weight_margin: i64, current_gold: i64) -> i64 {
    (weight_margin * -100) - (current_gold + 50) - 1
}

/// 짐 상태 변화 메시지 (encumber_msg 이식)
pub fn encumber_msg(old: BurdenLevel, new: BurdenLevel) -> Option<&'static str> {
    if old < new {
        Some(match new {
            BurdenLevel::Burdened => "Your movements are slowed slightly because of your load.",
            BurdenLevel::Stressed => "You rebalance your load. Movement is difficult.",
            BurdenLevel::Strained => "You stagger under your heavy load. Movement is very hard.",
            BurdenLevel::Overtaxed => "You can barely move with this load!",
            BurdenLevel::Overloaded => "You can't even move a handspan with this load!",
            _ => return None,
        })
    } else if old > new {
        Some(match new {
            BurdenLevel::Unencumbered => "Your movements are now unencumbered.",
            BurdenLevel::Burdened => "Your movements are only slowed slightly by your load.",
            BurdenLevel::Stressed => "You rebalance your load. Movement is still difficult.",
            BurdenLevel::Strained => "You stagger under your load. Movement is still very hard.",
            _ => return None,
        })
    } else {
        None
    }
}

/// 들어올리기 경고 메시지 (moderateloadmsg 등)
pub fn lift_warning_message(burden: BurdenLevel) -> Option<&'static str> {
    match burden {
        BurdenLevel::Burdened => Some("You have a little trouble lifting"),
        BurdenLevel::Stressed | BurdenLevel::Strained => Some("You have much trouble lifting"),
        BurdenLevel::Overtaxed | BurdenLevel::Overloaded => {
            Some("You have extreme difficulty lifting")
        }
        _ => None,
    }
}

/// 바닥 컨테이너 개수 (container_at 이식)
pub fn container_count(items: &[(ItemClass, &str)]) -> usize {
    items
        .iter()
        .filter(|(c, n)| {
            *c == ItemClass::Tool
                && (n.contains("bag")
                    || n.contains("box")
                    || n.contains("chest")
                    || n.contains("sack"))
        })
        .count()
}

/// 인접 몬스터 확인 (mon_beside 이식)
pub fn mon_beside(x: i32, y: i32, monsters: &[(i32, i32)]) -> bool {
    for dx in -1..=1 {
        for dy in -1..=1 {
            if monsters
                .iter()
                .any(|&(mx, my)| mx == x + dx && my == y + dy)
            {
                return true;
            }
        }
    }
    false
}

/// 아이템 클래스→기호 문자 (def_oc_syms 대응)
pub fn class_to_symbol(class: ItemClass) -> char {
    match class {
        ItemClass::Weapon => ')',
        ItemClass::Armor => '[',
        ItemClass::Ring => '=',
        ItemClass::Amulet => '"',
        ItemClass::Tool => '(',
        ItemClass::Food => '%',
        ItemClass::Potion => '!',
        ItemClass::Scroll => '?',
        ItemClass::Spellbook => '+',
        ItemClass::Wand => '/',
        ItemClass::Coin => '$',
        ItemClass::Gem | ItemClass::Rock => '*',
        ItemClass::Ball => '0',
        ItemClass::Chain => '_',
        ItemClass::Venom => '.',
        _ => '\\',
    }
}

/// 고유 클래스 기호 수집 (collect_obj_classes 이식)
pub fn collect_obj_classes(items: &[ItemClass]) -> (Vec<char>, usize) {
    let mut syms = Vec::new();
    for c in items {
        let s = class_to_symbol(*c);
        if !syms.contains(&s) {
            syms.push(s);
        }
    }
    (syms, items.len())
}

/// 메뉴 필터 (valid_menu_classes 시스템 이식)
#[derive(Debug, Clone, Default)]
pub struct MenuClassFilter {
    pub valid_classes: Vec<char>,
    pub class_filter: bool,
    pub bucx_filter: bool,
    pub shop_filter: bool,
}

impl MenuClassFilter {
    pub fn reset(&mut self) {
        self.valid_classes.clear();
        self.class_filter = false;
        self.bucx_filter = false;
        self.shop_filter = false;
    }
    pub fn add_class(&mut self, c: char) {
        if self.valid_classes.contains(&c) {
            return;
        }
        self.valid_classes.push(c);
        match c {
            'B' | 'U' | 'C' | 'X' => self.bucx_filter = true,
            'u' => self.shop_filter = true,
            _ => self.class_filter = true,
        }
    }
    pub fn has_class(&self, c: char) -> bool {
        self.valid_classes.contains(&c)
    }
}

/// BUC 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucStatus {
    Blessed,
    Uncursed,
    Cursed,
    Unknown,
}

/// 아이템 허용 여부 (allow_category 이식)
pub fn allow_category(
    filter: &MenuClassFilter,
    item_class: ItemClass,
    is_coin: bool,
    is_unpaid: bool,
    buc_known: bool,
    blessed: bool,
    cursed: bool,
    has_unpaid_contents: bool,
) -> bool {
    if is_coin {
        if filter.class_filter {
            return filter.has_class('$');
        }
        if filter.shop_filter {
            return is_unpaid;
        }
        if filter.bucx_filter {
            return filter.has_class('X') || filter.has_class('U');
        }
        return true;
    }
    let sym = class_to_symbol(item_class);
    if filter.class_filter && !filter.has_class(sym) {
        return false;
    }
    if filter.shop_filter && !is_unpaid && !has_unpaid_contents {
        return false;
    }
    if filter.bucx_filter {
        let b = if !buc_known {
            'X'
        } else if blessed {
            'B'
        } else if cursed {
            'C'
        } else {
            'U'
        };
        if !filter.has_class(b) {
            return false;
        }
    }
    true
}

/// 카테고리 수 카운트 (count_categories 이식)
pub fn count_categories(items: &[ItemClass]) -> usize {
    let mut seen = Vec::new();
    for c in items {
        if !seen.contains(c) {
            seen.push(*c);
        }
    }
    seen.len()
}

/// 컨테이너 무게 차이 (delta_cwt 이식)
pub fn delta_container_weight(is_boh: bool, wt_with: i32, wt_without: i32, item_wt: i32) -> i32 {
    if !is_boh {
        item_wt
    } else {
        wt_with - wt_without
    }
}

/// 운반 가능 수량 결과
#[derive(Debug, Clone)]
pub struct CarryResult {
    pub count: i64,
    pub weight_before: i32,
    pub weight_after: i32,
    pub message: Option<String>,
}

/// 운반 가능 수량 계산 (carry_count 이식)
pub fn carry_count(
    max_cap: i32,
    cur_load: i32,
    wt_per: i32,
    requested: i64,
    is_gold: bool,
    cur_gold: i64,
) -> CarryResult {
    let wb = max_cap - cur_load;
    let tw = if is_gold {
        (gold_weight(cur_gold + requested) - gold_weight(cur_gold)) as i32
    } else {
        wt_per * requested as i32
    };
    let wa = wb - tw;
    if wa >= 0 {
        return CarryResult {
            count: requested,
            weight_before: wb,
            weight_after: wa,
            message: None,
        };
    }
    let mut lift = 0i64;
    if is_gold {
        for q in 1..=requested {
            let gw = (gold_weight(cur_gold + q) - gold_weight(cur_gold)) as i32;
            if wb - gw < 0 {
                break;
            }
            lift = q;
        }
    } else if wt_per > 0 {
        lift = (wb / wt_per) as i64;
        if lift > requested {
            lift = requested;
        }
    }
    let fw = if is_gold {
        wb - (gold_weight(cur_gold + lift) - gold_weight(cur_gold)) as i32
    } else {
        wb - wt_per * lift as i32
    };
    let msg = if lift > 0 && lift < requested {
        Some(format!(
            "You can only lift {} of them.",
            if lift == 1 {
                "one".to_string()
            } else {
                format!("some ({})", lift)
            }
        ))
    } else if lift == 0 {
        Some("It is too heavy for you to lift.".into())
    } else {
        None
    };
    CarryResult {
        count: lift,
        weight_before: wb,
        weight_after: fw,
        message: msg,
    }
}

/// 들어올리기 판정 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiftResult {
    CanLift,
    CannotLift,
    Declined,
}

/// 들어올리기 종합 판정 (lift_object 이식)
pub fn can_lift_object(
    _n: &str,
    boulder: bool,
    loadstone: bool,
    sokoban: bool,
    throws_rocks: bool,
    inv_cnt: usize,
    already_has: bool,
    can_merge: bool,
    new_burden: BurdenLevel,
    threshold: BurdenLevel,
    telekinesis: bool,
) -> LiftResult {
    if boulder && sokoban {
        return LiftResult::CannotLift;
    }
    if loadstone || (boulder && throws_rocks) {
        return if inv_cnt < 52 || !already_has || can_merge {
            LiftResult::CanLift
        } else {
            LiftResult::CannotLift
        };
    }
    if new_burden > threshold && telekinesis {
        return LiftResult::Declined;
    }
    LiftResult::CanLift
}

/// 오토픽업 예외 규칙
#[derive(Debug, Clone)]
pub struct AutopickupException {
    pub pattern: String,
    pub grab: bool,
}

/// 개별 오토픽업 적격 판정 (autopick_testobj 이식)
pub fn autopick_test(
    class: ItemClass,
    name: &str,
    unpaid: bool,
    thrown: bool,
    types: &[ItemClass],
    pickup_thrown: bool,
    exc: &[AutopickupException],
) -> bool {
    if unpaid {
        return false;
    }
    let mut pick = types.is_empty() || types.contains(&class);
    for e in exc {
        if name.contains(&e.pattern) {
            pick = e.grab;
        }
    }
    if !pick && pickup_thrown && thrown {
        pick = true;
    }
    pick
}

/// 마법 가방 폭발 판정 (mbag_explodes 이식)
pub fn mbag_explodes(is_mbag: bool, is_cancel: bool, charges: i32, depth: u32, rng: u32) -> bool {
    if (is_cancel || is_mbag) && charges <= 0 {
        return false;
    }
    if is_mbag || is_cancel {
        let c = depth.min(7);
        return rng % (1u32 << c) <= depth;
    }
    false
}

/// 저주 마법 가방 소실 (boh_loss 이식)
pub fn boh_loss(is_mbag: bool, cursed: bool, vals: &[i64], rngs: &[u32]) -> (i64, Vec<bool>) {
    let mut loss = 0i64;
    let mut gone = vec![false; vals.len()];
    if !is_mbag || !cursed {
        return (0, gone);
    }
    for (i, (&v, &r)) in vals.iter().zip(rngs.iter()).enumerate() {
        if r % 13 == 0 {
            gone[i] = true;
            loss += v;
        }
    }
    (loss, gone)
}

/// 코카트리스 석화 위험 (fatal_corpse_mistake 이식)
pub fn fatal_corpse_check(
    corpse: bool,
    cockatrice: bool,
    gloves: bool,
    remote: bool,
    resist: bool,
) -> bool {
    corpse && cockatrice && !gloves && !remote && !resist
}

/// 라이더 시체 부활 (rider_corpse_revival 이식)
pub fn rider_corpse_check(corpse: bool, rider: bool) -> bool {
    corpse && rider
}

/// 아이스박스 제거 시 나이 복원 (removed_from_icebox 이식)
pub fn icebox_removal_age(frozen: u64, turn: u64) -> u64 {
    turn.saturating_sub(frozen)
}

/// 아이스박스 냉동 시 나이 저장값
pub fn icebox_freeze_age(real: u64, turn: u64) -> u64 {
    turn.saturating_sub(real)
}

/// 컨테이너 삽입 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerInsertResult {
    Ok,
    Blocked(String),
}

/// 컨테이너 삽입 판정 (in_container 이식)
pub fn can_insert_into_container(
    name: &str,
    _class: ItemClass,
    worn: bool,
    cursed_load: bool,
    quest_art: bool,
    leashed: bool,
    welded: bool,
    boulder: bool,
    big_statue: bool,
    is_box: bool,
    icebox: bool,
) -> ContainerInsertResult {
    if worn {
        return ContainerInsertResult::Blocked(format!(
            "You cannot {} something you are wearing.",
            if icebox { "refrigerate" } else { "stash" }
        ));
    }
    if cursed_load {
        return ContainerInsertResult::Blocked("The stone won't leave your person.".into());
    }
    if quest_art {
        return ContainerInsertResult::Blocked(format!(
            "{} cannot be confined in such trappings.",
            name
        ));
    }
    if leashed {
        return ContainerInsertResult::Blocked("It is attached to your pet.".into());
    }
    if welded {
        return ContainerInsertResult::Blocked("Your weapon is welded to your hand!".into());
    }
    if is_box || boulder || big_statue {
        return ContainerInsertResult::Blocked(format!(
            "You cannot fit {} into the container.",
            name
        ));
    }
    ContainerInsertResult::Ok
}

/// 슈뢰딩거 고양이 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuantumCatResult {
    Alive,
    Dead,
    NotQuantum,
}

/// 슈뢰딩거 고양이 관측 (observe_quantum_cat 이식)
pub fn observe_quantum_cat(is_s: bool, rng: u32) -> QuantumCatResult {
    if !is_s {
        QuantumCatResult::NotQuantum
    } else if rng % 2 == 0 {
        QuantumCatResult::Alive
    } else {
        QuantumCatResult::Dead
    }
}

/// 컨테이너 동작
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerAction {
    Look,
    TakeOut,
    PutIn,
    Both,
    BothReversed,
    StashOne,
    NextContainer,
    Quit,
}

/// 가능한 컨테이너 동작 목록 (in_or_out_menu 이식)
pub fn container_action_options(contents: bool, inv: bool, more: bool) -> Vec<ContainerAction> {
    let mut o = vec![ContainerAction::Look];
    if contents {
        o.push(ContainerAction::TakeOut);
    }
    if inv {
        o.push(ContainerAction::PutIn);
    }
    if contents {
        o.push(ContainerAction::Both);
    }
    if inv {
        o.push(ContainerAction::BothReversed);
        o.push(ContainerAction::StashOne);
    }
    if more {
        o.push(ContainerAction::NextContainer);
    }
    o.push(ContainerAction::Quit);
    o
}

/// 컨테이너 동작 단축키 (lootchars 이식)
pub fn container_action_key(a: ContainerAction) -> char {
    match a {
        ContainerAction::Look => ':',
        ContainerAction::TakeOut => 'o',
        ContainerAction::PutIn => 'i',
        ContainerAction::Both => 'b',
        ContainerAction::BothReversed => 'r',
        ContainerAction::StashOne => 's',
        ContainerAction::NextContainer => 'n',
        ContainerAction::Quit => 'q',
    }
}

/// 쏟아짐 메시지 (tipcontainer 일부)
pub fn spill_objects_message(count: usize) -> &'static str {
    if count > 1 {
        "Objects spill out:"
    } else {
        "An object spills out."
    }
}

/// 흘리기 판정 (dotip spillage 이식)
pub fn tip_spillage(name: &str, lit: bool, charges: bool) -> Option<&'static str> {
    if name.contains("candle") && lit {
        Some("wax")
    } else if (name.contains("oil") || name.contains("lamp")) && lit {
        Some("oil")
    } else if name.contains("grease") && charges {
        Some("grease")
    } else if name.contains("ration") || name.contains("wafer") {
        Some("crumbs")
    } else {
        None
    }
}

/// 손 사용 가능 여부 (u_handsy 이식)
pub fn can_use_hands(hands: bool, free: bool) -> Result<(), &'static str> {
    if !hands {
        Err("You have no hands!")
    } else if !free {
        Err("You have no free hand.")
    } else {
        Ok(())
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weight_capacity() {
        assert!(weight_capacity(18) > weight_capacity(10));
        assert!(weight_capacity(10) > weight_capacity(3));
    }
    #[test]
    fn test_burden_level() {
        assert_eq!(burden_level(100, 18), BurdenLevel::Unencumbered);
        assert_eq!(burden_level(5000, 10), BurdenLevel::Overloaded);
    }
    #[test]
    fn test_floor_message() {
        assert_eq!(floor_items_message(0, false, 0), "There is nothing here.");
        assert!(floor_items_message(0, true, 50).contains("50"));
    }
    #[test]
    fn test_autopickup() {
        let c = PickupConfig::default();
        assert!(should_autopickup(ItemClass::Coin, "gold piece", &c));
        assert!(!should_autopickup(ItemClass::Weapon, "sword", &c));
    }
    #[test]
    fn test_corpse_check() {
        assert!(can_pickup_corpse(0, 10, false).0);
        assert!(!can_pickup_corpse(0, 100, false).0);
    }
    #[test]
    fn test_boulder_pickup() {
        assert!(can_pickup_boulder(true, 10));
        assert!(can_pickup_boulder(false, 25));
        assert!(!can_pickup_boulder(false, 15));
    }

    // [v2.6.0] 신규 테스트
    #[test]
    fn test_gold_weight() {
        assert_eq!(gold_weight(100), 1);
        assert_eq!(gold_weight(50), 1);
        assert_eq!(gold_weight(49), 0);
        assert_eq!(gold_weight(250), 3);
    }
    #[test]
    fn test_gold_capacity() {
        assert_eq!(gold_capacity(-1, 0), 49);
    }
    #[test]
    fn test_encumber_msg_up() {
        assert!(
            encumber_msg(BurdenLevel::Unencumbered, BurdenLevel::Burdened)
                .unwrap()
                .contains("slowed")
        );
    }
    #[test]
    fn test_encumber_msg_down() {
        assert!(
            encumber_msg(BurdenLevel::Stressed, BurdenLevel::Unencumbered)
                .unwrap()
                .contains("unencumbered")
        );
    }
    #[test]
    fn test_encumber_msg_same() {
        assert!(encumber_msg(BurdenLevel::Burdened, BurdenLevel::Burdened).is_none());
    }
    #[test]
    fn test_lift_warning() {
        assert!(lift_warning_message(BurdenLevel::Burdened).is_some());
        assert!(lift_warning_message(BurdenLevel::Unencumbered).is_none());
    }
    #[test]
    fn test_container_count() {
        assert_eq!(
            container_count(&[
                (ItemClass::Tool, "large box"),
                (ItemClass::Weapon, "sword"),
                (ItemClass::Tool, "bag of holding")
            ]),
            2
        );
    }
    #[test]
    fn test_mon_beside() {
        assert!(mon_beside(4, 5, &[(5, 5), (10, 10)]));
        assert!(!mon_beside(0, 0, &[(5, 5)]));
    }
    #[test]
    fn test_class_sym() {
        assert_eq!(class_to_symbol(ItemClass::Weapon), ')');
        assert_eq!(class_to_symbol(ItemClass::Coin), '$');
    }
    #[test]
    fn test_collect_classes() {
        let (s, c) = collect_obj_classes(&[
            ItemClass::Weapon,
            ItemClass::Potion,
            ItemClass::Weapon,
            ItemClass::Armor,
        ]);
        assert_eq!(c, 4);
        assert_eq!(s.len(), 3);
    }
    #[test]
    fn test_menu_filter() {
        let mut f = MenuClassFilter::default();
        f.add_class('B');
        assert!(f.bucx_filter);
        f.add_class(')');
        assert!(f.class_filter);
        f.reset();
        assert!(f.valid_classes.is_empty());
    }
    #[test]
    fn test_allow_cat_buc() {
        let mut f = MenuClassFilter::default();
        f.add_class('B');
        assert!(allow_category(
            &f,
            ItemClass::Potion,
            false,
            false,
            true,
            true,
            false,
            false
        ));
        assert!(!allow_category(
            &f,
            ItemClass::Potion,
            false,
            false,
            true,
            false,
            true,
            false
        ));
    }
    #[test]
    fn test_count_cat() {
        assert_eq!(
            count_categories(&[ItemClass::Weapon, ItemClass::Potion, ItemClass::Weapon]),
            2
        );
    }
    #[test]
    fn test_carry_all() {
        let r = carry_count(1000, 500, 10, 10, false, 0);
        assert_eq!(r.count, 10);
        assert!(r.message.is_none());
    }
    #[test]
    fn test_carry_partial() {
        let r = carry_count(550, 500, 10, 10, false, 0);
        assert_eq!(r.count, 5);
        assert!(r.message.is_some());
    }
    #[test]
    fn test_carry_none() {
        assert_eq!(carry_count(500, 500, 10, 10, false, 0).count, 0);
    }
    #[test]
    fn test_lift_sokoban() {
        assert_eq!(
            can_lift_object(
                "",
                true,
                false,
                true,
                false,
                10,
                false,
                false,
                BurdenLevel::Unencumbered,
                BurdenLevel::Stressed,
                false
            ),
            LiftResult::CannotLift
        );
    }
    #[test]
    fn test_lift_loadstone() {
        assert_eq!(
            can_lift_object(
                "",
                false,
                true,
                false,
                false,
                10,
                false,
                false,
                BurdenLevel::Overloaded,
                BurdenLevel::Stressed,
                false
            ),
            LiftResult::CanLift
        );
    }
    #[test]
    fn test_autopick_basic() {
        assert!(autopick_test(
            ItemClass::Coin,
            "gold",
            false,
            false,
            &[ItemClass::Coin],
            false,
            &[]
        ));
        assert!(!autopick_test(
            ItemClass::Coin,
            "gold",
            true,
            false,
            &[ItemClass::Coin],
            false,
            &[]
        ));
    }
    #[test]
    fn test_autopick_exc() {
        let e = AutopickupException {
            pattern: "identify".into(),
            grab: true,
        };
        assert!(autopick_test(
            ItemClass::Scroll,
            "scroll of identify",
            false,
            false,
            &[],
            false,
            &[e]
        ));
    }
    #[test]
    fn test_fatal_corpse() {
        assert!(fatal_corpse_check(true, true, false, false, false));
        assert!(!fatal_corpse_check(true, true, true, false, false));
    }
    #[test]
    fn test_rider() {
        assert!(rider_corpse_check(true, true));
        assert!(!rider_corpse_check(false, true));
    }
    #[test]
    fn test_icebox_age() {
        let f = icebox_freeze_age(30, 100);
        assert_eq!(f, 70);
        assert_eq!(icebox_removal_age(f, 200), 130);
    }
    #[test]
    fn test_boh_loss() {
        let (l, g) = boh_loss(true, true, &[100, 200, 300], &[0, 1, 13]);
        assert_eq!(l, 400);
        assert_eq!(g, vec![true, false, true]);
    }
    #[test]
    fn test_boh_safe() {
        assert_eq!(boh_loss(true, false, &[100], &[0]).0, 0);
    }
    #[test]
    fn test_insert_worn() {
        assert!(matches!(
            can_insert_into_container(
                "mail",
                ItemClass::Armor,
                true,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                false
            ),
            ContainerInsertResult::Blocked(_)
        ));
    }
    #[test]
    fn test_insert_ok() {
        assert_eq!(
            can_insert_into_container(
                "dagger",
                ItemClass::Weapon,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                false,
                false
            ),
            ContainerInsertResult::Ok
        );
    }
    #[test]
    fn test_quantum() {
        assert_eq!(observe_quantum_cat(false, 0), QuantumCatResult::NotQuantum);
        assert_eq!(observe_quantum_cat(true, 0), QuantumCatResult::Alive);
        assert_eq!(observe_quantum_cat(true, 1), QuantumCatResult::Dead);
    }
    #[test]
    fn test_actions() {
        let o = container_action_options(true, true, false);
        assert!(o.contains(&ContainerAction::Look));
        assert!(!o.contains(&ContainerAction::NextContainer));
    }
    #[test]
    fn test_action_key() {
        assert_eq!(container_action_key(ContainerAction::Look), ':');
        assert_eq!(container_action_key(ContainerAction::Quit), 'q');
    }
    #[test]
    fn test_spill() {
        assert_eq!(spill_objects_message(1), "An object spills out.");
        assert_eq!(spill_objects_message(5), "Objects spill out:");
    }
    #[test]
    fn test_tip() {
        assert_eq!(tip_spillage("candle", true, false), Some("wax"));
        assert_eq!(tip_spillage("sword", false, false), None);
    }
    #[test]
    fn test_hands() {
        assert!(can_use_hands(true, true).is_ok());
        assert!(can_use_hands(false, true).is_err());
    }
    #[test]
    fn test_delta_wt() {
        assert_eq!(delta_container_weight(false, 100, 80, 20), 20);
        assert_eq!(delta_container_weight(true, 100, 85, 20), 15);
    }
}
