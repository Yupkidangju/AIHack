// ============================================================================
// [v2.30.0 Phase 94-5] 봉헌/제단 확장 (altar_phase94_ext.rs)
// 원본: NetHack 3.6.7 src/pray.c + fountain.c altar 관련 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 제단 행동 — altar_action (pray.c altar 로직)
// =============================================================================

/// [v2.30.0 94-5] 신앙 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Lawful,
    Neutral,
    Chaotic,
}

/// [v2.30.0 94-5] 제단 행동 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AltarAction {
    Pray,
    Sacrifice,
    DropItem,
    Convert,
}

/// [v2.30.0 94-5] 제단 행동 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AltarResult {
    /// 신이 기뻐함
    Pleased {
        message: String,
        gift: Option<String>,
    },
    /// 신이 분노함
    Angered { punishment: String },
    /// 아이템 축복
    ItemBlessed { item_name: String },
    /// 아이템 저주
    ItemCursed { item_name: String },
    /// 제단 전환
    AltarConverted { new_alignment: Alignment },
    /// 번개 (잘못된 제단)
    Lightning { damage: i32 },
    /// 몬스터 소환
    SummonMonster { monster: String },
    /// 아무 일 없음
    NoEffect,
}

/// [v2.30.0 94-5] 제단에서 기도
/// 원본: pray.c pray() 제단 분기
pub fn pray_at_altar(
    altar_alignment: Alignment,
    player_alignment: Alignment,
    piety: i32, // 신앙심 수치
    turns_since_last_pray: i32,
    player_hp_pct: i32,
    rng: &mut NetHackRng,
) -> AltarResult {
    // 잘못된 제단에서 기도
    if altar_alignment != player_alignment {
        if rng.rn2(3) == 0 {
            return AltarResult::Lightning {
                damage: rng.rn2(20) + 10,
            };
        }
        return AltarResult::Angered {
            punishment: "이교도의 제단에서 기도하다니!".to_string(),
        };
    }

    // 너무 자주 기도
    if turns_since_last_pray < 300 {
        return AltarResult::Angered {
            punishment: "그대의 기도가 너무 잦다.".to_string(),
        };
    }

    // HP 위급 → 자동 호의
    if player_hp_pct < 10 {
        return AltarResult::Pleased {
            message: "신이 그대의 위기를 감지했다.".to_string(),
            gift: Some("체력 회복".to_string()),
        };
    }

    // 신앙심 기반 결과
    if piety > 200 {
        // 매우 높은 신앙심 → 선물
        let gifts = ["축복된 무기", "축복된 갑옷", "마법 내성", "독 저항 부여"];
        let idx = rng.rn2(gifts.len() as i32) as usize;
        AltarResult::Pleased {
            message: "신이 그대를 축복한다!".to_string(),
            gift: Some(gifts[idx].to_string()),
        }
    } else if piety > 50 {
        AltarResult::Pleased {
            message: "신이 만족해한다.".to_string(),
            gift: None,
        }
    } else if piety < -50 {
        AltarResult::Angered {
            punishment: "신의 분노가 내려온다!".to_string(),
        }
    } else {
        AltarResult::NoEffect
    }
}

// =============================================================================
// [2] 제물 봉헌 — sacrifice (pray.c sacrificing)
// =============================================================================

/// [v2.30.0 94-5] 제물 가치 계산
pub fn sacrifice_value(
    corpse_level: i32,
    is_own_race: bool,
    altar_alignment: Alignment,
    player_alignment: Alignment,
    is_unicorn: bool,
) -> i32 {
    let mut value = corpse_level * 3;

    // 유니콘 제물 (정렬 보너스)
    if is_unicorn {
        if altar_alignment == player_alignment {
            value += 50;
        } else {
            value += 100; // 반대 정렬 유니콘이 더 가치 있음
        }
    }

    // 동족 살해 페널티
    if is_own_race {
        value = -100;
    }

    value
}

