// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-11] 펫 이동 AI 확장 모듈 (dogmove_ext.rs)
// 원본: NetHack 3.6.7 dogmove.c (영양 계산, 공격 대상 평가, 허기)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 크기 상수 (원본: mondata.h)
// =============================================================================

/// [v2.22.0 R34-11] 몬스터 크기 등급 (원본: MZ_xxx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterSize {
    Tiny = 0,
    Small = 1,
    Medium = 2,
    Large = 3,
    Huge = 4,
    Gigantic = 5,
}

// =============================================================================
// [2] 펫 영양 계산 (원본: dogmove.c:141-202 dog_nutrition)
// =============================================================================

/// [v2.22.0 R34-11] 음식 클래스별 영양 계산 입력
#[derive(Debug, Clone)]
pub struct PetNutritionInput {
    /// 음식 클래스인지
    pub is_food_class: bool,
    /// 시체인지
    pub is_corpse: bool,
    /// 시체일 때 몬스터 무게 (cwt)
    pub corpse_weight: i32,
    /// 시체일 때 기본 영양 (cnutrit)
    pub corpse_nutrition: i32,
    /// 음식이지만 시체가 아닐 때 기본 영양 (oc_nutrition)
    pub base_nutrition: i32,
    /// 금화인지 (COIN_CLASS)
    pub is_coin: bool,
    /// 금화 수량
    pub coin_quantity: i64,
    /// 비음식/비금화일 때 아이템 영양 (oc_nutrition)
    pub object_nutrition: i32,
    /// 아이템 무게 (owt)
    pub object_weight: i32,
    /// 몬스터 크기
    pub monster_size: MonsterSize,
    /// 이미 먹은 비율 (0 = 안 먹음, 일부 = 부분 섭취)
    pub eaten_fraction: Option<(i32, i32)>, // (현재값, 최대값) — eaten_stat 적용용
}

/// [v2.22.0 R34-11] 영양 계산 결과
#[derive(Debug, Clone)]
pub struct PetNutritionResult {
    /// 영양값
    pub nutrition: i32,
    /// 식사 턴 수
    pub eating_turns: i32,
}

/// [v2.22.0 R34-11] 펫 영양/식사시간 계산 (원본: dog_nutrition)
pub fn calc_pet_nutrition(input: &PetNutritionInput) -> PetNutritionResult {
    let (mut nutrit, mut eating) = if input.is_food_class {
        if input.is_corpse {
            // 시체: 식사시간 = 3 + (cwt >> 6), 영양 = cnutrit
            let eat = 3 + (input.corpse_weight >> 6);
            (input.corpse_nutrition, eat)
        } else {
            // 일반 음식
            (input.base_nutrition, input.base_nutrition / 10) // oc_delay ≈ nutrition/10 근사
        }
    } else if input.is_coin {
        // 금화
        let eat = ((input.coin_quantity / 2000) + 1).max(1) as i32;
        let nut = (input.coin_quantity / 20).max(0) as i32;
        (nut, eat)
    } else {
        // 기타 (젤라틴 큐브 등)
        let eat = (input.object_weight / 20 + 1).max(1);
        let nut = 5 * input.object_nutrition;
        (nut, eat)
    };

    // 크기 보정 (작을수록 더 많은 영양)
    let multiplier = match input.monster_size {
        MonsterSize::Tiny => 8,
        MonsterSize::Small => 6,
        MonsterSize::Medium => 5,
        MonsterSize::Large => 4,
        MonsterSize::Huge => 3,
        MonsterSize::Gigantic => 2,
    };
    if input.is_food_class {
        nutrit *= multiplier;
    }

    // 부분 섭취 보정 (eaten_stat)
    if let Some((current, max_val)) = input.eaten_fraction {
        if max_val > 0 {
            eating = eating * current / max_val;
            nutrit = nutrit * current / max_val;
        }
    }

    PetNutritionResult {
        nutrition: nutrit,
        eating_turns: eating.max(1),
    }
}

// =============================================================================
// [3] 공격 대상 점수 계산 (원본: dogmove.c:708-807 score_targ)
// =============================================================================

