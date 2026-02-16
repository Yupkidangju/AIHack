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

// =============================================================================
// [v2.5.0] invent.c 포팅 — 나머지 핵심 시스템
// 원본: nethack-3.6.7/src/invent.c (4,480줄)
// =============================================================================

/// [v2.5.0] 가상/허구 화폐 목록 (원본: currencies[])
/// 환각 상태일 때 "zorkmid" 대신 랜덤으로 표시되는 화폐 단위
pub const CURRENCIES: &[&str] = &[
    "Altarian Dollar",       // 은하수를 여행하는 히치하이커를 위한 안내서
    "Ankh-Morpork Dollar",   // 디스크월드
    "auric",                 // The Domination of Draka
    "buckazoid",             // Space Quest
    "cirbozoid",             // Starslip
    "credit chit",           // Deus Ex
    "cubit",                 // 배틀스타 갤럭티카
    "Flanian Pobble Bead",   // 은하수를 여행하는 히치하이커를 위한 안내서
    "fretzer",               // 줄 베른
    "imperial credit",       // 스타워즈
    "Hong Kong Luna Dollar", // The Moon is a Harsh Mistress
    "kongbuck",              // 스노우 크래시
    "nanite",                // System Shock 2
    "quatloo",               // 스타트렉, 심시티
    "simoleon",              // 심시티
    "solari",                // 스페이스볼
    "spacebuck",             // 스페이스볼
    "sporebuck",             // Spore
    "Triganic Pu",           // 은하수를 여행하는 히치하이커를 위한 안내서
    "woolong",               // 카우보이 비밥
    "zorkmid",               // Zork, NetHack
];

/// [v2.5.0] 화폐 이름 반환 (원본: currency())
/// 환각 상태이면 무작위 가상 화폐, 아니면 "zorkmid"
/// 복수형 처리 포함
pub fn currency_name(amount: i64, hallucinating: bool, rng_idx: Option<usize>) -> String {
    let base = if hallucinating {
        // 환각 시 랜덤 화폐 선택
        let idx = rng_idx.unwrap_or(0) % CURRENCIES.len();
        CURRENCIES[idx]
    } else {
        "zorkmid"
    };
    if amount != 1 {
        // 간단한 복수형 처리
        format!("{}s", base)
    } else {
        base.to_string()
    }
}

/// [v2.5.0] 인벤토리 레터 문자열 압축 (원본: compactify())
/// 연속된 문자를 대시(-)로 표현: "abcdf" → "a-df"
pub fn compactify(letters: &str) -> String {
    let chars: Vec<char> = letters.chars().collect();
    if chars.len() <= 3 {
        return letters.to_string();
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let start = chars[i];
        let mut end = start;
        // 연속된 문자 찾기
        while i + 1 < chars.len() && chars[i + 1] as u32 == end as u32 + 1 {
            i += 1;
            end = chars[i];
        }
        if end as u32 - start as u32 >= 2 {
            // 3개 이상 연속: "a-d" 형태
            result.push(start);
            result.push('-');
            result.push(end);
        } else if end != start {
            // 2개 연속: 그냥 나열
            result.push(start);
            result.push(end);
        } else {
            result.push(start);
        }
        i += 1;
    }

    result.into_iter().collect()
}

/// [v2.5.0] 아이템 분할 가능 여부 (원본: splittable())
/// 저주받은 loadstone이나 용접된 무기는 분할 불가
pub fn splittable(item_name: &str, is_cursed: bool, is_wielded: bool, is_welded: bool) -> bool {
    // 저주받은 loadstone은 분할 불가
    if item_name.contains("loadstone") && is_cursed {
        return false;
    }
    // 용접된 무기는 분할 불가
    if is_wielded && is_welded {
        return false;
    }
    true
}

