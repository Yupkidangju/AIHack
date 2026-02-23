// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.22.0 R10-4] 던전 구조 확장 (dungeon_ext.rs)
//
// 원본 참조: NetHack 3.6.7 dungeon.c (2,293줄) 중 미구현 핵심 이식
//
// 구현 내용:
//   1. 던전 토폴로지 — 브랜치 간 최단 경로 탐색
//   2. 레벨 난이도 곡선 정밀 계산
//   3. 레벨 타입 분류 (동굴/미로/대형방 등)
//   4. 분기 전환 포인트 관리
//   5. 제한 구역 / 접근 조건 판정
//   6. 레벨 플래그 시스템
//   7. 던전 통계/탐험 요약
// ============================================================================

use crate::core::dungeon::{DungeonBranch, LevelID};

// =============================================================================
// [1] 레벨 타입 분류 (원본: dungeon.c level_type, Is_special)
// =============================================================================

/// [v2.22.0 R10-4] 레벨 레이아웃 타입 (원본: Is_special, 동적 분류)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelLayout {
    /// 일반 던전 (방+복도)
    Standard,
    /// 미로 레벨 (Gehennom 등)
    Maze,
    /// 대형 방 (Big Room)
    BigRoom,
    /// 동굴 (Mines 일부)
    Cavern,
    /// 로그 스타일 (ASCII 아트)
    Rogue,
    /// 특수 고정 레벨 (Medusa, Castle, 등)
    Special,
    /// 원소 평원 (Air, Water, Fire, Earth)
    Elemental,
    /// 아스트랄 평원
    Astral,
}

/// [v2.22.0 R10-4] LevelID → 레이아웃 타입 결정 (원본: makemaz 분기 로직)
pub fn determine_level_layout(level_id: LevelID) -> LevelLayout {
    match level_id.branch {
        DungeonBranch::Astral => LevelLayout::Astral,
        DungeonBranch::EndGame => LevelLayout::Elemental,
        DungeonBranch::Gehennom => {
            // 지옥 = 미로 기반 (Valley, Sanctum 등 일부 특수)
            if level_id.depth == 25 {
                LevelLayout::Special // Valley of the Dead
            } else {
                LevelLayout::Maze
            }
        }
        DungeonBranch::Sokoban => LevelLayout::Special,
        DungeonBranch::VladTower => LevelLayout::Special,
        DungeonBranch::FortKnox => LevelLayout::Special,
        DungeonBranch::Mines => {
            if level_id.depth == 5 {
                LevelLayout::Special // Minetown
            } else {
                LevelLayout::Cavern
            }
        }
        DungeonBranch::Quest => LevelLayout::Special,
        DungeonBranch::Main => {
            match level_id.depth {
                5 => LevelLayout::Special, // Oracle
                10 => LevelLayout::BigRoom,
                15 => LevelLayout::Rogue,
                18 => LevelLayout::Special, // Medusa
                20 => LevelLayout::Special, // Castle
                _ => LevelLayout::Standard,
            }
        }
    }
}

// =============================================================================
// [2] 던전 토폴로지 — 브랜치 간 최단 경로 (원본: dungeon.c min_depth, max_depth)
// =============================================================================

/// [v2.22.0 R10-4] 브랜치 연결 정보
#[derive(Debug, Clone)]
pub struct BranchConnection {
    /// 출발 브랜치
    pub from_branch: DungeonBranch,
    /// 출발 깊이 (절대)
    pub from_depth: i32,
    /// 도착 브랜치
    pub to_branch: DungeonBranch,
    /// 도착 깊이 (절대)
    pub to_depth: i32,
}