/// [v2.22.0 R34-11] 공격 대상 평가 입력
#[derive(Debug, Clone)]
pub struct TargetScoreInput {
    /// 펫 레벨
    pub pet_level: i32,
    /// 대상 레벨
    pub target_level: i32,
    /// 대상 HP
    pub target_hp: i32,
    /// 대상이 평화적인지
    pub target_peaceful: bool,
    /// 대상이 펫인지
    pub target_tame: bool,
    /// 대상이 퀘스트 리더/가디언인지
    pub target_is_quest_friendly: bool,
    /// 대상의 첫 공격이 AT_NONE(수동적)인지
    pub target_passive: bool,
    /// 인접한지 (거리 ≤ 1)
    pub is_adjacent: bool,
    /// 아군이 대상 뒤에 있는지
    pub friends_behind: bool,
    /// 플레이어 레벨
    pub player_level: i32,
    /// 같은 진영의 성직자(동일 정렬)인지
    pub same_alignment_priest: bool,
}

/// [v2.22.0 R34-11] 공격 대상 점수 계산 (원본: score_targ)
/// 높을수록 공격 대상으로 더 적합
pub fn calc_target_score(input: &TargetScoreInput, rng: &mut NetHackRng) -> i64 {
    let mut score: i64 = 0;

    // 퀘스트 아군은 절대 공격하지 않음
    if input.target_is_quest_friendly {
        return -5000;
    }

    // 같은 정렬의 평화적 성직자 → 회피
    if input.same_alignment_priest && input.target_peaceful {
        return -5000;
    }

    // 인접한 몬스터 → 원거리 공격 대상으로 부적합
    if input.is_adjacent {
        return -3000;
    }

    // 펫/플레이어는 절대 대상으로 삼지 않음
    if input.target_tame {
        return -3000;
    }

    // 아군이 대상 뒤에 있으면 위험
    if input.friends_behind {
        return -3000;
    }

    // 적대적 몬스터 우선
    if !input.target_peaceful {
        score += 10;
    }

    // 수동적 몬스터 기피 (에너지 낭비)
    if input.target_passive {
        score -= 1000;
    }

    // 너무 약한 대상 기피
    if (input.target_level < 2 && input.pet_level > 5)
        || (input.pet_level > 12
            && input.target_level < input.pet_level - 9
            && input.player_level > 8
            && input.target_level < input.player_level - 7)
    {
        score -= 25;
    }

    // 너무 강한 대상 주저
    if input.target_level > input.pet_level + 4 {
        score -= (input.target_level - input.pet_level) as i64 * 20;
    }

    // 강한 대상 보너스 (너무 크지 않게)
    score += input.target_level as i64 * 2 + input.target_hp as i64 / 3;

    // 퍼지 팩터
    score += rng.rnd(5) as i64;

    score
}

// =============================================================================
// [4] 허기 판정 (원본: dogmove.c:355-397 dog_hunger)
// =============================================================================

/// [v2.22.0 R34-11] 허기 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HungerResult {
    /// 정상 (아직 배고프지 않음)
    Normal,
    /// 혼란 시작 (HP 1/3 감소)
    Confused { new_hp_max: i32, penalty: i32 },
    /// 굶주려 죽음
    Starved,
}

