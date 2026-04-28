// ============================================================================
// [v2.37.0 R25-3] 음식 부패 (food_spoil_ext.rs)
// 원본: NetHack 3.6.7 eat.c 부패 확장
// 음식 유형별 부패 속도, 보존, 영양가
// ============================================================================

/// [v2.37.0 R25-3] 음식 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoodType {
    Corpse,
    MeatStick,
    TinMeat, // 통조림
    FoodRation,
    Lembas, // 요정빵
    CramRation,
    Apple,
    Carrot,
    Banana,
    Egg,
    Tripe,
    Kelp,
}

/// [v2.37.0 R25-3] 부패 시간 (턴)
pub fn spoil_time(food: FoodType) -> Option<u64> {
    match food {
        FoodType::Corpse => Some(50),
        FoodType::Egg => Some(100),
        FoodType::Apple | FoodType::Banana => Some(200),
        FoodType::Tripe => Some(75),
        FoodType::MeatStick => Some(300),
        FoodType::TinMeat => None, // 통조림은 부패 안 함
        FoodType::FoodRation => None,
        FoodType::Lembas => None,
        FoodType::CramRation => None,
        FoodType::Carrot => Some(250),
        FoodType::Kelp => Some(150),
    }
}

/// [v2.37.0 R25-3] 영양가
pub fn nutrition(food: FoodType) -> i32 {
    match food {
        FoodType::Corpse => 100, // 시체마다 다르지만 기본값
        FoodType::FoodRation => 800,
        FoodType::Lembas => 800,
        FoodType::CramRation => 600,
        FoodType::TinMeat => 300,
        FoodType::MeatStick => 250,
        FoodType::Apple => 50,
        FoodType::Banana => 80,
        FoodType::Carrot => 50,
        FoodType::Egg => 80,
        FoodType::Tripe => 200,
        FoodType::Kelp => 30,
    }
}

/// [v2.37.0 R25-3] 부패 여부 판정
pub fn is_spoiled(food: FoodType, age: u64, in_icebox: bool) -> bool {
    if in_icebox {
        return false;
    } // 아이스박스 보존
    match spoil_time(food) {
        Some(time) => age > time,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spoil_time() {
        assert_eq!(spoil_time(FoodType::Corpse), Some(50));
        assert_eq!(spoil_time(FoodType::FoodRation), None);
    }

    #[test]
    fn test_nutrition() {
        assert_eq!(nutrition(FoodType::FoodRation), 800);
        assert_eq!(nutrition(FoodType::Apple), 50);
    }

    #[test]
    fn test_spoiled() {
        assert!(is_spoiled(FoodType::Corpse, 60, false));
        assert!(!is_spoiled(FoodType::Corpse, 30, false));
    }

    #[test]
    fn test_icebox() {
        assert!(!is_spoiled(FoodType::Corpse, 1000, true));
    }

    #[test]
    fn test_imperishable() {
        assert!(!is_spoiled(FoodType::FoodRation, 999999, false));
    }
}
