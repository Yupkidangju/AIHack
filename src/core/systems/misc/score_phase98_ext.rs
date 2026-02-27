// ============================================================================
// [v2.34.0 Phase 98-5] 점수/통계 확장 (score_phase98_ext.rs)
// 원본: NetHack 3.6.7 src/topten.c + end.c 점수/통계 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 점수 계산 — score_calc (topten.c 핵심)
// =============================================================================

/// [v2.34.0 98-5] 게임 달성 항목
#[derive(Debug, Clone)]
pub struct GameAchievements {
    pub deepest_level: i32,
    pub monsters_killed: i32,
    pub gold_collected: i32,
    pub items_identified: i32,
    pub artifacts_found: i32,
    pub turns_elapsed: i32,
    pub conducts_kept: Vec<String>,
    pub quest_completed: bool,
    pub amulet_obtained: bool,
    pub ascended: bool,
    pub player_level: i32,
    pub death_reason: Option<String>,
}

/// [v2.34.0 98-5] 점수 구성요소
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub base_score: i64,
    pub depth_bonus: i64,
    pub kill_bonus: i64,
    pub gold_bonus: i64,
    pub artifact_bonus: i64,
    pub conduct_bonus: i64,
    pub ascension_bonus: i64,
    pub speed_bonus: i64,
    pub total_score: i64,
}

/// [v2.34.0 98-5] 점수 계산
pub fn calculate_score(achievements: &GameAchievements) -> ScoreBreakdown {
    let base = achievements.player_level as i64 * 100;
    let depth = achievements.deepest_level as i64 * 50;
    let kills = achievements.monsters_killed as i64 * 5;
    let gold = (achievements.gold_collected as i64) / 10;
    let artifacts = achievements.artifacts_found as i64 * 500;

    let conducts = achievements.conducts_kept.len() as i64 * 1000;

    let ascension = if achievements.ascended {
        50000
    } else if achievements.amulet_obtained {
        10000
    } else if achievements.quest_completed {
        5000
    } else {
        0
    };

    // 스피드 보너스: 빠를수록 좋음
    let speed = if achievements.ascended && achievements.turns_elapsed > 0 {
        let turns = achievements.turns_elapsed as i64;
        if turns < 20000 {
            10000
        } else if turns < 40000 {
            5000
        } else if turns < 80000 {
            2000
        } else {
            0
        }
    } else {
        0
    };

    let total = base + depth + kills + gold + artifacts + conducts + ascension + speed;

    ScoreBreakdown {
        base_score: base,
        depth_bonus: depth,
        kill_bonus: kills,
        gold_bonus: gold,
        artifact_bonus: artifacts,
        conduct_bonus: conducts,
        ascension_bonus: ascension,
        speed_bonus: speed,
        total_score: total,
    }
}

// =============================================================================
// [2] 행동 강령 (conduct) — conducts (end.c 핵심)
// =============================================================================

/// [v2.34.0 98-5] 행동 강령 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conduct {
    Foodless,     // 음식 미섭취
    Vegan,        // 채식
    Vegetarian,   // 유채식
    Pacifist,     // 비폭력
    Atheist,      // 무신론
    Weaponless,   // 무기 미사용
    Illiterate,   // 읽기/쓰기 안 함
    Wishless,     // 소원 안 함
    Artiwishless, // 아티팩트 소원 안 함
    Genocideless, // 제노사이드 안 함
    Elberethless, // 엘베레스 안 씀
    Blind,        // 맹인 플레이
    Nudist,       // 방어구 미착용
    Polyselfless, // 변신 안 함
}

/// [v2.34.0 98-5] 강령 위반 확인
pub fn check_conduct_violation(conduct: Conduct, action: &str) -> Option<String> {
    match (conduct, action) {
        (Conduct::Foodless, "먹기") => Some("Foodless 강령 위반!".to_string()),
        (Conduct::Vegan, "고기 먹기") | (Conduct::Vegan, "유제품 먹기") => {
            Some("Vegan 강령 위반!".to_string())
        }
        (Conduct::Vegetarian, "고기 먹기") => Some("Vegetarian 강령 위반!".to_string()),
        (Conduct::Pacifist, "공격") => Some("Pacifist 강령 위반!".to_string()),
        (Conduct::Atheist, "기도") | (Conduct::Atheist, "봉헌") => {
            Some("Atheist 강령 위반!".to_string())
        }
        (Conduct::Weaponless, "무기 장착") => Some("Weaponless 강령 위반!".to_string()),
        (Conduct::Illiterate, "읽기") | (Conduct::Illiterate, "쓰기") => {
            Some("Illiterate 강령 위반!".to_string())
        }
        (Conduct::Wishless, "소원") => Some("Wishless 강령 위반!".to_string()),
        (Conduct::Genocideless, "제노사이드") => Some("Genocideless 강령 위반!".to_string()),
        (Conduct::Nudist, "갑옷 착용") => Some("Nudist 강령 위반!".to_string()),
        _ => None,
    }
}

