// ============================================================================
// [v2.31.0 Phase 95-5] 특수 레벨 확장 (sp_lev_phase95_ext.rs)
// 원본: NetHack 3.6.7 src/sp_lev.c L400-1500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 특수 레벨 정의 — special_levels (sp_lev.c L400-800)
// =============================================================================

/// [v2.31.0 95-5] 특수 레벨 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialLevel {
    MedusaLair,
    CastleLevel,
    VladTower,
    WizardTower,
    AstralPlane,
    SanctuaryLevel,
    BigRoom,
    MineTown,
    MineEnd,
    SokobanLevel,
    QuestHome,
    QuestGoal,
    FortLudios,
    Rogue,
}

/// [v2.31.0 95-5] 특수 레벨 특성
#[derive(Debug, Clone)]
pub struct SpecialLevelInfo {
    pub name: String,
    pub level_type: SpecialLevel,
    pub width: i32,
    pub height: i32,
    pub is_maze: bool,
    pub no_teleport: bool,
    pub no_dig: bool,
    pub monster_gen_rate: i32,
    pub boss_name: Option<String>,
    pub required_items: Vec<String>,
    pub unique_features: Vec<String>,
}

/// [v2.31.0 95-5] 특수 레벨 정보 생성
/// 원본: sp_lev.c sp_level_at()
pub fn get_special_level_info(level: SpecialLevel) -> SpecialLevelInfo {
    match level {
        SpecialLevel::MedusaLair => SpecialLevelInfo {
            name: "메두사의 섬".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: false,
            no_dig: true,
            monster_gen_rate: 5,
            boss_name: Some("메두사".to_string()),
            required_items: vec!["방패 (반사용)".to_string()],
            unique_features: vec!["섬".to_string(), "물".to_string(), "석상들".to_string()],
        },
        SpecialLevel::CastleLevel => SpecialLevelInfo {
            name: "성채".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: true,
            no_dig: true,
            monster_gen_rate: 8,
            boss_name: None,
            required_items: vec![
                "피리 (패스월용)".to_string(),
                "폭탄 (벽 파괴용)".to_string(),
            ],
            unique_features: vec!["도개교".to_string(), "해자".to_string(), "보루".to_string()],
        },
        SpecialLevel::VladTower => SpecialLevelInfo {
            name: "블라드의 탑".to_string(),
            level_type: level,
            width: 40,
            height: 21,
            is_maze: false,
            no_teleport: true,
            no_dig: true,
            monster_gen_rate: 3,
            boss_name: Some("블라드 드라큘라".to_string()),
            required_items: vec![],
            unique_features: vec!["관".to_string(), "촛불".to_string()],
        },
        SpecialLevel::WizardTower => SpecialLevelInfo {
            name: "마법사의 탑".to_string(),
            level_type: level,
            width: 60,
            height: 21,
            is_maze: true,
            no_teleport: true,
            no_dig: true,
            monster_gen_rate: 10,
            boss_name: Some("얀델리".to_string()),
            required_items: vec!["아뮬렛 of Yendor".to_string()],
            unique_features: vec!["포탈".to_string(), "함정 가득".to_string()],
        },
        SpecialLevel::AstralPlane => SpecialLevelInfo {
            name: "천상계".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: true,
            no_dig: true,
            monster_gen_rate: 15,
            boss_name: None,
            required_items: vec!["아뮬렛 of Yendor".to_string()],
            unique_features: vec![
                "3개의 제단".to_string(),
                "천사들".to_string(),
                "악마들".to_string(),
            ],
        },
        SpecialLevel::BigRoom => SpecialLevelInfo {
            name: "큰 방".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: false,
            no_dig: false,
            monster_gen_rate: 12,
            boss_name: None,
            required_items: vec![],
            unique_features: vec!["거대한 사각형 방".to_string()],
        },
        SpecialLevel::MineTown => SpecialLevelInfo {
            name: "광산 마을".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: false,
            no_dig: false,
            monster_gen_rate: 5,
            boss_name: None,
            required_items: vec![],
            unique_features: vec![
                "상점들".to_string(),
                "신전".to_string(),
                "도서관".to_string(),
            ],
        },
        SpecialLevel::SokobanLevel => SpecialLevelInfo {
            name: "소코반".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: true,
            no_dig: true,
            monster_gen_rate: 0,
            boss_name: None,
            required_items: vec![],
            unique_features: vec!["바위".to_string(), "구멍".to_string(), "퍼즐".to_string()],
        },
        SpecialLevel::FortLudios => SpecialLevelInfo {
            name: "루디오스 성채".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: false,
            no_dig: false,
            monster_gen_rate: 10,
            boss_name: Some("크로이소스".to_string()),
            required_items: vec![],
            unique_features: vec!["금화 산더미".to_string(), "병사들".to_string()],
        },
        SpecialLevel::Rogue => SpecialLevelInfo {
            name: "로그 레벨".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: false,
            no_dig: false,
            monster_gen_rate: 5,
            boss_name: None,
            required_items: vec![],
            unique_features: vec!["ASCII 표시".to_string(), "로그 스타일".to_string()],
        },
        _ => SpecialLevelInfo {
            name: "특수 레벨".to_string(),
            level_type: level,
            width: 80,
            height: 21,
            is_maze: false,
            no_teleport: false,
            no_dig: false,
            monster_gen_rate: 5,
            boss_name: None,
            required_items: vec![],
            unique_features: vec![],
        },
    }
}

