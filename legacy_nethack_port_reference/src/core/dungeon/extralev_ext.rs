// ============================================================================
// [v2.29.0 R17-3] 특수 레벨 (extralev_ext.rs)
// 원본: NetHack 3.6.7 extralev.c (282줄) + 엔드게임 레벨
// Mine End, Medusa, Castle, Valley, Sanctum, Astral Plane
// ============================================================================

/// [v2.29.0 R17-3] 특수 레벨 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialLevel {
    MineEnd,
    OracleLevel,
    BigRoom,
    MedusaLevel,
    CastleLevel,
    ValleyOfDead,
    Sanctum,
    VladsTower1,
    VladsTower2,
    VladsTower3,
    WizardTower1,
    WizardTower2,
    WizardTower3,
    AstralPlane,
    EarthPlane,
    WaterPlane,
    FirePlane,
    AirPlane,
}

/// [v2.29.0 R17-3] 특수 레벨 속성
#[derive(Debug, Clone)]
pub struct SpecialLevelInfo {
    pub level: SpecialLevel,
    pub depth: i32,
    pub branch: &'static str,
    pub is_maze: bool,
    pub no_teleport: bool,
    pub no_dig: bool,
    pub boss_name: Option<&'static str>,
}

/// [v2.29.0 R17-3] 특수 레벨 데이터베이스
pub fn special_levels() -> Vec<SpecialLevelInfo> {
    vec![
        SpecialLevelInfo {
            level: SpecialLevel::MineEnd,
            depth: 13,
            branch: "mines",
            is_maze: false,
            no_teleport: false,
            no_dig: false,
            boss_name: None,
        },
        SpecialLevelInfo {
            level: SpecialLevel::OracleLevel,
            depth: 5,
            branch: "main",
            is_maze: false,
            no_teleport: false,
            no_dig: false,
            boss_name: None,
        },
        SpecialLevelInfo {
            level: SpecialLevel::BigRoom,
            depth: 12,
            branch: "main",
            is_maze: false,
            no_teleport: false,
            no_dig: false,
            boss_name: None,
        },
        SpecialLevelInfo {
            level: SpecialLevel::MedusaLevel,
            depth: 24,
            branch: "main",
            is_maze: false,
            no_teleport: false,
            no_dig: true,
            boss_name: Some("Medusa"),
        },
        SpecialLevelInfo {
            level: SpecialLevel::CastleLevel,
            depth: 25,
            branch: "main",
            is_maze: false,
            no_teleport: true,
            no_dig: true,
            boss_name: None,
        },
        SpecialLevelInfo {
            level: SpecialLevel::ValleyOfDead,
            depth: 26,
            branch: "gehennom",
            is_maze: true,
            no_teleport: true,
            no_dig: false,
            boss_name: None,
        },
        SpecialLevelInfo {
            level: SpecialLevel::Sanctum,
            depth: 45,
            branch: "gehennom",
            is_maze: true,
            no_teleport: true,
            no_dig: true,
            boss_name: Some("High Priest"),
        },
        SpecialLevelInfo {
            level: SpecialLevel::VladsTower1,
            depth: 36,
            branch: "vlad",
            is_maze: true,
            no_teleport: true,
            no_dig: true,
            boss_name: Some("Vlad the Impaler"),
        },
        SpecialLevelInfo {
            level: SpecialLevel::WizardTower1,
            depth: 40,
            branch: "wizard",
            is_maze: true,
            no_teleport: true,
            no_dig: true,
            boss_name: Some("Wizard of Yendor"),
        },
        SpecialLevelInfo {
            level: SpecialLevel::AstralPlane,
            depth: 0,
            branch: "endgame",
            is_maze: false,
            no_teleport: true,
            no_dig: true,
            boss_name: Some("Riders"),
        },
        SpecialLevelInfo {
            level: SpecialLevel::EarthPlane,
            depth: 1,
            branch: "endgame",
            is_maze: true,
            no_teleport: true,
            no_dig: true,
            boss_name: None,
        },
        SpecialLevelInfo {
            level: SpecialLevel::WaterPlane,
            depth: 2,
            branch: "endgame",
            is_maze: true,
            no_teleport: true,
            no_dig: true,
            boss_name: None,
        },
        SpecialLevelInfo {
            level: SpecialLevel::FirePlane,
            depth: 3,
            branch: "endgame",
            is_maze: true,
            no_teleport: true,
            no_dig: true,
            boss_name: None,
        },
        SpecialLevelInfo {
            level: SpecialLevel::AirPlane,
            depth: 4,
            branch: "endgame",
            is_maze: true,
            no_teleport: true,
            no_dig: true,
            boss_name: None,
        },
    ]
}

/// [v2.29.0 R17-3] 깊이로 특수 레벨 조회
pub fn find_special_at_depth(depth: i32, branch: &str) -> Option<SpecialLevelInfo> {
    special_levels()
        .into_iter()
        .find(|sl| sl.depth == depth && sl.branch == branch)
}

/// [v2.29.0 R17-3] 보스 이름으로 조회
pub fn find_level_with_boss(boss: &str) -> Option<SpecialLevelInfo> {
    special_levels()
        .into_iter()
        .find(|sl| sl.boss_name == Some(boss))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_levels_count() {
        assert_eq!(special_levels().len(), 14);
    }

    #[test]
    fn test_find_medusa() {
        let sl = find_special_at_depth(24, "main");
        assert!(sl.is_some());
        assert_eq!(sl.unwrap().boss_name, Some("Medusa"));
    }

    #[test]
    fn test_find_boss() {
        let sl = find_level_with_boss("Wizard of Yendor");
        assert!(sl.is_some());
        assert!(sl.unwrap().no_teleport);
    }

    #[test]
    fn test_endgame_no_dig() {
        let astral = find_special_at_depth(0, "endgame").unwrap();
        assert!(astral.no_dig);
        assert!(astral.no_teleport);
    }

    #[test]
    fn test_not_found() {
        assert!(find_special_at_depth(99, "main").is_none());
    }
}
