// ============================================================================
// [v2.31.0 Phase 95-4] 전투 공식 확장 (combat_formula_phase95_ext.rs)
// 원본: NetHack 3.6.7 src/uhitm.c + mhitm.c 핵심 미이식 공식
// 순수 결과 패턴 — 전투 데미지/명중/크리티컬/반격 공식 통합
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 명중 공식 — attack_roll (uhitm.c L300-500)
// =============================================================================

/// [v2.31.0 95-4] 공격 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    Melee,
    Ranged,
    Spell,
    Touch,
    Gaze,
    Breath,
    Engulf,
    Kick,
    Headbutt,
}

/// [v2.31.0 95-4] 명중 판정 입력
#[derive(Debug, Clone)]
pub struct AttackRollInput {
    pub attacker_level: i32,
    pub attacker_str: i32,
    pub attacker_dex: i32,
    pub weapon_bonus: i32,
    pub skill_bonus: i32,
    pub target_ac: i32,
    pub attack_type: AttackType,
    pub is_two_weapon: bool,
    pub is_backstab: bool,
    pub is_riding: bool,
}

/// [v2.31.0 95-4] 명중 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttackRollResult {
    Hit { margin: i32 },
    CriticalHit { margin: i32 },
    Miss { margin: i32 },
    Fumble,
}

/// [v2.31.0 95-4] 명중 판정
pub fn attack_roll(input: &AttackRollInput, rng: &mut NetHackRng) -> AttackRollResult {
    // 대실패 (1d20 = 1)
    let roll = rng.rn2(20) + 1;
    if roll == 1 {
        return AttackRollResult::Fumble;
    }

    // 명중 보너스 계산
    let str_bonus = (input.attacker_str - 10) / 2;
    let dex_bonus = match input.attack_type {
        AttackType::Ranged => (input.attacker_dex - 10) / 2,
        _ => 0,
    };
    let level_bonus = input.attacker_level / 2;
    let backstab = if input.is_backstab { 4 } else { 0 };
    let riding = if input.is_riding { -2 } else { 0 };
    let two_weapon = if input.is_two_weapon { -2 } else { 0 };

    let total_bonus = str_bonus
        + dex_bonus
        + level_bonus
        + input.weapon_bonus
        + input.skill_bonus
        + backstab
        + riding
        + two_weapon;

    let to_hit = roll + total_bonus;
    let target = input.target_ac + 10;
    let margin = to_hit - target;

    // 대성공 (1d20 = 20)
    if roll == 20 || margin >= 10 {
        return AttackRollResult::CriticalHit { margin };
    }

    if margin >= 0 {
        AttackRollResult::Hit { margin }
    } else {
        AttackRollResult::Miss { margin }
    }
}

// =============================================================================
// [2] 데미지 공식 — damage_calc (uhitm.c L500-800)
// =============================================================================

/// [v2.31.0 95-4] 데미지 계산 입력
#[derive(Debug, Clone)]
pub struct DamageInput {
    pub weapon_dice: i32,  // 무기 주사위 면
    pub weapon_count: i32, // 주사위 개수
    pub str_bonus: i32,
    pub enchantment: i32,
    pub skill_bonus: i32,
    pub is_critical: bool,
    pub is_backstab: bool,
    pub vs_large: bool,     // 대형 적
    pub element_bonus: i32, // 원소 보너스
}

/// [v2.31.0 95-4] 데미지 계산
pub fn calculate_damage(input: &DamageInput, rng: &mut NetHackRng) -> i32 {
    let mut total = 0;

    // 기본 무기 데미지
    for _ in 0..input.weapon_count.max(1) {
        total += rng.rn2(input.weapon_dice.max(1)) + 1;
    }

    // 크리티컬 → 데미지 2배
    if input.is_critical {
        total *= 2;
    }

    // 뒤찌르기 → 데미지 3배
    if input.is_backstab {
        total *= 3;
    }

    // 보너스 추가
    total += input.str_bonus;
    total += input.enchantment;
    total += input.skill_bonus;
    total += input.element_bonus;

    // 대형 보너스
    if input.vs_large {
        total += input.weapon_dice / 2;
    }

    total.max(1)
}

// =============================================================================
// [3] 반격/반사 — counter (mhitm.c 반격 로직)
// =============================================================================

/// [v2.31.0 95-4] 반격 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CounterResult {
    Reflected { damage: i32 },
    Parried,
    Dodged,
    Thorns { damage: i32 },
    NoCounter,
}

