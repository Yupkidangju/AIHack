// ============================================================================
// [v2.27.0 Phase 91-1] 몬스터 마법 확장 (mcastu_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/mcastu.c L800-2000 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 마법 선택 — choose_spell (mcastu.c L800-1200)
// =============================================================================

/// [v2.27.0 91-1] 몬스터 주문 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterSpell {
    MagicMissile,
    FireBolt,
    IceBolt,
    LightningBolt,
    AcidSplash,
    Sleep,
    Blind,
    Confuse,
    Paralyze,
    Drain,
    Curse,
    Summon,
    Haste,
    Heal,
    Teleport,
    Destroy,
}

/// [v2.27.0 91-1] 몬스터 주문 선택 입력
#[derive(Debug, Clone)]
pub struct SpellSelectionInput {
    pub monster_level: i32,
    pub is_hostile: bool,
    pub target_in_range: bool,
    pub self_hp_low: bool,
    pub target_has_mr: bool,
    pub alignment: i32, // -1=혼돈, 0=중립, 1=질서
}

/// [v2.27.0 91-1] 몬스터 주문 선택
/// 원본: mcastu.c castmu() 분기
pub fn choose_monster_spell(input: &SpellSelectionInput, rng: &mut NetHackRng) -> MonsterSpell {
    // HP 낮으면 회복/도주 우선
    if input.self_hp_low {
        let heal_chance = rng.rn2(3);
        return match heal_chance {
            0 => MonsterSpell::Heal,
            1 => MonsterSpell::Teleport,
            _ => MonsterSpell::Haste,
        };
    }

    // 대상이 MR 있으면 직접 데미지 공격
    if input.target_has_mr {
        let spell = rng.rn2(5);
        return match spell {
            0 => MonsterSpell::MagicMissile, // MR에도 관통
            1 => MonsterSpell::FireBolt,
            2 => MonsterSpell::IceBolt,
            3 => MonsterSpell::LightningBolt,
            _ => MonsterSpell::AcidSplash,
        };
    }

    // 레벨 기반 주문 선택
    if input.monster_level >= 20 {
        let spell = rng.rn2(6);
        match spell {
            0 => MonsterSpell::Destroy,
            1 => MonsterSpell::Drain,
            2 => MonsterSpell::Summon,
            3 => MonsterSpell::Paralyze,
            4 => MonsterSpell::Curse,
            _ => MonsterSpell::LightningBolt,
        }
    } else if input.monster_level >= 10 {
        let spell = rng.rn2(6);
        match spell {
            0 => MonsterSpell::FireBolt,
            1 => MonsterSpell::IceBolt,
            2 => MonsterSpell::Sleep,
            3 => MonsterSpell::Blind,
            4 => MonsterSpell::Confuse,
            _ => MonsterSpell::MagicMissile,
        }
    } else {
        let spell = rng.rn2(4);
        match spell {
            0 => MonsterSpell::MagicMissile,
            1 => MonsterSpell::Sleep,
            2 => MonsterSpell::Confuse,
            _ => MonsterSpell::Blind,
        }
    }
}

// =============================================================================
// [2] 몬스터 주문 효과 — spell_effect (mcastu.c L1200-1800)
// =============================================================================

/// [v2.27.0 91-1] 주문 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellEffectResult {
    Damage { amount: i32, element: String },
    StatusEffect { effect: String, turns: i32 },
    Summon { count: i32 },
    SelfBuff { effect: String },
    CurseItems { count: i32 },
    DrainLevel { levels: i32 },
    DestroyArmor,
    NoEffect { reason: String },
}

