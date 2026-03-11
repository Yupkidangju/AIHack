// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 방은호 (Eunho Bang). Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-20] 주문 확장2 (spell_ext2.rs)
// 원본: NetHack 3.6.7 spell.c (주문 성공률, 기억 보존율, 역발/배컬, 에너지)
// ============================================================================

use crate::util::hacklib_ext::isqrt;

// =============================================================================
// [1] 주문 성공률 계산 (원본: spell.c:1712-1825 percent_success)
// =============================================================================

/// [v2.22.0 R34-20] 주문 성공률 계산 입력
#[derive(Debug, Clone)]
pub struct SpellSuccessInput {
    // 역할 기반값
    /// 역할 기본 시전 능력 (urole.spelbase)
    pub role_spelbase: i32,
    /// 역할 회복 주문 보너스 (urole.spelheal)
    pub role_spelheal: i32,
    /// 역할 갑옷 페널티 (urole.spelarmr)
    pub role_spelarmr: i32,
    /// 역할 방패 페널티 (urole.spelshld)
    pub role_spelshld: i32,
    /// 역할 특수 주문 보너스 (urole.spelsbon)
    pub role_spelsbon: i32,
    /// 역할 특수 주문 ID
    pub role_spelspec_id: i32,

    // 장비 상태
    /// 금속 갑옷 착용 중
    pub has_metallic_body_armor: bool,
    /// 로브 착용 중
    pub has_robe: bool,
    /// 방패 착용 중
    pub has_shield: bool,
    /// 무거운 방패 (small_shield 초과)
    pub has_heavy_shield: bool,
    /// 금속 투구 (helm of brilliance 제외)
    pub has_metallic_helmet: bool,
    /// 금속 장갑
    pub has_metallic_gloves: bool,
    /// 금속 부츠
    pub has_metallic_boots: bool,

    // 주문 정보
    /// 주문 ID
    pub spell_id: i32,
    /// 주문 레벨 (1-7)
    pub spell_level: i32,
    /// 회복 주문인지
    pub is_healing_spell: bool,

    // 플레이어 정보
    /// 마법 스탯 현재값 (Int 또는 Wis)
    pub magic_stat: i32,
    /// 플레이어 레벨
    pub player_level: i32,
    /// 해당 주문 계열 스킬 (0: 없음, 1: 기본, 2: 숙련, 3: 전문)
    pub skill_level: i32,
}

/// [v2.22.0 R34-20] 주문 성공률 계산 (원본: percent_success)
/// 반환: 0-100 사이 성공 퍼센트
pub fn calc_spell_success(input: &SpellSuccessInput) -> i32 {
    // 갑옷 페널티 상수
    const UARMHBON: i32 = 4;
    const UARMGBON: i32 = 6;
    const UARMFBON: i32 = 2;

    // [1] 시전 능력 (splcaster) 계산
    let mut splcaster = input.role_spelbase;
    let special = input.role_spelheal;

    // 금속 갑옷 페널티
    if input.has_metallic_body_armor {
        splcaster += if input.has_robe {
            input.role_spelarmr / 2
        } else {
            input.role_spelarmr
        };
    } else if input.has_robe {
        splcaster -= input.role_spelarmr;
    }

    // 방패 페널티
    if input.has_shield {
        splcaster += input.role_spelshld;
    }

    // 금속 투구/장갑/부츠 페널티
    if input.has_metallic_helmet {
        splcaster += UARMHBON;
    }
    if input.has_metallic_gloves {
        splcaster += UARMGBON;
    }
    if input.has_metallic_boots {
        splcaster += UARMFBON;
    }

    // 역할 특수 주문 보너스
    if input.spell_id == input.role_spelspec_id {
        splcaster += input.role_spelsbon;
    }

    // 회복 주문 보너스
    if input.is_healing_spell {
        splcaster += special;
    }

    // 상한 20
    if splcaster > 20 {
        splcaster = 20;
    }

    // [2] 학습 능력 (chance) 계산
    let mut chance = 11 * input.magic_stat / 2;

    // 난이도 계산
    let skill = (input.skill_level.max(1)) - 1; // unskilled → 0
    let difficulty = (input.spell_level - 1) * 4 - (skill * 6 + input.player_level / 3 + 1);

    if difficulty > 0 {
        chance -= isqrt(900 * difficulty + 2000);
    } else {
        let learning = 15 * (-difficulty) / input.spell_level;
        chance += learning.min(20);
    }

    // 클램프 (0~120)
    chance = chance.clamp(0, 120);

    // 무거운 방패 페널티
    if input.has_heavy_shield {
        if input.spell_id == input.role_spelspec_id {
            chance /= 2;
        } else {
            chance /= 4;
        }
    }

    // 최종 계산: 능력 × (20 - 시전능력) / 15 - 시전능력
    chance = chance * (20 - splcaster) / 15 - splcaster;

    chance.clamp(0, 100)
}

