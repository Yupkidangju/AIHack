// ============================================================================
// [v2.26.0 Phase 90-4] 장비 시스템 확장 (do_wear_phase90_ext.rs)
// 원본: NetHack 3.6.7 src/do_wear.c L800-2200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

// =============================================================================
// [1] 장비 착용 제한 — canwearobj (do_wear.c L800-1000)
// =============================================================================

/// [v2.26.0 90-4] 장비 착용 불가 사유
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WearFailure {
    /// 이미 해당 슬롯에 장비 중
    SlotOccupied { slot: String, current_item: String },
    /// 체형 불일치 (변이 중)
    BodyShapeMismatch { reason: String },
    /// 저주받은 기존 장비가 해제 안 됨
    CursedBlockingItem { slot: String },
    /// 장님이라 안경 착용 불가
    CannotSeeToWear,
    /// 아이템 유형이 슬롯에 안 맞음
    WrongSlot { expected: String },
    /// 양손무기와 방패 동시 착용 불가
    TwoHandedConflict,
}

/// [v2.26.0 90-4] 장비 착용 가능 여부 판정
/// 원본: do_wear.c canwearobj()
pub fn can_wear_check(
    item_slot: &str, // "helmet"/"armor"/"shield"/"cloak"/"gloves"/"boots"/"shirt"
    current_slots: &[(String, bool)], // (슬롯이름, 저주여부)
    is_polymorphed: bool,
    poly_nohands: bool,  // 변이 시 손 없음
    poly_nohelmet: bool, // 변이 시 머리 없음
    wielding_twohander: bool,
) -> Result<(), WearFailure> {
    // [1] 변이 체형 검사
    if is_polymorphed {
        if poly_nohands && matches!(item_slot, "gloves" | "shield") {
            return Err(WearFailure::BodyShapeMismatch {
                reason: "손이 없어 착용 불가".to_string(),
            });
        }
        if poly_nohelmet && item_slot == "helmet" {
            return Err(WearFailure::BodyShapeMismatch {
                reason: "머리 형태가 달라 착용 불가".to_string(),
            });
        }
    }

    // [2] 양손무기 + 방패 충돌
    if item_slot == "shield" && wielding_twohander {
        return Err(WearFailure::TwoHandedConflict);
    }

    // [3] 슬롯 점유 검사
    for (slot, cursed) in current_slots {
        if slot == item_slot {
            if *cursed {
                return Err(WearFailure::CursedBlockingItem { slot: slot.clone() });
            } else {
                return Err(WearFailure::SlotOccupied {
                    slot: slot.clone(),
                    current_item: "현재 장비".to_string(),
                });
            }
        }
    }

    Ok(())
}

// =============================================================================
// [2] 장비 효과 — equip_effects (do_wear.c L1200-1600)
// =============================================================================

/// [v2.26.0 90-4] 장비 착용 시 발생 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipEffect {
    /// AC 변화
    AcChange { delta: i32 },
    /// 속성 부여 (저항/능력)
    GrantProperty { property: String },
    /// 속성 제거 (벗을 때)
    RemoveProperty { property: String },
    /// 저주 메시지
    CurseMessage { message: String },
    /// 축복 효과
    BlessedBonus { description: String },
    /// 변이 (헬름 오브 폴리모프)
    TriggerPolymorph,
    /// 없음
    None,
}

/// [v2.26.0 90-4] 장비 착용 효과 판정
/// 원본: do_wear.c Ring_on(), Armor_on() 등
pub fn equip_effect(
    item_name: &str,
    enchantment: i32,
    is_cursed: bool,
    is_blessed: bool,
    slot: &str,
) -> Vec<EquipEffect> {
    let mut effects = Vec::new();

    // [1] AC 변화 (기본)
    let base_ac = match slot {
        "armor" => 3 + enchantment,
        "cloak" => 1 + enchantment,
        "helmet" => 1 + enchantment,
        "shield" => 1 + enchantment,
        "gloves" => 1 + enchantment,
        "boots" => 1 + enchantment,
        "shirt" => 0 + enchantment,
        _ => enchantment,
    };
    if base_ac != 0 {
        effects.push(EquipEffect::AcChange { delta: -base_ac }); // AC 감소 = 방어 증가
    }

    // [2] 특수 아이템 효과 (이름 기반)
    match item_name {
        "speed boots" => {
            effects.push(EquipEffect::GrantProperty {
                property: "speed".to_string(),
            });
        }
        "levitation boots" => {
            effects.push(EquipEffect::GrantProperty {
                property: "levitation".to_string(),
            });
        }
        "gauntlets of power" => {
            effects.push(EquipEffect::GrantProperty {
                property: "strength_25".to_string(),
            });
        }
        "helm of brilliance" => {
            effects.push(EquipEffect::GrantProperty {
                property: "intelligence_bonus".to_string(),
            });
        }
        "helm of opposite alignment" => {
            effects.push(EquipEffect::GrantProperty {
                property: "alignment_change".to_string(),
            });
        }
        "cloak of invisibility" => {
            effects.push(EquipEffect::GrantProperty {
                property: "invisibility".to_string(),
            });
        }
        "cloak of magic resistance" => {
            effects.push(EquipEffect::GrantProperty {
                property: "magic_resistance".to_string(),
            });
        }
        "shield of reflection" => {
            effects.push(EquipEffect::GrantProperty {
                property: "reflection".to_string(),
            });
        }
        _ => {}
    }

    // [3] 저주 효과
    if is_cursed {
        effects.push(EquipEffect::CurseMessage {
            message: format!("{}이(가) 몸에 들러붙는 느낌이 든다!", item_name),
        });
    }

    // [4] 축복 효과
    if is_blessed && enchantment > 0 {
        effects.push(EquipEffect::BlessedBonus {
            description: "성스러운 빛이 장비를 감싼다.".to_string(),
        });
    }

    effects
}

