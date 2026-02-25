// ============================================================================
// [v2.41.0 R29-3] 저주 시스템 (curse_system_ext.rs)
// 원본: NetHack 3.6.7 mkobj.c/pray.c 저주 확장
// 저주 판정, 장비 고착, 해제 조건
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.41.0 R29-3] 저주 원인
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurseCause {
    Fountain,
    Trap,
    Monster(String),
    SelfCurse,
    Altar,
    PrayerPunishment,
}

/// [v2.41.0 R29-3] 저주 판정 (아이템 획득 시)
pub fn curse_chance(luck: i32, depth: i32, rng: &mut NetHackRng) -> bool {
    let chance = (20 + depth - luck * 3).clamp(5, 50);
    rng.rn2(100) < chance
}

/// [v2.41.0 R29-3] 장비 고착 (저주된 장비 해제 불가)
pub fn is_stuck(is_cursed: bool, is_welded: bool) -> bool {
    is_cursed || is_welded
}

/// [v2.41.0 R29-3] 저주 해제 방법
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UncurseMethods {
    ScrollRemoveCurse,
    HolyWater,
    Prayer,
    ConfuseRemoveCurse, // 혼란 스크롤 → 축복으로 변환
}

pub fn uncurse_success(method: &UncurseMethods, blessed_scroll: bool) -> bool {
    match method {
        UncurseMethods::ScrollRemoveCurse => true,
        UncurseMethods::HolyWater => true,
        UncurseMethods::Prayer => blessed_scroll, // 기도는 신앙도 의존
        UncurseMethods::ConfuseRemoveCurse => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curse_chance() {
        let mut cursed = 0;
        for s in 0..200 {
            let mut rng = NetHackRng::new(s);
            if curse_chance(0, 20, &mut rng) {
                cursed += 1;
            }
        }
        assert!(cursed > 30 && cursed < 120);
    }

    #[test]
    fn test_stuck() {
        assert!(is_stuck(true, false));
        assert!(is_stuck(false, true));
        assert!(!is_stuck(false, false));
    }

    #[test]
    fn test_uncurse_scroll() {
        assert!(uncurse_success(&UncurseMethods::ScrollRemoveCurse, false));
    }

    #[test]
    fn test_uncurse_holy() {
        assert!(uncurse_success(&UncurseMethods::HolyWater, false));
    }

    #[test]
    fn test_lucky_less_curse() {
        let mut c_lucky = 0;
        let mut c_unlucky = 0;
        for s in 0..200 {
            let mut rng1 = NetHackRng::new(s);
            let mut rng2 = NetHackRng::new(s);
            if curse_chance(10, 10, &mut rng1) {
                c_lucky += 1;
            }
            if curse_chance(-10, 10, &mut rng2) {
                c_unlucky += 1;
            }
        }
        assert!(c_lucky < c_unlucky);
    }
}
