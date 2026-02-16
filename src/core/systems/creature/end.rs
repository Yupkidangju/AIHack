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
    Poisoned, // POISONING
    Starved,  // STARVING
    Drowned,  // DROWNING
    Burned,
    Crushed,
    StonedToDeath,
    Slimed,    // SLIMING
    Strangled, // STRANGULATION
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
    Turned, // TURNED_TO_STONE
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
    pub vegetarian: bool, // 梨꾩떇
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
// [v2.8.0] end.c 대량 이식 — 사망/점수/가치품/구원/디스클로저
// 원본: nethack-3.6.7/src/end.c (2,293줄)
// =============================================================================

/// [v2.8.0] 사망 유형별 문자열 테이블 (원본: end.c:292-298 deaths[])
pub const DEATH_NAMES: &[&str] = &[
    "died",
    "choked",
    "poisoned",
    "starvation",
    "drowning",
    "burning",
    "dissolving under the heat and pressure",
    "crushed",
    "turned to stone",
    "turned into slime",
    "genocided",
    "panic",
    "trickery",
    "quit",
    "escaped",
    "ascended",
];

/// [v2.8.0] 사망 시 "when you %s" 형태 문자열 (원본: end.c:300-309 ends[])
pub const END_NAMES: &[&str] = &[
    "died",
    "choked",
    "were poisoned",
    "starved",
    "drowned",
    "burned",
    "dissolved in the lava",
    "were crushed",
    "turned to stone",
    "turned into slime",
    "were genocided",
    "panicked",
    "were tricked",
    "quit",
    "escaped",
    "ascended",
];

/// [v2.8.0] DeathType → 인덱스 매핑 (DEATH_NAMES/END_NAMES 접근용)
pub fn death_type_index(dt: DeathType) -> usize {
    match dt {
        DeathType::Killed => 0,
        DeathType::Strangled | DeathType::Suffocated => 1,
        DeathType::Poisoned | DeathType::FoodPoisoning => 2,
        DeathType::Starved => 3,
        DeathType::Drowned | DeathType::FellInWater => 4,
        DeathType::Burned => 5,
        DeathType::Crushed => 7,
        DeathType::StonedToDeath | DeathType::Petrified | DeathType::Turned => 8,
        DeathType::Slimed => 9,
        DeathType::Genocide => 10,
        DeathType::Panicked => 11,
        DeathType::Tricked => 12,
        DeathType::Quit => 13,
        DeathType::Escaped => 14,
        DeathType::Ascended => 15,
        _ => 0,
    }
}

/// [v2.8.0] 사망 원인에 몬스터 정보가 포함된 서술 생성
/// (원본: end.c:415-529 done_in_by)
pub fn done_in_by(
    monster_name: &str,
    is_unique: bool,
    is_invisible: bool,
    is_ghost: bool,
    ghost_of: Option<&str>,
    is_shopkeeper: bool,
    shopkeeper_name: Option<&str>,
    is_female: bool,
) -> String {
    let mut buf = String::new();

    // 유니크 몬스터: "the" 접두사
    if is_unique {
        buf.push_str("the ");
    }

    // 투명 몬스터
    if is_invisible {
        buf.push_str("invisible ");
    }

    // 유령 특수 처리
    if is_ghost {
        buf.push_str("ghost");
        if let Some(name) = ghost_of {
            buf.push_str(&format!(" of {}", name));
        }
    } else if is_shopkeeper {
        // 상점 주인 특수 처리
        let honorific = if is_female { "Ms. " } else { "Mr. " };
        let shk = shopkeeper_name.unwrap_or("shopkeeper");
        buf.push_str(&format!("{}{}, the shopkeeper", honorific, shk));
    } else {
        buf.push_str(monster_name);
    }

    buf
}

