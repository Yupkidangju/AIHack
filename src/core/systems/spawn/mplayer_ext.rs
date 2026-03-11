// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-8] 플레이어 몬스터 장비 확장 모듈 (mplayer_ext.rs)
// 원본: NetHack 3.6.7 mplayer.c (역할별 장비 선정 순수 로직)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 상수 (원본: mplayer.c, objclass.h)
// =============================================================================

/// 장비 없음 표시 (원본: STRANGE_OBJECT = 0)
pub const NO_ITEM: i32 = 0;

// =============================================================================
// [2] 역할별 장비 결정 (원본: mk_mplayer, 137-255행)
// =============================================================================

/// [v2.22.0 R34-8] 플레이어 몬스터의 핵심 장비 4종
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MplayerEquipment {
    /// 무기 아이템 ID (NO_ITEM이면 없음)
    pub weapon: i32,
    /// 갑옷 아이템 ID
    pub armor: i32,
    /// 망토 아이템 ID
    pub cloak: i32,
    /// 투구 아이템 ID
    pub helm: i32,
    /// 방패 아이템 ID
    pub shield: i32,
}

/// [v2.22.0 R34-8] 역할 인덱스 (원본: PM_ARCHEOLOGIST 등)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MplayerRole {
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

/// [v2.22.0 R34-8] 역할별 무기/갑옷/망토/투구/방패 결정
/// (원본: mk_mplayer 167-255행의 switch문)
/// `default_weapon`, `default_armor` 등은 상위에서 rnd_class로 결정한 기본값
pub fn calc_mplayer_equipment(
    role: MplayerRole,
    default_weapon: i32,
    default_armor: i32,
    default_cloak: i32,
    default_helm: i32,
    default_shield: i32,
    rng: &mut NetHackRng,
) -> MplayerEquipment {
    let mut eq = MplayerEquipment {
        weapon: default_weapon,
        armor: default_armor,
        cloak: default_cloak,
        helm: default_helm,
        shield: default_shield,
    };

    match role {
        MplayerRole::Archeologist => {
            // 50% 확률로 채찍 지급
            if rng.rn2(2) != 0 {
                eq.weapon = 100; // BULLWHIP
            }
        }
        MplayerRole::Barbarian => {
            // 50% 양손검 또는 전투도끼 (방패 제거)
            if rng.rn2(2) != 0 {
                eq.weapon = if rng.rn2(2) != 0 { 101 } else { 102 }; // TWO_HANDED_SWORD / BATTLE_AXE
                eq.shield = NO_ITEM;
            }
            if rng.rn2(2) != 0 {
                eq.armor = 103; // PLATE_MAIL ~ CHAIN_MAIL 중 하나 (상위에서 rnd_class)
            }
        }
        MplayerRole::Caveman => {
            if rng.rn2(4) != 0 {
                eq.weapon = 104; // MACE
            }
        }
        MplayerRole::Healer => {
            if rng.rn2(4) != 0 {
                eq.weapon = 105; // QUARTERSTAFF
            }
            eq.cloak = 106; // LAB_COAT
        }
        MplayerRole::Knight => {
            if rng.rn2(4) != 0 {
                eq.weapon = 107; // LONG_SWORD
            }
        }
        MplayerRole::Monk => {
            // 1/3 확률로 수리검, 아니면 소지품 없음
            eq.weapon = if rng.rn2(3) == 0 { 108 } else { NO_ITEM }; // SHURIKEN
            eq.armor = NO_ITEM;
            eq.cloak = 109; // ROBE
            if rng.rn2(2) != 0 {
                eq.shield = NO_ITEM;
            }
        }
        MplayerRole::Priest => {
            if rng.rn2(2) != 0 {
                eq.weapon = 104; // MACE
            }
            if rng.rn2(4) != 0 {
                eq.cloak = 109; // ROBE
            }
        }
        MplayerRole::Ranger => {
            if rng.rn2(2) != 0 {
                eq.weapon = 110; // ELVEN_DAGGER
            }
        }
        MplayerRole::Rogue => {
            if rng.rn2(2) != 0 {
                eq.weapon = 111; // SHORT_SWORD
            }
        }
        MplayerRole::Samurai => {
            if rng.rn2(2) != 0 {
                eq.weapon = 112; // KATANA
            }
        }
        MplayerRole::Tourist => {
            if rng.rn2(4) != 0 {
                eq.weapon = 113; // DART (다트 10~20개)
            }
        }
        MplayerRole::Valkyrie => {
            if rng.rn2(2) != 0 {
                eq.weapon = 107; // LONG_SWORD
            }
            if rng.rn2(2) != 0 {
                eq.shield = 114; // SHIELD_OF_REFLECTION
            }
        }
        MplayerRole::Wizard => {
            if rng.rn2(4) != 0 {
                eq.weapon = 105; // QUARTERSTAFF
            }
            if rng.rn2(2) != 0 {
                eq.cloak = 115; // CLOAK_OF_MAGIC_RESISTANCE
            }
            if rng.rn2(4) != 0 {
                eq.helm = 116; // HELM_OF_BRILLIANCE
            }
            eq.shield = NO_ITEM;
        }
    }
    eq
}

// =============================================================================
// [3] 레벨/HP 결정 (원본: mk_mplayer 141행)
// =============================================================================

/// [v2.22.0 R34-8] 플레이어 몬스터 레벨 결정 (원본: mk_mplayer)
pub fn calc_mplayer_level(is_special: bool, rng: &mut NetHackRng) -> i32 {
    if is_special {
        rng.rn1(16, 15) // 15..30
    } else {
        rng.rnd(16) // 1..16
    }
}

