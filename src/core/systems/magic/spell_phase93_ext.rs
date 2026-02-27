// ============================================================================
// [v2.29.0 Phase 93-2] 마법 시스템 확장 (spell_phase93_ext.rs)
// 원본: NetHack 3.6.7 src/spell.c L400-1200 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 주문 시전 — cast_spell (spell.c L400-700)
// =============================================================================

/// [v2.29.0 93-2] 주문 학파
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellSchool {
    Attack,      // 공격 마법
    Healing,     // 치유 마법
    Divination,  // 점술 마법
    Enchantment, // 부여 마법
    Clerical,    // 성직 마법
    Escape,      // 도주 마법
    Matter,      // 물질 마법
}

/// [v2.29.0 93-2] 주문 정의
#[derive(Debug, Clone)]
pub struct SpellEntry {
    pub name: String,
    pub school: SpellSchool,
    pub level: i32,
    pub energy_cost: i32,
    pub max_range: i32,
    pub study_count: i32,
    pub retention_turns: i32, // 남은 기억 턴
}

/// [v2.29.0 93-2] 시전 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CastResult {
    /// 시전 성공
    Success { energy_used: i32, effect: String },
    /// 에너지 부족
    NotEnoughEnergy { required: i32, current: i32 },
    /// 주문 잊음
    Forgotten,
    /// 시전 실패 (레벨 부족)
    Failed { reason: String },
    /// 역효과
    Backfire { damage: i32, effect: String },
}

/// [v2.29.0 93-2] 주문 시전 판정
/// 원본: spell.c spelleffects()
pub fn cast_spell(
    spell: &SpellEntry,
    caster_level: i32,
    caster_int: i32,
    current_energy: i32,
    is_confused: bool,
    role_bonus: bool, // 위저드 등 마법 직업
    rng: &mut NetHackRng,
) -> CastResult {
    // 주문 잊음
    if spell.retention_turns <= 0 {
        return CastResult::Forgotten;
    }

    // 에너지 확인
    let cost = if role_bonus {
        spell.energy_cost * 3 / 4
    } else {
        spell.energy_cost
    };
    if current_energy < cost {
        return CastResult::NotEnoughEnergy {
            required: cost,
            current: current_energy,
        };
    }

    // 실패 확률: 주문 레벨 vs 시전자 레벨
    let fail_chance = (spell.level * 10 - caster_level * 3 - caster_int * 2).max(5);
    let roll = rng.rn2(100);
    if roll < fail_chance && !role_bonus {
        // 혼란 시 역효과
        if is_confused {
            return CastResult::Backfire {
                damage: rng.rn2(spell.level * 2) + 1,
                effect: "마법이 역류했다!".to_string(),
            };
        }
        return CastResult::Failed {
            reason: format!("{}을(를) 시전하는데 집중이 흐트러졌다.", spell.name),
        };
    }

    CastResult::Success {
        energy_used: cost,
        effect: format!("{}을(를) 시전했다!", spell.name),
    }
}

// =============================================================================
// [2] 주문 학습 — study_spell (spell.c L700-900)
// =============================================================================

/// [v2.29.0 93-2] 학습 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudyResult {
    /// 신규 학습
    Learned { spell_name: String },
    /// 기억 갱신
    Refreshed {
        spell_name: String,
        new_retention: i32,
    },
    /// 숙련도 증가
    Improved {
        spell_name: String,
        new_study_count: i32,
    },
    /// 학습 실패 (레벨 부족)
    TooComplex { required_level: i32 },
    /// 이미 최대
    AlreadyMastered,
}

/// [v2.29.0 93-2] 주문 학습
pub fn study_spell(
    spell_name: &str,
    spell_level: i32,
    caster_level: i32,
    caster_int: i32,
    current_study_count: i32,
    max_study: i32,
    rng: &mut NetHackRng,
) -> StudyResult {
    // 레벨 확인
    if spell_level > caster_level + 3 {
        return StudyResult::TooComplex {
            required_level: spell_level - 3,
        };
    }

    // 최대 학습
    if current_study_count >= max_study {
        return StudyResult::AlreadyMastered;
    }

    // 첫 학습
    if current_study_count == 0 {
        return StudyResult::Learned {
            spell_name: spell_name.to_string(),
        };
    }

    // INT 기반 학습 확률
    let learn_chance = caster_int * 5 + rng.rn2(20);
    if learn_chance > 50 {
        StudyResult::Improved {
            spell_name: spell_name.to_string(),
            new_study_count: current_study_count + 1,
        }
    } else {
        StudyResult::Refreshed {
            spell_name: spell_name.to_string(),
            new_retention: 20000 + rng.rn2(10000),
        }
    }
}

