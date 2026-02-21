// =============================================================================
// AIHack — do_wear_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
// [v2.19.0] do_wear.c 핵심 함수 이식 — Pure Result 패턴
// 원본: nethack-3.6.7/src/do_wear.c (2,663줄)
//
// 이식 대상:
//   canwearobj     (L1600+) → can_wear_check
//   takeoff_order  (L17-22) → takeoff_order_priority
//   Armor_on       (L162)   → armor_on_effect
//   Cloak_on       (L220)   → cloak_on_effect
//   Helmet_on      (L290)   → helmet_on_effect
//   Gloves_on      (L350)   → gloves_on_effect
//   Shield_on      (L410)   → shield_on_effect
//   Ring_on/off    (L600+)  → ring_on_effect / ring_off_effect
//   Amulet_on      (L500)   → amulet_on_effect
//   select_off     (L1200+) → select_off_check
// =============================================================================

// 현재 이 모듈에서는 RNG를 사용하지 않지만, 향후 확장을 위해 유지
#[cfg(test)]
use crate::util::rng::NetHackRng;

// =============================================================================
// [v2.19.0] 방어구 슬롯 / 장비 유형
// =============================================================================

/// 방어구 슬롯
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    Helmet,
    Cloak,
    Body,
    Shield,
    Gloves,
    Boots,
    Shirt,
}

/// 반지 슬롯
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RingSlot {
    Left,
    Right,
}

/// 장비 착용 순서 (원본: takeoff_order L17-22)
/// 탈의 시 이 순서의 역순으로 벗어야 함
pub fn takeoff_order() -> &'static [ArmorSlot] {
    // 원본: W_ARM, W_ARMC, W_ARMH, W_ARMS, W_ARMG, W_ARMF, W_ARMU
    &[
        ArmorSlot::Shield, // 발이 맨 먼저 벗겨짐
        ArmorSlot::Gloves,
        ArmorSlot::Boots,
        ArmorSlot::Helmet,
        ArmorSlot::Cloak,
        ArmorSlot::Body,
        ArmorSlot::Shirt,
    ]
}

// =============================================================================
// [v2.19.0] 1. can_wear_check — 착용 가능 판정
// 원본: do_wear.c canwearobj() ~L1600
// =============================================================================

/// 착용 불가 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WearBlockReason {
    /// 이미 해당 슬롯에 장비가 있음
    SlotOccupied,
    /// 형태 변환 상태로 착용 불가
    PolymorphedForm,
    /// 양손 무기 들고 방패 불가
    TwoHandedWeapon,
    /// 발굽 있어서 부츠 불가
    Hooves,
    /// 손이 없어서 장갑 불가
    NoHands,
    /// 머리가 이상해서 투구 불가
    HeadShape,
    /// 저주된 기존 장비
    CursedBlocker,
}

/// [v2.19.0] 착용 가능 여부 판정 (원본: canwearobj ~L1600)
pub fn can_wear_check(
    slot: ArmorSlot,
    slot_occupied: bool,
    has_cursed_blocker: bool,
    is_polymorphed: bool,
    has_two_handed_weapon: bool,
    has_hooves: bool,
    has_hands: bool,
    has_normal_head: bool,
) -> Result<(), WearBlockReason> {
    if slot_occupied {
        return Err(WearBlockReason::SlotOccupied);
    }
    if has_cursed_blocker {
        return Err(WearBlockReason::CursedBlocker);
    }

    match slot {
        ArmorSlot::Shield => {
            if has_two_handed_weapon {
                return Err(WearBlockReason::TwoHandedWeapon);
            }
        }
        ArmorSlot::Boots => {
            if has_hooves {
                return Err(WearBlockReason::Hooves);
            }
        }
        ArmorSlot::Gloves => {
            if !has_hands {
                return Err(WearBlockReason::NoHands);
            }
        }
        ArmorSlot::Helmet => {
            if !has_normal_head {
                return Err(WearBlockReason::HeadShape);
            }
            if is_polymorphed {
                return Err(WearBlockReason::PolymorphedForm);
            }
        }
        ArmorSlot::Body | ArmorSlot::Shirt => {
            if is_polymorphed {
                return Err(WearBlockReason::PolymorphedForm);
            }
        }
        ArmorSlot::Cloak => {} // 대부분 착용 가능
    }

    Ok(())
}

