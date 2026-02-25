// ============================================================================
// [v2.38.0 R26-2] 몬스터 생성 제약 (spawn_rule_ext.rs)
// 원본: NetHack 3.6.7 makemon.c 제약 확장
// 깊이 제한, 고유 몬스터 중복 방지, 제노사이드, 분기 제약
// ============================================================================

/// [v2.38.0 R26-2] 생성 거부 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpawnDenied {
    TooDeep,
    TooShallow,
    Genocided,
    UniqueAlreadyExists,
    WrongBranch,
    ExtinctSpecies,
}

/// [v2.38.0 R26-2] 생성 가능 여부 판정
pub fn can_spawn(
    monster_min_depth: i32,
    monster_max_depth: i32,
    current_depth: i32,
    is_genocided: bool,
    is_unique: bool,
    unique_exists: bool,
    required_branch: Option<&str>,
    current_branch: &str,
    species_count: i32,
    extinction_limit: i32,
) -> Result<(), SpawnDenied> {
    if is_genocided {
        return Err(SpawnDenied::Genocided);
    }
    if is_unique && unique_exists {
        return Err(SpawnDenied::UniqueAlreadyExists);
    }
    if current_depth < monster_min_depth {
        return Err(SpawnDenied::TooShallow);
    }
    if monster_max_depth > 0 && current_depth > monster_max_depth {
        return Err(SpawnDenied::TooDeep);
    }
    if let Some(branch) = required_branch {
        if branch != current_branch {
            return Err(SpawnDenied::WrongBranch);
        }
    }
    if species_count >= extinction_limit {
        return Err(SpawnDenied::ExtinctSpecies);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed() {
        assert!(can_spawn(1, 30, 15, false, false, false, None, "main", 0, 120).is_ok());
    }

    #[test]
    fn test_genocided() {
        assert_eq!(
            can_spawn(1, 30, 15, true, false, false, None, "main", 0, 120),
            Err(SpawnDenied::Genocided)
        );
    }

    #[test]
    fn test_unique_exists() {
        assert_eq!(
            can_spawn(1, 30, 15, false, true, true, None, "main", 0, 120),
            Err(SpawnDenied::UniqueAlreadyExists)
        );
    }

    #[test]
    fn test_wrong_branch() {
        assert_eq!(
            can_spawn(
                1,
                30,
                15,
                false,
                false,
                false,
                Some("mines"),
                "main",
                0,
                120
            ),
            Err(SpawnDenied::WrongBranch)
        );
    }

    #[test]
    fn test_too_shallow() {
        assert_eq!(
            can_spawn(10, 30, 5, false, false, false, None, "main", 0, 120),
            Err(SpawnDenied::TooShallow)
        );
    }

    #[test]
    fn test_extinct() {
        assert_eq!(
            can_spawn(1, 30, 15, false, false, false, None, "main", 120, 120),
            Err(SpawnDenied::ExtinctSpecies)
        );
    }
}
