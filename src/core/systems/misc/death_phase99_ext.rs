// ============================================================================
// [v2.35.0 Phase 99-2] 종료/사망 확장 (death_phase99_ext.rs)
// 원본: NetHack 3.6.7 src/end.c L300-1500 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 사망 원인 — death_reasons (end.c L300-800)
// =============================================================================

/// [v2.35.0 99-2] 사망 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathType {
    KilledByMonster,
    Poisoned,
    Starvation,
    Drowned,
    BurnedToAsh,
    Petrified,
    Crushed,
    Fell,
    Zapped,
    Disease,
    Genocide,
    Quit,
    Escaped,
    Ascended,
    TurnedToSlime,
    Disintegrated,
    SelfDestruct,
}

/// [v2.35.0 99-2] 사망 이벤트
#[derive(Debug, Clone)]
pub struct DeathEvent {
    pub death_type: DeathType,
    pub killer_name: Option<String>,
    pub location: String,
    pub dungeon_level: i32,
    pub turn: i32,
    pub player_level: i32,
    pub hp_at_death: i32,
    pub max_hp: i32,
    pub last_words: Option<String>,
}

/// [v2.35.0 99-2] 묘비 문구 생성
/// 원본: end.c outrip()
pub fn generate_epitaph(event: &DeathEvent) -> String {
    let cause = match event.death_type {
        DeathType::KilledByMonster => {
            format!(
                "{}에게 살해당함",
                event.killer_name.as_deref().unwrap_or("알 수 없는 것")
            )
        }
        DeathType::Poisoned => "독에 의해 사망".to_string(),
        DeathType::Starvation => "굶주려 사망".to_string(),
        DeathType::Drowned => "익사".to_string(),
        DeathType::BurnedToAsh => "불에 타 사망".to_string(),
        DeathType::Petrified => "석화됨".to_string(),
        DeathType::Crushed => "압사".to_string(),
        DeathType::Fell => "추락사".to_string(),
        DeathType::Zapped => "마법에 의해 사망".to_string(),
        DeathType::Disease => "질병으로 사망".to_string(),
        DeathType::Genocide => "제노사이드됨".to_string(),
        DeathType::TurnedToSlime => "슬라임으로 변함".to_string(),
        DeathType::Disintegrated => "소멸됨".to_string(),
        DeathType::SelfDestruct => "자폭".to_string(),
        DeathType::Quit => "게임을 그만둠".to_string(),
        DeathType::Escaped => "던전에서 탈출함".to_string(),
        DeathType::Ascended => "승천함!".to_string(),
    };

    format!(
        "┌─────────────────────┐\n\
         │       R.I.P.        │\n\
         │  {}  │\n\
         │                     │\n\
         │  {}  │\n\
         │  레벨 {} / 턴 {}  │\n\
         │  {}층  │\n\
         └─────────────────────┘",
        event.killer_name.as_deref().unwrap_or("용사"),
        cause,
        event.player_level,
        event.turn,
        event.dungeon_level,
    )
}

// =============================================================================
// [2] 게임 종료 통계 — game_over_stats (end.c L800-1500)
// =============================================================================

/// [v2.35.0 99-2] 종료 통계
#[derive(Debug, Clone)]
pub struct GameOverStats {
    pub final_score: i64,
    pub death_type: DeathType,
    pub monsters_killed: i32,
    pub max_level_reached: i32,
    pub deepest_dungeon: i32,
    pub gold_collected: i64,
    pub artifacts_found: i32,
    pub turns_played: i32,
    pub conducts_kept: Vec<String>,
    pub achievements: Vec<String>,
    pub rank: String,
}