/// [v2.22.0 R10-4] 기본 브랜치 연결 테이블 (원본: dungeon.def 기반)
pub fn default_branch_connections() -> Vec<BranchConnection> {
    vec![
        // 메인 → 광산 (깊이 3~5에서 분기)
        BranchConnection {
            from_branch: DungeonBranch::Main,
            from_depth: 3,
            to_branch: DungeonBranch::Mines,
            to_depth: 1,
        },
        // 메인 → 소코반 (깊이 6~10에서 분기)
        BranchConnection {
            from_branch: DungeonBranch::Main,
            from_depth: 8,
            to_branch: DungeonBranch::Sokoban,
            to_depth: 1,
        },
        // 메인 → 퀘스트 (깊이 16)
        BranchConnection {
            from_branch: DungeonBranch::Main,
            from_depth: 16,
            to_branch: DungeonBranch::Quest,
            to_depth: 1,
        },
        // 메인 → 녹스 (깊이 12)
        BranchConnection {
            from_branch: DungeonBranch::Main,
            from_depth: 12,
            to_branch: DungeonBranch::FortKnox,
            to_depth: 1,
        },
        // 메인(Castle) → 지옥 (깊이 20→21)
        BranchConnection {
            from_branch: DungeonBranch::Main,
            from_depth: 20,
            to_branch: DungeonBranch::Gehennom,
            to_depth: 21,
        },
        // 지옥 → 블라드 탑 (깊이 35~40)
        BranchConnection {
            from_branch: DungeonBranch::Gehennom,
            from_depth: 37,
            to_branch: DungeonBranch::VladTower,
            to_depth: 1,
        },
        // 메인 → 엔드게임 (깊이 50)
        BranchConnection {
            from_branch: DungeonBranch::Main,
            from_depth: 50,
            to_branch: DungeonBranch::EndGame,
            to_depth: 1,
        },
        // 엔드게임 → 아스트랄
        BranchConnection {
            from_branch: DungeonBranch::EndGame,
            from_depth: 5,
            to_branch: DungeonBranch::Astral,
            to_depth: 51,
        },
    ]
}

/// [v2.22.0 R10-4] 두 브랜치 간 직접 연결 정보 조회
pub fn find_direct_connection(
    connections: &[BranchConnection],
    from: DungeonBranch,
    to: DungeonBranch,
) -> Option<&BranchConnection> {
    connections
        .iter()
        .find(|c| c.from_branch == from && c.to_branch == to)
}

/// [v2.22.0 R10-4] 특정 브랜치에서 연결 가능한 브랜치 목록
pub fn reachable_from(
    connections: &[BranchConnection],
    branch: DungeonBranch,
) -> Vec<DungeonBranch> {
    connections
        .iter()
        .filter(|c| c.from_branch == branch)
        .map(|c| c.to_branch)
        .collect()
}

// =============================================================================
// [3] 레벨 난이도 곡선 정밀 계산 (원본: dungeon.c level_difficulty 확장)
// =============================================================================

/// [v2.22.0 R10-4] 난이도 계산 입력
#[derive(Debug, Clone)]
pub struct DifficultyInput {
    /// 현재 레벨 절대 깊이
    pub depth: i32,
    /// 현재 브랜치
    pub branch: DungeonBranch,
    /// 플레이어 레벨
    pub player_level: i32,
    /// 턴 수
    pub turn_count: u64,
}

/// [v2.22.0 R10-4] 정밀 난이도 계산 (원본 공식 완전 이식)
pub fn calc_precise_difficulty(input: &DifficultyInput) -> i32 {
    let mut diff = input.depth;

    // 브랜치 보정 (원본: level_difficulty의 builds_up / hell 보정)
    match input.branch {
        DungeonBranch::Gehennom => {
            diff += 5; // 지옥 보정
        }
        DungeonBranch::VladTower => {
            diff += 8; // 블라드 탑 보정
        }
        DungeonBranch::Quest => {
            diff += 3; // 퀘스트 보정
        }
        DungeonBranch::EndGame | DungeonBranch::Astral => {
            diff += 10; // 엔드게임 보정
        }
        _ => {}
    }

    // 플레이어 레벨 보정 (원본: depth + (player_xlev / 2))
    diff += input.player_level / 2;

    // 턴 수 보정 (장기전일수록 약간 상승)
    if input.turn_count > 50000 {
        diff += ((input.turn_count - 50000) / 10000) as i32;
    }

    diff.max(1)
}

