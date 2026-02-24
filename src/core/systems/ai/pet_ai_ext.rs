// ============================================================================
// [v2.31.0 R19-3] 펫 AI 확장 (pet_ai_ext.rs)
// 원본: NetHack 3.6.7 dog.c AI 확장
// 펫 행동 결정, 먹이 선호, 충성도, 훈련
// ============================================================================

/// [v2.31.0 R19-3] 펫 행동
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PetAction {
    FollowOwner,
    AttackMonster { target_id: u64 },
    EatFood { item_id: u64 },
    PickupItem { item_id: u64 },
    Guard,
    Wander,
    Sit,
}

/// [v2.31.0 R19-3] 펫 상태
#[derive(Debug, Clone)]
pub struct PetState {
    pub hunger: i32,  // 0~100 (100=극도로 배고픔)
    pub loyalty: i32, // 0~100 (0=야생화 직전)
    pub distance_to_owner: i32,
    pub nearby_monster: bool,
    pub nearby_food: bool,
    pub tameness: i32, // 1~20
}

/// [v2.31.0 R19-3] 펫 행동 결정 (원본: dog_move)
pub fn decide_pet_action(state: &PetState) -> PetAction {
    // 극도로 배고프면 음식 우선
    if state.hunger > 80 && state.nearby_food {
        return PetAction::EatFood { item_id: 0 };
    }

    // 충성도 낮으면 방황 (야생화 전조)
    if state.loyalty < 20 {
        return PetAction::Wander;
    }

    // 근처에 몬스터 + 충분한 충성도
    if state.nearby_monster && state.tameness >= 5 {
        return PetAction::AttackMonster { target_id: 0 };
    }

    // 주인에게서 너무 멀면 따라가기
    if state.distance_to_owner > 5 {
        return PetAction::FollowOwner;
    }

    // 주인 근처에서 가드
    if state.distance_to_owner <= 2 {
        return PetAction::Guard;
    }

    PetAction::FollowOwner
}

/// [v2.31.0 R19-3] 충성도 변화 (원본: dog.c tamedog)
pub fn update_loyalty(current: i32, fed: bool, hit_by_owner: bool, turns_since_fed: u64) -> i32 {
    let mut loyalty = current;
    if fed {
        loyalty += 10;
    }
    if hit_by_owner {
        loyalty -= 30;
    }
    if turns_since_fed > 200 {
        loyalty -= 5;
    }
    loyalty.clamp(0, 100)
}

/// [v2.31.0 R19-3] 먹이 선호 판별 (원본: dogfood)
pub fn food_preference(pet_type: &str, food_name: &str) -> i32 {
    match pet_type {
        "dog" | "wolf" => match food_name {
            "tripe ration" => 10,
            "meat" | "corpse" => 8,
            "food ration" => 3,
            _ => 1,
        },
        "cat" => match food_name {
            "tripe ration" => 10,
            "corpse" => 9,
            "fish" => 10,
            _ => 1,
        },
        "horse" => match food_name {
            "carrot" | "apple" => 10,
            "food ration" => 5,
            _ => 0,
        },
        _ => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_state() -> PetState {
        PetState {
            hunger: 30,
            loyalty: 80,
            distance_to_owner: 3,
            nearby_monster: false,
            nearby_food: false,
            tameness: 10,
        }
    }

    #[test]
    fn test_hungry_eat() {
        let state = PetState {
            hunger: 90,
            nearby_food: true,
            ..base_state()
        };
        assert!(matches!(
            decide_pet_action(&state),
            PetAction::EatFood { .. }
        ));
    }

    #[test]
    fn test_attack_monster() {
        let state = PetState {
            nearby_monster: true,
            ..base_state()
        };
        assert!(matches!(
            decide_pet_action(&state),
            PetAction::AttackMonster { .. }
        ));
    }

    #[test]
    fn test_follow_far() {
        let state = PetState {
            distance_to_owner: 8,
            ..base_state()
        };
        assert_eq!(decide_pet_action(&state), PetAction::FollowOwner);
    }

    #[test]
    fn test_loyalty_fed() {
        assert_eq!(update_loyalty(50, true, false, 0), 60);
    }

    #[test]
    fn test_loyalty_hit() {
        assert_eq!(update_loyalty(50, false, true, 0), 20);
    }

    #[test]
    fn test_food_dog() {
        assert_eq!(food_preference("dog", "tripe ration"), 10);
        assert_eq!(food_preference("dog", "food ration"), 3);
    }

    #[test]
    fn test_food_horse() {
        assert_eq!(food_preference("horse", "carrot"), 10);
    }
}
