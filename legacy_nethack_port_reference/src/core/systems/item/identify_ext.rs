// ============================================================================
// [v2.43.0 R31-3] 아이템 식별 (identify_ext.rs)
// 원본: NetHack 3.6.7 objnam.c/pickup.c 식별 확장
// BUC 감지, 가격 식별, 사용 식별, 타이프 이름
// ============================================================================

/// [v2.43.0 R31-3] 식별 수준
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IdentifyLevel {
    Unknown,
    BucKnown,
    TypeKnown,
    FullyIdentified,
}

/// [v2.43.0 R31-3] 식별 방법
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentifyMethod {
    ScrollIdentify,
    AltarDrop,   // BUC만
    PriceCheck,  // 가격 기반 추정
    UseIdentify, // 사용해서 판명
    SpellIdentify,
    Informal, // 비공식 이름 부여
}

pub fn identify_result(method: &IdentifyMethod) -> IdentifyLevel {
    match method {
        IdentifyMethod::ScrollIdentify | IdentifyMethod::SpellIdentify => {
            IdentifyLevel::FullyIdentified
        }
        IdentifyMethod::AltarDrop => IdentifyLevel::BucKnown,
        IdentifyMethod::PriceCheck => IdentifyLevel::TypeKnown,
        IdentifyMethod::UseIdentify => IdentifyLevel::FullyIdentified,
        IdentifyMethod::Informal => IdentifyLevel::TypeKnown,
    }
}

/// [v2.43.0 R31-3] 가격 기반 추정
pub fn price_identify(base_price: i32) -> Vec<&'static str> {
    match base_price {
        0 => vec!["worthless glass"],
        100 => vec!["potion of healing", "scroll of identify"],
        200 => vec!["potion of speed", "scroll of enchant weapon"],
        300 => vec!["potion of gain level", "scroll of genocide"],
        _ => vec!["unknown"],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_id() {
        assert_eq!(
            identify_result(&IdentifyMethod::ScrollIdentify),
            IdentifyLevel::FullyIdentified
        );
    }

    #[test]
    fn test_altar() {
        assert_eq!(
            identify_result(&IdentifyMethod::AltarDrop),
            IdentifyLevel::BucKnown
        );
    }

    #[test]
    fn test_price() {
        let guesses = price_identify(300);
        assert!(guesses.len() >= 2);
    }

    #[test]
    fn test_level_order() {
        assert!(IdentifyLevel::Unknown < IdentifyLevel::FullyIdentified);
    }
}
