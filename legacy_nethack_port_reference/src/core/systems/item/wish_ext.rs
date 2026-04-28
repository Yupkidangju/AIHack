// ============================================================================
// [v2.32.0 R20-1] 위시 파싱 (wish_ext.rs)
// 원본: NetHack 3.6.7 objnam.c readobjnam (위시 문자열→아이템)
// 아이템 소원 파싱, BUC/인챈트/수량 추출
// ============================================================================

/// [v2.32.0 R20-1] 위시 파싱 결과
#[derive(Debug, Clone, Default)]
pub struct WishParseResult {
    pub item_name: String,
    pub count: i32,
    pub enchantment: Option<i32>,
    pub blessed: bool,
    pub cursed: bool,
    pub erodeproof: bool,
    pub greased: bool,
    pub fixed: bool,
    pub raw_input: String,
}

/// [v2.32.0 R20-1] 위시 문자열 파싱 (원본: readobjnam)
pub fn parse_wish(input: &str) -> WishParseResult {
    let mut result = WishParseResult {
        raw_input: input.to_string(),
        count: 1,
        ..Default::default()
    };

    let mut tokens: Vec<&str> = input.split_whitespace().collect();
    if tokens.is_empty() {
        return result;
    }

    // 수량 파싱 ("3 blessed +2 long swords")
    if let Ok(n) = tokens[0].parse::<i32>() {
        result.count = n.clamp(1, 20);
        tokens.remove(0);
    }

    // BUC 파싱
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i].to_lowercase().as_str() {
            "blessed" | "holy" => {
                result.blessed = true;
                tokens.remove(i);
            }
            "cursed" | "unholy" => {
                result.cursed = true;
                tokens.remove(i);
            }
            "uncursed" => {
                tokens.remove(i);
            }
            "erodeproof" | "rustproof" | "fireproof" => {
                result.erodeproof = true;
                tokens.remove(i);
            }
            "greased" => {
                result.greased = true;
                tokens.remove(i);
            }
            "fixed" => {
                result.fixed = true;
                tokens.remove(i);
            }
            _ => {
                // 인챈트 (+N, -N)
                if let Some(rest) = tokens[i].strip_prefix('+') {
                    if let Ok(e) = rest.parse::<i32>() {
                        result.enchantment = Some(e.clamp(-5, 7));
                        tokens.remove(i);
                        continue;
                    }
                }
                if let Some(rest) = tokens[i].strip_prefix('-') {
                    if let Ok(e) = rest.parse::<i32>() {
                        result.enchantment = Some((-e).clamp(-5, 7));
                        tokens.remove(i);
                        continue;
                    }
                }
                i += 1;
            }
        }
    }

    result.item_name = tokens.join(" ");
    result
}

/// [v2.32.0 R20-1] 위시 금지 목록 (원본: 소원 불가 아이템)
pub fn is_wish_restricted(item_name: &str) -> bool {
    let restricted = [
        "amulet of yendor",
        "book of the dead",
        "candelabrum of invocation",
        "bell of opening",
        "quest artifact",
    ];
    let lower = item_name.to_lowercase();
    restricted.iter().any(|r| lower.contains(r))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_wish() {
        let r = parse_wish("long sword");
        assert_eq!(r.item_name, "long sword");
        assert_eq!(r.count, 1);
    }

    #[test]
    fn test_blessed_enchanted() {
        let r = parse_wish("blessed +3 silver dragon scale mail");
        assert!(r.blessed);
        assert_eq!(r.enchantment, Some(3));
        assert_eq!(r.item_name, "silver dragon scale mail");
    }

    #[test]
    fn test_count() {
        let r = parse_wish("3 blessed +2 darts");
        assert_eq!(r.count, 3);
        assert!(r.blessed);
        assert_eq!(r.enchantment, Some(2));
    }

    #[test]
    fn test_erodeproof() {
        let r = parse_wish("rustproof +2 long sword");
        assert!(r.erodeproof);
    }

    #[test]
    fn test_restricted() {
        assert!(is_wish_restricted("Amulet of Yendor"));
        assert!(!is_wish_restricted("long sword"));
    }
}