// =============================================================================
// [v2.19.0] 2. armor_on_effect — 갑옷 착용 효과
// 원본: do_wear.c Armor_on() L162+
// =============================================================================

/// 방어구 착용 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArmorEffect {
    /// AC 변화만
    AcChange { delta: i32 },
    /// 속성 부여 (은밀, 전위 등)
    PropertyGrant {
        property: &'static str,
        ac_delta: i32,
    },
    /// 저주/축복 효과
    BucEffect {
        message: &'static str,
        ac_delta: i32,
    },
}

/// [v2.19.0] 갑옷 착용 시 효과 (원본: Armor_on/Cloak_on/Helmet_on 등)
pub fn armor_on_effect(
    slot: ArmorSlot,
    ac_bonus: i32,
    is_blessed: bool,
    is_cursed: bool,
    special_property: Option<&str>,
) -> ArmorEffect {
    let ac_delta = -ac_bonus; // AC는 낮을수록 좋음

    if let Some(prop) = special_property {
        return ArmorEffect::PropertyGrant {
            property: match prop {
                "stealth" => "은밀",
                "displacement" => "전위",
                "invisibility" => "투명",
                "flying" => "비행",
                "speed" => "속도",
                "see_invisible" => "투시",
                "telepathy" => "텔레파시",
                _ => "기타",
            },
            ac_delta,
        };
    }

    if is_cursed {
        ArmorEffect::BucEffect {
            message: "저주가 느껴진다...",
            ac_delta,
        }
    } else if is_blessed && matches!(slot, ArmorSlot::Body | ArmorSlot::Cloak) {
        ArmorEffect::BucEffect {
            message: "축복의 온기가 느껴진다.",
            ac_delta,
        }
    } else {
        ArmorEffect::AcChange { delta: ac_delta }
    }
}

// =============================================================================
// [v2.19.0] 3. ring_on_effect — 반지 착용 효과
// 원본: do_wear.c Ring_on() L600+
// =============================================================================

/// 반지 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RingEffect {
    /// 속성 부여
    PropertyGrant { property: &'static str },
    /// 스탯 변화
    StatChange { stat: &'static str, delta: i32 },
    /// 텔레포트 (저주)
    Teleportitis,
    /// 배고픔 증가 (저주)
    HungerIncrease,
    /// 아무 효과 없음 (미식별)
    NoVisibleEffect,
}

/// [v2.19.0] 반지 착용 효과 (원본: Ring_on ~L600)
/// ring_name을 &'static str로 매핑하여 PropertyGrant에 안전하게 할당
pub fn ring_on_effect(ring_name: &str, enchantment: i32, is_cursed: bool) -> RingEffect {
    // 속성 부여 반지 — 각 arm에서 static 리터럴 직접 사용
    let property: Option<&'static str> = match ring_name {
        "fire resistance" => Some("fire resistance"),
        "cold resistance" => Some("cold resistance"),
        "shock resistance" => Some("shock resistance"),
        "poison resistance" => Some("poison resistance"),
        "free action" => Some("free action"),
        "see invisible" => Some("see invisible"),
        "regeneration" => Some("regeneration"),
        "teleport control" => Some("teleport control"),
        "slow digestion" => Some("slow digestion"),
        "conflict" => Some("conflict"),
        "warning" => Some("warning"),
        "stealth" => Some("stealth"),
        "levitation" => Some("levitation"),
        "searching" => Some("searching"),
        "sustain ability" => Some("sustain ability"),
        _ => None,
    };

    if let Some(prop) = property {
        return RingEffect::PropertyGrant { property: prop };
    }

    match ring_name {
        "protection" => RingEffect::StatChange {
            stat: "AC",
            delta: -enchantment,
        },
        "adornment" => RingEffect::StatChange {
            stat: "CHA",
            delta: enchantment,
        },
        "gain strength" => RingEffect::StatChange {
            stat: "STR",
            delta: enchantment,
        },
        "gain constitution" => RingEffect::StatChange {
            stat: "CON",
            delta: enchantment,
        },
        "increase accuracy" => RingEffect::StatChange {
            stat: "명중",
            delta: enchantment,
        },
        "increase damage" => RingEffect::StatChange {
            stat: "데미지",
            delta: enchantment,
        },
        "teleportation" => {
            if is_cursed {
                RingEffect::Teleportitis
            } else {
                RingEffect::PropertyGrant {
                    property: "teleportation",
                }
            }
        }
        "hunger" => RingEffect::HungerIncrease,
        _ => RingEffect::NoVisibleEffect,
    }
}

