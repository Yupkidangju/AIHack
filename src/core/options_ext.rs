// ============================================================================
// [v2.27.0 R15-1] 옵션 시스템 (options_ext.rs)
// 원본: NetHack 3.6.7 options.c (3,400줄)
// 게임 옵션 파싱, 검증, 기본값, autopickup 규칙
// ============================================================================

use std::collections::HashMap;

// =============================================================================
// [1] 옵션 값 타입
// =============================================================================

/// [v2.27.0 R15-1] 옵션 값
#[derive(Debug, Clone, PartialEq)]
pub enum OptionValue {
    Bool(bool),
    Int(i32),
    Str(String),
    List(Vec<String>),
}

/// [v2.27.0 R15-1] 옵션 정의
#[derive(Debug, Clone)]
pub struct OptionDef {
    pub name: String,
    pub description: String,
    pub default: OptionValue,
    pub category: OptionCategory,
}

/// [v2.27.0 R15-1] 옵션 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionCategory {
    Display,
    Gameplay,
    Interface,
    System,
}

// =============================================================================
// [2] 기본 옵션 (원본: options.c initoptions)
// =============================================================================

/// [v2.27.0 R15-1] 기본 옵션 세트
pub fn default_options() -> HashMap<String, OptionValue> {
    let mut opts = HashMap::new();
    opts.insert("autopickup".to_string(), OptionValue::Bool(true));
    opts.insert(
        "autopickup_types".to_string(),
        OptionValue::Str("$?!/=".to_string()),
    );
    opts.insert("color".to_string(), OptionValue::Bool(true));
    opts.insert("hilite_pet".to_string(), OptionValue::Bool(true));
    opts.insert("lit_corridor".to_string(), OptionValue::Bool(false));
    opts.insert(
        "msg_window".to_string(),
        OptionValue::Str("full".to_string()),
    );
    opts.insert("number_pad".to_string(), OptionValue::Bool(false));
    opts.insert("pet_type".to_string(), OptionValue::Str("dog".to_string()));
    opts.insert(
        "pickup_burden".to_string(),
        OptionValue::Str("unencumbered".to_string()),
    );
    opts.insert(
        "runmode".to_string(),
        OptionValue::Str("teleport".to_string()),
    );
    opts.insert("showexp".to_string(), OptionValue::Bool(true));
    opts.insert("showscore".to_string(), OptionValue::Bool(false));
    opts.insert("sortloot".to_string(), OptionValue::Str("full".to_string()));
    opts.insert("sparkle".to_string(), OptionValue::Bool(true));
    opts.insert("time".to_string(), OptionValue::Bool(true));
    opts
}

// =============================================================================
// [3] 옵션 파싱 (원본: options.c parseoptions)
// =============================================================================

/// [v2.27.0 R15-1] 옵션 문자열 파싱 (원본: parseoptions)
pub fn parse_option(input: &str) -> Result<(String, OptionValue), String> {
    let trimmed = input.trim();
    // !option → Bool false
    if let Some(name) = trimmed.strip_prefix('!') {
        return Ok((name.to_string(), OptionValue::Bool(false)));
    }
    // option:value
    if let Some((name, value)) = trimmed.split_once(':') {
        let name = name.trim().to_string();
        let value = value.trim();
        // 숫자 판별
        if let Ok(n) = value.parse::<i32>() {
            return Ok((name, OptionValue::Int(n)));
        }
        // true/false
        if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("on") {
            return Ok((name, OptionValue::Bool(true)));
        }
        if value.eq_ignore_ascii_case("false") || value.eq_ignore_ascii_case("off") {
            return Ok((name, OptionValue::Bool(false)));
        }
        return Ok((name, OptionValue::Str(value.to_string())));
    }
    // option 단독 → Bool true
    Ok((trimmed.to_string(), OptionValue::Bool(true)))
}

/// [v2.27.0 R15-1] RC 파일 여러 줄 파싱
pub fn parse_options_file(content: &str) -> Vec<Result<(String, OptionValue), String>> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|l| {
            let line = l.trim().strip_prefix("OPTIONS=").unwrap_or(l.trim());
            parse_option(line)
        })
        .collect()
}

// =============================================================================
// [4] autopickup 규칙 (원본: options.c add_autopickup_exception)
// =============================================================================

/// [v2.27.0 R15-1] autopickup 예외 규칙
#[derive(Debug, Clone)]
pub struct AutopickupRule {
    /// 패턴 (글로브)
    pub pattern: String,
    /// true = 줍기, false = 무시
    pub grab: bool,
}

/// [v2.27.0 R15-1] autopickup 판정
pub fn should_autopickup(
    item_name: &str,
    item_class: char,
    pickup_types: &str,
    rules: &[AutopickupRule],
) -> bool {
    // 규칙 우선 (마지막 매칭 규칙)
    for rule in rules.iter().rev() {
        if item_name.contains(&rule.pattern) {
            return rule.grab;
        }
    }
    // 기본: 클래스 기반
    pickup_types.contains(item_class)
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_options() {
        let opts = default_options();
        assert_eq!(opts.get("autopickup"), Some(&OptionValue::Bool(true)));
        assert_eq!(opts.get("color"), Some(&OptionValue::Bool(true)));
    }

    #[test]
    fn test_parse_bool_true() {
        let (name, val) = parse_option("color").unwrap();
        assert_eq!(name, "color");
        assert_eq!(val, OptionValue::Bool(true));
    }

    #[test]
    fn test_parse_bool_false() {
        let (name, val) = parse_option("!autopickup").unwrap();
        assert_eq!(name, "autopickup");
        assert_eq!(val, OptionValue::Bool(false));
    }

    #[test]
    fn test_parse_string() {
        let (name, val) = parse_option("pet_type:cat").unwrap();
        assert_eq!(name, "pet_type");
        assert_eq!(val, OptionValue::Str("cat".to_string()));
    }

    #[test]
    fn test_parse_int() {
        let (name, val) = parse_option("msghistory:20").unwrap();
        assert_eq!(val, OptionValue::Int(20));
    }

    #[test]
    fn test_autopickup_class() {
        assert!(should_autopickup("gold piece", '$', "$?!", &[]));
        assert!(!should_autopickup("corpse", '%', "$?!", &[]));
    }

    #[test]
    fn test_autopickup_rule_override() {
        let rules = vec![AutopickupRule {
            pattern: "corpse".to_string(),
            grab: true,
        }];
        assert!(should_autopickup("lichen corpse", '%', "$", &rules));
    }

    #[test]
    fn test_parse_file() {
        let content = "# comment\nOPTIONS=color\nOPTIONS=!autopickup\nOPTIONS=pet_type:cat\n";
        let results = parse_options_file(content);
        assert_eq!(results.len(), 3);
    }
}