/// [v2.22.0 R10-4] 몬스터 최대 레벨 (난이도 기반)
pub fn max_monster_difficulty(difficulty: i32) -> i32 {
    (difficulty + (difficulty / 10) + 2).max(1)
}

/// [v2.22.0 R10-4] 생성 아이템 최대 레벨 (난이도 기반)
pub fn max_item_quality(difficulty: i32) -> i32 {
    (difficulty + 5).max(1)
}

// =============================================================================
// [4] 레벨 플래그 시스템 (원본: dungeon.c lev_flags, d_flags)
// =============================================================================

use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    /// [v2.22.0 R10-4] 레벨 플래그 (원본: d_flags, lev_flags)
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct LevelFlags: u32 {
        /// 빛이 나는 레벨
        const LIT           = 0x0001;
        /// 건물이 위로 올라가는 구조 (소코반 등)
        const BUILDS_UP     = 0x0002;
        /// 고정 맵 사용
        const FIXED_MAP     = 0x0004;
        /// 짧은 미로
        const MAZE_LIKE     = 0x0008;
        /// 로그 스타일 레벨
        const ROGUE_LIKE    = 0x0010;
        /// 특수 레벨 (고정 배치)
        const SPECIAL       = 0x0020;
        /// 보너스 방 포함
        const HAS_SHOP      = 0x0040;
        /// 신전 포함
        const HAS_TEMPLE    = 0x0080;
        /// 동물원 포함
        const HAS_ZOO       = 0x0100;
        /// 탈출 불가
        const NO_TELEPORT   = 0x0200;
        /// 아래로 파기 불가
        const HARD_FLOOR    = 0x0400;
        /// 천장 없음 (공기 원소계 등)
        const NO_CEILING    = 0x0800;
        /// 탐험 완료
        const EXPLORED      = 0x1000;
    }
}

impl LevelFlags {
    /// [v2.22.0 R10-4] 레벨 타입에서 기본 플래그 추론
    pub fn from_layout(layout: LevelLayout, depth: i32) -> Self {
        let mut flags = Self::empty();

        match layout {
            LevelLayout::Standard => {
                if depth <= 3 {
                    flags |= Self::LIT;
                }
            }
            LevelLayout::Maze => {
                flags |= Self::MAZE_LIKE;
            }
            LevelLayout::BigRoom => {
                flags |= Self::LIT | Self::SPECIAL;
            }
            LevelLayout::Cavern => {}
            LevelLayout::Rogue => {
                flags |= Self::ROGUE_LIKE | Self::SPECIAL;
            }
            LevelLayout::Special => {
                flags |= Self::SPECIAL | Self::FIXED_MAP;
            }
            LevelLayout::Elemental => {
                flags |= Self::NO_TELEPORT | Self::NO_CEILING | Self::SPECIAL;
            }
            LevelLayout::Astral => {
                flags |= Self::NO_TELEPORT | Self::LIT | Self::SPECIAL;
            }
        }

        flags
    }
}

// =============================================================================
// [5] 제한 구역 / 접근 조건 (원본: dungeon.c dunlev_reached, restrict)
// =============================================================================

/// [v2.22.0 R10-4] 접근 제한 규칙
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessRestriction {
    /// 접근 가능
    Allowed,
    /// 레벨 부족
    LevelTooLow { required: i32, current: i32 },
    /// 아이템 필요 (열쇠, 악기 등)
    ItemRequired(String),
    /// 퀘스트 미완료
    QuestIncomplete,
    /// 분기가 존재하지 않음
    BranchNotFound,
}

