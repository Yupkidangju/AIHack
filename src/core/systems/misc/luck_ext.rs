// ============================================================================
// [v2.41.0 R29-5] 운/카르마 (luck_ext.rs)
// 원본: NetHack 3.6.7 hack.c luck 확장
// 운 값 변동, 럭스톤, 턴 감쇠, 이벤트 영향
// ============================================================================

/// [v2.41.0 R29-5] 운 변동 원인
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LuckEvent {
    SacrificeUnicorn(i32), // 같은 정렬 +, 다른 정렬 -
    BreakMirror,           // -2
    CrossBlackCat,         // -1
    ThrowGem,              // +1~3
    KillPeaceful,          // -1
    OfferAtAltar,          // +1~5
    Pray,                  // +1
}

pub fn luck_delta(event: &LuckEvent) -> i32 {
    match event {
        LuckEvent::SacrificeUnicorn(align_match) => *align_match,
        LuckEvent::BreakMirror => -2,
        LuckEvent::CrossBlackCat => -1,
        LuckEvent::ThrowGem => 2,
        LuckEvent::KillPeaceful => -1,
        LuckEvent::OfferAtAltar => 3,
        LuckEvent::Pray => 1,
    }
}

/// [v2.41.0 R29-5] 운 범위
pub fn clamp_luck(luck: i32, has_luckstone: bool) -> i32 {
    let range = if has_luckstone { 13 } else { 10 };
    luck.clamp(-range, range)
}

/// [v2.41.0 R29-5] 턴 감쇠 (럭스톤 없으면 0방향으로)
pub fn luck_decay(luck: i32, has_luckstone: bool) -> i32 {
    if has_luckstone {
        return luck;
    } // 럭스톤은 감쇠 방지
    if luck > 0 {
        luck - 1
    } else if luck < 0 {
        luck + 1
    } else {
        0
    }
}

/// [v2.41.0 R29-5] 운 기반 성공 보정
pub fn luck_bonus(luck: i32) -> i32 {
    luck.clamp(-5, 5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta() {
        assert_eq!(luck_delta(&LuckEvent::BreakMirror), -2);
        assert_eq!(luck_delta(&LuckEvent::Pray), 1);
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp_luck(15, true), 13);
        assert_eq!(clamp_luck(15, false), 10);
    }

    #[test]
    fn test_decay_no_stone() {
        assert_eq!(luck_decay(5, false), 4);
        assert_eq!(luck_decay(-3, false), -2);
    }

    #[test]
    fn test_decay_with_stone() {
        assert_eq!(luck_decay(5, true), 5);
    }

    #[test]
    fn test_bonus() {
        assert_eq!(luck_bonus(10), 5);
        assert_eq!(luck_bonus(-10), -5);
    }
}