// =============================================================================
// [2] 주문 기억 보존율 (원본: spell.c:1827-1871 spellretention)
// =============================================================================

/// [v2.22.0 R34-20] KEEN 상수 (주문 최대 기억 턴)
pub const KEEN: i64 = 20000;

/// [v2.22.0 R34-20] 주문 기억 보존율 계산 (원본: spellretention)
/// `turns_left`: 남은 기억 턴
/// `skill`: 스킬 레벨 (1: 기본, 2: 숙련, 3: 전문)
/// 반환: (low_percent, high_percent) 또는 None(만료)
pub fn calc_spell_retention(turns_left: i64, skill: i32) -> Option<(i64, i64)> {
    if turns_left < 1 {
        return None; // 만료됨
    }
    if turns_left >= KEEN {
        return Some((100, 100)); // 100% 보존
    }

    let percent = (turns_left - 1) / (KEEN / 100) + 1;
    let accuracy: i64 = match skill {
        3 => 2,  // 전문: 2% 간격
        2 => 5,  // 숙련: 5% 간격
        1 => 10, // 기본: 10% 간격
        _ => 25, // 미숙련: 25% 간격
    };

    // 범위 상한으로 반올림
    let high = accuracy * ((percent - 1) / accuracy + 1);
    let low = high - accuracy + 1;

    Some((low, high))
}

// =============================================================================
// [3] 주문 에너지 비용 (원본: spell.c:883-1015 spelleffects의 energy 계산부)
// =============================================================================

/// [v2.22.0 R34-20] 주문 에너지 비용 계산
/// `spell_level`: 주문 레벨
/// `skill_level`: 스킬 레벨 (1: 기본, 2: 숙련, 3: 전문)
/// `is_role_spell`: 역할 특수 주문인지
pub fn calc_spell_energy(spell_level: i32, skill_level: i32, is_role_spell: bool) -> i32 {
    let mut energy = spell_level * 5;

    // 숙련도 보너스
    if skill_level >= 2 {
        energy = energy * 4 / 5; // 숙련: 80%
    }
    if skill_level >= 3 {
        energy = energy * 4 / 5; // 전문: 추가 80% (총 64%)
    }

    // 역할 특수 주문 할인
    if is_role_spell {
        energy = energy / 2;
    }

    energy.max(1) // 최소 1
}

/// [v2.22.0 R34-20] 주문 허기 비용 계산 (원본: spelleffects의 hungr 계산부)
/// `spell_level`: 주문 레벨
/// `skill_level`: 스킬 레벨
pub fn calc_spell_hunger(spell_level: i32, skill_level: i32) -> i32 {
    let base = spell_level * 10;
    match skill_level {
        3 => 0,        // 전문: 허기 없음
        2 => base / 4, // 숙련: 25%
        1 => base / 2, // 기본: 50%
        _ => base,     // 미숙련: 100%
    }
}

// =============================================================================
// [4] 주문 역발 (원본: spell.c:842-881 spell_backfire)
// =============================================================================

/// [v2.22.0 R34-20] 주문 역발 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpellBackfireEffect {
    /// 혼란
    Confusion { duration: i32 },
    /// 피해
    Damage { amount: i32 },
    /// 에너지 손실
    EnergyLoss { amount: i32 },
}

/// [v2.22.0 R34-20] 주문 역발 효과 결정 (원본: spell_backfire)
pub fn determine_spell_backfire(
    spell_level: i32,
    rng: &mut crate::util::rng::NetHackRng,
) -> SpellBackfireEffect {
    let roll = rng.rn2(3);
    match roll {
        0 => SpellBackfireEffect::Confusion {
            duration: rng.rn1(spell_level * 3, 1),
        },
        1 => SpellBackfireEffect::Damage {
            amount: rng.rnd(spell_level * 2),
        },
        _ => SpellBackfireEffect::EnergyLoss {
            amount: rng.rnd(spell_level * 5),
        },
    }
}

// =============================================================================
// [5] 주문 유형 문자열 (원본: spell.c:728-754 spelltypemnemonic)
// =============================================================================

