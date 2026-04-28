// ============================================================================
// [v2.30.0 Phase 94-4] 다형성 확장 (polymorph_phase94_ext.rs)
// 원본: NetHack 3.6.7 src/polyself.c L200-1000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 변신 판정 — polymorph_check (polyself.c L200-500)
// =============================================================================

/// [v2.30.0 94-4] 변신 가능한 형태
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyForm {
    pub monster_name: String,
    pub monster_level: i32,
    pub hp_dice: i32,
    pub ac: i32,
    pub speed: i32,
    pub attacks: Vec<String>,
    pub resistances: Vec<String>,
    pub can_fly: bool,
    pub can_swim: bool,
    pub has_hands: bool,
    pub is_humanoid: bool,
}

/// [v2.30.0 94-4] 변신 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolymorphResult {
    /// 변신 성공
    Transformed { form: PolyForm, turns: i32 },
    /// 변신 실패 (저항 등)
    Resisted,
    /// 변신 해제 (원래 형태로)
    Reverted,
    /// 치명적 실패 (시스템 쇼크)
    SystemShock { damage: i32 },
    /// 영구 변신
    PermanentChange { form: PolyForm },
}

/// [v2.30.0 94-4] 랜덤 변신 형태 생성
fn generate_random_form(player_level: i32, rng: &mut NetHackRng) -> PolyForm {
    let forms = [
        ("드래곤", 20, 8, 2, 12, true, false),
        ("자이언트", 15, 6, 4, 8, false, false),
        ("거미", 5, 3, 6, 10, false, false),
        ("쥐", 1, 1, 9, 14, false, false),
        ("독수리", 8, 4, 5, 16, true, false),
        ("상어", 12, 5, 3, 14, false, true),
        ("늑대인간", 10, 5, 4, 12, false, false),
        ("님프", 7, 3, 7, 12, false, false),
        ("트롤", 12, 6, 2, 10, false, false),
        ("코볼트", 3, 2, 8, 8, false, false),
    ];

    let max_idx = ((player_level / 3) as usize + 1).min(forms.len());
    let idx = rng.rn2(max_idx as i32) as usize;
    let (name, level, hp, ac, speed, fly, swim) = forms[idx];

    PolyForm {
        monster_name: name.to_string(),
        monster_level: level,
        hp_dice: hp,
        ac,
        speed,
        attacks: vec!["기본 공격".to_string()],
        resistances: Vec::new(),
        can_fly: fly,
        can_swim: swim,
        has_hands: name == "님프" || name == "트롤" || name == "자이언트",
        is_humanoid: name == "님프" || name == "늑대인간" || name == "자이언트",
    }
}

/// [v2.30.0 94-4] 변신 판정
/// 원본: polyself.c polymon()
pub fn polymorph_self(
    player_level: i32,
    player_con: i32,
    has_polymorph_control: bool,
    has_poly_resistance: bool,
    rng: &mut NetHackRng,
) -> PolymorphResult {
    // 변신 저항
    if has_poly_resistance && rng.rn2(3) > 0 {
        return PolymorphResult::Resisted;
    }

    // 시스템 쇼크 (체질 기반)
    if rng.rn2(20) > player_con {
        return PolymorphResult::SystemShock {
            damage: rng.rn2(30) + 10,
        };
    }

    let form = generate_random_form(player_level, rng);
    let turns = rng.rn2(500) + 500;

    PolymorphResult::Transformed { form, turns }
}

// =============================================================================
// [2] 변신 능력 — poly_abilities (polyself.c L500-800)
// =============================================================================

/// [v2.30.0 94-4] 변신 형태 능력 확인
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolyAbilities {
    pub can_cast_spells: bool,
    pub can_use_items: bool,
    pub can_wear_armor: bool,
    pub can_open_doors: bool,
    pub natural_ac: i32,
    pub natural_attacks: Vec<String>,
    pub special_ability: Option<String>,
}