// =============================================================================
// [3] 주문 효과 계산 — spell_damage (spell.c L900-1200)
// =============================================================================

/// [v2.29.0 93-2] 공격 마법 데미지 계산
pub fn spell_damage(
    school: SpellSchool,
    spell_level: i32,
    caster_level: i32,
    is_skilled: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let base = match school {
        SpellSchool::Attack => {
            // 공격 마법: 레벨 * d6
            let dice = (spell_level + 1).min(8);
            let mut total = 0;
            for _ in 0..dice {
                total += rng.rn2(6) + 1;
            }
            total
        }
        SpellSchool::Matter => {
            // 물질 마법: 레벨 * d4
            let dice = spell_level.min(6);
            let mut total = 0;
            for _ in 0..dice {
                total += rng.rn2(4) + 1;
            }
            total
        }
        _ => 0, // 비공격 마법
    };

    // 숙련도 보너스
    let skill_bonus = if is_skilled { caster_level / 2 } else { 0 };

    base + skill_bonus
}

/// [v2.29.0 93-2] 치유 마법 회복량 계산
pub fn spell_heal_amount(
    spell_level: i32,
    caster_level: i32,
    is_blessed: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let base = spell_level * 4 + rng.rn2(caster_level) + caster_level / 2;
    if is_blessed {
        base * 3 / 2
    } else {
        base
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

    fn test_spell() -> SpellEntry {
        SpellEntry {
            name: "마법 미사일".to_string(),
            school: SpellSchool::Attack,
            level: 2,
            energy_cost: 5,
            max_range: 10,
            study_count: 3,
            retention_turns: 10000,
        }
    }

    #[test]
    fn test_cast_success() {
        let mut rng = test_rng();
        let spell = test_spell();
        let mut success = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = cast_spell(&spell, 10, 16, 50, false, true, &mut rng);
            if matches!(result, CastResult::Success { .. }) {
                success = true;
                break;
            }
        }
        assert!(success);
    }

    #[test]
    fn test_cast_no_energy() {
        let mut rng = test_rng();
        let spell = test_spell();
        let result = cast_spell(&spell, 10, 16, 2, false, false, &mut rng);
        assert!(matches!(result, CastResult::NotEnoughEnergy { .. }));
    }

    #[test]
    fn test_cast_forgotten() {
        let mut rng = test_rng();
        let mut spell = test_spell();
        spell.retention_turns = 0;
        let result = cast_spell(&spell, 10, 16, 50, false, false, &mut rng);
        assert!(matches!(result, CastResult::Forgotten));
    }

    #[test]
    fn test_study_new() {
        let mut rng = test_rng();
        let result = study_spell("마법 미사일", 2, 10, 16, 0, 10, &mut rng);
        assert!(matches!(result, StudyResult::Learned { .. }));
    }

    #[test]
    fn test_study_too_complex() {
        let mut rng = test_rng();
        let result = study_spell("바람의 소용돌이", 10, 3, 16, 0, 10, &mut rng);
        assert!(matches!(result, StudyResult::TooComplex { .. }));
    }

    #[test]
    fn test_study_mastered() {
        let mut rng = test_rng();
        let result = study_spell("마법 미사일", 2, 10, 16, 10, 10, &mut rng);
        assert!(matches!(result, StudyResult::AlreadyMastered));
    }

    #[test]
    fn test_attack_damage() {
        let mut rng = test_rng();
        let dmg = spell_damage(SpellSchool::Attack, 3, 10, true, &mut rng);
        assert!(dmg > 0);
    }

    #[test]
    fn test_heal_amount() {
        let mut rng = test_rng();
        let heal = spell_heal_amount(3, 10, false, &mut rng);
        assert!(heal > 0);
    }

    #[test]
    fn test_heal_blessed() {
        let mut rng1 = NetHackRng::new(42);
        let mut rng2 = NetHackRng::new(42);
        let normal = spell_heal_amount(3, 10, false, &mut rng1);
        let blessed = spell_heal_amount(3, 10, true, &mut rng2);
        assert!(blessed >= normal);
    }
}
