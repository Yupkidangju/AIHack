// ============================================================================
// [v2.25.0 R13-3] 아이템 명명 확장 (objnam_ext.rs)
// 원본: NetHack 3.6.7 objnam.c (4,438줄)
// 아이템 이름 파싱, 외관 시스템, BUC 포매팅
// ============================================================================

// =============================================================================
// [1] BUC 상태 표시 (원본: objnam.c doname, xprname)
// =============================================================================

/// [v2.25.0 R13-3] BUC 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucStatus {
    Blessed,
    Uncursed,
    Cursed,
    Unknown,
}

/// [v2.25.0 R13-3] BUC 접두어
pub fn buc_prefix(status: BucStatus) -> &'static str {
    match status {
        BucStatus::Blessed => "blessed ",
        BucStatus::Uncursed => "uncursed ",
        BucStatus::Cursed => "cursed ",
        BucStatus::Unknown => "",
    }
}

// =============================================================================
// [2] 아이템 이름 포매팅 (원본: objnam.c doname2)
// =============================================================================

/// [v2.25.0 R13-3] 아이템 명명 입력
#[derive(Debug, Clone)]
pub struct ItemNameInput {
    pub base_name: String,
    pub quantity: i32,
    pub enchantment: Option<i32>,
    pub buc: BucStatus,
    pub identified: bool,
    pub custom_name: Option<String>,
    pub appearance: Option<String>,
    pub item_class: char,
    pub eroded: i32,
    pub greased: bool,
}

/// [v2.25.0 R13-3] 포맷된 아이템 이름 생성 (원본: doname2)
pub fn format_item_name(input: &ItemNameInput) -> String {
    let mut parts: Vec<String> = Vec::new();

    // BUC (식별 시)
    if input.identified {
        let prefix = buc_prefix(input.buc);
        if !prefix.is_empty() {
            parts.push(prefix.trim().to_string());
        }
    }

    // 강화 수치
    if let Some(ench) = input.enchantment {
        if input.identified {
            parts.push(format!("{:+}", ench));
        }
    }

    // 그리스
    if input.greased {
        parts.push("greased".to_string());
    }

    // 부식
    if input.eroded > 0 {
        let erode_word = match input.eroded {
            1 => "rusty",
            2 => "very rusty",
            3 => "thoroughly rusty",
            _ => "rusty",
        };
        parts.push(erode_word.to_string());
    }

    // 이름 (외관 또는 기본)
    if !input.identified {
        if let Some(ref app) = input.appearance {
            parts.push(app.clone());
        } else {
            parts.push(input.base_name.clone());
        }
    } else {
        parts.push(input.base_name.clone());
    }

    // 수량
    let name = parts.join(" ");
    let name = if input.quantity > 1 {
        format!("{} {}", input.quantity, name)
    } else {
        name
    };

    // 커스텀 이름
    if let Some(ref custom) = input.custom_name {
        format!("{} named {}", name, custom)
    } else {
        name
    }
}

// =============================================================================
// [3] 외관(Appearance) 시스템 (원본: objnam.c shuffled descriptions)
// =============================================================================

/// [v2.25.0 R13-3] 포션 외관 테이블 (원본: objnam.c potion_descr)
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
];

/// [v2.25.0 R13-3] 스크롤 외관 테이블
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
    "VERR YLAND",
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
];

/// [v2.25.0 R13-3] 외관 이름으로 포션 인덱스 검색
pub fn find_potion_by_appearance(appearance: &str) -> Option<usize> {
    let lower = appearance.to_lowercase();
    POTION_APPEARANCES.iter().position(|&a| a == lower)
}

// =============================================================================
// [4] 아이템 파싱 (원본: objnam.c readobjnam — 위저드 모드)
// =============================================================================

/// [v2.25.0 R13-3] 아이템 이름 파서 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedItem {
    pub name: String,
    pub quantity: i32,
    pub enchantment: Option<i32>,
    pub buc: BucStatus,
}