/// [v2.27.0 91-1] 몬스터 주문 효과 판정
/// 원본: mcastu.c castmu() 효과 분기
pub fn spell_effect(
    spell: MonsterSpell,
    caster_level: i32,
    target_mr: bool,
    target_resistance: &[String],
    rng: &mut NetHackRng,
) -> SpellEffectResult {
    match spell {
        MonsterSpell::MagicMissile => {
            let dmg = rng.rn2(caster_level) + caster_level / 2 + 1;
            SpellEffectResult::Damage {
                amount: dmg,
                element: "마법 미사일".to_string(),
            }
        }
        MonsterSpell::FireBolt => {
            if target_resistance.iter().any(|r| r == "fire") {
                return SpellEffectResult::NoEffect {
                    reason: "화염 저항으로 무효화".to_string(),
                };
            }
            let dmg = rng.rn2(caster_level * 2) + 5;
            SpellEffectResult::Damage {
                amount: dmg,
                element: "화염".to_string(),
            }
        }
        MonsterSpell::IceBolt => {
            if target_resistance.iter().any(|r| r == "cold") {
                return SpellEffectResult::NoEffect {
                    reason: "냉기 저항으로 무효화".to_string(),
                };
            }
            let dmg = rng.rn2(caster_level * 2) + 5;
            SpellEffectResult::Damage {
                amount: dmg,
                element: "냉기".to_string(),
            }
        }
        MonsterSpell::LightningBolt => {
            if target_resistance.iter().any(|r| r == "shock") {
                return SpellEffectResult::NoEffect {
                    reason: "전격 저항으로 무효화".to_string(),
                };
            }
            let dmg = rng.rn2(caster_level * 2) + 8;
            SpellEffectResult::Damage {
                amount: dmg,
                element: "전격".to_string(),
            }
        }
        MonsterSpell::AcidSplash => {
            let dmg = rng.rn2(caster_level) + 3;
            SpellEffectResult::Damage {
                amount: dmg,
                element: "산성".to_string(),
            }
        }
        MonsterSpell::Sleep => {
            if target_mr || target_resistance.iter().any(|r| r == "sleep") {
                return SpellEffectResult::NoEffect {
                    reason: "수면 저항".to_string(),
                };
            }
            SpellEffectResult::StatusEffect {
                effect: "수면".to_string(),
                turns: rng.rn2(10) + 5,
            }
        }
        MonsterSpell::Blind => {
            if target_mr {
                return SpellEffectResult::NoEffect {
                    reason: "MR로 저항".to_string(),
                };
            }
            SpellEffectResult::StatusEffect {
                effect: "실명".to_string(),
                turns: rng.rn2(20) + 10,
            }
        }
        MonsterSpell::Confuse => {
            if target_mr {
                return SpellEffectResult::NoEffect {
                    reason: "MR로 저항".to_string(),
                };
            }
            SpellEffectResult::StatusEffect {
                effect: "혼란".to_string(),
                turns: rng.rn2(10) + 5,
            }
        }
        MonsterSpell::Paralyze => {
            if target_mr {
                return SpellEffectResult::NoEffect {
                    reason: "MR로 저항".to_string(),
                };
            }
            SpellEffectResult::StatusEffect {
                effect: "마비".to_string(),
                turns: rng.rn2(4) + 2,
            }
        }
        MonsterSpell::Drain => {
            if target_mr {
                return SpellEffectResult::NoEffect {
                    reason: "MR로 저항".to_string(),
                };
            }
            SpellEffectResult::DrainLevel { levels: 1 }
        }
        MonsterSpell::Curse => {
            let count = rng.rn2(3) + 1;
            SpellEffectResult::CurseItems { count }
        }
        MonsterSpell::Summon => {
            let count = rng.rn2(3) + 1;
            SpellEffectResult::Summon { count }
        }
        MonsterSpell::Haste => SpellEffectResult::SelfBuff {
            effect: "가속".to_string(),
        },
        MonsterSpell::Heal => SpellEffectResult::SelfBuff {
            effect: format!("회복 ({}HP)", caster_level * 2),
        },
        MonsterSpell::Teleport => SpellEffectResult::SelfBuff {
            effect: "텔레포트".to_string(),
        },
        MonsterSpell::Destroy => {
            if target_mr && rng.rn2(2) == 0 {
                return SpellEffectResult::NoEffect {
                    reason: "MR로 일부 저항".to_string(),
                };
            }
            SpellEffectResult::DestroyArmor
        }
    }
}

// =============================================================================
// [3] 주문 시전 확률 — can_cast (mcastu.c L600-750)
// =============================================================================

/// [v2.27.0 91-1] 주문 시전 가능 여부
pub fn can_monster_cast(
    monster_level: i32,
    is_cancelled: bool,
    is_stunned: bool,
    is_confused: bool,
    rng: &mut NetHackRng,
) -> bool {
    if is_cancelled {
        return false;
    }
    // 기절/혼란 시 시전 실패 확률 50%
    if is_stunned || is_confused {
        return rng.rn2(2) == 0;
    }
    // 기본 시전 확률: 1/3
    rng.rn2(3) == 0
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
    fn test_spell_selection_low_hp() {
        let mut rng = test_rng();
        let input = SpellSelectionInput {
            monster_level: 15,
            is_hostile: true,
            target_in_range: true,
            self_hp_low: true,
            target_has_mr: false,
            alignment: -1,
        };
        let spell = choose_monster_spell(&input, &mut rng);
        assert!(matches!(
            spell,
            MonsterSpell::Heal | MonsterSpell::Teleport | MonsterSpell::Haste
        ));
    }

    #[test]
    fn test_spell_selection_vs_mr() {
        let mut rng = test_rng();
        let input = SpellSelectionInput {
            monster_level: 15,
            is_hostile: true,
            target_in_range: true,
            self_hp_low: false,
            target_has_mr: true,
            alignment: 0,
        };
        let spell = choose_monster_spell(&input, &mut rng);
        assert!(matches!(
            spell,
            MonsterSpell::MagicMissile
                | MonsterSpell::FireBolt
                | MonsterSpell::IceBolt
                | MonsterSpell::LightningBolt
                | MonsterSpell::AcidSplash
        ));
    }

    #[test]
    fn test_spell_fire_resisted() {
        let mut rng = test_rng();
        let resist = vec!["fire".to_string()];
        let result = spell_effect(MonsterSpell::FireBolt, 15, false, &resist, &mut rng);
        assert!(matches!(result, SpellEffectResult::NoEffect { .. }));
    }

    #[test]
    fn test_spell_sleep_mr() {
        let mut rng = test_rng();
        let result = spell_effect(MonsterSpell::Sleep, 10, true, &[], &mut rng);
        assert!(matches!(result, SpellEffectResult::NoEffect { .. }));
    }

    #[test]
    fn test_spell_magic_missile_damage() {
        let mut rng = test_rng();
        let result = spell_effect(MonsterSpell::MagicMissile, 15, false, &[], &mut rng);
        assert!(matches!(result, SpellEffectResult::Damage { .. }));
    }

    #[test]
    fn test_spell_summon() {
        let mut rng = test_rng();
        let result = spell_effect(MonsterSpell::Summon, 20, false, &[], &mut rng);
        assert!(matches!(result, SpellEffectResult::Summon { .. }));
    }

    #[test]
    fn test_can_cast_cancelled() {
        let mut rng = test_rng();
        assert!(!can_monster_cast(15, true, false, false, &mut rng));
    }

    #[test]
    fn test_can_cast_normal() {
        let mut can = false;
        for seed in 0..10u64 {
            let mut rng = NetHackRng::new(seed);
            if can_monster_cast(15, false, false, false, &mut rng) {
                can = true;
                break;
            }
        }
        assert!(can, "10시드 중 하나는 시전 성공");
    }
}
