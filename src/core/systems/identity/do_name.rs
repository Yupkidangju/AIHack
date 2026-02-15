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
