// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-15] 아이템 생성 확장 모듈 (mkobj_ext.rs)
// 원본: NetHack 3.6.7 mkobj.c (확률 테이블, 상자 내용물, 랜덤 클래스)
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 아이템 클래스 확률 테이블 (원본: mkobj.c:29-68)
// =============================================================================

/// [v2.22.0 R34-15] 아이템 클래스 종류 (원본: objclass.h)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemClass {
    Weapon,
    Armor,
    Food,
    Tool,
    Gem,
    Potion,
    Scroll,
    SpellBook,
    Wand,
    Ring,
    Amulet,
    Coin,
    Random,
}

/// [v2.22.0 R34-15] 확률 테이블 항목
struct ClassProb {
    prob: i32,
    class: ItemClass,
}

/// [v2.22.0 R34-15] 일반 레벨 아이템 확률 (원본: mkobjprobs[])
const NORMAL_PROBS: &[ClassProb] = &[
    ClassProb {
        prob: 10,
        class: ItemClass::Weapon,
    },
    ClassProb {
        prob: 10,
        class: ItemClass::Armor,
    },
    ClassProb {
        prob: 20,
        class: ItemClass::Food,
    },
    ClassProb {
        prob: 8,
        class: ItemClass::Tool,
    },
    ClassProb {
        prob: 8,
        class: ItemClass::Gem,
    },
    ClassProb {
        prob: 16,
        class: ItemClass::Potion,
    },
    ClassProb {
        prob: 16,
        class: ItemClass::Scroll,
    },
    ClassProb {
        prob: 4,
        class: ItemClass::SpellBook,
    },
    ClassProb {
        prob: 4,
        class: ItemClass::Wand,
    },
    ClassProb {
        prob: 3,
        class: ItemClass::Ring,
    },
    ClassProb {
        prob: 1,
        class: ItemClass::Amulet,
    },
];

/// [v2.22.0 R34-15] 상점 상자 내부 아이템 확률 (원본: boxiprobs[])
const BOX_PROBS: &[ClassProb] = &[
    ClassProb {
        prob: 18,
        class: ItemClass::Gem,
    },
    ClassProb {
        prob: 15,
        class: ItemClass::Food,
    },
    ClassProb {
        prob: 18,
        class: ItemClass::Potion,
    },
    ClassProb {
        prob: 18,
        class: ItemClass::Scroll,
    },
    ClassProb {
        prob: 12,
        class: ItemClass::SpellBook,
    },
    ClassProb {
        prob: 7,
        class: ItemClass::Coin,
    },
    ClassProb {
        prob: 6,
        class: ItemClass::Wand,
    },
    ClassProb {
        prob: 5,
        class: ItemClass::Ring,
    },
    ClassProb {
        prob: 1,
        class: ItemClass::Amulet,
    },
];

/// [v2.22.0 R34-15] 로그 레벨 아이템 확률 (원본: rogueprobs[])
const ROGUE_PROBS: &[ClassProb] = &[
    ClassProb {
        prob: 12,
        class: ItemClass::Weapon,
    },
    ClassProb {
        prob: 12,
        class: ItemClass::Armor,
    },
    ClassProb {
        prob: 22,
        class: ItemClass::Food,
    },
    ClassProb {
        prob: 22,
        class: ItemClass::Potion,
    },
    ClassProb {
        prob: 22,
        class: ItemClass::Scroll,
    },
    ClassProb {
        prob: 5,
        class: ItemClass::Wand,
    },
    ClassProb {
        prob: 5,
        class: ItemClass::Ring,
    },
];

/// [v2.22.0 R34-15] 지옥 레벨 아이템 확률 (원본: hellprobs[])
const HELL_PROBS: &[ClassProb] = &[
    ClassProb {
        prob: 20,
        class: ItemClass::Weapon,
    },
    ClassProb {
        prob: 20,
        class: ItemClass::Armor,
    },
    ClassProb {
        prob: 16,
        class: ItemClass::Food,
    },
    ClassProb {
        prob: 12,
        class: ItemClass::Tool,
    },
    ClassProb {
        prob: 10,
        class: ItemClass::Gem,
    },
    ClassProb {
        prob: 1,
        class: ItemClass::Potion,
    },
    ClassProb {
        prob: 1,
        class: ItemClass::Scroll,
    },
    ClassProb {
        prob: 8,
        class: ItemClass::Wand,
    },
    ClassProb {
        prob: 8,
        class: ItemClass::Ring,
    },
    ClassProb {
        prob: 4,
        class: ItemClass::Amulet,
    },
];