/// [v2.22.0 R34-11] 펫 허기 판정 (원본: dog_hunger)
pub fn calc_pet_hunger(
    current_moves: i64,
    hunger_time: i64,
    hp: i32,
    hp_max: i32,
    penalty_already: i32,
    can_eat: bool, // carnivorous || herbivorous
) -> HungerResult {
    // 500턴 이상 안 먹었으면
    if current_moves > hunger_time + 500 {
        if !can_eat {
            // 먹을 수 없는 몬스터 → 무시 (시간 갱신)
            return HungerResult::Normal;
        }

        if penalty_already == 0 {
            // 처음 굶주림 → HP 1/3로 감소
            let new_max = hp_max / 3;
            let penalty = hp_max - new_max;
            if new_max <= 0 {
                return HungerResult::Starved;
            }
            return HungerResult::Confused {
                new_hp_max: new_max,
                penalty,
            };
        }

        // 750턴 이상 또는 HP 0 이하 → 아사
        if current_moves > hunger_time + 750 || hp <= 0 {
            return HungerResult::Starved;
        }
    }
    HungerResult::Normal
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pet_nutrition_corpse_tiny() {
        let input = PetNutritionInput {
            is_food_class: true,
            is_corpse: true,
            corpse_weight: 200,
            corpse_nutrition: 100,
            base_nutrition: 0,
            is_coin: false,
            coin_quantity: 0,
            object_nutrition: 0,
            object_weight: 0,
            monster_size: MonsterSize::Tiny,
            eaten_fraction: None,
        };
        let result = calc_pet_nutrition(&input);
        assert_eq!(result.nutrition, 800); // 100 * 8
    }

    #[test]
    fn test_pet_nutrition_corpse_large() {
        let input = PetNutritionInput {
            is_food_class: true,
            is_corpse: true,
            corpse_weight: 200,
            corpse_nutrition: 100,
            base_nutrition: 0,
            is_coin: false,
            coin_quantity: 0,
            object_nutrition: 0,
            object_weight: 0,
            monster_size: MonsterSize::Large,
            eaten_fraction: None,
        };
        let result = calc_pet_nutrition(&input);
        assert_eq!(result.nutrition, 400); // 100 * 4
    }

    #[test]
    fn test_pet_nutrition_coin() {
        let input = PetNutritionInput {
            is_food_class: false,
            is_corpse: false,
            corpse_weight: 0,
            corpse_nutrition: 0,
            base_nutrition: 0,
            is_coin: true,
            coin_quantity: 1000,
            object_nutrition: 0,
            object_weight: 0,
            monster_size: MonsterSize::Medium,
            eaten_fraction: None,
        };
        let result = calc_pet_nutrition(&input);
        assert_eq!(result.nutrition, 50); // 1000/20
        assert_eq!(result.eating_turns, 1); // 1000/2000+1
    }

    #[test]
    fn test_target_score_quest_friendly() {
        let mut rng = NetHackRng::new(42);
        let input = TargetScoreInput {
            pet_level: 10,
            target_level: 5,
            target_hp: 30,
            target_peaceful: false,
            target_tame: false,
            target_is_quest_friendly: true,
            target_passive: false,
            is_adjacent: false,
            friends_behind: false,
            player_level: 10,
            same_alignment_priest: false,
        };
        assert_eq!(calc_target_score(&input, &mut rng), -5000);
    }

    #[test]
    fn test_target_score_hostile_strong() {
        let mut rng = NetHackRng::new(42);
        let input = TargetScoreInput {
            pet_level: 10,
            target_level: 8,
            target_hp: 40,
            target_peaceful: false,
            target_tame: false,
            target_is_quest_friendly: false,
            target_passive: false,
            is_adjacent: false,
            friends_behind: false,
            player_level: 10,
            same_alignment_priest: false,
        };
        let score = calc_target_score(&input, &mut rng);
        assert!(score > 0); // 적대+적절한 레벨 = 양수
    }

    #[test]
    fn test_target_score_much_stronger() {
        let mut rng = NetHackRng::new(42);
        let input = TargetScoreInput {
            pet_level: 5,
            target_level: 20,
            target_hp: 100,
            target_peaceful: false,
            target_tame: false,
            target_is_quest_friendly: false,
            target_passive: false,
            is_adjacent: false,
            friends_behind: false,
            player_level: 5,
            same_alignment_priest: false,
        };
        let score = calc_target_score(&input, &mut rng);
        assert!(score < 0); // 15레벨 차이 = 큰 페널티
    }

    #[test]
    fn test_hunger_normal() {
        let result = calc_pet_hunger(1000, 800, 20, 30, 0, true);
        assert_eq!(result, HungerResult::Normal);
    }

    #[test]
    fn test_hunger_confused() {
        let result = calc_pet_hunger(1500, 900, 20, 30, 0, true);
        match result {
            HungerResult::Confused {
                new_hp_max,
                penalty,
            } => {
                assert_eq!(new_hp_max, 10); // 30/3
                assert_eq!(penalty, 20); // 30-10
            }
            _ => panic!("Expected Confused"),
        }
    }

    #[test]
    fn test_hunger_starved() {
        let result = calc_pet_hunger(2000, 900, 5, 10, 7, true);
        assert_eq!(result, HungerResult::Starved);
    }
}