// =============================================================================
// [3] 하이스코어 — high_scores (topten.c 핵심)
// =============================================================================

/// [v2.34.0 98-5] 하이스코어 항목
#[derive(Debug, Clone)]
pub struct HighScoreEntry {
    pub rank: i32,
    pub name: String,
    pub race: String,
    pub role: String,
    pub score: i64,
    pub deepest_level: i32,
    pub death_reason: String,
    pub turns: i32,
    pub ascended: bool,
}

/// [v2.34.0 98-5] 순위 삽입
pub fn insert_high_score(
    scores: &[HighScoreEntry],
    new_entry: &HighScoreEntry,
    max_entries: usize,
) -> (Vec<HighScoreEntry>, i32) {
    let mut all: Vec<HighScoreEntry> = scores.to_vec();
    let mut new = new_entry.clone();

    // 삽입 위치 찾기
    let pos = all
        .iter()
        .position(|s| s.score < new.score)
        .unwrap_or(all.len());

    new.rank = (pos + 1) as i32;
    all.insert(pos, new);

    // 최대 항목 수 유지
    all.truncate(max_entries);

    // 순위 재할당
    for (i, entry) in all.iter_mut().enumerate() {
        entry.rank = (i + 1) as i32;
    }

    let inserted_rank = (pos + 1) as i32;
    (all, inserted_rank)
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn base_achievements() -> GameAchievements {
        GameAchievements {
            deepest_level: 20,
            monsters_killed: 100,
            gold_collected: 5000,
            items_identified: 30,
            artifacts_found: 2,
            turns_elapsed: 30000,
            conducts_kept: vec!["Pacifist".to_string()],
            quest_completed: true,
            amulet_obtained: false,
            ascended: false,
            player_level: 15,
            death_reason: Some("드래곤에게".to_string()),
        }
    }

    #[test]
    fn test_basic_score() {
        let score = calculate_score(&base_achievements());
        assert!(score.total_score > 0);
        assert!(score.base_score > 0);
    }

    #[test]
    fn test_ascension_bonus() {
        let mut a = base_achievements();
        a.ascended = true;
        a.amulet_obtained = true;
        let score = calculate_score(&a);
        assert!(score.ascension_bonus >= 50000);
    }

    #[test]
    fn test_speed_bonus() {
        let mut a = base_achievements();
        a.ascended = true;
        a.turns_elapsed = 15000;
        let score = calculate_score(&a);
        assert_eq!(score.speed_bonus, 10000);
    }

    #[test]
    fn test_conduct_violation() {
        let result = check_conduct_violation(Conduct::Pacifist, "공격");
        assert!(result.is_some());
    }

    #[test]
    fn test_conduct_ok() {
        let result = check_conduct_violation(Conduct::Pacifist, "먹기");
        assert!(result.is_none());
    }

    #[test]
    fn test_vegan_violation() {
        let result = check_conduct_violation(Conduct::Vegan, "고기 먹기");
        assert!(result.is_some());
    }

    #[test]
    fn test_high_score_insert() {
        let existing = vec![HighScoreEntry {
            rank: 1,
            name: "용사".to_string(),
            race: "인간".to_string(),
            role: "전사".to_string(),
            score: 10000,
            deepest_level: 20,
            death_reason: "드래곤".to_string(),
            turns: 30000,
            ascended: false,
        }];
        let new = HighScoreEntry {
            rank: 0,
            name: "영웅".to_string(),
            race: "엘프".to_string(),
            role: "마법사".to_string(),
            score: 20000,
            deepest_level: 30,
            death_reason: "승천".to_string(),
            turns: 15000,
            ascended: true,
        };
        let (scores, rank) = insert_high_score(&existing, &new, 10);
        assert_eq!(rank, 1);
        assert_eq!(scores[0].name, "영웅");
    }

    #[test]
    fn test_high_score_truncate() {
        let existing: Vec<HighScoreEntry> = (0..10)
            .map(|i| HighScoreEntry {
                rank: i + 1,
                name: format!("플레이어{}", i),
                race: "인간".to_string(),
                role: "전사".to_string(),
                score: (10 - i) as i64 * 1000,
                deepest_level: 10,
                death_reason: "사망".to_string(),
                turns: 20000,
                ascended: false,
            })
            .collect();
        let new = HighScoreEntry {
            rank: 0,
            name: "신인".to_string(),
            race: "인간".to_string(),
            role: "전사".to_string(),
            score: 500,
            deepest_level: 5,
            death_reason: "쥐".to_string(),
            turns: 1000,
            ascended: false,
        };
        let (scores, _) = insert_high_score(&existing, &new, 10);
        assert_eq!(scores.len(), 10);
    }
}