/// [v2.8.0] 사망 사유 보정 (원본: end.c:546-567 fixup_death)
/// "petrified by X, while getting stoned" 같은 중복 제거
pub fn fixup_death_reason(
    death_type: DeathType,
    helpless_reason: Option<&str>,
) -> Option<&'static str> {
    match (death_type, helpless_reason) {
        // 석화 사망 + "getting stoned" → 무용 이유 제거
        (DeathType::StonedToDeath, Some("getting stoned"))
        | (DeathType::Turned, Some("getting stoned"))
        | (DeathType::Petrified, Some("getting stoned")) => None,
        // "fainted from lack of food" → "fainted"로 축약
        (DeathType::Starved, Some("fainted from lack of food")) => Some("fainted"),
        _ => helpless_reason.map(|_| "while helpless"),
    }
}

/// [v2.8.0] 생명 구원 (원본: end.c:866-908 savelife)
/// 생명의 부적 또는 위저드 모드로 사망을 회피할 때 상태 복원
pub fn savelife_restore(
    current_hp: &mut i32,
    max_hp: &mut i32,
    level: i32,
    hunger: &mut i32,
    was_choking: bool,
) -> Vec<&'static str> {
    let mut messages = Vec::new();

    // HP 최소값: max(2*level, 10)
    let hp_min = (2 * level).max(10);
    if *max_hp < hp_min {
        *max_hp = hp_min;
    }
    *current_hp = *max_hp;

    // 기아 해소
    if *hunger < 500 || was_choking {
        *hunger = 900; // init_uhunger 대응
        messages.push("You feel nourished.");
    }

    if was_choking {
        messages.push("You vomit ...");
    }

    messages.push("But wait...");
    messages.push("Your medallion begins to glow!");
    messages.push("You feel much better!");
    messages.push("The medallion crumbles to dust!");
    messages.push("You survived that attempt on your life.");

    messages
}

/// [v2.8.0] 가치품 데이터 (원본: end.c:27-30 valuable_data)
#[derive(Debug, Clone, Default)]
pub struct ValuableData {
    pub count: i64,
    pub name: String,
    pub typ: i32,
}

/// [v2.8.0] 인벤토리에서 가치품(보석, 아뮬렛) 수집
/// (원본: end.c:914-943 get_valuables)
pub fn get_valuables(items: &[(String, i32, i64, bool)]) -> (Vec<ValuableData>, Vec<ValuableData>) {
    let mut gems: Vec<ValuableData> = Vec::new();
    let mut amulets: Vec<ValuableData> = Vec::new();

    for (name, typ, count, is_artifact) in items {
        if *is_artifact {
            continue; // 아티팩트 제외
        }
        // 보석 (typ 0-99 범위 가정)
        if *typ >= 0 && *typ < 100 {
            if let Some(existing) = gems.iter_mut().find(|g| g.typ == *typ) {
                existing.count += count;
            } else {
                gems.push(ValuableData {
                    count: *count,
                    name: name.clone(),
                    typ: *typ,
                });
            }
        }
        // 아뮬렛 (typ 200-299 범위 가정)
        else if *typ >= 200 && *typ < 300 {
            if let Some(existing) = amulets.iter_mut().find(|a| a.typ == *typ) {
                existing.count += count;
            } else {
                amulets.push(ValuableData {
                    count: *count,
                    name: name.clone(),
                    typ: *typ,
                });
            }
        }
    }

    // 빈도순 정렬 (원본: end.c:949-971 sort_valuables)
    gems.sort_by(|a, b| b.count.cmp(&a.count));
    amulets.sort_by(|a, b| b.count.cmp(&a.count));

    (gems, amulets)
}

/// [v2.8.0] 아티팩트 점수 계산 (원본: end.c:1061-1095 artifact_score)
/// 아티팩트/특수 아이템(종, 사자의 서, 촛대)의 점수
pub fn artifact_score(artifact_name: &str, artifact_value: i64) -> i64 {
    // 원본: points = value * 5 / 2
    let _ = artifact_name; // 이름은 표시용
    artifact_value * 5 / 2
}

