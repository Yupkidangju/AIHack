// ============================================================================
// [v2.25.0 Phase 1-3] 옵션 시스템 확장 (options_phase1_ext.rs)
// 원본: NetHack 3.6.7 src/options.c L1500-3500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use std::collections::HashMap;

// =============================================================================
// [1] 옵션 검증 — option_validate (options.c L1500-1700)
// =============================================================================

/// [v2.25.0 1-3] 옵션 검증 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionValidation {
    /// 유효한 값
    Valid,
    /// 잘못된 값 (사유 포함)
    Invalid { reason: String },
    /// 읽기 전용 옵션 (변경 불가)
    ReadOnly,
    /// 알 수 없는 옵션
    UnknownOption,
}

/// [v2.25.0 1-3] 옵션 값 제약
#[derive(Debug, Clone)]
pub struct OptionConstraint {
    /// 옵션 이름
    pub name: String,
    /// 허용 값 유형
    pub value_type: ConstraintType,
    /// 읽기 전용 여부
    pub read_only: bool,
    /// 재시작 필요 여부
    pub requires_restart: bool,
}

/// [v2.25.0 1-3] 제약 유형
#[derive(Debug, Clone)]
pub enum ConstraintType {
    Boolean,
    IntRange { min: i32, max: i32 },
    StringEnum { allowed: Vec<String> },
    FreeText,
}

/// [v2.25.0 1-3] 옵션 검증
/// 원본: options.c doset() + option_validate()
pub fn validate_option(
    name: &str,
    value: &str,
    constraints: &[OptionConstraint],
) -> OptionValidation {
    let constraint = constraints.iter().find(|c| c.name == name);

    let constraint = match constraint {
        Some(c) => c,
        None => return OptionValidation::UnknownOption,
    };

    if constraint.read_only {
        return OptionValidation::ReadOnly;
    }

    match &constraint.value_type {
        ConstraintType::Boolean => {
            if matches!(
                value.to_lowercase().as_str(),
                "true" | "false" | "on" | "off" | "yes" | "no"
            ) {
                OptionValidation::Valid
            } else {
                OptionValidation::Invalid {
                    reason: format!("'{}' 옵션은 true/false만 허용됩니다.", name),
                }
            }
        }
        ConstraintType::IntRange { min, max } => match value.parse::<i32>() {
            Ok(n) if n >= *min && n <= *max => OptionValidation::Valid,
            Ok(n) => OptionValidation::Invalid {
                reason: format!(
                    "'{}' 옵션은 {}~{} 범위여야 합니다. (입력: {})",
                    name, min, max, n
                ),
            },
            Err(_) => OptionValidation::Invalid {
                reason: format!("'{}' 옵션은 숫자여야 합니다.", name),
            },
        },
        ConstraintType::StringEnum { allowed } => {
            if allowed.iter().any(|a| a.eq_ignore_ascii_case(value)) {
                OptionValidation::Valid
            } else {
                OptionValidation::Invalid {
                    reason: format!("'{}' 옵션 허용 값: {}", name, allowed.join(", ")),
                }
            }
        }
        ConstraintType::FreeText => OptionValidation::Valid,
    }
}

// =============================================================================
// [2] 기본 옵션 제약 테이블 — 원본 options.c 기반
// =============================================================================

/// [v2.25.0 1-3] 기본 옵션 제약 테이블
pub fn default_constraints() -> Vec<OptionConstraint> {
    vec![
        OptionConstraint {
            name: "autopickup".to_string(),
            value_type: ConstraintType::Boolean,
            read_only: false,
            requires_restart: false,
        },
        OptionConstraint {
            name: "color".to_string(),
            value_type: ConstraintType::Boolean,
            read_only: false,
            requires_restart: false,
        },
        OptionConstraint {
            name: "hilite_pet".to_string(),
            value_type: ConstraintType::Boolean,
            read_only: false,
            requires_restart: false,
        },
        OptionConstraint {
            name: "number_pad".to_string(),
            value_type: ConstraintType::Boolean,
            read_only: false,
            requires_restart: false,
        },
        OptionConstraint {
            name: "msghistory".to_string(),
            value_type: ConstraintType::IntRange { min: 1, max: 200 },
            read_only: false,
            requires_restart: false,
        },
        OptionConstraint {
            name: "pet_type".to_string(),
            value_type: ConstraintType::StringEnum {
                allowed: vec![
                    "dog".to_string(),
                    "cat".to_string(),
                    "none".to_string(),
                    "random".to_string(),
                ],
            },
            read_only: false,
            requires_restart: true,
        },
        OptionConstraint {
            name: "runmode".to_string(),
            value_type: ConstraintType::StringEnum {
                allowed: vec![
                    "teleport".to_string(),
                    "run".to_string(),
                    "walk".to_string(),
                    "crawl".to_string(),
                ],
            },
            read_only: false,
            requires_restart: false,
        },
        OptionConstraint {
            name: "pickup_burden".to_string(),
            value_type: ConstraintType::StringEnum {
                allowed: vec![
                    "unencumbered".to_string(),
                    "burdened".to_string(),
                    "stressed".to_string(),
                    "strained".to_string(),
                    "overtaxed".to_string(),
                    "overloaded".to_string(),
                ],
            },
            read_only: false,
            requires_restart: false,
        },
        OptionConstraint {
            name: "name".to_string(),
            value_type: ConstraintType::FreeText,
            read_only: true,
            requires_restart: false,
        },
        OptionConstraint {
            name: "race".to_string(),
            value_type: ConstraintType::FreeText,
            read_only: true,
            requires_restart: false,
        },
    ]
}

