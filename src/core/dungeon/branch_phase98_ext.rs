// ============================================================================
// [v2.34.0 Phase 98-2] 던전 분기 확장 (branch_phase98_ext.rs)
// 원본: NetHack 3.6.7 src/dungeon.c L500-1500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 던전 분기 — dungeon_branches (dungeon.c L500-1000)
// =============================================================================

/// [v2.34.0 98-2] 던전 분기 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonBranch {
    Main,
    GnomishMines,
    SokobanBranch,
    QuestBranch,
    FortLudiosBranch,
    VladTowerBranch,
    WizardTowerBranch,
    AstralPlaneBranch,
    Gehennom,
    EarthPlane,
    WaterPlane,
    FirePlane,
    AirPlane,
}

/// [v2.34.0 98-2] 분기 정보
#[derive(Debug, Clone)]
pub struct BranchInfo {
    pub branch: DungeonBranch,
    pub name: String,
    pub entry_level: i32,
    pub depth_range: (i32, i32),
    pub is_optional: bool,
    pub requires_item: Option<String>,
    pub special_rules: Vec<String>,
}

/// [v2.34.0 98-2] 분기 정보 조회
pub fn get_branch_info(branch: DungeonBranch) -> BranchInfo {
    match branch {
        DungeonBranch::Main => BranchInfo {
            branch,
            name: "메인 던전".to_string(),
            entry_level: 1,
            depth_range: (1, 30),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["상점 출현".to_string(), "오라클".to_string()],
        },
        DungeonBranch::GnomishMines => BranchInfo {
            branch,
            name: "그놈 광산".to_string(),
            entry_level: 3,
            depth_range: (3, 12),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["광산 마을".to_string(), "광산 끝".to_string()],
        },
        DungeonBranch::SokobanBranch => BranchInfo {
            branch,
            name: "소코반".to_string(),
            entry_level: 6,
            depth_range: (6, 10),
            is_optional: true,
            requires_item: None,
            special_rules: vec![
                "퍼즐".to_string(),
                "굴착 불가".to_string(),
                "텔레포트 불가".to_string(),
            ],
        },
        DungeonBranch::QuestBranch => BranchInfo {
            branch,
            name: "퀘스트".to_string(),
            entry_level: 14,
            depth_range: (14, 20),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["직업별 퀘스트".to_string(), "리더 대화".to_string()],
        },
        DungeonBranch::FortLudiosBranch => BranchInfo {
            branch,
            name: "루디오스 성채".to_string(),
            entry_level: 15,
            depth_range: (15, 15),
            is_optional: true,
            requires_item: Some("금화 (뇌물)".to_string()),
            special_rules: vec!["보물 가득".to_string()],
        },
        DungeonBranch::VladTowerBranch => BranchInfo {
            branch,
            name: "블라드의 탑".to_string(),
            entry_level: 25,
            depth_range: (25, 28),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["촛대 필요".to_string()],
        },
        DungeonBranch::WizardTowerBranch => BranchInfo {
            branch,
            name: "마법사의 탑".to_string(),
            entry_level: 28,
            depth_range: (28, 30),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["아뮬렛 획득".to_string(), "마법사 부활".to_string()],
        },
        DungeonBranch::Gehennom => BranchInfo {
            branch,
            name: "게헨놈".to_string(),
            entry_level: 20,
            depth_range: (20, 30),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["악마 출현".to_string(), "화염 지형".to_string()],
        },
        DungeonBranch::AstralPlaneBranch => BranchInfo {
            branch,
            name: "천상계".to_string(),
            entry_level: 31,
            depth_range: (31, 31),
            is_optional: false,
            requires_item: Some("아뮬렛 of Yendor".to_string()),
            special_rules: vec!["최종 전투".to_string(), "3개 제단".to_string()],
        },
        DungeonBranch::EarthPlane => BranchInfo {
            branch,
            name: "대지의 면".to_string(),
            entry_level: 30,
            depth_range: (30, 30),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["원소 생물".to_string()],
        },
        DungeonBranch::WaterPlane => BranchInfo {
            branch,
            name: "물의 면".to_string(),
            entry_level: 30,
            depth_range: (30, 30),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["수중 전투".to_string(), "호흡 필요".to_string()],
        },
        DungeonBranch::FirePlane => BranchInfo {
            branch,
            name: "불의 면".to_string(),
            entry_level: 30,
            depth_range: (30, 30),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["화염 저항 필수".to_string()],
        },
        DungeonBranch::AirPlane => BranchInfo {
            branch,
            name: "공기의 면".to_string(),
            entry_level: 30,
            depth_range: (30, 30),
            is_optional: false,
            requires_item: None,
            special_rules: vec!["비행 필요".to_string()],
        },
    }
}

