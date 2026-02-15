// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::entity::{ContainerProperties, ContainerType, Inventory, Item};
use crate::generated::ItemKind;
use legion::Entity;

///
pub struct InventorySystem;

impl InventorySystem {
    /// 특정 엔티티(주로 아이템)의 최종 무게를 계산 (재귀적)
    /// NetHack의 original weight() 로직 이식
    pub fn get_weight<S: legion::EntityStore>(world: &S, entity: Entity) -> u32 {
        let mut total_weight = 0;

        if let Ok(entry) = world.entry_ref(entity) {
            // 1. 아이템 자신의 무게 (quantity 반영)
            if let Ok(item) = entry.get_component::<Item>() {
                total_weight += item.weight * item.quantity;
            }

            //
            if entry
                .get_component::<crate::core::entity::ContainerTag>()
                .is_ok()
            {
                if let Ok(inv) = entry.get_component::<Inventory>() {
                    let mut content_weight = 0;
                    for &child in &inv.items {
                        content_weight += Self::get_weight(world, child);
                    }

                    // 3. 마법 가방(Bag of Holding) 효과 적용
                    if let Ok(props) = entry.get_component::<ContainerProperties>() {
                        if props.typ == ContainerType::BagOfHolding {
                            // NetHack BoH 공식:
                            //
                            // 일반인 경우: 1/2 (50%)
                            //
                            if let Ok(item) = entry.get_component::<Item>() {
                                if item.blessed {
                                    content_weight /= 4;
                                } else if item.cursed {
                                    content_weight *= 2;
                                } else {
                                    content_weight /= 2;
                                }
                            }
                        }
                    }
                    total_weight += content_weight;
                }
            }
        }

        total_weight
    }

    ///
    /// NetHack: BoH에 BoH를 넣거나, BoH에 Cancellation 완드를 넣을 때 발생
    pub fn check_boh_explosion<S: legion::EntityStore>(
        world: &S,
        container_ent: Entity,
        inserted_ent: Entity,
    ) -> bool {
        let container_is_boh = if let Ok(entry) = world.entry_ref(container_ent) {
            if let Ok(props) = entry.get_component::<ContainerProperties>() {
                props.typ == ContainerType::BagOfHolding
            } else {
                false
            }
        } else {
            false
        };

        let inserted_is_dangerous = if let Ok(entry) = world.entry_ref(inserted_ent) {
            // 1. 다른 가방 계열 (BoH, Bag of Tricks 등)
            if let Ok(props) = entry.get_component::<ContainerProperties>() {
                props.typ == ContainerType::BagOfHolding || props.typ == ContainerType::BagOfTricks
            } else {
                //
                if let Ok(item) = entry.get_component::<Item>() {
                    item.kind == ItemKind::WandOfCancellation
                } else {
                    false
                }
            }
        } else {
            false
        };

        container_is_boh && inserted_is_dangerous
    }
}

///
#[legion::system]
#[read_component(Inventory)]
pub fn autopickup_tick(_world: &mut legion::world::SubWorld) {
    //
}

///
#[legion::system]
#[read_component(Inventory)]
pub fn inventory_action(_world: &mut legion::world::SubWorld) {
    //
}

// =============================================================================
// [v2.3.1
// 원본: nethack-3.6.7/src/invent.c (4,161줄)
// =============================================================================

///
///
pub struct InventoryLetters {
    /// 사용 중인 레터
    used: [bool; 52],
}

impl InventoryLetters {
    pub fn new() -> Self {
        Self { used: [false; 52] }
    }

    ///
    fn letter_to_idx(letter: char) -> Option<usize> {
        match letter {
            'a'..='z' => Some((letter as u8 - b'a') as usize),
            'A'..='Z' => Some((letter as u8 - b'A') as usize + 26),
            _ => None,
        }
    }

    ///
    fn idx_to_letter(idx: usize) -> char {
        if idx < 26 {
            (b'a' + idx as u8) as char
        } else {
            (b'A' + (idx - 26) as u8) as char
        }
    }

    /// 사용 가능한 다음 레터 할당 (원본: assigninvlet)
    pub fn assign(&mut self) -> Option<char> {
        // 먼저 소문자, 그 다음 대문자
        for i in 0..52 {
            if !self.used[i] {
                self.used[i] = true;
                return Some(Self::idx_to_letter(i));
            }
        }
        None // 인벤토리 가득 참
    }