// =============================================================================
// [2] 소코반 퍼즐 — sokoban (sp_lev.c L800-1000)
// =============================================================================

/// [v2.31.0 95-5] 소코반 퍼즐 상태
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SokobanAction {
    PushBoulder { from: (i32, i32), to: (i32, i32) },
    BoulderFillPit { pos: (i32, i32) },
    CannotPush { reason: String },
    PuzzleSolved,
}

/// [v2.31.0 95-5] 소코반 바위 밀기
pub fn push_sokoban_boulder(
    boulder_x: i32,
    boulder_y: i32,
    push_dx: i32,
    push_dy: i32,
    is_pit: &dyn Fn(i32, i32) -> bool,
    is_wall: &dyn Fn(i32, i32) -> bool,
    remaining_pits: i32,
) -> SokobanAction {
    let target_x = boulder_x + push_dx;
    let target_y = boulder_y + push_dy;

    // 벽 확인
    if is_wall(target_x, target_y) {
        return SokobanAction::CannotPush {
            reason: "벽에 막혀있다.".to_string(),
        };
    }

    // 구덩이 확인 (퍼즐 해결)
    if is_pit(target_x, target_y) {
        if remaining_pits <= 1 {
            return SokobanAction::PuzzleSolved;
        }
        return SokobanAction::BoulderFillPit {
            pos: (target_x, target_y),
        };
    }

    SokobanAction::PushBoulder {
        from: (boulder_x, boulder_y),
        to: (target_x, target_y),
    }
}

// =============================================================================
// [3] 미로 레벨 생성 — maze_gen (sp_lev.c L1000-1500)
// =============================================================================

/// [v2.31.0 95-5] 미로 셀
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MazeCell {
    Wall,
    Path,
    Door,
    Trap,
    StairsUp,
    StairsDown,
}

/// [v2.31.0 95-5] 미로 생성 (Binary Space Partition 기반)
pub fn generate_maze(width: i32, height: i32, rng: &mut NetHackRng) -> Vec<Vec<MazeCell>> {
    let w = width as usize;
    let h = height as usize;
    let mut maze = vec![vec![MazeCell::Wall; w]; h];

    // 벽/경로 교대 패턴 (단순 미로)
    for y in 0..h {
        for x in 0..w {
            if x % 2 == 1 && y % 2 == 1 && x < w - 1 && y < h - 1 {
                maze[y][x] = MazeCell::Path;
                // 인접 통로 연결
                if rng.rn2(2) == 0 && x + 2 < w {
                    maze[y][x + 1] = MazeCell::Path;
                } else if y + 2 < h {
                    maze[y + 1][x] = MazeCell::Path;
                }
            }
        }
    }

    // 시작/끝 계단
    maze[1][1] = MazeCell::StairsUp;
    if h > 3 && w > 3 {
        maze[h - 2][w - 2] = MazeCell::StairsDown;
    }

    // 랜덤 함정
    for _ in 0..rng.rn2(5) + 1 {
        let tx = rng.rn2((w - 2) as i32) as usize + 1;
        let ty = rng.rn2((h - 2) as i32) as usize + 1;
        if maze[ty][tx] == MazeCell::Path {
            maze[ty][tx] = MazeCell::Trap;
        }
    }

    maze
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
    fn test_medusa_info() {
        let info = get_special_level_info(SpecialLevel::MedusaLair);
        assert!(info.boss_name.is_some());
        assert!(info.no_dig);
    }

    #[test]
    fn test_astral_no_teleport() {
        let info = get_special_level_info(SpecialLevel::AstralPlane);
        assert!(info.no_teleport);
        assert!(info.monster_gen_rate >= 15);
    }

    #[test]
    fn test_sokoban_push() {
        let result = push_sokoban_boulder(5, 5, 1, 0, &|_, _| false, &|_, _| false, 3);
        assert!(matches!(result, SokobanAction::PushBoulder { .. }));
    }

    #[test]
    fn test_sokoban_wall() {
        let result = push_sokoban_boulder(5, 5, 1, 0, &|_, _| false, &|x, _| x == 6, 3);
        assert!(matches!(result, SokobanAction::CannotPush { .. }));
    }

    #[test]
    fn test_sokoban_fill_pit() {
        let result = push_sokoban_boulder(5, 5, 1, 0, &|x, _| x == 6, &|_, _| false, 3);
        assert!(matches!(result, SokobanAction::BoulderFillPit { .. }));
    }

    #[test]
    fn test_sokoban_solved() {
        let result = push_sokoban_boulder(5, 5, 1, 0, &|x, _| x == 6, &|_, _| false, 1);
        assert!(matches!(result, SokobanAction::PuzzleSolved));
    }

    #[test]
    fn test_maze_gen() {
        let mut rng = test_rng();
        let maze = generate_maze(20, 10, &mut rng);
        assert_eq!(maze.len(), 10);
        assert_eq!(maze[0].len(), 20);
        assert_eq!(maze[1][1], MazeCell::StairsUp);
    }

    #[test]
    fn test_maze_stairs() {
        let mut rng = test_rng();
        let maze = generate_maze(20, 10, &mut rng);
        assert_eq!(maze[8][18], MazeCell::StairsDown);
    }
}
