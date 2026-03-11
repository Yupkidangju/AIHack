// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.24.0 R12-5] 게임 종료/스코어 (end_ext.rs)
//
// 원본 참조: NetHack 3.6.7 end.c (1,641줄)
//
// 구현 내용:
//   1. 사망 원인 분류
//   2. 스코어 계산
//   3. 하이스코어 테이블
//   4. 묘비 텍스트 생성
//   5. 게임 통계 집계
// ============================================================================

// =============================================================================
// [1] 사망 원인 (원본: end.c killer)
// =============================================================================

/// [v2.24.0 R12-5] 사망 원인 카테고리
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeathCause {
    /// 몬스터에 의한 사망
    KilledByMonster { monster_name: String },
    /// 함정
    KilledByTrap(String),
    /// 굶주림
    Starvation,
    /// 질병
    Disease,
    /// 석화
    Petrification,
    /// 익사
    Drowning,
    /// 용암
    LavaDeath,
    /// 독
    Poisoning,
    /// 낙하
    Falling,
    /// 자살 (quit)
    Quit,
    /// 탈출 (승리)
    Escaped,
    /// 승천 (완전 승리)
    Ascended,
    /// 기타
    Other(String),
}

/// [v2.24.0 R12-5] 사망 원인 텍스트 (원본: killer_format)
pub fn death_message(cause: &DeathCause) -> String {
    match cause {
        DeathCause::KilledByMonster { monster_name } => {
            format!("killed by {}", monster_name)
        }
        DeathCause::KilledByTrap(trap) => format!("killed by a {}", trap),
        DeathCause::Starvation => "died of starvation".to_string(),
        DeathCause::Disease => "died of disease".to_string(),
        DeathCause::Petrification => "turned to stone".to_string(),
        DeathCause::Drowning => "drowned".to_string(),
        DeathCause::LavaDeath => "burned in lava".to_string(),
        DeathCause::Poisoning => "died of poisoning".to_string(),
        DeathCause::Falling => "fell to death".to_string(),
        DeathCause::Quit => "quit".to_string(),
        DeathCause::Escaped => "escaped the dungeon".to_string(),
        DeathCause::Ascended => "ascended to demigodhood".to_string(),
        DeathCause::Other(msg) => msg.clone(),
    }
}

// =============================================================================
// [2] 스코어 계산 (원본: end.c done_in_by, calc_score)
// =============================================================================

/// [v2.24.0 R12-5] 스코어 계산 입력
#[derive(Debug, Clone)]
pub struct ScoreInput {
    /// 골드 보유량
    pub gold: i64,
    /// 던전 최대 깊이
    pub max_depth: i32,
    /// 처치한 몬스터 수
    pub kills: i32,
    /// 아티팩트 보유 수
    pub artifacts: i32,
    /// 승천 여부
    pub ascended: bool,
    /// 퀘스트 완료 여부
    pub quest_completed: bool,
    /// 플레이어 레벨
    pub player_level: i32,
    /// 총 턴 수
    pub turns: u64,
    /// 사망 원인
    pub cause: DeathCause,
}

/// [v2.24.0 R12-5] 스코어 계산 (원본: calc_score)
pub fn calc_score(input: &ScoreInput) -> i64 {
    let mut score: i64 = 0;

    // 골드
    score += input.gold;

    // 깊이 보너스
    score += (input.max_depth as i64) * 100;

    // 킬 보너스
    score += (input.kills as i64) * 10;

    // 아티팩트 보너스
    score += (input.artifacts as i64) * 2500;

    // 레벨 보너스
    score += (input.player_level as i64) * 500;

    // 퀘스트 보너스
    if input.quest_completed {
        score += 10000;
    }

    // 승천 보너스
    if input.ascended {
        score += 50000;
    }

    // 효율 보너스 (적은 턴으로 높은 깊이)
    if input.turns > 0 {
        let efficiency = (input.max_depth as i64 * 1000) / (input.turns as i64).max(1);
        score += efficiency;
    }

    score.max(0)
}

// =============================================================================
// [3] 하이스코어 테이블 (원본: end.c topten)
// =============================================================================

/// [v2.24.0 R12-5] 하이스코어 엔트리
#[derive(Debug, Clone)]
pub struct HighScoreEntry {
    /// 순위
    pub rank: u32,
    /// 플레이어 이름
    pub name: String,
    /// 역할
    pub role: String,
    /// 점수
    pub score: i64,
    /// 최대 깊이
    pub max_depth: i32,
    /// 사망 원인
    pub death_cause: String,
    /// 타임스탬프
    pub timestamp: u64,
}

/// [v2.24.0 R12-5] 하이스코어 테이블에 삽입 위치 결정
pub fn find_insertion_rank(table: &[HighScoreEntry], new_score: i64) -> Option<u32> {
    for (i, entry) in table.iter().enumerate() {
        if new_score > entry.score {
            return Some(i as u32 + 1);
        }
    }
    // 테이블 끝에 삽입
    if table.len() < 100 {
        Some(table.len() as u32 + 1)
    } else {
        None // 100위 밖
    }
}

// =============================================================================
// [4] 묘비 텍스트 (원본: end.c outrip)
// =============================================================================

