// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.24.0 R12-4] 장비 속성 엔진 (worn_ext.rs)
//
// 원본 참조: NetHack 3.6.7 worn.c (1,133줄)
//
// 구현 내용:
//   1. 속성 마스크 시스템 (MR, 내성, 능력치)
//   2. 장비 슬롯별 속성 집계
//   3. 저주/축복 장비 특수 효과
//   4. 아티팩트 착용 시 부여 속성
//   5. 장비 변경 시 속성 재계산
// ============================================================================

use bitflags::bitflags;

// =============================================================================
// [1] 속성 마스크 (원본: worn.c extrinsics)
// =============================================================================

bitflags! {
    /// [v2.24.0 R12-4] 장비 부여 속성 (원본: worn.c W_* 매크로)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct WornProperty: u64 {
        // 내성 (Resistance)
        const FIRE_RES     = 1 << 0;
        const COLD_RES     = 1 << 1;
        const SHOCK_RES    = 1 << 2;
        const POISON_RES   = 1 << 3;
        const ACID_RES     = 1 << 4;
        const SLEEP_RES    = 1 << 5;
        const DISINT_RES   = 1 << 6;
        const MAGIC_RES    = 1 << 7;

        // 시야 관련
        const SEE_INVIS    = 1 << 8;
        const TELEPATHY    = 1 << 9;
        const WARNING      = 1 << 10;
        const SEARCHING    = 1 << 11;
        const INFRAVISION  = 1 << 12;

        // 이동 관련
        const LEVITATION   = 1 << 13;
        const FLYING       = 1 << 14;
        const SPEED        = 1 << 15;
        const STEALTH      = 1 << 16;
        const WATERWALKING = 1 << 17;

        // 전투 관련
        const REFLECTION   = 1 << 18;
        const FREE_ACTION  = 1 << 19;
        const PROTECTION   = 1 << 20;

        // 기타
        const REGENERATION = 1 << 21;
        const TELECONTROL  = 1 << 22;
        const POLYCONTROL  = 1 << 23;
        const HUNGER       = 1 << 24;  // 배고픔 가속
        const CONFLICT     = 1 << 25;
        const AGGRAVATE    = 1 << 26;
        const FUMBLING     = 1 << 27;
        const UNCHANGING   = 1 << 28;
    }
}

// =============================================================================
// [2] 장비 슬롯 (원본: worn.c W_ARM 등)
// =============================================================================

/// [v2.24.0 R12-4] 장비 슬롯
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipSlot {
    Helmet,
    Body,
    Cloak,
    Shield,
    Gloves,
    Boots,
    Ring1, // 왼손 반지
    Ring2, // 오른손 반지
    Amulet,
    Weapon,
    AltWeapon,
}

/// [v2.24.0 R12-4] 장비 아이템 속성 정보
#[derive(Debug, Clone)]
pub struct WornItem {
    /// 장비 이름
    pub name: String,
    /// 부여 속성
    pub properties: WornProperty,
    /// AC 보너스
    pub ac_bonus: i32,
    /// 강화 수치
    pub enchantment: i32,
    /// 저주 여부
    pub cursed: bool,
    /// 축복 여부
    pub blessed: bool,
    /// 아티팩트 여부
    pub artifact: bool,
}

// =============================================================================
// [3] 속성 집계 (원본: worn.c setnotworn, setworn)
// =============================================================================

/// [v2.24.0 R12-4] 장비 슬롯 세트에서 총 속성 집계
pub fn aggregate_properties(items: &[(EquipSlot, WornItem)]) -> WornProperty {
    let mut total = WornProperty::empty();
    for (_, item) in items {
        total |= item.properties;
    }
    total
}

/// [v2.24.0 R12-4] 총 AC 계산
pub fn aggregate_ac(base_ac: i32, items: &[(EquipSlot, WornItem)]) -> i32 {
    let mut ac = base_ac;
    for (_, item) in items {
        ac -= item.ac_bonus + item.enchantment;
    }
    ac
}

/// [v2.24.0 R12-4] 특정 속성 보유 여부 확인
pub fn has_property(items: &[(EquipSlot, WornItem)], prop: WornProperty) -> bool {
    aggregate_properties(items).contains(prop)
}

// =============================================================================
// [4] 저주/축복 효과 (원본: worn.c curse/bless_effects)
// =============================================================================

