// ============================================================================
// [v2.30.0 Phase 94-2] 장비 관리 확장 (worn_phase94_ext.rs)
// 원본: NetHack 3.6.7 src/worn.c L100-600 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 장비 슬롯 시스템 — equipment_slots (worn.c L100-300)
// =============================================================================

/// [v2.30.0 94-2] 장비 슬롯
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    Helmet,
    Cloak,
    Body,
    Shield,
    Gloves,
    Boots,
    AmuletSlot,
    RingLeft,
    RingRight,
    WeaponMain,
    WeaponOff,
    Quiver,
    Blindfold,
}

/// [v2.30.0 94-2] 장비 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipEffect {
    pub ac_bonus: i32,
    pub damage_bonus: i32,
    pub speed_mod: i32,
    pub resistances: Vec<String>,
    pub special_effects: Vec<String>,
}

/// [v2.30.0 94-2] 장비 속성
#[derive(Debug, Clone)]
pub struct EquipmentItem {
    pub name: String,
    pub slot: EquipSlot,
    pub base_ac: i32,
    pub enchantment: i32,
    pub is_cursed: bool,
    pub is_blessed: bool,
    pub material: String,
    pub eroded: i32,
    pub special_property: Option<String>,
}

/// [v2.30.0 94-2] 장비 효과 계산
/// 원본: worn.c set_wear()
pub fn calculate_equipment_effect(item: &EquipmentItem) -> EquipEffect {
    let mut ac = item.base_ac + item.enchantment;
    if item.eroded > 0 {
        ac -= item.eroded;
    }

    let mut resistances = Vec::new();
    let mut specials = Vec::new();

    // 특수 속성 적용
    if let Some(ref prop) = item.special_property {
        match prop.as_str() {
            "화염 저항" => resistances.push("화염 저항".to_string()),
            "냉기 저항" => resistances.push("냉기 저항".to_string()),
            "독 저항" => resistances.push("독 저항".to_string()),
            "마법 저항" => resistances.push("마법 저항".to_string()),
            "투명 감지" => specials.push("투명 감지".to_string()),
            "텔레포트 제어" => specials.push("텔레포트 제어".to_string()),
            "반사" => specials.push("반사".to_string()),
            "부유" => specials.push("부유".to_string()),
            "속도" => specials.push("속도".to_string()),
            _ => {}
        }
    }

    // 재질 보너스
    let mat_bonus = match item.material.as_str() {
        "미스릴" => 1,
        "드래곤 가죽" => 2,
        "아다만틴" => 3,
        _ => 0,
    };
    ac += mat_bonus;

    let speed_mod = if specials.iter().any(|s| s == "속도") { 1 } else { 0 };

    EquipEffect {
        ac_bonus: ac.max(0),
        damage_bonus: item.enchantment.max(0),
        speed_mod,
        resistances,
        special_effects: specials,
    }
}

// =============================================================================
// [2] 총 AC 계산 — total_ac (worn.c L300-500)
// =============================================================================

/// [v2.30.0 94-2] 전체 장비 AC 합산
pub fn calculate_total_ac(
    equipped: &[EquipmentItem],
    dex_bonus: i32,
    _rng: &mut NetHackRng,
) -> i32 {
    let base = 10; // NetHack 기본 AC
    let armor_ac: i32 = equipped.iter()
        .map(|e| calculate_equipment_effect(e).ac_bonus)
        .sum();
    let dex_mod = (dex_bonus - 10) / 2;

    (base - armor_ac - dex_mod).max(-20)
}

// =============================================================================
// [3] 저주 장비 해제 — uncurse (worn.c L500-600)
// =============================================================================

/// [v2.30.0 94-2] 저주 해제 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UncurseResult {
    Success { item_name: String },
    StillCursed { reason: String },
    NotCursed,
}

/// [v2.30.0 94-2] 저주 장비 해제 시도
pub fn try_uncurse(
    item: &EquipmentItem,
    scroll_blessed: bool,
    rng: &mut NetHackRng,
) -> UncurseResult {
    if !item.is_cursed {
        return UncurseResult::NotCursed;
    }

    let success_chance = if scroll_blessed { 90 } else { 50 };
    if rng.rn2(100) < success_chance {
        UncurseResult::Success {
            item_name: item.name.clone(),
        }
    } else {
        UncurseResult::StillCursed {
            reason: "저주가 너무 강력하다.".to_string(),
        }
    }
}

