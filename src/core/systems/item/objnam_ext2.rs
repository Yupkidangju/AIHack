// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-22] 오브젝트 이름 확장2 (objnam_ext2.rs)
// 원본: NetHack 3.6.7 objnam.c (영어 문법: 관사, 복수형, 단수화)
// ============================================================================

// =============================================================================
// [1] 관사 판정 (원본: objnam.c An/an → just_an)
// =============================================================================

/// [v2.22.0 R34-22] 영어 부정관사 반환 ("a" 또는 "an")
/// 단어가 모음으로 시작하면 "an", 아니면 "a"
pub fn article_an(word: &str) -> &'static str {
    let first = word.chars().next().unwrap_or('x');
    if "aeiouAEIOU".contains(first) {
        "an"
    } else {
        "a"
    }
}

/// [v2.22.0 R34-22] "a/an + 단어" 전체 생성
pub fn a_or_an(word: &str) -> String {
    format!("{} {}", article_an(word), word)
}

// =============================================================================
// [2] 영어 복수형 규칙 (원본: objnam.c makeplural 약 200줄)
// =============================================================================

/// [v2.22.0 R34-22] 영어 명사를 복수형으로 변환 (원본: makeplural 간소화)
/// NetHack의 주요 복수형 규칙 이식
pub fn make_plural(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let lower = word.to_lowercase();

    // 예외 단어 (불규칙 복수형)
    let irregulars: &[(&str, &str)] = &[
        ("tooth", "teeth"),
        ("foot", "feet"),
        ("mouse", "mice"),
        ("louse", "lice"),
        ("child", "children"),
        ("goose", "geese"),
        ("ox", "oxen"),
        ("man", "men"),
        ("woman", "women"),
        ("knife", "knives"),
        ("wolf", "wolves"),
        ("leaf", "leaves"),
        ("loaf", "loaves"),
        ("thief", "thieves"),
        ("staff", "staves"),
        ("dwarf", "dwarves"),
        ("elf", "elves"),
        ("half", "halves"),
        ("shelf", "shelves"),
        ("fungus", "fungi"),
        ("cactus", "cacti"),
        ("vortex", "vortices"),
        ("zombie", "zombies"),
        ("mummy", "mummies"),
        ("jelly", "jellies"),
    ];

    for &(singular, plural) in irregulars {
        if lower == singular {
            // 원래 대소문자 보존
            if word.chars().next().unwrap().is_uppercase() {
                let mut p = plural.to_string();
                if let Some(c) = p.get_mut(0..1) {
                    c.make_ascii_uppercase();
                }
                return p;
            }
            return plural.to_string();
        }
    }

    // 변하지 않는 단어 (단복동형)
    let uncountable = [
        "sheep", "deer", "fish", "gold", "water", "lava", "kelp", "mail", "leather", "food", "acid",
    ];
    if uncountable.contains(&lower.as_str()) {
        return word.to_string();
    }

    // 규칙 적용
    if lower.ends_with("ss")
        || lower.ends_with("sh")
        || lower.ends_with("ch")
        || lower.ends_with("x")
        || lower.ends_with("z")
    {
        // bus → buses 제외, 이미 -ss인 경우도 포함
        format!("{}es", word)
    } else if lower.ends_with("us") {
        // status → statuses (라틴어 단어는 위 예외에서 처리)
        format!("{}es", word)
    } else if lower.ends_with('y') {
        let before_y = lower
            .as_bytes()
            .get(lower.len() - 2)
            .copied()
            .unwrap_or(b' ');
        if b"aeiou".contains(&before_y) {
            // boy → boys
            format!("{}s", word)
        } else {
            // city → cities
            format!("{}ies", &word[..word.len() - 1])
        }
    } else if lower.ends_with("fe") {
        // wife → wives
        format!("{}ves", &word[..word.len() - 2])
    } else if lower.ends_with('f') && !lower.ends_with("ff") {
        // scarf → scarves (하지만 staff는 예외에서 처리)
        format!("{}ves", &word[..word.len() - 1])
    } else if lower.ends_with('s') || lower.ends_with('o') {
        format!("{}es", word)
    } else {
        format!("{}s", word)
    }
}

// =============================================================================
// [3] 단수화 (원본: objnam.c makesingular 약 200줄)
// =============================================================================