/// [v2.31.0 95-4] 반격 판정
pub fn counter_check(
    defender_dex: i32,
    defender_level: i32,
    has_reflection: bool,
    has_thorns: bool,
    attack_type: AttackType,
    rng: &mut NetHackRng,
) -> CounterResult {
    // 반사 (원거리/주문/시선/브레스)
    if has_reflection
        && matches!(
            attack_type,
            AttackType::Ranged | AttackType::Spell | AttackType::Gaze | AttackType::Breath
        )
    {
        return CounterResult::Reflected {
            damage: rng.rn2(10) + 5,
        };
    }

    // 가시 갑옷
    if has_thorns
        && matches!(
            attack_type,
            AttackType::Melee | AttackType::Touch | AttackType::Engulf
        )
    {
        return CounterResult::Thorns {
            damage: rng.rn2(4) + 1,
        };
    }

    // 회피 (DEX 기반)
    let dodge_chance = defender_dex * 2 + defender_level;
    if rng.rn2(100) < dodge_chance / 3 {
        return CounterResult::Dodged;
    }

    // 패리 (레벨 기반)
    if rng.rn2(50) < defender_level {
        return CounterResult::Parried;
    }

    CounterResult::NoCounter
}

// =============================================================================
// [4] 경험치 계산 — exp_calc
// =============================================================================

/// [v2.31.0 95-4] 경험치 계산
pub fn calculate_exp_gain(
    monster_level: i32,
    monster_hp_max: i32,
    player_level: i32,
    is_unique: bool,
) -> i32 {
    let base = monster_level * 5 + monster_hp_max / 2;
    let level_diff = (monster_level - player_level).max(0);
    let difficulty_bonus = level_diff * 10;
    let unique_bonus = if is_unique { base } else { 0 };

    (base + difficulty_bonus + unique_bonus).max(1)
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

    fn base_attack() -> AttackRollInput {
        AttackRollInput {
            attacker_level: 10,
            attacker_str: 16,
            attacker_dex: 14,
            weapon_bonus: 3,
            skill_bonus: 2,
            target_ac: 5,
            attack_type: AttackType::Melee,
            is_two_weapon: false,
            is_backstab: false,
            is_riding: false,
        }
    }

    #[test]
    fn test_attack_hit() {
        let input = base_attack();
        let mut hit = false;
        for seed in 0..20u64 {
            let mut rng = NetHackRng::new(seed);
            let result = attack_roll(&input, &mut rng);
            if matches!(
                result,
                AttackRollResult::Hit { .. } | AttackRollResult::CriticalHit { .. }
            ) {
                hit = true;
                break;
            }
        }
        assert!(hit);
    }

    #[test]
    fn test_fumble() {
        let input = base_attack();
        let mut fumbled = false;
        for seed in 0..100u64 {
            let mut rng = NetHackRng::new(seed);
            let result = attack_roll(&input, &mut rng);
            if matches!(result, AttackRollResult::Fumble) {
                fumbled = true;
                break;
            }
        }
        assert!(fumbled);
    }

    #[test]
    fn test_damage_normal() {
        let mut rng = test_rng();
        let input = DamageInput {
            weapon_dice: 8,
            weapon_count: 1,
            str_bonus: 3,
            enchantment: 2,
            skill_bonus: 1,
            is_critical: false,
            is_backstab: false,
            vs_large: false,
            element_bonus: 0,
        };
        let dmg = calculate_damage(&input, &mut rng);
        assert!(dmg >= 7); // 최소 1+3+2+1
    }

    #[test]
    fn test_damage_critical() {
        let mut rng = test_rng();
        let input = DamageInput {
            weapon_dice: 8,
            weapon_count: 1,
            str_bonus: 3,
            enchantment: 2,
            skill_bonus: 1,
            is_critical: true,
            is_backstab: false,
            vs_large: false,
            element_bonus: 0,
        };
        let dmg = calculate_damage(&input, &mut rng);
        assert!(dmg >= 8); // 크리티컬 2배
    }

    #[test]
    fn test_counter_reflect() {
        let mut rng = test_rng();
        let result = counter_check(14, 10, true, false, AttackType::Spell, &mut rng);
        assert!(matches!(result, CounterResult::Reflected { .. }));
    }

    #[test]
    fn test_counter_thorns() {
        let mut rng = test_rng();
        let result = counter_check(14, 10, false, true, AttackType::Melee, &mut rng);
        assert!(matches!(result, CounterResult::Thorns { .. }));
    }

    #[test]
    fn test_exp_unique() {
        let normal = calculate_exp_gain(10, 50, 10, false);
        let unique = calculate_exp_gain(10, 50, 10, true);
        assert!(unique > normal);
    }

    #[test]
    fn test_exp_higher_level() {
        let easy = calculate_exp_gain(5, 30, 10, false);
        let hard = calculate_exp_gain(15, 80, 10, false);
        assert!(hard > easy);
    }
}