// =============================================================================
// [4] 부식/손상 — erosion (worn.c L600+)
// =============================================================================

/// [v2.30.0 94-2] 부식 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErosionResult {
    Eroded { new_level: i32, message: String },
    Destroyed { item_name: String },
    Resistant,
    NoEffect,
}

/// [v2.30.0 94-2] 부식 적용
pub fn apply_erosion(
    item: &EquipmentItem,
    erosion_type: &str, // "녹", "불", "산"
    rng: &mut NetHackRng,
) -> ErosionResult {
    // 재질 기반 저항
    let resistant = match (erosion_type, item.material.as_str()) {
        ("녹", "미스릴") | ("녹", "금") | ("녹", "나무") => true,
        ("불", "금속") | ("불", "미스릴") => true,
        ("산", "유리") => true,
        _ => false,
    };

    if resistant {
        return ErosionResult::Resistant;
    }

    // 이미 최대 부식
    if item.eroded >= 3 {
        if rng.rn2(3) == 0 {
            return ErosionResult::Destroyed {
                item_name: item.name.clone(),
            };
        }
        return ErosionResult::NoEffect;
    }

    let msg = match erosion_type {
        "녹" => format!("{}이(가) 녹슬었다!", item.name),
        "불" => format!("{}이(가) 그을렸다!", item.name),
        "산" => format!("{}이(가) 부식되었다!", item.name),
        _ => format!("{}이(가) 손상되었다!", item.name),
    };

    ErosionResult::Eroded {
        new_level: item.eroded + 1,
        message: msg,
    }
}

// =============================================================================
// [테스트]
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    fn test_armor() -> EquipmentItem {
        EquipmentItem {
            name: "쇠사슬 갑옷".to_string(),
            slot: EquipSlot::Body,
            base_ac: 5,
            enchantment: 2,
            is_cursed: false,
            is_blessed: false,
            material: "금속".to_string(),
            eroded: 0,
            special_property: None,
        }
    }

    #[test]
    fn test_equip_effect_basic() {
        let armor = test_armor();
        let effect = calculate_equipment_effect(&armor);
        assert_eq!(effect.ac_bonus, 7); // 5 + 2
    }

    #[test]
    fn test_equip_effect_eroded() {
        let mut armor = test_armor();
        armor.eroded = 2;
        let effect = calculate_equipment_effect(&armor);
        assert_eq!(effect.ac_bonus, 5); // 5 + 2 - 2
    }

    #[test]
    fn test_equip_resistance() {
        let mut armor = test_armor();
        armor.special_property = Some("화염 저항".to_string());
        let effect = calculate_equipment_effect(&armor);
        assert!(effect.resistances.contains(&"화염 저항".to_string()));
    }

    #[test]
    fn test_mithril_bonus() {
        let mut armor = test_armor();
        armor.material = "미스릴".to_string();
        let effect = calculate_equipment_effect(&armor);
        assert_eq!(effect.ac_bonus, 8); // 5 + 2 + 1
    }

    #[test]
    fn test_total_ac() {
        let mut rng = test_rng();
        let equipped = vec![test_armor()];
        let ac = calculate_total_ac(&equipped, 14, &mut rng);
        assert!(ac < 10); // AC가 낮아져야
    }

    #[test]
    fn test_uncurse_success() {
        let mut rng = test_rng();
        let mut armor = test_armor();
        armor.is_cursed = true;
        let mut success = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = try_uncurse(&armor, true, &mut rng);
            if matches!(result, UncurseResult::Success { .. }) {
                success = true;
                break;
            }
        }
        assert!(success);
    }

    #[test]
    fn test_uncurse_not_cursed() {
        let mut rng = test_rng();
        let armor = test_armor();
        let result = try_uncurse(&armor, false, &mut rng);
        assert!(matches!(result, UncurseResult::NotCursed));
    }

    #[test]
    fn test_erosion_normal() {
        let mut rng = test_rng();
        let armor = test_armor();
        let result = apply_erosion(&armor, "녹", &mut rng);
        assert!(matches!(result, ErosionResult::Eroded { .. }));
    }

    #[test]
    fn test_erosion_resistant() {
        let mut rng = test_rng();
        let mut armor = test_armor();
        armor.material = "미스릴".to_string();
        let result = apply_erosion(&armor, "녹", &mut rng);
        assert!(matches!(result, ErosionResult::Resistant));
    }
}
