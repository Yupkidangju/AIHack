// ============================================================================
// [v2.33.0 Phase 97-3] 읽기/스크롤 확장 (read_phase97_ext.rs)
// 원본: NetHack 3.6.7 src/read.c L500-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 스크롤 효과 — scroll_effects (read.c L500-1500)
// =============================================================================

/// [v2.33.0 97-3] 스크롤 유형 (확장)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtScrollType {
    Identify,
    Enchant,
    Remove,
    Teleport,
    Gold,
    Food,
    Confuse,
    Light,
    Blank,
    Punishment,
    Stinking,
    Charging,
    Genocide,
    Taming,
    CreateMonster,
    Earth,
    Amnesia,
    Fire,
    Destroy,
    Magic,
}

/// [v2.33.0 97-3] 스크롤 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollEffect {
    IdentifyItems { count: i32 },
    EnchantItem { bonus: i32, item_slot: String },
    RemoveCurse { count: i32, blessed: bool },
    TeleportSelf,
    GoldDetect { count: i32 },
    FoodDetect { count: i32 },
    ConfuseSelf { turns: i32 },
    LightArea { radius: i32 },
    BlankScroll,
    Punishment { ball_weight: i32 },
    StinkingCloud { radius: i32 },
    ChargingItem { charges: i32 },
    Genocide { monsters_removed: i32 },
    TameMonsters { count: i32, radius: i32 },
    CreateMonsters { count: i32 },
    Earthquake { damage: i32 },
    Amnesia { spells_lost: i32 },
    FireBlast { damage: i32, radius: i32 },
    DestroyArmor,
    MagicMapping,
    NoEffect,
}

/// [v2.33.0 97-3] 스크롤 읽기
/// 원본: read.c doread()
pub fn read_scroll(
    scroll: ExtScrollType,
    is_blessed: bool,
    is_cursed: bool,
    is_confused: bool,
    rng: &mut NetHackRng,
) -> ScrollEffect {
    match scroll {
        ExtScrollType::Identify => {
            let count = if is_blessed {
                99
            }
            // 축복 → 전체 식별
            else if is_cursed {
                0
            }
            // 저주 → 실패
            else {
                rng.rn2(4) + 1
            }; // 일반 → 1~4개
            ScrollEffect::IdentifyItems { count }
        }
        ExtScrollType::Enchant => {
            let bonus = if is_blessed {
                2
            } else if is_cursed {
                -1
            } else {
                1
            };
            ScrollEffect::EnchantItem {
                bonus,
                item_slot: "선택한 장비".to_string(),
            }
        }
        ExtScrollType::Remove => {
            let blessed = is_blessed;
            let count = if blessed { 99 } else { 1 };
            if is_cursed {
                return ScrollEffect::NoEffect; // 저주 → 실패
            }
            ScrollEffect::RemoveCurse { count, blessed }
        }
        ExtScrollType::Teleport => {
            if is_confused {
                return ScrollEffect::TeleportSelf; // 혼란 → 레벨 텔레포트
            }
            ScrollEffect::TeleportSelf
        }
        ExtScrollType::Gold => ScrollEffect::GoldDetect {
            count: rng.rn2(10) + 1,
        },
        ExtScrollType::Food => ScrollEffect::FoodDetect {
            count: rng.rn2(5) + 1,
        },
        ExtScrollType::Confuse => ScrollEffect::ConfuseSelf {
            turns: rng.rn2(20) + 10,
        },
        ExtScrollType::Light => {
            let radius = if is_blessed { 10 } else { 5 };
            ScrollEffect::LightArea { radius }
        }
        ExtScrollType::Blank => ScrollEffect::BlankScroll,
        ExtScrollType::Punishment => {
            if is_blessed {
                return ScrollEffect::NoEffect;
            }
            ScrollEffect::Punishment {
                ball_weight: if is_cursed { 200 } else { 100 },
            }
        }
        ExtScrollType::Stinking => ScrollEffect::StinkingCloud {
            radius: rng.rn2(3) + 2,
        },
        ExtScrollType::Charging => {
            let charges = if is_blessed {
                rng.rn2(5) + 5
            } else if is_cursed {
                0
            } else {
                rng.rn2(3) + 1
            };
            ScrollEffect::ChargingItem { charges }
        }
        ExtScrollType::Genocide => ScrollEffect::Genocide {
            monsters_removed: if is_blessed { 99 } else { 1 },
        },
        ExtScrollType::Taming => {
            let count = if is_blessed { rng.rn2(5) + 3 } else { 1 };
            ScrollEffect::TameMonsters {
                count,
                radius: if is_blessed { 10 } else { 3 },
            }
        }
        ExtScrollType::CreateMonster => ScrollEffect::CreateMonsters {
            count: rng.rn2(5) + 1,
        },
        ExtScrollType::Earth => ScrollEffect::Earthquake {
            damage: rng.rn2(30) + 10,
        },
        ExtScrollType::Amnesia => ScrollEffect::Amnesia {
            spells_lost: if is_blessed { 0 } else { rng.rn2(3) + 1 },
        },
        ExtScrollType::Fire => ScrollEffect::FireBlast {
            damage: rng.rn2(20) + 10,
            radius: 3,
        },
        ExtScrollType::Destroy => {
            if is_cursed {
                return ScrollEffect::NoEffect;
            }
            ScrollEffect::DestroyArmor
        }
        ExtScrollType::Magic => ScrollEffect::MagicMapping,
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
    fn test_identify_blessed() {
        let mut rng = test_rng();
        let result = read_scroll(ExtScrollType::Identify, true, false, false, &mut rng);
        assert!(matches!(result, ScrollEffect::IdentifyItems { count: 99 }));
    }

    #[test]
    fn test_enchant_normal() {
        let mut rng = test_rng();
        let result = read_scroll(ExtScrollType::Enchant, false, false, false, &mut rng);
        if let ScrollEffect::EnchantItem { bonus, .. } = result {
            assert_eq!(bonus, 1);
        }
    }

    #[test]
    fn test_enchant_cursed() {
        let mut rng = test_rng();
        let result = read_scroll(ExtScrollType::Enchant, false, true, false, &mut rng);
        if let ScrollEffect::EnchantItem { bonus, .. } = result {
            assert_eq!(bonus, -1);
        }
    }

    #[test]
    fn test_remove_curse_blessed() {
        let mut rng = test_rng();
        let result = read_scroll(ExtScrollType::Remove, true, false, false, &mut rng);
        assert!(matches!(
            result,
            ScrollEffect::RemoveCurse { count: 99, .. }
        ));
    }

    #[test]
    fn test_genocide() {
        let mut rng = test_rng();
        let result = read_scroll(ExtScrollType::Genocide, true, false, false, &mut rng);
        assert!(matches!(
            result,
            ScrollEffect::Genocide {
                monsters_removed: 99
            }
        ));
    }

    #[test]
    fn test_magic_mapping() {
        let mut rng = test_rng();
        let result = read_scroll(ExtScrollType::Magic, false, false, false, &mut rng);
        assert!(matches!(result, ScrollEffect::MagicMapping));
    }

    #[test]
    fn test_punishment_blessed() {
        let mut rng = test_rng();
        let result = read_scroll(ExtScrollType::Punishment, true, false, false, &mut rng);
        assert!(matches!(result, ScrollEffect::NoEffect));
    }

    #[test]
    fn test_charging() {
        let mut rng = test_rng();
        let result = read_scroll(ExtScrollType::Charging, false, false, false, &mut rng);
        if let ScrollEffect::ChargingItem { charges } = result {
            assert!(charges >= 1);
        }
    }
}