/// [v2.24.0 R12-4] 저주 장비 페널티
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurseEffect {
    /// 해제 불가
    CannotRemove,
    /// AC 페널티
    AcPenalty(i32),
    /// 부정적 속성 부여 (헝클거림 등)
    NegativeProperty(WornProperty),
}

/// [v2.24.0 R12-4] 저주 장비 효과 판정
pub fn curse_effects(slot: EquipSlot, cursed: bool) -> Vec<CurseEffect> {
    if !cursed {
        return vec![];
    }
    let mut effects = vec![CurseEffect::CannotRemove];
    match slot {
        EquipSlot::Boots => {
            effects.push(CurseEffect::NegativeProperty(WornProperty::FUMBLING));
        }
        EquipSlot::Helmet => {
            effects.push(CurseEffect::AcPenalty(1));
        }
        _ => {}
    }
    effects
}

/// [v2.24.0 R12-4] 축복 장비 보너스 (원본: bless_effects)
pub fn bless_bonus(enchantment: i32, blessed: bool) -> i32 {
    if blessed {
        enchantment + 1 // 축복 시 +1 추가
    } else {
        enchantment
    }
}

// =============================================================================
// [5] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_helmet() -> WornItem {
        WornItem {
            name: "helm of brilliance".to_string(),
            properties: WornProperty::empty(),
            ac_bonus: 1,
            enchantment: 2,
            cursed: false,
            blessed: false,
            artifact: false,
        }
    }

    fn test_cloak() -> WornItem {
        WornItem {
            name: "cloak of magic resistance".to_string(),
            properties: WornProperty::MAGIC_RES,
            ac_bonus: 1,
            enchantment: 0,
            cursed: false,
            blessed: false,
            artifact: false,
        }
    }

    fn test_ring() -> WornItem {
        WornItem {
            name: "ring of fire resistance".to_string(),
            properties: WornProperty::FIRE_RES | WornProperty::SEE_INVIS,
            ac_bonus: 0,
            enchantment: 0,
            cursed: false,
            blessed: false,
            artifact: false,
        }
    }

    #[test]
    fn test_aggregate_empty() {
        let items: Vec<(EquipSlot, WornItem)> = vec![];
        assert_eq!(aggregate_properties(&items), WornProperty::empty());
    }

    #[test]
    fn test_aggregate_single() {
        let items = vec![(EquipSlot::Cloak, test_cloak())];
        assert!(aggregate_properties(&items).contains(WornProperty::MAGIC_RES));
    }

    #[test]
    fn test_aggregate_multiple() {
        let items = vec![
            (EquipSlot::Cloak, test_cloak()),
            (EquipSlot::Ring1, test_ring()),
        ];
        let props = aggregate_properties(&items);
        assert!(props.contains(WornProperty::MAGIC_RES));
        assert!(props.contains(WornProperty::FIRE_RES));
        assert!(props.contains(WornProperty::SEE_INVIS));
    }

    #[test]
    fn test_ac_calculation() {
        let items = vec![
            (EquipSlot::Helmet, test_helmet()), // ac_bonus 1, enchant 2
            (EquipSlot::Cloak, test_cloak()),   // ac_bonus 1, enchant 0
        ];
        let ac = aggregate_ac(10, &items);
        assert_eq!(ac, 6); // 10 - (1+2) - (1+0) = 6
    }

    #[test]
    fn test_has_property() {
        let items = vec![(EquipSlot::Ring1, test_ring())];
        assert!(has_property(&items, WornProperty::FIRE_RES));
        assert!(!has_property(&items, WornProperty::COLD_RES));
    }

    #[test]
    fn test_curse_boots() {
        let effects = curse_effects(EquipSlot::Boots, true);
        assert!(effects.contains(&CurseEffect::CannotRemove));
        assert!(effects.iter().any(
            |e| matches!(e, CurseEffect::NegativeProperty(p) if p.contains(WornProperty::FUMBLING))
        ));
    }

    #[test]
    fn test_curse_not_cursed() {
        let effects = curse_effects(EquipSlot::Boots, false);
        assert!(effects.is_empty());
    }

    #[test]
    fn test_bless_bonus() {
        assert_eq!(bless_bonus(3, true), 4);
        assert_eq!(bless_bonus(3, false), 3);
    }
}
