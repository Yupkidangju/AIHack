// ============================================================================
// [v2.38.0 Phase 102-2] 몬스터 생태/습성 통합 (ecology_phase102_ext.rs)
// 원본: NetHack 3.6.7 src/mondata.c + monst.c 미이식 생태 데이터
// 순수 결과 패턴
//
// 구현 범위:
//   - 몬스터 식성(초식/잡식/육식/금속식/돌식 등)
//   - 활동 시간대(주행성/야행성/지하성)
//   - 서식지 적합도 계산
//   - 몬스터 간 관계(적대/공생/포식)
//   - 몬스터 그룹 행동 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 식성 — monster_diet
// =============================================================================

/// [v2.38.0 102-2] 식성 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DietType {
    Herbivore,   // 초식: 풀, 음식만
    Carnivore,   // 육식: 고기, 시체
    Omnivore,    // 잡식: 모든 음식
    Metallivore, // 금속식: 무기/갑옷
    Lithivore,   // 돌식: 돌/보석
    Undead,      // 언데드: 식사 불필요
    Amorphous,   // 무정형: 유기물 전반
    None,        // 식사 불필요 (골렘 등)
}

/// [v2.38.0 102-2] 활동 시간대
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityPeriod {
    Diurnal,      // 주행성: 밝은 곳에서 강함
    Nocturnal,    // 야행성: 어두운 곳에서 강함
    Subterranean, // 지하성: 던전 심층에서 강함
    Constant,     // 항시 활동
}

/// [v2.38.0 102-2] 서식지
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Habitat {
    Land,
    Water,
    Amphibious,
    Flying,
    Lava,
    Any,
}

/// [v2.38.0 102-2] 몬스터 생태 프로파일
#[derive(Debug, Clone)]
pub struct EcologyProfile {
    pub monster_name: String,
    pub diet: DietType,
    pub activity: ActivityPeriod,
    pub habitat: Habitat,
    pub social: bool,                // 군집 생활 여부
    pub territorial: bool,           // 영역 행동 여부
    pub pack_size: i32,              // 평균 무리 크기 (0=단독)
    pub preferred_depth: (i32, i32), // (최소, 최대) 선호 깊이
}

/// [v2.38.0 102-2] 몬스터 생태 프로파일 생성
pub fn create_ecology(name: &str) -> EcologyProfile {
    match name {
        "개미" | "ant" => EcologyProfile {
            monster_name: name.to_string(),
            diet: DietType::Omnivore,
            activity: ActivityPeriod::Subterranean,
            habitat: Habitat::Land,
            social: true,
            territorial: true,
            pack_size: 8,
            preferred_depth: (3, 15),
        },
        "드래곤" | "dragon" => EcologyProfile {
            monster_name: name.to_string(),
            diet: DietType::Carnivore,
            activity: ActivityPeriod::Constant,
            habitat: Habitat::Flying,
            social: false,
            territorial: true,
            pack_size: 0,
            preferred_depth: (20, 40),
        },
        "좀비" | "zombie" => EcologyProfile {
            monster_name: name.to_string(),
            diet: DietType::Undead,
            activity: ActivityPeriod::Nocturnal,
            habitat: Habitat::Land,
            social: true,
            territorial: false,
            pack_size: 5,
            preferred_depth: (5, 30),
        },
        "녹 괴물" | "rust monster" => EcologyProfile {
            monster_name: name.to_string(),
            diet: DietType::Metallivore,
            activity: ActivityPeriod::Subterranean,
            habitat: Habitat::Land,
            social: false,
            territorial: false,
            pack_size: 0,
            preferred_depth: (8, 20),
        },
        "물의 요정" | "water nymph" => EcologyProfile {
            monster_name: name.to_string(),
            diet: DietType::None,
            activity: ActivityPeriod::Constant,
            habitat: Habitat::Water,
            social: false,
            territorial: false,
            pack_size: 0,
            preferred_depth: (5, 25),
        },
        _ => EcologyProfile {
            monster_name: name.to_string(),
            diet: DietType::Omnivore,
            activity: ActivityPeriod::Constant,
            habitat: Habitat::Land,
            social: false,
            territorial: false,
            pack_size: 0,
            preferred_depth: (1, 50),
        },
    }
}

