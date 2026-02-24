// ============================================================================
// [v2.31.0 R19-2] 성직자 봉헌 확장 (priest_temple_ext.rs)
// 원본: NetHack 3.6.7 priest.c (470줄) 추가 포팅
// 성직자 행동, 성수 생성, 신전 보호, 제단 관리
// ============================================================================

/// [v2.31.0 R19-2] 성직자 행동
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriestAction {
    SellHolyWater { price: i32 },
    GuideSacrifice,
    WarnIntruder,
    AttackIntruder,
    Heal { amount: i32 },
    Ignore,
}

/// [v2.31.0 R19-2] 성직자 반응 (원본: pri_move)
pub fn priest_reaction(
    player_in_temple: bool,
    player_alignment_matches: bool,
    player_hostile: bool,
    player_has_amulet: bool,
) -> PriestAction {
    if player_hostile {
        return PriestAction::AttackIntruder;
    }
    if player_in_temple && !player_alignment_matches {
        return PriestAction::WarnIntruder;
    }
    if player_has_amulet && player_in_temple {
        return PriestAction::AttackIntruder;
    }
    if player_in_temple && player_alignment_matches {
        return PriestAction::SellHolyWater { price: 100 };
    }
    PriestAction::Ignore
}

/// [v2.31.0 R19-2] 성수 가격
pub fn holy_water_price(player_level: i32, is_charismatic: bool) -> i32 {
    let base = 100 + player_level * 5;
    if is_charismatic {
        base * 3 / 4
    } else {
        base
    }
}

/// [v2.31.0 R19-2] 제단 종류
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempleType {
    Lawful,
    Neutral,
    Chaotic,
    Moloch,
}

pub fn offering_multiplier(temple: TempleType) -> f64 {
    match temple {
        TempleType::Lawful => 1.0,
        TempleType::Neutral => 1.2,
        TempleType::Chaotic => 1.5,
        TempleType::Moloch => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friendly() {
        assert!(matches!(
            priest_reaction(true, true, false, false),
            PriestAction::SellHolyWater { .. }
        ));
    }

    #[test]
    fn test_wrong_align() {
        assert_eq!(
            priest_reaction(true, false, false, false),
            PriestAction::WarnIntruder
        );
    }

    #[test]
    fn test_hostile() {
        assert_eq!(
            priest_reaction(true, true, true, false),
            PriestAction::AttackIntruder
        );
    }

    #[test]
    fn test_price() {
        assert_eq!(holy_water_price(10, false), 150);
    }

    #[test]
    fn test_moloch() {
        assert_eq!(offering_multiplier(TempleType::Moloch), 0.0);
    }
}