/// [v2.30.0 94-5] 제물 봉헌
pub fn offer_sacrifice(
    sacrifice_val: i32,
    altar_alignment: Alignment,
    player_alignment: Alignment,
    rng: &mut NetHackRng,
) -> AltarResult {
    if sacrifice_val < 0 {
        return AltarResult::Angered {
            punishment: "동족 살해의 죄!".to_string(),
        };
    }

    if sacrifice_val > 80 && rng.rn2(3) == 0 {
        return AltarResult::Pleased {
            message: "신이 크게 기뻐한다!".to_string(),
            gift: Some("축복된 아이템".to_string()),
        };
    }

    if sacrifice_val > 30 {
        AltarResult::Pleased {
            message: "신이 제물을 받아들였다.".to_string(),
            gift: None,
        }
    } else {
        AltarResult::NoEffect
    }
}

// =============================================================================
// [3] 아이템 드랍 — drop_on_altar
// =============================================================================

/// [v2.30.0 94-5] 제단 위 아이템 드랍 효과
pub fn drop_on_altar(
    item_name: &str,
    is_buc_unknown: bool,
    item_buc: i32, // -1=저주, 0=무축, 1=축복
    altar_alignment: Alignment,
    player_alignment: Alignment,
    rng: &mut NetHackRng,
) -> AltarResult {
    // BUC 식별 (제단 위에 올려놓으면 반짝임)
    if is_buc_unknown {
        match item_buc {
            1 => {
                return AltarResult::ItemBlessed {
                    item_name: item_name.to_string(),
                }
            }
            -1 => {
                return AltarResult::ItemCursed {
                    item_name: item_name.to_string(),
                }
            }
            _ => return AltarResult::NoEffect,
        }
    }

    AltarResult::NoEffect
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
    fn test_pray_correct_altar() {
        let mut found_pleased = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result =
                pray_at_altar(Alignment::Lawful, Alignment::Lawful, 100, 500, 80, &mut rng);
            if matches!(result, AltarResult::Pleased { .. }) {
                found_pleased = true;
                break;
            }
        }
        assert!(found_pleased);
    }

    #[test]
    fn test_pray_wrong_altar() {
        let mut anger = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = pray_at_altar(
                Alignment::Chaotic,
                Alignment::Lawful,
                100,
                500,
                80,
                &mut rng,
            );
            if matches!(
                result,
                AltarResult::Angered { .. } | AltarResult::Lightning { .. }
            ) {
                anger = true;
                break;
            }
        }
        assert!(anger);
    }

    #[test]
    fn test_pray_too_often() {
        let mut rng = test_rng();
        let result = pray_at_altar(Alignment::Lawful, Alignment::Lawful, 100, 100, 80, &mut rng);
        assert!(matches!(result, AltarResult::Angered { .. }));
    }

    #[test]
    fn test_sacrifice_value_unicorn() {
        let val = sacrifice_value(10, false, Alignment::Lawful, Alignment::Chaotic, true);
        assert!(val > 100);
    }

    #[test]
    fn test_sacrifice_own_race() {
        let val = sacrifice_value(10, true, Alignment::Lawful, Alignment::Lawful, false);
        assert!(val < 0);
    }

    #[test]
    fn test_offer_sacrifice_high() {
        let mut pleased = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = offer_sacrifice(100, Alignment::Lawful, Alignment::Lawful, &mut rng);
            if matches!(result, AltarResult::Pleased { .. }) {
                pleased = true;
                break;
            }
        }
        assert!(pleased);
    }

    #[test]
    fn test_drop_buc_identify() {
        let mut rng = test_rng();
        let result = drop_on_altar(
            "검",
            true,
            1,
            Alignment::Lawful,
            Alignment::Lawful,
            &mut rng,
        );
        assert!(matches!(result, AltarResult::ItemBlessed { .. }));
    }

    #[test]
    fn test_drop_cursed_identify() {
        let mut rng = test_rng();
        let result = drop_on_altar(
            "갑옷",
            true,
            -1,
            Alignment::Lawful,
            Alignment::Lawful,
            &mut rng,
        );
        assert!(matches!(result, AltarResult::ItemCursed { .. }));
    }
}
