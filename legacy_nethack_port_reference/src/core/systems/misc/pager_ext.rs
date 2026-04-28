// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-7] 정보 표시 확장 모듈 (pager_ext.rs)
// 원본: NetHack 3.6.7 pager.c (순수 문자열 처리/판정 로직)
// ============================================================================

// =============================================================================
// [1] 상수 (원본: pager.c, display.h)
// =============================================================================

/// 몬스터 감지 방법 비트플래그 (원본: monseen.h)
pub const MONSEEN_NORMAL: u32 = 0x01;
pub const MONSEEN_SEEINVIS: u32 = 0x02;
pub const MONSEEN_INFRAVIS: u32 = 0x04;
pub const MONSEEN_TELEPAT: u32 = 0x08;
pub const MONSEEN_XRAYVIS: u32 = 0x10;
pub const MONSEEN_DETECT: u32 = 0x20;
pub const MONSEEN_WARNMON: u32 = 0x40;

// =============================================================================
// [2] 문자열 중복 추가 (원본: pager.c:52 append_str)
// =============================================================================

/// [v2.22.0 R34-7] 버퍼에 " or " + 새 문자열 추가 (이미 존재하면 건너뜀)
/// (원본: pager.c:52 append_str)
/// 반환: true면 추가됨
pub fn append_description(buf: &mut String, new_str: &str, max_len: usize) -> bool {
    // 이미 존재하면 건너뜀
    if buf.to_lowercase().contains(&new_str.to_lowercase()) {
        return false;
    }
    if buf.len() >= max_len {
        return false;
    }
    buf.push_str(" or ");
    let space = max_len.saturating_sub(buf.len());
    if new_str.len() <= space {
        buf.push_str(new_str);
    } else {
        buf.push_str(&new_str[..space]);
    }
    true
}

// =============================================================================
// [3] 이름 정규화 (원본: pager.c:600-659 checkfile의 접두사 제거 로직)
// =============================================================================

/// [v2.22.0 R34-7] 데이터 조회용 이름 정규화 (원본: checkfile 내 접두사 제거)
/// 관사, 수사, 상태 접두사, BUC 접두사, 인챈트 접두사를 제거하고
/// 조회 가능한 기본 이름만 남김
pub fn normalize_lookup_name(input: &str) -> String {
    let mut s = input.to_lowercase();

    // "interior of " 제거
    if let Some(rest) = s.strip_prefix("interior of ") {
        s = rest.to_string();
    }

    // 관사 제거
    for prefix in &["a ", "an ", "the ", "some "] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
            break;
        }
    }

    // 숫자 접두사 제거 ("2 ya" → "ya")
    if s.starts_with(|c: char| c.is_ascii_digit()) {
        let trimmed = s.trim_start_matches(|c: char| c.is_ascii_digit());
        s = trimmed.strip_prefix(' ').unwrap_or(trimmed).to_string();
    }

    // "pair of " 제거
    if let Some(rest) = s.strip_prefix("pair of ") {
        s = rest.to_string();
    }

    // 상태 접두사 제거 (길이 순 — 긴 것 우선)
    for prefix in &["peaceful ", "invisible ", "saddled ", "tame "] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
        }
    }

    // BUC 접두사 제거
    for prefix in &["uncursed ", "blessed ", "cursed "] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
            break;
        }
    }

    // 비어있음/부분 사용 접두사
    for prefix in &["empty ", "partly used ", "partly eaten "] {
        if let Some(rest) = s.strip_prefix(prefix) {
            s = rest.to_string();
        }
    }

    // "statue of X" → "statue"
    if s.starts_with("statue of ") {
        s = "statue".to_string();
    } else if s.starts_with("figurine of ") {
        s = "figurine".to_string();
    }

    // 인챈트 접두사 제거 ("+0 ", "-1 " 등)
    if s.starts_with('+') || s.starts_with('-') {
        if s.len() > 1 && s.chars().nth(1).map_or(false, |c| c.is_ascii_digit()) {
            let rest = s[1..].trim_start_matches(|c: char| c.is_ascii_digit());
            s = rest.strip_prefix(' ').unwrap_or(rest).to_string();
        }
    }

    // "moist towel" → "wet towel"
    if s.starts_with("moist towel") {
        s = s.replacen("moist", "wet", 1);
    }

    // " named X" → X 부분은 alt로 분리 (여기서는 " named " 이전만 반환)
    if let Some(idx) = s.find(" named ") {
        s.truncate(idx);
    } else if let Some(idx) = s.find(" called ") {
        s.truncate(idx);
    }

    // ", " 에서 잘라냄
    if let Some(idx) = s.find(", ") {
        s.truncate(idx);
    }

    // " (" 에서 잘라냄 (charges, lit 등)
    if let Some(idx) = s.find(" (") {
        s.truncate(idx);
    }

    s.trim().to_string()
}

