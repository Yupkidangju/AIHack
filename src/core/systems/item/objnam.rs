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

// =============================================================================
// [v2.4.0] objnam.c 대량 이식
// 원본: nethack-3.6.7/src/objnam.c (4,362줄)
// 일본어 아이템명, 유니크 판정, 동사 활용, 소유격, erosion 세분화 등
// =============================================================================

// ---------------------------------------------------------------------------
// 일본어 아이템명 테이블 (원본: objnam.c:49-61 Japanese_items)
// ---------------------------------------------------------------------------

/// [v2.4.0] 일본어 아이템명 매핑 (원본: Japanese_items[])
pub const JAPANESE_ITEMS: &[(&str, &str)] = &[
    ("short sword", "wakizashi"),
    ("broadsword", "ninja-to"),
    ("flail", "nunchaku"),
    ("glaive", "naginata"),
    ("lock pick", "osaku"),
    ("wooden harp", "koto"),
    ("knife", "shito"),
    ("plate mail", "tanko"),
    ("helmet", "kabuto"),
    ("leather gloves", "yugake"),
    ("food ration", "gunyoki"),
    ("potion of booze", "sake"),
];

/// [v2.4.0] 일본어 아이템명 조회 (원본: Japanese_item_name)
pub fn japanese_item_name(english_name: &str) -> Option<&'static str> {
    JAPANESE_ITEMS
        .iter()
        .find(|(eng, _)| eng.eq_ignore_ascii_case(english_name))
        .map(|(_, jpn)| *jpn)
}

// ---------------------------------------------------------------------------
// 유니크 아이템/몬스터 판정 (원본: objnam.c:828-865)
// ---------------------------------------------------------------------------

/// [v2.4.0] 유니크 아이템 판정 (원본: the_unique_obj)
pub fn the_unique_obj(is_unique: bool, known: bool, is_fake_amulet: bool) -> bool {
    if is_fake_amulet && !known {
        return true; // 거짓 부적 — 유니크인 척
    }
    is_unique && known
}

/// [v2.4.0] 유니크 몬스터 판정 (원본: the_unique_pm)
pub fn the_unique_pm(is_unique: bool, is_pname: bool) -> bool {
    if is_pname {
        return false;
    } // 고유명사면 "the" 불필요
    is_unique
}

// ---------------------------------------------------------------------------
// 동사 활용 시스템 (원본: objnam.c:2019-2147 vtense/otense)
// ---------------------------------------------------------------------------

/// [v2.4.0] 특수 주어 목록 — 복수형처럼 보이지만 단수 (원본: special_subjs)
pub const SPECIAL_SUBJECTS: &[&str] = &[
    "erinys",
    "manes",
    "Cyclops",
    "Hippocrates",
    "Pelias",
    "aklys",
    "amnesia",
    "detect monsters",
    "paralysis",
    "shape changers",
    "nemesis",
];

/// [v2.4.0] 동사 3인칭 단수 현재형 변환 (원본: vtense)
pub fn vtense(verb: &str) -> String {
    if verb.is_empty() {
        return verb.to_string();
    }

    // 특수 동사
    if verb.eq_ignore_ascii_case("are") {
        return "is".to_string();
    }
    if verb.eq_ignore_ascii_case("have") {
        return "has".to_string();
    }

    let last = verb.chars().last().unwrap();
    let len = verb.len();

    // z, x, s, ch, sh로 끝나면 "es" 추가
    if matches!(last, 'z' | 'x' | 's')
        || (len >= 2 && last == 'h' && matches!(verb.as_bytes()[len - 2], b'c' | b's'))
        || (len == 2 && last == 'o')
    {
        return format!("{}es", verb);
    }

    // 자음 + y → ies
    if last == 'y' && len >= 2 {
        let prev = verb.as_bytes()[len - 2] as char;
        if !"aeiou".contains(prev) {
            return format!("{}ies", &verb[..len - 1]);
        }
    }

    format!("{}s", verb)
}

/// [v2.4.0] 주어에 따른 동사 활용 (원본: otense)
pub fn otense(is_plural: bool, verb: &str) -> String {
    if is_plural {
        verb.to_string() // 복수 주어 → 동사 원형
    } else {
        vtense(verb) // 단수 주어 → 3인칭 단수
    }
}

// ---------------------------------------------------------------------------
// 소유격/표시 이름 보조 (원본: objnam.c:1787-1987)
// ---------------------------------------------------------------------------

/// [v2.4.0] 수량+이름+동사 결합 (원본: aobjnam)
pub fn aobjnam(name: &str, quantity: u32, verb: Option<&str>) -> String {
    let mut result = if quantity != 1 {
        format!("{} {}", quantity, pluralize(name))
    } else {
        name.to_string()
    };
    if let Some(v) = verb {
        let conjugated = otense(quantity != 1, v);
        result.push(' ');
        result.push_str(&conjugated);
    }
    result
}

/// [v2.4.0] "your" + 이름 조합 (원본: yname 간소화)
pub fn yname(name: &str, is_carried: bool) -> String {
    if is_carried {
        format!("your {}", name)
    } else {
        the(name)
    }
}

/// [v2.4.0] "The" + 이름 + 동사 조합 (원본: Tobjnam)
pub fn tobjnam(name: &str, verb: Option<&str>) -> String {
    let mut result = the_cap(name);
    if let Some(v) = verb {
        let conjugated = vtense(v);
        result.push(' ');
        result.push_str(&conjugated);
    }
    result
}

