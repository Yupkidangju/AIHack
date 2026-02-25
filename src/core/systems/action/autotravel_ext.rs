// ============================================================================
// [v2.43.0 R31-4] 자동행동 (autotravel_ext.rs)
// 원본: NetHack 3.6.7 cmd.c/travel 확장
// 자동이동, 탐색 모드, 위험 회피, 중단 조건
// ============================================================================

/// [v2.43.0 R31-4] 자동이동 중단 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TravelInterrupt {
    MonsterSeen,
    TrapDetected,
    DoorFound,
    ItemFound,
    Arrived,
    WallHit,
    HpLow,
    StatusChange,
}

/// [v2.43.0 R31-4] 자동이동 체크
pub fn should_interrupt(
    monster_nearby: bool,
    trap_adjacent: bool,
    door_adjacent: bool,
    item_on_tile: bool,
    arrived: bool,
    hp_pct: f64,
) -> Option<TravelInterrupt> {
    if monster_nearby {
        return Some(TravelInterrupt::MonsterSeen);
    }
    if hp_pct < 0.3 {
        return Some(TravelInterrupt::HpLow);
    }
    if trap_adjacent {
        return Some(TravelInterrupt::TrapDetected);
    }
    if door_adjacent {
        return Some(TravelInterrupt::DoorFound);
    }
    if item_on_tile {
        return Some(TravelInterrupt::ItemFound);
    }
    if arrived {
        return Some(TravelInterrupt::Arrived);
    }
    None
}

/// [v2.43.0 R31-4] 자동탐색 우선순위
pub fn explore_priority(unexplored: bool, has_stairs: bool, has_shop: bool) -> i32 {
    let mut p = 0;
    if unexplored {
        p += 10;
    }
    if has_stairs {
        p += 5;
    }
    if has_shop {
        p += 3;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_interrupt() {
        assert_eq!(
            should_interrupt(true, false, false, false, false, 1.0),
            Some(TravelInterrupt::MonsterSeen)
        );
    }

    #[test]
    fn test_arrived() {
        assert_eq!(
            should_interrupt(false, false, false, false, true, 1.0),
            Some(TravelInterrupt::Arrived)
        );
    }

    #[test]
    fn test_no_interrupt() {
        assert_eq!(
            should_interrupt(false, false, false, false, false, 1.0),
            None
        );
    }

    #[test]
    fn test_hp_low() {
        assert_eq!(
            should_interrupt(false, false, false, false, false, 0.2),
            Some(TravelInterrupt::HpLow)
        );
    }

    #[test]
    fn test_priority() {
        assert!(explore_priority(true, true, false) > explore_priority(false, false, false));
    }
}