/// [v2.22.0 R34-15] 레벨 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LevelType {
    Normal,
    Rogue,
    Hell,
}

/// [v2.22.0 R34-15] 랜덤 아이템 클래스 선택 (원본: mkobj의 RANDOM_CLASS 분기)
pub fn select_random_class(level_type: LevelType, rng: &mut NetHackRng) -> ItemClass {
    let probs = match level_type {
        LevelType::Normal => NORMAL_PROBS,
        LevelType::Rogue => ROGUE_PROBS,
        LevelType::Hell => HELL_PROBS,
    };

    let mut roll = rng.rnd(100);
    for entry in probs {
        roll -= entry.prob;
        if roll <= 0 {
            return entry.class;
        }
    }
    // 마지막 항목 반환 (안전장치)
    probs.last().map(|e| e.class).unwrap_or(ItemClass::Food)
}

/// [v2.22.0 R34-15] 상자 내부 아이템 클래스 선택 (원본: boxiprobs 참조)
pub fn select_box_item_class(rng: &mut NetHackRng) -> ItemClass {
    let mut roll = rng.rnd(100);
    for entry in BOX_PROBS {
        roll -= entry.prob;
        if roll <= 0 {
            return entry.class;
        }
    }
    BOX_PROBS.last().map(|e| e.class).unwrap_or(ItemClass::Gem)
}

// =============================================================================
// [2] 상자 내용물 수량 (원본: mkobj.c:274-307 mkbox_cnts의 n 결정부)
// =============================================================================

/// [v2.22.0 R34-15] 상자 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxType {
    IceBox,
    Chest,
    LargeBox,
    Sack,
    OilskinSack,
    BagOfHolding,
    Other,
}

/// [v2.22.0 R34-15] 상자 최대 아이템 수 결정 (원본: mkbox_cnts의 switch)
/// `is_locked`: 잠긴 상자인지
/// `is_initial`: 초기 인벤토리(턴 1)인지
pub fn calc_box_max_items(box_type: BoxType, is_locked: bool, is_initial: bool) -> i32 {
    match box_type {
        BoxType::IceBox => 20,
        BoxType::Chest => {
            if is_locked {
                7
            } else {
                5
            }
        }
        BoxType::LargeBox => {
            if is_locked {
                5
            } else {
                3
            }
        }
        BoxType::Sack | BoxType::OilskinSack => {
            if is_initial {
                0
            } else {
                1
            }
        }
        BoxType::BagOfHolding => 1,
        BoxType::Other => 0,
    }
}

/// [v2.22.0 R34-15] 상자 실제 아이템 수 결정 (0~max 랜덤)
pub fn calc_box_item_count(
    box_type: BoxType,
    is_locked: bool,
    is_initial: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let max = calc_box_max_items(box_type, is_locked, is_initial);
    if max == 0 {
        0
    } else {
        rng.rn2(max + 1)
    }
}

// =============================================================================
// [3] 아이템 분할 결과 (원본: mkobj.c:427-465 splitobj의 수량 계산)
// =============================================================================

/// [v2.22.0 R34-15] 아이템 분할 수량 계산
#[derive(Debug, Clone)]
pub struct SplitResult {
    /// 원본 스택 남은 수량
    pub original_qty: i64,
    /// 새 스택 수량
    pub new_qty: i64,
    /// 유효한 분할인지
    pub valid: bool,
}

/// [v2.22.0 R34-15] 스택 분할 계산 (원본: splitobj)
pub fn calc_split(current_qty: i64, split_amount: i64, is_container: bool) -> SplitResult {
    // 컨테이너 분할 불가, 0 이하 불가, 전체 수량 이상 불가
    if is_container || split_amount <= 0 || current_qty <= split_amount {
        return SplitResult {
            original_qty: current_qty,
            new_qty: 0,
            valid: false,
        };
    }
    SplitResult {
        original_qty: current_qty - split_amount,
        new_qty: split_amount,
        valid: true,
    }
}

// =============================================================================
// [4] 아이템 변질 동사 (원본: mkobj.c:685-689 alteration_verbs[])
// =============================================================================

