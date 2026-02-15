// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// =============================================================================
//
//
// [v1.9.0
// =============================================================================
//
//
//
//

use crate::core::entity::object::ItemClass;

// =============================================================================
//
// =============================================================================

///
///
pub fn doname(
    template_name: &str,
    quantity: i32,
    blessed: bool,
    cursed: bool,
    bknown: bool,
    known: bool,
    spe: i32,
    class: ItemClass,
    erosion1: i32,
    erosion2: i32,
    user_name: Option<&str>,
    artifact: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    //
    if quantity <= 0 {
        return format!("(no {}", template_name);
    }

    //
    if bknown {
        if blessed {
            parts.push("blessed".to_string());
        } else if cursed {
            parts.push("cursed".to_string());
        }
        //
    }

    //
    let erosion_str = erosion_adjective(erosion1, erosion2);
    if !erosion_str.is_empty() {
        parts.push(erosion_str);
    }

    //
    if known && (class == ItemClass::Weapon || class == ItemClass::Armor) && spe != 0 {
        parts.push(format!("{:+}", spe));
    }

    //
    if quantity > 1 {
        parts.push(makeplural(template_name));
    } else {
        parts.push(template_name.to_string());
    }

    //
    if let Some(art) = artifact {
        parts.push(format!("({})", art));
    }

    //
    if let Some(uname) = user_name {
        parts.push(format!("named {}", uname));
    }

    //
    let item_str = parts.join(" ");
    if quantity > 1 {
        format!("{} {}", quantity, item_str)
    } else {
        let article = item_article(template_name, &item_str);
        format!("{} {}", article, item_str)
    }
}

///
pub fn simpleonames(template_name: &str, quantity: i32) -> String {
    if quantity > 1 {
        format!("{} {}", quantity, makeplural(template_name))
    } else {
        template_name.to_string()
    }
}

///
pub fn xname(
    template_name: &str,
    known: bool,
    class: ItemClass,
    appearance: Option<&str>,
) -> String {
    if known {
        template_name.to_string()
    } else {
        //
        match appearance {
            Some(app) => app.to_string(),
            None => unidentified_name(template_name, class),
        }
    }
}

// =============================================================================
//
// =============================================================================

///
fn item_article(name: &str, _full_str: &str) -> &'static str {
    let first = name.chars().next().unwrap_or(' ').to_ascii_lowercase();
    match first {
        'a' | 'e' | 'i' | 'o' | 'u' => "an",
        _ => "a",
    }
}

///
pub fn the_name(name: &str) -> String {
    format!("the {}", name)
}

///
pub fn the_name_upper(name: &str) -> String {
    format!("The {}", name)
}

