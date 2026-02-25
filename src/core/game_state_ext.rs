// ============================================================================
// [v2.43.0 R31-5] 게임 상태 요약 (game_state_ext.rs)
// 최종 모듈: 게임 전체 상태 스냅샷
// 턴 수, 레벨, 점수, 주요 이벤트 추적
// ============================================================================

/// [v2.43.0 R31-5] 게임 상태 스냅샷
#[derive(Debug, Clone)]
pub struct GameSnapshot {
    pub turn: u64,
    pub depth: i32,
    pub branch: String,
    pub hp: i32,
    pub max_hp: i32,
    pub gold: i64,
    pub ac: i32,
    pub level: i32,
    pub score: i64,
    pub monsters_killed: i64,
    pub conducts: Vec<String>,
    pub artifacts_held: Vec<String>,
    pub has_amulet: bool,
}

/// [v2.43.0 R31-5] 게임 진행도
pub fn progress_pct(snapshot: &GameSnapshot) -> f64 {
    let mut p = 0.0;
    p += (snapshot.depth as f64 / 50.0) * 30.0;
    p += (snapshot.level as f64 / 30.0) * 20.0;
    if snapshot.has_amulet {
        p += 30.0;
    }
    p += (snapshot.artifacts_held.len() as f64).min(5.0) * 4.0;
    p.min(100.0)
}

/// [v2.43.0 R31-5] 난이도 평가
pub fn difficulty_rating(snapshot: &GameSnapshot) -> &'static str {
    let ratio = snapshot.hp as f64 / snapshot.max_hp.max(1) as f64;
    match (snapshot.depth, ratio) {
        (_, r) if r < 0.2 => "위험!",
        (d, _) if d > 40 => "극한",
        (d, _) if d > 25 => "상급",
        (d, _) if d > 10 => "중급",
        _ => "초급",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_snap() -> GameSnapshot {
        GameSnapshot {
            turn: 5000,
            depth: 15,
            branch: "main".into(),
            hp: 50,
            max_hp: 80,
            gold: 1000,
            ac: -5,
            level: 12,
            score: 30000,
            monsters_killed: 200,
            conducts: vec![],
            artifacts_held: vec!["Excalibur".into()],
            has_amulet: false,
        }
    }

    #[test]
    fn test_progress() {
        let p = progress_pct(&test_snap());
        assert!(p > 10.0 && p < 60.0);
    }

    #[test]
    fn test_amulet_progress() {
        let mut s = test_snap();
        s.has_amulet = true;
        assert!(progress_pct(&s) > 50.0);
    }

    #[test]
    fn test_difficulty() {
        assert_eq!(difficulty_rating(&test_snap()), "중급");
    }

    #[test]
    fn test_danger() {
        let mut s = test_snap();
        s.hp = 5;
        assert_eq!(difficulty_rating(&s), "위험!");
    }
}