// =============================================================================
// [3] 장비 해제 순서 — takeoff_order (do_wear.c L1800-2000)
// =============================================================================

/// [v2.26.0 90-4] 장비 해제 순서 (계층 제약 반영)
/// 원본: do_wear.c canremoveobj()
/// 규칙: 망토→갑옷→셔츠 순서 (망토가 위에)
pub fn takeoff_order(target_slot: &str, worn_layers: &[&str]) -> Vec<String> {
    let layer_order = ["cloak", "armor", "shirt"];
    let mut must_remove = Vec::new();

    let target_idx = layer_order.iter().position(|&s| s == target_slot);

    if let Some(idx) = target_idx {
        // 위 계층부터 먼저 벗어야 함
        for i in 0..idx {
            if worn_layers.contains(&layer_order[i]) {
                must_remove.push(layer_order[i].to_string());
            }
        }
    }

    must_remove.push(target_slot.to_string());
    must_remove
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- can_wear_check ---

    #[test]
    fn test_wear_ok() {
        let result = can_wear_check("helmet", &[], false, false, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wear_slot_occupied() {
        let slots = vec![("helmet".to_string(), false)];
        let result = can_wear_check("helmet", &slots, false, false, false, false);
        assert!(matches!(result, Err(WearFailure::SlotOccupied { .. })));
    }

    #[test]
    fn test_wear_cursed_blocking() {
        let slots = vec![("helmet".to_string(), true)];
        let result = can_wear_check("helmet", &slots, false, false, false, false);
        assert!(matches!(
            result,
            Err(WearFailure::CursedBlockingItem { .. })
        ));
    }

    #[test]
    fn test_wear_twohander_shield() {
        let result = can_wear_check("shield", &[], false, false, false, true);
        assert!(matches!(result, Err(WearFailure::TwoHandedConflict)));
    }

    #[test]
    fn test_wear_polymorph_nohands() {
        let result = can_wear_check("gloves", &[], true, true, false, false);
        assert!(matches!(result, Err(WearFailure::BodyShapeMismatch { .. })));
    }

    // --- equip_effect ---

    #[test]
    fn test_armor_ac() {
        let effects = equip_effect("plate mail", 2, false, false, "armor");
        let ac = effects
            .iter()
            .find(|e| matches!(e, EquipEffect::AcChange { .. }));
        assert!(ac.is_some());
    }

    #[test]
    fn test_speed_boots() {
        let effects = equip_effect("speed boots", 0, false, false, "boots");
        assert!(effects
            .iter()
            .any(|e| matches!(e, EquipEffect::GrantProperty { property } if property == "speed")));
    }

    #[test]
    fn test_cursed_message() {
        let effects = equip_effect("leather armor", 0, true, false, "armor");
        assert!(effects
            .iter()
            .any(|e| matches!(e, EquipEffect::CurseMessage { .. })));
    }

    // --- takeoff_order ---

    #[test]
    fn test_takeoff_armor_with_cloak() {
        let order = takeoff_order("armor", &["cloak", "armor"]);
        assert_eq!(order, vec!["cloak".to_string(), "armor".to_string()]);
    }

    #[test]
    fn test_takeoff_cloak_alone() {
        let order = takeoff_order("cloak", &["cloak"]);
        assert_eq!(order, vec!["cloak".to_string()]);
    }

    #[test]
    fn test_takeoff_helmet_no_layers() {
        let order = takeoff_order("helmet", &["helmet"]);
        assert_eq!(order, vec!["helmet".to_string()]);
    }
}