    /// 특정 레터 할당
    pub fn assign_specific(&mut self, letter: char) -> bool {
        if let Some(idx) = Self::letter_to_idx(letter) {
            if !self.used[idx] {
                self.used[idx] = true;
                return true;
            }
        }
        false
    }

    /// 레터 해제 (아이템 제거 시)
    pub fn release(&mut self, letter: char) {
        if let Some(idx) = Self::letter_to_idx(letter) {
            self.used[idx] = false;
        }
    }

    /// 남은 슬롯 수
    pub fn available_count(&self) -> usize {
        self.used.iter().filter(|&&u| !u).count()
    }

    ///
    pub fn is_full(&self) -> bool {
        self.available_count() == 0
    }
}

/// [v2.3.1] 아이템 분류 순서 (원본: invent.c class_order / let_to_name)
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemClass {
    Gold = 0,      // 금화
    Amulet = 1,    // 목걸이
    Weapon = 2,    // 무기
    Armor = 3,     // 방어구
    Ring = 4,      // 반지
    Tool = 5,      // 도구
    Food = 6,      // 음식
    Potion = 7,    // 물약
    Scroll = 8,    // 두루마리
    Spellbook = 9, // 주문서
    Wand = 10,     // 지팡이
    Gem = 11,      // 보석
    Rock = 12,     // 바위
    Chain = 13,    // 사슬
    Ball = 14,     // 공
    Other = 15,    // 기타
}

/// [v2.3.1] 아이템 클래스 이름 (원본: def_oc_syms)
pub fn item_class_name(class: ItemClass) -> &'static str {
    match class {
        ItemClass::Gold => "Coins",
        ItemClass::Amulet => "Amulets",
        ItemClass::Weapon => "Weapons",
        ItemClass::Armor => "Armor",
        ItemClass::Ring => "Rings",
        ItemClass::Tool => "Tools",
        ItemClass::Food => "Comestibles",
        ItemClass::Potion => "Potions",
        ItemClass::Scroll => "Scrolls",
        ItemClass::Spellbook => "Spellbooks",
        ItemClass::Wand => "Wands",
        ItemClass::Gem => "Gems/Stones",
        ItemClass::Rock => "Large Stones",
        ItemClass::Chain => "Chains",
        ItemClass::Ball => "Iron balls",
        ItemClass::Other => "Miscellaneous",
    }
}

/// [v2.3.1] 클래스 심볼 → ItemClass (원본: oclass)
pub fn symbol_to_class(sym: char) -> ItemClass {
    match sym {
        '$' => ItemClass::Gold,
        '"' => ItemClass::Amulet,
        ')' => ItemClass::Weapon,
        '[' => ItemClass::Armor,
        '=' => ItemClass::Ring,
        '(' => ItemClass::Tool,
        '%' => ItemClass::Food,
        '!' => ItemClass::Potion,
        '?' => ItemClass::Scroll,
        '+' => ItemClass::Spellbook,
        '/' => ItemClass::Wand,
        '*' => ItemClass::Gem,
        '`' => ItemClass::Rock,
        _ => ItemClass::Other,
    }
}

///
#[derive(Debug, Clone)]
pub struct InvSortKey {
    /// 분류 순서
    pub class: ItemClass,
    /// BUC 순서: blessed=0, uncursed=1, cursed=2, unknown=3
    pub buc_order: u8,
    /// 이름 (소팅용)
    pub name: String,
    /// 수량 (역순: 많은 것이 위)
    pub quantity: i32,
}

impl InvSortKey {
    pub fn new(
        class: ItemClass,
        blessed: bool,
        cursed: bool,
        known: bool,
        name: &str,
        qty: i32,
    ) -> Self {
        let buc_order = if !known {
            3
        } else if blessed {
            0
        } else if cursed {
            2
        } else {
            1
        };
        Self {
            class,
            buc_order,
            name: name.to_lowercase(),
            quantity: qty,
        }
    }
}

/// [v2.3.1] 짐 무게 상태 (원본: invent.c near_capacity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BurdenStatus {
    Unencumbered = 0,
    Burdened = 1,
    Stressed = 2,
    Strained = 3,
    Overtaxed = 4,
    Overloaded = 5,
}

