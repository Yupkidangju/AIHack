// ============================================================================
// [v2.35.0 R23-5] 종합 점수 계산 (score_calc_ext.rs)
// 원본: NetHack 3.6.7 topten.c 점수 계산 확장
// 킬 점수, 금 점수, 어센션 보너스, 최종 점수
// ============================================================================

/// [v2.35.0 R23-5] 점수 구성요소
#[derive(Debug, Clone, Default)]
pub struct ScoreBreakdown {
    pub gold_score: i64,
    pub kill_score: i64,
    pub depth_bonus: i64,
    pub conduct_bonus: i64,
    pub ascension_bonus: i64,
    pub artifact_bonus: i64,
    pub turn_penalty: i64,
    pub total: i64,
}

/// [v2.35.0 R23-5] 종합 점수 계산
pub fn calculate_score(
    gold: i64,
    kills: i64,
    max_depth: i32,
    conducts_kept: i32,
    ascended: bool,
    artifacts_collected: i32,
    turns: i64,
) -> ScoreBreakdown {
    let gold_score = gold;
    let kill_score = kills * 4;
    let depth_bonus = max_depth as i64 * 100;
    let conduct_bonus = conducts_kept as i64 * 5000;
    let ascension_bonus = if ascended { 100000 } else { 0 };
    let artifact_bonus = artifacts_collected as i64 * 2500;
    let turn_penalty = turns / 1000; // 긴 플레이 페널티

    let total =
        gold_score + kill_score + depth_bonus + conduct_bonus + ascension_bonus + artifact_bonus
            - turn_penalty;

    ScoreBreakdown {
        gold_score,
        kill_score,
        depth_bonus,
        conduct_bonus,
        ascension_bonus,
        artifact_bonus,
        turn_penalty,
        total,
    }
}

/// [v2.35.0 R23-5] 행동 규범 (conduct) 목록
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Conduct {
    Foodless,     // 음식 미섭취
    Vegan,        // 비건
    Vegetarian,   // 채식
    Atheist,      // 기도 미사용
    Weaponless,   // 무기 미사용
    Pacifist,     // 비살생
    Illiterate,   // 읽기/쓰기 미사용
    Polypileless, // 변신 미사용
    Wishless,     // 소원 미사용
    Artless,      // 아티팩트 미사용
}

/// [v2.35.0 R23-5] 행동 규범 점수 가중치
pub fn conduct_weight(conduct: Conduct) -> i64 {
    match conduct {
        Conduct::Foodless => 10000,
        Conduct::Pacifist => 8000,
        Conduct::Atheist => 7000,
        Conduct::Weaponless => 6000,
        Conduct::Wishless => 5000,
        Conduct::Vegan => 4000,
        Conduct::Vegetarian => 3000,
        Conduct::Illiterate => 3000,
        Conduct::Polypileless => 2000,
        Conduct::Artless => 2000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_score() {
        let s = calculate_score(10000, 100, 30, 0, false, 0, 5000);
        assert_eq!(s.gold_score, 10000);
        assert_eq!(s.kill_score, 400);
        assert_eq!(s.depth_bonus, 3000);
    }

    #[test]
    fn test_ascension() {
        let s = calculate_score(0, 0, 46, 0, true, 0, 0);
        assert_eq!(s.ascension_bonus, 100000);
    }

    #[test]
    fn test_conducts() {
        let s = calculate_score(0, 0, 1, 5, false, 0, 0);
        assert_eq!(s.conduct_bonus, 25000);
    }

    #[test]
    fn test_turn_penalty() {
        let s = calculate_score(0, 0, 1, 0, false, 0, 100000);
        assert_eq!(s.turn_penalty, 100);
    }

    #[test]
    fn test_conduct_weights() {
        assert_eq!(conduct_weight(Conduct::Foodless), 10000);
        assert_eq!(conduct_weight(Conduct::Pacifist), 8000);
    }
}