// =============================================================================
// [4] 몬스터 감지 방법 설명 (원본: look_at_monster의 how_seen 비트 해석)
// =============================================================================

/// [v2.22.0 R34-7] 몬스터를 어떻게 감지했는지 설명 문자열 생성
/// (원본: pager.c:326-394 look_at_monster의 how_seen 비트)
pub fn describe_monseen(how_seen: u32, is_hallucinating: bool) -> Vec<&'static str> {
    let mut methods = Vec::new();
    let mut bits = how_seen;

    if bits & MONSEEN_NORMAL != 0 {
        methods.push("normal vision");
        bits &= !MONSEEN_NORMAL;
    }
    if bits & MONSEEN_SEEINVIS != 0 {
        methods.push("see invisible");
        bits &= !MONSEEN_SEEINVIS;
    }
    if bits & MONSEEN_INFRAVIS != 0 {
        methods.push("infravision");
        bits &= !MONSEEN_INFRAVIS;
    }
    if bits & MONSEEN_TELEPAT != 0 {
        methods.push("telepathy");
        bits &= !MONSEEN_TELEPAT;
    }
    if bits & MONSEEN_XRAYVIS != 0 {
        methods.push("astral vision");
        bits &= !MONSEEN_XRAYVIS;
    }
    if bits & MONSEEN_DETECT != 0 {
        methods.push("monster detection");
        bits &= !MONSEEN_DETECT;
    }
    if bits & MONSEEN_WARNMON != 0 {
        if is_hallucinating {
            methods.push("paranoid delusion");
        } else {
            methods.push("warned");
        }
        bits &= !MONSEEN_WARNMON;
    }
    // 알 수 없는 비트가 있으면 "unknown" 추가
    if bits != 0 {
        methods.push("unknown");
    }

    methods
}

/// [v2.22.0 R34-7] 자기 감지 방법 설명 (원본: lookat 내 자기 자신 감지)
/// `has_infravision`, `has_telepathy`, `has_detect_monsters`
pub fn describe_self_seen(
    has_infravision: bool,
    has_telepathy: bool,
    has_detect_monsters: bool,
) -> Option<String> {
    let mut how: u32 = 0;
    if has_infravision {
        how |= 1;
    }
    if has_telepathy {
        how |= 2;
    }
    if has_detect_monsters {
        how |= 4;
    }

    if how == 0 {
        return None;
    }

    let mut parts = Vec::new();
    if how & 1 != 0 {
        parts.push("infravision");
    }
    if how & 2 != 0 {
        parts.push("telepathy");
    }
    if how & 4 != 0 {
        parts.push("monster detection");
    }

    Some(format!("[seen: {}]", parts.join(", ")))
}

// =============================================================================
// [5] 오브젝트 위치 설명 (원본: look_at_object의 위치 접미사)
// =============================================================================

/// [v2.22.0 R34-7] 오브젝트 위치 접미사 (원본: pager.c:261-273 look_at_object)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectLocation {
    /// 일반 바닥
    Floor,
    /// 매장됨
    Buried,
    /// 돌에 박힘
    EmbeddedInStone,
    /// 벽에 박힘
    EmbeddedInWall,
    /// 문에 박힘
    EmbeddedInDoor,
    /// 물 속
    InWater,
    /// 용암 속
    InLava,
}

/// [v2.22.0 R34-7] 오브젝트 위치 접미사 문자열
pub fn object_location_suffix(loc: &ObjectLocation) -> &'static str {
    match loc {
        ObjectLocation::Floor => "",
        ObjectLocation::Buried => " (buried)",
        ObjectLocation::EmbeddedInStone => " embedded in stone",
        ObjectLocation::EmbeddedInWall => " embedded in a wall",
        ObjectLocation::EmbeddedInDoor => " embedded in a door",
        ObjectLocation::InWater => " in water",
        ObjectLocation::InLava => " in molten lava",
    }
}

// =============================================================================
// [6] 몬스터 상태 접두사 (원본: look_at_monster의 tame/peaceful/tail)
// =============================================================================

/// [v2.22.0 R34-7] 몬스터 설명 접두사 생성 (원본: look_at_monster)
pub fn monster_description_prefix(
    is_tame: bool,
    is_peaceful: bool,
    is_tail: bool,
    is_shopkeeper: bool,
    is_accurate: bool,
) -> String {
    let mut prefix = String::new();

    if is_tail {
        if is_shopkeeper && is_accurate {
            prefix.push_str("tail of ");
        } else {
            prefix.push_str("tail of a ");
        }
    }

    if is_tame && is_accurate {
        prefix.push_str("tame ");
    } else if is_peaceful && is_accurate {
        prefix.push_str("peaceful ");
    }

    prefix
}

