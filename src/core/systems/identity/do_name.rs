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
//

use crate::ui::log::GameLog;

// =============================================================================
//
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingMode {
    NameItem,
    CallType,
    NameMonster,
}

///
#[derive(Debug, Clone)]
pub struct NamingResult {
    pub success: bool,
    pub message: String,
    pub target_name: String,
    pub new_name: String,
}

// =============================================================================
//
// =============================================================================

///
pub fn name_item(
    item_base_name: &str,
    new_name: &str,
    log: &mut GameLog,
    turn: u64,
) -> NamingResult {
    //
    if new_name.is_empty() {
        log.add(
            &format!("You remove the name from the {}.", item_base_name),
            turn,
        );
        return NamingResult {
            success: true,
            message: format!("Name removed from {}.", item_base_name),
            target_name: item_base_name.to_string(),
            new_name: String::new(),
        };
    }

    //
    let sanitized = sanitize_name(new_name);
    if sanitized.is_empty() {
        return NamingResult {
            success: false,
            message: "That's not a valid name.".to_string(),
            target_name: item_base_name.to_string(),
            new_name: String::new(),
        };
    }

    //
    let artifact_match = check_artifact_naming(&sanitized);
    if let Some(art_msg) = artifact_match {
        log.add(&art_msg, turn);
    }

    log.add(
        &format!("You name the {} \"{}\".", item_base_name, sanitized),
        turn,
    );

    NamingResult {
        success: true,
        message: format!("Named {} as \"{}\".", item_base_name, sanitized),
        target_name: item_base_name.to_string(),
        new_name: sanitized,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn call_type(type_name: &str, new_call: &str, log: &mut GameLog, turn: u64) -> NamingResult {
    if new_call.is_empty() {
        log.add(&format!("You remove the call from {}.", type_name), turn);
        return NamingResult {
            success: true,
            message: format!("Call removed from {}.", type_name),
            target_name: type_name.to_string(),
            new_name: String::new(),
        };
    }

    let sanitized = sanitize_name(new_call);
    if sanitized.is_empty() {
        return NamingResult {
            success: false,
            message: "That's not a valid name.".to_string(),
            target_name: type_name.to_string(),
            new_name: String::new(),
        };
    }

    log.add(&format!("You call {} \"{}\".", type_name, sanitized), turn);

    NamingResult {
        success: true,
        message: format!("Called {} as \"{}\".", type_name, sanitized),
        target_name: type_name.to_string(),
        new_name: sanitized,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn name_monster(
    monster_type: &str,
    new_name: &str,
    log: &mut GameLog,
    turn: u64,
) -> NamingResult {
    if new_name.is_empty() {
        return NamingResult {
            success: false,
            message: "No name given.".to_string(),
            target_name: monster_type.to_string(),
            new_name: String::new(),
        };
    }

    let sanitized = sanitize_name(new_name);

    //
    if sanitized.len() > 63 {
        return NamingResult {
            success: false,
            message: "That name is too long!".to_string(),
            target_name: monster_type.to_string(),
            new_name: String::new(),
        };
    }

    log.add(
        &format!("You name the {} \"{}\".", monster_type, sanitized),
        turn,
    );

    NamingResult {
        success: true,
        message: format!("Named the {} as \"{}\".", monster_type, sanitized),
        target_name: monster_type.to_string(),
        new_name: sanitized,
    }
}

// =============================================================================
//
// =============================================================================

///
fn sanitize_name(name: &str) -> String {
    //
    let trimmed = name.trim();

    //
    let sanitized: String = trimmed
        .chars()
        .filter(|c| !c.is_control())
        .take(63)
        .collect();

    sanitized
}

// =============================================================================
//
// =============================================================================

///
fn check_artifact_naming(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "excalibur" => {
            Some("A voice intones: \"The naming of names is a dangerous thing.\"".to_string())
        }
        "stormbringer" => Some("Dark energies course through the weapon...".to_string()),
        "sting" => Some("The weapon glows faintly blue.".to_string()),
        "orcrist" => Some("The weapon begins to glow brightly!".to_string()),
        "mjollnir" => Some("Thunder rumbles in the distance.".to_string()),
        "grayswandir" => Some("You feel a sense of cosmic balance.".to_string()),
        _ => None,
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn distant_name(item_name: &str, can_see: bool) -> String {
    if can_see {
        item_name.to_string()
    } else {
        "something".to_string()
    }
}

///
/// user_name > call_name > appearance > base_name
pub fn display_name(
    base_name: &str,
    user_name: Option<&str>,
    call_name: Option<&str>,
    identified: bool,
) -> String {
    //
    if identified {
        if let Some(uname) = user_name {
            if !uname.is_empty() {
                return format!("{} named {}", base_name, uname);
            }
        }
        return base_name.to_string();
    }

    //
    if let Some(cname) = call_name {
        if !cname.is_empty() {
            return format!("{} called {}", base_name, cname);
        }
    }

    base_name.to_string()
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::log::GameLog;

    #[test]
    fn test_name_item() {
        let mut log = GameLog::new(50);
        let result = name_item("long sword", "Frostbrand", &mut log, 1);
        assert!(result.success);
        assert_eq!(result.new_name, "Frostbrand");
    }

    #[test]
    fn test_name_item_empty() {
        let mut log = GameLog::new(50);
        let result = name_item("long sword", "", &mut log, 1);
        assert!(result.success);
        assert!(result.new_name.is_empty());
    }

    #[test]
    fn test_call_type() {
        let mut log = GameLog::new(50);
        let result = call_type("amber potion", "healing?", &mut log, 1);
        assert!(result.success);
        assert_eq!(result.new_name, "healing?");
    }

    #[test]
    fn test_name_monster() {
        let mut log = GameLog::new(50);
        let result = name_monster("kitten", "Whiskers", &mut log, 1);
        assert!(result.success);
        assert_eq!(result.new_name, "Whiskers");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("  hello  "), "hello");
        assert_eq!(sanitize_name(""), "");
        //
        let long_name = "a".repeat(100);
        assert_eq!(sanitize_name(&long_name).len(), 63);
    }

    #[test]
    fn test_display_name() {
        //
        assert_eq!(
            display_name("long sword", Some("Excalibur"), None, true),
            "long sword named Excalibur"
        );
        //
        assert_eq!(
            display_name("amber potion", None, Some("healing?"), false),
            "amber potion called healing?"
        );
        //
        assert_eq!(
            display_name("amber potion", None, None, false),
            "amber potion"
        );
    }

    #[test]
    fn test_artifact_naming() {
        let msg = check_artifact_naming("Excalibur");
        assert!(msg.is_some());
        let msg = check_artifact_naming("random name");
        assert!(msg.is_none());
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

use crate::util::rng::NetHackRng;

///
pub fn generate_random_name(rng: &mut NetHackRng, gender: bool) -> String {
    //
    let male_names = [
        "Rex", "Spot", "Rusty", "Killer", "Fang", "Spike", "Ace", "King", "Duke", "Prince",
        "Shadow", "Lucky", "Max", "Zeus", "Buddy", "Rocky", "Thor", "Bruno", "Titan", "Wolf",
    ];
    let female_names = [
        "Kitty", "Iris", "Luna", "Bella", "Daisy", "Misty", "Angel", "Nala", "Chloe", "Stella",
        "Ruby", "Willow", "Ivy", "Sasha", "Amber", "Pearl", "Jade", "Flora", "Aurora", "Athena",
    ];

    let names = if gender {
        &female_names[..]
    } else {
        &male_names[..]
    };
    let idx = rng.rn2(names.len() as i32) as usize;
    names[idx].to_string()
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterTitle {
    None,
    Elite,    // "elite"
    Ancient,  // "ancient"
    Alpha,    // "alpha"
    Dire,     // "dire"
    Spectral, // "spectral"
    Cursed,   // "cursed"
}

///
pub fn title_prefix(title: MonsterTitle) -> &'static str {
    match title {
        MonsterTitle::None => "",
        MonsterTitle::Elite => "elite ",
        MonsterTitle::Ancient => "ancient ",
        MonsterTitle::Alpha => "alpha ",
        MonsterTitle::Dire => "dire ",
        MonsterTitle::Spectral => "spectral ",
        MonsterTitle::Cursed => "cursed ",
    }
}

///
pub fn titled_monster_name(base_name: &str, title: MonsterTitle) -> String {
    let prefix = title_prefix(title);
    if prefix.is_empty() {
        base_name.to_string()
    } else {
        format!("{}{}", prefix, base_name)
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn buc_adjective(blessed: bool, cursed: bool, known: bool) -> &'static str {
    if !known {
        ""
    } else if blessed {
        "blessed "
    } else if cursed {
        "cursed "
    } else {
        "uncursed "
    }
}

///
pub fn enchantment_display(enchant: i32) -> String {
    if enchant >= 0 {
        format!("+{} ", enchant)
    } else {
        format!("{} ", enchant)
    }
}

///
pub fn full_item_name(
    base_name: &str,
    enchant: Option<i32>,
    blessed: bool,
    cursed: bool,
    buc_known: bool,
    user_name: Option<&str>,
    quantity: i32,
) -> String {
    let mut parts = Vec::new();

    //
    if quantity > 1 {
        parts.push(format!("{} ", quantity));
    }

    // BUC
    let buc = buc_adjective(blessed, cursed, buc_known);
    if !buc.is_empty() {
        parts.push(buc.to_string());
    }

    //
    if let Some(enc) = enchant {
        parts.push(enchantment_display(enc));
    }

    //
    if quantity > 1 {
        parts.push(pluralize(base_name));
    } else {
        parts.push(base_name.to_string());
    }

    //
    if let Some(uname) = user_name {
        if !uname.is_empty() {
            parts.push(format!(" named {}", uname));
        }
    }

    parts.join("")
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn pluralize(name: &str) -> String {
    if name.is_empty() {
        return name.to_string();
    }

    //
    let lower = name.to_lowercase();
    match lower.as_str() {
        "staff" => return "staves".to_string(),
        "leaf" => return "leaves".to_string(),
        "knife" => return "knives".to_string(),
        "loaf" => return "loaves".to_string(),
        "tooth" => return "teeth".to_string(),
        "foot" => return "feet".to_string(),
        "mouse" => return "mice".to_string(),
        "louse" => return "lice".to_string(),
        "child" => return "children".to_string(),
        "goose" => return "geese".to_string(),
        _ => {}
    }

    //
    if name.ends_with('s') || name.ends_with("es") {
        return name.to_string();
    }

    //
    if name.ends_with('y')
        && !name.ends_with("ey")
        && !name.ends_with("ay")
        && !name.ends_with("oy")
    {
        return format!("{}ies", &name[..name.len() - 1]);
    }
    if name.ends_with("sh") || name.ends_with("ch") || name.ends_with('x') || name.ends_with('z') {
        return format!("{}es", name);
    }
    if name.ends_with('f') {
        return format!("{}ves", &name[..name.len() - 1]);
    }
    if name.ends_with("fe") {
        return format!("{}ves", &name[..name.len() - 2]);
    }

    format!("{}s", name)
}

///
pub fn singularize(name: &str) -> String {
    if name.is_empty() {
        return name.to_string();
    }

    //
    let lower = name.to_lowercase();
    match lower.as_str() {
        "staves" => return "staff".to_string(),
        "leaves" => return "leaf".to_string(),
        "knives" => return "knife".to_string(),
        "teeth" => return "tooth".to_string(),
        "feet" => return "foot".to_string(),
        "mice" => return "mouse".to_string(),
        "children" => return "child".to_string(),
        "geese" => return "goose".to_string(),
        _ => {}
    }

    //
    if name.ends_with("ies") {
        return format!("{}y", &name[..name.len() - 3]);
    }
    if name.ends_with("ves") {
        return format!("{}f", &name[..name.len() - 3]);
    }
    if name.ends_with("es") {
        return name[..name.len() - 2].to_string();
    }
    if name.ends_with('s') {
        return name[..name.len() - 1].to_string();
    }

    name.to_string()
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn possessive(name: &str) -> String {
    if name.ends_with('s') {
        format!("{}'", name)
    } else {
        format!("{}'s", name)
    }
}

///
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

///
pub fn article(name: &str) -> &'static str {
    if name.is_empty() {
        return "a";
    }
    let first = name.chars().next().unwrap().to_lowercase().next().unwrap();
    match first {
        'a' | 'e' | 'i' | 'o' | 'u' => "an",
        _ => "a",
    }
}

///
pub fn a_name(name: &str) -> String {
    format!("{} {}", article(name), name)
}

///
pub fn the_name(name: &str) -> String {
    format!("the {}", name)
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod do_name_extended_tests {
    use super::*;

    #[test]
    fn test_random_name() {
        let mut rng = NetHackRng::new(42);
        let name = generate_random_name(&mut rng, false);
        assert!(!name.is_empty());
    }

    #[test]
    fn test_titled_monster() {
        assert_eq!(
            titled_monster_name("dragon", MonsterTitle::Ancient),
            "ancient dragon"
        );
        assert_eq!(titled_monster_name("orc", MonsterTitle::None), "orc");
    }

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize("sword"), "swords");
        assert_eq!(pluralize("staff"), "staves");
        assert_eq!(pluralize("knife"), "knives");
        assert_eq!(pluralize("torch"), "torches");
        assert_eq!(pluralize("berry"), "berries");
    }

    #[test]
    fn test_singularize() {
        assert_eq!(singularize("swords"), "sword");
        assert_eq!(singularize("staves"), "staff");
        assert_eq!(singularize("teeth"), "tooth");
    }

    #[test]
    fn test_possessive() {
        assert_eq!(possessive("orc"), "orc's");
        assert_eq!(possessive("wolves"), "wolves'");
    }

    #[test]
    fn test_direction_name() {
        assert_eq!(direction_name(0, -1), "north");
        assert_eq!(direction_name(1, 1), "southeast");
        assert_eq!(direction_name(0, 0), "here");
    }

    #[test]
    fn test_article() {
        assert_eq!(article("orc"), "an");
        assert_eq!(article("sword"), "a");
        assert_eq!(a_name("elven dagger"), "an elven dagger");
    }

    #[test]
    fn test_full_item_name() {
        let name = full_item_name("long sword", Some(2), true, false, true, None, 1);
        assert_eq!(name, "blessed +2 long sword");
    }

    #[test]
    fn test_full_item_name_plural() {
        let name = full_item_name("arrow", None, false, false, false, None, 5);
        assert_eq!(name, "5 arrows");
    }

    #[test]
    fn test_enchantment_display() {
        assert_eq!(enchantment_display(2), "+2 ");
        assert_eq!(enchantment_display(-1), "-1 ");
    }
}

// =============================================================================
// [v2.4.0] do_name.c 후반부 대량 이식
// 원본: nethack-3.6.7/src/do_name.c (2103-2289)
// 환각 색상/액체, Discworld 소설, 오크 이름, 코요테, 좌표 설명 등
// =============================================================================

// ---------------------------------------------------------------------------
// 환각용 색상 테이블 (원본: do_name.c:2103-2111 hcolors[])
// ---------------------------------------------------------------------------

/// [v2.4.0] 환각 시 표시되는 가짜 색상 테이블 (원본: hcolors[])
pub const HALLUCINATION_COLORS: &[&str] = &[
    "ultraviolet",
    "infrared",
    "bluish-orange",
    "reddish-green",
    "dark white",
    "light black",
    "sky blue-pink",
    "salty",
    "sweet",
    "sour",
    "bitter",
    "striped",
    "spiral",
    "swirly",
    "plaid",
    "checkered",
    "argyle",
    "paisley",
    "blotchy",
    "guernsey-spotted",
    "polka-dotted",
    "square",
    "round",
    "triangular",
    "cabernet",
    "sangria",
    "fuchsia",
    "wisteria",
    "lemon-lime",
    "strawberry-banana",
    "peppermint",
    "romantic",
    "incandescent",
    "octarine", // Discworld: 마법의 색
];

/// [v2.4.0] 환각 상태에서 색상 반환 (원본: hcolor)
pub fn hcolor(color_pref: Option<&str>, rng: &mut NetHackRng) -> String {
    match color_pref {
        Some(c) => c.to_string(),
        None => {
            let idx = rng.rn2(HALLUCINATION_COLORS.len() as i32) as usize;
            HALLUCINATION_COLORS[idx].to_string()
        }
    }
}

// ---------------------------------------------------------------------------
// 환각용 액체 테이블 (원본: do_name.c:2133-2141 hliquids[])
// ---------------------------------------------------------------------------

/// [v2.4.0] 환각 시 표시되는 가짜 액체 테이블 (원본: hliquids[])
pub const HALLUCINATION_LIQUIDS: &[&str] = &[
    "yoghurt",
    "oobleck",
    "clotted blood",
    "diluted water",
    "purified water",
    "instant coffee",
    "tea",
    "herbal infusion",
    "liquid rainbow",
    "creamy foam",
    "mulled wine",
    "bouillon",
    "nectar",
    "grog",
    "flubber",
    "ketchup",
    "slow light",
    "oil",
    "vinaigrette",
    "liquid crystal",
    "honey",
    "caramel sauce",
    "ink",
    "aqueous humour",
    "milk substitute",
    "fruit juice",
    "glowing lava",
    "gastric acid",
    "mineral water",
    "cough syrup",
    "quicksilver",
    "sweet vitriol",
    "grey goo",
    "pink slime",
];

/// [v2.4.0] 환각 상태에서 액체 반환 (원본: hliquid)
pub fn hliquid(liquid_pref: Option<&str>, rng: &mut NetHackRng) -> String {
    match liquid_pref {
        Some(l) => l.to_string(),
        None => {
            let idx = rng.rn2(HALLUCINATION_LIQUIDS.len() as i32) as usize;
            HALLUCINATION_LIQUIDS[idx].to_string()
        }
    }
}

// ---------------------------------------------------------------------------
// Discworld 소설 목록 (원본: do_name.c:2232-2244 sir_Terry_novels[])
// ---------------------------------------------------------------------------

/// [v2.4.0] Terry Pratchett Discworld 소설 목록 (원본: sir_Terry_novels[])
pub const DISCWORLD_NOVELS: &[&str] = &[
    "The Colour of Magic",
    "The Light Fantastic",
    "Equal Rites",
    "Mort",
    "Sourcery",
    "Wyrd Sisters",
    "Pyramids",
    "Guards! Guards!",
    "Eric",
    "Moving Pictures",
    "Reaper Man",
    "Witches Abroad",
    "Small Gods",
    "Lords and Ladies",
    "Men at Arms",
    "Soul Music",
    "Interesting Times",
    "Maskerade",
    "Feet of Clay",
    "Hogfather",
    "Jingo",
    "The Last Continent",
    "Carpe Jugulum",
    "The Fifth Elephant",
    "The Truth",
    "Thief of Time",
    "The Last Hero",
    "The Amazing Maurice and His Educated Rodents",
    "Night Watch",
    "The Wee Free Men",
    "Monstrous Regiment",
    "A Hat Full of Sky",
    "Going Postal",
    "Thud!",
    "Wintersmith",
    "Making Money",
    "Unseen Academicals",
    "I Shall Wear Midnight",
    "Snuff",
    "Raising Steam",
    "The Shepherd's Crown",
];

/// [v2.4.0] 랜덤 소설 제목 (원본: noveltitle)
pub fn novel_title(rng: &mut NetHackRng) -> &'static str {
    let idx = rng.rn2(DISCWORLD_NOVELS.len() as i32) as usize;
    DISCWORLD_NOVELS[idx]
}

/// [v2.4.0] 소설 이름 조회 (원본: lookup_novel)
/// 미국식 "Color" ↔ 영국식 "Colour" 변환 지원
pub fn lookup_novel(name: &str) -> Option<&'static str> {
    // "The Color of Magic" → "The Colour of Magic" 변환
    let normalized = if name.eq_ignore_ascii_case("The Color of Magic")
        || name.eq_ignore_ascii_case("Color of Magic")
    {
        "The Colour of Magic"
    } else {
        name
    };

    DISCWORLD_NOVELS
        .iter()
        .find(|n| {
            n.eq_ignore_ascii_case(normalized)
                || format!("The {}", normalized).eq_ignore_ascii_case(n)
        })
        .copied()
}

// ---------------------------------------------------------------------------
// 코요테 별칭 (원본: do_name.c:2153-2162 coynames[])
// ---------------------------------------------------------------------------

/// [v2.4.0] 로드러너의 천적 코요테 학명 별칭 (원본: coynames[])
pub const COYOTE_ALIASES: &[&str] = &[
    "Carnivorous Vulgaris",
    "Road-Runnerus Digestus",
    "Eatibus Anythingus",
    "Famishus-Famishus",
    "Eatibus Almost Anythingus",
    "Eatius Birdius",
    "Famishius Fantasticus",
    "Eternalii Famishiis",
    "Famishus Vulgarus",
    "Famishius Vulgaris Ingeniusi",
    "Eatius-Slobbius",
    "Hardheadipus Oedipus",
    "Carnivorous Slobbius",
    "Hard-Headipus Ravenus",
    "Evereadii Eatibus",
    "Apetitius Giganticus",
    "Hungrii Flea-Bagius",
    "Overconfidentii Vulgaris",
    "Caninus Nervous Rex",
    "Grotesques Appetitus",
    "Nemesis Ridiculii",
    "Canis latrans",
];

/// [v2.4.0] 코요테 이름 생성 (원본: coyotename)
pub fn coyote_name(monster_name: &str, monster_id: u32, is_cancelled: bool) -> String {
    let alias = if is_cancelled {
        COYOTE_ALIASES[COYOTE_ALIASES.len() - 1] // 마지막: "Canis latrans" (정식 학명)
    } else {
        COYOTE_ALIASES[(monster_id as usize) % (COYOTE_ALIASES.len() - 1)]
    };
    format!("{} - {}", monster_name, alias)
}

// ---------------------------------------------------------------------------
// 오크 이름 생성 (원본: do_name.c:2178-2196 rndorcname)
// ---------------------------------------------------------------------------

/// [v2.4.0] 오크 이름용 모음
const ORC_VOWELS: &[&str] = &["a", "ai", "og", "u"];

/// [v2.4.0] 오크 이름용 자음
const ORC_CONSONANTS: &[&str] = &[
    "gor", "gris", "un", "bane", "ruk", "oth", "ul", "z", "thos", "akh", "hai",
];

/// [v2.4.0] 랜덤 오크 이름 생성 (원본: rndorcname)
pub fn random_orc_name(rng: &mut NetHackRng) -> String {
    let syllable_count = 3 + rng.rn2(2) as usize; // 3~4 음절
    let mut vowel_start = rng.rn2(2) != 0;
    let mut name = String::new();

    for i in 0..syllable_count {
        // 30분의 1 확률로 하이픈 삽입 (첫 음절 제외)
        if i > 0 && rng.rn2(30) == 0 {
            name.push('-');
        }

        if vowel_start {
            let idx = rng.rn2(ORC_VOWELS.len() as i32) as usize;
            name.push_str(ORC_VOWELS[idx]);
        } else {
            let idx = rng.rn2(ORC_CONSONANTS.len() as i32) as usize;
            name.push_str(ORC_CONSONANTS[idx]);
        }
        vowel_start = !vowel_start; // 모음/자음 교대
    }

    // 첫 글자 대문자
    let mut chars = name.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => name,
    }
}

// ---------------------------------------------------------------------------
// 좌표 설명 확장 (원본: do_name.c:458-493 dxdy_to_dist_descr)
// ---------------------------------------------------------------------------

/// [v2.4.0] 좌표 차이를 거리 설명으로 변환 (원본: dxdy_to_dist_descr)
pub fn distance_description(dx: i32, dy: i32, full_dir: bool) -> String {
    if dx == 0 && dy == 0 {
        return "here".to_string();
    }

    // 단일 이동 방향인 경우
    if dx.abs() <= 1 && dy.abs() <= 1 {
        return direction_name(dx, dy).to_string();
    }

    // 복합 이동
    let dir_names: [(&str, &str); 4] =
        [("n", "north"), ("s", "south"), ("w", "west"), ("e", "east")];

    let mut parts = Vec::new();

    if dy != 0 {
        let dir_idx = if dy > 0 { 1 } else { 0 };
        let dir = if full_dir {
            dir_names[dir_idx].1
        } else {
            dir_names[dir_idx].0
        };
        parts.push(format!("{}{}", dy.abs(), dir));
    }

    if dx != 0 {
        let dir_idx = if dx > 0 { 3 } else { 2 };
        let dir = if full_dir {
            dir_names[dir_idx].1
        } else {
            dir_names[dir_idx].0
        };
        parts.push(format!("{}{}", dx.abs(), dir));
    }

    parts.join(",")
}

// ---------------------------------------------------------------------------
// 몬스터 관사 헬퍼 (원본: do_name.c:1898-1939)
// ---------------------------------------------------------------------------

/// [v2.4.0] 몬스터 관사 유형 (원본: ARTICLE_NONE/THE/A/YOUR)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterArticle {
    /// 관사 없음
    None,
    /// 정관사 "the"
    The,
    /// 부정관사 "a/an"
    A,
    /// 소유격 "your"
    Your,
}

/// [v2.4.0] 몬스터 이름에 관사 적용 (원본: mon_nam, a_monnam, Adjmonnam)
pub fn monster_with_article(name: &str, article_type: MonsterArticle) -> String {
    match article_type {
        MonsterArticle::None => name.to_string(),
        MonsterArticle::The => format!("the {}", name),
        MonsterArticle::A => a_name(name),
        MonsterArticle::Your => format!("your {}", name),
    }
}

/// [v2.4.0] 대문자 시작 몬스터 이름 (원본: Adjmonnam, Amonnam)
pub fn monster_with_article_cap(name: &str, article_type: MonsterArticle) -> String {
    let result = monster_with_article(name, article_type);
    let mut chars = result.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => result,
    }
}

// ---------------------------------------------------------------------------
// 재귀적 대명사 (원본: do_name.c:1964-1987 mon_nam_too)
// ---------------------------------------------------------------------------

/// [v2.4.0] 성별 기반 재귀 대명사 (원본: mon_nam_too)
pub fn reflexive_pronoun(gender: u8) -> &'static str {
    match gender {
        0 => "himself",
        1 => "herself",
        _ => "itself",
    }
}

/// [v2.4.0] 로그 이름 (원본: roguename)
pub fn rogue_name(rng: &mut NetHackRng) -> &'static str {
    match rng.rn2(3) {
        0 => "Glenn Wichman",
        1 => "Michael Toy",
        _ => "Kenneth Arnold",
    }
}

