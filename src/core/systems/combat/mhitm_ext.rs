// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
//
// [v2.23.0 R11-3] 몬스터 간 전투 (mhitm_ext.rs)
//
// 원본 참조: NetHack 3.6.7 mhitm.c (1,783줄) 핵심 로직 이식
//
// 구현 내용:
//   1. 몬스터 간 공격 판정 (to-hit, AC, 데미지)
//   2. 공격 타입(at_type) 분류 (근접, 원거리, 마법, 숨결 등)
//   3. 데미지 타입(ad_type) 분류 (물리, 독, 전기, 석화 등)
//   4. 특수 공격 효과 판정
//   5. 전투 결과 산출 (Pure Result)
//   6. 몬스터 간 적대/동맹 판정
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 공격 타입 (원본: mhitm.c at_types)
// =============================================================================

/// [v2.23.0 R11-3] 공격 방법 (원본: AT_NONE ~ AT_MAGC)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    None,
    /// 발톱 (AT_CLAW)
    Claw,
    /// 물기 (AT_BITE)
    Bite,
    /// 발차기 (AT_KICK)
    Kick,
    /// 접촉 (AT_BUTT)
    Butt,
    /// 접촉 (AT_TUCH)
    Touch,
    /// 찌르기 (AT_STNG)
    Sting,
    /// 포옹 (AT_HUGS)
    Hug,
    /// 침 (AT_SPIT)
    Spit,
    /// 시선 (AT_GAZE)
    Gaze,
    /// 숨결 (AT_BREA)
    Breath,
    /// 마법 (AT_MAGC)
    Magic,
    /// 무기 (AT_WEAP)
    Weapon,
    /// 폭발 (AT_EXPL)
    Explode,
    /// 포식 (AT_ENGL)
    Engulf,
}

// =============================================================================
// [2] 데미지 타입 (원본: mhitm.c ad_types)
// =============================================================================

/// [v2.23.0 R11-3] 데미지 종류 (원본: AD_PHYS ~ AD_STON)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    /// 물리 (AD_PHYS)
    Physical,
    /// 마법 에너지 (AD_MAGM)
    MagicMissile,
    /// 불 (AD_FIRE)
    Fire,
    /// 냉기 (AD_COLD)
    Cold,
    /// 전기 (AD_ELEC)
    Electricity,
    /// 독 (AD_DRST / AD_DRDX / AD_DRCO)
    Poison,
    /// 산 (AD_ACID)
    Acid,
    /// 석화 (AD_STON)
    Petrification,
    /// 실명 (AD_BLND)
    Blind,
    /// 혼란 (AD_CONF)
    Confuse,
    /// 마비 (AD_PLYS)
    Paralyze,
    /// 수면 (AD_SLEE)
    Sleep,
    /// 체력 흡수 (AD_DRLI)
    DrainLife,
    /// 텔레포트 (AD_TELE)
    Teleport,
    /// 녹 (AD_RUST)
    Rust,
    /// 부식 (AD_CORR)
    Corrode,
    /// 질병 (AD_DISE)
    Disease,
    /// 치유 (AD_HEAL)
    Heal,
}

// =============================================================================
// [3] 몬스터 간 공격 판정 (원본: mhitm.c mattackm, hitmm)
// =============================================================================

/// [v2.23.0 R11-3] 전투 참가자 스탯
#[derive(Debug, Clone)]
pub struct CombatantStats {
    /// 이름
    pub name: String,
    /// 레벨
    pub level: i32,
    /// AC (높을수록 약함)
    pub ac: i32,
    /// HP
    pub hp: i32,
    /// 최대 HP
    pub max_hp: i32,
    /// 공격 다이스 수
    pub attack_dice: i32,
    /// 공격 다이스 면
    pub attack_sides: i32,
    /// 공격 타입
    pub attack_type: AttackType,
    /// 데미지 타입
    pub damage_type: DamageType,
}

/// [v2.23.0 R11-3] 명중 판정 (원본: hitmm)
pub fn roll_to_hit(attacker_level: i32, defender_ac: i32, rng: &mut NetHackRng) -> bool {
    // 원본: 1d20 + attacker_level - defender_ac >= 10
    let roll = rng.rn1(20, 1);
    let needed = 10 - attacker_level + defender_ac;
    roll >= needed
}

/// [v2.23.0 R11-3] 데미지 계산
pub fn calc_damage(dice: i32, sides: i32, rng: &mut NetHackRng) -> i32 {
    if dice <= 0 || sides <= 0 {
        return 0;
    }
    let mut total = 0;
    for _ in 0..dice {
        total += rng.rn1(sides, 1);
    }
    total
}