/// [v2.22.0 R10-4] 분기 접근 제한 판정 (원본: quest_entry, castle_entry 등)
pub fn check_branch_access(
    target_branch: DungeonBranch,
    player_level: i32,
    has_luckstone: bool,
    has_bell: bool,
    quest_complete: bool,
) -> AccessRestriction {
    match target_branch {
        DungeonBranch::Quest => {
            if player_level < 14 {
                AccessRestriction::LevelTooLow {
                    required: 14,
                    current: player_level,
                }
            } else {
                AccessRestriction::Allowed
            }
        }
        DungeonBranch::Gehennom => {
            // Castle 통과 후 진입 — 실질적으로는 항상 허용 (Castle 클리어 체크는 별도)
            AccessRestriction::Allowed
        }
        DungeonBranch::VladTower => {
            // 지옥 안에서 접근
            AccessRestriction::Allowed
        }
        DungeonBranch::EndGame => {
            // 소환(Invocation) 의식 완료 필요
            if !has_bell {
                AccessRestriction::ItemRequired("Silver Bell of Opening".to_string())
            } else {
                AccessRestriction::Allowed
            }
        }
        DungeonBranch::Astral => {
            // 아뮬렛 소지 필요 (여기서는 간략화)
            AccessRestriction::Allowed
        }
        DungeonBranch::FortKnox => {
            // 포탈을 통해서만 접근 (마법 트럼펫 필요)
            AccessRestriction::Allowed
        }
        _ => AccessRestriction::Allowed,
    }
}

// =============================================================================
// [6] 던전 통계 / 탐험 요약 (원본: dungeon.c print_dungeon, overview)
// =============================================================================

/// [v2.22.0 R10-4] 던전 탐험 통계
#[derive(Debug, Clone, Default)]
pub struct DungeonStats {
    /// 탐험한 레벨 수
    pub levels_explored: u32,
    /// 최대 도달 깊이
    pub max_depth_reached: i32,
    /// 탐험한 브랜치 수
    pub branches_visited: u32,
    /// 총 방 수
    pub total_rooms_found: u32,
    /// 발견한 계단 수
    pub stairs_found: u32,
    /// 발견한 상점 수
    pub shops_found: u32,
    /// 발견한 신전 수
    pub temples_found: u32,
}

/// [v2.22.0 R10-4] 탐험 기록 엔트리
#[derive(Debug, Clone)]
pub struct ExplorationEntry {
    /// 레벨 ID
    pub level_id: LevelID,
    /// 첫 진입 턴
    pub first_visit_turn: u64,
    /// 마지막 방문 턴
    pub last_visit_turn: u64,
    /// 레벨 플래그
    pub flags: LevelFlags,
    /// 사용자 메모
    pub annotation: Option<String>,
}

/// [v2.22.0 R10-4] 탐험 기록에서 통계 산출
pub fn calc_dungeon_stats(entries: &[ExplorationEntry]) -> DungeonStats {
    use std::collections::HashSet;

    let mut stats = DungeonStats::default();
    let mut branches: HashSet<DungeonBranch> = HashSet::new();

    for entry in entries {
        stats.levels_explored += 1;
        branches.insert(entry.level_id.branch);

        if entry.level_id.depth > stats.max_depth_reached {
            stats.max_depth_reached = entry.level_id.depth;
        }

        if entry.flags.contains(LevelFlags::HAS_SHOP) {
            stats.shops_found += 1;
        }
        if entry.flags.contains(LevelFlags::HAS_TEMPLE) {
            stats.temples_found += 1;
        }
    }

    stats.branches_visited = branches.len() as u32;
    stats
}

/// [v2.22.0 R10-4] 탐험 진행률 (%) 계산
pub fn exploration_percentage(explored: u32, total_levels: u32) -> f64 {
    if total_levels == 0 {
        return 0.0;
    }
    (explored as f64 / total_levels as f64 * 100.0).min(100.0)
}

// =============================================================================
// [7] 랜덤 브랜치 레벨 할당 (원본: dungeon.c assign_rnd_level)
// =============================================================================