/// [v2.8.0] 특수 아이템 점수 목록 생성
pub fn score_special_items(items: &[(String, bool, i64)]) -> Vec<(String, i64, i64)> {
    let mut results = Vec::new();
    let special_items = [
        "Bell of Opening",
        "Book of the Dead",
        "Candelabrum of Invocation",
        "Amulet of Yendor",
    ];

    for (name, is_artifact, value) in items {
        if *is_artifact || special_items.iter().any(|si| name.contains(si)) {
            let points = artifact_score(name, *value);
            results.push((name.clone(), *value, points));
        }
    }
    results
}

/// [v2.8.0] 점수 계산 확장 — 가치품/아티팩트 보정 포함
/// (원본: end.c:1097-1452 done() 내부 점수 계산 부분)
pub fn calculate_score_extended(
    player: &Player,
    death_info: &DeathInfo,
    max_depth: i32,
    carried_gold: u64,
    turns: u64,
    artifact_points: i64,
    amulet_of_yendor_carried: bool,
) -> i64 {
    let mut score = calculate_score(player, death_info, max_depth, carried_gold, turns);

    // 아티팩트 보너스
    score += artifact_points;

    // 아뮬렛 소지 보너스 (승천이 아닌 경우에도)
    if amulet_of_yendor_carried && death_info.death_type != DeathType::Ascended {
        score += 50000; // 원본: 아뮬렛 자체 점수
    }

    score.max(0)
}

/// [v2.8.0] 디스클로저 카테고리 (원본: end.c:793-863 disclose)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureCategory {
    /// 소지품 (원본: 'i')
    Inventory,
    /// 속성 (원본: 'a')
    Attributes,
    /// 처치 목록 (원본: 'v')
    Vanquished,
    /// 학살 목록 (원본: 'g')
    Genocided,
    /// 행동 규율 (원본: 'c')
    Conducts,
    /// 던전 개요 (원본: 'o')
    Overview,
}

/// [v2.8.0] 디스클로저 설정 (원본: end.c:653-694)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisclosureOption {
    /// 확인 없이 표시
    AlwaysShow,
    /// 확인 없이 숨김
    AlwaysHide,
    /// 확인 후 기본 예
    PromptDefaultYes,
    /// 확인 후 기본 아니오
    PromptDefaultNo,
}

/// [v2.8.0] 디스클로저 정보 구조체
#[derive(Debug, Clone)]
pub struct DisclosureInfo {
    pub categories: Vec<(DisclosureCategory, DisclosureOption)>,
}

impl Default for DisclosureInfo {
    fn default() -> Self {
        Self {
            categories: vec![
                (
                    DisclosureCategory::Inventory,
                    DisclosureOption::PromptDefaultYes,
                ),
                (
                    DisclosureCategory::Attributes,
                    DisclosureOption::PromptDefaultNo,
                ),
                (
                    DisclosureCategory::Vanquished,
                    DisclosureOption::PromptDefaultNo,
                ),
                (
                    DisclosureCategory::Genocided,
                    DisclosureOption::PromptDefaultNo,
                ),
                (
                    DisclosureCategory::Conducts,
                    DisclosureOption::PromptDefaultNo,
                ),
                (
                    DisclosureCategory::Overview,
                    DisclosureOption::PromptDefaultNo,
                ),
            ],
        }
    }
}

/// [v2.8.0] 무덤에서 부활할 몬스터 결정 (원본: end.c:513-525 ugrave_arise)
/// 사망 시 특정 몬스터에게 죽으면 언데드로 부활 가능
pub fn grave_arise_monster(killer_symbol: char, player_race: Race) -> Option<&'static str> {
    match killer_symbol {
        'W' => Some("wraith"), // S_WRAITH
        'M' => {
            // 미라 — 종족별 미라
            match player_race {
                Race::Human => Some("human mummy"),
                Race::Elf => Some("elf mummy"),
                Race::Dwarf => Some("dwarf mummy"),
                Race::Orc => Some("orc mummy"),
                Race::Gnome => Some("gnome mummy"),
            }
        }
        'V' => {
            // 뱀파이어 — 인간만
            if player_race == Race::Human {
                Some("vampire")
            } else {
                None
            }
        }
        'Z' => Some("ghoul"), // PM_GHOUL과 동일 분류
        _ => None,
    }
}