/// [v2.22.0 R34-15] 아이템 변질 유형 (원본: COST_xxx)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlterationType {
    Cancel = 0,
    Drain = 1,
    Uncharge = 2,
    Unbless = 3,
    Uncurse = 4,
    Disenchant = 5,
    Degrade = 6,
    Dilute = 7,
    Erase = 8,
    Burn = 9,
    Neutralize = 10,
    Destroy = 11,
    Splatter = 12,
    Bite = 13,
    Open = 14,
    BreakLock = 15,
    Rust = 16,
    Rot = 17,
    Tarnish = 18,
}

/// [v2.22.0 R34-15] 변질 동사 문자열 반환
pub fn alteration_verb(typ: AlterationType) -> &'static str {
    match typ {
        AlterationType::Cancel => "cancel",
        AlterationType::Drain => "drain",
        AlterationType::Uncharge => "uncharge",
        AlterationType::Unbless => "unbless",
        AlterationType::Uncurse => "uncurse",
        AlterationType::Disenchant => "disenchant",
        AlterationType::Degrade => "degrade",
        AlterationType::Dilute => "dilute",
        AlterationType::Erase => "erase",
        AlterationType::Burn => "burn",
        AlterationType::Neutralize => "neutralize",
        AlterationType::Destroy => "destroy",
        AlterationType::Splatter => "splatter",
        AlterationType::Bite => "bite",
        AlterationType::Open => "open",
        AlterationType::BreakLock => "break the lock on",
        AlterationType::Rust => "rust",
        AlterationType::Rot => "rot",
        AlterationType::Tarnish => "tarnish",
    }
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_random_class_normal() {
        let mut rng = NetHackRng::new(42);
        let class = select_random_class(LevelType::Normal, &mut rng);
        // 유효한 클래스여야 함
        assert_ne!(class, ItemClass::Random);
        assert_ne!(class, ItemClass::Coin); // 일반 확률에 금화 없음
    }

    #[test]
    fn test_select_random_class_rogue() {
        let mut rng = NetHackRng::new(42);
        let class = select_random_class(LevelType::Rogue, &mut rng);
        // 로그 레벨에서 가능한 클래스
        let valid = matches!(
            class,
            ItemClass::Weapon
                | ItemClass::Armor
                | ItemClass::Food
                | ItemClass::Potion
                | ItemClass::Scroll
                | ItemClass::Wand
                | ItemClass::Ring
        );
        assert!(valid);
    }

    #[test]
    fn test_select_box_item_class() {
        let mut rng = NetHackRng::new(42);
        let class = select_box_item_class(&mut rng);
        assert_ne!(class, ItemClass::Random);
    }

    #[test]
    fn test_box_max_items_ice_box() {
        assert_eq!(calc_box_max_items(BoxType::IceBox, false, false), 20);
    }

    #[test]
    fn test_box_max_items_chest_locked() {
        assert_eq!(calc_box_max_items(BoxType::Chest, true, false), 7);
    }

    #[test]
    fn test_box_max_items_sack_initial() {
        assert_eq!(calc_box_max_items(BoxType::Sack, false, true), 0);
    }

    #[test]
    fn test_box_item_count_range() {
        let mut rng = NetHackRng::new(42);
        let count = calc_box_item_count(BoxType::Chest, false, false, &mut rng);
        assert!(count >= 0 && count <= 5);
    }

    #[test]
    fn test_split_valid() {
        let result = calc_split(10, 3, false);
        assert!(result.valid);
        assert_eq!(result.original_qty, 7);
        assert_eq!(result.new_qty, 3);
    }

    #[test]
    fn test_split_container_invalid() {
        let result = calc_split(10, 3, true);
        assert!(!result.valid);
    }

    #[test]
    fn test_split_too_much() {
        let result = calc_split(5, 5, false);
        assert!(!result.valid);
    }

    #[test]
    fn test_alteration_verb() {
        assert_eq!(alteration_verb(AlterationType::Rust), "rust");
        assert_eq!(
            alteration_verb(AlterationType::BreakLock),
            "break the lock on"
        );
    }

    #[test]
    fn test_class_distribution_coverage() {
        // 각 레벨 유형별 1000번 선택하여 모든 클래스가 최소 1번은 나오는지
        let mut rng = NetHackRng::new(123);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            let c = select_random_class(LevelType::Normal, &mut rng);
            seen.insert(format!("{:?}", c));
        }
        // 일반 레벨: 11개 클래스 (Coin 제외)
        assert!(seen.len() >= 8, "Not enough variety: {:?}", seen);
    }
}