/// [v2.35.0 99-2] 칭호 반환
pub fn get_rank(role: &str, level: i32) -> String {
    match role {
        "전사" | "barbarian" => match level {
            1..=5 => "자유민".to_string(),
            6..=10 => "전투원".to_string(),
            11..=15 => "전사".to_string(),
            16..=20 => "영웅".to_string(),
            21..=25 => "전쟁군주".to_string(),
            _ => "대군주".to_string(),
        },
        "마법사" | "wizard" => match level {
            1..=5 => "견습생".to_string(),
            6..=10 => "마법사 제자".to_string(),
            11..=15 => "마법사".to_string(),
            16..=20 => "대마법사".to_string(),
            21..=25 => "마도사".to_string(),
            _ => "아크메이지".to_string(),
        },
        "도적" | "rogue" => match level {
            1..=5 => "소매치기".to_string(),
            6..=10 => "절도범".to_string(),
            11..=15 => "도적".to_string(),
            16..=20 => "침입자".to_string(),
            21..=25 => "도적 대장".to_string(),
            _ => "그림자 군주".to_string(),
        },
        "성직자" | "priest" => match level {
            1..=5 => "수습 사제".to_string(),
            6..=10 => "사제".to_string(),
            11..=15 => "성직자".to_string(),
            16..=20 => "대주교".to_string(),
            21..=25 => "추기경".to_string(),
            _ => "교황".to_string(),
        },
        _ => match level {
            1..=10 => "모험가".to_string(),
            11..=20 => "영웅".to_string(),
            _ => "전설".to_string(),
        },
    }
}

/// [v2.35.0 99-2] 게임 종료 달성 목록
pub fn generate_achievements(
    ascended: bool,
    quest_done: bool,
    sokoban_done: bool,
    mines_done: bool,
    artifacts: i32,
    conducts: &[String],
) -> Vec<String> {
    let mut achievements = Vec::new();
    if ascended {
        achievements.push("🏆 승천!".to_string());
    }
    if quest_done {
        achievements.push("⚔️ 퀘스트 완료".to_string());
    }
    if sokoban_done {
        achievements.push("🧩 소코반 클리어".to_string());
    }
    if mines_done {
        achievements.push("⛏️ 광산 탐험".to_string());
    }
    if artifacts >= 3 {
        achievements.push("💎 아티팩트 수집가".to_string());
    }
    if conducts.len() >= 3 {
        achievements.push("🎯 강령 수호자".to_string());
    }
    if conducts.iter().any(|c| c == "Pacifist") {
        achievements.push("☮️ 평화주의자".to_string());
    }
    if conducts.iter().any(|c| c == "Foodless") {
        achievements.push("🍃 금식가".to_string());
    }
    achievements
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epitaph_monster() {
        let event = DeathEvent {
            death_type: DeathType::KilledByMonster,
            killer_name: Some("드래곤".to_string()),
            location: "메인 던전".to_string(),
            dungeon_level: 15,
            turn: 10000,
            player_level: 12,
            hp_at_death: 0,
            max_hp: 80,
            last_words: None,
        };
        let epitaph = generate_epitaph(&event);
        assert!(epitaph.contains("드래곤"));
        assert!(epitaph.contains("R.I.P."));
    }

    #[test]
    fn test_epitaph_ascension() {
        let event = DeathEvent {
            death_type: DeathType::Ascended,
            killer_name: Some("영웅".to_string()),
            location: "천상계".to_string(),
            dungeon_level: 31,
            turn: 40000,
            player_level: 25,
            hp_at_death: 150,
            max_hp: 150,
            last_words: Some("승리!".to_string()),
        };
        let epitaph = generate_epitaph(&event);
        assert!(epitaph.contains("승천"));
    }

    #[test]
    fn test_rank_warrior() {
        assert_eq!(get_rank("전사", 1), "자유민");
        assert_eq!(get_rank("전사", 30), "대군주");
    }

    #[test]
    fn test_rank_wizard() {
        assert_eq!(get_rank("마법사", 25), "마도사");
    }

    #[test]
    fn test_achievements_ascended() {
        let a = generate_achievements(true, true, true, true, 5, &["Pacifist".to_string()]);
        assert!(a.iter().any(|x| x.contains("승천")));
        assert!(a.iter().any(|x| x.contains("평화주의")));
    }

    #[test]
    fn test_achievements_empty() {
        let a = generate_achievements(false, false, false, false, 0, &[]);
        assert!(a.is_empty());
    }

    #[test]
    fn test_rank_default() {
        assert_eq!(get_rank("관광객", 15), "영웅");
    }
}