/// [v2.30.0 94-4] 변신 형태 능력 산정
pub fn poly_form_abilities(form: &PolyForm) -> PolyAbilities {
    let can_use = form.has_hands;
    let special = if form.can_fly {
        Some("비행".to_string())
    } else if form.can_swim {
        Some("수영".to_string())
    } else {
        None
    };

    let mut attacks = form.attacks.clone();
    if form.monster_name == "드래곤" {
        attacks.push("브레스 공격".to_string());
    }
    if form.monster_name == "트롤" {
        attacks.push("재생".to_string());
    }

    PolyAbilities {
        can_cast_spells: form.is_humanoid,
        can_use_items: can_use,
        can_wear_armor: form.is_humanoid,
        can_open_doors: form.has_hands,
        natural_ac: form.ac,
        natural_attacks: attacks,
        special_ability: special,
    }
}

// =============================================================================
// [3] HP 계산 — poly_hp (polyself.c L800-1000)
// =============================================================================

/// [v2.30.0 94-4] 변신 HP 계산
pub fn calculate_poly_hp(form: &PolyForm, player_level: i32, rng: &mut NetHackRng) -> i32 {
    let dice = form.hp_dice.max(1);
    let mut hp = 0;
    for _ in 0..dice {
        hp += rng.rn2(8) + 1;
    }
    // 플레이어 레벨 보너스
    hp += player_level / 2;
    hp.max(1)
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

    #[test]
    fn test_polymorph_success() {
        let mut success = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = polymorph_self(15, 16, false, false, &mut rng);
            if matches!(result, PolymorphResult::Transformed { .. }) {
                success = true;
                break;
            }
        }
        assert!(success);
    }

    #[test]
    fn test_polymorph_resisted() {
        let mut resisted = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = polymorph_self(15, 16, false, true, &mut rng);
            if matches!(result, PolymorphResult::Resisted) {
                resisted = true;
                break;
            }
        }
        assert!(resisted);
    }

    #[test]
    fn test_system_shock() {
        let mut shocked = false;
        for seed in 0..30u64 {
            let mut rng = NetHackRng::new(seed);
            let result = polymorph_self(15, 3, false, false, &mut rng); // 낮은 CON
            if matches!(result, PolymorphResult::SystemShock { .. }) {
                shocked = true;
                break;
            }
        }
        assert!(shocked);
    }

    #[test]
    fn test_form_abilities_dragon() {
        let form = PolyForm {
            monster_name: "드래곤".to_string(),
            monster_level: 20,
            hp_dice: 8,
            ac: 2,
            speed: 12,
            attacks: vec!["물기".to_string()],
            resistances: vec!["화염".to_string()],
            can_fly: true,
            can_swim: false,
            has_hands: false,
            is_humanoid: false,
        };
        let abilities = poly_form_abilities(&form);
        assert!(!abilities.can_cast_spells);
        assert!(abilities
            .natural_attacks
            .iter()
            .any(|a| a.contains("브레스")));
        assert_eq!(abilities.special_ability, Some("비행".to_string()));
    }

    #[test]
    fn test_form_abilities_humanoid() {
        let form = PolyForm {
            monster_name: "자이언트".to_string(),
            monster_level: 15,
            hp_dice: 6,
            ac: 4,
            speed: 8,
            attacks: vec!["주먹".to_string()],
            resistances: vec![],
            can_fly: false,
            can_swim: false,
            has_hands: true,
            is_humanoid: true,
        };
        let abilities = poly_form_abilities(&form);
        assert!(abilities.can_cast_spells);
        assert!(abilities.can_wear_armor);
    }

    #[test]
    fn test_poly_hp() {
        let mut rng = test_rng();
        let form = PolyForm {
            monster_name: "트롤".to_string(),
            monster_level: 12,
            hp_dice: 6,
            ac: 2,
            speed: 10,
            attacks: vec![],
            resistances: vec![],
            can_fly: false,
            can_swim: false,
            has_hands: true,
            is_humanoid: false,
        };
        let hp = calculate_poly_hp(&form, 10, &mut rng);
        assert!(hp >= 6); // 최소 6d1 + 5
    }
}