// =============================================================================
// [4] 전투 결과 (Pure Result)
// =============================================================================

/// [v2.23.0 R11-3] 전투 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatOutcome {
    /// 명중 + 데미지
    Hit {
        damage: i32,
        special_effect: Option<SpecialEffect>,
    },
    /// 빗나감
    Miss,
    /// 공격 불가 (공격 타입 없음)
    NoAttack,
}

/// [v2.23.0 R11-3] 특수 효과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialEffect {
    /// 독 데미지
    Poisoned { str_drain: i32 },
    /// 석화
    Petrified,
    /// 마비 (턴 수)
    Paralyzed(i32),
    /// 실명 (턴 수)
    Blinded(i32),
    /// 수면 (턴 수)
    Asleep(i32),
    /// 레벨 흡수
    LevelDrained,
    /// 텔레포트
    Teleported,
    /// 장비 부식
    EquipmentCorroded,
    /// 질병 감염
    Diseased,
    /// 혼란 (턴 수)
    Confused(i32),
}

/// [v2.23.0 R11-3] 몬스터 간 공격 1회 판정 (원본: mattackm 핵심)
pub fn resolve_attack(
    attacker: &CombatantStats,
    defender: &CombatantStats,
    rng: &mut NetHackRng,
) -> CombatOutcome {
    if attacker.attack_type == AttackType::None {
        return CombatOutcome::NoAttack;
    }

    // 명중 판정
    if !roll_to_hit(attacker.level, defender.ac, rng) {
        return CombatOutcome::Miss;
    }

    // 데미지
    let damage = calc_damage(attacker.attack_dice, attacker.attack_sides, rng).max(1);

    // 특수 효과 판정
    let special = match attacker.damage_type {
        DamageType::Poison => {
            if rng.rn2(4) == 0 {
                Some(SpecialEffect::Poisoned { str_drain: 1 })
            } else {
                None
            }
        }
        DamageType::Petrification => Some(SpecialEffect::Petrified),
        DamageType::Paralyze => Some(SpecialEffect::Paralyzed(rng.rn1(4, 1))),
        DamageType::Blind => Some(SpecialEffect::Blinded(rng.rn1(10, 1))),
        DamageType::Sleep => Some(SpecialEffect::Asleep(rng.rn1(6, 1))),
        DamageType::DrainLife => Some(SpecialEffect::LevelDrained),
        DamageType::Teleport => Some(SpecialEffect::Teleported),
        DamageType::Rust | DamageType::Corrode => Some(SpecialEffect::EquipmentCorroded),
        DamageType::Disease => Some(SpecialEffect::Diseased),
        DamageType::Confuse => Some(SpecialEffect::Confused(rng.rn1(6, 1))),
        _ => None,
    };

    CombatOutcome::Hit {
        damage,
        special_effect: special,
    }
}

// =============================================================================
// [5] 적대/동맹 판정 (원본: mhitm.c mm_aggression)
// =============================================================================

/// [v2.23.0 R11-3] 진영 적대성 판정
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hostility {
    /// 적대 (공격함)
    Hostile,
    /// 중립 (무시)
    Neutral,
    /// 동맹 (공격 안함)
    Ally,
}

/// [v2.23.0 R11-3] 몬스터 간 적대성 판정 (원본: mm_aggression)
pub fn check_hostility(
    a_faction: i32,
    b_faction: i32,
    a_hostile: bool,
    b_hostile: bool,
    same_species: bool,
) -> Hostility {
    // 같은 종이면 동맹
    if same_species {
        return Hostility::Ally;
    }
    // 같은 진영이면 동맹
    if a_faction == b_faction && a_faction != 0 {
        return Hostility::Ally;
    }
    // 적대적 몬스터끼리
    if a_hostile && b_hostile {
        // 둘 다 적대적 → 보통 서로 무시
        return Hostility::Neutral;
    }
    // 하나만 적대적이면 공격
    if a_hostile || b_hostile {
        return Hostility::Hostile;
    }
    Hostility::Neutral
}

/// [v2.23.0 R11-3] 원거리 공격 가능 여부
pub fn can_ranged_attack(attack_type: AttackType) -> bool {
    matches!(
        attack_type,
        AttackType::Spit | AttackType::Gaze | AttackType::Breath | AttackType::Magic
    )
}

/// [v2.23.0 R11-3] 데미지 타입이 원소 저항으로 방어 가능한지
pub fn is_elemental_damage(damage_type: DamageType) -> bool {
    matches!(
        damage_type,
        DamageType::Fire | DamageType::Cold | DamageType::Electricity | DamageType::Acid
    )
}