/// [v2.8.0] 멸종 몬스터 수 카운트 (원본: end.c:num_extinct 참조)
pub fn num_extinct(monster_counts: &[i32]) -> usize {
    monster_counts.iter().filter(|&&c| c == 0).count()
}

/// [v2.8.0] 사망 시각 포맷 (덤프용)
pub fn format_death_time(turns: u64) -> String {
    let hours = turns / 1000;
    let mins = (turns % 1000) / 17; // 대략적 변환
    format!("Turn {} (~{}h {}m)", turns, hours, mins)
}

/// [v2.8.0] 게임 결과 요약 문자열 생성 (덤프/하이스코어용)
pub fn game_result_summary(
    player_name: &str,
    role: PlayerClass,
    race: Race,
    alignment: Alignment,
    level: i32,
    death_info: &DeathInfo,
    score: i64,
) -> String {
    let rank = rank_of(role, level);
    let idx = death_type_index(death_info.death_type);
    let end_verb = if idx < END_NAMES.len() {
        END_NAMES[idx]
    } else {
        "died"
    };

    format!(
        "{} the {} {} {} {}, {} on level {} with {} points",
        player_name,
        alignment_name(alignment),
        race_name(race),
        rank,
        role_name(role),
        end_verb,
        death_info.depth,
        score,
    )
}

/// 정렬 이름 반환
fn alignment_name(a: Alignment) -> &'static str {
    match a {
        Alignment::Lawful => "Lawful",
        Alignment::Neutral => "Neutral",
        Alignment::Chaotic => "Chaotic",
    }
}

/// 종족 이름 반환
fn race_name(r: Race) -> &'static str {
    match r {
        Race::Human => "Human",
        Race::Elf => "Elven",
        Race::Dwarf => "Dwarven",
        Race::Orc => "Orcish",
        Race::Gnome => "Gnomish",
    }
}

/// 직업 이름 반환
fn role_name(r: PlayerClass) -> &'static str {
    match r {
        PlayerClass::Archeologist => "Archeologist",
        PlayerClass::Barbarian => "Barbarian",
        PlayerClass::Healer => "Healer",
        PlayerClass::Knight => "Knight",
        PlayerClass::Monk => "Monk",
        PlayerClass::Priest => "Priest",
        PlayerClass::Ranger => "Ranger",
        PlayerClass::Rogue => "Rogue",
        PlayerClass::Samurai => "Samurai",
        PlayerClass::Tourist => "Tourist",
        PlayerClass::Valkyrie => "Valkyrie",
        PlayerClass::Wizard => "Wizard",
    }
}

// =============================================================================
// [v2.8.0] Conduct 확장 메서드
// =============================================================================

impl Conduct {
    /// [v2.8.0] 소원 기록 (원본: u.uconduct.wishes)
    pub fn record_wish(&mut self) {
        self.wishes += 1;
    }

    /// [v2.8.0] 변신 기록 (원본: u.uconduct.polypiles)
    pub fn record_polymorph(&mut self) {
        self.polyselfless = false;
        self.polymorphless = false;
    }

    /// [v2.8.0] 학살 기록 (더 이상 genocideless하지 않음)
    pub fn record_genocide(&mut self) {
        self.genocideless = false;
    }

    /// [v2.8.0] 엘베레스 기록
    pub fn record_elbereth(&mut self) {
        self.elbereth_written += 1;
    }

    /// [v2.8.0] 무기 사용 기록
    pub fn record_weapon_use(&mut self) {
        self.weaponless = false;
    }

    /// [v2.8.0] 아티팩트 터치 기록
    pub fn record_artifact_touch(&mut self) {
        self.artifacts_touched += 1;
    }

    /// [v2.8.0] 채식 식사 기록 (비건 아닌 채식)
    pub fn record_eat_veggy(&mut self) {
        self.foodless = false;
        // 비건은 유지, 단 유제품/달걀이면 비건도 깨짐
    }

