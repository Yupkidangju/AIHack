// ============================================================================
// [v2.28.0 R16-2] 몬스터 마법 (mcastu_ext.rs)
// 원본: NetHack 3.6.7 mcastu.c (792줄)
// 몬스터의 공격/방어/소환 주문 시스템
// ============================================================================

use crate::util::rng::NetHackRng;

/// [v2.28.0 R16-2] 몬스터 주문 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonsterSpell {
    // 공격
    MagicMissile { damage: i32 },
    FireBolt { damage: i32 },
    IceBolt { damage: i32 },
    LightningBolt { damage: i32 },
    AcidSplash { damage: i32 },
    DrainLife { amount: i32 },
    // 상태이상
    Blind { turns: i32 },
    Paralyze { turns: i32 },
    Confuse { turns: i32 },
    Sleep { turns: i32 },
    // 소환
    SummonNasties { count: i32 },
    SummonUndead { count: i32 },
    // 방어
    HealSelf { amount: i32 },
    Teleport,
    Haste,
    Invisibility,
    // 특수
    CurseItems,
    DestroyArmor,
    RemoveBlessing,
}

/// [v2.28.0 R16-2] 주문 선택 입력
#[derive(Debug, Clone)]
pub struct CastInput {
    pub caster_level: i32,
    pub caster_hp_ratio: f64,
    pub distance_to_target: i32,
    pub target_has_mr: bool, // 마법 저항
    pub target_has_reflection: bool,
    pub is_covetous: bool, // 보물 탐욕 (위자드, 블라드 등)
}

/// [v2.28.0 R16-2] 공격 주문 선택 (원본: castmu)
pub fn choose_attack_spell(input: &CastInput, rng: &mut NetHackRng) -> MonsterSpell {
    let base_dmg = input.caster_level.max(1);

    // 반사 있으면 불/얼음/번개 회피
    if input.target_has_reflection {
        return MonsterSpell::DrainLife {
            amount: base_dmg / 2,
        };
    }

    let roll = rng.rn2(6);
    match roll {
        0 => MonsterSpell::MagicMissile {
            damage: rng.rn1(base_dmg, 1),
        },
        1 => MonsterSpell::FireBolt {
            damage: rng.rn1(base_dmg, 2),
        },
        2 => MonsterSpell::IceBolt {
            damage: rng.rn1(base_dmg, 2),
        },
        3 => MonsterSpell::LightningBolt {
            damage: rng.rn1(base_dmg, 3),
        },
        4 => MonsterSpell::Confuse {
            turns: rng.rn1(5, 1),
        },
        _ => MonsterSpell::Blind {
            turns: rng.rn1(10, 3),
        },
    }
}

/// [v2.28.0 R16-2] 방어 주문 선택 (원본: castmu defensive)
pub fn choose_defensive_spell(input: &CastInput, rng: &mut NetHackRng) -> MonsterSpell {
    if input.caster_hp_ratio < 0.3 {
        return MonsterSpell::HealSelf {
            amount: input.caster_level * 3,
        };
    }
    if input.caster_hp_ratio < 0.5 && input.is_covetous {
        return MonsterSpell::Teleport;
    }
    let roll = rng.rn2(3);
    match roll {
        0 => MonsterSpell::Haste,
        1 => MonsterSpell::Invisibility,
        _ => MonsterSpell::HealSelf {
            amount: input.caster_level * 2,
        },
    }
}

/// [v2.28.0 R16-2] 저주 주문 (원본: castmu curse)
pub fn choose_curse_spell(input: &CastInput, rng: &mut NetHackRng) -> MonsterSpell {
    if input.target_has_mr && rng.rn2(2) == 0 {
        return MonsterSpell::RemoveBlessing; // MR 무시
    }
    let roll = rng.rn2(3);
    match roll {
        0 => MonsterSpell::CurseItems,
        1 => MonsterSpell::DestroyArmor,
        _ => MonsterSpell::RemoveBlessing,
    }
}

/// [v2.28.0 R16-2] 통합 주문 결정 (원본: castmu)
pub fn decide_spell(input: &CastInput, rng: &mut NetHackRng) -> MonsterSpell {
    // HP 낮으면 방어 우선
    if input.caster_hp_ratio < 0.3 {
        return choose_defensive_spell(input, rng);
    }
    // 보물탐욕 몬스터는 소환 자주
    if input.is_covetous && rng.rn2(3) == 0 {
        return MonsterSpell::SummonNasties {
            count: rng.rn1(3, 1),
        };
    }
    // 원거리면 공격, 근접이면 저주
    if input.distance_to_target > 1 {
        choose_attack_spell(input, rng)
    } else {
        if rng.rn2(2) == 0 {
            choose_curse_spell(input, rng)
        } else {
            choose_attack_spell(input, rng)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_input() -> CastInput {
        CastInput {
            caster_level: 15,
            caster_hp_ratio: 0.8,
            distance_to_target: 5,
            target_has_mr: false,
            target_has_reflection: false,
            is_covetous: false,
        }
    }

    #[test]
    fn test_attack_spell() {
        let mut rng = NetHackRng::new(42);
        let spell = choose_attack_spell(&test_input(), &mut rng);
        assert!(matches!(
            spell,
            MonsterSpell::MagicMissile { .. }
                | MonsterSpell::FireBolt { .. }
                | MonsterSpell::IceBolt { .. }
                | MonsterSpell::LightningBolt { .. }
                | MonsterSpell::Confuse { .. }
                | MonsterSpell::Blind { .. }
        ));
    }

    #[test]
    fn test_reflection_avoidance() {
        let mut rng = NetHackRng::new(42);
        let input = CastInput {
            target_has_reflection: true,
            ..test_input()
        };
        let spell = choose_attack_spell(&input, &mut rng);
        assert!(matches!(spell, MonsterSpell::DrainLife { .. }));
    }

    #[test]
    fn test_defensive_low_hp() {
        let mut rng = NetHackRng::new(42);
        let input = CastInput {
            caster_hp_ratio: 0.2,
            ..test_input()
        };
        let spell = choose_defensive_spell(&input, &mut rng);
        assert!(matches!(spell, MonsterSpell::HealSelf { .. }));
    }

    #[test]
    fn test_decide_defensive_priority() {
        let mut rng = NetHackRng::new(42);
        let input = CastInput {
            caster_hp_ratio: 0.1,
            ..test_input()
        };
        let spell = decide_spell(&input, &mut rng);
        assert!(matches!(spell, MonsterSpell::HealSelf { .. }));
    }

    #[test]
    fn test_curse_spell() {
        let mut rng = NetHackRng::new(42);
        let spell = choose_curse_spell(&test_input(), &mut rng);
        assert!(matches!(
            spell,
            MonsterSpell::CurseItems | MonsterSpell::DestroyArmor | MonsterSpell::RemoveBlessing
        ));
    }
}
