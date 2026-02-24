// ============================================================================
// [v2.34.0 R22-4] 상태이상 타이머 (status_timer_ext.rs)
// 원본: NetHack 3.6.7 timeout.c 상태이상 타이머 관리
// 각 상태이상의 남은 턴, 만료, 중첩
// ============================================================================

/// [v2.34.0 R22-4] 상태이상 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    Blind,
    Confused,
    Stunned,
    Hallucinating,
    Paralyzed,
    Sleeping,
    Sick,
    Stoning, // 석화 진행
    Sliming, // 슬라임화 진행
    Strangled,
    Levitating,
    Invisible,
    Hasted,
    Slowed,
}

/// [v2.34.0 R22-4] 타이머 엔트리
#[derive(Debug, Clone)]
pub struct StatusTimer {
    pub effect: StatusEffect,
    pub remaining_turns: i32,
    pub source: String,
}

/// [v2.34.0 R22-4] 턴 경과
pub fn tick_timer(timer: &mut StatusTimer) -> bool {
    timer.remaining_turns -= 1;
    timer.remaining_turns <= 0
}

/// [v2.34.0 R22-4] 효과 추가/중첩 (더 긴 쪽 유지)
pub fn apply_or_extend(
    timers: &mut Vec<StatusTimer>,
    effect: StatusEffect,
    turns: i32,
    source: &str,
) {
    if let Some(existing) = timers.iter_mut().find(|t| t.effect == effect) {
        if turns > existing.remaining_turns {
            existing.remaining_turns = turns;
            existing.source = source.to_string();
        }
    } else {
        timers.push(StatusTimer {
            effect,
            remaining_turns: turns,
            source: source.to_string(),
        });
    }
}

/// [v2.34.0 R22-4] 만료된 효과 제거
pub fn remove_expired(timers: &mut Vec<StatusTimer>) -> Vec<StatusEffect> {
    let expired: Vec<StatusEffect> = timers
        .iter()
        .filter(|t| t.remaining_turns <= 0)
        .map(|t| t.effect)
        .collect();
    timers.retain(|t| t.remaining_turns > 0);
    expired
}

/// [v2.34.0 R22-4] 특정 효과 활성 여부
pub fn is_active(timers: &[StatusTimer], effect: StatusEffect) -> bool {
    timers
        .iter()
        .any(|t| t.effect == effect && t.remaining_turns > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick() {
        let mut t = StatusTimer {
            effect: StatusEffect::Blind,
            remaining_turns: 2,
            source: "potion".into(),
        };
        assert!(!tick_timer(&mut t));
        assert!(tick_timer(&mut t)); // 만료
    }

    #[test]
    fn test_apply_new() {
        let mut timers = vec![];
        apply_or_extend(&mut timers, StatusEffect::Confused, 10, "trap");
        assert_eq!(timers.len(), 1);
    }

    #[test]
    fn test_extend() {
        let mut timers = vec![StatusTimer {
            effect: StatusEffect::Confused,
            remaining_turns: 5,
            source: "a".into(),
        }];
        apply_or_extend(&mut timers, StatusEffect::Confused, 15, "b");
        assert_eq!(timers[0].remaining_turns, 15);
    }

    #[test]
    fn test_no_downgrade() {
        let mut timers = vec![StatusTimer {
            effect: StatusEffect::Blind,
            remaining_turns: 20,
            source: "a".into(),
        }];
        apply_or_extend(&mut timers, StatusEffect::Blind, 5, "b");
        assert_eq!(timers[0].remaining_turns, 20); // 더 짧으면 무시
    }

    #[test]
    fn test_expired_removal() {
        let mut timers = vec![
            StatusTimer {
                effect: StatusEffect::Stunned,
                remaining_turns: 0,
                source: "a".into(),
            },
            StatusTimer {
                effect: StatusEffect::Hasted,
                remaining_turns: 5,
                source: "b".into(),
            },
        ];
        let expired = remove_expired(&mut timers);
        assert_eq!(expired, vec![StatusEffect::Stunned]);
        assert_eq!(timers.len(), 1);
    }

    #[test]
    fn test_is_active() {
        let timers = vec![StatusTimer {
            effect: StatusEffect::Invisible,
            remaining_turns: 10,
            source: "ring".into(),
        }];
        assert!(is_active(&timers, StatusEffect::Invisible));
        assert!(!is_active(&timers, StatusEffect::Blind));
    }
}
