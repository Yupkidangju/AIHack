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

use crate::core::entity::object::{ItemClass, ItemTemplate, Material};

// =============================================================================
//
// =============================================================================

///
///
pub fn xname(
    template: &ItemTemplate,
    blessed: bool,
    cursed: bool,
    known: bool,
    bknown: bool,
    spe: i8,
    quantity: u32,
    user_name: Option<&str>,
    artifact: Option<&str>,
    oeroded: u8,
    oeroded2: u8,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    //
    if bknown {
        if blessed {
            parts.push("blessed".into());
        } else if cursed {
            parts.push("cursed".into());
        }
    }

    //
    let erosion = erosion_text(oeroded, oeroded2);
    if !erosion.is_empty() {
        parts.push(erosion);
    }

    //
    if known && spe != 0 {
        parts.push(format!("{:+}", spe));
    }

    //
    if quantity > 1 {
        parts.push(format!("{}", quantity));
    }

    //
    let base = if quantity > 1 {
        pluralize(&template.name)
    } else {
        template.name.clone()
    };
    parts.push(base);

    //
    if let Some(uname) = user_name {
        parts.push(format!("named {}", uname));
    }

    //
    if let Some(aname) = artifact {
        return aname.to_string();
    }

    parts.join(" ")
}

///
pub fn doname(
    template: &ItemTemplate,
    blessed: bool,
    cursed: bool,
    known: bool,
    bknown: bool,
    spe: i8,
    quantity: u32,
    user_name: Option<&str>,
    artifact: Option<&str>,
    oeroded: u8,
    oeroded2: u8,
    unpaid: bool,
    price: u32,
) -> String {
    let mut name = xname(
        template, blessed, cursed, known, bknown, spe, quantity, user_name, artifact, oeroded,
        oeroded2,
    );

    //
    if unpaid {
        name.push_str(&format!(" (unpaid, {} zm)", price));
    }

    name
}