/// [v2.25.0 R13-3] 아이템 이름 파싱 (위저드 모드용)
pub fn parse_item_name(input: &str) -> ParsedItem {
    let mut text = input.trim().to_string();
    let mut quantity = 1;
    let mut enchantment = None;
    let mut buc = BucStatus::Unknown;

    // BUC 파싱
    if text.starts_with("blessed ") {
        buc = BucStatus::Blessed;
        text = text[8..].to_string();
    } else if text.starts_with("cursed ") {
        buc = BucStatus::Cursed;
        text = text[7..].to_string();
    } else if text.starts_with("uncursed ") {
        buc = BucStatus::Uncursed;
        text = text[9..].to_string();
    }

    // 강화 수치 파싱 (+N, -N)
    if let Some(rest) = text.strip_prefix('+') {
        if let Some(space_idx) = rest.find(' ') {
            if let Ok(n) = rest[..space_idx].parse::<i32>() {
                enchantment = Some(n);
                text = rest[space_idx + 1..].to_string();
            }
        }
    } else if let Some(rest) = text.strip_prefix('-') {
        if let Some(space_idx) = rest.find(' ') {
            if let Ok(n) = rest[..space_idx].parse::<i32>() {
                enchantment = Some(-n);
                text = rest[space_idx + 1..].to_string();
            }
        }
    }

    // 수량 파싱 (숫자 시작)
    if let Some(space_idx) = text.find(' ') {
        if let Ok(n) = text[..space_idx].parse::<i32>() {
            quantity = n;
            text = text[space_idx + 1..].to_string();
        }
    }

    ParsedItem {
        name: text,
        quantity,
        enchantment,
        buc,
    }
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buc_prefix() {
        assert_eq!(buc_prefix(BucStatus::Blessed), "blessed ");
        assert_eq!(buc_prefix(BucStatus::Unknown), "");
    }

    #[test]
    fn test_format_basic() {
        let input = ItemNameInput {
            base_name: "long sword".to_string(),
            quantity: 1,
            enchantment: Some(2),
            buc: BucStatus::Blessed,
            identified: true,
            custom_name: None,
            appearance: None,
            item_class: ')',
            eroded: 0,
            greased: false,
        };
        let name = format_item_name(&input);
        assert!(name.contains("blessed"));
        assert!(name.contains("+2"));
        assert!(name.contains("long sword"));
    }

    #[test]
    fn test_format_unidentified() {
        let input = ItemNameInput {
            base_name: "potion of healing".to_string(),
            quantity: 2,
            enchantment: None,
            buc: BucStatus::Unknown,
            identified: false,
            custom_name: None,
            appearance: Some("pink potion".to_string()),
            item_class: '!',
            eroded: 0,
            greased: false,
        };
        let name = format_item_name(&input);
        assert!(name.contains("2 pink potion"));
        assert!(!name.contains("healing"));
    }

    #[test]
    fn test_format_named() {
        let input = ItemNameInput {
            base_name: "katana".to_string(),
            quantity: 1,
            enchantment: Some(4),
            buc: BucStatus::Uncursed,
            identified: true,
            custom_name: Some("Snickersnee".to_string()),
            appearance: None,
            item_class: ')',
            eroded: 0,
            greased: false,
        };
        let name = format_item_name(&input);
        assert!(name.contains("named Snickersnee"));
    }

    #[test]
    fn test_format_eroded() {
        let input = ItemNameInput {
            base_name: "iron helmet".to_string(),
            quantity: 1,
            enchantment: None,
            buc: BucStatus::Unknown,
            identified: true,
            custom_name: None,
            appearance: None,
            item_class: '[',
            eroded: 2,
            greased: false,
        };
        let name = format_item_name(&input);
        assert!(name.contains("very rusty"));
    }

    #[test]
    fn test_potion_appearance() {
        assert_eq!(find_potion_by_appearance("ruby"), Some(0));
        assert_eq!(find_potion_by_appearance("nonexistent"), None);
    }

    #[test]
    fn test_parse_basic() {
        let p = parse_item_name("long sword");
        assert_eq!(p.name, "long sword");
        assert_eq!(p.buc, BucStatus::Unknown);
    }

    #[test]
    fn test_parse_blessed_enchanted() {
        let p = parse_item_name("blessed +3 silver saber");
        assert_eq!(p.buc, BucStatus::Blessed);
        assert_eq!(p.enchantment, Some(3));
        assert_eq!(p.name, "silver saber");
    }

    #[test]
    fn test_parse_cursed() {
        let p = parse_item_name("cursed -1 helm of brilliance");
        assert_eq!(p.buc, BucStatus::Cursed);
        assert_eq!(p.enchantment, Some(-1));
    }

    #[test]
    fn test_parse_quantity() {
        let p = parse_item_name("5 daggers");
        assert_eq!(p.quantity, 5);
        assert_eq!(p.name, "daggers");
    }
}