/// [v2.3.1] 짐 상태 판정 (원본: near_capacity/calc_capacity)
/// weight_carried: 현재 짐 무게, max_capacity: 최대 적재량
pub fn burden_status(weight_carried: u32, max_capacity: u32) -> BurdenStatus {
    if max_capacity == 0 {
        return BurdenStatus::Overloaded;
    }
    let ratio = (weight_carried as f32 / max_capacity as f32 * 100.0) as u32;
    match ratio {
        0..=50 => BurdenStatus::Unencumbered,
        51..=66 => BurdenStatus::Burdened,
        67..=80 => BurdenStatus::Stressed,
        81..=90 => BurdenStatus::Strained,
        91..=100 => BurdenStatus::Overtaxed,
        _ => BurdenStatus::Overloaded,
    }
}

/// [v2.3.1] 짐 상태 이름 (원본: enc_stat[])
pub fn burden_name(status: BurdenStatus) -> &'static str {
    match status {
        BurdenStatus::Unencumbered => "",
        BurdenStatus::Burdened => "Burdened",
        BurdenStatus::Stressed => "Stressed",
        BurdenStatus::Strained => "Strained",
        BurdenStatus::Overtaxed => "Overtaxed",
        BurdenStatus::Overloaded => "Overloaded",
    }
}

/// [v2.3.1] 짐 상태에 따른 속도 감소 (원본: encumber_msg)
pub fn burden_speed_penalty(status: BurdenStatus) -> i32 {
    match status {
        BurdenStatus::Unencumbered => 0,
        BurdenStatus::Burdened => -1,
        BurdenStatus::Stressed => -3,
        BurdenStatus::Strained => -5,
        BurdenStatus::Overtaxed => -7,
        BurdenStatus::Overloaded => -9,
    }
}

/// [v2.3.1] 최대 적재량 계산 (원본: calc_capacity → max_cap)
/// strength: 힘 수치, constitution: 체력 수치
pub fn calc_max_capacity(strength: i32, constitution: i32) -> u32 {
    // 원본: 25 * (str + con/2) + 50
    let base = 25 * (strength + constitution / 2) + 50;
    base.max(100) as u32
}

/// [v2.3.1] 아이템 스택 가능 여부 (원본: mergable)
pub fn can_merge(
    name1: &str,
    name2: &str,
    blessed1: bool,
    blessed2: bool,
    cursed1: bool,
    cursed2: bool,
    enchant1: i32,
    enchant2: i32,
) -> bool {
    name1 == name2 && blessed1 == blessed2 && cursed1 == cursed2 && enchant1 == enchant2
}

/// [v2.3.1] 아이템 수 카운트 (원본: inv_cnt)
///
pub fn inventory_count(quantities: &[i32]) -> i32 {
    quantities.iter().sum()
}

///
pub fn inventory_header(total_items: i32, total_weight: u32, max_cap: u32) -> String {
    let status = burden_status(total_weight, max_cap);
    let status_str = burden_name(status);
    let status_part = if status_str.is_empty() {
        String::new()
    } else {
        format!(" [{}]", status_str)
    };
    format!(
        "Inventory ({} item{}, {}/{} wt){}",
        total_items,
        if total_items != 1 { "s" } else { "" },
        total_weight,
        max_cap,
        status_part,
    )
}

///
/// 반환: 금화 표시 문자열
pub fn gold_display(amount: i64) -> String {
    if amount == 0 {
        "No gold pieces.".to_string()
    } else if amount == 1 {
        "1 gold piece.".to_string()
    } else {
        format!("{} gold pieces.", amount)
    }
}

/// [v2.3.1] BUC 상태 표시 문자열 (원본: doname → buc)
pub fn buc_label(blessed: bool, cursed: bool, known: bool) -> &'static str {
    if !known {
        ""
    } else if blessed {
        "blessed"
    } else if cursed {
        "cursed"
    } else {
        "uncursed"
    }
}

/// [v2.3.1] 강화 수치 표시 (원본: doname → + or -)
pub fn enchantment_label(enchantment: i32) -> String {
    if enchantment >= 0 {
        format!("+{}", enchantment)
    } else {
        format!("{}", enchantment)
    }
}