// ---------------------------------------------------------------------------
// 테스트
// ---------------------------------------------------------------------------
#[cfg(test)]
mod do_name_v240_tests {
    use super::*;

    #[test]
    fn test_hallucination_colors() {
        let mut rng = NetHackRng::new(42);
        let color = hcolor(None, &mut rng);
        assert!(!color.is_empty());
        assert!(HALLUCINATION_COLORS.contains(&color.as_str()));

        // 선호 색상 지정 시 그대로 반환
        assert_eq!(hcolor(Some("red"), &mut rng), "red");
    }

    #[test]
    fn test_hallucination_liquids() {
        let mut rng = NetHackRng::new(42);
        let liquid = hliquid(None, &mut rng);
        assert!(!liquid.is_empty());
        assert!(HALLUCINATION_LIQUIDS.contains(&liquid.as_str()));
    }

    #[test]
    fn test_novel_title() {
        let mut rng = NetHackRng::new(42);
        let title = novel_title(&mut rng);
        assert!(DISCWORLD_NOVELS.contains(&title));
    }

    #[test]
    fn test_lookup_novel() {
        assert_eq!(lookup_novel("Mort"), Some("Mort"));
        assert_eq!(
            lookup_novel("The Color of Magic"),
            Some("The Colour of Magic")
        );
        assert_eq!(lookup_novel("nonexistent book"), None);
    }