/// [v2.22.0 R34-8] 플레이어 몬스터 HP 결정 (원본: mk_mplayer)
pub fn calc_mplayer_hp(level: i32, is_special: bool, rng: &mut NetHackRng) -> i32 {
    let hp = rng.d(level, 10);
    let hp = if is_special { hp + 30 } else { hp };
    hp.max(1)
}

// =============================================================================
// [4] 아이템 생성 확률 테이블 (원본: mkobj.c 28-50행)
// =============================================================================

/// [v2.22.0 R34-8] 아이템 클래스 확률 엔트리
#[derive(Debug, Clone, Copy)]
pub struct ItemClassProb {
    /// 가중치 (확률 비율)
    pub weight: i32,
    /// 아이템 클래스 문자
    pub class: char,
}

/// [v2.22.0 R34-8] 일반 던전 아이템 확률 (원본: mkobjprobs[])
pub const MKOBJ_PROBS: &[ItemClassProb] = &[
    ItemClassProb {
        weight: 10,
        class: ')',
    }, // WEAPON_CLASS
    ItemClassProb {
        weight: 10,
        class: '[',
    }, // ARMOR_CLASS
    ItemClassProb {
        weight: 20,
        class: '%',
    }, // FOOD_CLASS
    ItemClassProb {
        weight: 8,
        class: '(',
    }, // TOOL_CLASS
    ItemClassProb {
        weight: 8,
        class: '!',
    }, // POTION_CLASS
    ItemClassProb {
        weight: 8,
        class: '?',
    }, // SCROLL_CLASS
    ItemClassProb {
        weight: 1,
        class: '+',
    }, // SPBOOK_CLASS
    ItemClassProb {
        weight: 8,
        class: '/',
    }, // WAND_CLASS
    ItemClassProb {
        weight: 8,
        class: '*',
    }, // GEM_CLASS
    ItemClassProb {
        weight: 8,
        class: '=',
    }, // RING_CLASS
    ItemClassProb {
        weight: 1,
        class: '"',
    }, // AMULET_CLASS
];

/// [v2.22.0 R34-8] 로그 수준 아이템 확률 (원본: rogueprobs[])
pub const ROGUE_PROBS: &[ItemClassProb] = &[
    ItemClassProb {
        weight: 12,
        class: ')',
    },
    ItemClassProb {
        weight: 12,
        class: '[',
    },
    ItemClassProb {
        weight: 22,
        class: '%',
    },
    ItemClassProb {
        weight: 22,
        class: '!',
    },
    ItemClassProb {
        weight: 12,
        class: '?',
    },
    ItemClassProb {
        weight: 1,
        class: '+',
    },
    ItemClassProb {
        weight: 7,
        class: '$',
    }, // COIN_CLASS
    ItemClassProb {
        weight: 6,
        class: '/',
    },
    ItemClassProb {
        weight: 5,
        class: '=',
    },
    ItemClassProb {
        weight: 1,
        class: '"',
    },
];

/// [v2.22.0 R34-8] 확률 테이블에서 아이템 클래스 선택 (원본: mkobj random_class 패턴)
pub fn select_item_class(probs: &[ItemClassProb], rng: &mut NetHackRng) -> char {
    let total: i32 = probs.iter().map(|p| p.weight).sum();
    if total <= 0 {
        return '%';
    } // 폴백

    let mut roll = rng.rnd(total);
    for p in probs {
        roll -= p.weight;
        if roll <= 0 {
            return p.class;
        }
    }
    probs.last().map_or('%', |p| p.class)
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mplayer_level_special() {
        let mut rng = NetHackRng::new(42);
        let lv = calc_mplayer_level(true, &mut rng);
        assert!(lv >= 15 && lv <= 30);
    }

    #[test]
    fn test_mplayer_level_normal() {
        let mut rng = NetHackRng::new(42);
        let lv = calc_mplayer_level(false, &mut rng);
        assert!(lv >= 1 && lv <= 16);
    }

    #[test]
    fn test_mplayer_hp() {
        let mut rng = NetHackRng::new(42);
        let hp = calc_mplayer_hp(10, true, &mut rng);
        assert!(hp >= 31); // d(10,10) 최소 10 + 30 = 40
    }

    #[test]
    fn test_monk_no_armor() {
        let mut rng = NetHackRng::new(42);
        let eq = calc_mplayer_equipment(MplayerRole::Monk, 50, 50, 50, 50, 50, &mut rng);
        assert_eq!(eq.armor, NO_ITEM);
        assert_eq!(eq.cloak, 109); // ROBE
    }

    #[test]
    fn test_wizard_no_shield() {
        let mut rng = NetHackRng::new(42);
        let eq = calc_mplayer_equipment(MplayerRole::Wizard, 50, 50, 50, 50, 50, &mut rng);
        assert_eq!(eq.shield, NO_ITEM);
    }

    #[test]
    fn test_select_class_distribution() {
        let mut rng = NetHackRng::new(42);
        let mut food_count = 0;
        for _ in 0..1000 {
            if select_item_class(MKOBJ_PROBS, &mut rng) == '%' {
                food_count += 1;
            }
        }
        // FOOD 가중치 = 20/90 ≈ 22%
        assert!(food_count > 100 && food_count < 400);
    }

    #[test]
    fn test_select_class_rogue() {
        let mut rng = NetHackRng::new(42);
        let cls = select_item_class(ROGUE_PROBS, &mut rng);
        // 유효한 클래스 중 하나여야 함
        assert!(")][%!?+$/=\"".contains(cls));
    }

    #[test]
    fn test_healer_cloak() {
        let mut rng = NetHackRng::new(42);
        let eq = calc_mplayer_equipment(MplayerRole::Healer, 50, 50, 50, 50, 50, &mut rng);
        assert_eq!(eq.cloak, 106); // LAB_COAT
    }
}