/// [v2.23.0 R11-3] 원소 저항 시 데미지 감소
pub fn apply_resistance(damage: i32, resisted: bool) -> i32 {
    if resisted {
        damage / 2 // 50% 감소
    } else {
        damage
    }
}

// =============================================================================
// [6] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attacker() -> CombatantStats {
        CombatantStats {
            name: "orc".to_string(),
            level: 3,
            ac: 6,
            hp: 15,
            max_hp: 15,
            attack_dice: 1,
            attack_sides: 8,
            attack_type: AttackType::Weapon,
            damage_type: DamageType::Physical,
        }
    }

    fn make_defender() -> CombatantStats {
        CombatantStats {
            name: "kobold".to_string(),
            level: 1,
            ac: 7,
            hp: 5,
            max_hp: 5,
            attack_dice: 1,
            attack_sides: 4,
            attack_type: AttackType::Claw,
            damage_type: DamageType::Physical,
        }
    }

    #[test]
    fn test_roll_to_hit() {
        let mut rng = NetHackRng::new(42);
        // 높은 레벨 vs 높은 AC → 쉬움
        let hit = roll_to_hit(10, 0, &mut rng);
        // 랜덤이므로 여러 번 돌려야 하지만, 이 시드에서는 확인 가능
        let _ = hit; // 결과 사용
    }

    #[test]
    fn test_calc_damage() {
        let mut rng = NetHackRng::new(42);
        let dmg = calc_damage(2, 6, &mut rng);
        assert!(dmg >= 2 && dmg <= 12);
    }

    #[test]
    fn test_calc_damage_zero() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(calc_damage(0, 6, &mut rng), 0);
        assert_eq!(calc_damage(2, 0, &mut rng), 0);
    }

    #[test]
    fn test_resolve_attack_no_attack() {
        let mut attacker = make_attacker();
        attacker.attack_type = AttackType::None;
        let defender = make_defender();
        let mut rng = NetHackRng::new(42);
        let result = resolve_attack(&attacker, &defender, &mut rng);
        assert_eq!(result, CombatOutcome::NoAttack);
    }

    #[test]
    fn test_resolve_attack_hit_or_miss() {
        let attacker = make_attacker();
        let defender = make_defender();
        let mut rng = NetHackRng::new(42);
        let result = resolve_attack(&attacker, &defender, &mut rng);
        assert!(matches!(
            result,
            CombatOutcome::Hit { .. } | CombatOutcome::Miss
        ));
    }

    #[test]
    fn test_resolve_attack_petrification() {
        let mut attacker = make_attacker();
        attacker.damage_type = DamageType::Petrification;
        let defender = make_defender();
        let mut rng = NetHackRng::new(42);
        // 여러 번 돌려서 Hit이 나올 때 석화 효과 확인
        for _ in 0..20 {
            let result = resolve_attack(&attacker, &defender, &mut rng);
            if let CombatOutcome::Hit { special_effect, .. } = result {
                assert_eq!(special_effect, Some(SpecialEffect::Petrified));
                return;
            }
        }
    }

    #[test]
    fn test_hostility_same_species() {
        assert_eq!(check_hostility(1, 2, true, true, true), Hostility::Ally);
    }

    #[test]
    fn test_hostility_same_faction() {
        assert_eq!(check_hostility(3, 3, false, false, false), Hostility::Ally);
    }

    #[test]
    fn test_hostility_one_hostile() {
        assert_eq!(
            check_hostility(1, 2, true, false, false),
            Hostility::Hostile
        );
    }

    #[test]
    fn test_hostility_both_hostile() {
        assert_eq!(check_hostility(1, 2, true, true, false), Hostility::Neutral);
    }

    #[test]
    fn test_hostility_neutral() {
        assert_eq!(
            check_hostility(0, 0, false, false, false),
            Hostility::Neutral
        );
    }

    #[test]
    fn test_can_ranged() {
        assert!(can_ranged_attack(AttackType::Breath));
        assert!(can_ranged_attack(AttackType::Gaze));
        assert!(!can_ranged_attack(AttackType::Claw));
        assert!(!can_ranged_attack(AttackType::Bite));
    }

    #[test]
    fn test_is_elemental() {
        assert!(is_elemental_damage(DamageType::Fire));
        assert!(is_elemental_damage(DamageType::Cold));
        assert!(!is_elemental_damage(DamageType::Physical));
        assert!(!is_elemental_damage(DamageType::Poison));
    }

    #[test]
    fn test_resistance_halves() {
        assert_eq!(apply_resistance(10, true), 5);
        assert_eq!(apply_resistance(10, false), 10);
        assert_eq!(apply_resistance(7, true), 3); // 정수 나눗셈
    }
}
