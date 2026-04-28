// ============================================================================
// [v2.28.0 Phase 92-3] 역할/직업 확장 (role_phase92_ext.rs)
// 원본: NetHack 3.6.7 src/role.c L400-1000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 직업별 고유 능력 — role_abilities (role.c L400-700)
// =============================================================================

/// [v2.28.0 92-3] 직업 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Archeologist,
    Barbarian,
    Caveperson,
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

/// [v2.28.0 92-3] 종족 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Race {
    Human,
    Elf,
    Dwarf,
    Gnome,
    Orc,
}

/// [v2.28.0 92-3] 직업별 기본 능력치
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleAbilities {
    pub str_base: i32,
    pub dex_base: i32,
    pub con_base: i32,
    pub int_base: i32,
    pub wis_base: i32,
    pub cha_base: i32,
    pub hp_bonus: i32,
    pub energy_bonus: i32,
    pub special_abilities: Vec<String>,
}

/// [v2.28.0 92-3] 직업별 기본 능력치 반환
/// 원본: role.c init_role() 내 직업별 분기
pub fn role_base_abilities(role: Role) -> RoleAbilities {
    match role {
        Role::Archeologist => RoleAbilities {
            str_base: 7,
            dex_base: 10,
            con_base: 7,
            int_base: 10,
            wis_base: 7,
            cha_base: 7,
            hp_bonus: 0,
            energy_bonus: 5,
            special_abilities: vec![
                "속도 부츠 시작".to_string(),
                "곡괭이 시작".to_string(),
                "탐지 스킬".to_string(),
            ],
        },
        Role::Barbarian => RoleAbilities {
            str_base: 16,
            dex_base: 15,
            con_base: 16,
            int_base: 7,
            wis_base: 7,
            cha_base: 6,
            hp_bonus: 10,
            energy_bonus: 0,
            special_abilities: vec!["독 저항".to_string(), "2핸드 무기 숙련".to_string()],
        },
        Role::Caveperson => RoleAbilities {
            str_base: 10,
            dex_base: 7,
            con_base: 10,
            int_base: 7,
            wis_base: 7,
            cha_base: 7,
            hp_bonus: 5,
            energy_bonus: 0,
            special_abilities: vec!["곤봉 시작".to_string(), "투척 숙련".to_string()],
        },
        Role::Healer => RoleAbilities {
            str_base: 7,
            dex_base: 7,
            con_base: 11,
            int_base: 11,
            wis_base: 13,
            cha_base: 7,
            hp_bonus: 0,
            energy_bonus: 10,
            special_abilities: vec![
                "독 저항".to_string(),
                "치료 스킬".to_string(),
                "치료 키트 시작".to_string(),
            ],
        },
        Role::Knight => RoleAbilities {
            str_base: 13,
            dex_base: 7,
            con_base: 13,
            int_base: 7,
            wis_base: 14,
            cha_base: 17,
            hp_bonus: 5,
            energy_bonus: 5,
            special_abilities: vec![
                "랜스 시작".to_string(),
                "기마 숙련".to_string(),
                "코드 오브 아너".to_string(),
            ],
        },
        Role::Monk => RoleAbilities {
            str_base: 10,
            dex_base: 8,
            con_base: 7,
            int_base: 7,
            wis_base: 8,
            cha_base: 7,
            hp_bonus: 3,
            energy_bonus: 8,
            special_abilities: vec![
                "맨손 공격 보너스".to_string(),
                "AC 보너스".to_string(),
                "속도 보너스".to_string(),
                "채식주의".to_string(),
            ],
        },
        Role::Priest => RoleAbilities {
            str_base: 7,
            dex_base: 7,
            con_base: 7,
            int_base: 7,
            wis_base: 10,
            cha_base: 7,
            hp_bonus: 0,
            energy_bonus: 10,
            special_abilities: vec![
                "신앙심".to_string(),
                "저주 탐지".to_string(),
                "둔기 숙련".to_string(),
            ],
        },
        Role::Ranger => RoleAbilities {
            str_base: 13,
            dex_base: 13,
            con_base: 13,
            int_base: 9,
            wis_base: 13,
            cha_base: 7,
            hp_bonus: 3,
            energy_bonus: 3,
            special_abilities: vec![
                "활 숙련".to_string(),
                "다연사".to_string(),
                "탐지 보너스".to_string(),
            ],
        },
        Role::Rogue => RoleAbilities {
            str_base: 7,
            dex_base: 10,
            con_base: 7,
            int_base: 7,
            wis_base: 7,
            cha_base: 10,
            hp_bonus: 0,
            energy_bonus: 3,
            special_abilities: vec![
                "뒤찌르기".to_string(),
                "자물쇠 따기".to_string(),
                "훔치기".to_string(),
                "독 저항".to_string(),
            ],
        },
        Role::Samurai => RoleAbilities {
            str_base: 10,
            dex_base: 8,
            con_base: 7,
            int_base: 7,
            wis_base: 7,
            cha_base: 7,
            hp_bonus: 5,
            energy_bonus: 0,
            special_abilities: vec![
                "카타나 시작".to_string(),
                "이도류".to_string(),
                "속도 보너스".to_string(),
            ],
        },
        Role::Tourist => RoleAbilities {
            str_base: 7,
            dex_base: 10,
            con_base: 7,
            int_base: 10,
            wis_base: 7,
            cha_base: 10,
            hp_bonus: 0,
            energy_bonus: 3,
            special_abilities: vec![
                "카메라 시작".to_string(),
                "다트 시작".to_string(),
                "금화 보너스".to_string(),
            ],
        },
        Role::Valkyrie => RoleAbilities {
            str_base: 10,
            dex_base: 7,
            con_base: 10,
            int_base: 7,
            wis_base: 7,
            cha_base: 7,
            hp_bonus: 8,
            energy_bonus: 0,
            special_abilities: vec![
                "냉기 저항".to_string(),
                "도끼 숙련".to_string(),
                "독 저항".to_string(),
            ],
        },
        Role::Wizard => RoleAbilities {
            str_base: 7,
            dex_base: 7,
            con_base: 7,
            int_base: 10,
            wis_base: 7,
            cha_base: 7,
            hp_bonus: 0,
            energy_bonus: 15,
            special_abilities: vec![
                "마법 숙련".to_string(),
                "지팡이 충전 보너스".to_string(),
                "텔레포트 제어".to_string(),
            ],
        },
    }
}

