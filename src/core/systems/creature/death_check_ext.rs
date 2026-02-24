// ============================================================================
// [v2.36.0 R24-4] 죽음/부활 판정 (death_check_ext.rs)
// 원본: NetHack 3.6.7 uhitm.c/end.c 죽음 확장
// 사망 원인, 라이프세이빙, 유령 생성
// ============================================================================

/// [v2.36.0 R24-4] 사망 원인
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeathCause {
    Monster(String),
    Starvation,
    Poisoning,
    Petrification,
    Drowning,
    Burning,
    Falling,
    Crushing,
    Zapped(String), // 마법
    Sickness,
    Strangulation,
    Genocide, // 자신 제노사이드
    Quit,
    Escaped,
    Ascended,
}

/// [v2.36.0 R24-4] 부활 판정
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeathResult {
    /// 진짜 사망
    Dead { cause: String, epitaph: String },
    /// 라이프세이빙으로 부활
    LifeSaved,
    /// 아뮬렛 of life saving 소진
    Resurrected { hp_restored: i32 },
}

/// [v2.36.0 R24-4] 사망 판정 (원본: done)
pub fn check_death(
    hp: i32,
    cause: DeathCause,
    has_life_saving: bool,
    has_amulet_life: bool,
) -> DeathResult {
    if hp > 0 {
        // 살아있음 — 이 함수가 호출되면 안 됨
        return DeathResult::Resurrected { hp_restored: hp };
    }

    // 일부 즉사는 라이프세이빙도 불가 (제노사이드, 퇴장)
    let unresurrectable = matches!(cause, DeathCause::Genocide | DeathCause::Quit);
    if unresurrectable {
        return DeathResult::Dead {
            cause: format!("{:?}", cause),
            epitaph: "자초한 일이었다.".to_string(),
        };
    }

    // 라이프세이빙
    if has_life_saving || has_amulet_life {
        return DeathResult::LifeSaved;
    }

    let epitaph = match &cause {
        DeathCause::Monster(name) => format!("{}에게 죽었다.", name),
        DeathCause::Starvation => "굶어 죽었다.".to_string(),
        DeathCause::Petrification => "돌이 되었다.".to_string(),
        DeathCause::Drowning => "익사했다.".to_string(),
        DeathCause::Ascended => "승천했다!".to_string(),
        _ => "알 수 없는 원인으로 죽었다.".to_string(),
    };

    DeathResult::Dead {
        cause: format!("{:?}", cause),
        epitaph,
    }
}

/// [v2.36.0 R24-4] 유령 생성 가능 여부
pub fn can_leave_ghost(cause: &DeathCause) -> bool {
    !matches!(
        cause,
        DeathCause::Quit | DeathCause::Escaped | DeathCause::Ascended | DeathCause::Genocide
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_death() {
        let r = check_death(0, DeathCause::Monster("dragon".into()), false, false);
        assert!(matches!(r, DeathResult::Dead { .. }));
    }

    #[test]
    fn test_life_saving() {
        let r = check_death(0, DeathCause::Monster("lich".into()), true, false);
        assert_eq!(r, DeathResult::LifeSaved);
    }

    #[test]
    fn test_genocide_no_save() {
        let r = check_death(0, DeathCause::Genocide, true, true);
        assert!(matches!(r, DeathResult::Dead { .. }));
    }

    #[test]
    fn test_ghost() {
        assert!(can_leave_ghost(&DeathCause::Monster("troll".into())));
        assert!(!can_leave_ghost(&DeathCause::Ascended));
    }

    #[test]
    fn test_epitaph() {
        if let DeathResult::Dead { epitaph, .. } =
            check_death(0, DeathCause::Starvation, false, false)
        {
            assert!(epitaph.contains("굶어"));
        }
    }
}
