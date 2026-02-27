// ============================================================================
// [v2.33.0 Phase 97-5] 옵션/설정 확장 (options_phase97_ext.rs)
// 원본: NetHack 3.6.7 src/options.c L300-1500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 게임 옵션 — game_options (options.c L300-800)
// =============================================================================

/// [v2.33.0 97-5] 게임 옵션 항목
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameOption {
    AutoPickup(bool),
    AutoPickupTypes(Vec<String>),
    NumberPad(bool),
    ShowInventWeight(bool),
    ShowExpBar(bool),
    ConfirmAttack(bool),
    SafePet(bool),
    Verbose(bool),
    SortPack(bool),
    PushWeapon(bool),
    SparkleEffect(bool),
    LitCorridor(bool),
    ShowScore(bool),
    ShowTime(bool),
    ShowHP(bool),
    ColorEnabled(bool),
    Hilite(Vec<String>),
    MessageLimit(i32),
    Name(String),
    Race(String),
    Role(String),
    Gender(String),
    Align(String),
}

/// [v2.33.0 97-5] 옵션 설정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptionSetResult {
    Set { option_name: String, value: String },
    InvalidValue { option_name: String, reason: String },
    ReadOnly { option_name: String },
    Unknown { option_name: String },
}

/// [v2.33.0 97-5] 옵션 파싱
/// 원본: options.c initoptions()
pub fn parse_option(line: &str) -> OptionSetResult {
    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        let parts2: Vec<&str> = line.splitn(2, '=').collect();
        if parts2.len() != 2 {
            return OptionSetResult::Unknown {
                option_name: line.to_string(),
            };
        }
        return parse_option_kv(parts2[0].trim(), parts2[1].trim());
    }
    parse_option_kv(parts[0].trim(), parts[1].trim())
}

fn parse_option_kv(key: &str, value: &str) -> OptionSetResult {
    match key.to_lowercase().as_str() {
        "autopickup" | "자동줍기" => {
            if value == "true" || value == "on" || value == "예" {
                OptionSetResult::Set {
                    option_name: "autopickup".to_string(),
                    value: "true".to_string(),
                }
            } else if value == "false" || value == "off" || value == "아니오" {
                OptionSetResult::Set {
                    option_name: "autopickup".to_string(),
                    value: "false".to_string(),
                }
            } else {
                OptionSetResult::InvalidValue {
                    option_name: "autopickup".to_string(),
                    reason: "true/false만 허용".to_string(),
                }
            }
        }
        "name" | "이름" => {
            if value.is_empty() {
                OptionSetResult::InvalidValue {
                    option_name: "name".to_string(),
                    reason: "이름이 비어있다.".to_string(),
                }
            } else {
                OptionSetResult::Set {
                    option_name: "name".to_string(),
                    value: value.to_string(),
                }
            }
        }
        "race" | "종족" => {
            let valid = [
                "인간",
                "엘프",
                "드워프",
                "그놈",
                "오크",
                "human",
                "elf",
                "dwarf",
                "gnome",
                "orc",
            ];
            if valid
                .iter()
                .any(|v| v.to_lowercase() == value.to_lowercase())
            {
                OptionSetResult::Set {
                    option_name: "race".to_string(),
                    value: value.to_string(),
                }
            } else {
                OptionSetResult::InvalidValue {
                    option_name: "race".to_string(),
                    reason: format!("유효한 종족: {:?}", valid),
                }
            }
        }
        "role" | "직업" => {
            let valid = [
                "전사",
                "마법사",
                "기사",
                "도적",
                "성직자",
                "레인저",
                "발키리",
                "수도승",
                "관광객",
                "barbarian",
                "wizard",
                "knight",
                "rogue",
                "priest",
                "ranger",
                "valkyrie",
                "monk",
                "tourist",
            ];
            if valid
                .iter()
                .any(|v| v.to_lowercase() == value.to_lowercase())
            {
                OptionSetResult::Set {
                    option_name: "role".to_string(),
                    value: value.to_string(),
                }
            } else {
                OptionSetResult::InvalidValue {
                    option_name: "role".to_string(),
                    reason: format!("유효한 직업: {:?}", valid),
                }
            }
        }
        "color" | "색상" => OptionSetResult::Set {
            option_name: "color".to_string(),
            value: value.to_string(),
        },
        "msglimit" | "메시지제한" => match value.parse::<i32>() {
            Ok(n) if n > 0 && n <= 1000 => OptionSetResult::Set {
                option_name: "msglimit".to_string(),
                value: value.to_string(),
            },
            _ => OptionSetResult::InvalidValue {
                option_name: "msglimit".to_string(),
                reason: "1~1000 범위의 숫자만 허용".to_string(),
            },
        },
        "version" | "버전" => OptionSetResult::ReadOnly {
            option_name: "version".to_string(),
        },
        _ => OptionSetResult::Unknown {
            option_name: key.to_string(),
        },
    }
}

