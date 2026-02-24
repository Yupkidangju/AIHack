// ============================================================================
// [v2.36.0 R24-2] 포션 퀘프 효과 (potion_quaff_ext.rs)
// 원본: NetHack 3.6.7 potion.c quaff 확장
// 개별 포션 효과, BUC 변형, 혼란/환각 상태 변이
// ============================================================================

/// [v2.36.0 R24-2] 포션 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuaffEffect {
    Heal(i32),
    FullHeal,
    GainAbility(String),
    GainLevel,
    Speed(i32), // 턴 수
    Invisibility(i32),
    SeeInvisible,
    Levitation(i32),
    Confusion(i32),
    Blindness(i32),
    Hallucination(i32),
    Sleeping(i32),
    Paralysis(i32),
    Sickness,
    Restore,       // 능력치 복원
    Enlightenment, // 상태 확인
    Polymorph,
    Booze(i32), // 혼란+치유
    Nothing,
}

/// [v2.36.0 R24-2] 포션 마시기 (원본: dodrink quaff)
pub fn quaff_potion(potion_name: &str, blessed: bool, cursed: bool) -> QuaffEffect {
    match potion_name {
        "healing" => {
            let amount = if blessed {
                20
            } else if cursed {
                4
            } else {
                8
            };
            QuaffEffect::Heal(amount)
        }
        "extra healing" => {
            let amount = if blessed { 40 } else { 20 };
            QuaffEffect::Heal(amount)
        }
        "full healing" => QuaffEffect::FullHeal,
        "gain ability" => QuaffEffect::GainAbility(if blessed {
            "all".to_string()
        } else {
            "random".to_string()
        }),
        "gain level" => {
            if cursed {
                QuaffEffect::Nothing
            } else {
                QuaffEffect::GainLevel
            }
        }
        "speed" => QuaffEffect::Speed(if blessed { 200 } else { 100 }),
        "invisibility" => {
            if cursed {
                QuaffEffect::SeeInvisible
            } else {
                QuaffEffect::Invisibility(if blessed { 300 } else { 150 })
            }
        }
        "see invisible" => QuaffEffect::SeeInvisible,
        "levitation" => QuaffEffect::Levitation(if blessed { 250 } else { 150 }),
        "confusion" => QuaffEffect::Confusion(if blessed { 5 } else { 20 }),
        "blindness" => QuaffEffect::Blindness(if blessed { 5 } else { 200 }),
        "hallucination" => QuaffEffect::Hallucination(if blessed { 5 } else { 300 }),
        "sleeping" => QuaffEffect::Sleeping(if blessed { 1 } else { 20 }),
        "paralysis" => QuaffEffect::Paralysis(if blessed { 1 } else { 10 }),
        "sickness" => {
            if blessed {
                QuaffEffect::Nothing
            } else {
                QuaffEffect::Sickness
            }
        }
        "restore ability" => QuaffEffect::Restore,
        "enlightenment" => QuaffEffect::Enlightenment,
        "polymorph" => QuaffEffect::Polymorph,
        "booze" => QuaffEffect::Booze(if blessed { 1 } else { 15 }),
        _ => QuaffEffect::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healing() {
        assert_eq!(quaff_potion("healing", true, false), QuaffEffect::Heal(20));
        assert_eq!(quaff_potion("healing", false, true), QuaffEffect::Heal(4));
    }

    #[test]
    fn test_full_healing() {
        assert_eq!(
            quaff_potion("full healing", false, false),
            QuaffEffect::FullHeal
        );
    }

    #[test]
    fn test_gain_level_cursed() {
        assert_eq!(
            quaff_potion("gain level", false, true),
            QuaffEffect::Nothing
        );
    }

    #[test]
    fn test_blessed_shorter() {
        if let QuaffEffect::Confusion(t) = quaff_potion("confusion", true, false) {
            assert_eq!(t, 5);
        }
        if let QuaffEffect::Confusion(t) = quaff_potion("confusion", false, false) {
            assert_eq!(t, 20);
        }
    }

    #[test]
    fn test_sickness_blessed() {
        assert_eq!(quaff_potion("sickness", true, false), QuaffEffect::Nothing);
    }
}