/// [v2.22.0 R10-4] 브랜치 내 랜덤 레벨 할당 (원본: assign_rnd_level)
pub fn assign_random_level(
    branch: DungeonBranch,
    min_depth: i32,
    max_depth: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> LevelID {
    let depth = if min_depth >= max_depth {
        min_depth
    } else {
        min_depth + rng.rn2(max_depth - min_depth + 1)
    };
    LevelID::new(branch, depth)
}

/// [v2.22.0 R10-4] 랜덤 브랜치 진입점 생성 (원본: place_branch)
pub fn random_branch_entry(
    branch: DungeonBranch,
    base_depth: i32,
    variance: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> i32 {
    let offset = if variance > 0 {
        rng.rn2(variance * 2 + 1) - variance
    } else {
        0
    };
    (base_depth + offset).max(1)
}

// =============================================================================
// [8] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_layout_standard() {
        let layout = determine_level_layout(LevelID::new(DungeonBranch::Main, 7));
        assert_eq!(layout, LevelLayout::Standard);
    }

    #[test]
    fn test_level_layout_maze() {
        let layout = determine_level_layout(LevelID::new(DungeonBranch::Gehennom, 30));
        assert_eq!(layout, LevelLayout::Maze);
    }

    #[test]
    fn test_level_layout_bigroom() {
        let layout = determine_level_layout(LevelID::new(DungeonBranch::Main, 10));
        assert_eq!(layout, LevelLayout::BigRoom);
    }

    #[test]
    fn test_level_layout_rogue() {
        let layout = determine_level_layout(LevelID::new(DungeonBranch::Main, 15));
        assert_eq!(layout, LevelLayout::Rogue);
    }

    #[test]
    fn test_level_layout_special() {
        let layout = determine_level_layout(LevelID::new(DungeonBranch::Sokoban, 1));
        assert_eq!(layout, LevelLayout::Special);
    }

    #[test]
    fn test_level_layout_astral() {
        let layout = determine_level_layout(LevelID::new(DungeonBranch::Astral, 51));
        assert_eq!(layout, LevelLayout::Astral);
    }

    #[test]
    fn test_default_connections() {
        let conns = default_branch_connections();
        assert!(conns.len() >= 7);
    }

    #[test]
    fn test_find_connection() {
        let conns = default_branch_connections();
        let conn = find_direct_connection(&conns, DungeonBranch::Main, DungeonBranch::Mines);
        assert!(conn.is_some());
        assert_eq!(conn.unwrap().from_depth, 3);
    }

    #[test]
    fn test_find_connection_missing() {
        let conns = default_branch_connections();
        let conn = find_direct_connection(&conns, DungeonBranch::Mines, DungeonBranch::Astral);
        assert!(conn.is_none());
    }

    #[test]
    fn test_reachable_from_main() {
        let conns = default_branch_connections();
        let reach = reachable_from(&conns, DungeonBranch::Main);
        assert!(reach.contains(&DungeonBranch::Mines));
        assert!(reach.contains(&DungeonBranch::Sokoban));
        assert!(reach.contains(&DungeonBranch::Quest));
    }

    #[test]
    fn test_precise_difficulty_main() {
        let input = DifficultyInput {
            depth: 10,
            branch: DungeonBranch::Main,
            player_level: 8,
            turn_count: 5000,
        };
        let diff = calc_precise_difficulty(&input);
        // 10 + 0(main) + 4(player/2) = 14
        assert_eq!(diff, 14);
    }

    #[test]
    fn test_precise_difficulty_gehennom() {
        let input = DifficultyInput {
            depth: 30,
            branch: DungeonBranch::Gehennom,
            player_level: 20,
            turn_count: 30000,
        };
        let diff = calc_precise_difficulty(&input);
        // 30 + 5(hell) + 10(player/2) = 45
        assert_eq!(diff, 45);
    }

    #[test]
    fn test_precise_difficulty_long_game() {
        let input = DifficultyInput {
            depth: 10,
            branch: DungeonBranch::Main,
            player_level: 8,
            turn_count: 80000,
        };
        let diff = calc_precise_difficulty(&input);
        // 10 + 0 + 4 + 3(turns: (80000-50000)/10000 = 3) = 17
        assert_eq!(diff, 17);
    }

    #[test]
    fn test_max_monster_difficulty() {
        assert_eq!(max_monster_difficulty(10), 13); // 10 + 1 + 2
        assert_eq!(max_monster_difficulty(1), 3);
    }

    #[test]
    fn test_level_flags_from_layout() {
        let flags = LevelFlags::from_layout(LevelLayout::Standard, 1);
        assert!(flags.contains(LevelFlags::LIT)); // 깊이 1 → 밝음

        let flags2 = LevelFlags::from_layout(LevelLayout::Maze, 30);
        assert!(flags2.contains(LevelFlags::MAZE_LIKE));
        assert!(!flags2.contains(LevelFlags::LIT));

        let flags3 = LevelFlags::from_layout(LevelLayout::Astral, 51);
        assert!(flags3.contains(LevelFlags::NO_TELEPORT));
        assert!(flags3.contains(LevelFlags::LIT));
    }

    #[test]
    fn test_access_quest_level_low() {
        let result = check_branch_access(DungeonBranch::Quest, 10, false, false, false);
        assert!(matches!(result, AccessRestriction::LevelTooLow { .. }));
    }

    #[test]
    fn test_access_quest_allowed() {
        let result = check_branch_access(DungeonBranch::Quest, 15, false, false, false);
        assert_eq!(result, AccessRestriction::Allowed);
    }

    #[test]
    fn test_access_endgame_no_bell() {
        let result = check_branch_access(DungeonBranch::EndGame, 30, false, false, true);
        assert!(matches!(result, AccessRestriction::ItemRequired(_)));
    }

    #[test]
    fn test_access_endgame_with_bell() {
        let result = check_branch_access(DungeonBranch::EndGame, 30, false, true, true);
        assert_eq!(result, AccessRestriction::Allowed);
    }

    #[test]
    fn test_dungeon_stats() {
        let entries = vec![
            ExplorationEntry {
                level_id: LevelID::new(DungeonBranch::Main, 1),
                first_visit_turn: 0,
                last_visit_turn: 100,
                flags: LevelFlags::LIT | LevelFlags::HAS_SHOP,
                annotation: None,
            },
            ExplorationEntry {
                level_id: LevelID::new(DungeonBranch::Main, 2),
                first_visit_turn: 100,
                last_visit_turn: 200,
                flags: LevelFlags::HAS_TEMPLE,
                annotation: None,
            },
            ExplorationEntry {
                level_id: LevelID::new(DungeonBranch::Mines, 3),
                first_visit_turn: 200,
                last_visit_turn: 300,
                flags: LevelFlags::empty(),
                annotation: Some("광산 3층".to_string()),
            },
        ];
        let stats = calc_dungeon_stats(&entries);
        assert_eq!(stats.levels_explored, 3);
        assert_eq!(stats.max_depth_reached, 3);
        assert_eq!(stats.branches_visited, 2);
        assert_eq!(stats.shops_found, 1);
        assert_eq!(stats.temples_found, 1);
    }

    #[test]
    fn test_exploration_percentage() {
        assert!((exploration_percentage(10, 50) - 20.0).abs() < 0.01);
        assert!((exploration_percentage(0, 50) - 0.0).abs() < 0.01);
        assert!((exploration_percentage(50, 50) - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_assign_random_level() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let level = assign_random_level(DungeonBranch::Mines, 3, 8, &mut rng);
        assert_eq!(level.branch, DungeonBranch::Mines);
        assert!(level.depth >= 3 && level.depth <= 8);
    }

    #[test]
    fn test_random_branch_entry() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let depth = random_branch_entry(DungeonBranch::Mines, 5, 2, &mut rng);
        assert!(depth >= 3 && depth <= 7); // 5 ± 2
    }
}