// ---------------------------------------------------------------------------
// Erosion 세분화 (원본: objnam.c:867-940)
// ---------------------------------------------------------------------------

/// [v2.4.0] 침식 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErosionType {
    /// 녹 (원본: is_rustprone)
    Rust,
    /// 부식 (원본: is_corrodeable)
    Corrode,
    /// 탄화 (원본: is_flammable → "burnt")
    Burn,
    /// 부패 (원본: oc_material == VEGGY → "rotted")
    Rot,
}

/// [v2.4.0] 침식 상태 상세 텍스트 (원본: add_erosion_words 확장)
pub fn erosion_detail_text(level: u8, etype: ErosionType) -> &'static str {
    let _prefix = match level {
        0 => return "",
        1 => "",
        2 => "very ",
        _ => "thoroughly ",
    };
    match (level, etype) {
        (1, ErosionType::Rust) => "rusty",
        (2, ErosionType::Rust) => "very rusty",
        (3, ErosionType::Rust) => "thoroughly rusty",
        (1, ErosionType::Corrode) => "corroded",
        (2, ErosionType::Corrode) => "very corroded",
        (3, ErosionType::Corrode) => "thoroughly corroded",
        (1, ErosionType::Burn) => "burnt",
        (2, ErosionType::Burn) => "very burnt",
        (3, ErosionType::Burn) => "thoroughly burnt",
        (1, ErosionType::Rot) => "rotted",
        (2, ErosionType::Rot) => "very rotted",
        (3, ErosionType::Rot) => "thoroughly rotted",
        _ => "",
    }
}

/// [v2.4.0] 침식 방지 텍스트 (원본: oerodeproof 처리)
pub fn erodeproof_text(etype: ErosionType, is_crysknife: bool) -> &'static str {
    if is_crysknife {
        return "fixed";
    }
    match etype {
        ErosionType::Rust => "rustproof",
        ErosionType::Corrode => "corrodeproof",
        ErosionType::Burn => "fireproof",
        ErosionType::Rot => "fixed",
    }
}

/// [v2.4.0] 침식이 유의미한지 판정 (원본: erosion_matters)
pub fn erosion_matters(class: ItemClass) -> bool {
    matches!(
        class,
        ItemClass::Weapon | ItemClass::Armor | ItemClass::Ball | ItemClass::Chain | ItemClass::Tool
    )
}

// ---------------------------------------------------------------------------
// 사인용 이름 (원본: objnam.c:1496-1561 killer_xname)
// ---------------------------------------------------------------------------

/// [v2.4.0] 사인 표시용 이름 (원본: killer_xname 간소화)
pub fn killer_xname(
    base_name: &str,
    is_artifact: bool,
    artifact_name: Option<&str>,
    is_unique: bool,
    is_pname: bool,
    quantity: u32,
) -> String {
    // 아티팩트면 아티팩트 이름 그대로
    if is_artifact {
        if let Some(aname) = artifact_name {
            return aname.to_string();
        }
    }

    let name = base_name.to_string();

    // 관사 적용
    if quantity == 1 {
        if is_pname || is_unique {
            the(&name)
        } else {
            an(&name)
        }
    } else {
        format!("{} {}", quantity, pluralize(&name))
    }
}

// ---------------------------------------------------------------------------
// corpse_xname 상세화 (원본: objnam.c:1380-1474)
// ---------------------------------------------------------------------------

/// [v2.4.0] 시체 상세 이름 (원본: corpse_xname 확장)
pub fn corpse_xname_detailed(
    monster_name: &str,
    adjective: Option<&str>,
    is_unique: bool,
    is_pname: bool,
    quantity: u32,
    omit_corpse: bool,
) -> String {
    let mut result = String::new();

    // 유니크 몬스터 "the" 접두어
    if is_unique && !is_pname {
        result.push_str("the ");
    }

    // 형용사와 몬스터명 결합
    if let Some(adj) = adjective {
        if is_pname {
            // 고유명사의 소유격 + 형용사: "Medusa's cursed corpse"
            result.push_str(&format!("{}'s {} ", monster_name, adj));
        } else {
            result.push_str(&format!("{} {} ", adj, monster_name));
        }
    } else {
        result.push_str(monster_name);
        result.push(' ');
    }

    // "corpse" 접미어
    if !omit_corpse {
        result.push_str("corpse");
        if quantity > 1 {
            result.push('s');
        }
    }

    result.trim().to_string()
}

// ---------------------------------------------------------------------------
// singplur 확장 — one_off 테이블 (원본: objnam.c:2156-2180)
// ---------------------------------------------------------------------------

/// [v2.4.0] 불규칙 단복수 변환 테이블 (원본: one_off[])
pub const ONE_OFF_PLURALS: &[(&str, &str)] = &[
    ("child", "children"),
    ("cubus", "cubi"), // incubus/succubus
    ("culus", "culi"), // homunculus
    ("djinni", "djinn"),
    ("erinys", "erinyes"),
    ("foot", "feet"),
    ("fungus", "fungi"),
    ("goose", "geese"),
    ("knife", "knives"),
    ("labrum", "labra"), // candelabrum
    ("louse", "lice"),
    ("mouse", "mice"),
    ("mumak", "mumakil"),
    ("nemesis", "nemeses"),
    ("ovum", "ova"),
    ("ox", "oxen"),
    ("passerby", "passersby"),
    ("rtex", "rtices"), // vortex
    ("serum", "sera"),
    ("staff", "staves"),
    ("tooth", "teeth"),
];