// =============================================================================
// [v2.19.0] 4. amulet_on_effect — 목걸이 착용 효과
// 원본: do_wear.c Amulet_on() L500+
// =============================================================================

/// [v2.19.0] 목걸이 착용 효과 (원본: Amulet_on ~L500)
pub fn amulet_on_effect(amulet_name: &str) -> &'static str {
    match amulet_name {
        "life saving" => "생명 구원의 힘이 느껴진다.",
        "strangulation" => "목이 조여온다!",
        "change" => "몸이 변하기 시작한다...",
        "unchanging" => "형태가 안정된다.",
        "reflection" => "표면이 빛난다.",
        "magical breathing" => "호흡이 편안해진다.",
        "restful sleep" => "졸음이 밀려온다...",
        "ESP" => "정신 감각이 확장된다.",
        "versus poison" => "독에 대한 내성이 생긴다.",
        _ => "특별한 느낌은 없다.",
    }
}

// =============================================================================
// [v2.19.0] 5. wear_delay — 착용 소요 턴
// 원본: do_wear.c 각 Armor_on 함수의 delay 값
// =============================================================================

/// [v2.19.0] 착용 소요 턴 (원본: 각 Armor_on 함수)
pub fn wear_delay(slot: ArmorSlot) -> i32 {
    match slot {
        ArmorSlot::Body => 5,
        ArmorSlot::Cloak => 1,
        ArmorSlot::Helmet => 1,
        ArmorSlot::Shield => 1,
        ArmorSlot::Gloves => 1,
        ArmorSlot::Boots => 1,
        ArmorSlot::Shirt => 5,
    }
}

/// [v2.19.0] 탈의 소요 턴
pub fn takeoff_delay(slot: ArmorSlot) -> i32 {
    match slot {
        ArmorSlot::Body => 5,
        ArmorSlot::Cloak => 1,
        ArmorSlot::Helmet => 1,
        ArmorSlot::Shield => 1,
        ArmorSlot::Gloves => 1,
        ArmorSlot::Boots => 1,
        ArmorSlot::Shirt => 5,
    }
}

// =============================================================================
// [v2.19.0] 6. select_off_check — 탈의 순서 판정
// 원본: do_wear.c select_off() ~L1200
// =============================================================================

/// [v2.19.0] 탈의 선행 조건 확인 — 벗으려면 먼저 벗어야 할 장비
pub fn select_off_blockers(target_slot: ArmorSlot) -> Vec<ArmorSlot> {
    match target_slot {
        // 셔츠를 벗으려면 갑옷과 망토를 먼저 벗어야 함
        ArmorSlot::Shirt => vec![ArmorSlot::Body, ArmorSlot::Cloak],
        // 갑옷을 벗으려면 망토를 먼저 벗어야 함
        ArmorSlot::Body => vec![ArmorSlot::Cloak],
        // 나머지는 직접 벗기 가능
        _ => vec![],
    }
}