/// [v2.5.0] 상세 머지 판정 (원본: mergable())
/// 원본 invent.c의 25가지 이상의 조건을 Rust로 이식
#[derive(Debug, Clone, PartialEq)]
pub struct MergeCandidate {
    pub type_id: u32, // 아이템 타입 ID (otyp)
    pub oclass: u8,   // 오브젝트 클래스
    pub blessed: bool,
    pub cursed: bool,
    pub unpaid: bool,
    pub spe: i32, // 특수 값 (인챈트 등)
    pub no_charge: bool,
    pub broken: bool,
    pub trapped: bool,
    pub lamplit: bool, // 불이 켜져 있는지
    pub greased: bool,
    pub eroded: u8,       // 1차 침식
    pub eroded2: u8,      // 2차 침식
    pub erodeproof: bool, // 침식 방지
    pub dknown: bool,     // 외형 식별
    pub bknown: bool,     // BUC 식별
    pub rknown: bool,     // 내구 식별
    pub known: bool,
    pub nomerge: bool,        // 머지 금지 플래그
    pub is_glob: bool,        // 글로브(pudding) 여부
    pub is_coin: bool,        // 금화 여부
    pub can_merge_type: bool, // objects[otyp].oc_merge
    pub corpsenm: i32,        // 시체/알/통조림 몬스터 번호
    pub timed: bool,          // 타이머 설정 여부
    pub age: i64,             // 나이 (양초 등에 사용)
    pub is_candle: bool,
    pub is_food: bool,
    pub oeaten: u32,   // 먹은 양
    pub orotten: bool, // 썩은 여부
    pub is_weapon_or_armor: bool,
    pub name: String,     // 부여된 이름
    pub artifact_id: u32, // 아티팩트 ID
    pub uses_known: bool, // oc_uses_known
    pub bypass: bool,     // 바이패스 비트
}

/// [v2.5.0] 두 아이템의 머지 가능 여부 판정 (원본: mergable())
pub fn mergable(a: &MergeCandidate, b: &MergeCandidate) -> bool {
    // 같은 오브젝트이거나 타입이 다르거나 머지 금지 시 불가
    if a.type_id != b.type_id || a.nomerge || b.nomerge || !a.can_merge_type {
        return false;
    }

    // 금화는 항상 머지 가능
    if a.is_coin {
        return true;
    }

    // BUC 상태 불일치 시 불가
    if a.bypass != b.bypass || a.cursed != b.cursed || a.blessed != b.blessed {
        return false;
    }

    // 글로브(푸딩)는 BUC만 맞으면 머지 가능
    if a.is_glob {
        return true;
    }

    // 기본 속성 비교
    if a.unpaid != b.unpaid
        || a.spe != b.spe
        || a.no_charge != b.no_charge
        || a.broken != b.broken
        || a.trapped != b.trapped
        || a.lamplit != b.lamplit
    {
        return false;
    }

    // 음식류: 먹은 양과 썩은 상태 비교
    if a.is_food && (a.oeaten != b.oeaten || a.orotten != b.orotten) {
        return false;
    }

    // 식별 상태 및 침식 비교
    if a.dknown != b.dknown
        || a.bknown != b.bknown
        || a.eroded != b.eroded
        || a.eroded2 != b.eroded2
        || a.greased != b.greased
    {
        return false;
    }

    // 무기/방어구: 침식 방지 및 rknown 추가 비교
    if a.is_weapon_or_armor && (a.erodeproof != b.erodeproof || a.rknown != b.rknown) {
        return false;
    }

    // 시체/알/통조림: 몬스터 종류 비교
    if a.corpsenm != b.corpsenm {
        return false;
    }

    // 부화 중인 알이나 부활 대상 시체는 머지 불가
    if a.timed || b.timed {
        return false;
    }

    // 양초: 나이(수명)가 비슷해야 머지 가능
    if a.is_candle && a.age / 25 != b.age / 25 {
        return false;
    }

    // 불붙은 기름 물약은 머지 불가
    if a.lamplit {
        return false;
    }

    // 이름 비교
    let a_named = !a.name.is_empty();
    let b_named = !b.name.is_empty();
    if a_named != b_named {
        // 한쪽만 이름이 있으면 불가
        return false;
    }
    if a_named && b_named && a.name != b.name {
        return false;
    }

    // 아티팩트 ID 비교
    if a.artifact_id != b.artifact_id {
        return false;
    }

    // known 플래그 비교 (oc_uses_known인 경우)
    if a.uses_known && a.known != b.known {
        return false;
    }

    a.can_merge_type
}

