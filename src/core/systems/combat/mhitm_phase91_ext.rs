// ============================================================================
// [v2.27.0 Phase 91-7] 몬스터 간 전투 확장 (mhitm_phase91_ext.rs)
// 원본: NetHack 3.6.7 src/mhitm.c L400-900 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 몬스터 대 몬스터 명중 판정 (mhitm.c L400-500)
// =============================================================================

/// [v2.27.0 91-7] 몬스터간 전투 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonVsMonResult {
    /// 명중 + 데미지
    Hit { damage: i32, message: String },
    /// 빗나감
    Miss { message: String },
    /// 방어자 사망
    Kill { message: String },
    /// 특수 효과 (석화/독/드레인 등)
    SpecialEffect { effect: String },
    /// 도주
    Flee { message: String },
}

/// [v2.27.0 91-7] 몬스터간 전투 입력
#[derive(Debug, Clone)]
pub struct MonVsMonInput {
    pub attacker_name: String,
    pub defender_name: String,
    pub attacker_level: i32,
    pub defender_level: i32,
    pub attacker_ac: i32,
    pub defender_ac: i32,
    pub attack_damage_dice: i32,
    pub attack_damage_sides: i32,
    pub defender_hp: i32,
    pub is_tame_attacker: bool,
    pub is_tame_defender: bool,
}

/// [v2.27.0 91-7] 몬스터간 전투 해결
/// 원본: mhitm.c mattackm()
pub fn resolve_mon_vs_mon(input: &MonVsMonInput, rng: &mut NetHackRng) -> MonVsMonResult {
    // 길들인 몬스터끼리 → 싸우지 않음 (도주)
    if input.is_tame_attacker && input.is_tame_defender {
        return MonVsMonResult::Flee {
            message: format!(
                "{}이(가) {}에게서 물러났다.",
                input.attacker_name, input.defender_name
            ),
        };
    }

    // 명중 판정: 1d20 + attacker_level >= defender_ac + 10
    let roll = rng.rn2(20) + 1;
    let to_hit = roll + input.attacker_level;
    let target = input.defender_ac + 10;

    if to_hit < target {
        return MonVsMonResult::Miss {
            message: format!("{}의 공격이 빗나갔다.", input.attacker_name),
        };
    }

    // 데미지 계산
    let mut damage = 0;
    for _ in 0..input.attack_damage_dice {
        damage += rng.rn2(input.attack_damage_sides) + 1;
    }
    damage = damage.max(1);

    // 사망 판정
    if damage >= input.defender_hp {
        return MonVsMonResult::Kill {
            message: format!(
                "{}이(가) {}을(를) 쓰러뜨렸다!",
                input.attacker_name, input.defender_name
            ),
        };
    }

    MonVsMonResult::Hit {
        damage,
        message: format!(
            "{}이(가) {}을(를) 공격했다! ({}데미지)",
            input.attacker_name, input.defender_name, damage
        ),
    }
}

// =============================================================================
// [2] 몬스터 간 특수 전투 — special_mon_attack (mhitm.c L600-900)
// =============================================================================

/// [v2.27.0 91-7] 몬스터 특수 공격 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonSpecialAttack {
    Poison { damage: i32 },
    Petrify,
    LevelDrain,
    Acid { damage: i32 },
    Engulf,
    Steal,
    Paralyze { turns: i32 },
    None,
}

/// [v2.27.0 91-7] 몬스터 간 특수 공격 판정
pub fn mon_special_attack(
    attack_type: i32,
    attacker_level: i32,
    defender_has_resist: bool,
    rng: &mut NetHackRng,
) -> MonSpecialAttack {
    if defender_has_resist {
        return MonSpecialAttack::None;
    }

    match attack_type {
        1 => MonSpecialAttack::Poison {
            damage: rng.rn2(attacker_level / 2 + 1).max(1),
        },
        2 => MonSpecialAttack::Petrify,
        3 => MonSpecialAttack::LevelDrain,
        4 => MonSpecialAttack::Acid {
            damage: rng.rn2(attacker_level) + 1,
        },
        5 => MonSpecialAttack::Engulf,
        6 => MonSpecialAttack::Steal,
        7 => MonSpecialAttack::Paralyze {
            turns: rng.rn2(3) + 1,
        },
        _ => MonSpecialAttack::None,
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

    fn test_input() -> MonVsMonInput {
        MonVsMonInput {
            attacker_name: "고블린".to_string(),
            defender_name: "오크".to_string(),
            attacker_level: 5,
            defender_level: 3,
            attacker_ac: 5,
            defender_ac: 5,
            attack_damage_dice: 1,
            attack_damage_sides: 6,
            defender_hp: 20,
            is_tame_attacker: false,
            is_tame_defender: false,
        }
    }

    #[test]
    fn test_mon_vs_mon_basic() {
        let mut hit = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = resolve_mon_vs_mon(&test_input(), &mut rng);
            if matches!(result, MonVsMonResult::Hit { .. }) {
                hit = true;
                break;
            }
        }
        assert!(hit, "20시드 중 하나는 명중");
    }

    #[test]
    fn test_tame_vs_tame_flee() {
        let mut rng = test_rng();
        let mut input = test_input();
        input.is_tame_attacker = true;
        input.is_tame_defender = true;
        assert!(matches!(
            resolve_mon_vs_mon(&input, &mut rng),
            MonVsMonResult::Flee { .. }
        ));
    }

    #[test]
    fn test_kill_low_hp() {
        let mut got_kill = false;
        for seed in 0..30u64 {
            let mut rng = NetHackRng::new(seed);
            let mut input = test_input();
            input.defender_hp = 1;
            let result = resolve_mon_vs_mon(&input, &mut rng);
            if matches!(result, MonVsMonResult::Kill { .. }) {
                got_kill = true;
                break;
            }
        }
        assert!(got_kill, "HP 1이면 쓰러져야 함");
    }

    #[test]
    fn test_special_poison() {
        let mut rng = test_rng();
        let result = mon_special_attack(1, 10, false, &mut rng);
        assert!(matches!(result, MonSpecialAttack::Poison { .. }));
    }

    #[test]
    fn test_special_resisted() {
        let mut rng = test_rng();
        let result = mon_special_attack(2, 10, true, &mut rng);
        assert_eq!(result, MonSpecialAttack::None);
    }
}