// =============================================================================
// [v2.19.0] 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    // --- can_wear_check ---
    #[test]
    fn test_wear_ok() {
        assert!(can_wear_check(
            ArmorSlot::Cloak,
            false,
            false,
            false,
            false,
            false,
            true,
            true
        )
        .is_ok());
    }

    #[test]
    fn test_wear_occupied() {
        assert_eq!(
            can_wear_check(
                ArmorSlot::Helmet,
                true,
                false,
                false,
                false,
                false,
                true,
                true
            )
            .unwrap_err(),
            WearBlockReason::SlotOccupied
        );
    }

    #[test]
    fn test_wear_two_handed() {
        assert_eq!(
            can_wear_check(
                ArmorSlot::Shield,
                false,
                false,
                false,
                true,
                false,
                true,
                true
            )
            .unwrap_err(),
            WearBlockReason::TwoHandedWeapon
        );
    }

    #[test]
    fn test_wear_hooves() {
        assert_eq!(
            can_wear_check(
                ArmorSlot::Boots,
                false,
                false,
                false,
                false,
                true,
                true,
                true
            )
            .unwrap_err(),
            WearBlockReason::Hooves
        );
    }

    #[test]
    fn test_wear_no_hands() {
        assert_eq!(
            can_wear_check(
                ArmorSlot::Gloves,
                false,
                false,
                false,
                false,
                false,
                false,
                true
            )
            .unwrap_err(),
            WearBlockReason::NoHands
        );
    }

    // --- armor_on_effect ---
    #[test]
    fn test_armor_ac_only() {
        let e = armor_on_effect(ArmorSlot::Body, 5, false, false, None);
        assert!(matches!(e, ArmorEffect::AcChange { delta: -5 }));
    }

    #[test]
    fn test_armor_stealth() {
        let e = armor_on_effect(ArmorSlot::Boots, 1, false, false, Some("stealth"));
        match e {
            ArmorEffect::PropertyGrant { property, .. } => assert_eq!(property, "은밀"),
            _ => panic!("은밀 속성"),
        }
    }

    #[test]
    fn test_armor_cursed() {
        let e = armor_on_effect(ArmorSlot::Body, 3, false, true, None);
        match e {
            ArmorEffect::BucEffect { message, .. } => assert!(message.contains("저주")),
            _ => panic!("저주 효과"),
        }
    }

    // --- ring_on_effect ---
    #[test]
    fn test_ring_property() {
        let e = ring_on_effect("fire resistance", 0, false);
        assert!(matches!(e, RingEffect::PropertyGrant { .. }));
    }

    #[test]
    fn test_ring_stat() {
        let e = ring_on_effect("protection", 3, false);
        match e {
            RingEffect::StatChange { stat, delta } => {
                assert_eq!(stat, "AC");
                assert_eq!(delta, -3);
            }
            _ => panic!("보호 반지"),
        }
    }

    #[test]
    fn test_ring_cursed_teleport() {
        let e = ring_on_effect("teleportation", 0, true);
        assert_eq!(e, RingEffect::Teleportitis);
    }

    // --- amulet ---
    #[test]
    fn test_amulet_strangulation() {
        let msg = amulet_on_effect("strangulation");
        assert!(msg.contains("조여"));
    }

    // --- wear/takeoff delay ---
    #[test]
    fn test_wear_delay_body() {
        assert_eq!(wear_delay(ArmorSlot::Body), 5);
    }

    #[test]
    fn test_wear_delay_cloak() {
        assert_eq!(wear_delay(ArmorSlot::Cloak), 1);
    }

    // --- select_off_blockers ---
    #[test]
    fn test_shirt_blockers() {
        let b = select_off_blockers(ArmorSlot::Shirt);
        assert!(b.contains(&ArmorSlot::Body));
        assert!(b.contains(&ArmorSlot::Cloak));
    }

    #[test]
    fn test_body_blockers() {
        let b = select_off_blockers(ArmorSlot::Body);
        assert!(b.contains(&ArmorSlot::Cloak));
    }

    #[test]
    fn test_helmet_no_blockers() {
        let b = select_off_blockers(ArmorSlot::Helmet);
        assert!(b.is_empty());
    }
}