    #[test]
    fn test_coyote_name() {
        let name = coyote_name("coyote", 5, false);
        assert!(name.starts_with("coyote - "));

        let cancelled = coyote_name("coyote", 0, true);
        assert!(cancelled.contains("Canis latrans"));
    }

    #[test]
    fn test_random_orc_name() {
        let mut rng = NetHackRng::new(42);
        let name = random_orc_name(&mut rng);
        assert!(!name.is_empty());
        // 첫 글자 대문자 확인
        assert!(name.chars().next().unwrap().is_uppercase());
    }

    #[test]
    fn test_distance_description() {
        assert_eq!(distance_description(0, 0, false), "here");
        assert_eq!(distance_description(0, -1, false), "north");
        assert_eq!(distance_description(3, -5, false), "5n,3e");
        assert_eq!(distance_description(3, -5, true), "5north,3east");
    }

    #[test]
    fn test_monster_with_article() {
        assert_eq!(monster_with_article("orc", MonsterArticle::The), "the orc");
        assert_eq!(monster_with_article("orc", MonsterArticle::A), "an orc");
        assert_eq!(
            monster_with_article("orc", MonsterArticle::Your),
            "your orc"
        );
        assert_eq!(monster_with_article("orc", MonsterArticle::None), "orc");
    }

    #[test]
    fn test_monster_with_article_cap() {
        assert_eq!(
            monster_with_article_cap("orc", MonsterArticle::The),
            "The orc"
        );
        assert_eq!(monster_with_article_cap("orc", MonsterArticle::A), "An orc");
    }

    #[test]
    fn test_reflexive_pronoun() {
        assert_eq!(reflexive_pronoun(0), "himself");
        assert_eq!(reflexive_pronoun(1), "herself");
        assert_eq!(reflexive_pronoun(2), "itself");
    }

    #[test]
    fn test_rogue_name() {
        let mut rng = NetHackRng::new(42);
        let name = rogue_name(&mut rng);
        assert!(!name.is_empty());
    }
}
