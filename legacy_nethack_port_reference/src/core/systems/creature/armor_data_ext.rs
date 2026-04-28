// ============================================================================
// [v2.39.0 R27-1] 갑옷 데이터 (armor_data_ext.rs)
// 원본: NetHack 3.6.7 objects.c/do_wear.c 갑옷 데이터
// 갑옷 유형별 AC, 무게, 마법 속성, 장착 슬롯
// ============================================================================

/// [v2.39.0 R27-1] 장착 슬롯
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    Helm,
    Body,
    Cloak,
    Shield,
    Gloves,
    Boots,
}

/// [v2.39.0 R27-1] 갑옷 프로필
#[derive(Debug, Clone)]
pub struct ArmorProfile {
    pub name: &'static str,
    pub slot: ArmorSlot,
    pub base_ac: i32,
    pub weight: i32,
    pub magic_cancellation: i32, // MC 0~3
    pub special: Option<&'static str>,
}

pub fn armor_table() -> Vec<ArmorProfile> {
    vec![
        ArmorProfile {
            name: "plate mail",
            slot: ArmorSlot::Body,
            base_ac: 7,
            weight: 450,
            magic_cancellation: 2,
            special: None,
        },
        ArmorProfile {
            name: "chain mail",
            slot: ArmorSlot::Body,
            base_ac: 5,
            weight: 300,
            magic_cancellation: 1,
            special: None,
        },
        ArmorProfile {
            name: "leather armor",
            slot: ArmorSlot::Body,
            base_ac: 2,
            weight: 150,
            magic_cancellation: 0,
            special: None,
        },
        ArmorProfile {
            name: "silver dragon scale mail",
            slot: ArmorSlot::Body,
            base_ac: 9,
            weight: 40,
            magic_cancellation: 3,
            special: Some("reflection"),
        },
        ArmorProfile {
            name: "gray dragon scale mail",
            slot: ArmorSlot::Body,
            base_ac: 9,
            weight: 40,
            magic_cancellation: 3,
            special: Some("magic resistance"),
        },
        ArmorProfile {
            name: "helm of brilliance",
            slot: ArmorSlot::Helm,
            base_ac: 1,
            weight: 50,
            magic_cancellation: 0,
            special: Some("int+wis boost"),
        },
        ArmorProfile {
            name: "gauntlets of power",
            slot: ArmorSlot::Gloves,
            base_ac: 1,
            weight: 30,
            magic_cancellation: 0,
            special: Some("str 25"),
        },
        ArmorProfile {
            name: "speed boots",
            slot: ArmorSlot::Boots,
            base_ac: 1,
            weight: 20,
            magic_cancellation: 0,
            special: Some("very fast"),
        },
        ArmorProfile {
            name: "cloak of magic resistance",
            slot: ArmorSlot::Cloak,
            base_ac: 1,
            weight: 10,
            magic_cancellation: 3,
            special: Some("magic resistance"),
        },
        ArmorProfile {
            name: "shield of reflection",
            slot: ArmorSlot::Shield,
            base_ac: 2,
            weight: 50,
            magic_cancellation: 0,
            special: Some("reflection"),
        },
    ]
}

pub fn find_armor(name: &str) -> Option<ArmorProfile> {
    armor_table().into_iter().find(|a| a.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        assert_eq!(armor_table().len(), 10);
    }

    #[test]
    fn test_sdsm() {
        let a = find_armor("silver dragon scale mail").unwrap();
        assert_eq!(a.base_ac, 9);
        assert_eq!(a.special, Some("reflection"));
    }

    #[test]
    fn test_slot() {
        let a = find_armor("speed boots").unwrap();
        assert_eq!(a.slot, ArmorSlot::Boots);
    }

    #[test]
    fn test_mc() {
        let a = find_armor("cloak of magic resistance").unwrap();
        assert_eq!(a.magic_cancellation, 3);
    }

    #[test]
    fn test_not_found() {
        assert!(find_armor("xyz").is_none());
    }
}