// =============================================================================
// [3] 옵션 표시 — doset (options.c L2000-2200)
// =============================================================================

/// [v2.25.0 1-3] 옵션 표시 항목
#[derive(Debug, Clone)]
pub struct OptionDisplay {
    pub name: String,
    pub current_value: String,
    pub is_boolean: bool,
    pub is_read_only: bool,
    pub description: String,
}

/// [v2.25.0 1-3] 현재 옵션 상태를 표시용으로 변환
pub fn options_to_display(
    current: &HashMap<String, String>,
    constraints: &[OptionConstraint],
) -> Vec<OptionDisplay> {
    let mut result: Vec<OptionDisplay> = constraints
        .iter()
        .map(|c| {
            let value = current
                .get(&c.name)
                .cloned()
                .unwrap_or_else(|| "(미설정)".to_string());
            let is_bool = matches!(c.value_type, ConstraintType::Boolean);
            OptionDisplay {
                name: c.name.clone(),
                current_value: value,
                is_boolean: is_bool,
                is_read_only: c.read_only,
                description: format_constraint_desc(&c.value_type),
            }
        })
        .collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

/// 제약 설명 문자열 생성
fn format_constraint_desc(ct: &ConstraintType) -> String {
    match ct {
        ConstraintType::Boolean => "true/false".to_string(),
        ConstraintType::IntRange { min, max } => format!("정수 ({}~{})", min, max),
        ConstraintType::StringEnum { allowed } => format!("[{}]", allowed.join("|")),
        ConstraintType::FreeText => "자유 텍스트".to_string(),
    }
}

// =============================================================================
// [4] 옵션 변경 이벤트 — options_change_effects (options.c L2500-2700)
// =============================================================================

/// [v2.25.0 1-3] 옵션 변경 부작용
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionSideEffect {
    /// 화면 갱신 필요
    RedrawRequired,
    /// 키 바인딩 재로드 필요
    RebindKeys,
    /// 재시작 필요
    RestartRequired,
    /// 없음
    None,
}

/// [v2.25.0 1-3] 옵션 변경 부작용 판정
pub fn option_change_effect(name: &str, constraints: &[OptionConstraint]) -> OptionSideEffect {
    let constraint = constraints.iter().find(|c| c.name == name);
    match constraint {
        Some(c) if c.requires_restart => OptionSideEffect::RestartRequired,
        _ => match name {
            "color" | "hilite_pet" | "lit_corridor" | "sparkle" => OptionSideEffect::RedrawRequired,
            "number_pad" => OptionSideEffect::RebindKeys,
            _ => OptionSideEffect::None,
        },
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn constraints() -> Vec<OptionConstraint> {
        default_constraints()
    }

    // --- validate_option ---

    #[test]
    fn test_validate_bool_ok() {
        let c = constraints();
        assert_eq!(
            validate_option("color", "true", &c),
            OptionValidation::Valid
        );
    }

    #[test]
    fn test_validate_bool_invalid() {
        let c = constraints();
        assert!(matches!(
            validate_option("color", "maybe", &c),
            OptionValidation::Invalid { .. }
        ));
    }

    #[test]
    fn test_validate_int_ok() {
        let c = constraints();
        assert_eq!(
            validate_option("msghistory", "50", &c),
            OptionValidation::Valid
        );
    }

    #[test]
    fn test_validate_int_out_of_range() {
        let c = constraints();
        assert!(matches!(
            validate_option("msghistory", "999", &c),
            OptionValidation::Invalid { .. }
        ));
    }

    #[test]
    fn test_validate_enum_ok() {
        let c = constraints();
        assert_eq!(
            validate_option("pet_type", "cat", &c),
            OptionValidation::Valid
        );
    }

    #[test]
    fn test_validate_enum_invalid() {
        let c = constraints();
        assert!(matches!(
            validate_option("pet_type", "dragon", &c),
            OptionValidation::Invalid { .. }
        ));
    }

    #[test]
    fn test_validate_readonly() {
        let c = constraints();
        assert_eq!(
            validate_option("name", "test", &c),
            OptionValidation::ReadOnly
        );
    }

    #[test]
    fn test_validate_unknown() {
        let c = constraints();
        assert_eq!(
            validate_option("nonexistent", "value", &c),
            OptionValidation::UnknownOption
        );
    }

    // --- options_to_display ---

    #[test]
    fn test_display_list() {
        let c = constraints();
        let mut current = HashMap::new();
        current.insert("color".to_string(), "true".to_string());
        let display = options_to_display(&current, &c);
        assert!(!display.is_empty());
        let color_opt = display.iter().find(|d| d.name == "color");
        assert!(color_opt.is_some());
    }

    // --- option_change_effect ---

    #[test]
    fn test_effect_redraw() {
        let c = constraints();
        assert_eq!(
            option_change_effect("color", &c),
            OptionSideEffect::RedrawRequired
        );
    }

    #[test]
    fn test_effect_rebind() {
        let c = constraints();
        assert_eq!(
            option_change_effect("number_pad", &c),
            OptionSideEffect::RebindKeys
        );
    }

    #[test]
    fn test_effect_restart() {
        let c = constraints();
        assert_eq!(
            option_change_effect("pet_type", &c),
            OptionSideEffect::RestartRequired
        );
    }

    #[test]
    fn test_effect_none() {
        let c = constraints();
        assert_eq!(
            option_change_effect("autopickup", &c),
            OptionSideEffect::None
        );
    }
}
