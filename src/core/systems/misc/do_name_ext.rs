// ============================================================================
// [v2.25.0 R13-5] 명명 시스템 (do_name_ext.rs)
// 원본: NetHack 3.6.7 do_name.c (1,820줄)
// 커스텀 명명, 좌표 이름, 몬스터 호칭
// ============================================================================

// =============================================================================
// [1] 커스텀 명명 (원본: do_name.c do_naming)
// =============================================================================

/// [v2.25.0 R13-5] 명명 타겟
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingTarget {
    /// 아이템 종류 전체 (call)
    ItemClass { class_id: i32, label: String },
    /// 개별 아이템 (name)
    SingleItem { item_id: u64, label: String },
    /// 몬스터 (name)
    Monster { monster_id: u64, label: String },
}

/// [v2.25.0 R13-5] 명명 유효성 검사
pub fn validate_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("이름이 비어 있습니다.".to_string());
    }
    if trimmed.len() > 63 {
        return Err("이름이 너무 깁니다 (최대 63자).".to_string());
    }
    // 특수 문자 필터링 (원본: 일부 문자 금지)
    if trimmed.contains('\0') || trimmed.contains('\n') {
        return Err("사용할 수 없는 문자가 포함되어 있습니다.".to_string());
    }
    Ok(trimmed.to_string())
}

// =============================================================================
// [2] 좌표 이름 (원본: do_name.c coord_desc, far_look)
// =============================================================================

/// [v2.25.0 R13-5] 좌표 설명 요소
#[derive(Debug, Clone)]
pub struct CoordDescription {
    /// 타일 설명
    pub terrain: String,
    /// 아이템 설명 (있으면)
    pub item: Option<String>,
    /// 몬스터 설명 (있으면)
    pub monster: Option<String>,
    /// 함정 설명 (있으면)
    pub trap: Option<String>,
    /// 각인 (있으면)
    pub engraving: Option<String>,
}

/// [v2.25.0 R13-5] 좌표 설명 포매팅 (원본: look_at)
pub fn format_coord_description(desc: &CoordDescription) -> String {
    let mut parts = vec![desc.terrain.clone()];

    if let Some(ref monster) = desc.monster {
        parts.push(format!("  {}", monster));
    }
    if let Some(ref item) = desc.item {
        parts.push(format!("  {}", item));
    }
    if let Some(ref trap) = desc.trap {
        parts.push(format!("  {}", trap));
    }
    if let Some(ref eng) = desc.engraving {
        parts.push(format!("  \"{}\"", eng));
    }

    parts.join("\n")
}

// =============================================================================
// [3] 몬스터 호칭 (원본: do_name.c x_monnam, Monnam)
// =============================================================================

/// [v2.25.0 R13-5] 몬스터 호칭 생성 모드
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonNameMode {
    /// 소문자 (a kobold)
    Normal,
    /// 대문자 (The kobold)
    Capitalized,
    /// 소문자 정관사 (the kobold)
    TheNormal,
    /// 이름만 (Fido)
    NameOnly,
}

/// [v2.25.0 R13-5] 몬스터 호칭 생성 (원본: x_monnam)
pub fn format_monster_name(
    name: &str,
    custom_name: Option<&str>,
    is_tame: bool,
    is_invisible: bool,
    is_hallucinating: bool,
    mode: MonNameMode,
) -> String {
    if is_hallucinating {
        return match mode {
            MonNameMode::Capitalized | MonNameMode::TheNormal => "Something".to_string(),
            _ => "something".to_string(),
        };
    }

    let mut result = String::new();

    // 접두어
    match mode {
        MonNameMode::Capitalized => result.push_str("The "),
        MonNameMode::TheNormal => result.push_str("the "),
        _ => {}
    }

    // 투명 접두어
    if is_invisible {
        result.push_str("invisible ");
    }

    // 이름
    if let Some(custom) = custom_name {
        if mode == MonNameMode::NameOnly {
            return custom.to_string();
        }
        result.push_str(name);
        result.push_str(&format!(" called {}", custom));
    } else {
        result.push_str(name);
    }

    // 길들임 후미
    if is_tame {
        result.push_str(" (tame)");
    }

    result
}

// =============================================================================
// [4] 방향 이름 (원본: do_name.c dirnam)
// =============================================================================

/// [v2.25.0 R13-5] 8방향 이름
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

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name_ok() {
        assert!(validate_name("Excalibur").is_ok());
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long = "a".repeat(64);
        assert!(validate_name(&long).is_err());
    }

    #[test]
    fn test_coord_desc() {
        let desc = CoordDescription {
            terrain: "stone floor".to_string(),
            item: Some("a long sword".to_string()),
            monster: None,
            trap: None,
            engraving: Some("Elbereth".to_string()),
        };
        let text = format_coord_description(&desc);
        assert!(text.contains("stone floor"));
        assert!(text.contains("long sword"));
        assert!(text.contains("Elbereth"));
    }

    #[test]
    fn test_mon_name_normal() {
        let name = format_monster_name("kobold", None, false, false, false, MonNameMode::Normal);
        assert_eq!(name, "kobold");
    }

    #[test]
    fn test_mon_name_capitalized() {
        let name = format_monster_name(
            "kobold",
            None,
            false,
            false,
            false,
            MonNameMode::Capitalized,
        );
        assert_eq!(name, "The kobold");
    }

    #[test]
    fn test_mon_name_invisible() {
        let name = format_monster_name("stalker", None, false, true, false, MonNameMode::Normal);
        assert!(name.contains("invisible"));
    }

    #[test]
    fn test_mon_name_tame() {
        let name =
            format_monster_name("dog", Some("Fido"), true, false, false, MonNameMode::Normal);
        assert!(name.contains("called Fido"));
        assert!(name.contains("(tame)"));
    }

    #[test]
    fn test_mon_name_halluc() {
        let name = format_monster_name("dragon", None, false, false, true, MonNameMode::Normal);
        assert_eq!(name, "something");
    }

    #[test]
    fn test_direction_name() {
        assert_eq!(direction_name(0, -1), "north");
        assert_eq!(direction_name(1, 1), "southeast");
        assert_eq!(direction_name(0, 0), "here");
    }
}