    /// [v2.8.0] 비건 위반 기록
    pub fn record_eat_non_vegan(&mut self) {
        self.foodless = false;
        self.vegan = false;
    }

    /// [v2.8.0] 전체 행동 카운트
    pub fn total_violations(&self) -> u32 {
        let mut count = 0u32;
        if !self.atheist {
            count += 1;
        }
        if !self.pacifist {
            count += 1;
        }
        if !self.illiterate {
            count += 1;
        }
        if !self.vegetarian {
            count += 1;
        }
        if !self.vegan {
            count += 1;
        }
        if !self.foodless {
            count += 1;
        }
        if !self.weaponless {
            count += 1;
        }
        if !self.genocideless {
            count += 1;
        }
        if !self.polyselfless {
            count += 1;
        }
        if self.wishes > 0 {
            count += 1;
        }
        count
    }

    /// [v2.8.0] 행동 규율 상세 요약
    pub fn detailed_summary(&self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.push(format!("Kills: {}", self.kills));
        lines.push(format!("Wishes: {}", self.wishes));
        lines.push(format!("Artifacts touched: {}", self.artifacts_touched));
        lines.push(format!("Elbereths written: {}", self.elbereth_written));
        let conducts = self.active_conducts();
        if conducts.is_empty() {
            lines.push("No special conducts maintained.".to_string());
        } else {
            lines.push(format!("Conducts maintained: {}", conducts.join(", ")));
        }
        lines
    }
}

// =============================================================================
// [v2.8.0] 하이스코어 보드 확장
// =============================================================================

impl HighScoreBoard {
    /// [v2.8.0] 특정 역할의 최고 점수
    pub fn best_for_role(&self, role: PlayerClass) -> Option<&TopTenEntry> {
        self.entries.iter().find(|e| e.role == role)
    }

    /// [v2.8.0] 특정 종족의 최고 점수
    pub fn best_for_race(&self, race: Race) -> Option<&TopTenEntry> {
        self.entries.iter().find(|e| e.race == race)
    }

    /// [v2.8.0] 전체 항목 수
    pub fn total_entries(&self) -> usize {
        self.entries.len()
    }

    /// [v2.8.0] 보드 비우기
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// [v2.8.0] 지정 점수 이상 항목 수
    pub fn entries_above_score(&self, min_score: i64) -> usize {
        self.entries.iter().filter(|e| e.score >= min_score).count()
    }
}