/// [v2.4.0] 불변 복수형 목록 (원본: as_is[])
pub const INVARIANT_PLURALS: &[&str] = &[
    "boots",
    "shoes",
    "gloves",
    "lenses",
    "scales",
    "eyes",
    "gauntlets",
    "iron bars",
    "bison",
    "deer",
    "elk",
    "fish",
    "fowl",
    "tuna",
    "yaki",
    "-hai",
    "krill",
    "manes",
    "moose",
    "ninja",
    "sheep",
    "ronin",
    "roshi",
    "shito",
    "tengu",
    "ki-rin",
    "Nazgul",
    "gunyoki",
    "piranha",
    "samurai",
    "shuriken",
];

/// [v2.4.0] 확장 복수화 — one_off 테이블 포함 (원본: makeplural 확장)
pub fn pluralize_extended(name: &str) -> String {
    if name.is_empty() {
        return name.to_string();
    }

    // 불변 복수 체크
    for inv in INVARIANT_PLURALS {
        if name.ends_with(inv) || name.eq_ignore_ascii_case(inv) {
            return name.to_string();
        }
    }

    // one_off 테이블 체크
    for (sing, plur) in ONE_OFF_PLURALS {
        if name.ends_with(sing) {
            let base = &name[..name.len() - sing.len()];
            return format!("{}{}", base, plur);
        }
    }

    // man → men (caveman → cavemen 등, 단 human/shaman은 제외)
    if name.ends_with("man") && !name.ends_with("human") && !name.ends_with("shaman") {
        return format!("{}en", &name[..name.len() - 2]);
    }

    // 기본 pluralize로 위임
    pluralize(name)
}

/// [v2.4.0] 확장 단수화 — one_off 테이블 포함 (원본: makesingular 확장)
pub fn singularize_extended(name: &str) -> String {
    if name.is_empty() {
        return name.to_string();
    }

    // 불변 체크
    for inv in INVARIANT_PLURALS {
        if name.ends_with(inv) || name.eq_ignore_ascii_case(inv) {
            return name.to_string();
        }
    }

    // one_off 역방향 체크
    for (sing, plur) in ONE_OFF_PLURALS {
        if name.ends_with(plur) {
            let base = &name[..name.len() - plur.len()];
            return format!("{}{}", base, sing);
        }
    }

    // men → man
    if name.ends_with("men") && !name.ends_with("omen") {
        return format!("{}an", &name[..name.len() - 2]);
    }

    singularize(name)
}

// ---------------------------------------------------------------------------
// compound 분리 (원본: objnam.c:2273-2306 singplur_compound)
// ---------------------------------------------------------------------------

/// [v2.4.0] 복합어 분리점 찾기 (원본: singplur_compound)
/// "lump of royal jelly" → Some(4) (" of " 위치)
pub fn find_compound_split(name: &str) -> Option<usize> {
    let compounds: &[&str] = &[
        " of ",
        " labeled ",
        " called ",
        " named ",
        " above",
        " versus ",
        " from ",
        " in ",
        " on ",
        " a la ",
        " with",
        " de ",
        " d'",
        " du ",
        "-in-",
        "-at-",
    ];
    for compound in compounds {
        if let Some(pos) = name.find(compound) {
            return Some(pos);
        }
    }
    None
}

/// [v2.4.0] 복합어의 주요 부분만 복수화 (원본: makeplural 내 compound 처리)
pub fn pluralize_compound(name: &str) -> String {
    if let Some(pos) = find_compound_split(name) {
        let main_part = &name[..pos];
        let rest = &name[pos..];
        format!("{}{}", pluralize_extended(main_part), rest)
    } else {
        pluralize_extended(name)
    }
}

// ---------------------------------------------------------------------------
// 관사 정밀 판정 (원본: objnam.c:1665-1691 just_an)
// ---------------------------------------------------------------------------

/// [v2.4.0] 정밀 관사 판정 (원본: just_an)
/// 단일 문자, "the"/"ice" 등 무관사, "unicorn" u-예외 등 처리
pub fn just_an(name: &str) -> &'static str {
    if name.is_empty() {
        return "a ";
    }

    let lower = name.to_ascii_lowercase();

    // 단일 문자
    if name.len() == 1 {
        return if "aefhilmnosx".contains(lower.as_bytes()[0] as char) {
            "an "
        } else {
            "a "
        };
    }

    // 무관사 특수 케이스
    if lower.starts_with("the ") || lower == "molten lava" || lower == "iron bars" || lower == "ice"
    {
        return "";
    }

    let first = lower.as_bytes()[0] as char;

    // u-예외 (unicorn, uranium, useful 등은 "a")
    if first == 'u'
        && (lower.starts_with("unicorn")
            || lower.starts_with("uranium")
            || lower.starts_with("useful"))
    {
        return "a ";
    }

    // 일반 모음
    if "aeiou".contains(first) && !lower.starts_with("one-") && !lower.starts_with("eucalyptus") {
        return "an ";
    }

    // x + 자음은 "an" (x-ray 등)
    if first == 'x' && name.len() >= 2 && !"aeiou".contains(name.as_bytes()[1] as char) {
        return "an ";
    }

    "a "
}

/// [v2.4.0] 정밀 관사가 적용된 an() (원본: an 확장)
pub fn an_precise(name: &str) -> String {
    let article = just_an(name);
    format!("{}{}", article, name)
}

// ---------------------------------------------------------------------------
// 테스트
// ---------------------------------------------------------------------------
#[cfg(test)]
mod objnam_v240_tests {
    use super::*;

