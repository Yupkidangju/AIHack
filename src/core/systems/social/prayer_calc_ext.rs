// ============================================================================
// [v2.39.0 R27-4] 기도 시스템 (prayer_calc_ext.rs)
// 원본: NetHack 3.6.7 pray.c 확장
// 신앙도, 기도 성공률, 신의 응답, 쿨다운
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.39.0 R27-4] 신의 응답
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrayerResponse {
    Blessing, // 축복 + 장비 BUC 개선
    FullHeal,
    CureSickness,
    FixAttributes,
    GrantProtection,
    Ignored,    // 무시됨
    Angry,      // 분노 (역효과)
    Smite(i32), // 천벌 데미지
}

/// [v2.39.0 R27-4] 기도 판정
pub fn pray(
    piety: i32,     // 0~100
    alignment: i32, // -128~127
    turns_since_pray: u64,
    hp_ratio: f64, // 현재HP/최대HP
    rng: &mut NetHackRng,
) -> PrayerResponse {
    // 쿨다운 미충족
    if turns_since_pray < 300 {
        return PrayerResponse::Angry;
    }

    // 정렬 음수 → 천벌
    if alignment < -10 {
        return PrayerResponse::Smite(rng.rn1(10, 5));
    }

    // 위급 상황 (HP 10% 이하) + 신앙도 충분
    if hp_ratio <= 0.1 && piety >= 30 {
        return PrayerResponse::FullHeal;
    }

    let roll = rng.rn2(100);
    if roll < piety {
        if roll < 20 {
            PrayerResponse::Blessing
        } else if roll < 40 {
            PrayerResponse::GrantProtection
        } else if roll < 60 {
            PrayerResponse::FixAttributes
        } else {
            PrayerResponse::CureSickness
        }
    } else {
        PrayerResponse::Ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cooldown() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(pray(80, 50, 100, 1.0, &mut rng), PrayerResponse::Angry);
    }

    #[test]
    fn test_negative_align() {
        let mut rng = NetHackRng::new(42);
        let r = pray(80, -20, 500, 1.0, &mut rng);
        assert!(matches!(r, PrayerResponse::Smite(_)));
    }

    #[test]
    fn test_emergency() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(pray(50, 50, 500, 0.05, &mut rng), PrayerResponse::FullHeal);
    }

    #[test]
    fn test_high_piety() {
        let mut success = 0;
        for s in 0..100 {
            let mut rng = NetHackRng::new(s);
            if pray(90, 50, 500, 0.5, &mut rng) != PrayerResponse::Ignored {
                success += 1;
            }
        }
        assert!(success > 70);
    }

    #[test]
    fn test_low_piety() {
        let mut ignored = 0;
        for s in 0..50 {
            let mut rng = NetHackRng::new(s);
            if pray(10, 50, 500, 0.5, &mut rng) == PrayerResponse::Ignored {
                ignored += 1;
            }
        }
        assert!(ignored > 30);
    }
}