/// [v2.24.0 R12-5] 묘비 텍스트 생성
pub fn generate_tombstone(
    name: &str,
    role: &str,
    score: i64,
    cause: &DeathCause,
    max_depth: i32,
) -> Vec<String> {
    let death_msg = death_message(cause);
    vec![
        "          ----------".to_string(),
        "         /          \\".to_string(),
        "        /    REST    \\".to_string(),
        "       /      IN      \\".to_string(),
        "      /     PEACE      \\".to_string(),
        "     /                  \\".to_string(),
        format!("    |  {:^18}  |", name),
        format!("    |  {:^18}  |", format!("the {}", role)),
        format!("    |  {:^18}  |", format!("Score: {}", score)),
        format!("    |  {:^18}  |", format!("Depth: {}", max_depth)),
        format!("    |  {:^18}  |", death_msg),
        "    |                    |".to_string(),
        "   *|     *  *  *       |*".to_string(),
        "  _________)/\\__/\\______/".to_string(),
    ]
}

// =============================================================================
// [5] 게임 통계 (원본: end.c)
// =============================================================================

/// [v2.24.0 R12-5] 게임 종료 시 통계
#[derive(Debug, Clone, Default)]
pub struct GameEndStats {
    /// 총 턴 수
    pub total_turns: u64,
    /// 몬스터 처치 수
    pub monsters_killed: i32,
    /// 사망 횟수 (변신 중 사망 포함)
    pub deaths: i32,
    /// 최대 깊이
    pub deepest_level: i32,
    /// 총 획득 골드
    pub total_gold_earned: i64,
    /// 먹은 음식 수
    pub food_eaten: i32,
    /// 사용한 포션 수
    pub potions_used: i32,
    /// 읽은 스크롤 수
    pub scrolls_read: i32,
}

/// [v2.24.0 R12-5] 통계 요약 텍스트
pub fn stats_summary(stats: &GameEndStats) -> Vec<String> {
    vec![
        format!("총 턴 수: {}", stats.total_turns),
        format!("몬스터 처치: {} 마리", stats.monsters_killed),
        format!("최대 깊이: {} 층", stats.deepest_level),
        format!("총 획득 금화: {} zm", stats.total_gold_earned),
        format!(
            "음식: {} 개, 포션: {} 개, 스크롤: {} 개",
            stats.food_eaten, stats.potions_used, stats.scrolls_read
        ),
    ]
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_score_input() -> ScoreInput {
        ScoreInput {
            gold: 10000,
            max_depth: 20,
            kills: 100,
            artifacts: 2,
            ascended: false,
            quest_completed: false,
            player_level: 15,
            turns: 5000,
            cause: DeathCause::KilledByMonster {
                monster_name: "dragon".to_string(),
            },
        }
    }

    #[test]
    fn test_death_message_monster() {
        let cause = DeathCause::KilledByMonster {
            monster_name: "green dragon".to_string(),
        };
        assert_eq!(death_message(&cause), "killed by green dragon");
    }

    #[test]
    fn test_death_message_ascended() {
        assert_eq!(
            death_message(&DeathCause::Ascended),
            "ascended to demigodhood"
        );
    }

    #[test]
    fn test_death_message_starvation() {
        assert_eq!(death_message(&DeathCause::Starvation), "died of starvation");
    }

    #[test]
    fn test_calc_score_basic() {
        let input = test_score_input();
        let score = calc_score(&input);
        // 10000 + 20*100 + 100*10 + 2*2500 + 15*500 + efficiency
        assert!(score > 25000);
    }

    #[test]
    fn test_calc_score_ascended() {
        let mut input = test_score_input();
        input.ascended = true;
        input.quest_completed = true;
        let score = calc_score(&input);
        assert!(score > 85000); // +50000+10000
    }

    #[test]
    fn test_find_rank_top() {
        let table = vec![
            HighScoreEntry {
                rank: 1,
                name: "A".to_string(),
                role: "V".to_string(),
                score: 50000,
                max_depth: 30,
                death_cause: "d".to_string(),
                timestamp: 0,
            },
            HighScoreEntry {
                rank: 2,
                name: "B".to_string(),
                role: "W".to_string(),
                score: 30000,
                max_depth: 20,
                death_cause: "d".to_string(),
                timestamp: 0,
            },
        ];
        assert_eq!(find_insertion_rank(&table, 60000), Some(1));
    }

    #[test]
    fn test_find_rank_middle() {
        let table = vec![
            HighScoreEntry {
                rank: 1,
                name: "A".to_string(),
                role: "V".to_string(),
                score: 50000,
                max_depth: 30,
                death_cause: "d".to_string(),
                timestamp: 0,
            },
            HighScoreEntry {
                rank: 2,
                name: "B".to_string(),
                role: "W".to_string(),
                score: 30000,
                max_depth: 20,
                death_cause: "d".to_string(),
                timestamp: 0,
            },
        ];
        assert_eq!(find_insertion_rank(&table, 40000), Some(2));
    }

    #[test]
    fn test_tombstone_lines() {
        let cause = DeathCause::KilledByMonster {
            monster_name: "Medusa".to_string(),
        };
        let lines = generate_tombstone("Player", "Valkyrie", 50000, &cause, 25);
        assert!(lines.len() >= 10);
        assert!(lines.iter().any(|l| l.contains("Player")));
    }

    #[test]
    fn test_game_stats() {
        let stats = GameEndStats {
            total_turns: 10000,
            monsters_killed: 200,
            deaths: 3,
            deepest_level: 30,
            total_gold_earned: 50000,
            food_eaten: 50,
            potions_used: 20,
            scrolls_read: 15,
        };
        let summary = stats_summary(&stats);
        assert!(summary.len() >= 4);
        assert!(summary[0].contains("10000"));
    }
}
