// ============================================================================
// [v2.37.0 Phase 101-2] 속성/저주/축복 통합 (buc_phase101_ext.rs)
// 원본: NetHack 3.6.7 전반 BUC(Blessed/Uncursed/Cursed) 시스템 통합
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] BUC 시스템 — buc_system
// =============================================================================

/// [v2.37.0 101-2] BUC 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BucStatus {
    Blessed,
    Uncursed,
    Cursed,
    Unknown,
}

/// [v2.37.0 101-2] BUC 효과 수치
#[derive(Debug, Clone)]
pub struct BucEffect {
    pub status: BucStatus,
    pub damage_modifier: i32,
    pub enchantment_modifier: i32,
    pub price_modifier: f64,
    pub can_unequip: bool,
    pub extra_effect: Option<String>,
}

/// [v2.37.0 101-2] BUC별 효과 계산
pub fn calculate_buc_effect(status: BucStatus, item_type: &str) -> BucEffect {
    match status {
        BucStatus::Blessed => BucEffect {
            status,
            damage_modifier: 2,
            enchantment_modifier: 1,
            price_modifier: 1.5,
            can_unequip: true,
            extra_effect: match item_type {
                "포션" => Some("효과 강화".to_string()),
                "스크롤" => Some("효과 증폭".to_string()),
                "갑옷" => Some("내구도 보너스".to_string()),
                "음식" => Some("영양 증가".to_string()),
                _ => None,
            },
        },
        BucStatus::Uncursed => BucEffect {
            status,
            damage_modifier: 0,
            enchantment_modifier: 0,
            price_modifier: 1.0,
            can_unequip: true,
            extra_effect: None,
        },
        BucStatus::Cursed => BucEffect {
            status,
            damage_modifier: -2,
            enchantment_modifier: -1,
            price_modifier: 0.5,
            can_unequip: false,
            extra_effect: match item_type {
                "포션" => Some("부작용 발생".to_string()),
                "스크롤" => Some("효과 역전".to_string()),
                "갑옷" => Some("벗을 수 없음".to_string()),
                "음식" => Some("식중독 위험".to_string()),
                _ => Some("저주받은 상태".to_string()),
            },
        },
        BucStatus::Unknown => BucEffect {
            status,
            damage_modifier: 0,
            enchantment_modifier: 0,
            price_modifier: 0.8,
            can_unequip: true,
            extra_effect: None,
        },
    }
}

/// [v2.37.0 101-2] 축복/저주 전환
pub fn change_buc_status(
    current: BucStatus,
    action: &str,
    rng: &mut NetHackRng,
) -> (BucStatus, String) {
    match action {
        "축복" => match current {
            BucStatus::Cursed => (BucStatus::Uncursed, "저주가 풀렸다!".to_string()),
            BucStatus::Uncursed => (BucStatus::Blessed, "축복을 받았다!".to_string()),
            BucStatus::Blessed => (BucStatus::Blessed, "이미 축복받은 상태이다.".to_string()),
            BucStatus::Unknown => (BucStatus::Blessed, "축복을 받았다!".to_string()),
        },
        "저주" => match current {
            BucStatus::Blessed => (BucStatus::Uncursed, "축복이 사라졌다.".to_string()),
            BucStatus::Uncursed => (BucStatus::Cursed, "저주를 받았다!".to_string()),
            BucStatus::Cursed => (BucStatus::Cursed, "이미 저주받은 상태이다.".to_string()),
            BucStatus::Unknown => (BucStatus::Cursed, "저주를 받았다!".to_string()),
        },
        "감정" => {
            let msg = match current {
                BucStatus::Blessed => "이 아이템은 축복받은 상태이다.",
                BucStatus::Uncursed => "이 아이템은 저주받지 않은 상태이다.",
                BucStatus::Cursed => "이 아이템은 저주받은 상태이다!",
                BucStatus::Unknown => "알 수 없는 상태이다.",
            };
            (current, msg.to_string())
        }
        "제단 감정" => {
            // 제단에 놓으면 확률적으로 감정
            if rng.rn2(3) == 0 {
                (
                    current,
                    format!("제단이 빛나며 아이템의 상태가 드러난다: {:?}", current),
                )
            } else {
                (current, "아무 일도 일어나지 않았다.".to_string())
            }
        }
        _ => (current, "알 수 없는 행동이다.".to_string()),
    }
}

/// [v2.37.0 101-2] 강화 시도
pub fn try_enchant(current_enchant: i32, buc: BucStatus, rng: &mut NetHackRng) -> (i32, String) {
    let bonus = match buc {
        BucStatus::Blessed => rng.rn2(3) + 1,   // +1~+3
        BucStatus::Uncursed => rng.rn2(2) + 1,  // +1~+2
        BucStatus::Cursed => -(rng.rn2(2) + 1), // -1~-2
        BucStatus::Unknown => rng.rn2(3) - 1,   // -1~+1
    };

    let new_enchant = current_enchant + bonus;

    // 과강화 폭발 체크 (+6 이상)
    if new_enchant > 5 && rng.rn2(new_enchant - 4) > 0 {
        return (0, "아이템이 폭발했다!".to_string());
    }

    let msg = if bonus > 0 {
        format!("강화 성공! (+{} → +{})", current_enchant, new_enchant)
    } else if bonus < 0 {
        format!("강화 실패... (+{} → +{})", current_enchant, new_enchant)
    } else {
        "아무 변화도 없었다.".to_string()
    };

    (new_enchant, msg)
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
    fn test_blessed_potion() {
        let eff = calculate_buc_effect(BucStatus::Blessed, "포션");
        assert_eq!(eff.damage_modifier, 2);
        assert!(eff.extra_effect.is_some());
    }

    #[test]
    fn test_cursed_armor() {
        let eff = calculate_buc_effect(BucStatus::Cursed, "갑옷");
        assert!(!eff.can_unequip);
    }

    #[test]
    fn test_bless_cursed() {
        let mut rng = test_rng();
        let (new, msg) = change_buc_status(BucStatus::Cursed, "축복", &mut rng);
        assert_eq!(new, BucStatus::Uncursed);
        assert!(msg.contains("풀렸다"));
    }

    #[test]
    fn test_curse_blessed() {
        let mut rng = test_rng();
        let (new, _) = change_buc_status(BucStatus::Blessed, "저주", &mut rng);
        assert_eq!(new, BucStatus::Uncursed);
    }

    #[test]
    fn test_identify() {
        let mut rng = test_rng();
        let (s, msg) = change_buc_status(BucStatus::Blessed, "감정", &mut rng);
        assert_eq!(s, BucStatus::Blessed);
        assert!(msg.contains("축복"));
    }

    #[test]
    fn test_enchant_blessed() {
        let mut rng = test_rng();
        let (new_e, _) = try_enchant(2, BucStatus::Blessed, &mut rng);
        assert!(new_e > 2 || new_e == 0); // 성공 또는 폭발
    }

    #[test]
    fn test_enchant_cursed() {
        let mut rng = test_rng();
        let (new_e, _) = try_enchant(3, BucStatus::Cursed, &mut rng);
        assert!(new_e <= 3);
    }

    #[test]
    fn test_price_modifier() {
        let blessed = calculate_buc_effect(BucStatus::Blessed, "검");
        let cursed = calculate_buc_effect(BucStatus::Cursed, "검");
        assert!(blessed.price_modifier > cursed.price_modifier);
    }
}
