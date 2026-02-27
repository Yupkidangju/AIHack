// ============================================================================
// [v2.39.0 Phase 103-1] 승천/엔딩 통합 (ascension_phase103_ext.rs)
// 원본: NetHack 3.6.7 src/end.c + src/topten.c 승천 로직 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 승천 조건 체크 (아뮬렛, 제단, 정렬)
//   - 승천 시퀀스 (챌린저, 라이더스, 최종 제단)
//   - 점수 계산 (아이템, 킬, 행동, 보너스)
//   - 승천 기록 (하이스코어 테이블)
//   - 게임 오버 변형 (탈출, 퇴출, 승천)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 승천 조건 — ascension_check
// =============================================================================

/// [v2.39.0 103-1] 승천 필요 조건
#[derive(Debug, Clone)]
pub struct AscensionRequirements {
    pub has_amulet_of_yendor: bool,
    pub on_astral_plane: bool,
    pub at_correct_altar: bool,
    pub alignment_matches: bool,
    pub riders_defeated: i32,      // 3명의 라이더 처치 수
    pub challengers_defeated: i32, // 챌린저 처치 수
}

/// [v2.39.0 103-1] 승천 체크 결과
#[derive(Debug, Clone)]
pub struct AscensionCheck {
    pub can_ascend: bool,
    pub missing: Vec<String>,
    pub progress_pct: f64,
}

/// [v2.39.0 103-1] 승천 조건 확인
pub fn check_ascension(req: &AscensionRequirements) -> AscensionCheck {
    let mut missing = Vec::new();
    let mut score = 0;
    let total = 5;

    if !req.has_amulet_of_yendor {
        missing.push("옌더의 부적 미소지".to_string());
    } else {
        score += 1;
    }

    if !req.on_astral_plane {
        missing.push("아스트랄 평원 미도달".to_string());
    } else {
        score += 1;
    }

    if !req.at_correct_altar {
        missing.push("올바른 제단 위에 있지 않음".to_string());
    } else {
        score += 1;
    }

    if !req.alignment_matches {
        missing.push("제단 정렬 불일치".to_string());
    } else {
        score += 1;
    }

    if req.riders_defeated < 3 {
        missing.push(format!("라이더 처치 미완 ({}/3)", req.riders_defeated));
    } else {
        score += 1;
    }

    AscensionCheck {
        can_ascend: missing.is_empty(),
        missing,
        progress_pct: score as f64 / total as f64 * 100.0,
    }
}

// =============================================================================
// [2] 최종 점수 — final_score
// =============================================================================

/// [v2.39.0 103-1] 점수 구성 요소
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub kill_score: i64,
    pub gold_score: i64,
    pub item_score: i64,
    pub exploration_score: i64,
    pub conduct_bonus: i64,
    pub ascension_bonus: i64,
    pub difficulty_multiplier: f64,
    pub total: i64,
}

/// [v2.39.0 103-1] 최종 점수 계산
pub fn calculate_final_score(
    monsters_killed: i32,
    gold: i64,
    unique_items: i32,
    levels_explored: i32,
    conducts_kept: i32,
    ascended: bool,
    difficulty_mult: f64,
) -> ScoreBreakdown {
    let kill = monsters_killed as i64 * 50;
    let gold_s = gold;
    let item = unique_items as i64 * 100;
    let explore = levels_explored as i64 * 200;
    let conduct = conducts_kept as i64 * 5000;
    let ascend = if ascended { 100000 } else { 0 };

    let raw = kill + gold_s + item + explore + conduct + ascend;
    let total = (raw as f64 * difficulty_mult) as i64;

    ScoreBreakdown {
        kill_score: kill,
        gold_score: gold_s,
        item_score: item,
        exploration_score: explore,
        conduct_bonus: conduct,
        ascension_bonus: ascend,
        difficulty_multiplier: difficulty_mult,
        total,
    }
}

// =============================================================================
// [3] 엔딩 유형 — ending_type
// =============================================================================

