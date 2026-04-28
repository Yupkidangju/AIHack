// ============================================================================
// [v2.27.0 Phase 91-8] 읽기 시스템 확장 (read_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/read.c L800-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 두루마리 효과 — scroll_effect (read.c L800-1500)
// =============================================================================

/// [v2.27.0 91-8] 두루마리 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollType {
    Identify,
    RemoveCurse,
    Enchant,
    Teleport,
    CreateMonster,
    Genocide,
    MagicMapping,
    FireScroll,
    EarthScroll,
    Punish,
    Stink,
    Blank,
    Light,
    Gold,
    Taming,
    Charging,
    Confuse,
    Destroy,
}

/// [v2.27.0 91-8] 두루마리 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScrollEffectResult {
    /// 아이템 식별
    Identify { count: i32 },
    /// 저주 해제
    RemoveCurse { count: i32, blessed: bool },
    /// 아이템 강화
    Enchant { slot: String, amount: i32 },
    /// 텔레포트
    Teleport,
    /// 몬스터 소환
    SummonMonsters { count: i32 },
    /// 제노사이드 (종족 제거)
    Genocide { prompt: bool },
    /// 마법 지도
    MagicMap,
    /// 화염 피해
    FireDamage { amount: i32 },
    /// 지진
    Earthquake { radius: i32 },
    /// 처벌 (철구)
    Punishment,
    /// 금화 생성
    CreateGold { amount: i32 },
    /// 빛
    Light { radius: i32 },
    /// 길들기
    TameMonsters { radius: i32 },
    /// 충전
    Charge { count: i32 },
    /// 혼란 부여
    ConfuseEffect,
    /// 갑옷 파괴
    DestroyArmor,
    /// 아무 효과 없음
    NoEffect { message: String },
}

/// [v2.27.0 91-8] 두루마리 효과 판정
/// 원본: read.c seffects()
pub fn scroll_effect(
    scroll: ScrollType,
    is_confused: bool,
    is_blessed: bool,
    is_cursed: bool,
    rng: &mut NetHackRng,
) -> ScrollEffectResult {
    match scroll {
        ScrollType::Identify => {
            let count = if is_blessed {
                -1
            } else if is_cursed {
                1
            } else {
                rng.rn2(3) + 1
            };
            ScrollEffectResult::Identify { count } // -1 = 전체 식별
        }
        ScrollType::RemoveCurse => {
            if is_cursed {
                return ScrollEffectResult::NoEffect {
                    message: "두루마리가 바스라졌다.".to_string(),
                };
            }
            let count = if is_blessed { -1 } else { rng.rn2(3) + 1 };
            ScrollEffectResult::RemoveCurse {
                count,
                blessed: is_blessed,
            }
        }
        ScrollType::Enchant => {
            let amount = if is_blessed {
                2
            } else if is_cursed {
                -1
            } else {
                1
            };
            let slot = if is_confused {
                "weapon".to_string()
            } else {
                "armor".to_string()
            };
            ScrollEffectResult::Enchant { slot, amount }
        }
        ScrollType::Teleport => ScrollEffectResult::Teleport,
        ScrollType::CreateMonster => {
            let count = if is_cursed {
                rng.rn2(4) + 3
            } else {
                rng.rn2(2) + 1
            };
            ScrollEffectResult::SummonMonsters { count }
        }
        ScrollType::Genocide => ScrollEffectResult::Genocide { prompt: !is_cursed },
        ScrollType::MagicMapping => ScrollEffectResult::MagicMap,
        ScrollType::FireScroll => {
            let damage = if is_cursed {
                rng.rn2(12) + 12
            } else {
                rng.rn2(6) + 3
            };
            ScrollEffectResult::FireDamage { amount: damage }
        }
        ScrollType::EarthScroll => ScrollEffectResult::Earthquake {
            radius: if is_blessed { 3 } else { 5 },
        },
        ScrollType::Punish => ScrollEffectResult::Punishment,
        ScrollType::Gold => {
            let amount = rng.rn2(100) + 50;
            ScrollEffectResult::CreateGold { amount }
        }
        ScrollType::Light => ScrollEffectResult::Light {
            radius: if is_blessed { 7 } else { 5 },
        },
        ScrollType::Taming => ScrollEffectResult::TameMonsters {
            radius: if is_blessed { 5 } else { 3 },
        },
        ScrollType::Charging => {
            let count = if is_blessed { 2 } else { 1 };
            ScrollEffectResult::Charge { count }
        }
        ScrollType::Confuse => ScrollEffectResult::ConfuseEffect,
        ScrollType::Destroy => {
            if is_blessed {
                ScrollEffectResult::NoEffect {
                    message: "축복의 보호로 효과 없음.".to_string(),
                }
            } else {
                ScrollEffectResult::DestroyArmor
            }
        }
        ScrollType::Blank | ScrollType::Stink => ScrollEffectResult::NoEffect {
            message: "아무 일도 일어나지 않았다.".to_string(),
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
    fn test_identify_blessed() {
        let mut rng = test_rng();
        let result = scroll_effect(ScrollType::Identify, false, true, false, &mut rng);
        assert!(matches!(result, ScrollEffectResult::Identify { count: -1 }));
    }

    #[test]
    fn test_remove_curse_cursed() {
        let mut rng = test_rng();
        let result = scroll_effect(ScrollType::RemoveCurse, false, false, true, &mut rng);
        assert!(matches!(result, ScrollEffectResult::NoEffect { .. }));
    }

    #[test]
    fn test_enchant_armor() {
        let mut rng = test_rng();
        let result = scroll_effect(ScrollType::Enchant, false, false, false, &mut rng);
        if let ScrollEffectResult::Enchant { slot, amount } = result {
            assert_eq!(slot, "armor");
            assert_eq!(amount, 1);
        } else {
            panic!("강화 기대");
        }
    }

    #[test]
    fn test_enchant_confused() {
        let mut rng = test_rng();
        let result = scroll_effect(ScrollType::Enchant, true, false, false, &mut rng);
        if let ScrollEffectResult::Enchant { slot, .. } = result {
            assert_eq!(slot, "weapon");
        }
    }

    #[test]
    fn test_teleport_scroll() {
        let mut rng = test_rng();
        let result = scroll_effect(ScrollType::Teleport, false, false, false, &mut rng);
        assert!(matches!(result, ScrollEffectResult::Teleport));
    }

    #[test]
    fn test_genocide_prompt() {
        let mut rng = test_rng();
        let result = scroll_effect(ScrollType::Genocide, false, false, false, &mut rng);
        assert!(matches!(
            result,
            ScrollEffectResult::Genocide { prompt: true }
        ));
    }

    #[test]
    fn test_blank_no_effect() {
        let mut rng = test_rng();
        let result = scroll_effect(ScrollType::Blank, false, false, false, &mut rng);
        assert!(matches!(result, ScrollEffectResult::NoEffect { .. }));
    }
}
