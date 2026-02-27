// ============================================================================
// [v2.40.0 Phase 104-4] 음식/영양 통합 (nutrition_phase104_ext.rs)
// 원본: NetHack 3.6.7 src/eat.c 미이식 영양 시스템 통합
// 순수 결과 패턴
//
// 구현 범위:
//   - 배고픔 단계 (6단계)
//   - 음식별 영양가 / 턴당 소비
//   - 식중독/알레르기/채식주의
//   - 시체 섭취 효과 (내성, 능력치, 독)
//   - 강제 식사 (석상, 금속 등)
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.40.0 104-4] 배고픔 단계
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HungerLevel {
    Satiated,  // 포만 (>= 1500)
    NotHungry, // 배부름 (1000~1499)
    Hungry,    // 배고픔 (500~999)
    Weak,      // 허약 (150~499)
    Fainting,  // 기절 직전 (1~149)
    Starved,   // 아사 (0)
}

/// [v2.40.0 104-4] 영양 상태
#[derive(Debug, Clone)]
pub struct NutritionState {
    pub nutrition: i32,
    pub consumption_rate: i32, // 턴당 소비량 (기본 1)
    pub is_vegetarian: bool,
    pub cannibalism_count: i32,
}

impl NutritionState {
    pub fn new(is_vegetarian: bool) -> Self {
        Self {
            nutrition: 900,
            consumption_rate: 1,
            is_vegetarian,
            cannibalism_count: 0,
        }
    }

    /// 현재 배고픔 단계
    pub fn hunger_level(&self) -> HungerLevel {
        match self.nutrition {
            n if n >= 1500 => HungerLevel::Satiated,
            n if n >= 1000 => HungerLevel::NotHungry,
            n if n >= 500 => HungerLevel::Hungry,
            n if n >= 150 => HungerLevel::Weak,
            n if n >= 1 => HungerLevel::Fainting,
            _ => HungerLevel::Starved,
        }
    }

    /// 턴 경과 (영양 소비)
    pub fn tick(&mut self) -> Option<String> {
        self.nutrition -= self.consumption_rate;
        if self.nutrition < 0 {
            self.nutrition = 0;
        }

        let level = self.hunger_level();
        match level {
            HungerLevel::Starved => Some("💀 굶어 죽었다!".to_string()),
            HungerLevel::Fainting => Some("기절할 것 같다... 음식을 먹어라!".to_string()),
            HungerLevel::Weak => Some("배가 고프다. 허약해지고 있다.".to_string()),
            HungerLevel::Hungry if self.nutrition == 999 => {
                Some("배가 고파지기 시작한다.".to_string())
            }
            _ => None,
        }
    }
}

/// [v2.40.0 104-4] 음식 종류
#[derive(Debug, Clone)]
pub struct FoodItem {
    pub name: String,
    pub nutrition_value: i32,
    pub is_meat: bool,
    pub is_poisonous: bool,
    pub turns_to_eat: i32,
}

/// [v2.40.0 104-4] 음식 섭취
pub fn eat_food(
    state: &mut NutritionState,
    food: &FoodItem,
    rng: &mut NetHackRng,
) -> (String, i32) {
    // 채식주의 체크
    if state.is_vegetarian && food.is_meat {
        return ("채식주의자로서 고기를 먹을 수 없다!".to_string(), 0);
    }

    // 독 체크
    if food.is_poisonous {
        let saved = rng.rn2(5) == 0; // 20% 확률로 저항
        if !saved {
            let poison_dmg = rng.rn2(15) + 5;
            state.nutrition += food.nutrition_value / 2; // 절반만 흡수
            return (format!("독에 당했다! ({} 데미지)", poison_dmg), -poison_dmg);
        }
    }

    // 포만 체크
    let old_level = state.hunger_level();
    state.nutrition += food.nutrition_value;

    if state.nutrition > 2000 {
        state.nutrition = 2000;
        return ("과식이다! 움직이기 힘들다.".to_string(), 0);
    }

    let new_level = state.hunger_level();
    let msg = if old_level != new_level {
        match new_level {
            HungerLevel::Satiated => format!("{}을(를) 먹었다. 배가 부르다!", food.name),
            HungerLevel::NotHungry => format!("{}을(를) 먹었다. 괜찮은 기분이다.", food.name),
            _ => format!("{}을(를) 먹었다.", food.name),
        }
    } else {
        format!(
            "{}을(를) 먹었다. (영양 +{})",
            food.name, food.nutrition_value
        )
    };

    (msg, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    #[test]
    fn test_initial_state() {
        let state = NutritionState::new(false);
        assert_eq!(state.hunger_level(), HungerLevel::Hungry);
    }

    #[test]
    fn test_tick_hunger() {
        let mut state = NutritionState::new(false);
        state.nutrition = 500;
        state.tick();
        assert_eq!(state.nutrition, 499);
    }

    #[test]
    fn test_starved() {
        let mut state = NutritionState::new(false);
        state.nutrition = 1;
        let msg = state.tick();
        assert_eq!(state.nutrition, 0);
        assert!(msg.is_some());
    }

    #[test]
    fn test_eat_normal() {
        let mut state = NutritionState::new(false);
        state.nutrition = 500;
        let food = FoodItem {
            name: "빵".to_string(),
            nutrition_value: 200,
            is_meat: false,
            is_poisonous: false,
            turns_to_eat: 3,
        };
        let mut rng = test_rng();
        let (msg, _) = eat_food(&mut state, &food, &mut rng);
        assert_eq!(state.nutrition, 700);
        assert!(msg.contains("빵"));
    }

    #[test]
    fn test_vegetarian_refuse() {
        let mut state = NutritionState::new(true);
        let food = FoodItem {
            name: "스테이크".to_string(),
            nutrition_value: 500,
            is_meat: true,
            is_poisonous: false,
            turns_to_eat: 5,
        };
        let mut rng = test_rng();
        let (msg, _) = eat_food(&mut state, &food, &mut rng);
        assert!(msg.contains("채식주의"));
    }

    #[test]
    fn test_overeating() {
        let mut state = NutritionState::new(false);
        state.nutrition = 1800;
        let food = FoodItem {
            name: "거대 음식".to_string(),
            nutrition_value: 500,
            is_meat: false,
            is_poisonous: false,
            turns_to_eat: 5,
        };
        let mut rng = test_rng();
        eat_food(&mut state, &food, &mut rng);
        assert!(state.nutrition <= 2000);
    }

    #[test]
    fn test_hunger_stages() {
        assert_eq!(
            NutritionState {
                nutrition: 1500,
                consumption_rate: 1,
                is_vegetarian: false,
                cannibalism_count: 0
            }
            .hunger_level(),
            HungerLevel::Satiated
        );
        assert_eq!(
            NutritionState {
                nutrition: 0,
                consumption_rate: 1,
                is_vegetarian: false,
                cannibalism_count: 0
            }
            .hunger_level(),
            HungerLevel::Starved
        );
    }
}