// =============================================================================
// [2] 자동 줍기 설정 — autopickup (options.c L800-1200)
// =============================================================================

/// [v2.33.0 97-5] 자동 줍기 필터
#[derive(Debug, Clone)]
pub struct AutoPickupConfig {
    pub enabled: bool,
    pub types: Vec<String>,
    pub max_weight: i32,
    pub exclude_cursed: bool,
}

/// [v2.33.0 97-5] 자동 줍기 판정
pub fn should_autopickup(
    config: &AutoPickupConfig,
    item_type: &str,
    item_weight: i32,
    is_cursed: bool,
) -> bool {
    if !config.enabled {
        return false;
    }
    if config.exclude_cursed && is_cursed {
        return false;
    }
    if item_weight > config.max_weight {
        return false;
    }
    if config.types.is_empty() {
        return true;
    }
    config.types.iter().any(|t| t == item_type)
}

// =============================================================================
// [3] 키바인드 — keybind (options.c L1200-1500)
// =============================================================================

/// [v2.33.0 97-5] 키바인드 항목
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBind {
    pub key: String,
    pub command: String,
    pub is_default: bool,
}

/// [v2.33.0 97-5] 키바인드 재매핑 결과
pub fn rebind_key(key: &str, command: &str, existing_binds: &[KeyBind]) -> Result<KeyBind, String> {
    // 필수 키는 재매핑 불가
    let reserved = ["q", "Q", "S"];
    if reserved.contains(&key) {
        return Err(format!("'{}'는 예약된 키입니다.", key));
    }

    // 충돌 확인
    if existing_binds.iter().any(|b| b.key == key && b.is_default) {
        // 기본 키는 경고하지만 허용
    }

    Ok(KeyBind {
        key: key.to_string(),
        command: command.to_string(),
        is_default: false,
    })
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_autopickup() {
        let result = parse_option("autopickup:true");
        assert!(matches!(result, OptionSetResult::Set { .. }));
    }

    #[test]
    fn test_parse_name() {
        let result = parse_option("name=용사");
        if let OptionSetResult::Set { value, .. } = result {
            assert_eq!(value, "용사");
        }
    }

    #[test]
    fn test_parse_invalid_race() {
        let result = parse_option("race:트롤");
        assert!(matches!(result, OptionSetResult::InvalidValue { .. }));
    }

    #[test]
    fn test_parse_readonly() {
        let result = parse_option("version:1.0");
        assert!(matches!(result, OptionSetResult::ReadOnly { .. }));
    }

    #[test]
    fn test_parse_unknown() {
        let result = parse_option("foobar:baz");
        assert!(matches!(result, OptionSetResult::Unknown { .. }));
    }

    #[test]
    fn test_autopickup_enabled() {
        let config = AutoPickupConfig {
            enabled: true,
            types: vec!["금화".to_string()],
            max_weight: 50,
            exclude_cursed: true,
        };
        assert!(should_autopickup(&config, "금화", 10, false));
        assert!(!should_autopickup(&config, "금화", 10, true)); // 저주
        assert!(!should_autopickup(&config, "바위", 10, false)); // 유형 불일치
    }

    #[test]
    fn test_rebind_key() {
        let result = rebind_key("z", "cast", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rebind_reserved() {
        let result = rebind_key("q", "cast", &[]);
        assert!(result.is_err());
    }
}
