// ============================================================================
// [v2.43.0 R31-2] 몬스터 AI 전술 (ai_tactic_ext.rs)
// 원본: NetHack 3.6.7 monmove.c/mon.c 전술 확장
// 몬스터 전투 스타일, 도주, 잠복
// ============================================================================

/// [v2.43.0 R31-2] AI 전술
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTactic {
    Aggressive, // 즉시 공격
    Cautious,   // HP<50% 시 후퇴
    Sniper,     // 원거리 우선
    Ambush,     // 잠복 후 기습
    Flee,       // 도주
    Summon,     // 소환 후 후방
    Healer,     // 아군 치유 우선
    Berserk,    // HP 관계없이 돌격
}

/// [v2.43.0 R31-2] 전술 결정
pub fn choose_tactic(
    hp_pct: f64,
    has_ranged: bool,
    can_summon: bool,
    can_heal: bool,
    is_cornered: bool,
) -> AiTactic {
    if hp_pct < 0.2 && !is_cornered {
        return AiTactic::Flee;
    }
    if hp_pct < 0.2 && is_cornered {
        return AiTactic::Berserk;
    }
    if can_heal && hp_pct < 0.5 {
        return AiTactic::Healer;
    }
    if can_summon && hp_pct < 0.7 {
        return AiTactic::Summon;
    }
    if has_ranged {
        return AiTactic::Sniper;
    }
    if hp_pct < 0.5 {
        return AiTactic::Cautious;
    }
    AiTactic::Aggressive
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flee() {
        assert_eq!(
            choose_tactic(0.15, false, false, false, false),
            AiTactic::Flee
        );
    }

    #[test]
    fn test_berserk() {
        assert_eq!(
            choose_tactic(0.1, false, false, false, true),
            AiTactic::Berserk
        );
    }

    #[test]
    fn test_sniper() {
        assert_eq!(
            choose_tactic(0.8, true, false, false, false),
            AiTactic::Sniper
        );
    }

    #[test]
    fn test_aggressive() {
        assert_eq!(
            choose_tactic(0.9, false, false, false, false),
            AiTactic::Aggressive
        );
    }

    #[test]
    fn test_summon() {
        assert_eq!(
            choose_tactic(0.6, false, true, false, false),
            AiTactic::Summon
        );
    }
}
