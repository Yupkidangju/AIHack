// ============================================================================
// [v2.26.0 R14-1] 역할/종족 시스템 (role_ext.rs)
// 원본: NetHack 3.6.7 role.c (2,540줄)
// 13역할, 5종족, 성별/정렬, 시작 장비, 스킬 적성
// ============================================================================

// =============================================================================
// [1] 역할 정의 (원본: role.c roles[])
// =============================================================================

/// [v2.26.0 R14-1] 역할
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Archeologist,
    Barbarian,
    Caveman,
    Healer,
    Knight,
    Monk,
    Priest,
    Ranger,
    Rogue,
    Samurai,
    Tourist,
    Valkyrie,
    Wizard,
}

/// [v2.26.0 R14-1] 종족
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    Gnome,
    Orc,
}

/// [v2.26.0 R14-1] 정렬
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Lawful,
    Neutral,
    Chaotic,
}

/// [v2.26.0 R14-1] 역할-종족 호환성 (원본: role.c ok_race)
pub fn valid_race_for_role(role: Role, race: Race) -> bool {
    match role {
        Role::Archeologist => matches!(race, Race::Human | Race::Dwarf | Race::Gnome),
        Role::Barbarian => matches!(race, Race::Human | Race::Orc),
        Role::Caveman => matches!(race, Race::Human | Race::Dwarf | Race::Gnome),
        Role::Healer => matches!(race, Race::Human | Race::Gnome),
        Role::Knight => matches!(race, Race::Human),
        Role::Monk => matches!(race, Race::Human),
        Role::Priest => matches!(race, Race::Human | Race::Elf),
        Role::Ranger => matches!(race, Race::Human | Race::Elf | Race::Gnome | Race::Orc),
        Role::Rogue => matches!(race, Race::Human | Race::Orc),
        Role::Samurai => matches!(race, Race::Human),
        Role::Tourist => matches!(race, Race::Human),
        Role::Valkyrie => matches!(race, Race::Human | Race::Dwarf),
        Role::Wizard => matches!(race, Race::Human | Race::Elf | Race::Gnome | Race::Orc),
    }
}

/// [v2.26.0 R14-1] 역할 허용 정렬
pub fn valid_alignment_for_role(role: Role) -> Vec<Alignment> {
    match role {
        Role::Archeologist => vec![Alignment::Lawful, Alignment::Neutral],
        Role::Barbarian => vec![Alignment::Neutral, Alignment::Chaotic],
        Role::Caveman => vec![Alignment::Lawful, Alignment::Neutral],
        Role::Healer => vec![Alignment::Neutral],
        Role::Knight => vec![Alignment::Lawful],
        Role::Monk => vec![Alignment::Lawful, Alignment::Neutral, Alignment::Chaotic],
        Role::Priest => vec![Alignment::Lawful, Alignment::Neutral, Alignment::Chaotic],
        Role::Ranger => vec![Alignment::Neutral, Alignment::Chaotic],
        Role::Rogue => vec![Alignment::Chaotic],
        Role::Samurai => vec![Alignment::Lawful],
        Role::Tourist => vec![Alignment::Neutral],
        Role::Valkyrie => vec![Alignment::Lawful, Alignment::Neutral],
        Role::Wizard => vec![Alignment::Neutral, Alignment::Chaotic],
    }
}

// =============================================================================
// [2] 시작 능력치 (원본: role.c init_attr)
// =============================================================================

/// [v2.26.0 R14-1] 기본 능력치
#[derive(Debug, Clone)]
pub struct BaseStats {
    pub str_: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
}

/// [v2.26.0 R14-1] 역할별 시작 능력치 (원본: init_attr 배열)
pub fn base_stats_for_role(role: Role) -> BaseStats {
    match role {
        Role::Archeologist => BaseStats {
            str_: 7,
            dex: 10,
            con: 7,
            int: 7,
            wis: 7,
            cha: 7,
        },
        Role::Barbarian => BaseStats {
            str_: 16,
            dex: 15,
            con: 16,
            int: 7,
            wis: 7,
            cha: 6,
        },
        Role::Caveman => BaseStats {
            str_: 10,
            dex: 7,
            con: 10,
            int: 7,
            wis: 7,
            cha: 7,
        },
        Role::Healer => BaseStats {
            str_: 7,
            dex: 7,
            con: 11,
            int: 7,
            wis: 14,
            cha: 7,
        },
        Role::Knight => BaseStats {
            str_: 13,
            dex: 7,
            con: 14,
            int: 7,
            wis: 7,
            cha: 17,
        },
        Role::Monk => BaseStats {
            str_: 10,
            dex: 8,
            con: 7,
            int: 7,
            wis: 8,
            cha: 7,
        },
        Role::Priest => BaseStats {
            str_: 7,
            dex: 7,
            con: 7,
            int: 7,
            wis: 10,
            cha: 7,
        },
        Role::Ranger => BaseStats {
            str_: 13,
            dex: 13,
            con: 13,
            int: 13,
            wis: 13,
            cha: 7,
        },
        Role::Rogue => BaseStats {
            str_: 7,
            dex: 10,
            con: 7,
            int: 7,
            wis: 7,
            cha: 7,
        },
        Role::Samurai => BaseStats {
            str_: 10,
            dex: 8,
            con: 10,
            int: 7,
            wis: 7,
            cha: 7,
        },
        Role::Tourist => BaseStats {
            str_: 7,
            dex: 10,
            con: 7,
            int: 10,
            wis: 7,
            cha: 10,
        },
        Role::Valkyrie => BaseStats {
            str_: 10,
            dex: 7,
            con: 10,
            int: 7,
            wis: 7,
            cha: 7,
        },
        Role::Wizard => BaseStats {
            str_: 7,
            dex: 7,
            con: 7,
            int: 10,
            wis: 7,
            cha: 7,
        },
    }
}

