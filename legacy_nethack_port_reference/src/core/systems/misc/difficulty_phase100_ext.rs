// ============================================================================
// [v2.36.0 Phase 100-1] 난이도/밸런스 통합 (difficulty_phase100_ext.rs)
// 원본: NetHack 3.6.7 전반 난이도 조정 로직 통합
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 난이도 프로파일 — difficulty_profile
// =============================================================================

/// [v2.36.0 100-1] 난이도 단계
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifficultyTier {
    Tutorial,  // 튜토리얼 (사실상 불사)
    Easy,      // 쉬움
    Normal,    // 보통 (원본 기준)
    Hard,      // 어려움
    Expert,    // 전문가
    Nightmare, // 악몽 (원본보다 난해)
}

/// [v2.36.0 100-1] 난이도 매개변수
#[derive(Debug, Clone)]
pub struct DifficultyParams {
    pub tier: DifficultyTier,
    pub monster_hp_mult: f64,
    pub monster_damage_mult: f64,
    pub monster_spawn_rate: f64,
    pub item_drop_rate: f64,
    pub trap_density: f64,
    pub shop_price_mult: f64,
    pub food_consumption_rate: f64,
    pub hp_regen_rate: f64,
    pub xp_mult: f64,
    pub respawn_on_death: bool,
}

/// [v2.36.0 100-1] 난이도별 매개변수 생성
pub fn get_difficulty_params(tier: DifficultyTier) -> DifficultyParams {
    match tier {
        DifficultyTier::Tutorial => DifficultyParams {
            tier,
            monster_hp_mult: 0.5,
            monster_damage_mult: 0.3,
            monster_spawn_rate: 0.5,
            item_drop_rate: 2.0,
            trap_density: 0.2,
            shop_price_mult: 0.5,
            food_consumption_rate: 0.5,
            hp_regen_rate: 2.0,
            xp_mult: 2.0,
            respawn_on_death: true,
        },
        DifficultyTier::Easy => DifficultyParams {
            tier,
            monster_hp_mult: 0.75,
            monster_damage_mult: 0.7,
            monster_spawn_rate: 0.8,
            item_drop_rate: 1.5,
            trap_density: 0.5,
            shop_price_mult: 0.8,
            food_consumption_rate: 0.8,
            hp_regen_rate: 1.5,
            xp_mult: 1.5,
            respawn_on_death: false,
        },
        DifficultyTier::Normal => DifficultyParams {
            tier,
            monster_hp_mult: 1.0,
            monster_damage_mult: 1.0,
            monster_spawn_rate: 1.0,
            item_drop_rate: 1.0,
            trap_density: 1.0,
            shop_price_mult: 1.0,
            food_consumption_rate: 1.0,
            hp_regen_rate: 1.0,
            xp_mult: 1.0,
            respawn_on_death: false,
        },
        DifficultyTier::Hard => DifficultyParams {
            tier,
            monster_hp_mult: 1.5,
            monster_damage_mult: 1.3,
            monster_spawn_rate: 1.3,
            item_drop_rate: 0.8,
            trap_density: 1.5,
            shop_price_mult: 1.5,
            food_consumption_rate: 1.2,
            hp_regen_rate: 0.8,
            xp_mult: 0.8,
            respawn_on_death: false,
        },
        DifficultyTier::Expert => DifficultyParams {
            tier,
            monster_hp_mult: 2.0,
            monster_damage_mult: 1.5,
            monster_spawn_rate: 1.5,
            item_drop_rate: 0.6,
            trap_density: 2.0,
            shop_price_mult: 2.0,
            food_consumption_rate: 1.5,
            hp_regen_rate: 0.5,
            xp_mult: 0.6,
            respawn_on_death: false,
        },
        DifficultyTier::Nightmare => DifficultyParams {
            tier,
            monster_hp_mult: 3.0,
            monster_damage_mult: 2.0,
            monster_spawn_rate: 2.0,
            item_drop_rate: 0.4,
            trap_density: 3.0,
            shop_price_mult: 3.0,
            food_consumption_rate: 2.0,
            hp_regen_rate: 0.3,
            xp_mult: 0.4,
            respawn_on_death: false,
        },
    }
}

// =============================================================================
// [2] 동적 난이도 조정 — adaptive_difficulty
// =============================================================================

/// [v2.36.0 100-1] 플레이어 실력 평가
#[derive(Debug, Clone)]
pub struct PlayerPerformance {
    pub deaths_count: i32,
    pub avg_turns_per_level: i32,
    pub monsters_killed_ratio: f64,
    pub items_used_efficiently: f64,
    pub damage_taken_ratio: f64,
    pub current_streak: i32,
}