// =============================================================================
// [2] 레벨 특성 판정 — level_features (dungeon.c L1000-1500)
// =============================================================================

/// [v2.34.0 98-2] 레벨 특성
#[derive(Debug, Clone)]
pub struct LevelFeatures {
    pub depth: i32,
    pub branch: DungeonBranch,
    pub has_shops: bool,
    pub has_altars: bool,
    pub has_fountains: bool,
    pub has_special_room: bool,
    pub monster_difficulty: i32,
    pub light_level: i32,
    pub ambient_message: Option<String>,
}

/// [v2.34.0 98-2] 레벨 특성 생성
pub fn generate_level_features(
    depth: i32,
    branch: DungeonBranch,
    rng: &mut NetHackRng,
) -> LevelFeatures {
    let has_shops = rng.rn2(10) < 3 && depth > 3;
    let has_altars = rng.rn2(8) < 2;
    let has_fountains = rng.rn2(6) < 2;
    let has_special = rng.rn2(5) < 2 && depth > 5;

    let difficulty = depth
        + match branch {
            DungeonBranch::Gehennom => 10,
            DungeonBranch::AstralPlaneBranch => 20,
            DungeonBranch::GnomishMines => -2,
            _ => 0,
        };

    let light = match branch {
        DungeonBranch::Gehennom => 1,
        DungeonBranch::GnomishMines => 3,
        _ if depth > 20 => 2,
        _ => 5,
    };

    let ambient = match branch {
        DungeonBranch::Gehennom => Some("불의 냄새가 풍긴다...".to_string()),
        DungeonBranch::GnomishMines => Some("곡괭이 소리가 들린다.".to_string()),
        DungeonBranch::WaterPlane => Some("모든 곳이 물이다!".to_string()),
        _ if depth > 25 => Some("위험한 기운이 감돈다.".to_string()),
        _ => None,
    };

    LevelFeatures {
        depth,
        branch,
        has_shops,
        has_altars,
        has_fountains,
        has_special_room: has_special,
        monster_difficulty: difficulty.max(1),
        light_level: light,
        ambient_message: ambient,
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_main_branch() {
        let info = get_branch_info(DungeonBranch::Main);
        assert_eq!(info.entry_level, 1);
        assert!(!info.is_optional);
    }

    #[test]
    fn test_sokoban_optional() {
        let info = get_branch_info(DungeonBranch::SokobanBranch);
        assert!(info.is_optional);
        assert!(info
            .special_rules
            .iter()
            .any(|r| r.contains("텔레포트 불가")));
    }

    #[test]
    fn test_astral_requires_amulet() {
        let info = get_branch_info(DungeonBranch::AstralPlaneBranch);
        assert!(info.requires_item.is_some());
    }

    #[test]
    fn test_gehennom_difficulty() {
        let mut rng = test_rng();
        let features = generate_level_features(25, DungeonBranch::Gehennom, &mut rng);
        assert!(features.monster_difficulty > 30);
    }

    #[test]
    fn test_mine_ambient() {
        let mut rng = test_rng();
        let features = generate_level_features(5, DungeonBranch::GnomishMines, &mut rng);
        assert!(features.ambient_message.is_some());
    }

    #[test]
    fn test_level_features_basic() {
        let mut rng = test_rng();
        let features = generate_level_features(10, DungeonBranch::Main, &mut rng);
        assert_eq!(features.depth, 10);
        assert!(features.monster_difficulty >= 10);
    }

    #[test]
    fn test_all_branches() {
        let branches = [
            DungeonBranch::Main,
            DungeonBranch::GnomishMines,
            DungeonBranch::SokobanBranch,
            DungeonBranch::Gehennom,
        ];
        for b in &branches {
            let info = get_branch_info(*b);
            assert!(!info.name.is_empty());
        }
    }
}