    #[test]
    fn test_japanese_items() {
        assert_eq!(japanese_item_name("short sword"), Some("wakizashi"));
        assert_eq!(japanese_item_name("glaive"), Some("naginata"));
        assert_eq!(japanese_item_name("longsword"), None);
    }

    #[test]
    fn test_unique_obj() {
        assert!(the_unique_obj(true, true, false));
        assert!(!the_unique_obj(true, false, false));
        assert!(the_unique_obj(false, false, true)); // 가짜 부적
    }

    #[test]
    fn test_vtense() {
        assert_eq!(vtense("are"), "is");
        assert_eq!(vtense("have"), "has");
        assert_eq!(vtense("hit"), "hits");
        assert_eq!(vtense("miss"), "misses");
        assert_eq!(vtense("push"), "pushes");
        assert_eq!(vtense("carry"), "carries");
    }

    #[test]
    fn test_otense() {
        assert_eq!(otense(true, "hit"), "hit");
        assert_eq!(otense(false, "hit"), "hits");
    }

    #[test]
    fn test_aobjnam() {
        assert_eq!(aobjnam("arrow", 3, Some("hit")), "3 arrows hit");
        assert_eq!(aobjnam("arrow", 1, Some("hit")), "arrow hits");
        assert_eq!(aobjnam("sword", 1, None), "sword");
    }

    #[test]
    fn test_yname() {
        assert_eq!(yname("sword", true), "your sword");
        assert_eq!(yname("sword", false), "the sword");
    }

    #[test]
    fn test_tobjnam() {
        assert_eq!(tobjnam("wand", Some("glow")), "The wand glows");
    }

    #[test]
    fn test_erosion_detail() {
        assert_eq!(erosion_detail_text(1, ErosionType::Rust), "rusty");
        assert_eq!(
            erosion_detail_text(3, ErosionType::Corrode),
            "thoroughly corroded"
        );
        assert_eq!(erosion_detail_text(0, ErosionType::Burn), "");
    }

    #[test]
    fn test_erodeproof() {
        assert_eq!(erodeproof_text(ErosionType::Rust, false), "rustproof");
        assert_eq!(erodeproof_text(ErosionType::Burn, false), "fireproof");
        assert_eq!(erodeproof_text(ErosionType::Rust, true), "fixed");
    }

    #[test]
    fn test_killer_xname() {
        assert_eq!(
            killer_xname("long sword", false, None, false, false, 1),
            "a long sword"
        );
        assert_eq!(
            killer_xname("Excalibur", true, Some("Excalibur"), false, true, 1),
            "Excalibur"
        );
    }

    #[test]
    fn test_corpse_xname_detailed() {
        let name = corpse_xname_detailed("newt", None, false, false, 1, false);
        assert_eq!(name, "newt corpse");
        let name = corpse_xname_detailed("Oracle", None, true, false, 1, false);
        assert!(name.starts_with("the "));
    }

    #[test]
    fn test_pluralize_extended() {
        assert_eq!(pluralize_extended("homunculus"), "homunculi");
        assert_eq!(pluralize_extended("caveman"), "cavemen");
        assert_eq!(pluralize_extended("djinni"), "djinn");
        assert_eq!(pluralize_extended("samurai"), "samurai");
    }

    #[test]
    fn test_singularize_extended() {
        assert_eq!(singularize_extended("homunculi"), "homunculus");
        assert_eq!(singularize_extended("cavemen"), "caveman");
        assert_eq!(singularize_extended("djinn"), "djinni");
    }

    #[test]
    fn test_pluralize_compound() {
        assert_eq!(
            pluralize_compound("lump of royal jelly"),
            "lumps of royal jelly"
        );
        assert_eq!(
            pluralize_compound("scroll of identify"),
            "scrolls of identify"
        );
    }

    #[test]
    fn test_just_an() {
        assert_eq!(just_an("sword"), "a ");
        assert_eq!(just_an("axe"), "an ");
        assert_eq!(just_an("unicorn horn"), "a ");
        assert_eq!(just_an("iron bars"), "");
    }
}

// =============================================================================
// [v2.4.0] objnam.c 추가 대량 이식 #2
// 원본: nethack-3.6.7/src/objnam.c (2669-4362)
// wish 파싱, fuzzy match, 대체 철자, 장비 간이명 등
// =============================================================================

// ---------------------------------------------------------------------------
// Fuzzy match (원본: objnam.c:2669-2761 wishymatch)
// ---------------------------------------------------------------------------

/// [v2.4.0] 퍼지 문자열 비교 — 공백/하이픈/대소문자 무시 (원본: fuzzymatch)
pub fn fuzzymatch(a: &str, b: &str, ignore_chars: &str) -> bool {
    let normalize = |s: &str| -> String {
        s.chars()
            .filter(|c| !ignore_chars.contains(*c))
            .flat_map(|c| c.to_lowercase())
            .collect()
    };
    normalize(a) == normalize(b)
}

