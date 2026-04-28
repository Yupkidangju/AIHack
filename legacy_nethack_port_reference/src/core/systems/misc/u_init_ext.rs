// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-9] 초기 장비 확장 모듈 (u_init_ext.rs)
// 원본: NetHack 3.6.7 u_init.c (역할별 초기 장비 + 종족 치환)
// ============================================================================

// =============================================================================
// [1] 초기 장비 아이템 구조체 (원본: struct trobj)
// =============================================================================

/// [v2.22.0 R34-9] 초기 인벤토리 엔트리 (원본: struct trobj)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialItem {
    /// 아이템 타입 ID (0=미정의)
    pub item_type: i32,
    /// 강화치 (i8::MAX=미정의)
    pub enchantment: i8,
    /// 아이템 클래스
    pub item_class: char,
    /// 수량
    pub quantity: u8,
    /// 축복 상태 (0=저주, 1=축복, 2=미정)
    pub bless_state: u8,
}

pub const UNDEF_TYP: i32 = 0;
pub const UNDEF_SPE: i8 = 127; // '\177'
pub const UNDEF_BLESS: u8 = 2;

// =============================================================================
// [2] 종족별 아이템 치환 (원본: inv_subs[])
// =============================================================================

/// [v2.22.0 R34-9] 종족별 아이템 치환 규칙
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventorySubstitution {
    /// 종족 몬스터 ID (-1 = 종료 마커)
    pub race_pm: i32,
    /// 원래 아이템
    pub original: i32,
    /// 치환 아이템
    pub substitute: i32,
}

/// [v2.22.0 R34-9] 종족별 치환 테이블 (원본: inv_subs[])
/// 종족 인덱스: 0=Elf, 1=Orc, 2=Dwarf, 3=Gnome
pub const RACE_ELF: i32 = 0;
pub const RACE_ORC: i32 = 1;
pub const RACE_DWARF: i32 = 2;
pub const RACE_GNOME: i32 = 3;

/// [v2.22.0 R34-9] 아이템 ID 상수 (원본: objects.h 기반)
pub mod items {
    pub const DAGGER: i32 = 1;
    pub const ELVEN_DAGGER: i32 = 2;
    pub const ORCISH_DAGGER: i32 = 3;
    pub const SPEAR: i32 = 4;
    pub const ELVEN_SPEAR: i32 = 5;
    pub const ORCISH_SPEAR: i32 = 6;
    pub const DWARVISH_SPEAR: i32 = 7;
    pub const SHORT_SWORD: i32 = 8;
    pub const ELVEN_SHORT_SWORD: i32 = 9;
    pub const ORCISH_SHORT_SWORD: i32 = 10;
    pub const DWARVISH_SHORT_SWORD: i32 = 11;
    pub const BOW: i32 = 12;
    pub const ELVEN_BOW: i32 = 13;
    pub const ORCISH_BOW: i32 = 14;
    pub const CROSSBOW: i32 = 15;
    pub const ARROW: i32 = 16;
    pub const ELVEN_ARROW: i32 = 17;
    pub const ORCISH_ARROW: i32 = 18;
    pub const CROSSBOW_BOLT: i32 = 19;
    pub const HELMET: i32 = 20;
    pub const ELVEN_LEATHER_HELM: i32 = 21;
    pub const ORCISH_HELM: i32 = 22;
    pub const DWARVISH_IRON_HELM: i32 = 23;
    pub const SMALL_SHIELD: i32 = 24;
    pub const ORCISH_SHIELD: i32 = 25;
    pub const CLOAK_OF_DISPLACEMENT: i32 = 26;
    pub const ELVEN_CLOAK: i32 = 27;
    pub const RING_MAIL: i32 = 28;
    pub const ORCISH_RING_MAIL: i32 = 29;
    pub const CHAIN_MAIL: i32 = 30;
    pub const ORCISH_CHAIN_MAIL: i32 = 31;
    pub const CRAM_RATION: i32 = 32;
    pub const LEMBAS_WAFER: i32 = 33;
    pub const TRIPE_RATION: i32 = 34;
}

