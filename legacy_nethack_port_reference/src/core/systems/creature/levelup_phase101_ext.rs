// ============================================================================
// [v2.37.0 Phase 101-5] 캐릭터 진행/레벨업 통합 (levelup_phase101_ext.rs)
// 원본: NetHack 3.6.7 src/exper.c + role.c 핵심 미이식 함수 통합
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 레벨업 시스템 — levelup (exper.c 핵심)
// =============================================================================

/// [v2.37.0 101-5] 레벨업 요구 경험치 테이블
pub fn xp_for_level(level: i32) -> i64 {
    match level {
        1 => 0,
        2 => 20,
        3 => 40,
        4 => 80,
        5 => 160,
        6 => 320,
        7 => 640,
        8 => 1280,
        9 => 2560,
        10 => 5120,
        11 => 10000,
        12 => 20000,
        13 => 40000,
        14 => 80000,
        15 => 160000,
        16 => 320000,
        17 => 640000,
        18 => 1280000,
        19 => 2560000,
        20 => 5120000,
        21..=30 => 5120000 + (level as i64 - 20) * 2000000,
        _ => 99999999,
    }
}

/// [v2.37.0 101-5] 레벨업 결과
#[derive(Debug, Clone)]
pub struct LevelUpResult {
    pub new_level: i32,
    pub hp_gain: i32,
    pub mp_gain: i32,
    pub skills_gained: Vec<String>,
    pub stat_increase: Option<(String, i32)>,
    pub title_change: Option<String>,
    pub message: String,
}

/// [v2.37.0 101-5] 레벨업 실행
pub fn process_level_up(
    current_level: i32,
    role: &str,
    con: i32,
    wis: i32,
    rng: &mut NetHackRng,
) -> LevelUpResult {
    let new_level = current_level + 1;

    // HP 획득 (직업 + 체력 기반)
    let base_hp = match role {
        "전사" | "barbarian" | "발키리" | "valkyrie" => rng.rn2(8) + 4, // d8+4
        "기사" | "knight" => rng.rn2(6) + 3,                            // d6+3
        "마법사" | "wizard" => rng.rn2(4) + 2,                          // d4+2
        "도적" | "rogue" => rng.rn2(6) + 2,                             // d6+2
        "성직자" | "priest" => rng.rn2(6) + 2,                          // d6+2
        "수도승" | "monk" => rng.rn2(8) + 2,                            // d8+2
        _ => rng.rn2(6) + 2,
    };
    let con_bonus = if con > 15 { (con - 15) / 2 } else { 0 };
    let hp_gain = base_hp + con_bonus;

    // MP 획득 (직업 + 지혜 기반)
    let base_mp = match role {
        "마법사" | "wizard" => rng.rn2(6) + 4,
        "성직자" | "priest" => rng.rn2(4) + 3,
        "수도승" | "monk" => rng.rn2(4) + 2,
        _ => rng.rn2(2) + 1,
    };
    let wis_bonus = if wis > 15 { (wis - 15) / 3 } else { 0 };
    let mp_gain = base_mp + wis_bonus;

    // 스킬 획득
    let skills = match (role, new_level) {
        ("전사", 5) | ("barbarian", 5) => vec!["분노 타격".to_string()],
        ("전사", 10) | ("barbarian", 10) => vec!["광전사의 분노".to_string()],
        ("마법사", 5) | ("wizard", 5) => vec!["마나 폭발".to_string()],
        ("마법사", 10) | ("wizard", 10) => vec!["차원 문".to_string()],
        ("도적", 5) | ("rogue", 5) => vec!["은밀 이동".to_string()],
        ("도적", 10) | ("rogue", 10) => vec!["급소 찌르기".to_string()],
        _ => vec![],
    };

    // 능력치 증가 (5레벨마다)
    let stat = if new_level % 5 == 0 {
        let stats = ["힘", "민첩", "체력", "지능", "지혜", "매력"];
        let chosen = stats[rng.rn2(6) as usize];
        Some((chosen.to_string(), 1))
    } else {
        None
    };

    // 칭호 변경
    let title = if new_level % 5 == 0 {
        Some(format!("Lv.{} {} 칭호 획득!", new_level, role))
    } else {
        None
    };

    let msg = format!(
        "레벨 {}! HP +{}, MP +{}{}",
        new_level,
        hp_gain,
        mp_gain,
        if !skills.is_empty() {
            format!(", 새 기술: {}", skills.join(", "))
        } else {
            String::new()
        },
    );

    LevelUpResult {
        new_level,
        hp_gain,
        mp_gain,
        skills_gained: skills,
        stat_increase: stat,
        title_change: title,
        message: msg,
    }
}

/// [v2.37.0 101-5] 경험치 → 레벨 확인
pub fn check_level_from_xp(xp: i64) -> i32 {
    for lvl in (1..=30).rev() {
        if xp >= xp_for_level(lvl) {
            return lvl;
        }
    }
    1
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
    fn test_xp_table() {
        assert_eq!(xp_for_level(1), 0);
        assert!(xp_for_level(10) > xp_for_level(5));
        assert!(xp_for_level(20) > xp_for_level(15));
    }

    #[test]
    fn test_warrior_levelup() {
        let mut rng = test_rng();
        let result = process_level_up(1, "전사", 16, 10, &mut rng);
        assert_eq!(result.new_level, 2);
        assert!(result.hp_gain >= 4);
    }

    #[test]
    fn test_wizard_mp() {
        let mut rng = test_rng();
        let result = process_level_up(1, "마법사", 10, 18, &mut rng);
        assert!(result.mp_gain >= 4);
    }

    #[test]
    fn test_skill_gain() {
        let mut rng = test_rng();
        let result = process_level_up(4, "전사", 14, 10, &mut rng);
        assert!(!result.skills_gained.is_empty());
    }

    #[test]
    fn test_stat_at_5() {
        let mut rng = test_rng();
        let result = process_level_up(4, "전사", 14, 10, &mut rng);
        assert!(result.stat_increase.is_some());
    }

    #[test]
    fn test_level_from_xp() {
        assert_eq!(check_level_from_xp(0), 1);
        assert_eq!(check_level_from_xp(100), 4);
        assert_eq!(check_level_from_xp(5120), 10);
    }

    #[test]
    fn test_title_at_milestone() {
        let mut rng = test_rng();
        let result = process_level_up(9, "마법사", 10, 18, &mut rng);
        assert!(result.title_change.is_some());
    }

    #[test]
    fn test_con_bonus() {
        let mut rng = test_rng();
        let low = process_level_up(1, "전사", 10, 10, &mut rng);
        let mut rng2 = NetHackRng::new(42);
        let high = process_level_up(1, "전사", 20, 10, &mut rng2);
        // 높은 체력은 더 많은 HP를 준다
        assert!(high.hp_gain >= low.hp_gain);
    }
}