// =============================================================================
// [7] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_description() {
        let mut buf = "sword".to_string();
        assert!(append_description(&mut buf, "axe", 256));
        assert_eq!(buf, "sword or axe");
        // 이미 존재하는 것은 추가하지 않음
        assert!(!append_description(&mut buf, "axe", 256));
    }

    #[test]
    fn test_append_description_maxlen() {
        let mut buf = "a".repeat(256);
        assert!(!append_description(&mut buf, "test", 256));
    }

    #[test]
    fn test_normalize_article() {
        assert_eq!(normalize_lookup_name("a sword"), "sword");
        assert_eq!(normalize_lookup_name("an amulet"), "amulet");
        assert_eq!(normalize_lookup_name("the Amulet"), "amulet");
        assert_eq!(normalize_lookup_name("some food"), "food");
    }

    #[test]
    fn test_normalize_prefix() {
        assert_eq!(normalize_lookup_name("tame kitten"), "kitten");
        assert_eq!(normalize_lookup_name("peaceful kobold"), "kobold");
        assert_eq!(normalize_lookup_name("invisible stalker"), "stalker");
    }

    #[test]
    fn test_normalize_buc() {
        assert_eq!(normalize_lookup_name("blessed +3 long sword"), "long sword");
        assert_eq!(normalize_lookup_name("cursed -1 dagger"), "dagger");
        assert_eq!(normalize_lookup_name("uncursed potion"), "potion");
    }

    #[test]
    fn test_normalize_statue() {
        assert_eq!(normalize_lookup_name("statue of a gnome"), "statue");
        assert_eq!(normalize_lookup_name("figurine of an orc"), "figurine");
    }

    #[test]
    fn test_normalize_named() {
        assert_eq!(normalize_lookup_name("sword named Excalibur"), "sword");
        assert_eq!(normalize_lookup_name("potion called healing"), "potion");
    }

    #[test]
    fn test_normalize_count() {
        assert_eq!(normalize_lookup_name("2 ya"), "ya");
        assert_eq!(normalize_lookup_name("15 gold pieces"), "gold pieces");
    }

    #[test]
    fn test_normalize_charges() {
        assert_eq!(normalize_lookup_name("wand of fire (0:4)"), "wand of fire");
    }

    #[test]
    fn test_normalize_moist() {
        assert_eq!(normalize_lookup_name("moist towel"), "wet towel");
    }

    #[test]
    fn test_describe_monseen_normal() {
        let methods = describe_monseen(MONSEEN_NORMAL, false);
        assert_eq!(methods, vec!["normal vision"]);
    }

    #[test]
    fn test_describe_monseen_multi() {
        let methods = describe_monseen(MONSEEN_SEEINVIS | MONSEEN_TELEPAT | MONSEEN_DETECT, false);
        assert_eq!(
            methods,
            vec!["see invisible", "telepathy", "monster detection"]
        );
    }

    #[test]
    fn test_describe_monseen_hallu() {
        let methods = describe_monseen(MONSEEN_WARNMON, true);
        assert_eq!(methods, vec!["paranoid delusion"]);
    }

    #[test]
    fn test_describe_self_seen() {
        assert!(describe_self_seen(false, false, false).is_none());
        assert_eq!(
            describe_self_seen(true, false, false).unwrap(),
            "[seen: infravision]"
        );
        assert_eq!(
            describe_self_seen(true, true, true).unwrap(),
            "[seen: infravision, telepathy, monster detection]"
        );
    }

    #[test]
    fn test_object_location() {
        assert_eq!(object_location_suffix(&ObjectLocation::Floor), "");
        assert_eq!(object_location_suffix(&ObjectLocation::Buried), " (buried)");
        assert_eq!(
            object_location_suffix(&ObjectLocation::InWater),
            " in water"
        );
    }

    #[test]
    fn test_monster_prefix_tame() {
        let prefix = monster_description_prefix(true, false, false, false, true);
        assert_eq!(prefix, "tame ");
    }

    #[test]
    fn test_monster_prefix_tail() {
        let prefix = monster_description_prefix(false, true, true, false, true);
        assert_eq!(prefix, "tail of a peaceful ");
    }

    #[test]
    fn test_monster_prefix_shopkeeper_tail() {
        let prefix = monster_description_prefix(false, false, true, true, true);
        assert_eq!(prefix, "tail of ");
    }

    #[test]
    fn test_normalize_interior() {
        assert_eq!(
            normalize_lookup_name("interior of a purple worm"),
            "purple worm"
        );
    }
}
