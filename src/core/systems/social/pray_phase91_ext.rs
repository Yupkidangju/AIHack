// ============================================================================
// [v2.27.0 Phase 91-6] 기도 시스템 확장 (pray_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/pray.c L400-1000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 기도 결과 판정 — prayer_result (pray.c L400-700)
// =============================================================================

/// [v2.27.0 91-6] 기도 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrayerResult {
    /// 축복 — 신의 호의
    Blessed { effects: Vec<String> },
    /// 무시 — 신이 반응하지 않음
    Ignored,
    /// 분노 — 신의 징벌
    Angered { punishment: String },
    /// 죽음 — 잘못된 신에게 기도
    SmitedDown { message: String },
}

/// [v2.27.0 91-6] 기도 입력
#[derive(Debug, Clone)]
pub struct PrayerInput {
    pub alignment: i32,        // -1=혼돈, 0=중립, 1=질서
    pub god_alignment: i32,    // 플레이어의 신 정렬
    pub alignment_record: i32, // 정렬 기록 (높을수록 호의)
    pub prayer_timeout: i32,   // 마지막 기도 후 턴 수
    pub hp_pct: i32,           // 현재 HP %
    pub is_in_hell: bool,
    pub is_on_altar: bool,
    pub luck: i32,
}

/// [v2.27.0 91-6] 기도 결과 판정
/// 원본: pray.c dopray() + prayer_done()
pub fn resolve_prayer(input: &PrayerInput, rng: &mut NetHackRng) -> PrayerResult {
    // 잘못된 신에게 기도 → 징벌
    if input.alignment != input.god_alignment {
        return PrayerResult::Angered {
            punishment: "신이 분노하여 번개를 내렸다!".to_string(),
        };
    }

    // 지옥에서 기도 → 매우 위험
    if input.is_in_hell && rng.rn2(3) != 0 {
        return PrayerResult::Angered {
            punishment: "이 장소에서의 기도는 위험하다...".to_string(),
        };
    }

    // 기도 쿨다운 (300턴)
    if input.prayer_timeout < 300 {
        return PrayerResult::Ignored;
    }

    // 정렬 기록 기반 판정
    let favor = input.alignment_record + input.luck * 3;

    if favor < -10 {
        // 매우 낮은 신앙 → 징벌
        return PrayerResult::Angered {
            punishment: "신의 인내가 한계에 달했다.".to_string(),
        };
    }

    if favor < 0 {
        // 낮은 신앙 → 무시
        return PrayerResult::Ignored;
    }

    // 긍정적 결과 — 효과 생성
    let mut effects = Vec::new();

    // HP 위험 시 회복
    if input.hp_pct < 25 {
        effects.push("HP 완전 회복".to_string());
    }

    // 제단 위에서 기도 → 추가 축복
    if input.is_on_altar {
        effects.push("장비 축복".to_string());
    }

    // 높은 호의 → 추가 선물
    if favor > 20 {
        let gift = rng.rn2(4);
        match gift {
            0 => effects.push("능력치 증가".to_string()),
            1 => effects.push("마법 저항 부여".to_string()),
            2 => effects.push("저주 해제".to_string()),
            _ => effects.push("행운 증가".to_string()),
        }
    }

    if effects.is_empty() {
        effects.push("평화로운 느낌이 든다.".to_string());
    }

    PrayerResult::Blessed { effects }
}

// =============================================================================
// [2] 신의 선물 — divine_gift (pray.c L700-900)
// =============================================================================

/// [v2.27.0 91-6] 신의 선물 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DivineGift {
    /// 아티팩트 무기
    ArtifactWeapon { name: String },
    /// 능력치 증가
    StatBoost { stat: String, amount: i32 },
    /// 완전 회복
    FullHeal,
    /// 저주 해제
    Uncurse { count: i32 },
    /// 축복 부여
    Bless { count: i32 },
    /// 없음
    None,
}

/// [v2.27.0 91-6] 아티팩트 선물 가능 여부
pub fn divine_gift_check(
    alignment_record: i32,
    gifts_received: i32,
    player_level: i32,
    rng: &mut NetHackRng,
) -> DivineGift {
    // 선물 기준: alignment_record > 10 + gifts * 10
    let threshold = 10 + gifts_received * 10;
    if alignment_record < threshold {
        return DivineGift::None;
    }

    // 레벨 기반 확률
    if rng.rn2(player_level + 5) < 3 {
        return DivineGift::None;
    }

    let gift_type = rng.rn2(5);
    match gift_type {
        0 => DivineGift::ArtifactWeapon {
            name: "신성한 무기".to_string(),
        },
        1 => DivineGift::StatBoost {
            stat: "WIS".to_string(),
            amount: 1,
        },
        2 => DivineGift::FullHeal,
        3 => DivineGift::Uncurse {
            count: rng.rn2(3) + 1,
        },
        _ => DivineGift::Bless {
            count: rng.rn2(3) + 1,
        },
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_prayer_wrong_alignment() {
        let mut rng = test_rng();
        let input = PrayerInput {
            alignment: 1,
            god_alignment: -1,
            alignment_record: 50,
            prayer_timeout: 500,
            hp_pct: 50,
            is_in_hell: false,
            is_on_altar: false,
            luck: 5,
        };
        assert!(matches!(
            resolve_prayer(&input, &mut rng),
            PrayerResult::Angered { .. }
        ));
    }

    #[test]
    fn test_prayer_cooldown() {
        let mut rng = test_rng();
        let input = PrayerInput {
            alignment: 1,
            god_alignment: 1,
            alignment_record: 50,
            prayer_timeout: 100,
            hp_pct: 50,
            is_in_hell: false,
            is_on_altar: false,
            luck: 5,
        };
        assert_eq!(resolve_prayer(&input, &mut rng), PrayerResult::Ignored);
    }

    #[test]
    fn test_prayer_blessed() {
        let mut rng = test_rng();
        let input = PrayerInput {
            alignment: 1,
            god_alignment: 1,
            alignment_record: 50,
            prayer_timeout: 500,
            hp_pct: 50,
            is_in_hell: false,
            is_on_altar: true,
            luck: 5,
        };
        let result = resolve_prayer(&input, &mut rng);
        assert!(matches!(result, PrayerResult::Blessed { .. }));
    }

    #[test]
    fn test_prayer_low_hp_heal() {
        let mut rng = test_rng();
        let input = PrayerInput {
            alignment: 1,
            god_alignment: 1,
            alignment_record: 10,
            prayer_timeout: 500,
            hp_pct: 10,
            is_in_hell: false,
            is_on_altar: false,
            luck: 0,
        };
        if let PrayerResult::Blessed { effects } = resolve_prayer(&input, &mut rng) {
            assert!(effects.iter().any(|e| e.contains("HP")));
        }
    }

    #[test]
    fn test_divine_gift_low_record() {
        let mut rng = test_rng();
        assert!(matches!(
            divine_gift_check(5, 0, 10, &mut rng),
            DivineGift::None
        ));
    }

    #[test]
    fn test_divine_gift_high_record() {
        let mut got_gift = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let gift = divine_gift_check(50, 0, 20, &mut rng);
            if !matches!(gift, DivineGift::None) {
                got_gift = true;
                break;
            }
        }
        assert!(got_gift, "높은 신앙이면 선물 받아야 함");
    }
}