/// [v2.22.0 R34-20] 주문 스킬 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpellSkillType {
    Attack,
    Healing,
    Divination,
    Enchantment,
    Clerical,
    Escape,
    Matter,
}

/// [v2.22.0 R34-20] 주문 유형 문자열 (원본: spelltypemnemonic)
pub fn spell_type_name(skill: SpellSkillType) -> &'static str {
    match skill {
        SpellSkillType::Attack => "attack",
        SpellSkillType::Healing => "healing",
        SpellSkillType::Divination => "divination",
        SpellSkillType::Enchantment => "enchantment",
        SpellSkillType::Clerical => "clerical",
        SpellSkillType::Escape => "escape",
        SpellSkillType::Matter => "matter",
    }
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> SpellSuccessInput {
        SpellSuccessInput {
            role_spelbase: 1,
            role_spelheal: 2,
            role_spelarmr: 10,
            role_spelshld: 1,
            role_spelsbon: -4,
            role_spelspec_id: 100,
            has_metallic_body_armor: false,
            has_robe: false,
            has_shield: false,
            has_heavy_shield: false,
            has_metallic_helmet: false,
            has_metallic_gloves: false,
            has_metallic_boots: false,
            spell_id: 50,
            spell_level: 1,
            is_healing_spell: false,
            magic_stat: 14,
            player_level: 10,
            skill_level: 1,
        }
    }

    #[test]
    fn test_spell_success_basic() {
        let input = base_input();
        let chance = calc_spell_success(&input);
        assert!(chance >= 0 && chance <= 100);
    }

    #[test]
    fn test_spell_success_high_stat() {
        let mut input = base_input();
        input.magic_stat = 25; // 매우 높은 지능
        input.player_level = 20;
        input.skill_level = 3;
        let chance = calc_spell_success(&input);
        assert!(
            chance > 50,
            "High stat should yield good chance: {}",
            chance
        );
    }

    #[test]
    fn test_spell_success_metal_penalty() {
        let mut input = base_input();
        input.has_metallic_body_armor = true;
        input.has_metallic_helmet = true;
        input.has_metallic_gloves = true;
        let chance_armored = calc_spell_success(&input);

        input.has_metallic_body_armor = false;
        input.has_metallic_helmet = false;
        input.has_metallic_gloves = false;
        let chance_unarmored = calc_spell_success(&input);

        assert!(
            chance_armored < chance_unarmored,
            "Metal armor should reduce chance: {} vs {}",
            chance_armored,
            chance_unarmored
        );
    }

    #[test]
    fn test_spell_retention_expired() {
        assert_eq!(calc_spell_retention(0, 1), None);
    }

    #[test]
    fn test_spell_retention_full() {
        assert_eq!(calc_spell_retention(KEEN, 1), Some((100, 100)));
    }

    #[test]
    fn test_spell_retention_expert() {
        let result = calc_spell_retention(10000, 3);
        let (low, high) = result.unwrap();
        assert!(high - low == 1); // 전문: 2% 간격
    }

    #[test]
    fn test_spell_retention_unskilled() {
        let result = calc_spell_retention(10000, 0);
        let (low, high) = result.unwrap();
        assert!(high - low == 24); // 미숙련: 25% 간격
    }

    #[test]
    fn test_spell_energy() {
        assert_eq!(calc_spell_energy(1, 0, false), 5); // 레벨1, 미숙련
        assert_eq!(calc_spell_energy(1, 2, false), 4); // 레벨1, 숙련: 80%
        assert_eq!(calc_spell_energy(5, 0, true), 12); // 레벨5, 역할 특수: 50%
    }

    #[test]
    fn test_spell_hunger() {
        assert_eq!(calc_spell_hunger(1, 3), 0); // 전문: 없음
        assert_eq!(calc_spell_hunger(3, 0), 30); // 미숙련: 100%
        assert_eq!(calc_spell_hunger(2, 1), 10); // 기본: 50%
    }

    #[test]
    fn test_spell_backfire() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        let effect = determine_spell_backfire(3, &mut rng);
        assert!(matches!(
            effect,
            SpellBackfireEffect::Confusion { .. }
                | SpellBackfireEffect::Damage { .. }
                | SpellBackfireEffect::EnergyLoss { .. }
        ));
    }

    #[test]
    fn test_spell_type_name() {
        assert_eq!(spell_type_name(SpellSkillType::Attack), "attack");
        assert_eq!(spell_type_name(SpellSkillType::Matter), "matter");
    }
}
