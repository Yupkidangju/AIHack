// ============================================================================
// [v2.42.0 R30-5] 레벨 전환 효과 (level_change_ext.rs)
// 원본: NetHack 3.6.7 do.c 레벨 전환 확장
// 레벨 이동 시 효과, 보존, 몬스터 이동
// ============================================================================

/// [v2.42.0 R30-5] 레벨 전환 시 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelTransitionEffect {
    PetFollows,
    PetStays, // 펫이 남음 (너무 멀거나 부유 등)
    MonstersReset,
    ItemsPreserved,
    BranchEntry(String),
    LostInventory, // 특수 이벤트
}

/// [v2.42.0 R30-5] 펫 동행 판정
pub fn pet_follows(
    distance_to_pet: i32,
    pet_leashed: bool,
    pet_can_fly: bool,
    going_down_hole: bool,
) -> bool {
    if pet_leashed {
        return true;
    }
    if going_down_hole && !pet_can_fly {
        return false;
    }
    distance_to_pet <= 3
}

/// [v2.42.0 R30-5] 레벨 초기화 필요 여부
pub fn needs_level_gen(visited_before: bool) -> bool {
    !visited_before
}

/// [v2.42.0 R30-5] 분기 진입 메시지
pub fn branch_entry_message(branch: &str) -> String {
    match branch {
        "mines" => "광산의 기운이 느껴진다!".to_string(),
        "sokoban" => "퍼즐 같은 방이 보인다...".to_string(),
        "quest" => "퀘스트의 부름을 느낀다!".to_string(),
        "gehennom" => "지옥의 열기가 타오른다!".to_string(),
        "astral" => "승천의 시간이 가까워졌다!".to_string(),
        _ => "새로운 영역에 들어섰다.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pet_leash() {
        assert!(pet_follows(10, true, false, false));
    }

    #[test]
    fn test_pet_far() {
        assert!(!pet_follows(5, false, false, false));
    }

    #[test]
    fn test_pet_hole() {
        assert!(!pet_follows(1, false, false, true));
    }

    #[test]
    fn test_needs_gen() {
        assert!(needs_level_gen(false));
        assert!(!needs_level_gen(true));
    }

    #[test]
    fn test_branch_msg() {
        assert!(branch_entry_message("gehennom").contains("지옥"));
    }
}
