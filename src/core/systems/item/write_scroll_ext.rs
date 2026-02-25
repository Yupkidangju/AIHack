// ============================================================================
// [v2.42.0 R30-4] 두루마리/마커 (write_scroll_ext.rs)
// 원본: NetHack 3.6.7 write.c 확장
// 마법 마커로 스크롤/스펠북 작성
// ============================================================================

/// [v2.42.0 R30-4] 작성 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteResult {
    Success { charges_used: i32 },
    InkInsufficient,
    CursedMarker,  // 저주된 마커 → 가짜 스크롤
    UnknownScroll, // 미감정 스크롤 작성 불가
}

/// [v2.42.0 R30-4] 잉크 비용
pub fn ink_cost(scroll_level: i32) -> i32 {
    match scroll_level {
        1..=2 => 8,
        3..=4 => 16,
        5..=6 => 24,
        _ => 32,
    }
}

/// [v2.42.0 R30-4] 작성 판정
pub fn write_scroll(
    marker_charges: i32,
    scroll_level: i32,
    scroll_known: bool,
    marker_cursed: bool,
) -> WriteResult {
    if !scroll_known {
        return WriteResult::UnknownScroll;
    }
    if marker_cursed {
        return WriteResult::CursedMarker;
    }
    let cost = ink_cost(scroll_level);
    if marker_charges < cost {
        return WriteResult::InkInsufficient;
    }
    WriteResult::Success { charges_used: cost }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_ok() {
        let r = write_scroll(50, 3, true, false);
        assert!(matches!(r, WriteResult::Success { charges_used: 16 }));
    }

    #[test]
    fn test_no_ink() {
        assert_eq!(
            write_scroll(5, 5, true, false),
            WriteResult::InkInsufficient
        );
    }

    #[test]
    fn test_cursed() {
        assert_eq!(write_scroll(50, 1, true, true), WriteResult::CursedMarker);
    }

    #[test]
    fn test_unknown() {
        assert_eq!(
            write_scroll(50, 1, false, false),
            WriteResult::UnknownScroll
        );
    }
}