/// [v2.4.0] 소원 문자열 매칭 (원본: wishymatch)
/// "of" 반전 처리 포함 ("boots of speed" ↔ "speed boots")
pub fn wishymatch(user_str: &str, obj_str: &str, retry_inverted: bool) -> bool {
    // 공백/하이픈 무시 비교
    if fuzzymatch(user_str, obj_str, " -") {
        return true;
    }

    if retry_inverted {
        // "foo of bar" ↔ "bar foo" 반전 시도
        let u_of = user_str.to_lowercase().find(" of ");
        let o_of = obj_str.to_lowercase().find(" of ");

        match (u_of, o_of) {
            (Some(pos), None) => {
                // 사용자가 "foo of bar"로 입력 → "bar foo"로 변환
                let bar = &user_str[pos + 4..];
                let foo = &user_str[..pos];
                let inverted = format!("{} {}", bar, foo);
                return fuzzymatch(&inverted, obj_str, " -");
            }
            (None, Some(pos)) => {
                // 원본이 "foo of bar" → "bar foo"로 변환
                let bar = &obj_str[pos + 4..];
                let foo = &obj_str[..pos];
                let inverted = format!("{} {}", bar, foo);
                return fuzzymatch(user_str, &inverted, " -");
            }
            _ => {}
        }
    }

    // 특수 변환: dwarvish ↔ dwarven, elven ↔ elvish
    let ul = user_str.to_lowercase();
    let ol = obj_str.to_lowercase();
    if ol.starts_with("dwarvish ") && ul.starts_with("dwarven ") {
        return fuzzymatch(&ul[8..], &ol[9..], " -");
    }
    if ol.starts_with("elven ") {
        if ul.starts_with("elvish ") {
            return fuzzymatch(&ul[7..], &ol[6..], " -");
        } else if ul.starts_with("elfin ") {
            return fuzzymatch(&ul[6..], &ol[6..], " -");
        }
    }

    // detect ↔ detection 변환
    if ol.starts_with("detect ") {
        if let Some(pos) = ul.find(" detection") {
            if pos + " detection".len() == ul.len() {
                let detected = &ul[..pos];
                let converted = format!("detect {}", detected);
                return fuzzymatch(&converted, obj_str, " -");
            }
        }
    }

    // aluminum ↔ aluminium
    if ol == "aluminum" && ul == "aluminium" {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// 대체 철자 테이블 (원본: objnam.c:2797-2840 spellings)
// ---------------------------------------------------------------------------

/// [v2.4.0] 대체 철자 매핑 (원본: spellings[])
pub const ALT_SPELLINGS: &[(&str, &str)] = &[
    ("pickax", "pick-axe"),
    ("whip", "bullwhip"),
    ("saber", "silver saber"),
    ("silver sabre", "silver saber"),
    ("smooth shield", "shield of reflection"),
    ("grey dragon scale mail", "gray dragon scale mail"),
    ("grey dragon scales", "gray dragon scales"),
    ("iron ball", "heavy iron ball"),
    ("lantern", "brass lantern"),
    ("mattock", "dwarvish mattock"),
    ("amulet of poison resistance", "amulet versus poison"),
    ("potion of sleep", "potion of sleeping"),
    ("stone", "rock"),
    ("camera", "expensive camera"),
    ("tee shirt", "T-shirt"),
    ("can", "tin"),
    ("can opener", "tin opener"),
    ("kelp", "kelp frond"),
    ("eucalyptus", "eucalyptus leaf"),
    ("royal jelly", "lump of royal jelly"),
    ("lembas", "lembas wafer"),
    ("cookie", "fortune cookie"),
    ("pie", "cream pie"),
    ("marker", "magic marker"),
    ("hook", "grappling hook"),
    ("grappling iron", "grappling hook"),
    ("grapnel", "grappling hook"),
    ("grapple", "grappling hook"),
    ("box", "large box"),
    ("luck stone", "luckstone"),
    ("load stone", "loadstone"),
    ("touch stone", "touchstone"),
    ("flintstone", "flint"),
];

/// [v2.4.0] 대체 철자로 정식명 조회
pub fn lookup_alt_spelling(input: &str) -> Option<&'static str> {
    ALT_SPELLINGS
        .iter()
        .find(|(alt, _)| alt.eq_ignore_ascii_case(input))
        .map(|(_, canonical)| *canonical)
}

// ---------------------------------------------------------------------------
// 아이템 범위 분류 (원본: objnam.c:2763-2791 o_ranges)
// ---------------------------------------------------------------------------

/// [v2.4.0] 소원 하위 범위 (원본: o_ranges[])
pub const ITEM_RANGES: &[(&str, ItemClass)] = &[
    ("bag", ItemClass::Tool),
    ("lamp", ItemClass::Tool),
    ("candle", ItemClass::Tool),
    ("horn", ItemClass::Tool),
    ("shield", ItemClass::Armor),
    ("hat", ItemClass::Armor),
    ("helm", ItemClass::Armor),
    ("gloves", ItemClass::Armor),
    ("gauntlets", ItemClass::Armor),
    ("boots", ItemClass::Armor),
    ("shoes", ItemClass::Armor),
    ("cloak", ItemClass::Armor),
    ("shirt", ItemClass::Armor),
    ("dragon scales", ItemClass::Armor),
    ("dragon scale mail", ItemClass::Armor),
    ("sword", ItemClass::Weapon),
    ("venom", ItemClass::Venom),
    ("gray stone", ItemClass::Gem),
    ("grey stone", ItemClass::Gem),
];

/// [v2.4.0] 범위 이름으로 아이템 클래스 조회
pub fn lookup_item_range(name: &str) -> Option<ItemClass> {
    ITEM_RANGES
        .iter()
        .find(|(rname, _)| rname.eq_ignore_ascii_case(name))
        .map(|(_, class)| *class)
}

// ---------------------------------------------------------------------------
// 아이템 클래스 기호 매핑 (원본: objnam.c:2007-2017 wrp/wrpsym)
// ---------------------------------------------------------------------------

/// [v2.4.0] 클래스 이름 → 기호 매핑 (원본: wrp/wrpsym)
pub const CLASS_NAME_MAP: &[(&str, ItemClass)] = &[
    ("wand", ItemClass::Wand),
    ("ring", ItemClass::Ring),
    ("potion", ItemClass::Potion),
    ("scroll", ItemClass::Scroll),
    ("gem", ItemClass::Gem),
    ("amulet", ItemClass::Amulet),
    ("spellbook", ItemClass::Spellbook),
    ("spell book", ItemClass::Spellbook),
    ("weapon", ItemClass::Weapon),
    ("armor", ItemClass::Armor),
    ("tool", ItemClass::Tool),
    ("food", ItemClass::Food),
    ("comestible", ItemClass::Food),
];

/// [v2.4.0] 클래스 이름에서 아이템 클래스 조회
pub fn class_from_name(name: &str) -> Option<ItemClass> {
    CLASS_NAME_MAP
        .iter()
        .find(|(cname, _)| cname.eq_ignore_ascii_case(name))
        .map(|(_, class)| *class)
}

/// [v2.4.0] 단일 문자 아이템 클래스 기호 매핑 (원본: def_char_to_objclass)
pub fn class_from_char(ch: char) -> Option<ItemClass> {
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
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// 장비 간이명 (원본: objnam.c:4169-4250)
// ---------------------------------------------------------------------------

/// [v2.4.0] 갑옷 간이명 (원본: suit_simple_name)
pub fn suit_simple_name(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("dragon") && lower.contains("mail") {
        return "dragon mail";
    }
    if lower.contains("dragon") && lower.contains("scales") {
        return "dragon scales";
    }
    if lower.ends_with(" mail") {
        return "mail";
    }
    if lower.ends_with(" jacket") {
        return "jacket";
    }
    "suit"
}

/// [v2.4.0] 외투 간이명 (원본: cloak_simple_name)
pub fn cloak_simple_name(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("robe") {
        return "robe";
    }
    if lower.contains("mummy wrapping") {
        return "wrapping";
    }
    if lower.contains("alchemy smock") || lower.contains("apron") {
        return "smock";
    }
    "cloak"
}

/// [v2.4.0] 투구 간이명 (원본: helm_simple_name)
pub fn helm_simple_name(is_metallic: bool) -> &'static str {
    if is_metallic {
        "helm"
    } else {
        "hat"
    }
}

/// [v2.4.0] 장갑 간이명 (원본: gloves_simple_name)
pub fn gloves_simple_name(name: &str) -> &'static str {
    if name.to_lowercase().contains("gauntlet") {
        "gauntlets"
    } else {
        "gloves"
    }
}

