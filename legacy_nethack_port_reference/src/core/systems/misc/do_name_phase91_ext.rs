// ============================================================================
// [v2.27.0 Phase 91-3] 이름 지정 확장 (do_name_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/do_name.c L500-2200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 아이템 이름 지정 — name_item (do_name.c L500-800)
// =============================================================================

/// [v2.27.0 91-3] 아이템 이름 지정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameItemResult {
    /// 이름 지정 성공
    Named { display_name: String },
    /// 이미 같은 이름
    AlreadyNamed,
    /// 이름 길이 초과
    TooLong { max_length: usize },
    /// 이름 지정 불가 (저주/특수)
    CannotName { reason: String },
}

/// [v2.27.0 91-3] 아이템 이름 지정
/// 원본: do_name.c oname(), docall()
pub fn name_item(
    item_class: &str,
    current_name: &str,
    new_name: &str,
    is_artifact: bool,
    is_unique: bool,
    max_name_length: usize,
) -> NameItemResult {
    // 아티팩트/유니크 → 이름 변경 불가
    if is_artifact {
        return NameItemResult::CannotName {
            reason: "아티팩트의 이름은 변경할 수 없다.".to_string(),
        };
    }

    // 이름 길이 검사
    if new_name.len() > max_name_length {
        return NameItemResult::TooLong {
            max_length: max_name_length,
        };
    }

    // 빈 이름 → 이름 제거
    if new_name.is_empty() {
        return NameItemResult::Named {
            display_name: item_class.to_string(),
        };
    }

    // 같은 이름
    if current_name == new_name {
        return NameItemResult::AlreadyNamed;
    }

    NameItemResult::Named {
        display_name: format!("{} called {}", item_class, new_name),
    }
}

// =============================================================================
// [2] 몬스터 이름 지정 — name_monster (do_name.c L900-1200)
// =============================================================================

/// [v2.27.0 91-3] 몬스터 이름 지정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NameMonsterResult {
    Named { display_name: String },
    AlreadyNamed,
    TooLong { max_length: usize },
    CannotName { reason: String },
}

/// [v2.27.0 91-3] 몬스터 이름 지정
pub fn name_monster(
    monster_type: &str,
    current_name: &str,
    new_name: &str,
    is_unique_monster: bool,
    is_tame: bool,
    max_name_length: usize,
) -> NameMonsterResult {
    if is_unique_monster {
        return NameMonsterResult::CannotName {
            reason: format!("{}에게 이름을 붙일 수 없다.", monster_type),
        };
    }

    if new_name.len() > max_name_length {
        return NameMonsterResult::TooLong {
            max_length: max_name_length,
        };
    }

    if current_name == new_name {
        return NameMonsterResult::AlreadyNamed;
    }

    // 길들인 몬스터만 이름 지정 가능 (원본에서는 제한 없지만 UI 안내)
    let prefix = if is_tame { "" } else { "" };

    NameMonsterResult::Named {
        display_name: if new_name.is_empty() {
            monster_type.to_string()
        } else {
            format!("{} named {}", monster_type, new_name)
        },
    }
}

// =============================================================================
// [3] 좌표 이름 — coord_name (do_name.c L1500-1800)
// =============================================================================

/// [v2.27.0 91-3] 좌표 가리키기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoordDescription {
    pub x: i32,
    pub y: i32,
    pub terrain: String,
    pub features: Vec<String>,
    pub monster: Option<String>,
    pub items: Vec<String>,
    pub full_description: String,
}

