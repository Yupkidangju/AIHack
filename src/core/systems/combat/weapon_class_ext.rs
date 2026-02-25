// ============================================================================
// [v2.38.0 R26-3] 무기 계열 (weapon_class_ext.rs)
// 원본: NetHack 3.6.7 weapon.c 분류 확장
// 무기 카테고리, 양손/한손, 사거리, 특수 속성
// ============================================================================

/// [v2.38.0 R26-3] 무기 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponCategory {
    Sword,
    Axe,
    Mace,
    Polearm,
    Spear,
    Dagger,
    Bow,
    Crossbow,
    Sling,
    Whip,
    Staff,
    Lance,
    Trident,
    Martial, // 맨손/무술
}

/// [v2.38.0 R26-3] 무기 속성
#[derive(Debug, Clone)]
pub struct WeaponProfile {
    pub name: &'static str,
    pub category: WeaponCategory,
    pub two_handed: bool,
    pub range: i32,          // 0=근접, 1+=원거리
    pub base_damage_sm: i32, // vs small
    pub base_damage_lg: i32, // vs large
    pub weight: i32,
    pub silver: bool,
}

/// [v2.38.0 R26-3] 무기 테이블 (발췌)
pub fn weapon_table() -> Vec<WeaponProfile> {
    vec![
        WeaponProfile {
            name: "long sword",
            category: WeaponCategory::Sword,
            two_handed: false,
            range: 0,
            base_damage_sm: 8,
            base_damage_lg: 12,
            weight: 40,
            silver: false,
        },
        WeaponProfile {
            name: "two-handed sword",
            category: WeaponCategory::Sword,
            two_handed: true,
            range: 0,
            base_damage_sm: 12,
            base_damage_lg: 18,
            weight: 150,
            silver: false,
        },
        WeaponProfile {
            name: "silver saber",
            category: WeaponCategory::Sword,
            two_handed: false,
            range: 0,
            base_damage_sm: 8,
            base_damage_lg: 8,
            weight: 40,
            silver: true,
        },
        WeaponProfile {
            name: "battle-axe",
            category: WeaponCategory::Axe,
            two_handed: true,
            range: 0,
            base_damage_sm: 10,
            base_damage_lg: 10,
            weight: 120,
            silver: false,
        },
        WeaponProfile {
            name: "dagger",
            category: WeaponCategory::Dagger,
            two_handed: false,
            range: 1,
            base_damage_sm: 4,
            base_damage_lg: 3,
            weight: 10,
            silver: false,
        },
        WeaponProfile {
            name: "silver dagger",
            category: WeaponCategory::Dagger,
            two_handed: false,
            range: 1,
            base_damage_sm: 4,
            base_damage_lg: 3,
            weight: 12,
            silver: true,
        },
        WeaponProfile {
            name: "bow",
            category: WeaponCategory::Bow,
            two_handed: true,
            range: 7,
            base_damage_sm: 0,
            base_damage_lg: 0,
            weight: 30,
            silver: false,
        },
        WeaponProfile {
            name: "crossbow",
            category: WeaponCategory::Crossbow,
            two_handed: true,
            range: 8,
            base_damage_sm: 0,
            base_damage_lg: 0,
            weight: 50,
            silver: false,
        },
        WeaponProfile {
            name: "lance",
            category: WeaponCategory::Lance,
            two_handed: false,
            range: 0,
            base_damage_sm: 6,
            base_damage_lg: 8,
            weight: 180,
            silver: false,
        },
        WeaponProfile {
            name: "quarterstaff",
            category: WeaponCategory::Staff,
            two_handed: true,
            range: 0,
            base_damage_sm: 6,
            base_damage_lg: 6,
            weight: 40,
            silver: false,
        },
    ]
}

/// [v2.38.0 R26-3] 은 무기 보너스
pub fn silver_bonus(silver: bool, target_demon: bool, target_undead: bool) -> i32 {
    if silver && (target_demon || target_undead) {
        20
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_count() {
        assert_eq!(weapon_table().len(), 10);
    }

    #[test]
    fn test_two_handed() {
        let twohs: Vec<_> = weapon_table()
            .into_iter()
            .filter(|w| w.two_handed)
            .collect();
        assert!(twohs.len() >= 3);
    }

    #[test]
    fn test_silver() {
        let silvers: Vec<_> = weapon_table().into_iter().filter(|w| w.silver).collect();
        assert_eq!(silvers.len(), 2);
    }

    #[test]
    fn test_silver_bonus() {
        assert_eq!(silver_bonus(true, true, false), 20);
        assert_eq!(silver_bonus(false, true, false), 0);
    }

    #[test]
    fn test_ranged() {
        let bow = weapon_table()
            .into_iter()
            .find(|w| w.name == "bow")
            .unwrap();
        assert!(bow.range > 0);
    }
}