// ---------------------------------------------------------------------------
// badman 판정 (원본: objnam.c:2621-2667)
// ---------------------------------------------------------------------------

/// [v2.4.0] man→men 변환 불가 접두어 (원본: no_men[])
pub const NO_MEN_PREFIXES: &[&str] = &[
    "albu",
    "antihu",
    "anti",
    "ata",
    "auto",
    "bildungsro",
    "cai",
    "cay",
    "ceru",
    "corner",
    "decu",
    "des",
    "dura",
    "fir",
    "hanu",
    "het",
    "infrahu",
    "inhu",
    "nonhu",
    "otto",
    "out",
    "prehu",
    "protohu",
    "subhu",
    "superhu",
    "talis",
    "unhu",
    "sha",
    "hu",
    "un",
    "le",
    "re",
    "so",
    "to",
    "at",
    "a",
];

/// [v2.4.0] men→man 변환 불가 접두어 (원본: no_man[])
pub const NO_MAN_PREFIXES: &[&str] = &[
    "abdo", "acu", "agno", "ceru", "cogno", "cycla", "fleh", "grava", "hegu", "preno", "sonar",
    "speci", "dai", "exa", "fla", "sta", "teg", "tegu", "vela", "da", "hy", "lu", "no", "nu", "ra",
    "ru", "se", "vi", "ya", "o", "a",
];