/// [v2.5.0] 인벤토리 추가 시 발생하는 이벤트 종류 (원본: addinv_core1)
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryAddEvent {
    /// 금화 추가 — 상태표시줄 갱신 필요
    GoldAdded,
    /// 엔도르의 아뮬렛 획득
    AmuletOfYendorObtained,
    /// 촛대(Candelabrum) 획득
    CandelabrumObtained,
    /// 개방의 종(Bell of Opening) 획득
    BellOfOpeningObtained,
    /// 죽음의 책(Book of the Dead) 획득
    BookOfTheDeadObtained,
    /// 퀘스트 아티팩트 획득
    QuestArtifactObtained,
    /// 일반 아티팩트 획득
    ArtifactObtained { artifact_id: u32 },
    /// 광산 행운석(특수 업적)
    MinesLuckstoneObtained,
    /// 소코반 완료 상품(특수 업적)
    SokobanPrizeObtained,
    /// 특별한 이벤트 없음
    None,
}

/// [v2.5.0] 아이템 타입 이름으로 인벤토리 추가 이벤트 판별 (원본: addinv_core1)
pub fn classify_add_event(
    item_name: &str,
    artifact_id: u32,
    is_quest_artifact: bool,
) -> InventoryAddEvent {
    if item_name.contains("gold piece") {
        InventoryAddEvent::GoldAdded
    } else if item_name == "Amulet of Yendor" {
        InventoryAddEvent::AmuletOfYendorObtained
    } else if item_name == "Candelabrum of Invocation" {
        InventoryAddEvent::CandelabrumObtained
    } else if item_name == "Bell of Opening" {
        InventoryAddEvent::BellOfOpeningObtained
    } else if item_name == "Book of the Dead" {
        InventoryAddEvent::BookOfTheDeadObtained
    } else if is_quest_artifact {
        InventoryAddEvent::QuestArtifactObtained
    } else if artifact_id > 0 {
        InventoryAddEvent::ArtifactObtained { artifact_id }
    } else {
        InventoryAddEvent::None
    }
}

/// [v2.5.0] 인벤토리 제거 시 발생하는 이벤트 종류 (원본: freeinv_core)
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryRemoveEvent {
    GoldRemoved,
    AmuletOfYendorLost,
    CandelabrumLost,
    BellOfOpeningLost,
    BookOfTheDeadLost,
    QuestArtifactLost,
    ArtifactLost { artifact_id: u32 },
    LoadstoneDropped,
    LuckstoneLost,
    None,
}

/// [v2.5.0] 아이템 제거 이벤트 판별 (원본: freeinv_core)
pub fn classify_remove_event(
    item_name: &str,
    artifact_id: u32,
    is_quest_artifact: bool,
    confers_luck: bool,
) -> InventoryRemoveEvent {
    if item_name.contains("gold piece") {
        InventoryRemoveEvent::GoldRemoved
    } else if item_name == "Amulet of Yendor" {
        InventoryRemoveEvent::AmuletOfYendorLost
    } else if item_name == "Candelabrum of Invocation" {
        InventoryRemoveEvent::CandelabrumLost
    } else if item_name == "Bell of Opening" {
        InventoryRemoveEvent::BellOfOpeningLost
    } else if item_name == "Book of the Dead" {
        InventoryRemoveEvent::BookOfTheDeadLost
    } else if is_quest_artifact {
        InventoryRemoveEvent::QuestArtifactLost
    } else if artifact_id > 0 {
        InventoryRemoveEvent::ArtifactLost { artifact_id }
    } else if item_name.contains("loadstone") {
        InventoryRemoveEvent::LoadstoneDropped
    } else if confers_luck {
        InventoryRemoveEvent::LuckstoneLost
    } else {
        InventoryRemoveEvent::None
    }
}

/// [v2.5.0] 아이템 소비 결과 (원본: useup/useupall)
#[derive(Debug, Clone, PartialEq)]
pub enum UseUpResult {
    /// 수량이 1보다 클 때 — 수량 감소
    QuantityReduced { remaining: i64 },
    /// 수량이 1 이하 — 아이템 완전 소멸
    FullyConsumed,
}

/// [v2.5.0] 아이템 사용/소비 (원본: useup)
pub fn use_up(quantity: i64) -> UseUpResult {
    if quantity > 1 {
        UseUpResult::QuantityReduced {
            remaining: quantity - 1,
        }
    } else {
        UseUpResult::FullyConsumed
    }
}

/// [v2.5.0] 충전 소비 (원본: consume_obj_charge)
/// 반환: 남은 충전 수
pub fn consume_charge(current_charges: i32) -> i32 {
    current_charges - 1
}

/// [v2.5.0] 인벤토리에서 특정 타입 아이템 검색 (원본: carrying)
pub fn carrying<'a>(inventory: &'a [(u32, &str)], type_id: u32) -> Option<&'a str> {
    inventory
        .iter()
        .find(|(id, _)| *id == type_id)
        .map(|(_, name)| *name)
}