/// [v2.26.0 R14-1] 종족 스탯 보정
pub fn race_stat_bonus(race: Race) -> BaseStats {
    match race {
        Race::Human => BaseStats {
            str_: 0,
            dex: 0,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        },
        Race::Elf => BaseStats {
            str_: -1,
            dex: 1,
            con: -1,
            int: 1,
            wis: 1,
            cha: 1,
        },
        Race::Dwarf => BaseStats {
            str_: 1,
            dex: 0,
            con: 1,
            int: 0,
            wis: 1,
            cha: -1,
        },
        Race::Gnome => BaseStats {
            str_: -1,
            dex: 1,
            con: 0,
            int: 1,
            wis: 0,
            cha: -1,
        },
        Race::Orc => BaseStats {
            str_: 1,
            dex: 0,
            con: 1,
            int: -1,
            wis: -1,
            cha: -2,
        },
    }
}

/// [v2.26.0 R14-1] 최종 시작 스탯 계산
pub fn calc_starting_stats(role: Role, race: Race) -> BaseStats {
    let base = base_stats_for_role(role);
    let bonus = race_stat_bonus(race);
    BaseStats {
        str_: (base.str_ + bonus.str_).max(3),
        dex: (base.dex + bonus.dex).max(3),
        con: (base.con + bonus.con).max(3),
        int: (base.int + bonus.int).max(3),
        wis: (base.wis + bonus.wis).max(3),
        cha: (base.cha + bonus.cha).max(3),
    }
}

// =============================================================================
// [3] 스킬 적성 (원본: role.c skill_init)
// =============================================================================

/// [v2.26.0 R14-1] 스킬 적성 등급
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SkillAptitude {
    Restricted,
    Unskilled,
    Basic,
    Skilled,
    Expert,
    Master,
    GrandMaster,
}

/// [v2.26.0 R14-1] 역할별 검술 적성
pub fn melee_aptitude(role: Role) -> SkillAptitude {
    match role {
        Role::Barbarian | Role::Knight | Role::Samurai | Role::Valkyrie => SkillAptitude::Expert,
        Role::Ranger | Role::Rogue | Role::Caveman => SkillAptitude::Skilled,
        Role::Monk => SkillAptitude::GrandMaster, // 무기 대신 격투
        Role::Wizard | Role::Healer | Role::Tourist => SkillAptitude::Basic,
        _ => SkillAptitude::Skilled,
    }
}

/// [v2.26.0 R14-1] 역할별 마법 적성
pub fn spell_aptitude(role: Role) -> SkillAptitude {
    match role {
        Role::Wizard => SkillAptitude::Expert,
        Role::Priest | Role::Healer | Role::Monk => SkillAptitude::Skilled,
        Role::Ranger | Role::Knight | Role::Archeologist => SkillAptitude::Basic,
        Role::Barbarian | Role::Caveman | Role::Valkyrie => SkillAptitude::Restricted,
        _ => SkillAptitude::Basic,
    }
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_race() {
        assert!(valid_race_for_role(Role::Knight, Race::Human));
        assert!(!valid_race_for_role(Role::Knight, Race::Orc));
    }

    #[test]
    fn test_valid_alignment() {
        assert_eq!(
            valid_alignment_for_role(Role::Rogue),
            vec![Alignment::Chaotic]
        );
        assert_eq!(
            valid_alignment_for_role(Role::Healer),
            vec![Alignment::Neutral]
        );
    }

    #[test]
    fn test_starting_stats() {
        let stats = calc_starting_stats(Role::Barbarian, Race::Human);
        assert_eq!(stats.str_, 16);
        assert_eq!(stats.con, 16);
    }

    #[test]
    fn test_race_bonus_elf() {
        let stats = calc_starting_stats(Role::Wizard, Race::Elf);
        assert_eq!(stats.int, 11); // 10 + 1
        assert_eq!(stats.cha, 8); // 7 + 1
    }

    #[test]
    fn test_melee_aptitude() {
        assert_eq!(melee_aptitude(Role::Barbarian), SkillAptitude::Expert);
        assert_eq!(melee_aptitude(Role::Wizard), SkillAptitude::Basic);
    }

    #[test]
    fn test_spell_aptitude() {
        assert_eq!(spell_aptitude(Role::Wizard), SkillAptitude::Expert);
        assert_eq!(spell_aptitude(Role::Barbarian), SkillAptitude::Restricted);
    }

    #[test]
    fn test_all_roles_have_valid_race() {
        let roles = [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveman,
            Role::Healer,
            Role::Knight,
            Role::Monk,
            Role::Priest,
            Role::Ranger,
            Role::Rogue,
            Role::Samurai,
            Role::Tourist,
            Role::Valkyrie,
            Role::Wizard,
        ];
        for role in &roles {
            assert!(valid_race_for_role(*role, Race::Human)); // 모든 역할은 인간 가능
        }
    }

    #[test]
    fn test_min_stat_floor() {
        let stats = calc_starting_stats(Role::Wizard, Race::Orc);
        assert!(stats.cha >= 3); // 7 + (-2) = 5 >= 3
    }
}
