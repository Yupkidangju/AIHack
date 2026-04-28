// ============================================================================
// [v2.32.0 Phase 96-3] 아이템 식별 확장 (objnam_phase96_ext.rs)
// 원본: NetHack 3.6.7 src/objnam.c L200-1500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 아이템 식별 — identification (objnam.c L200-600)
// =============================================================================

/// [v2.32.0 96-3] 식별 수준
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifyLevel {
    Unknown,         // 미식별
    Appearance,      // 외관만
    Blessed,         // BUC 식별
    Named,           // 이름만
    FullyIdentified, // 완전 식별
}

/// [v2.32.0 96-3] 아이템 표시 이름
#[derive(Debug, Clone)]
pub struct ItemDisplay {
    pub raw_name: String,
    pub display_name: String,
    pub id_level: IdentifyLevel,
    pub buc_string: Option<String>,
    pub enchant_string: Option<String>,
    pub erosion_string: Option<String>,
    pub custom_name: Option<String>,
}

/// [v2.32.0 96-3] 미식별 외관 테이블
pub fn get_unidentified_appearance(item_type: &str, appearance_idx: i32) -> String {
    match item_type {
        "포션" => {
            let colors = [
                "빨간",
                "파란",
                "초록",
                "노란",
                "흰",
                "검은",
                "보라",
                "주황",
                "핑크",
                "갈색",
                "투명한",
                "탁한",
                "반짝이는",
            ];
            let idx = (appearance_idx as usize) % colors.len();
            format!("{} 포션", colors[idx])
        }
        "스크롤" => {
            let labels = [
                "ZELGO MER",
                "JUYED AWK YACC",
                "NR 9",
                "XIXAXA XOXAXA XUXAXA",
                "PRATYAVAYAH",
                "DAIYEN FANSEN",
                "GARVEN DEH",
                "READ ME",
                "ANDOVA BEGARIN",
                "KIRJE",
                "VE FORBRULL",
                "TEMOV",
            ];
            let idx = (appearance_idx as usize) % labels.len();
            format!("\"{}\"라고 적힌 스크롤", labels[idx])
        }
        "지팡이" => {
            let materials = [
                "호두나무",
                "단풍나무",
                "참나무",
                "대나무",
                "은",
                "구리",
                "수정",
                "강철",
                "뼈",
            ];
            let idx = (appearance_idx as usize) % materials.len();
            format!("{} 지팡이", materials[idx])
        }
        "반지" => {
            let gems = [
                "루비",
                "다이아몬드",
                "사파이어",
                "에메랄드",
                "오닉스",
                "자수정",
                "토파즈",
                "옥",
                "호박",
                "진주",
            ];
            let idx = (appearance_idx as usize) % gems.len();
            format!("{} 반지", gems[idx])
        }
        "부적" => {
            let shapes = [
                "삼각형",
                "원형",
                "사각형",
                "눈물방울",
                "해골",
                "뱀 모양",
                "눈 모양",
                "별 모양",
            ];
            let idx = (appearance_idx as usize) % shapes.len();
            format!("{} 부적", shapes[idx])
        }
        _ => format!("{} (미식별)", item_type),
    }
}

/// [v2.32.0 96-3] 아이템 표시 이름 생성
/// 원본: objnam.c doname() + xname()
pub fn format_item_name(
    true_name: &str,
    item_type: &str,
    id_level: IdentifyLevel,
    appearance_idx: i32,
    enchantment: Option<i32>,
    buc: Option<i32>, // -1=저주, 0=무축, 1=축복
    erosion: i32,
    custom_name: Option<&str>,
    quantity: i32,
) -> ItemDisplay {
    let mut display = String::new();
    let mut buc_str = None;
    let mut ench_str = None;
    let mut eros_str = None;

    // 수량
    if quantity > 1 {
        display.push_str(&format!("{}개의 ", quantity));
    }

    // BUC 표시
    if id_level >= IdentifyLevel::Blessed {
        if let Some(b) = buc {
            let s = match b {
                1 => "축복된",
                -1 => "저주된",
                _ => "무축의",
            };
            display.push_str(s);
            display.push(' ');
            buc_str = Some(s.to_string());
        }
    }

    // 부식 표시
    if erosion > 0 {
        let e = match erosion {
            1 => "녹슨",
            2 => "매우 녹슨",
            3 => "완전히 녹슨",
            _ => "손상된",
        };
        display.push_str(e);
        display.push(' ');
        eros_str = Some(e.to_string());
    }

    // 인챈트 표시
    if id_level >= IdentifyLevel::FullyIdentified {
        if let Some(e) = enchantment {
            let s = if e >= 0 {
                format!("+{}", e)
            } else {
                format!("{}", e)
            };
            display.push_str(&s);
            display.push(' ');
            ench_str = Some(s);
        }
    }

    // 이름
    match id_level {
        IdentifyLevel::Unknown | IdentifyLevel::Appearance => {
            display.push_str(&get_unidentified_appearance(item_type, appearance_idx));
        }
        _ => {
            display.push_str(true_name);
        }
    }

    // 커스텀 이름
    let cname = custom_name.map(|n| {
        display.push_str(&format!(" ({}이라 불리는)", n));
        n.to_string()
    });

    ItemDisplay {
        raw_name: true_name.to_string(),
        display_name: display,
        id_level,
        buc_string: buc_str,
        enchant_string: ench_str,
        erosion_string: eros_str,
        custom_name: cname,
    }
}