/// [v2.36.0 100-1] 적응형 난이도 조정
pub fn adjust_difficulty(
    current: &DifficultyParams,
    performance: &PlayerPerformance,
) -> DifficultyParams {
    let mut adjusted = current.clone();

    // 많이 죽으면 난이도 하향
    if performance.deaths_count > 5 {
        adjusted.monster_damage_mult *= 0.9;
        adjusted.item_drop_rate *= 1.1;
    }

    // 너무 쉽게 진행하면 난이도 상향
    if performance.monsters_killed_ratio > 0.9 && performance.damage_taken_ratio < 0.2 {
        adjusted.monster_hp_mult *= 1.1;
        adjusted.monster_spawn_rate *= 1.1;
    }

    // 연승 보너스
    if performance.current_streak > 3 {
        adjusted.xp_mult *= 1.2;
    }

    adjusted
}

// =============================================================================
// [3] 밸런스 체크 — balance_check
// =============================================================================

/// [v2.36.0 100-1] 밸런스 지표
#[derive(Debug, Clone)]
pub struct BalanceReport {
    pub player_power: f64,
    pub environment_threat: f64,
    pub balance_ratio: f64,
    pub recommendation: String,
}

/// [v2.36.0 100-1] 밸런스 분석
pub fn analyze_balance(
    player_level: i32,
    dungeon_depth: i32,
    equipment_score: i32,
    monster_avg_level: i32,
    rng: &mut NetHackRng,
) -> BalanceReport {
    let player_power = player_level as f64 * 10.0 + equipment_score as f64 * 2.0;
    let threat = dungeon_depth as f64 * 8.0 + monster_avg_level as f64 * 5.0;
    let ratio = if threat > 0.0 {
        player_power / threat
    } else {
        999.0
    };

    let recommendation = if ratio > 2.0 {
        "플레이어가 너무 강함: 더 깊은 곳으로".to_string()
    } else if ratio > 1.2 {
        "양호한 밸런스".to_string()
    } else if ratio > 0.8 {
        "적절한 도전".to_string()
    } else if ratio > 0.5 {
        "위험 수준: 주의 필요".to_string()
    } else {
        "극도로 위험: 후퇴 권장".to_string()
    };

    // 난수 요소로 약간의 불확실성 추가
    let _variance = rng.rn2(10) as f64 / 100.0;

    BalanceReport {
        player_power,
        environment_threat: threat,
        balance_ratio: ratio,
        recommendation,
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
    fn test_normal_difficulty() {
        let p = get_difficulty_params(DifficultyTier::Normal);
        assert_eq!(p.monster_hp_mult, 1.0);
        assert_eq!(p.xp_mult, 1.0);
    }

    #[test]
    fn test_tutorial_easy() {
        let p = get_difficulty_params(DifficultyTier::Tutorial);
        assert!(p.monster_damage_mult < 0.5);
        assert!(p.respawn_on_death);
    }

    #[test]
    fn test_nightmare_hard() {
        let p = get_difficulty_params(DifficultyTier::Nightmare);
        assert!(p.monster_hp_mult >= 3.0);
    }

    #[test]
    fn test_adaptive_deaths() {
        let base = get_difficulty_params(DifficultyTier::Normal);
        let perf = PlayerPerformance {
            deaths_count: 10,
            avg_turns_per_level: 2000,
            monsters_killed_ratio: 0.5,
            items_used_efficiently: 0.5,
            damage_taken_ratio: 0.5,
            current_streak: 0,
        };
        let adjusted = adjust_difficulty(&base, &perf);
        assert!(adjusted.monster_damage_mult < base.monster_damage_mult);
    }

    #[test]
    fn test_adaptive_pro() {
        let base = get_difficulty_params(DifficultyTier::Normal);
        let perf = PlayerPerformance {
            deaths_count: 0,
            avg_turns_per_level: 500,
            monsters_killed_ratio: 0.95,
            items_used_efficiently: 0.9,
            damage_taken_ratio: 0.1,
            current_streak: 5,
        };
        let adjusted = adjust_difficulty(&base, &perf);
        assert!(adjusted.monster_hp_mult > base.monster_hp_mult);
    }

    #[test]
    fn test_balance_strong() {
        let mut rng = test_rng();
        let report = analyze_balance(20, 5, 50, 5, &mut rng);
        assert!(report.balance_ratio > 2.0);
    }

    #[test]
    fn test_balance_dangerous() {
        let mut rng = test_rng();
        let report = analyze_balance(5, 25, 10, 20, &mut rng);
        assert!(report.balance_ratio < 0.8);
    }

    #[test]
    fn test_all_tiers() {
        let tiers = [
            DifficultyTier::Tutorial,
            DifficultyTier::Easy,
            DifficultyTier::Normal,
            DifficultyTier::Hard,
            DifficultyTier::Expert,
            DifficultyTier::Nightmare,
        ];
        for t in &tiers {
            let p = get_difficulty_params(*t);
            assert!(p.monster_hp_mult > 0.0);
        }
    }
}
