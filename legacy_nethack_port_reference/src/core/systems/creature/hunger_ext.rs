// ============================================================================
// [v2.42.0 R30-2] 배고픔 시스템 (hunger_ext.rs)
// 원본: NetHack 3.6.7 eat.c/hack.c 배고픔 확장
// 배고픔 단계, 소모율, 영양 흡수, 기아사
// ============================================================================

/// [v2.42.0 R30-2] 배고픔 단계
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HungerLevel {
    Satiated,
    NotHungry,
    Hungry,
    Weak,
    Fainting,
    Fainted,
    Starved,
}

/// [v2.42.0 R30-2] 영양→배고픔 단계
pub fn hunger_from_nutrition(nutrition: i32) -> HungerLevel {
    match nutrition {
        2001.. => HungerLevel::Satiated,
        151..=2000 => HungerLevel::NotHungry,
        51..=150 => HungerLevel::Hungry,
        1..=50 => HungerLevel::Weak,
        -49..=0 => HungerLevel::Fainting,
        -99..=-50 => HungerLevel::Fainted,
        _ => HungerLevel::Starved,
    }
}

/// [v2.42.0 R30-2] 턴당 영양 소모
pub fn nutrition_burn_rate(encumbrance: i32, is_regenerating: bool, is_ring_hunger: bool) -> i32 {
    let base = 1;
    let enc_cost = encumbrance;
    let regen_cost = if is_regenerating { 1 } else { 0 };
    let ring_cost = if is_ring_hunger { 1 } else { 0 };
    base + enc_cost + regen_cost + ring_cost
}

/// [v2.42.0 R30-2] 과식 위험
pub fn overeating_risk(nutrition: i32, food_nutrition: i32) -> bool {
    nutrition + food_nutrition > 2500
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levels() {
        assert_eq!(hunger_from_nutrition(1000), HungerLevel::NotHungry);
        assert_eq!(hunger_from_nutrition(100), HungerLevel::Hungry);
        assert_eq!(hunger_from_nutrition(-200), HungerLevel::Starved);
    }

    #[test]
    fn test_burn_rate() {
        assert_eq!(nutrition_burn_rate(0, false, false), 1);
        assert_eq!(nutrition_burn_rate(2, true, true), 5);
    }

    #[test]
    fn test_overeat() {
        assert!(overeating_risk(2400, 200));
        assert!(!overeating_risk(1000, 200));
    }

    #[test]
    fn test_satiated() {
        assert_eq!(hunger_from_nutrition(3000), HungerLevel::Satiated);
    }
}