/// [v2.22.0 R34-22] 영어 명사를 단수형으로 변환 (원본: makesingular 간소화)
pub fn make_singular(word: &str) -> String {
    if word.is_empty() {
        return String::new();
    }

    let lower = word.to_lowercase();

    // 불규칙 복수 → 단수
    let irregulars: &[(&str, &str)] = &[
        ("teeth", "tooth"),
        ("feet", "foot"),
        ("mice", "mouse"),
        ("lice", "louse"),
        ("children", "child"),
        ("geese", "goose"),
        ("oxen", "ox"),
        ("men", "man"),
        ("women", "woman"),
        ("knives", "knife"),
        ("wolves", "wolf"),
        ("leaves", "leaf"),
        ("loaves", "loaf"),
        ("thieves", "thief"),
        ("staves", "staff"),
        ("dwarves", "dwarf"),
        ("elves", "elf"),
        ("halves", "half"),
        ("shelves", "shelf"),
        ("fungi", "fungus"),
        ("cacti", "cactus"),
        ("vortices", "vortex"),
        ("zombies", "zombie"),
        ("mummies", "mummy"),
        ("jellies", "jelly"),
    ];

    for &(plural, singular) in irregulars {
        if lower == plural {
            if word.chars().next().unwrap().is_uppercase() {
                let mut s = singular.to_string();
                if let Some(c) = s.get_mut(0..1) {
                    c.make_ascii_uppercase();
                }
                return s;
            }
            return singular.to_string();
        }
    }

    // 규칙 역적용
    if lower.ends_with("ies") && lower.len() > 3 {
        // cities → city
        format!("{}y", &word[..word.len() - 3])
    } else if lower.ends_with("ves") && lower.len() > 3 {
        // knives → knife, wolves → wolf
        let stem = &word[..word.len() - 3];
        // "ves" → "fe" 또는 "f" 판정
        format!("{}f", stem)
    } else if lower.ends_with("sses")
        || lower.ends_with("shes")
        || lower.ends_with("ches")
        || lower.ends_with("xes")
        || lower.ends_with("zes")
    {
        // buses → bus
        word[..word.len() - 2].to_string()
    } else if lower.ends_with("uses") {
        // statuses → status
        word[..word.len() - 2].to_string()
    } else if lower.ends_with('s') && !lower.ends_with("ss") {
        // normal -s 제거
        word[..word.len() - 1].to_string()
    } else {
        word.to_string()
    }
}

// =============================================================================
// [4] 글롭 크기 설명 (원본: objnam.c xname_flags의 glob 관련 분기)
// =============================================================================

/// [v2.22.0 R34-22] 글롭 크기 설명 (원본: xname_flags의 glob 분기)
pub fn glob_size_prefix(weight: i32) -> &'static str {
    if weight <= 100 {
        "small "
    } else if weight > 500 {
        "very large "
    } else if weight > 300 {
        "large "
    } else {
        ""
    }
}

// =============================================================================
// [5] 일본어 아이템 이름 매핑 (원본: objnam.c:48-63 Japanese_items)
// =============================================================================

/// [v2.22.0 R34-22] 일본어 아이템 이름 테이블 (원본: Japanese_items)
pub fn japanese_item_name(item_type: &str) -> Option<&'static str> {
    match item_type {
        "short sword" => Some("wakizashi"),
        "broadsword" => Some("ninja-to"),
        "flail" => Some("nunchaku"),
        "glaive" => Some("naginata"),
        "long sword" => Some("katana"),
        "two-handed sword" => Some("tsurugi"),
        "helmet" => Some("kabuto"),
        "plate mail" => Some("tanko"),
        "lock pick" => Some("osaku"),
        "wooden harp" => Some("koto"),
        "knife" => Some("shito"),
        "booze" => Some("sake"),
        _ => None,
    }
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_an_vowel() {
        assert_eq!(article_an("arrow"), "an");
        assert_eq!(article_an("orc"), "an");
    }

    #[test]
    fn test_article_an_consonant() {
        assert_eq!(article_an("sword"), "a");
        assert_eq!(article_an("goblin"), "a");
    }

    #[test]
    fn test_a_or_an() {
        assert_eq!(a_or_an("arrow"), "an arrow");
        assert_eq!(a_or_an("sword"), "a sword");
    }

    #[test]
    fn test_plural_regular() {
        assert_eq!(make_plural("sword"), "swords");
        assert_eq!(make_plural("potion"), "potions");
    }

    #[test]
    fn test_plural_es() {
        assert_eq!(make_plural("torch"), "torches");
        assert_eq!(make_plural("box"), "boxes");
        assert_eq!(make_plural("glass"), "glasses");
    }

    #[test]
    fn test_plural_ies() {
        assert_eq!(make_plural("mummy"), "mummies");
        assert_eq!(make_plural("jelly"), "jellies");
    }

    #[test]
    fn test_plural_irregular() {
        assert_eq!(make_plural("tooth"), "teeth");
        assert_eq!(make_plural("foot"), "feet");
        assert_eq!(make_plural("knife"), "knives");
        assert_eq!(make_plural("elf"), "elves");
    }

    #[test]
    fn test_plural_uncountable() {
        assert_eq!(make_plural("sheep"), "sheep");
        assert_eq!(make_plural("gold"), "gold");
    }

    #[test]
    fn test_singular_regular() {
        assert_eq!(make_singular("swords"), "sword");
        assert_eq!(make_singular("potions"), "potion");
    }

    #[test]
    fn test_singular_ies() {
        assert_eq!(make_singular("mummies"), "mummy");
    }

    #[test]
    fn test_singular_irregular() {
        assert_eq!(make_singular("teeth"), "tooth");
        assert_eq!(make_singular("elves"), "elf");
    }

    #[test]
    fn test_glob_size() {
        assert_eq!(glob_size_prefix(50), "small ");
        assert_eq!(glob_size_prefix(200), "");
        assert_eq!(glob_size_prefix(400), "large ");
        assert_eq!(glob_size_prefix(600), "very large ");
    }

    #[test]
    fn test_japanese_items() {
        assert_eq!(japanese_item_name("long sword"), Some("katana"));
        assert_eq!(japanese_item_name("helmet"), Some("kabuto"));
        assert_eq!(japanese_item_name("unknown"), None);
    }
}