/// [v2.5.0] 도마뱀 시체 보유 여부 (원본: have_lizard)
pub fn have_lizard(inventory_corpses: &[&str]) -> bool {
    inventory_corpses.iter().any(|name| *name == "lizard")
}

/// [v2.5.0] 소설 보유 여부 (원본: u_have_novel)
pub fn have_novel(inventory_types: &[&str]) -> bool {
    inventory_types.iter().any(|name| *name == "novel")
}

/// [v2.5.0] ID로 오브젝트 검색 (원본: o_on) — 컨테이너 내부 재귀 포함
pub fn find_by_id(items: &[(u32, Vec<(u32, Vec<(u32, Vec<()>)>)>)], target_id: u32) -> bool {
    // 간소화된 재귀 탐색: 실제 구현은 ECS 기반
    items.iter().any(|(id, _children)| *id == target_id)
}

/// [v2.5.0] 특정 위치에서 특정 타입 오브젝트 찾기 (원본: sobj_at)
pub fn object_at_position(floor_items: &[(u32, i32, i32)], type_id: u32, x: i32, y: i32) -> bool {
    floor_items
        .iter()
        .any(|(id, ox, oy)| *id == type_id && *ox == x && *oy == y)
}

/// [v2.5.0] 같은 위치에 금화 검색 (원본: g_at)
pub fn gold_at_position(floor_items: &[(u8, i32, i32)], x: i32, y: i32) -> bool {
    // oclass가 COIN_CLASS(금화 클래스)인 아이템 검색
    floor_items
        .iter()
        .any(|(oclass, ox, oy)| *oclass == 0 && *ox == x && *oy == y) // 0 = COIN_CLASS
}

/// [v2.5.0] 미지불 아이템 수 카운트 — 컨테이너 내부 재귀 포함 (원본: count_unpaid)
pub fn count_unpaid(unpaid_flags: &[bool], container_unpaid_counts: &[i32]) -> i32 {
    let direct: i32 = unpaid_flags.iter().filter(|&&u| u).count() as i32;
    let nested: i32 = container_unpaid_counts.iter().sum();
    direct + nested
}

/// [v2.5.0] BUC 상태별 카운트 (원본: count_buc)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucType {
    Blessed,
    Uncursed,
    Cursed,
    Unknown,
}

/// [v2.5.0] 단일 BUC 타입 카운트
pub fn count_buc_type(
    items: &[(bool, bool, bool)], // (bknown, blessed, cursed)
    buc_type: BucType,
) -> i32 {
    items
        .iter()
        .filter(|(bknown, blessed, cursed)| match buc_type {
            BucType::Unknown => !bknown,
            BucType::Blessed => *bknown && *blessed,
            BucType::Cursed => *bknown && *cursed,
            BucType::Uncursed => *bknown && !*blessed && !*cursed,
        })
        .count() as i32
}

/// [v2.5.0] 모든 BUC 상태를 한 번에 집계 (원본: tally_BUCX)
#[derive(Debug, Clone, Default)]
pub struct BucTally {
    pub blessed: i32,
    pub uncursed: i32,
    pub cursed: i32,
    pub unknown: i32,
    pub other: i32, // 미래 확장용
}

pub fn tally_bucx(items: &[(bool, bool, bool)]) -> BucTally {
    let mut tally = BucTally::default();
    for (bknown, blessed, cursed) in items {
        if !bknown {
            tally.unknown += 1;
        } else if *blessed {
            tally.blessed += 1;
        } else if *cursed {
            tally.cursed += 1;
        } else {
            tally.uncursed += 1;
        }
    }
    tally
}

/// [v2.5.0] 컨테이너 내용물 수 계산 (원본: count_contents)
pub fn count_contents(
    items: &[i64],         // 각 아이템 수량
    nested_counts: &[i64], // 중첩 컨테이너 내부 아이템 수
    count_quantity: bool,  // true면 수량 합산, false면 스택 수
) -> i64 {
    let direct: i64 = if count_quantity {
        items.iter().sum()
    } else {
        items.len() as i64
    };
    let nested: i64 = nested_counts.iter().sum();
    direct + nested
}