// =============================================================================
// [2] 가격 식별 — price_id (objnam.c + shk.c 연동)
// =============================================================================

/// [v2.32.0 96-3] 가격 기반 식별 힌트
pub fn price_identify_hint(item_type: &str, observed_price: i32) -> Vec<String> {
    match item_type {
        "포션" => match observed_price {
            0..=50 => vec![
                "물".to_string(),
                "과일 주스".to_string(),
                "질병".to_string(),
            ],
            51..=100 => vec!["투명".to_string(), "실명".to_string(), "혼란".to_string()],
            101..=200 => vec!["치유".to_string(), "속도".to_string(), "잠".to_string()],
            201..=300 => vec!["대치유".to_string(), "레벨업".to_string()],
            _ => vec!["완전 치유".to_string(), "소원".to_string()],
        },
        "스크롤" => match observed_price {
            0..=20 => vec!["식별".to_string(), "빛".to_string()],
            21..=60 => vec!["마법 부여".to_string(), "제거".to_string()],
            61..=100 => vec!["순간이동".to_string(), "금 감지".to_string()],
            101..=200 => vec!["충전".to_string(), "대학살".to_string()],
            _ => vec!["소원".to_string()],
        },
        _ => vec!["알 수 없음".to_string()],
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unid_potion() {
        let name = get_unidentified_appearance("포션", 0);
        assert!(name.contains("포션"));
    }

    #[test]
    fn test_unid_scroll() {
        let name = get_unidentified_appearance("스크롤", 3);
        assert!(name.contains("스크롤"));
    }

    #[test]
    fn test_format_unknown() {
        let display = format_item_name(
            "치유의 포션",
            "포션",
            IdentifyLevel::Unknown,
            1,
            Some(0),
            None,
            0,
            None,
            1,
        );
        assert!(!display.display_name.contains("치유"));
        assert!(display.display_name.contains("포션"));
    }

    #[test]
    fn test_format_identified() {
        let display = format_item_name(
            "치유의 포션",
            "포션",
            IdentifyLevel::FullyIdentified,
            1,
            Some(0),
            Some(1),
            0,
            None,
            1,
        );
        assert!(display.display_name.contains("치유"));
        assert!(display.display_name.contains("축복"));
    }

    #[test]
    fn test_format_quantity() {
        let display = format_item_name(
            "화살",
            "무기",
            IdentifyLevel::FullyIdentified,
            0,
            Some(2),
            None,
            0,
            None,
            20,
        );
        assert!(display.display_name.contains("20개"));
    }

    #[test]
    fn test_format_erosion() {
        let display = format_item_name(
            "검",
            "무기",
            IdentifyLevel::FullyIdentified,
            0,
            Some(0),
            None,
            2,
            None,
            1,
        );
        assert!(display.display_name.contains("녹슨"));
    }

    #[test]
    fn test_price_hint_potion() {
        let hints = price_identify_hint("포션", 150);
        assert!(!hints.is_empty());
    }

    #[test]
    fn test_custom_name() {
        let display = format_item_name(
            "검",
            "무기",
            IdentifyLevel::FullyIdentified,
            0,
            Some(3),
            None,
            0,
            Some("듀란달"),
            1,
        );
        assert!(display.display_name.contains("듀란달"));
    }
}
