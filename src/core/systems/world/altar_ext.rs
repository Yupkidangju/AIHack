// ============================================================================
// [v2.30.0 R18-3] 제단/봉헌 (altar_ext.rs)
// 원본: NetHack 3.6.7 pray.c altar, invent.c BUC
// 제단 정렬 판별, 봉헌 효과, BUC 감정
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.30.0 R18-3] 제단 정렬
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltarAlignment {
    Lawful,
    Neutral,
    Chaotic,
    Unaligned,
}

/// [v2.30.0 R18-3] 봉헌 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OfferingResult {
    /// 신이 기뻐함 (축복)
    Accepted,
    /// 매우 기뻐함 (아티팩트/소원)
    GreatlyAccepted,
    /// 무시됨
    Ignored,
    /// 분노 (잘못된 제단)
    Angered,
    /// 시체 봉헌 (영양/효과)
    CorpseAccepted(String),
}

/// [v2.30.0 R18-3] 봉헌 판정
pub fn make_offering(
    altar_align: AltarAlignment,
    player_align: AltarAlignment,
    item_value: i32,
    is_corpse: bool,
    rng: &mut NetHackRng,
) -> OfferingResult {
    // 정렬 불일치 → 분노
    if altar_align != AltarAlignment::Unaligned && altar_align != player_align {
        return OfferingResult::Angered;
    }
    // 시체 봉헌
    if is_corpse {
        return OfferingResult::CorpseAccepted("신이 시체를 받아들였다.".to_string());
    }
    // 가치 기반
    if item_value >= 400 && rng.rn2(10) == 0 {
        return OfferingResult::GreatlyAccepted;
    }
    if item_value >= 50 {
        return OfferingResult::Accepted;
    }
    OfferingResult::Ignored
}

/// [v2.30.0 R18-3] BUC 감정 (제단 위 아이템)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucStatus {
    Blessed,
    Uncursed,
    Cursed,
    Unknown,
}

/// [v2.30.0 R18-3] 제단 위 아이템 BUC 감정
pub fn altar_identify_buc(
    item_buc: BucStatus,
    altar_align: AltarAlignment,
    player_align: AltarAlignment,
) -> BucStatus {
    // 같은 정렬 제단에서만 감정 가능
    if altar_align == player_align || altar_align == AltarAlignment::Unaligned {
        item_buc // 실제 BUC 공개
    } else {
        BucStatus::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offering_accepted() {
        let mut rng = NetHackRng::new(42);
        let result = make_offering(
            AltarAlignment::Lawful,
            AltarAlignment::Lawful,
            100,
            false,
            &mut rng,
        );
        assert_eq!(result, OfferingResult::Accepted);
    }

    #[test]
    fn test_offering_angered() {
        let mut rng = NetHackRng::new(42);
        let result = make_offering(
            AltarAlignment::Chaotic,
            AltarAlignment::Lawful,
            100,
            false,
            &mut rng,
        );
        assert_eq!(result, OfferingResult::Angered);
    }

    #[test]
    fn test_offering_corpse() {
        let mut rng = NetHackRng::new(42);
        let result = make_offering(
            AltarAlignment::Neutral,
            AltarAlignment::Neutral,
            0,
            true,
            &mut rng,
        );
        assert!(matches!(result, OfferingResult::CorpseAccepted(_)));
    }

    #[test]
    fn test_buc_identify_same_align() {
        assert_eq!(
            altar_identify_buc(
                BucStatus::Blessed,
                AltarAlignment::Lawful,
                AltarAlignment::Lawful
            ),
            BucStatus::Blessed
        );
    }

    #[test]
    fn test_buc_identify_wrong_align() {
        assert_eq!(
            altar_identify_buc(
                BucStatus::Blessed,
                AltarAlignment::Chaotic,
                AltarAlignment::Lawful
            ),
            BucStatus::Unknown
        );
    }
}