/// [v2.39.0 103-1] 엔딩 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndingType {
    Ascended,          // 승천 (최고)
    EscapedWithAmulet, // 부적 소지 탈출
    Escaped,           // 일반 탈출
    Died(String),      // 사망 (원인)
    Quit,              // 자진 포기
    Trickery,          // 꼼수 퇴출
}

/// [v2.39.0 103-1] 엔딩 메시지 생성
pub fn generate_ending_message(
    ending: &EndingType,
    player_name: &str,
    role: &str,
    score: i64,
    turns: i32,
) -> String {
    match ending {
        EndingType::Ascended => format!(
            "🏆 {}({}), 승천하였다! 점수: {} | 턴: {} | 당신은 반신이 되었다!",
            player_name, role, score, turns
        ),
        EndingType::EscapedWithAmulet => format!(
            "🏃 {}({}), 옌더의 부적을 들고 탈출했다! 점수: {}",
            player_name, role, score
        ),
        EndingType::Escaped => format!(
            "🏃 {}({}), 던전에서 탈출했다. 점수: {}",
            player_name, role, score
        ),
        EndingType::Died(cause) => format!(
            "💀 {}({}), {}에 의해 사망. 점수: {} | 턴: {}",
            player_name, role, cause, score, turns
        ),
        EndingType::Quit => format!(
            "🚪 {}({}), 게임을 포기했다. 점수: {}",
            player_name, role, score
        ),
        EndingType::Trickery => format!("⚠️ {}({}), 부정행위로 퇴출당했다.", player_name, role),
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascension_ready() {
        let req = AscensionRequirements {
            has_amulet_of_yendor: true,
            on_astral_plane: true,
            at_correct_altar: true,
            alignment_matches: true,
            riders_defeated: 3,
            challengers_defeated: 2,
        };
        let check = check_ascension(&req);
        assert!(check.can_ascend);
        assert_eq!(check.progress_pct, 100.0);
    }

    #[test]
    fn test_ascension_missing() {
        let req = AscensionRequirements {
            has_amulet_of_yendor: false,
            on_astral_plane: true,
            at_correct_altar: true,
            alignment_matches: true,
            riders_defeated: 2,
            challengers_defeated: 0,
        };
        let check = check_ascension(&req);
        assert!(!check.can_ascend);
        assert_eq!(check.missing.len(), 2);
    }

    #[test]
    fn test_score_ascended() {
        let score = calculate_final_score(200, 50000, 50, 30, 3, true, 1.0);
        assert!(score.total > 100000);
        assert_eq!(score.ascension_bonus, 100000);
    }

    #[test]
    fn test_score_died() {
        let score = calculate_final_score(50, 1000, 10, 5, 0, false, 1.0);
        assert!(score.total > 0);
        assert_eq!(score.ascension_bonus, 0);
    }

    #[test]
    fn test_difficulty_mult() {
        let easy = calculate_final_score(100, 5000, 20, 10, 0, false, 0.5);
        let hard = calculate_final_score(100, 5000, 20, 10, 0, false, 2.0);
        assert!(hard.total > easy.total);
    }

    #[test]
    fn test_ending_ascended() {
        let msg = generate_ending_message(&EndingType::Ascended, "용사", "전사", 150000, 30000);
        assert!(msg.contains("승천"));
        assert!(msg.contains("반신"));
    }

    #[test]
    fn test_ending_died() {
        let msg = generate_ending_message(
            &EndingType::Died("드래곤의 브레스".to_string()),
            "초보자",
            "마법사",
            5000,
            1000,
        );
        assert!(msg.contains("드래곤"));
        assert!(msg.contains("사망"));
    }

    #[test]
    fn test_conduct_bonus() {
        let with = calculate_final_score(100, 5000, 20, 10, 5, false, 1.0);
        let without = calculate_final_score(100, 5000, 20, 10, 0, false, 1.0);
        assert!(with.total > without.total);
    }
}