///
pub fn an_name(name: &str) -> String {
    let first = name.chars().next().unwrap_or(' ').to_ascii_lowercase();
    if matches!(first, 'a' | 'e' | 'i' | 'o' | 'u') {
        format!("an {}", name)
    } else {
        format!("a {}", name)
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn makeplural(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    //
    let special = [
        ("potion of", "potions of"),
        ("scroll of", "scrolls of"),
        ("spellbook of", "spellbooks of"),
        ("ring of", "rings of"),
        ("wand of", "wands of"),
        ("pair of", "pairs of"),
        ("piece of", "pieces of"),
        ("clove of", "cloves of"),
    ];
    for (singular, plural) in &special {
        if name.starts_with(singular) {
            return name.replacen(singular, plural, 1);
        }
    }

    //
    if let Some(of_idx) = name.find(" of ") {
        let base = &name[..of_idx];
        let rest = &name[of_idx..];
        return format!("{}{}", pluralize_word(base), rest);
    }

    pluralize_word(name)
}

///
fn pluralize_word(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    //
    let irregular: &[(&str, &str)] = &[
        ("staff", "staves"),
        ("leaf", "leaves"),
        ("loaf", "loaves"),
        ("knife", "knives"),
        ("tooth", "teeth"),
        ("foot", "feet"),
        ("goose", "geese"),
        ("mouse", "mice"),
        ("louse", "lice"),
        ("child", "children"),
        ("ox", "oxen"),
        ("vortex", "vortices"),
        ("fungus", "fungi"),
        ("cactus", "cacti"),
        ("homunculus", "homunculi"),
        ("succubus", "succubi"),
        ("incubus", "incubi"),
        ("matzoh", "matzot"),
        ("matzah", "matzot"),
    ];
    for (sing, plur) in irregular {
        if word == *sing {
            return plur.to_string();
        }
    }

    //
    let invariant = ["sheep", "deer", "fish", "samurai", "ronin", "shuriken"];
    for inv in &invariant {
        if word == *inv {
            return word.to_string();
        }
    }

    let last = word.chars().last().unwrap_or(' ');
    let len = word.len();

    // -man ??-men
    if word.ends_with("man") && len > 3 {
        return format!("{}men", &word[..len - 3]);
    }

    //
    if last == 'y' && len > 1 {
        let prev = word.chars().nth(len - 2).unwrap_or(' ');
        if !matches!(prev, 'a' | 'e' | 'i' | 'o' | 'u') {
            return format!("{}ies", &word[..len - 1]);
        }
    }

    // -s, -x, -z, -sh, -ch ??+es
    if last == 's' || last == 'x' || last == 'z' {
        return format!("{}es", word);
    }
    if word.ends_with("sh") || word.ends_with("ch") {
        return format!("{}es", word);
    }

    //
    if last == 'f' && !word.ends_with("staff") {
        return format!("{}ves", &word[..len - 1]);
    }
    if word.ends_with("fe") && len > 2 {
        return format!("{}ves", &word[..len - 2]);
    }

    //
    format!("{}s", word)
}

///
pub fn makesingular(name: &str) -> String {
    if name.is_empty() {
        return String::new();
    }

    //
    let special = [
        ("potions of", "potion of"),
        ("scrolls of", "scrolls of"),
        ("spellbooks of", "spellbook of"),
        ("rings of", "ring of"),
        ("wands of", "wand of"),
        ("pairs of", "pair of"),
        ("pieces of", "piece of"),
    ];
    for (plural, singular) in &special {
        if name.starts_with(plural) {
            return name.replacen(plural, singular, 1);
        }
    }

    // -ies ??-y
    if name.ends_with("ies") {
        return format!("{}y", &name[..name.len() - 3]);
    }
    // -ves ??-f / -fe
    if name.ends_with("ves") {
        let base = &name[..name.len() - 3];
        //
        if base == "sta" {
            return "staff".to_string();
        }
        return format!("{}f", base);
    }
    // -es ??remove
    if name.ends_with("es") && name.len() > 3 {
        let base = &name[..name.len() - 2];
        let last = base.chars().last().unwrap_or(' ');
        if matches!(last, 's' | 'x' | 'z') || base.ends_with("sh") || base.ends_with("ch") {
            return base.to_string();
        }
    }
    // -s ??remove
    if name.ends_with('s') && !name.ends_with("ss") {
        return name[..name.len() - 1].to_string();
    }

    name.to_string()
}

// =============================================================================
//
// =============================================================================

///
pub fn buc_string(blessed: bool, cursed: bool, bknown: bool) -> &'static str {
    if !bknown {
        ""
    } else if blessed {
        "blessed"
    } else if cursed {
        "cursed"
    } else {
        "uncursed"
    }
}

///
pub fn buc_color(blessed: bool, cursed: bool, bknown: bool) -> [u8; 3] {
    if !bknown {
        [200, 200, 200]
    } else if blessed {
        [100, 200, 255]
    } else if cursed {
        [255, 80, 80]
    } else {
        [200, 200, 200]
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn erosion_adjective(eroded: i32, eroded2: i32) -> String {
    //
    let mut parts = Vec::new();

    match eroded {
        1 => parts.push("rusty"),
        2 => parts.push("very rusty"),
        3 => parts.push("thoroughly rusty"),
        _ => {}
    }
    match eroded2 {
        1 => parts.push("burnt"),
        2 => parts.push("very burnt"),
        3 => parts.push("thoroughly burnt"),
        _ => {}
    }

    parts.join(" ")
}

///
pub fn erosion_ac_penalty(eroded: i32, eroded2: i32) -> i32 {
    eroded.max(eroded2)
}

// =============================================================================
//
// =============================================================================

///
pub fn unidentified_name(template_name: &str, class: ItemClass) -> String {
    match class {
        ItemClass::Potion => potion_appearance(template_name),
        ItemClass::Scroll => scroll_appearance(template_name),
        ItemClass::Ring => ring_appearance(template_name),
        ItemClass::Wand => wand_appearance(template_name),
        ItemClass::Spellbook => spellbook_appearance(template_name),
        ItemClass::Amulet => amulet_appearance(template_name),
        ItemClass::Gem => gem_appearance(template_name),
        _ => template_name.to_string(),
    }
}

///
fn potion_appearance(name: &str) -> String {
    //
    //
    let colors = [
        "ruby",
        "pink",
        "orange",
        "yellow",
        "emerald",
        "dark green",
        "cyan",
        "sky blue",
        "brilliant blue",
        "magenta",
        "purple-red",
        "puce",
        "milky",
        "swirly",
        "bubbly",
        "smoky",
        "cloudy",
        "effervescent",
        "black",
        "golden",
        "brown",
        "fizzy",
        "dark",
        "white",
        "murky",
        "clear",
    ];
    let idx = simple_hash(name) % colors.len();
    format!("{} potion", colors[idx])
}

///
fn scroll_appearance(name: &str) -> String {
    let labels = [
        "ZELGO MER",
        "JUYED AWK YACC",
        "NR 9",
        "XIXAXA XOXAXA XUXAXA",
        "PRAKAL XANAHT",
        "DAIYEN FANSEN",
        "LEP GEX VEN ZEA",
        "PRIRUTSENIE",
        "ELBIB YLANSEN",
        "VERR YED HULL",
        "YUM YUM",
        "KERNOD WEL",
        "ELAM EANSEN",
        "DUAM XNAHT",
        "ANDOVA BEGARIN",
        "KIRJE",
        "VE FORBRULL",
        "THARR",
        "GARVEN DEH",
        "READ ME",
        "TEMOV",
    ];
    let idx = simple_hash(name) % labels.len();
    format!("scroll labeled {}", labels[idx])
}

///
fn ring_appearance(name: &str) -> String {
    let materials = [
        "wooden",
        "granite",
        "opal",
        "clay",
        "coral",
        "black onyx",
        "moonstone",
        "tiger eye",
        "jade",
        "bronze",
        "agate",
        "topaz",
        "sapphire",
        "ruby",
        "diamond",
        "pearl",
        "iron",
        "brass",
        "copper",
        "twisted",
        "steel",
        "wire",
        "engagement",
        "shiny",
    ];
    let idx = simple_hash(name) % materials.len();
    format!("{} ring", materials[idx])
}

///
fn wand_appearance(name: &str) -> String {
    let materials = [
        "glass",
        "balsa",
        "crystal",
        "maple",
        "pine",
        "oak",
        "ebony",
        "marble",
        "tin",
        "brass",
        "copper",
        "silver",
        "platinum",
        "iridium",
        "zinc",
        "aluminum",
        "uranium",
        "iron",
        "steel",
        "hexagonal",
        "short",
        "runed",
        "long",
        "curved",
        "forked",
        "spiked",
        "jeweled",
    ];
    let idx = simple_hash(name) % materials.len();
    format!("{} wand", materials[idx])
}

///
fn spellbook_appearance(name: &str) -> String {
    let covers = [
        "parchment",
        "vellum",
        "ragged",
        "dog eared",
        "mottled",
        "stained",
        "cloth",
        "leather",
        "white",
        "pink",
        "red",
        "orange",
        "yellow",
        "velvet",
        "light green",
        "dark green",
        "turquoise",
        "cyan",
        "light blue",
        "dark blue",
        "indigo",
        "magenta",
        "purple",
        "violet",
        "tan",
        "plaid",
        "light brown",
        "dark brown",
        "gray",
        "wrinkled",
        "dusty",
        "bronze",
        "copper",
        "silver",
        "gold",
        "glittering",
        "shining",
        "dull",
        "thin",
        "thick",
    ];
    let idx = simple_hash(name) % covers.len();
    format!("{} spellbook", covers[idx])
}

///
fn amulet_appearance(name: &str) -> String {
    let shapes = [
        "circular",
        "spherical",
        "oval",
        "triangular",
        "pyramidal",
        "square",
        "concave",
        "hexagonal",
        "octagonal",
    ];
    let idx = simple_hash(name) % shapes.len();
    format!("{} amulet", shapes[idx])
}

///
fn gem_appearance(name: &str) -> String {
    let colors = [
        "white",
        "white",
        "red",
        "orange",
        "blue",
        "black",
        "green",
        "green",
        "yellow",
        "yellowish brown",
        "violet",
        "red",
        "blue",
        "red",
        "green",
        "orange",
        "yellow",
    ];
    let idx = simple_hash(name) % colors.len();
    format!("{} gem", colors[idx])
}

///
fn simple_hash(s: &str) -> usize {
    let mut h: usize = 0;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as usize);
    }
    h
}

// =============================================================================
//
// =============================================================================

///
pub fn class_display_name(class: ItemClass) -> &'static str {
    match class {
        ItemClass::Weapon => "Weapons",
        ItemClass::Armor => "Armor",
        ItemClass::Ring => "Rings",
        ItemClass::Amulet => "Amulets",
        ItemClass::Tool => "Tools",
        ItemClass::Food => "Comestibles",
        ItemClass::Potion => "Potions",
        ItemClass::Scroll => "Scrolls",
        ItemClass::Spellbook => "Spellbooks",
        ItemClass::Wand => "Wands",
        ItemClass::Coin => "Coins",
        ItemClass::Gem => "Gems/Stones",
        ItemClass::Rock => "Large Rocks",
        _ => "All",
    }
}