/// [v2.22.0 R34-9] 종족별 치환 테이블 (원본: inv_subs[])
pub static INVENTORY_SUBS: &[InventorySubstitution] = &[
    // 엘프
    InventorySubstitution {
        race_pm: RACE_ELF,
        original: items::DAGGER,
        substitute: items::ELVEN_DAGGER,
    },
    InventorySubstitution {
        race_pm: RACE_ELF,
        original: items::SPEAR,
        substitute: items::ELVEN_SPEAR,
    },
    InventorySubstitution {
        race_pm: RACE_ELF,
        original: items::SHORT_SWORD,
        substitute: items::ELVEN_SHORT_SWORD,
    },
    InventorySubstitution {
        race_pm: RACE_ELF,
        original: items::BOW,
        substitute: items::ELVEN_BOW,
    },
    InventorySubstitution {
        race_pm: RACE_ELF,
        original: items::ARROW,
        substitute: items::ELVEN_ARROW,
    },
    InventorySubstitution {
        race_pm: RACE_ELF,
        original: items::HELMET,
        substitute: items::ELVEN_LEATHER_HELM,
    },
    InventorySubstitution {
        race_pm: RACE_ELF,
        original: items::CLOAK_OF_DISPLACEMENT,
        substitute: items::ELVEN_CLOAK,
    },
    InventorySubstitution {
        race_pm: RACE_ELF,
        original: items::CRAM_RATION,
        substitute: items::LEMBAS_WAFER,
    },
    // 오크
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::DAGGER,
        substitute: items::ORCISH_DAGGER,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::SPEAR,
        substitute: items::ORCISH_SPEAR,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::SHORT_SWORD,
        substitute: items::ORCISH_SHORT_SWORD,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::BOW,
        substitute: items::ORCISH_BOW,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::ARROW,
        substitute: items::ORCISH_ARROW,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::HELMET,
        substitute: items::ORCISH_HELM,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::SMALL_SHIELD,
        substitute: items::ORCISH_SHIELD,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::RING_MAIL,
        substitute: items::ORCISH_RING_MAIL,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::CHAIN_MAIL,
        substitute: items::ORCISH_CHAIN_MAIL,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::CRAM_RATION,
        substitute: items::TRIPE_RATION,
    },
    InventorySubstitution {
        race_pm: RACE_ORC,
        original: items::LEMBAS_WAFER,
        substitute: items::TRIPE_RATION,
    },
    // 드워프
    InventorySubstitution {
        race_pm: RACE_DWARF,
        original: items::SPEAR,
        substitute: items::DWARVISH_SPEAR,
    },
    InventorySubstitution {
        race_pm: RACE_DWARF,
        original: items::SHORT_SWORD,
        substitute: items::DWARVISH_SHORT_SWORD,
    },
    InventorySubstitution {
        race_pm: RACE_DWARF,
        original: items::HELMET,
        substitute: items::DWARVISH_IRON_HELM,
    },
    InventorySubstitution {
        race_pm: RACE_DWARF,
        original: items::LEMBAS_WAFER,
        substitute: items::CRAM_RATION,
    },
    // 노움
    InventorySubstitution {
        race_pm: RACE_GNOME,
        original: items::BOW,
        substitute: items::CROSSBOW,
    },
    InventorySubstitution {
        race_pm: RACE_GNOME,
        original: items::ARROW,
        substitute: items::CROSSBOW_BOLT,
    },
];

/// [v2.22.0 R34-9] 종족에 따라 아이템 치환 적용
/// (원본: u_init.c 813-820행)
pub fn apply_race_substitution(race_pm: i32, item_type: i32) -> i32 {
    for sub in INVENTORY_SUBS {
        if sub.race_pm == race_pm && sub.original == item_type {
            return sub.substitute;
        }
    }
    item_type // 치환 없음
}

/// [v2.22.0 R34-9] 초기 인벤토리 목록에 종족 치환 일괄 적용
pub fn apply_race_subs_to_inventory(race_pm: i32, items: &mut [InitialItem]) {
    for item in items.iter_mut() {
        item.item_type = apply_race_substitution(race_pm, item.item_type);
    }
}

// =============================================================================
// [3] 초기 소지금 계산 (원본: u_init 697, 751, 769행 등)
// =============================================================================

/// [v2.22.0 R34-9] 역할별 초기 소지금
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleId {
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

/// [v2.22.0 R34-9] 역할별 초기 금화 계산 (원본: u_init의 u.umoney0 설정)
pub fn initial_gold(role: RoleId, rng_rn1_1000_1001: i32, rng_rnd_1000: i32) -> i32 {
    match role {
        RoleId::Healer => rng_rn1_1000_1001, // rn1(1000, 1001) → 1001..2000
        RoleId::Rogue => 0,                  // 로그는 금화 0으로 시작
        RoleId::Tourist => rng_rnd_1000,     // rnd(1000) → 1..1000
        _ => 0,                              // 다른 역할은 기본 0 (디버그 모드에서 추가될 수 있음)
    }
}

// =============================================================================
// [4] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elf_dagger_to_elven() {
        assert_eq!(
            apply_race_substitution(RACE_ELF, items::DAGGER),
            items::ELVEN_DAGGER
        );
    }

    #[test]
    fn test_orc_ring_mail() {
        assert_eq!(
            apply_race_substitution(RACE_ORC, items::RING_MAIL),
            items::ORCISH_RING_MAIL
        );
    }

    #[test]
    fn test_dwarf_helmet() {
        assert_eq!(
            apply_race_substitution(RACE_DWARF, items::HELMET),
            items::DWARVISH_IRON_HELM
        );
    }

    #[test]
    fn test_gnome_bow_to_crossbow() {
        assert_eq!(
            apply_race_substitution(RACE_GNOME, items::BOW),
            items::CROSSBOW
        );
    }

    #[test]
    fn test_no_substitution() {
        // 엘프에겐 RING_MAIL 치환이 없으므로 원본 반환
        assert_eq!(
            apply_race_substitution(RACE_ELF, items::RING_MAIL),
            items::RING_MAIL
        );
    }

    #[test]
    fn test_batch_substitution() {
        let mut inv = vec![
            InitialItem {
                item_type: items::DAGGER,
                enchantment: 1,
                item_class: ')',
                quantity: 1,
                bless_state: UNDEF_BLESS,
            },
            InitialItem {
                item_type: items::BOW,
                enchantment: 1,
                item_class: ')',
                quantity: 1,
                bless_state: UNDEF_BLESS,
            },
        ];
        apply_race_subs_to_inventory(RACE_ELF, &mut inv);
        assert_eq!(inv[0].item_type, items::ELVEN_DAGGER);
        assert_eq!(inv[1].item_type, items::ELVEN_BOW);
    }

    #[test]
    fn test_orc_lembas_to_tripe() {
        assert_eq!(
            apply_race_substitution(RACE_ORC, items::LEMBAS_WAFER),
            items::TRIPE_RATION
        );
    }

    #[test]
    fn test_initial_gold_healer() {
        assert_eq!(initial_gold(RoleId::Healer, 1500, 0), 1500);
    }

    #[test]
    fn test_initial_gold_rogue() {
        assert_eq!(initial_gold(RoleId::Rogue, 0, 500), 0);
    }

    #[test]
    fn test_initial_gold_tourist() {
        assert_eq!(initial_gold(RoleId::Tourist, 0, 750), 750);
    }
}
