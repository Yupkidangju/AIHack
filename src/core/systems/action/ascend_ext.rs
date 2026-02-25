// ============================================================================
// [v2.40.0 R28-2] 승천 시퀀스 (ascend_ext.rs)
// 원본: NetHack 3.6.7 end.c 승천 확장
// 승천 조건, 아뮬렛, 엘리멘탈 플레인, 최종 판정
// ============================================================================

/// [v2.40.0 R28-2] 승천 필수 아이템
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AscensionArtifact {
    AmuletOfYendor,
    BookOfTheDead,
    BellOfOpening,
    CandelabrumOfInvocation,
}

/// [v2.40.0 R28-2] 승천 체크리스트
pub fn ascension_ready(
    has_amulet: bool,
    has_book: bool,
    has_bell: bool,
    has_candelabrum: bool,
    on_astral_plane: bool,
    offered_at_altar: bool,
    correct_alignment: bool,
) -> Result<(), String> {
    if !has_amulet {
        return Err("아뮬렛 of Yendor가 없다.".into());
    }
    if !on_astral_plane {
        return Err("아스트랄 플레인이 아니다.".into());
    }
    if !offered_at_altar {
        return Err("제단에 봉헌하지 않았다.".into());
    }
    if !correct_alignment {
        return Err("잘못된 정렬의 제단이다.".into());
    }
    Ok(())
}

/// [v2.40.0 R28-2] 인보케이션 (원본: perform_invocation)
pub fn can_invoke(
    has_book: bool,
    has_bell: bool,
    has_candelabrum: bool,
    candles_lit: bool,
) -> bool {
    has_book && has_bell && has_candelabrum && candles_lit
}

/// [v2.40.0 R28-2] 엘리멘탈 플레인 순서
pub fn elemental_plane_order() -> Vec<&'static str> {
    vec![
        "Plane of Earth",
        "Plane of Air",
        "Plane of Fire",
        "Plane of Water",
        "Astral Plane",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ready() {
        assert!(ascension_ready(true, true, true, true, true, true, true).is_ok());
    }

    #[test]
    fn test_no_amulet() {
        assert!(ascension_ready(false, true, true, true, true, true, true).is_err());
    }

    #[test]
    fn test_wrong_altar() {
        assert!(ascension_ready(true, true, true, true, true, true, false).is_err());
    }

    #[test]
    fn test_invoke() {
        assert!(can_invoke(true, true, true, true));
        assert!(!can_invoke(true, true, true, false));
    }

    #[test]
    fn test_plane_order() {
        let planes = elemental_plane_order();
        assert_eq!(planes.len(), 5);
        assert_eq!(planes[4], "Astral Plane");
    }
}