///
pub fn char_to_class(ch: char) -> Option<ItemClass> {
    match ch {
        ')' => Some(ItemClass::Weapon),
        '[' => Some(ItemClass::Armor),
        '=' => Some(ItemClass::Ring),
        '"' => Some(ItemClass::Amulet),
        '(' => Some(ItemClass::Tool),
        '%' => Some(ItemClass::Food),
        '!' => Some(ItemClass::Potion),
        '?' => Some(ItemClass::Scroll),
        '+' => Some(ItemClass::Spellbook),
        '/' => Some(ItemClass::Wand),
        '$' => Some(ItemClass::Coin),
        '*' => Some(ItemClass::Gem),
        '`' => Some(ItemClass::Rock),
        _ => None,
    }
}

///
pub fn class_to_char(class: ItemClass) -> char {
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
        ItemClass::Gem => '*',
        ItemClass::Rock => '`',
        _ => '?',
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn item_description(template_name: &str, class: ItemClass) -> String {
    match class {
        ItemClass::Weapon => format!("A weapon called {}.", template_name),
        ItemClass::Armor => format!("A piece of armor called {}.", template_name),
        ItemClass::Ring => format!(
            "A magical ring: {}. It must be worn to take effect.",
            template_name
        ),
        ItemClass::Amulet => format!("An amulet: {}. Wear it around your neck.", template_name),
        ItemClass::Potion => format!(
            "A potion of {}. Quaff it to gain its effects.",
            template_name
        ),
        ItemClass::Scroll => format!(
            "A scroll of {}. Read it to invoke its magic.",
            template_name
        ),
        ItemClass::Wand => format!("A wand of {}. Zap it to release its power.", template_name),
        ItemClass::Spellbook => format!(
            "A spellbook of {}. Study it to learn the spell.",
            template_name
        ),
        ItemClass::Food => format!("{}. It can be eaten.", template_name),
        ItemClass::Tool => format!("A tool: {}.", template_name),
        ItemClass::Gem => format!("A gem: {}. It may be valuable.", template_name),
        _ => format!("{}.", template_name),
    }
}

// =============================================================================
//
// =============================================================================

///
///
///
pub fn parse_wish_string(input: &str) -> WishResult {
    let input = input.trim().to_lowercase();
    let mut result = WishResult {
        name: String::new(),
        quantity: 1,
        blessed: false,
        cursed: false,
        spe: 0,
        eroded: 0,
    };

    let mut remaining = input.as_str();

    //
    if let Some(space_idx) = remaining.find(' ') {
        if let Ok(qty) = remaining[..space_idx].parse::<i32>() {
            result.quantity = qty.max(1);
            remaining = &remaining[space_idx + 1..];
        }
    }

    //
    if remaining.starts_with("blessed ") {
        result.blessed = true;
        remaining = &remaining[8..];
    } else if remaining.starts_with("cursed ") {
        result.cursed = true;
        remaining = &remaining[7..];
    } else if remaining.starts_with("uncursed ") {
        remaining = &remaining[9..];
    }

    //
    if remaining.starts_with('+') || remaining.starts_with('-') {
        if let Some(space_idx) = remaining.find(' ') {
            if let Ok(n) = remaining[..space_idx].parse::<i32>() {
                result.spe = n;
                remaining = &remaining[space_idx + 1..];
            }
        }
    }

    //
    if remaining.starts_with("rusty ") {
        result.eroded = 1;
        remaining = &remaining[6..];
    } else if remaining.starts_with("very rusty ") {
        result.eroded = 2;
        remaining = &remaining[11..];
    } else if remaining.starts_with("thoroughly rusty ") {
        result.eroded = 3;
        remaining = &remaining[17..];
    }

    result.name = remaining.to_string();
    result
}

///
#[derive(Debug, Clone)]
pub struct WishResult {
    pub name: String,
    pub quantity: i32,
    pub blessed: bool,
    pub cursed: bool,
    pub spe: i32,
    pub eroded: i32,
}

// =============================================================================
//
// =============================================================================

///
pub fn price_string(base_price: u32, quantity: i32, blessed: bool, cursed: bool) -> String {
    let mut price = base_price as i32 * quantity;
    if blessed {
        price = (price * 3) / 2;
    }
    if cursed {
        price /= 2;
    }
    format!("{} gold piece{}", price, if price != 1 { "s" } else { "" })
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_makeplural_items() {
        assert_eq!(makeplural("potion of healing"), "potions of healing");
        assert_eq!(makeplural("scroll of identify"), "scrolls of identify");
        assert_eq!(makeplural("dagger"), "daggers");
        assert_eq!(makeplural("elf"), "elves");
        assert_eq!(makeplural("staff"), "staves");
    }

    #[test]
    fn test_makesingular() {
        assert_eq!(makesingular("daggers"), "dagger");
        assert_eq!(makesingular("potions of healing"), "potion of healing");
    }

    #[test]
    fn test_buc_string() {
        assert_eq!(buc_string(true, false, true), "blessed");
        assert_eq!(buc_string(false, true, true), "cursed");
        assert_eq!(buc_string(false, false, true), "uncursed");
        assert_eq!(buc_string(true, false, false), "");
    }

    #[test]
    fn test_parse_wish_string() {
        let wish = parse_wish_string("blessed +3 silver dragon scale mail");
        assert!(wish.blessed);
        assert_eq!(wish.spe, 3);
        assert_eq!(wish.name, "silver dragon scale mail");

        let wish2 = parse_wish_string("2 cursed scrolls of identify");
        assert_eq!(wish2.quantity, 2);
        assert!(wish2.cursed);
    }

    #[test]
    fn test_erosion() {
        assert_eq!(erosion_adjective(1, 0), "rusty");
        assert_eq!(erosion_adjective(2, 1), "very rusty burnt");
        assert_eq!(erosion_adjective(0, 0), "");
    }
}
