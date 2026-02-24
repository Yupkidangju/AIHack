// ============================================================================
// [v2.33.0 R21-3] 인그레이브 계산 (engrave_calc_ext.rs)
// 원본: NetHack 3.6.7 engrave.c 확장
// 각인 내구도, 먼지/영구 각인, Elbereth 효과
// ============================================================================

/// [v2.33.0 R21-3] 각인 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngraveType {
    Dust,       // 먼지 (쉽게 사라짐)
    Engrave,    // 단단한 도구 (내구)
    Burn,       // 마법 (반영구)
    BloodWrite, // 피 (빠르게 사라짐)
    Headstone,  // 묘비 (영구)
}

/// [v2.33.0 R21-3] 각인 내구도 (턴 수)
pub fn engrave_durability(etype: EngraveType) -> i32 {
    match etype {
        EngraveType::Dust => 20,
        EngraveType::Engrave => 200,
        EngraveType::Burn => 500,
        EngraveType::BloodWrite => 10,
        EngraveType::Headstone => i32::MAX,
    }
}

/// [v2.33.0 R21-3] Elbereth 유효성 (원본: scare_monst)
pub fn elbereth_effective(
    engrave_type: EngraveType,
    age_turns: i32,
    player_standing_on: bool,
) -> bool {
    if !player_standing_on {
        return false;
    }
    let durability = engrave_durability(engrave_type);
    age_turns < durability
}

/// [v2.33.0 R21-3] Elbereth 공포 효과 대상 판별
pub fn monster_fears_elbereth(monster_level: i32, is_unique: bool, is_blind: bool) -> bool {
    if is_unique {
        return false;
    } // 유니크 몬스터는 면역
    if is_blind {
        return false;
    } // 보이지 않으면 무효
    monster_level < 20 // 레벨 20 이상은 무효
}

/// [v2.33.0 R21-3] 각인 속도 (도구별)
pub fn engrave_turns(text_len: usize, etype: EngraveType) -> i32 {
    let per_char = match etype {
        EngraveType::Dust => 1,
        EngraveType::Engrave => 2,
        EngraveType::Burn => 1,
        EngraveType::BloodWrite => 1,
        EngraveType::Headstone => 0,
    };
    (text_len as i32 * per_char).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_durability() {
        assert_eq!(engrave_durability(EngraveType::Dust), 20);
        assert_eq!(engrave_durability(EngraveType::Headstone), i32::MAX);
    }

    #[test]
    fn test_elbereth_fresh() {
        assert!(elbereth_effective(EngraveType::Burn, 10, true));
    }

    #[test]
    fn test_elbereth_expired() {
        assert!(!elbereth_effective(EngraveType::Dust, 30, true));
    }

    #[test]
    fn test_elbereth_not_standing() {
        assert!(!elbereth_effective(EngraveType::Burn, 0, false));
    }

    #[test]
    fn test_monster_fears() {
        assert!(monster_fears_elbereth(5, false, false));
        assert!(!monster_fears_elbereth(5, true, false)); // 유니크
        assert!(!monster_fears_elbereth(25, false, false)); // 고레벨
    }

    #[test]
    fn test_engrave_turns() {
        assert_eq!(engrave_turns(7, EngraveType::Engrave), 14);
    }
}
