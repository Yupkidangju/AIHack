// ============================================================================
// [v2.39.0 R27-5] 고유 몬스터 데이터 (unique_mon_ext.rs)
// 원본: NetHack 3.6.7 monst.c 유니크 확장
// 유니크 몬스터 테이블, 특수 능력, 보스전 로직
// ============================================================================

/// [v2.39.0 R27-5] 유니크 몬스터 데이터
#[derive(Debug, Clone)]
pub struct UniqueMon {
    pub name: &'static str,
    pub symbol: char,
    pub level: i32,
    pub hp: i32,
    pub ac: i32,
    pub speed: i32,
    pub special: &'static str,
}

pub fn unique_monsters() -> Vec<UniqueMon> {
    vec![
        UniqueMon {
            name: "Medusa",
            symbol: '@',
            level: 20,
            hp: 120,
            ac: 2,
            speed: 12,
            special: "gaze petrify",
        },
        UniqueMon {
            name: "Vlad the Impaler",
            symbol: 'V',
            level: 14,
            hp: 100,
            ac: 1,
            speed: 18,
            special: "drain life",
        },
        UniqueMon {
            name: "Wizard of Yendor",
            symbol: '@',
            level: 30,
            hp: 250,
            ac: -8,
            speed: 12,
            special: "summon undead, steal AoY",
        },
        UniqueMon {
            name: "Demogorgon",
            symbol: '&',
            level: 57,
            hp: 600,
            ac: -10,
            speed: 15,
            special: "disease, summon demons",
        },
        UniqueMon {
            name: "Orcus",
            symbol: '&',
            level: 30,
            hp: 300,
            ac: -6,
            speed: 9,
            special: "wand of death",
        },
        UniqueMon {
            name: "Asmodeus",
            symbol: '&',
            level: 35,
            hp: 350,
            ac: -7,
            speed: 12,
            special: "cold attack",
        },
        UniqueMon {
            name: "Yeenoghu",
            symbol: '&',
            level: 28,
            hp: 280,
            ac: -5,
            speed: 18,
            special: "confuse, paralyze",
        },
        UniqueMon {
            name: "Death",
            symbol: '&',
            level: 100,
            hp: 999,
            ac: -20,
            speed: 24,
            special: "insta-kill",
        },
        UniqueMon {
            name: "Famine",
            symbol: '&',
            level: 100,
            hp: 999,
            ac: -20,
            speed: 24,
            special: "hunger drain",
        },
        UniqueMon {
            name: "Pestilence",
            symbol: '&',
            level: 100,
            hp: 999,
            ac: -20,
            speed: 24,
            special: "disease",
        },
    ]
}

pub fn find_unique(name: &str) -> Option<UniqueMon> {
    unique_monsters()
        .into_iter()
        .find(|m| m.name.eq_ignore_ascii_case(name))
}

pub fn is_rider(name: &str) -> bool {
    matches!(name, "Death" | "Famine" | "Pestilence")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count() {
        assert_eq!(unique_monsters().len(), 10);
    }

    #[test]
    fn test_find() {
        let m = find_unique("Medusa").unwrap();
        assert_eq!(m.level, 20);
    }

    #[test]
    fn test_riders() {
        assert!(is_rider("Death"));
        assert!(!is_rider("Medusa"));
    }

    #[test]
    fn test_demogorgon() {
        let d = find_unique("Demogorgon").unwrap();
        assert_eq!(d.hp, 600);
    }

    #[test]
    fn test_not_found() {
        assert!(find_unique("Pikachu").is_none());
    }
}