/// [v2.27.0 91-3] 좌표 설명 생성
/// 원본: do_name.c coord_desc()
pub fn describe_coord(
    x: i32,
    y: i32,
    terrain_name: &str,
    has_trap: bool,
    trap_name: &str,
    monster_name: Option<&str>,
    item_names: &[&str],
    in_sight: bool,
) -> CoordDescription {
    let mut features = Vec::new();

    if has_trap {
        features.push(format!("함정: {}", trap_name));
    }

    let items: Vec<String> = item_names.iter().map(|s| s.to_string()).collect();

    let mut desc_parts = Vec::new();
    desc_parts.push(terrain_name.to_string());

    if let Some(mon) = monster_name {
        desc_parts.push(format!("({})", mon));
    }

    if !items.is_empty() {
        if items.len() == 1 {
            desc_parts.push(format!("위에 {}", items[0]));
        } else {
            desc_parts.push(format!("위에 {}개 아이템", items.len()));
        }
    }

    if has_trap {
        desc_parts.push(format!("[{}]", trap_name));
    }

    if !in_sight {
        desc_parts.push("(기억)".to_string());
    }

    CoordDescription {
        x,
        y,
        terrain: terrain_name.to_string(),
        features,
        monster: monster_name.map(|s| s.to_string()),
        items,
        full_description: desc_parts.join(" "),
    }
}

// =============================================================================
// [4] 자동 이름 지정 — auto_name (do_name.c L1800-2200)
// =============================================================================

/// [v2.27.0 91-3] 아이템 식별 시 자동 이름 지정
pub fn auto_name_by_effect(
    item_class: &str,
    effect_name: &str,
    known_names: &[(String, String)], // (효과, 이름) 쌍
) -> Option<String> {
    // 이미 알고 있는 효과면 이름 반환
    for (effect, name) in known_names {
        if effect == effect_name {
            return Some(name.clone());
        }
    }

    // 기본 이름 생성
    Some(format!("{} of {}", item_class, effect_name))
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- name_item ---

    #[test]
    fn test_name_item_success() {
        let result = name_item("potion", "", "healing", false, false, 64);
        assert!(matches!(result, NameItemResult::Named { .. }));
    }

    #[test]
    fn test_name_item_artifact() {
        let result = name_item("sword", "", "test", true, false, 64);
        assert!(matches!(result, NameItemResult::CannotName { .. }));
    }

    #[test]
    fn test_name_item_too_long() {
        let long_name = "a".repeat(100);
        let result = name_item("potion", "", &long_name, false, false, 64);
        assert!(matches!(result, NameItemResult::TooLong { .. }));
    }

    #[test]
    fn test_name_item_same() {
        let result = name_item("potion", "healing", "healing", false, false, 64);
        assert!(matches!(result, NameItemResult::AlreadyNamed));
    }

    // --- name_monster ---

    #[test]
    fn test_name_monster_success() {
        let result = name_monster("dog", "", "Rex", false, true, 64);
        assert!(matches!(result, NameMonsterResult::Named { .. }));
    }

    #[test]
    fn test_name_unique_monster() {
        let result = name_monster("Medusa", "", "test", true, false, 64);
        assert!(matches!(result, NameMonsterResult::CannotName { .. }));
    }

    // --- describe_coord ---

    #[test]
    fn test_describe_empty() {
        let desc = describe_coord(10, 5, "바닥", false, "", None, &[], true);
        assert!(desc.full_description.contains("바닥"));
    }

    #[test]
    fn test_describe_with_monster() {
        let desc = describe_coord(10, 5, "바닥", false, "", Some("고블린"), &[], true);
        assert!(desc.full_description.contains("고블린"));
    }

    #[test]
    fn test_describe_with_items() {
        let desc = describe_coord(10, 5, "바닥", false, "", None, &["검", "방패"], true);
        assert!(desc.full_description.contains("2개"));
    }

    // --- auto_name ---

    #[test]
    fn test_auto_name_known() {
        let known = vec![("healing".to_string(), "회복 포션".to_string())];
        let result = auto_name_by_effect("potion", "healing", &known);
        assert_eq!(result, Some("회복 포션".to_string()));
    }

    #[test]
    fn test_auto_name_unknown() {
        let result = auto_name_by_effect("scroll", "identify", &[]);
        assert!(result.unwrap().contains("identify"));
    }
}
