// ============================================================================
// [v2.41.0 R29-4] 마법 학교 (spell_school_ext.rs)
// 원본: NetHack 3.6.7 spell.c 학교 확장
// 마법 학교별 분류, 캐스팅 비용, 실패율
// ============================================================================

/// [v2.41.0 R29-4] 마법 학교
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellSchool {
    Attack,
    Healing,
    Divination,
    Enchantment,
    Clerical,
    Escape,
    Matter,
}

/// [v2.41.0 R29-4] 주문 데이터
#[derive(Debug, Clone)]
pub struct SpellInfo {
    pub name: &'static str,
    pub school: SpellSchool,
    pub level: i32, // 1~7
    pub energy_cost: i32,
    pub direction: bool, // true=방향 지정 필요
}

pub fn spell_book() -> Vec<SpellInfo> {
    vec![
        SpellInfo {
            name: "magic missile",
            school: SpellSchool::Attack,
            level: 2,
            energy_cost: 7,
            direction: true,
        },
        SpellInfo {
            name: "fireball",
            school: SpellSchool::Attack,
            level: 4,
            energy_cost: 16,
            direction: true,
        },
        SpellInfo {
            name: "cone of cold",
            school: SpellSchool::Attack,
            level: 4,
            energy_cost: 16,
            direction: true,
        },
        SpellInfo {
            name: "finger of death",
            school: SpellSchool::Attack,
            level: 7,
            energy_cost: 35,
            direction: true,
        },
        SpellInfo {
            name: "healing",
            school: SpellSchool::Healing,
            level: 1,
            energy_cost: 5,
            direction: false,
        },
        SpellInfo {
            name: "extra healing",
            school: SpellSchool::Healing,
            level: 3,
            energy_cost: 10,
            direction: false,
        },
        SpellInfo {
            name: "detect monsters",
            school: SpellSchool::Divination,
            level: 1,
            energy_cost: 5,
            direction: false,
        },
        SpellInfo {
            name: "identify",
            school: SpellSchool::Divination,
            level: 3,
            energy_cost: 13,
            direction: false,
        },
        SpellInfo {
            name: "remove curse",
            school: SpellSchool::Clerical,
            level: 3,
            energy_cost: 13,
            direction: false,
        },
        SpellInfo {
            name: "teleport away",
            school: SpellSchool::Escape,
            level: 6,
            energy_cost: 30,
            direction: true,
        },
        SpellInfo {
            name: "polymorph",
            school: SpellSchool::Matter,
            level: 6,
            energy_cost: 30,
            direction: true,
        },
        SpellInfo {
            name: "dig",
            school: SpellSchool::Matter,
            level: 5,
            energy_cost: 25,
            direction: true,
        },
    ]
}

/// [v2.41.0 R29-4] 실패율 (원본: percent_success)
pub fn cast_failure_rate(spell_level: i32, caster_int: i32, skill_level: i32) -> i32 {
    let base = spell_level * 15 - caster_int * 2 - skill_level * 10;
    base.clamp(0, 95)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_count() {
        assert_eq!(spell_book().len(), 12);
    }

    #[test]
    fn test_high_level() {
        let fod = spell_book()
            .into_iter()
            .find(|s| s.name == "finger of death")
            .unwrap();
        assert_eq!(fod.level, 7);
        assert_eq!(fod.energy_cost, 35);
    }

    #[test]
    fn test_failure_easy() {
        let rate = cast_failure_rate(1, 18, 3);
        assert_eq!(rate, 0); // 15 - 36 - 30 < 0 → 0
    }

    #[test]
    fn test_failure_hard() {
        let rate = cast_failure_rate(7, 10, 0);
        assert!(rate > 50);
    }

    #[test]
    fn test_schools() {
        let healings: Vec<_> = spell_book()
            .into_iter()
            .filter(|s| s.school == SpellSchool::Healing)
            .collect();
        assert_eq!(healings.len(), 2);
    }
}