// =============================================================================
// 테스트
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

    // [v2.8.0] 신규 테스트

    #[test]
    fn test_death_names() {
        assert_eq!(DEATH_NAMES.len(), 16);
        assert_eq!(DEATH_NAMES[0], "died");
        assert_eq!(DEATH_NAMES[15], "ascended");
    }

    #[test]
    fn test_end_names() {
        assert_eq!(END_NAMES.len(), 16);
        assert_eq!(END_NAMES[14], "escaped");
    }

    #[test]
    fn test_death_type_index() {
        assert_eq!(death_type_index(DeathType::Killed), 0);
        assert_eq!(death_type_index(DeathType::Ascended), 15);
        assert_eq!(death_type_index(DeathType::Starved), 3);
    }

    #[test]
    fn test_done_in_by_basic() {
        let s = done_in_by("orc", false, false, false, None, false, None, false);
        assert_eq!(s, "orc");
    }

    #[test]
    fn test_done_in_by_unique() {
        let s = done_in_by("Medusa", true, false, false, None, false, None, false);
        assert!(s.starts_with("the "));
    }

    #[test]
    fn test_done_in_by_ghost() {
        let s = done_in_by(
            "ghost",
            false,
            false,
            true,
            Some("Dudley"),
            false,
            None,
            false,
        );
        assert!(s.contains("ghost of Dudley"));
    }

    #[test]
    fn test_done_in_by_shopkeeper() {
        let s = done_in_by("", false, false, false, None, true, Some("Izchak"), true);
        assert!(s.contains("Ms."));
        assert!(s.contains("Izchak"));
    }

    #[test]
    fn test_fixup_death_stoning() {
        let result = fixup_death_reason(DeathType::StonedToDeath, Some("getting stoned"));
        assert!(result.is_none());
    }

    #[test]
    fn test_fixup_death_starving() {
        let result = fixup_death_reason(DeathType::Starved, Some("fainted from lack of food"));
        assert_eq!(result, Some("fainted"));
    }

    #[test]
    fn test_savelife() {
        let mut hp = 0;
        let mut maxhp = 5;
        let mut hunger = 200;
        let msgs = savelife_restore(&mut hp, &mut maxhp, 10, &mut hunger, false);
        assert_eq!(hp, 20); // max(2*10, 10)
        assert_eq!(maxhp, 20);
        assert_eq!(hunger, 900);
        assert!(msgs.len() >= 5);
    }

    #[test]
    fn test_artifact_score_calc() {
        let points = artifact_score("Excalibur", 4000);
        assert_eq!(points, 10000); // 4000 * 5 / 2
    }

    #[test]
    fn test_get_valuables() {
        let items = vec![
            ("diamond".to_string(), 10, 3, false),
            ("ruby".to_string(), 11, 5, false),
            ("diamond".to_string(), 10, 2, false),
            ("Excalibur".to_string(), 50, 1, true), // 아티팩트 제외
        ];
        let (gems, _amulets) = get_valuables(&items);
        assert_eq!(gems.len(), 2);
        // diamond: 3+2=5, ruby: 5
        let diamond = gems.iter().find(|g| g.name == "diamond").unwrap();
        assert_eq!(diamond.count, 5);
    }

    #[test]
    fn test_grave_arise() {
        assert_eq!(grave_arise_monster('W', Race::Human), Some("wraith"));
        assert_eq!(grave_arise_monster('V', Race::Human), Some("vampire"));
        assert_eq!(grave_arise_monster('V', Race::Elf), None);
        assert!(grave_arise_monster('M', Race::Elf).unwrap().contains("elf"));
    }

    #[test]
    fn test_conduct_extensions() {
        let mut c = Conduct::new();
        c.record_wish();
        assert_eq!(c.wishes, 1);
        assert!(!c.active_conducts().contains(&"wishless"));
        c.record_polymorph();
        assert!(!c.polyselfless);
        c.record_genocide();
        assert!(!c.genocideless);
        assert!(c.total_violations() > 0);
    }

    #[test]
    fn test_conduct_detailed_summary() {
        let c = Conduct::new();
        let lines = c.detailed_summary();
        assert!(lines.len() >= 4);
    }

    #[test]
    fn test_highscore_extensions() {
        let mut board = HighScoreBoard::new();
        assert_eq!(board.total_entries(), 0);
        let entry = TopTenEntry {
            score: 5000,
            name: "ValkyriePro".to_string(),
            death_cause: "ascended".to_string(),
            role: PlayerClass::Valkyrie,
            race: Race::Human,
            alignment: Alignment::Neutral,
            level: 30,
            max_depth: 50,
            turns: 20000,
        };
        board.add_entry(entry);
        assert!(board.best_for_role(PlayerClass::Valkyrie).is_some());
        assert!(board.best_for_race(Race::Human).is_some());
        assert_eq!(board.entries_above_score(1000), 1);
        board.clear();
        assert_eq!(board.total_entries(), 0);
    }

    #[test]
    fn test_disclosure_default() {
        let info = DisclosureInfo::default();
        assert_eq!(info.categories.len(), 6);
    }

    #[test]
    fn test_num_extinct() {
        let counts = vec![5, 0, 3, 0, 7, 0];
        assert_eq!(num_extinct(&counts), 3);
    }

    #[test]
    fn test_game_result_summary() {
        let info = DeathInfo::new(DeathType::Ascended, "ascended", 50000, 50);
        let s = game_result_summary(
            "TestHero",
            PlayerClass::Valkyrie,
            Race::Human,
            Alignment::Neutral,
            30,
            &info,
            100000,
        );
        assert!(s.contains("TestHero"));
        assert!(s.contains("ascended"));
        assert!(s.contains("100000"));
    }
}