/// [v2.4.0] man↔men 변환이 무효한지 판정 (원본: badman)
pub fn badman(base: &str, to_plural: bool) -> bool {
    if base.len() < 4 {
        return false;
    }

    let prefixes = if to_plural {
        NO_MEN_PREFIXES
    } else {
        NO_MAN_PREFIXES
    };
    let suffix_len = 3; // "man" or "men"

    for prefix in prefixes {
        if base.len() >= prefix.len() + suffix_len {
            let start = base.len() - prefix.len() - suffix_len;
            let candidate = &base[start..start + prefix.len()];
            if candidate.eq_ignore_ascii_case(prefix) {
                // 단어 경계 확인 (시작이거나 앞에 공백)
                if start == 0 || base.as_bytes()[start - 1] == b' ' {
                    return true;
                }
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Wish 접두사 파싱 (원본: objnam.c:2930-3116 readobjnam 전반)
// ---------------------------------------------------------------------------

/// [v2.4.0] 소원 문자열에서 발견되는 접두사 정보 (원본: readobjnam 접두사 루프)
#[derive(Debug, Default)]
pub struct WishPrefixes {
    /// 수량
    pub count: u32,
    /// 특수 수치 (+/-)
    pub spe: i8,
    /// 수치 부호 (0=미지정, 1=양, -1=음)
    pub spe_sign: i8,
    /// 축복 여부
    pub blessed: bool,
    /// 저주 여부
    pub cursed: bool,
    /// 비저주 여부
    pub uncursed: bool,
    /// 부식방지 여부
    pub erodeproof: bool,
    /// 독칠 여부
    pub poisoned: bool,
    /// 기름칠 여부
    pub greased: bool,
    /// 1차 침식 레벨
    pub eroded: u8,
    /// 2차 침식 레벨
    pub eroded2: u8,
    /// 일부 먹은 상태
    pub halfeaten: bool,
    /// 점등 여부
    pub lit: Option<bool>,
    /// 빈 라벨 (blank/unlabeled)
    pub unlabeled: bool,
    /// 잠김/열림/파손 상태
    pub lock_state: Option<LockState>,
    /// 함정 여부
    pub trapped: Option<bool>,
    /// 희석 여부
    pub diluted: bool,
    /// 역사적 여부
    pub historic: bool,
    /// 나머지 이름 문자열
    pub remaining: String,
}

/// [v2.4.0] 잠금 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LockState {
    Locked,
    Unlocked,
    Broken,
}

/// [v2.4.0] 소원 문자열 접두사 파싱 (원본: readobjnam 루프 전반부)
pub fn parse_wish_prefixes(input: &str) -> WishPrefixes {
    let mut result = WishPrefixes::default();
    let mut s = input.trim().to_string();
    let mut very: u8 = 0;

    // 반복적으로 접두사 제거
    loop {
        let trimmed = s.trim_start();
        if trimmed.is_empty() {
            break;
        }

        // 관사 제거
        if let Some(rest) = trimmed
            .strip_prefix("an ")
            .or_else(|| trimmed.strip_prefix("a "))
        {
            if result.count == 0 {
                result.count = 1;
            }
            s = rest.to_string();
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("the ") {
            s = rest.to_string();
            continue;
        }

        // 수량
        if result.count == 0 {
            if let Some(end) = trimmed.find(|c: char| !c.is_ascii_digit()) {
                if end > 0 {
                    if let Ok(n) = trimmed[..end].parse::<u32>() {
                        result.count = n;
                        s = trimmed[end..].trim_start().to_string();
                        continue;
                    }
                }
            }
        }

        // +/- 수치
        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let sign = if trimmed.starts_with('+') { 1i8 } else { -1i8 };
            let rest = &trimmed[1..];
            if let Some(end) = rest.find(|c: char| !c.is_ascii_digit()) {
                if end > 0 {
                    if let Ok(n) = rest[..end].parse::<i8>() {
                        result.spe = n;
                        result.spe_sign = sign;
                        s = rest[end..].trim_start().to_string();
                        continue;
                    }
                }
            }
        }

        // 접두사 키워드 매칭
        let lower = trimmed.to_lowercase();
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "blessed ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "holy "))
        {
            result.blessed = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "cursed ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "unholy "))
        {
            result.cursed = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "uncursed ") {
            result.uncursed = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "rustproof ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "erodeproof "))
            .or_else(|| strip_prefix_ci(&lower, trimmed, "corrodeproof "))
            .or_else(|| strip_prefix_ci(&lower, trimmed, "fixed "))
            .or_else(|| strip_prefix_ci(&lower, trimmed, "fireproof "))
            .or_else(|| strip_prefix_ci(&lower, trimmed, "rotproof "))
        {
            result.erodeproof = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "poisoned ") {
            result.poisoned = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "greased ") {
            result.greased = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "lit ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "burning "))
        {
            result.lit = Some(true);
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "unlit ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "extinguished "))
        {
            result.lit = Some(false);
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "unlabeled ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "unlabelled "))
            .or_else(|| strip_prefix_ci(&lower, trimmed, "blank "))
        {
            result.unlabeled = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "locked ") {
            result.lock_state = Some(LockState::Locked);
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "unlocked ") {
            result.lock_state = Some(LockState::Unlocked);
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "broken ") {
            result.lock_state = Some(LockState::Broken);
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "very ") {
            very = 1;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "thoroughly ") {
            very = 2;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "rusty ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "rusted "))
            .or_else(|| strip_prefix_ci(&lower, trimmed, "burnt "))
            .or_else(|| strip_prefix_ci(&lower, trimmed, "burned "))
        {
            result.eroded = 1 + very;
            very = 0;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "corroded ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "rotted "))
        {
            result.eroded2 = 1 + very;
            very = 0;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "partly eaten ")
            .or_else(|| strip_prefix_ci(&lower, trimmed, "partially eaten "))
        {
            result.halfeaten = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "historic ") {
            result.historic = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "diluted ") {
            result.diluted = true;
            s = rest;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(&lower, trimmed, "empty ") {
            s = rest;
            continue;
        }

        // 인식 불가 → 루프 종료
        break;
    }

    if result.count == 0 {
        result.count = 1;
    }
    result.remaining = s;
    result
}

/// [v2.4.0] 대소문자 무시 접두사 제거 유틸리티
fn strip_prefix_ci(lower: &str, original: &str, prefix: &str) -> Option<String> {
    if lower.starts_with(prefix) {
        Some(original[prefix.len()..].to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// 소원 "named"/"called" 분리 (원본: readobjnam 3182-3199)
// ---------------------------------------------------------------------------

/// [v2.4.0] 소원 문자열에서 "named"/"called"/"labeled" 부분 분리
#[derive(Debug, Default)]
pub struct WishNameParts {
    /// 기본 아이템명
    pub base: String,
    /// "named XXX"
    pub named: Option<String>,
    /// "called XXX"
    pub called: Option<String>,
    /// "labeled XXX"
    pub labeled: Option<String>,
}

/// [v2.4.0] 소원 문자열에서 이름 부분 분리 (원본: readobjnam 이름 파싱)
pub fn parse_wish_name(input: &str) -> WishNameParts {
    let mut result = WishNameParts::default();
    let mut remaining = input.to_string();

    // " named " 분리
    if let Some(pos) = remaining.to_lowercase().find(" named ") {
        result.named = Some(remaining[pos + 7..].to_string());
        remaining = remaining[..pos].to_string();
    }

    // " called " 분리
    if let Some(pos) = remaining.to_lowercase().find(" called ") {
        result.called = Some(remaining[pos + 8..].to_string());
        remaining = remaining[..pos].to_string();
    }

    // " labeled " / " labelled " 분리
    if let Some(pos) = remaining.to_lowercase().find(" labeled ") {
        result.labeled = Some(remaining[pos + 9..].to_string());
        remaining = remaining[..pos].to_string();
    } else if let Some(pos) = remaining.to_lowercase().find(" labelled ") {
        result.labeled = Some(remaining[pos + 10..].to_string());
        remaining = remaining[..pos].to_string();
    }

    // "pair of" / "set of" 처리
    let lower_remaining = remaining.to_lowercase();
    if let Some(_rest) = lower_remaining.strip_prefix("pair of ") {
        remaining = remaining[8..].to_string();
    } else if let Some(_rest) = lower_remaining.strip_prefix("pairs of ") {
        remaining = remaining[9..].to_string();
    } else if let Some(_rest) = lower_remaining.strip_prefix("set of ") {
        remaining = remaining[7..].to_string();
    } else if let Some(_rest) = lower_remaining.strip_prefix("sets of ") {
        remaining = remaining[8..].to_string();
    }

    result.base = remaining.trim().to_string();
    result
}

// ---------------------------------------------------------------------------
// 테스트
// ---------------------------------------------------------------------------
#[cfg(test)]
mod objnam_v240_wish_tests {
    use super::*;

    #[test]
    fn test_fuzzymatch() {
        assert!(fuzzymatch("pick axe", "pick-axe", " -"));
        assert!(fuzzymatch("Pick Axe", "pick-axe", " -"));
        assert!(!fuzzymatch("sword", "axe", " -"));
    }

    #[test]
    fn test_wishymatch() {
        assert!(wishymatch("pick axe", "pick-axe", false));
        assert!(wishymatch("boots of speed", "speed boots", true));
        assert!(wishymatch("dwarven mattock", "dwarvish mattock", true));
        assert!(wishymatch("aluminium", "aluminum", true));
    }

    #[test]
    fn test_alt_spellings() {
        assert_eq!(lookup_alt_spelling("whip"), Some("bullwhip"));
        assert_eq!(lookup_alt_spelling("lantern"), Some("brass lantern"));
        assert_eq!(lookup_alt_spelling("nonexistent"), None);
    }

    #[test]
    fn test_item_range() {
        assert_eq!(lookup_item_range("shield"), Some(ItemClass::Armor));
        assert_eq!(lookup_item_range("sword"), Some(ItemClass::Weapon));
        assert_eq!(lookup_item_range("unknown"), None);
    }

    #[test]
    fn test_class_from_char() {
        assert_eq!(class_from_char('/'), Some(ItemClass::Wand));
        assert_eq!(class_from_char('['), Some(ItemClass::Armor));
        assert_eq!(class_from_char('Z'), None);
    }

    #[test]
    fn test_suit_simple_name() {
        assert_eq!(suit_simple_name("gray dragon scale mail"), "dragon mail");
        assert_eq!(suit_simple_name("plate mail"), "mail");
        assert_eq!(suit_simple_name("leather jacket"), "jacket");
        assert_eq!(suit_simple_name("strange armor"), "suit");
    }

    #[test]
    fn test_cloak_simple_name() {
        assert_eq!(cloak_simple_name("robe"), "robe");
        assert_eq!(cloak_simple_name("mummy wrapping"), "wrapping");
        assert_eq!(cloak_simple_name("elven cloak"), "cloak");
    }

    #[test]
    fn test_helm_simple_name() {
        assert_eq!(helm_simple_name(true), "helm");
        assert_eq!(helm_simple_name(false), "hat");
    }

    #[test]
    fn test_gloves_simple_name() {
        assert_eq!(gloves_simple_name("gauntlets of power"), "gauntlets");
        assert_eq!(gloves_simple_name("leather gloves"), "gloves");
    }

    #[test]
    fn test_badman() {
        assert!(badman("human", true)); // "human" → man→men 불가
        assert!(badman("shaman", true)); // "shaman" → man→men 불가
        assert!(!badman("caveman", true)); // "caveman" → cavemen 가능
    }

    #[test]
    fn test_parse_wish_prefixes() {
        let p = parse_wish_prefixes("blessed +3 rustproof long sword");
        assert!(p.blessed);
        assert_eq!(p.spe, 3);
        assert_eq!(p.spe_sign, 1);
        assert!(p.erodeproof);
        assert_eq!(p.remaining, "long sword");

        let p2 = parse_wish_prefixes("2 cursed poisoned arrows");
        assert_eq!(p2.count, 2);
        assert!(p2.cursed);
        assert!(p2.poisoned);
        assert_eq!(p2.remaining, "arrows");
    }

    #[test]
    fn test_parse_wish_name() {
        let n = parse_wish_name("long sword named Excalibur");
        assert_eq!(n.base, "long sword");
        assert_eq!(n.named, Some("Excalibur".to_string()));

        let n2 = parse_wish_name("scroll labeled ZELGO MER");
        assert_eq!(n2.base, "scroll");
        assert_eq!(n2.labeled, Some("ZELGO MER".to_string()));
    }
}