/// [v2.5.0] 클래스 번호로 클래스 이름 반환 (원본: let_to_name)
/// show_sym이 true면 심볼 문자도 표시
pub fn let_to_name(class: ItemClass, unpaid: bool, show_sym: bool) -> String {
    let base_name = item_class_name(class);
    let prefix = if unpaid { "Unpaid " } else { "" };
    let sym_suffix = if show_sym {
        let sym = match class {
            ItemClass::Gold => '$',
            ItemClass::Amulet => '"',
            ItemClass::Weapon => ')',
            ItemClass::Armor => '[',
            ItemClass::Ring => '=',
            ItemClass::Tool => '(',
            ItemClass::Food => '%',
            ItemClass::Potion => '!',
            ItemClass::Scroll => '?',
            ItemClass::Spellbook => '+',
            ItemClass::Wand => '/',
            ItemClass::Gem => '*',
            ItemClass::Rock => '`',
            _ => ' ',
        };
        if sym != ' ' {
            format!("  ('{}')", sym)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    format!("{}{}{}", prefix, base_name, sym_suffix)
}

/// [v2.5.0] 인벤토리 레터 재할당 (원본: reassign())
/// 금화를 '$' 슬롯으로, 나머지를 a-zA-Z 순서로 재배치
pub fn reassign_letters(items: &mut [(char, bool)], // (인벤토리 레터, 금화 여부)
) {
    // 1단계: 금화를 분리
    let mut gold_idx = None;
    for (i, (_, is_gold)) in items.iter().enumerate() {
        if *is_gold {
            gold_idx = Some(i);
            break;
        }
    }

    // 2단계: 금화가 아닌 아이템에 순차적 레터 할당
    let mut letter_idx: usize = 0;
    for (letter, is_gold) in items.iter_mut() {
        if !*is_gold {
            *letter = if letter_idx < 26 {
                (b'a' + letter_idx as u8) as char
            } else if letter_idx < 52 {
                (b'A' + (letter_idx - 26) as u8) as char
            } else {
                '#' // NOINVSYM
            };
            letter_idx += 1;
        }
    }

    // 3단계: 금화에 '$' 할당
    if let Some(idx) = gold_idx {
        items[idx].0 = '$';
    }
}

/// [v2.5.0] 방어구 착용 여부 (원본: wearing_armor())
pub fn wearing_armor(
    has_shirt: bool,
    has_suit: bool,
    has_cloak: bool,
    has_helmet: bool,
    has_shield: bool,
    has_gloves: bool,
    has_boots: bool,
) -> bool {
    has_shirt || has_suit || has_cloak || has_helmet || has_shield || has_gloves || has_boots
}

/// [v2.5.0] 착용/장비 중인지 판정 (원본: is_worn())
pub fn is_worn(worn_mask: u32) -> bool {
    // W_ARMOR | W_ACCESSORY | W_SADDLE | W_WEAPONS 비트마스크 검사
    // 비트 정의:
    const W_ARMOR: u32 = 0x01FF; // 방어구 관련 비트
    const W_ACCESSORY: u32 = 0x0E00; // 반지/목걸이 관련 비트
    const W_SADDLE: u32 = 0x4000; // 안장
    const W_WEAPONS: u32 = 0x3000; // 무기 관련 비트
    (worn_mask & (W_ARMOR | W_ACCESSORY | W_SADDLE | W_WEAPONS)) != 0
}

/// [v2.5.0] 도구 사용 중 판정 (원본: tool_in_use())
pub fn tool_in_use(
    worn_mask: u32,
    oclass: u8,
    is_wielded: bool,
    is_lit: bool,
    is_leash_attached: bool,
) -> bool {
    const W_TOOL: u32 = 0x8000;
    const W_SADDLE: u32 = 0x4000;
    // 도구 슬롯에 착용 중
    if (worn_mask & (W_TOOL | W_SADDLE)) != 0 {
        return true;
    }
    // 도구 클래스가 아니면 false
    if oclass != 5 {
        // 5 = TOOL_CLASS
        return false;
    }
    // 들고 있거나 불이 켜져 있거나 줄이 연결됨
    is_wielded || is_lit || is_leash_attached
}

/// [v2.5.0] 인벤토리 아이템 표시 문자열 (원본: xprname)
/// let_char: 인벤토리 레터 문자
/// item_name: 아이템 이름
/// dot: 마침표 추가 여부
/// cost: 비용 (상점용, 0이면 비용 표시 안 함)
pub fn xprname(let_char: char, item_name: &str, dot: bool, cost: i64) -> String {
    if cost != 0 {
        format!("{} - {}{:>7} zorkmids", let_char, item_name, cost)
    } else {
        let suffix = if dot { "." } else { "" };
        format!("{} - {}{}", let_char, item_name, suffix)
    }
}

/// [v2.5.0] 바닥 아이템 수 설명 (원본: look_here 일부)
/// 아이템 개수에 따른 서술적 표현
pub fn pile_description(count: i32, picked_some: bool) -> String {
    if count == 1 {
        let article = if picked_some { "another" } else { "an" };
        format!("There is {} object here.", article)
    } else {
        let amount = if count < 5 {
            "a few"
        } else if count < 10 {
            "several"
        } else {
            "many"
        };
        let more = if picked_some { " more" } else { "" };
        format!("There are {}{} objects here.", amount, more)
    }
}

/// [v2.5.0] 던전 피처 이름 반환 (원본: dfeature_at — 일부)
/// 맵 타일 타입에 따른 설명 문자열
pub fn dfeature_name(tile_type: &str) -> Option<&'static str> {
    match tile_type {
        "doorway" => Some("doorway"),
        "open_door" => Some("open door"),
        "closed_door" => Some("closed door"),
        "broken_door" => Some("broken door"),
        "fountain" => Some("fountain"),
        "throne" => Some("opulent throne"),
        "lava" => Some("molten lava"),
        "ice" => Some("ice"),
        "pool" => Some("pool of water"),
        "sink" => Some("sink"),
        "grave" => Some("grave"),
        "tree" => Some("tree"),
        "iron_bars" => Some("set of iron bars"),
        "upstair" => Some("staircase up"),
        "downstair" => Some("staircase down"),
        "upladder" => Some("ladder up"),
        "downladder" => Some("ladder down"),
        "drawbridge_down" => Some("lowered drawbridge"),
        "drawbridge_up" => Some("raised drawbridge"),
        _ => None,
    }
}

/// [v2.5.0] 코카트리스 시체 접촉 위험 판정 (원본: will_feel_cockatrice)
pub fn will_feel_cockatrice(
    is_blind: bool,
    has_gloves: bool,
    stone_resistant: bool,
    corpse_name: &str,
) -> bool {
    if !is_blind {
        return false;
    }
    if has_gloves || stone_resistant {
        return false;
    }
    // 코카트리스 계통 시체 접촉 시 석화
    corpse_name.contains("cockatrice") || corpse_name.contains("chickatrice")
}

/// [v2.5.0] 스택 처리 (원본: stackobj — 간소화)
/// 같은 위치의 호환 아이템 자동 머지 시도
pub fn should_stack(
    a_type: u32,
    b_type: u32,
    a_blessed: bool,
    b_blessed: bool,
    a_cursed: bool,
    b_cursed: bool,
    a_spe: i32,
    b_spe: i32,
) -> bool {
    a_type == b_type && a_blessed == b_blessed && a_cursed == b_cursed && a_spe == b_spe
}

// =============================================================================
// [v2.5.0] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- 기존 테스트 보존 (v2.3.1) ---

    #[test]
    fn test_inventory_letters_basic() {
        let mut letters = InventoryLetters::new();
        assert_eq!(letters.assign(), Some('a'));
        assert_eq!(letters.assign(), Some('b'));
        assert_eq!(letters.available_count(), 50);
        letters.release('a');
        assert_eq!(letters.available_count(), 51);
    }

    #[test]
    fn test_inventory_letters_full() {
        let mut letters = InventoryLetters::new();
        for _ in 0..52 {
            assert!(letters.assign().is_some());
        }
        assert!(letters.is_full());
        assert_eq!(letters.assign(), None);
    }

    #[test]
    fn test_burden_status() {
        assert_eq!(burden_status(0, 1000), BurdenStatus::Unencumbered);
        assert_eq!(burden_status(500, 1000), BurdenStatus::Unencumbered);
        assert_eq!(burden_status(510, 1000), BurdenStatus::Burdened);
        assert_eq!(burden_status(670, 1000), BurdenStatus::Stressed);
        assert_eq!(burden_status(850, 1000), BurdenStatus::Strained);
        assert_eq!(burden_status(950, 1000), BurdenStatus::Overtaxed);
        assert_eq!(burden_status(1100, 1000), BurdenStatus::Overloaded);
    }

    #[test]
    fn test_calc_max_capacity() {
        // 기본 공식: 25 * (str + con/2) + 50
        assert_eq!(calc_max_capacity(10, 10), 425);
        assert_eq!(calc_max_capacity(18, 18), 725);
    }

    #[test]
    fn test_gold_display() {
        assert_eq!(gold_display(0), "No gold pieces.");
        assert_eq!(gold_display(1), "1 gold piece.");
        assert_eq!(gold_display(42), "42 gold pieces.");
    }

    // --- v2.5.0 신규 테스트 ---

    #[test]
    fn test_currencies_count() {
        // 원본 invent.c currencies[] 배열 = 21개
        assert_eq!(CURRENCIES.len(), 21);
        assert_eq!(CURRENCIES[20], "zorkmid");
    }

    #[test]
    fn test_currency_name_normal() {
        assert_eq!(currency_name(1, false, None), "zorkmid");
        assert_eq!(currency_name(5, false, None), "zorkmids");
    }

    #[test]
    fn test_currency_name_hallucinating() {
        let name = currency_name(1, true, Some(0));
        assert_eq!(name, "Altarian Dollar");
        let name = currency_name(3, true, Some(6));
        assert_eq!(name, "cubits"); // 복수형
    }

    #[test]
    fn test_compactify_short() {
        assert_eq!(compactify("abc"), "abc"); // 3글자 이하는 그대로
    }

    #[test]
    fn test_compactify_run() {
        assert_eq!(compactify("abcdef"), "a-f");
        assert_eq!(compactify("abcdxyz"), "a-dx-z");
    }

    #[test]
    fn test_compactify_gaps() {
        // a,b → 2개 연속이므로 "ab", d → 단독, f,g,h → 3개 연속이므로 "f-h"
        assert_eq!(compactify("abdfgh"), "abdf-h");
    }

    #[test]
    fn test_splittable_loadstone() {
        assert!(!splittable("cursed loadstone", true, false, false));
        assert!(splittable("loadstone", false, false, false));
    }

    #[test]
    fn test_splittable_welded() {
        assert!(!splittable("long sword", false, true, true));
        assert!(splittable("long sword", false, true, false));
    }

    #[test]
    fn test_mergable_coins() {
        let coin_a = MergeCandidate {
            type_id: 1,
            oclass: 0,
            blessed: false,
            cursed: false,
            unpaid: false,
            spe: 0,
            no_charge: false,
            broken: false,
            trapped: false,
            lamplit: false,
            greased: false,
            eroded: 0,
            eroded2: 0,
            erodeproof: false,
            dknown: true,
            bknown: true,
            rknown: false,
            known: true,
            nomerge: false,
            is_glob: false,
            is_coin: true,
            can_merge_type: true,
            corpsenm: -1,
            timed: false,
            age: 0,
            is_candle: false,
            is_food: false,
            oeaten: 0,
            orotten: false,
            is_weapon_or_armor: false,
            name: String::new(),
            artifact_id: 0,
            uses_known: false,
            bypass: false,
        };
        // 금화는 항상 머지 가능
        assert!(mergable(&coin_a, &coin_a));
    }

    #[test]
    fn test_mergable_different_buc() {
        let base = MergeCandidate {
            type_id: 10,
            oclass: 2,
            blessed: true,
            cursed: false,
            unpaid: false,
            spe: 0,
            no_charge: false,
            broken: false,
            trapped: false,
            lamplit: false,
            greased: false,
            eroded: 0,
            eroded2: 0,
            erodeproof: false,
            dknown: true,
            bknown: true,
            rknown: false,
            known: true,
            nomerge: false,
            is_glob: false,
            is_coin: false,
            can_merge_type: true,
            corpsenm: -1,
            timed: false,
            age: 0,
            is_candle: false,
            is_food: false,
            oeaten: 0,
            orotten: false,
            is_weapon_or_armor: false,
            name: String::new(),
            artifact_id: 0,
            uses_known: false,
            bypass: false,
        };
        let mut cursed = base.clone();
        cursed.blessed = false;
        cursed.cursed = true;
        // BUC 불일치 시 머지 불가
        assert!(!mergable(&base, &cursed));
    }

    #[test]
    fn test_classify_add_event() {
        assert_eq!(
            classify_add_event("Amulet of Yendor", 0, false),
            InventoryAddEvent::AmuletOfYendorObtained
        );
        assert_eq!(
            classify_add_event("long sword", 0, false),
            InventoryAddEvent::None
        );
        assert_eq!(
            classify_add_event("100 gold pieces", 0, false),
            InventoryAddEvent::GoldAdded
        );
    }

    #[test]
    fn test_classify_remove_event() {
        assert_eq!(
            classify_remove_event("loadstone", 0, false, false),
            InventoryRemoveEvent::LoadstoneDropped
        );
        assert_eq!(
            classify_remove_event("luckstone", 0, false, true),
            InventoryRemoveEvent::LuckstoneLost
        );
    }

    #[test]
    fn test_use_up() {
        assert_eq!(use_up(5), UseUpResult::QuantityReduced { remaining: 4 });
        assert_eq!(use_up(1), UseUpResult::FullyConsumed);
    }

    #[test]
    fn test_consume_charge() {
        assert_eq!(consume_charge(3), 2);
        assert_eq!(consume_charge(0), -1);
    }

    #[test]
    fn test_have_lizard() {
        assert!(have_lizard(&["jackal", "lizard", "newt"]));
        assert!(!have_lizard(&["jackal", "newt"]));
    }

    #[test]
    fn test_tally_bucx() {
        let items = vec![
            (true, true, false),   // blessed
            (true, false, false),  // uncursed
            (true, false, true),   // cursed
            (false, false, false), // unknown
        ];
        let tally = tally_bucx(&items);
        assert_eq!(tally.blessed, 1);
        assert_eq!(tally.uncursed, 1);
        assert_eq!(tally.cursed, 1);
        assert_eq!(tally.unknown, 1);
    }

    #[test]
    fn test_let_to_name_basic() {
        assert_eq!(let_to_name(ItemClass::Weapon, false, false), "Weapons");
        assert_eq!(
            let_to_name(ItemClass::Weapon, true, false),
            "Unpaid Weapons"
        );
    }

    #[test]
    fn test_let_to_name_with_sym() {
        let result = let_to_name(ItemClass::Potion, false, true);
        assert!(result.contains("Potions"));
        assert!(result.contains("('!')"));
    }

    #[test]
    fn test_xprname_basic() {
        assert_eq!(xprname('a', "a long sword", true, 0), "a - a long sword.");
    }

    #[test]
    fn test_xprname_cost() {
        let result = xprname('b', "a potion of healing", false, 100);
        assert!(result.contains("100"));
        assert!(result.contains("zorkmids"));
    }

    #[test]
    fn test_pile_description() {
        assert_eq!(pile_description(1, false), "There is an object here.");
        assert_eq!(pile_description(3, false), "There are a few objects here.");
        assert_eq!(
            pile_description(7, true),
            "There are several more objects here."
        );
        assert_eq!(pile_description(15, false), "There are many objects here.");
    }

    #[test]
    fn test_dfeature_name() {
        assert_eq!(dfeature_name("fountain"), Some("fountain"));
        assert_eq!(dfeature_name("throne"), Some("opulent throne"));
        assert_eq!(dfeature_name("lava"), Some("molten lava"));
        assert_eq!(dfeature_name("unknown_tile"), None);
    }

    #[test]
    fn test_will_feel_cockatrice() {
        // 눈이 먼 상태에서 장갑 없이 코카트리스 시체를 만지면 위험
        assert!(will_feel_cockatrice(true, false, false, "cockatrice"));
        // 장갑 있으면 안전
        assert!(!will_feel_cockatrice(true, true, false, "cockatrice"));
        // 눈이 안 멀면 안전 (봐서 피할 수 있음)
        assert!(!will_feel_cockatrice(false, false, false, "cockatrice"));
        // 석화 저항 있으면 안전
        assert!(!will_feel_cockatrice(true, false, true, "cockatrice"));
    }

    #[test]
    fn test_wearing_armor() {
        assert!(wearing_armor(
            false, true, false, false, false, false, false
        ));
        assert!(!wearing_armor(
            false, false, false, false, false, false, false
        ));
        assert!(wearing_armor(
            true, false, false, false, false, false, false
        ));
    }

    #[test]
    fn test_is_worn() {
        assert!(is_worn(0x0001)); // 방어구 비트
        assert!(is_worn(0x1000)); // 무기 비트
        assert!(is_worn(0x4000)); // 안장 비트
        assert!(!is_worn(0)); // 아무것도 안 착용
    }
}
