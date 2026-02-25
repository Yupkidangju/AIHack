// ============================================================================
// [v2.42.0 R30-3] 정렬 시스템 (alignment_ext.rs)
// 원본: NetHack 3.6.7 attrib.c/pray.c 정렬 확장
// 정렬 변동, 정렬 기록, 신 분노
// ============================================================================

/// [v2.42.0 R30-3] 정렬
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Lawful,
    Neutral,
    Chaotic,
}

/// [v2.42.0 R30-3] 정렬 기록 변동 원인
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlignAction {
    KillAligned,    // 같은 정렬 몬스터 처치 → -
    KillOpposed,    // 반대 정렬 처치 → +
    Sacrifice,      // 봉헌 → +
    StealFromShop,  // 도둑질 → -
    AttackPeaceful, // 평화 몬스터 공격 → -
    Pray,           // 기도 → 약간 +
    Convert,        // 개종 → 리셋
}

pub fn align_delta(action: &AlignAction) -> i32 {
    match action {
        AlignAction::KillAligned => -2,
        AlignAction::KillOpposed => 1,
        AlignAction::Sacrifice => 3,
        AlignAction::StealFromShop => -5,
        AlignAction::AttackPeaceful => -3,
        AlignAction::Pray => 1,
        AlignAction::Convert => 0,
    }
}

pub fn update_alignment(current: i32, action: &AlignAction) -> i32 {
    let delta = align_delta(action);
    (current + delta).clamp(-128, 127)
}

/// [v2.42.0 R30-3] 신 분노 여부
pub fn god_angry(alignment_record: i32) -> bool {
    alignment_record < -10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sacrifice() {
        assert_eq!(update_alignment(10, &AlignAction::Sacrifice), 13);
    }

    #[test]
    fn test_steal() {
        assert_eq!(update_alignment(0, &AlignAction::StealFromShop), -5);
    }

    #[test]
    fn test_god_angry() {
        assert!(god_angry(-15));
        assert!(!god_angry(0));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(update_alignment(126, &AlignAction::Sacrifice), 127);
    }
}