// =============================================================================
// [2] 종족별 보정 — race_modifiers (role.c L700-900)
// =============================================================================

/// [v2.28.0 92-3] 종족별 능력치 보정
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaceModifiers {
    pub str_mod: i32,
    pub dex_mod: i32,
    pub con_mod: i32,
    pub int_mod: i32,
    pub wis_mod: i32,
    pub cha_mod: i32,
    pub infravision: bool,
    pub special_traits: Vec<String>,
}

/// [v2.28.0 92-3] 종족별 보정값
pub fn race_modifiers(race: Race) -> RaceModifiers {
    match race {
        Race::Human => RaceModifiers {
            str_mod: 0,
            dex_mod: 0,
            con_mod: 0,
            int_mod: 0,
            wis_mod: 0,
            cha_mod: 0,
            infravision: false,
            special_traits: vec!["범용적".to_string()],
        },
        Race::Elf => RaceModifiers {
            str_mod: -1,
            dex_mod: 1,
            con_mod: -1,
            int_mod: 1,
            wis_mod: 1,
            cha_mod: 1,
            infravision: true,
            special_traits: vec!["수면 저항".to_string(), "투명 감지".to_string()],
        },
        Race::Dwarf => RaceModifiers {
            str_mod: 1,
            dex_mod: 0,
            con_mod: 1,
            int_mod: -1,
            wis_mod: 0,
            cha_mod: -1,
            infravision: true,
            special_traits: vec!["채굴 보너스".to_string(), "독 저항".to_string()],
        },
        Race::Gnome => RaceModifiers {
            str_mod: -1,
            dex_mod: 1,
            con_mod: 0,
            int_mod: 1,
            wis_mod: 0,
            cha_mod: -1,
            infravision: true,
            special_traits: vec!["소형".to_string(), "채굴 보너스".to_string()],
        },
        Race::Orc => RaceModifiers {
            str_mod: 1,
            dex_mod: 0,
            con_mod: 1,
            int_mod: -1,
            wis_mod: -1,
            cha_mod: -2,
            infravision: true,
            special_traits: vec!["독 저항".to_string(), "오크 공감".to_string()],
        },
    }
}