/// [v2.38.0 102-2] 서식지 적합도 (0.0~1.0)
pub fn habitat_fitness(profile: &EcologyProfile, depth: i32, is_lit: bool, terrain: &str) -> f64 {
    let mut fitness = 0.5; // 기본 적합도

    // 깊이 적합도
    let (min_d, max_d) = profile.preferred_depth;
    if depth >= min_d && depth <= max_d {
        fitness += 0.3;
    } else if depth < min_d {
        fitness -= (min_d - depth) as f64 * 0.05;
    } else {
        fitness -= (depth - max_d) as f64 * 0.03;
    }

    // 활동 시간대
    match profile.activity {
        ActivityPeriod::Nocturnal => {
            if !is_lit {
                fitness += 0.2;
            } else {
                fitness -= 0.1;
            }
        }
        ActivityPeriod::Diurnal => {
            if is_lit {
                fitness += 0.1;
            }
        }
        ActivityPeriod::Subterranean => {
            if depth > 10 {
                fitness += 0.15;
            }
        }
        _ => {}
    }

    // 지형 적합도
    match (profile.habitat, terrain) {
        (Habitat::Water, "물") | (Habitat::Water, "water") => fitness += 0.3,
        (Habitat::Lava, "용암") | (Habitat::Lava, "lava") => fitness += 0.3,
        (Habitat::Land, "물") => fitness -= 0.5,
        _ => {}
    }

    // [0.0, 1.0] 범위로 클램핑
    fitness.max(0.0).min(1.0)
}

/// [v2.38.0 102-2] 몬스터 관계 판단
pub fn check_relationship(a: &str, b: &str) -> &'static str {
    match (a, b) {
        ("고양이", "쥐") | ("cat", "rat") => "포식",
        ("개", "고양이") | ("dog", "cat") => "적대",
        ("개미", "개미") | ("ant", "ant") => "공생",
        ("드래곤", _) | ("dragon", _) => "적대",
        _ if a == b => "중립",
        _ => "무관심",
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ant_ecology() {
        let eco = create_ecology("개미");
        assert!(eco.social);
        assert_eq!(eco.pack_size, 8);
    }

    #[test]
    fn test_dragon_ecology() {
        let eco = create_ecology("드래곤");
        assert_eq!(eco.diet, DietType::Carnivore);
        assert!(!eco.social);
    }

    #[test]
    fn test_zombie_nocturnal() {
        let eco = create_ecology("좀비");
        assert_eq!(eco.activity, ActivityPeriod::Nocturnal);
    }

    #[test]
    fn test_habitat_fitness_good() {
        let eco = create_ecology("개미");
        let f = habitat_fitness(&eco, 10, false, "땅");
        assert!(f >= 0.5);
    }

    #[test]
    fn test_habitat_fitness_bad() {
        let eco = create_ecology("드래곤");
        let f = habitat_fitness(&eco, 1, true, "땅");
        assert!(f < 0.5); // 얕은 층에서 나쁜 적합도
    }

    #[test]
    fn test_nocturnal_bonus() {
        let eco = create_ecology("좀비");
        let dark = habitat_fitness(&eco, 10, false, "땅");
        let lit = habitat_fitness(&eco, 10, true, "땅");
        assert!(dark > lit);
    }

    #[test]
    fn test_relationship_predator() {
        assert_eq!(check_relationship("고양이", "쥐"), "포식");
    }

    #[test]
    fn test_relationship_same() {
        assert_eq!(check_relationship("오크", "오크"), "중립");
    }

    #[test]
    fn test_rust_monster() {
        let eco = create_ecology("녹 괴물");
        assert_eq!(eco.diet, DietType::Metallivore);
    }
}
