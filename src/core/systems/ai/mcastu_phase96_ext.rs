// ============================================================================
// [v2.32.0 Phase 96-1] 몬스터 마법 확장 (mcastu_phase96_ext.rs)
// 원본: NetHack 3.6.7 src/mcastu.c L200-1000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 마법 카테고리 — spell_category (mcastu.c L200-400)
// =============================================================================

/// [v2.32.0 96-1] 몬스터 마법 카테고리
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonSpellCategory {
    Attack,
    Healing,
    Summoning,
    Curse,
    Utility,
    Special,
}

/// [v2.32.0 96-1] 몬스터 마법 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonSpellResult {
    /// 공격 마법 (대상에 데미지)
    DamageSpell { damage: i32, element: String },
    /// 자가 치유
    SelfHeal { amount: i32 },
    /// 소환
    Summon { monster: String, count: i32 },
    /// 저주 (장비/상태)
    CurseTarget { effect: String, turns: i32 },
    /// 유틸리티 (텔레포트, 가속 등)
    UtilityEffect { effect: String },
    /// 특수 (파괴 광선, 지진 등)
    SpecialSpell { name: String, effect: String },
    /// 시전 실패
    Fizzled,
}

/// [v2.32.0 96-1] 몬스터 공격 마법
/// 원본: mcastu.c castmu()
pub fn cast_monster_attack_spell(
    caster_level: i32,
    caster_int: i32,
    target_mr: i32, // 마법 저항
    rng: &mut NetHackRng,
) -> MonSpellResult {
    // 시전 성공 확인
    let cast_chance = caster_level * 3 + caster_int * 2;
    if rng.rn2(100) >= cast_chance.min(95) {
        return MonSpellResult::Fizzled;
    }

    let spells = [
        ("마법 미사일", "마법"),
        ("화염구", "화염"),
        ("냉기 원뿔", "냉기"),
        ("번개", "전기"),
        ("산성 화살", "산"),
        ("정신 파괴", "정신"),
    ];

    let idx = rng.rn2(spells.len() as i32) as usize;
    let (name, element) = spells[idx];

    // 마법 저항 체크
    if target_mr > 0 && rng.rn2(100) < target_mr {
        return MonSpellResult::DamageSpell {
            damage: 0,
            element: format!("{} (저항됨)", element),
        };
    }

    let base_dmg = caster_level + rng.rn2(caster_level / 2 + 1);
    MonSpellResult::DamageSpell {
        damage: base_dmg.max(1),
        element: element.to_string(),
    }
}

/// [v2.32.0 96-1] 몬스터 치유 마법
pub fn cast_monster_heal(
    caster_level: i32,
    current_hp: i32,
    max_hp: i32,
    rng: &mut NetHackRng,
) -> MonSpellResult {
    if current_hp >= max_hp {
        return MonSpellResult::Fizzled;
    }
    let heal = rng.rn2(caster_level * 2) + caster_level;
    MonSpellResult::SelfHeal {
        amount: heal.min(max_hp - current_hp),
    }
}

/// [v2.32.0 96-1] 몬스터 소환 마법
pub fn cast_monster_summon(caster_level: i32, rng: &mut NetHackRng) -> MonSpellResult {
    let summons = ["코볼트", "오크", "스켈레톤", "좀비", "임프", "바실리스크"];
    let idx = rng.rn2(summons.len() as i32) as usize;
    let count = rng.rn2(3) + 1;

    MonSpellResult::Summon {
        monster: summons[idx].to_string(),
        count,
    }
}

/// [v2.32.0 96-1] 몬스터 저주 마법
pub fn cast_monster_curse(target_mr: i32, rng: &mut NetHackRng) -> MonSpellResult {
    if target_mr > 0 && rng.rn2(100) < target_mr {
        return MonSpellResult::Fizzled;
    }

    let curses = [
        ("실명", 10),
        ("혼란", 8),
        ("감속", 15),
        ("독", 5),
        ("약화", 12),
        ("침묵", 10),
    ];
    let idx = rng.rn2(curses.len() as i32) as usize;
    let (effect, turns) = curses[idx];

    MonSpellResult::CurseTarget {
        effect: effect.to_string(),
        turns,
    }
}

// =============================================================================
// [2] 이름 짓기 확장 — naming (do_name.c 핵심 로직)
// =============================================================================

/// [v2.32.0 96-1] 이름 짓기 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingResult {
    Named { old_name: String, new_name: String },
    AlreadyNamed { current: String },
    CannotName { reason: String },
    ArtifactNamed { artifact: String },
}

/// [v2.32.0 96-1] 아이템 이름 짓기
pub fn name_item(
    current_name: &str,
    new_name: &str,
    is_artifact: bool,
    is_unique: bool,
) -> NamingResult {
    // 아티팩트는 이름 변경 불가
    if is_artifact {
        return NamingResult::CannotName {
            reason: "아티팩트의 이름은 변경할 수 없다.".to_string(),
        };
    }

    // 빈 이름 금지
    if new_name.is_empty() {
        return NamingResult::CannotName {
            reason: "이름이 비어있다.".to_string(),
        };
    }

    // 아티팩트 이름 시도 감지
    let artifact_names = [
        "Excalibur",
        "Mjollnir",
        "Stormbringer",
        "Grayswandir",
        "Frost Brand",
        "Fire Brand",
        "Sting",
        "Orcrist",
    ];
    for art in &artifact_names {
        if new_name.to_lowercase() == art.to_lowercase() {
            return NamingResult::ArtifactNamed {
                artifact: art.to_string(),
            };
        }
    }

    NamingResult::Named {
        old_name: current_name.to_string(),
        new_name: new_name.to_string(),
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

    #[test]
    fn test_attack_spell() {
        let mut hit = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = cast_monster_attack_spell(15, 18, 0, &mut rng);
            if let MonSpellResult::DamageSpell { damage, .. } = result {
                if damage > 0 {
                    hit = true;
                    break;
                }
            }
        }
        assert!(hit);
    }

    #[test]
    fn test_heal_spell() {
        let mut rng = test_rng();
        let result = cast_monster_heal(10, 30, 100, &mut rng);
        assert!(matches!(result, MonSpellResult::SelfHeal { .. }));
    }

    #[test]
    fn test_heal_full_hp() {
        let mut rng = test_rng();
        let result = cast_monster_heal(10, 100, 100, &mut rng);
        assert!(matches!(result, MonSpellResult::Fizzled));
    }

    #[test]
    fn test_summon() {
        let mut rng = test_rng();
        let result = cast_monster_summon(10, &mut rng);
        if let MonSpellResult::Summon { count, .. } = result {
            assert!(count >= 1 && count <= 3);
        }
    }

    #[test]
    fn test_curse_resisted() {
        let mut resisted = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = cast_monster_curse(90, &mut rng); // 높은 MR
            if matches!(result, MonSpellResult::Fizzled) {
                resisted = true;
                break;
            }
        }
        assert!(resisted);
    }

    #[test]
    fn test_naming() {
        let result = name_item("검", "듀란달", false, false);
        assert!(matches!(result, NamingResult::Named { .. }));
    }

    #[test]
    fn test_naming_artifact_block() {
        let result = name_item("검", "새이름", true, false);
        assert!(matches!(result, NamingResult::CannotName { .. }));
    }

    #[test]
    fn test_naming_detect_artifact() {
        let result = name_item("검", "Excalibur", false, false);
        assert!(matches!(result, NamingResult::ArtifactNamed { .. }));
    }
}