// =============================================================================
// [3] 직업-종족 호환성 — role_race_compat (role.c L900-1000)
// =============================================================================

/// [v2.28.0 92-3] 직업-종족 호환성 확인
pub fn is_role_race_compatible(role: Role, race: Race) -> bool {
    match role {
        Role::Archeologist => matches!(race, Race::Human | Race::Dwarf | Race::Gnome),
        Role::Barbarian => matches!(race, Race::Human | Race::Orc),
        Role::Caveperson => matches!(race, Race::Human | Race::Dwarf | Race::Gnome),
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

// =============================================================================
// [4] 초기 능력치 통합 계산 — combined_stats
// =============================================================================

/// [v2.28.0 92-3] 초기 능력치 통합 계산
pub fn calculate_initial_stats(role: Role, race: Race) -> [i32; 6] {
    let base = role_base_abilities(role);
    let mods = race_modifiers(race);

    [
        (base.str_base + mods.str_mod).max(3).min(25),
        (base.dex_base + mods.dex_mod).max(3).min(25),
        (base.con_base + mods.con_mod).max(3).min(25),
        (base.int_base + mods.int_mod).max(3).min(25),
        (base.wis_base + mods.wis_mod).max(3).min(25),
        (base.cha_base + mods.cha_mod).max(3).min(25),
    ]
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barbarian_abilities() {
        let ab = role_base_abilities(Role::Barbarian);
        assert_eq!(ab.str_base, 16);
        assert_eq!(ab.hp_bonus, 10);
        assert!(ab.special_abilities.iter().any(|s| s.contains("독")));
    }

    #[test]
    fn test_wizard_abilities() {
        let ab = role_base_abilities(Role::Wizard);
        assert_eq!(ab.energy_bonus, 15);
        assert!(ab.special_abilities.iter().any(|s| s.contains("마법")));
    }

    #[test]
    fn test_elf_modifiers() {
        let mods = race_modifiers(Race::Elf);
        assert_eq!(mods.dex_mod, 1);
        assert_eq!(mods.str_mod, -1);
        assert!(mods.infravision);
    }

    #[test]
    fn test_knight_human_only() {
        assert!(is_role_race_compatible(Role::Knight, Race::Human));
        assert!(!is_role_race_compatible(Role::Knight, Race::Elf));
        assert!(!is_role_race_compatible(Role::Knight, Race::Orc));
    }

    #[test]
    fn test_wizard_multi_race() {
        assert!(is_role_race_compatible(Role::Wizard, Race::Human));
        assert!(is_role_race_compatible(Role::Wizard, Race::Elf));
        assert!(is_role_race_compatible(Role::Wizard, Race::Gnome));
        assert!(is_role_race_compatible(Role::Wizard, Race::Orc));
    }

    #[test]
    fn test_combined_stats_bounds() {
        let stats = calculate_initial_stats(Role::Wizard, Race::Gnome);
        for s in &stats {
            assert!(*s >= 3 && *s <= 25);
        }
    }

    #[test]
    fn test_combined_stats_elf_wizard() {
        let stats = calculate_initial_stats(Role::Wizard, Race::Elf);
        // STR: 7 + (-1) = 6, DEX: 7+1=8, INT: 10+1=11
        assert_eq!(stats[0], 6); // STR
        assert_eq!(stats[1], 8); // DEX
        assert_eq!(stats[3], 11); // INT
    }

    #[test]
    fn test_all_roles_have_abilities() {
        let roles = [
            Role::Archeologist,
            Role::Barbarian,
            Role::Caveperson,
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
            let ab = role_base_abilities(*role);
            assert!(
                !ab.special_abilities.is_empty(),
                "{:?}에 특수능력 필요",
                role
            );
        }
    }

    #[test]
    fn test_race_traits() {
        let races = [Race::Human, Race::Elf, Race::Dwarf, Race::Gnome, Race::Orc];
        for race in &races {
            let mods = race_modifiers(*race);
            assert!(!mods.special_traits.is_empty());
        }
    }
}