///
pub fn simple_typename(class: ItemClass) -> &'static str {
    match class {
        ItemClass::Weapon => "weapon",
        ItemClass::Armor => "armor",
        ItemClass::Ring => "ring",
        ItemClass::Amulet => "amulet",
        ItemClass::Tool => "tool",
        ItemClass::Food => "food",
        ItemClass::Potion => "potion",
        ItemClass::Scroll => "scroll",
        ItemClass::Spellbook => "spellbook",
        ItemClass::Wand => "wand",
        ItemClass::Coin => "coin",
        ItemClass::Gem => "gem",
        ItemClass::Rock => "rock",
        ItemClass::Ball => "iron ball",
        ItemClass::Chain => "iron chain",
        ItemClass::Venom => "venom",
        ItemClass::Random => "strange object",
        ItemClass::IllObj => "strange object",
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn erosion_text(oeroded: u8, oeroded2: u8) -> String {
    let rust = match oeroded {
        1 => "rusty ",
        2 => "very rusty ",
        3 => "thoroughly rusty ",
        _ => "",
    };
    let burn = match oeroded2 {
        1 => "burnt ",
        2 => "very burnt ",
        3 => "thoroughly burnt ",
        _ => "",
    };
    format!("{}{}", rust, burn).trim().to_string()
}

// =============================================================================
//
// =============================================================================

///
pub fn pluralize(name: &str) -> String {
    if name.is_empty() {
        return name.to_string();
    }

    //
    let special: &[(&str, &str)] = &[
        ("knife", "knives"),
        ("staff", "staves"),
        ("tooth", "teeth"),
        ("foot", "feet"),
        ("goose", "geese"),
        ("mouse", "mice"),
        ("louse", "lice"),
        ("child", "children"),
        ("wolf", "wolves"),
        ("leaf", "leaves"),
        ("loaf", "loaves"),
        ("thief", "thieves"),
        ("matzoh", "matzot"),
        ("vortex", "vortices"),
        ("fungus", "fungi"),
        ("cactus", "cacti"),
        ("locus", "loci"),
        ("lotus", "lotuses"),
    ];

    for (sing, plur) in special {
        if name.ends_with(sing) {
            let base = &name[..name.len() - sing.len()];
            return format!("{}{}", base, plur);
        }
    }

    //
    let invariant = [
        "gold piece",
        "pair of",
        "samurai",
        "ronin",
        "sheep",
        "ki-rin",
        "shuriken",
        "tengu",
    ];
    for inv in &invariant {
        if name.contains(inv) {
            return name.to_string();
        }
    }

    //
    if name.ends_with("ss")
        || name.ends_with("sh")
        || name.ends_with("ch")
        || name.ends_with('x')
        || name.ends_with('z')
    {
        format!("{}es", name)
    } else if name.ends_with('s') {
        format!("{}es", name)
    } else if name.ends_with('y') && !name.ends_with("ey") && !name.ends_with("oy") {
        format!("{}ies", &name[..name.len() - 1])
    } else if name.ends_with('o') && !name.ends_with("oo") {
        format!("{}es", name)
    } else if name.ends_with('f') && !name.ends_with("ff") {
        format!("{}ves", &name[..name.len() - 1])
    } else if name.ends_with("fe") {
        format!("{}ves", &name[..name.len() - 2])
    } else {
        format!("{}s", name)
    }
}

///
pub fn singularize(name: &str) -> String {
    if name.is_empty() {
        return name.to_string();
    }

    //
    let special: &[(&str, &str)] = &[
        ("knives", "knife"),
        ("staves", "staff"),
        ("teeth", "tooth"),
        ("feet", "foot"),
        ("geese", "goose"),
        ("mice", "mouse"),
        ("lice", "louse"),
        ("children", "child"),
        ("wolves", "wolf"),
        ("leaves", "leaf"),
        ("loaves", "loaf"),
        ("thieves", "thief"),
        ("vortices", "vortex"),
        ("fungi", "fungus"),
    ];

    for (plur, sing) in special {
        if name.ends_with(plur) {
            let base = &name[..name.len() - plur.len()];
            return format!("{}{}", base, sing);
        }
    }

    //
    if name.ends_with("ies") {
        format!("{}y", &name[..name.len() - 3])
    } else if name.ends_with("ves") {
        format!("{}f", &name[..name.len() - 3])
    } else if name.ends_with("ses")
        || name.ends_with("xes")
        || name.ends_with("zes")
        || name.ends_with("ches")
        || name.ends_with("shes")
    {
        name[..name.len() - 2].to_string()
    } else if name.ends_with('s') && !name.ends_with("ss") {
        name[..name.len() - 1].to_string()
    } else {
        name.to_string()
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn an(name: &str) -> String {
    if name.is_empty() {
        return name.to_string();
    }
    let first = name.chars().next().unwrap().to_ascii_lowercase();
    if "aeiou".contains(first) {
        format!("an {}", name)
    } else {
        format!("a {}", name)
    }
}

///
pub fn the(name: &str) -> String {
    format!("the {}", name)
}

///
pub fn an_cap(name: &str) -> String {
    let s = an(name);
    capitalize(&s)
}

///
pub fn the_cap(name: &str) -> String {
    capitalize(&the(name))
}

///
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn material_name(mat: Material) -> &'static str {
    match mat {
        Material::Liquid => "liquid",
        Material::Wax => "wax",
        Material::Veggy => "wooden",
        Material::Flesh => "flesh",
        Material::Paper => "paper",
        Material::Cloth => "cloth",
        Material::Leather => "leather",
        Material::Wood => "wooden",
        Material::Bone => "bone",
        Material::DragonHide => "dragon hide",
        Material::Iron => "iron",
        Material::Metal => "metal",
        Material::Copper => "copper",
        Material::Silver => "silver",
        Material::Gold => "gold",
        Material::Platinum => "platinum",
        Material::Mithril => "mithril",
        Material::Plastic => "plastic",
        Material::Glass => "glass",
        Material::Gemstone => "gemstone",
        Material::Mineral => "mineral",
    }
}

// =============================================================================
//
// =============================================================================

///
pub fn corpse_xname(monster_name: &str) -> String {
    format!("{} corpse", monster_name)
}

///
pub fn class_prefix(class: ItemClass) -> &'static str {
    match class {
        ItemClass::Weapon => "[weapon] ",
        ItemClass::Armor => "[armor] ",
        ItemClass::Ring => "[ring] ",
        ItemClass::Amulet => "[amulet] ",
        ItemClass::Tool => "[tool] ",
        ItemClass::Food => "[food] ",
        ItemClass::Potion => "[potion] ",
        ItemClass::Scroll => "[scroll] ",
        ItemClass::Spellbook => "[book] ",
        ItemClass::Wand => "[wand] ",
        ItemClass::Coin => "[coin] ",
        ItemClass::Gem => "[gem] ",
        _ => "",
    }
}

///
pub fn wand_charge_text(charges: i8, known: bool) -> String {
    if known {
        format!("({}:{})", 0, charges)
    } else {
        String::new()
    }
}

///
pub fn container_label(item_count: usize, locked: bool) -> String {
    let lock_str = if locked { " (locked)" } else { "" };
    if item_count == 0 {
        format!("(empty){}", lock_str)
    } else {
        format!(
            "({} item{}){}",
            item_count,
            if item_count != 1 { "s" } else { "" },
            lock_str
        )
    }
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pluralize() {
        assert_eq!(pluralize("knife"), "knives");
        assert_eq!(pluralize("sword"), "swords");
        assert_eq!(pluralize("mace"), "maces");
        assert_eq!(pluralize("torch"), "torches");
        assert_eq!(pluralize("staff"), "staves");
        assert_eq!(pluralize("potion"), "potions");
    }

    #[test]
    fn test_singularize() {
        assert_eq!(singularize("knives"), "knife");
        assert_eq!(singularize("swords"), "sword");
        assert_eq!(singularize("staves"), "staff");
    }

    #[test]
    fn test_articles() {
        assert_eq!(an("sword"), "a sword");
        assert_eq!(an("axe"), "an axe");
        assert_eq!(the("sword"), "the sword");
    }

    #[test]
    fn test_erosion() {
        assert_eq!(erosion_text(0, 0), "");
        assert_eq!(erosion_text(1, 0), "rusty");
        assert_eq!(erosion_text(2, 1), "very rusty burnt");
    }

    #[test]
    fn test_simple_typename() {
        assert_eq!(simple_typename(ItemClass::Weapon), "weapon");
        assert_eq!(simple_typename(ItemClass::Potion), "potion");
    }

    #[test]
    fn test_material_name() {
        assert_eq!(material_name(Material::Iron), "iron");
        assert_eq!(material_name(Material::Silver), "silver");
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub const POTION_APPEARANCES: &[&str] = &[
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
    "icy",
    "slimy",
    "silvery",
    "luminescent",
];

///
pub fn potion_appearance_name(appearance_idx: usize) -> String {
    if appearance_idx < POTION_APPEARANCES.len() {
        format!("{} potion", POTION_APPEARANCES[appearance_idx])
    } else {
        "strange potion".to_string()
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub const SCROLL_APPEARANCES: &[&str] = &[
    "ZELGO MER",
    "JUYED AWK YACC",
    "NR 9",
    "XIXAXA XOXAXA XUXAXA",
    "PRATYAVAYAH",
    "DAIYEN FANSEN",
    "LEP GEX VEN ZEA",
    "PRIRUTSENIE",
    "ELBIB YLANSEN",
    "VERR YED HULL",
    "VENZAR BORGAVVE",
    "THARR",
    "YUM YUM",
    "KERNOD WEL",
    "ELAM EANSEN",
    "DUAM XNAHT",
    "ANDOVA BEGARIN",
    "KIRJE",
    "VE FORBRANSEN",
    "HACKEM MUCHE",
    "VELOX NEB",
    "FOOBIE BLETCH",
    "TEMOV",
    "GARVEN DEH",
    "READ ME",
    "ETAOIN SHRDLU",
    "MAPIRO MAHAMA DIROMAT",
    "ASHPD",
    "ZLORFIK",
    "GNIK SANSEN",
];

///
pub fn scroll_appearance_name(appearance_idx: usize) -> String {
    if appearance_idx < SCROLL_APPEARANCES.len() {
        format!("scroll labeled {}", SCROLL_APPEARANCES[appearance_idx])
    } else {
        "unlabeled scroll".to_string()
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub const RING_APPEARANCES: &[&str] = &[
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
    "gold",
    "silver",
];

///
pub fn ring_appearance_name(appearance_idx: usize) -> String {
    if appearance_idx < RING_APPEARANCES.len() {
        format!("{} ring", RING_APPEARANCES[appearance_idx])
    } else {
        "plain ring".to_string()
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub const WAND_APPEARANCES: &[&str] = &[
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

///
pub fn wand_appearance_name(appearance_idx: usize) -> String {
    if appearance_idx < WAND_APPEARANCES.len() {
        format!("{} wand", WAND_APPEARANCES[appearance_idx])
    } else {
        "plain wand".to_string()
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn format_gold(amount: u64) -> String {
    if amount == 1 {
        "1 gold piece".to_string()
    } else {
        format!("{} gold pieces", amount)
    }
}

///
pub fn format_gold_compact(amount: u64) -> String {
    if amount >= 1_000_000 {
        format!("{}M", amount / 1_000_000)
    } else if amount >= 1_000 {
        format!("{}K", amount / 1_000)
    } else {
        format!("{}", amount)
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn quantity_label(name: &str, count: u32) -> String {
    match count {
        0 => format!("no {}", pluralize(name)),
        1 => an(name),
        2..=9 => format!("{} {}", count, pluralize(name)),
        _ => format!("{} {}", count, pluralize(name)),
    }
}

///
pub fn weight_label(weight: i32) -> String {
    if weight < 0 {
        "weightless".to_string()
    } else if weight == 0 {
        "negligible weight".to_string()
    } else {
        format!("{} aum", weight) // aum = arbitrary unit of measurement
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn buc_label(blessed: bool, cursed: bool, bknown: bool) -> &'static str {
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
pub fn full_display_name(
    base_name: &str,
    blessed: bool,
    cursed: bool,
    bknown: bool,
    spe: i8,
    known: bool,
    quantity: u32,
    eroded: u8,
    eroded2: u8,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    //
    if quantity > 1 {
        parts.push(format!("{}", quantity));
    }

    // BUC
    let buc = buc_label(blessed, cursed, bknown);
    if !buc.is_empty() {
        parts.push(buc.to_string());
    }

    //
    let erosion = erosion_text(eroded, eroded2);
    if !erosion.is_empty() {
        parts.push(erosion);
    }

    //
    if known && spe != 0 {
        parts.push(format!("{:+}", spe));
    }

    //
    if quantity > 1 {
        parts.push(pluralize(base_name));
    } else {
        parts.push(base_name.to_string());
    }

    parts.join(" ")
}

// =============================================================================
// [v2.3.2
// =============================================================================

///
pub fn can_merge_items(
    name_a: &str,
    name_b: &str,
    blessed_a: bool,
    blessed_b: bool,
    cursed_a: bool,
    cursed_b: bool,
    spe_a: i8,
    spe_b: i8,
    eroded_a: u8,
    eroded_b: u8,
) -> bool {
    //
    name_a == name_b
        && blessed_a == blessed_b
        && cursed_a == cursed_b
        && spe_a == spe_b
        && eroded_a == eroded_b
}

///
pub fn assign_inventory_letter(index: usize) -> char {
    if index < 26 {
        (b'a' + index as u8) as char
    } else if index < 52 {
        (b'A' + (index - 26) as u8) as char
    } else {
        '#'
    }
}

// =============================================================================
// [v2.3.2
// =============================================================================
#[cfg(test)]
mod objnam_extended_tests {
    use super::*;

    #[test]
    fn test_potion_appearance() {
        assert_eq!(potion_appearance_name(0), "ruby potion");
        assert_eq!(potion_appearance_name(14), "bubbly potion");
        assert_eq!(potion_appearance_name(999), "strange potion");
    }

    #[test]
    fn test_scroll_appearance() {
        assert_eq!(scroll_appearance_name(0), "scroll labeled ZELGO MER");
        assert_eq!(scroll_appearance_name(999), "unlabeled scroll");
    }

    #[test]
    fn test_ring_appearance() {
        assert_eq!(ring_appearance_name(0), "wooden ring");
    }

    #[test]
    fn test_wand_appearance() {
        assert_eq!(wand_appearance_name(0), "glass wand");
    }

    #[test]
    fn test_format_gold() {
        assert_eq!(format_gold(1), "1 gold piece");
        assert_eq!(format_gold(100), "100 gold pieces");
        assert_eq!(format_gold_compact(1_500_000), "1M");
        assert_eq!(format_gold_compact(5_000), "5K");
    }

    #[test]
    fn test_quantity_label() {
        assert_eq!(quantity_label("sword", 0), "no swords");
        assert_eq!(quantity_label("axe", 1), "an axe");
        assert_eq!(quantity_label("potion", 3), "3 potions");
    }

    #[test]
    fn test_buc_label() {
        assert_eq!(buc_label(true, false, true), "blessed");
        assert_eq!(buc_label(false, true, true), "cursed");
        assert_eq!(buc_label(false, false, true), "uncursed");
        assert_eq!(buc_label(true, false, false), "");
    }

    #[test]
    fn test_full_display_name() {
        let name = full_display_name("long sword", true, false, true, 3, true, 1, 0, 0);
        assert!(name.contains("blessed"));
        assert!(name.contains("+3"));
        assert!(name.contains("long sword"));
    }

    #[test]
    fn test_can_merge() {
        assert!(can_merge_items(
            "arrow", "arrow", false, false, false, false, 0, 0, 0, 0
        ));
        assert!(!can_merge_items(
            "arrow", "arrow", true, false, false, false, 0, 0, 0, 0
        ));
    }

    #[test]
    fn test_inventory_letter() {
        assert_eq!(assign_inventory_letter(0), 'a');
        assert_eq!(assign_inventory_letter(25), 'z');
        assert_eq!(assign_inventory_letter(26), 'A');
        assert_eq!(assign_inventory_letter(51), 'Z');
    }
}
