// =============================================================================
// AIHack — mhitu_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
// =============================================================================
// [v2.19.0] mhitu.c 핵심 함수 이식 — Pure Result 패턴
// 원본: nethack-3.6.7/src/mhitu.c (2,991줄)
//
// 이식 대상:
//   getmattk     (L260-352)  → getmattk_result
//   mattacku     (L354-800)  → mattacku_precheck
//   demon_summon (L590-594)  → demon_summon_check
//   were_change  (L596-640)  → were_transform_check
//   wildmiss     (L142-216)  → wildmiss_type
//   gulpmu       (L1200+)    → gulpmu_damage
//   explmu       (L1100+)    → explmu_damage
//   passiveum    (L2600+)    → passiveum_result
//   diseasemu    (L1600+)    → disease_check
//   u_slip_free  (L2500+)    → u_slip_free_check
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [v2.19.0] 공격 유형 / 데미지 유형 (원본 mhitu.c에서 참조)
// =============================================================================

/// 몬스터 공격 방식 (원본: AT_* 상수)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    Claw,     // 발톱 (AT_CLAW)
    Bite,     // 물기 (AT_BITE)
    Kick,     // 발차기 (AT_KICK)
    Butt,     // 박치기 (AT_BUTT)
    Touch,    // 접촉 (AT_TUCH)
    Sting,    // 쏘기 (AT_STNG)
    Hug,      // 포옹 (AT_HUGS)
    Weapon,   // 무기 (AT_WEAP)
    Magic,    // 마법 (AT_MAGC)
    Breath,   // 브레스 (AT_BREA)
    Spit,     // 뱉기 (AT_SPIT)
    Gaze,     // 응시 (AT_GAZE)
    Tentacle, // 촉수 (AT_TENT)
    Engulf,   // 삼킴 (AT_ENGL)
    Explode,  // 자폭 (AT_EXPL)
}

/// 몬스터 데미지 유형 (원본: AD_* 상수)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageClass {
    Physical,      // AD_PHYS
    MagicMissile,  // AD_MAGM
    Fire,          // AD_FIRE
    Cold,          // AD_COLD
    Sleep,         // AD_SLEE
    Disintegrate,  // AD_DISN
    Electric,      // AD_ELEC
    Poison,        // AD_DRST
    Acid,          // AD_ACID
    DrainLife,     // AD_DRLI
    Disease,       // AD_DISE
    Seduce,        // AD_SEDU
    Stun,          // AD_STUN
    EnergyDrain,   // AD_DREN
    Petrify,       // AD_STON
    Paralyze,      // AD_PLYS
    Famish,        // AD_FAMN
    Pestilence,    // AD_PEST
    Hallucination, // AD_HALU
    SexualAttack,  // AD_SSEX
}

/// 몬스터 공격 정보
#[derive(Debug, Clone)]
pub struct AttackInfo {
    pub attack_type: AttackType,
    pub damage_class: DamageClass,
    pub dice_num: i32,
    pub dice_sides: i32,
}

// =============================================================================
// [v2.19.0] 1. getmattk_result — 공격 종류 대체 판정
// 원본: mhitu.c L260-352 getmattk()
// =============================================================================

/// 공격 대체 결과
#[derive(Debug, Clone)]
pub struct AttackOverride {
    /// 최종 공격 정보 (대체되지 않으면 원본과 동일)
    pub attack: AttackInfo,
    /// 대체 발생 여부
    pub was_overridden: bool,
    /// 대체 사유
    pub reason: &'static str,
}

/// [v2.19.0] 공격 종류 대체 판정 (원본: getmattk L260-352)
/// 연속 질병 공격 방지, 에너지 드레인 조정, 삼킴 재사용 제한 등
pub fn getmattk_result(
    attack: &AttackInfo,
    attack_index: usize,
    prev_attack_hit: bool,
    prev_damage_class: Option<DamageClass>,
    attacker_cancelled: bool,
    engulf_on_cooldown: bool,
    player_energy: i32,
    player_max_energy: i32,
    player_level: i32,
) -> AttackOverride {
    let mut result = attack.clone();
    let mut overridden = false;
    let mut reason = "";

    // 원본: 연속 질병/기아 공격 방지 (같은 유형이 연속 명중 시 stun으로 대체)
    if attack_index > 0 && prev_attack_hit {
        if matches!(
            attack.damage_class,
            DamageClass::Disease | DamageClass::Pestilence | DamageClass::Famish
        ) {
            if prev_damage_class == Some(attack.damage_class) {
                result.damage_class = DamageClass::Stun;
                overridden = true;
                reason = "연속 질병/기아 공격 방지 → 기절로 대체";
            }
        }
    }

    // 원본: 에너지 드레인 조정 (플레이어 에너지에 비례)
    if attack.damage_class == DamageClass::EnergyDrain {
        let ulev = player_level.max(6);
        if player_energy <= 5 * ulev && result.dice_num > 1 {
            result.dice_num -= 1; // 저에너지: 2d6 → 1d6
            if player_max_energy <= 2 * ulev && result.dice_sides > 3 {
                result.dice_sides -= 3; // 극저에너지: 1d6 → 1d3
            }
            overridden = true;
            reason = "에너지 드레인 약화 (저에너지)";
        } else if player_energy > 12 * ulev {
            result.dice_num += 1; // 고에너지: 2d6 → 3d6
            if player_max_energy > 20 * ulev {
                result.dice_sides += 3; // 극고에너지: 3d6 → 3d9
            }
            overridden = true;
            reason = "에너지 드레인 강화 (고에너지)";
        }
    }

    // 원본: 삼킴 재사용 제한 → 접촉/발톱으로 대체
    if attack.attack_type == AttackType::Engulf && engulf_on_cooldown {
        if matches!(
            attack.damage_class,
            DamageClass::Acid | DamageClass::Electric | DamageClass::Cold | DamageClass::Fire
        ) {
            result.attack_type = AttackType::Touch;
        } else {
            result.attack_type = AttackType::Claw;
            result.damage_class = DamageClass::Physical;
        }
        result.dice_num = 1;
        result.dice_sides = 6;
        overridden = true;
        reason = "삼킴 쿨다운 → 접촉/발톱으로 대체";
    }

    // 원본: 취소된 무기 공격자 → 물리 데미지로 강제
    if attacker_cancelled
        && attack.attack_type == AttackType::Weapon
        && attack.damage_class != DamageClass::Physical
    {
        result.damage_class = DamageClass::Physical;
        overridden = true;
        reason = "취소된 공격자 → 물리 데미지";
    }

    AttackOverride {
        attack: result,
        was_overridden: overridden,
        reason,
    }
}

// =============================================================================
// [v2.19.0] 2. mattacku_precheck — 공격 전 사전 조건 판정
// 원본: mhitu.c mattacku() L354-654 초반부
// =============================================================================

/// 공격 전 사전 조건 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrecheckResult {
    /// 공격 진행 가능
    Proceed,
    /// 공격 불가: 이유
    Blocked(&'static str),
    /// 탈 것을 대신 공격
    AttackSteed,
}

/// [v2.19.0] 공격 전 사전 조건 (원본: mattacku L354-654)
pub fn mattacku_precheck(
    attacker_dead: bool,
    attacker_is_steed: bool,
    player_swallowed: bool,
    is_swallower: bool,
    player_invulnerable: bool,
    player_underwater: bool,
    attacker_is_swimmer: bool,
    has_steed: bool,
    attacker_is_orc: bool,
    distance_squared: i32,
    rng: &mut NetHackRng,
) -> PrecheckResult {
    // 원본: 죽은 몬스터는 공격 불가
    if attacker_dead {
        return PrecheckResult::Blocked("공격자 사망");
    }

    // 원본: 수중에서 수영 불가 몬스터는 공격 불가
    if player_underwater && !attacker_is_swimmer {
        return PrecheckResult::Blocked("수중 공격 불가");
    }

    // 원본: 삼킨 상태 — 삼킨 자만 공격 가능
    if player_swallowed && !is_swallower {
        return PrecheckResult::Blocked("삼킨 자가 아님");
    }

    // 원본: 플레이어 무적 상태
    if player_invulnerable {
        return PrecheckResult::Blocked("무적 상태");
    }

    // 원본: 탈 것이 공격자인 경우
    if attacker_is_steed {
        return PrecheckResult::Blocked("자신의 탈 것");
    }

    // 원본: 오크 몬스터가 탈 것을 공격 (25-50% 확률)
    if has_steed && distance_squared <= 2 {
        let orc_chance = if attacker_is_orc { 2 } else { 4 };
        if rng.rn2(orc_chance) == 0 {
            return PrecheckResult::AttackSteed;
        }
    }

    PrecheckResult::Proceed
}

// =============================================================================
// [v2.19.0] 3. demon_summon_check — 악마 소환 확률
// 원본: mhitu.c L590-594
// =============================================================================

/// [v2.19.0] 악마 소환 확률 (원본: mattacku ~L590)
/// 악마 (발로그/서큐버스/인큐버스 제외)는 1/13 확률로 소환
pub fn demon_summon_check(
    is_demon: bool,
    is_balrog: bool,
    is_succubus_incubus: bool,
    is_cancelled: bool,
    is_ranged: bool,
    rng: &mut NetHackRng,
) -> bool {
    is_demon
        && !is_balrog
        && !is_succubus_incubus
        && !is_cancelled
        && !is_ranged
        && rng.rn2(13) == 0
}

// =============================================================================
// [v2.19.0] 4. were_transform_check — 수인 변신 확률
// 원본: mhitu.c L596-640
// =============================================================================

/// 수인 변신 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WereResult {
    /// 변신 안 함
    NoChange,
    /// 인간 → 늑대 형태 변신
    TransformToAnimal,
    /// 늑대 → 인간 형태 변신
    TransformToHuman,
    /// 동료 소환
    SummonAllies,
}

/// [v2.19.0] 수인 변신 확률 (원본: mattacku ~L596)
pub fn were_transform_check(
    is_were: bool,
    is_human_form: bool,
    is_cancelled: bool,
    is_night: bool,
    rng: &mut NetHackRng,
) -> WereResult {
    if !is_were || is_cancelled {
        return WereResult::NoChange;
    }

    // 원본: 10% 확률로 동료 소환
    if rng.rn2(10) == 0 {
        return WereResult::SummonAllies;
    }

    if is_human_form {
        // 원본: rn2(5 - night()*2) — 밤에는 40%, 낮에는 20%
        let threshold = if is_night { 3 } else { 5 };
        if rng.rn2(threshold) == 0 {
            return WereResult::TransformToAnimal;
        }
    } else {
        // 원본: !rn2(30) — 3.3% 확률로 인간으로 복귀
        if rng.rn2(30) == 0 {
            return WereResult::TransformToHuman;
        }
    }

    WereResult::NoChange
}

// =============================================================================
// [v2.19.0] 5. wildmiss_type — 빗나간 공격 유형
// 원본: mhitu.c wildmiss() L142-216
// =============================================================================

/// 빗나간 공격 유형
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WildMissType {
    /// 보이지 않아서 빗나감 — 랜덤 메시지
    BlindSwing { variant: i32 },
    /// 전위 이미지를 공격
    HitDisplacedImage,
    /// 수중 왜곡으로 빗나감
    UnderwaterDistortion,
}

/// [v2.19.0] 빗나간 공격 유형 결정 (원본: wildmiss L142-216)
pub fn wildmiss_type(
    attacker_can_see: bool,
    player_invisible: bool,
    attacker_perceives_invis: bool,
    player_displaced: bool,
    player_underwater: bool,
    rng: &mut NetHackRng,
) -> WildMissType {
    if !attacker_can_see || (player_invisible && !attacker_perceives_invis) {
        WildMissType::BlindSwing {
            variant: rng.rn2(3),
        }
    } else if player_displaced {
        WildMissType::HitDisplacedImage
    } else if player_underwater {
        WildMissType::UnderwaterDistortion
    } else {
        // 불가능한 상태지만 기본값
        WildMissType::BlindSwing { variant: 0 }
    }
}

// =============================================================================
// [v2.19.0] 6. gulpmu_damage — 삼킴 공격 데미지
// 원본: mhitu.c gulpmu() ~L1200
// =============================================================================

/// [v2.19.0] 삼킴 공격 데미지 (원본: gulpmu ~L1200)
/// 삼킴 지속 데미지는 유형에 따라 다름
pub fn gulpmu_damage(
    damage_class: DamageClass,
    dice_num: i32,
    dice_sides: i32,
    has_resistance: bool,
    rng: &mut NetHackRng,
) -> i32 {
    if has_resistance {
        return 0;
    }
    let mut dmg = 0;
    for _ in 0..dice_num {
        dmg += rng.rnd(dice_sides);
    }
    // 원본: 산성은 방어구도 부식
    if damage_class == DamageClass::Acid {
        dmg += rng.rnd(4); // 추가 산 데미지
    }
    dmg
}

// =============================================================================
// [v2.19.0] 7. explmu_damage — 자폭 공격 데미지
// 원본: mhitu.c explmu() ~L1100
// =============================================================================

/// [v2.19.0] 자폭 공격 데미지 (원본: explmu ~L1100)
/// 자폭 시 몬스터 사망, 데미지는 주사위 기반
pub fn explmu_damage(
    dice_num: i32,
    dice_sides: i32,
    has_resistance: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let mut dmg = 0;
    for _ in 0..dice_num {
        dmg += rng.rnd(dice_sides);
    }
    if has_resistance {
        dmg / 2 // 저항 시 절반
    } else {
        dmg
    }
}

// =============================================================================
// [v2.19.0] 8. passiveum_result — 플레이어 패시브 반격
// 원본: mhitu.c passiveum() ~L2600
// =============================================================================

/// 패시브 반격 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassiveResult {
    /// 반격 없음
    None,
    /// 반격 데미지 발생 (산/전기/불 등)
    Damage {
        amount: i32,
        damage_type: &'static str,
    },
    /// 석화 반격 (코카트리스 시체 등)
    Petrify,
    /// 독 반격
    Poison,
}

/// [v2.19.0] 플레이어 패시브 반격 판정 (원본: passiveum ~L2600)
pub fn passiveum_result(
    player_is_acid_body: bool,
    player_is_shock_body: bool,
    player_is_fire_body: bool,
    player_is_cold_body: bool,
    player_has_cockatrice_corpse: bool,
    attack_type: AttackType,
    rng: &mut NetHackRng,
) -> PassiveResult {
    // 석화 반격 (코카트리스 시체 들고 있는 상태에서 접촉 공격 받음)
    if player_has_cockatrice_corpse
        && matches!(
            attack_type,
            AttackType::Claw | AttackType::Touch | AttackType::Bite
        )
    {
        return PassiveResult::Petrify;
    }

    // 산성 몸체
    if player_is_acid_body
        && matches!(
            attack_type,
            AttackType::Claw | AttackType::Touch | AttackType::Bite | AttackType::Hug
        )
    {
        return PassiveResult::Damage {
            amount: rng.rnd(4),
            damage_type: "산",
        };
    }

    // 전기 몸체
    if player_is_shock_body
        && matches!(
            attack_type,
            AttackType::Claw | AttackType::Touch | AttackType::Bite | AttackType::Hug
        )
    {
        return PassiveResult::Damage {
            amount: rng.rnd(6),
            damage_type: "전기",
        };
    }

    // 화염 몸체
    if player_is_fire_body
        && matches!(
            attack_type,
            AttackType::Claw | AttackType::Touch | AttackType::Bite | AttackType::Hug
        )
    {
        return PassiveResult::Damage {
            amount: rng.rnd(6),
            damage_type: "화염",
        };
    }

    // 냉기 몸체
    if player_is_cold_body
        && matches!(
            attack_type,
            AttackType::Claw | AttackType::Touch | AttackType::Bite | AttackType::Hug
        )
    {
        return PassiveResult::Damage {
            amount: rng.rnd(4),
            damage_type: "냉기",
        };
    }

    PassiveResult::None
}

// =============================================================================
// [v2.19.0] 9. disease_check — 질병 감염 확률
// 원본: mhitu.c diseasemu() ~L1600
// =============================================================================

/// [v2.19.0] 질병 감염 확률 (원본: diseasemu ~L1600)
/// 감염 확률: 1/8 기본, 축복 유니콘 뿔이 있으면 면역
pub fn disease_check(
    has_sick_resistance: bool,
    has_blessed_unicorn_horn: bool,
    rng: &mut NetHackRng,
) -> bool {
    if has_sick_resistance || has_blessed_unicorn_horn {
        return false;
    }
    // 원본: 1/8 확률
    rng.rn2(8) == 0
}

// =============================================================================
// [v2.19.0] 10. u_slip_free_check — 미끄러져 풀려남
// 원본: mhitu.c u_slip_free() ~L2500
// =============================================================================

/// [v2.19.0] 미끄러져 풀려남 (원본: u_slip_free ~L2500)
/// 그리스 방어구, 비정형체, 매우 작은 몬스터 등
pub fn u_slip_free_check(
    player_is_amorphous: bool,
    player_is_small_form: bool,
    player_has_greased_armor: bool,
    player_dexterity: i32,
    rng: &mut NetHackRng,
) -> bool {
    // 비정형체는 무조건 풀려남
    if player_is_amorphous {
        return true;
    }
    // 그리스 방어구 보호
    if player_has_greased_armor {
        return true;
    }
    // 매우 작은 폼은 50% 확률로 풀려남
    if player_is_small_form && rng.rn2(2) == 0 {
        return true;
    }
    // 민첩성 기반: dex >= 15이면 rn2(4)
    if player_dexterity >= 15 && rng.rn2(4) == 0 {
        return true;
    }
    false
}

// =============================================================================
// [v2.19.0] 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    fn make_attack(at: AttackType, dc: DamageClass, dn: i32, ds: i32) -> AttackInfo {
        AttackInfo {
            attack_type: at,
            damage_class: dc,
            dice_num: dn,
            dice_sides: ds,
        }
    }

    // --- getmattk_result ---

    #[test]
    fn test_getmattk_no_override() {
        let atk = make_attack(AttackType::Claw, DamageClass::Physical, 1, 6);
        let r = getmattk_result(&atk, 0, false, None, false, false, 50, 100, 10);
        assert!(!r.was_overridden);
    }

    #[test]
    fn test_getmattk_consecutive_disease() {
        let atk = make_attack(AttackType::Bite, DamageClass::Disease, 1, 4);
        let r = getmattk_result(
            &atk,
            1,
            true,
            Some(DamageClass::Disease),
            false,
            false,
            50,
            100,
            10,
        );
        assert!(r.was_overridden);
        assert_eq!(r.attack.damage_class, DamageClass::Stun);
    }

    #[test]
    fn test_getmattk_energy_drain_low() {
        let atk = make_attack(AttackType::Touch, DamageClass::EnergyDrain, 2, 6);
        let r = getmattk_result(&atk, 0, false, None, false, false, 10, 20, 10);
        assert!(r.was_overridden);
        assert!(r.attack.dice_num < 2, "저에너지 약화");
    }

    #[test]
    fn test_getmattk_engulf_cooldown() {
        let atk = make_attack(AttackType::Engulf, DamageClass::Acid, 2, 8);
        let r = getmattk_result(&atk, 0, false, None, false, true, 50, 100, 10);
        assert!(r.was_overridden);
        assert_eq!(r.attack.attack_type, AttackType::Touch);
    }

    #[test]
    fn test_getmattk_cancelled_weapon() {
        let atk = make_attack(AttackType::Weapon, DamageClass::DrainLife, 1, 6);
        let r = getmattk_result(&atk, 0, false, None, true, false, 50, 100, 10);
        assert!(r.was_overridden);
        assert_eq!(r.attack.damage_class, DamageClass::Physical);
    }

    // --- mattacku_precheck ---

    #[test]
    fn test_precheck_proceed() {
        let mut rng = test_rng();
        let r = mattacku_precheck(
            false, false, false, false, false, false, false, false, false, 1, &mut rng,
        );
        assert_eq!(r, PrecheckResult::Proceed);
    }

    #[test]
    fn test_precheck_dead() {
        let mut rng = test_rng();
        let r = mattacku_precheck(
            true, false, false, false, false, false, false, false, false, 1, &mut rng,
        );
        assert_eq!(r, PrecheckResult::Blocked("공격자 사망"));
    }

    #[test]
    fn test_precheck_invulnerable() {
        let mut rng = test_rng();
        let r = mattacku_precheck(
            false, false, false, false, true, false, false, false, false, 1, &mut rng,
        );
        assert_eq!(r, PrecheckResult::Blocked("무적 상태"));
    }

    #[test]
    fn test_precheck_steed() {
        let mut rng = test_rng();
        let r = mattacku_precheck(
            false, true, false, false, false, false, false, false, false, 1, &mut rng,
        );
        assert_eq!(r, PrecheckResult::Blocked("자신의 탈 것"));
    }

    // --- demon_summon_check ---

    #[test]
    fn test_demon_summon_basic() {
        let mut rng = test_rng();
        let mut summoned = 0;
        for _ in 0..1300 {
            if demon_summon_check(true, false, false, false, false, &mut rng) {
                summoned += 1;
            }
        }
        // ~7.7% 확률 (1/13)
        assert!(summoned > 50 && summoned < 200, "악마 소환: {}", summoned);
    }

    #[test]
    fn test_demon_summon_balrog_excluded() {
        let mut rng = test_rng();
        assert!(!demon_summon_check(
            true, true, false, false, false, &mut rng
        ));
    }

    // --- were_transform_check ---

    #[test]
    fn test_were_no_change_not_were() {
        let mut rng = test_rng();
        assert_eq!(
            were_transform_check(false, true, false, false, &mut rng),
            WereResult::NoChange
        );
    }

    #[test]
    fn test_were_transform_possible() {
        let mut rng = test_rng();
        let mut transforms = 0;
        for _ in 0..500 {
            let r = were_transform_check(true, true, false, true, &mut rng);
            if r == WereResult::TransformToAnimal || r == WereResult::SummonAllies {
                transforms += 1;
            }
        }
        assert!(transforms > 50, "수인 변신: {}", transforms);
    }

    // --- wildmiss_type ---

    #[test]
    fn test_wildmiss_blind() {
        let mut rng = test_rng();
        let r = wildmiss_type(false, true, false, false, false, &mut rng);
        matches!(r, WildMissType::BlindSwing { .. });
    }

    #[test]
    fn test_wildmiss_displaced() {
        let mut rng = test_rng();
        let r = wildmiss_type(true, false, false, true, false, &mut rng);
        assert_eq!(r, WildMissType::HitDisplacedImage);
    }

    #[test]
    fn test_wildmiss_underwater() {
        let mut rng = test_rng();
        let r = wildmiss_type(true, false, false, false, true, &mut rng);
        assert_eq!(r, WildMissType::UnderwaterDistortion);
    }

    // --- gulpmu_damage ---

    #[test]
    fn test_gulp_resist() {
        let mut rng = test_rng();
        assert_eq!(gulpmu_damage(DamageClass::Fire, 2, 6, true, &mut rng), 0);
    }

    #[test]
    fn test_gulp_acid_extra() {
        let mut rng = test_rng();
        let dmg = gulpmu_damage(DamageClass::Acid, 1, 4, false, &mut rng);
        assert!(dmg >= 2, "산 추가: {}", dmg); // 기본 + 추가
    }

    // --- explmu_damage ---

    #[test]
    fn test_explmu_resist_half() {
        let mut rng = test_rng();
        let full = explmu_damage(4, 6, false, &mut rng);
        let mut rng2 = NetHackRng::new(42);
        let half = explmu_damage(4, 6, true, &mut rng2);
        assert!(half <= full, "저항 절반");
    }

    // --- passiveum_result ---

    #[test]
    fn test_passive_cockatrice() {
        let mut rng = test_rng();
        let r = passiveum_result(
            false,
            false,
            false,
            false,
            true,
            AttackType::Touch,
            &mut rng,
        );
        assert_eq!(r, PassiveResult::Petrify);
    }

    #[test]
    fn test_passive_acid_body() {
        let mut rng = test_rng();
        let r = passiveum_result(true, false, false, false, false, AttackType::Claw, &mut rng);
        matches!(r, PassiveResult::Damage { .. });
    }

    #[test]
    fn test_passive_none() {
        let mut rng = test_rng();
        let r = passiveum_result(
            false,
            false,
            false,
            false,
            false,
            AttackType::Magic,
            &mut rng,
        );
        assert_eq!(r, PassiveResult::None);
    }

    // --- disease_check ---

    #[test]
    fn test_disease_resistant() {
        let mut rng = test_rng();
        assert!(!disease_check(true, false, &mut rng));
    }

    #[test]
    fn test_disease_unicorn_horn() {
        let mut rng = test_rng();
        assert!(!disease_check(false, true, &mut rng));
    }

    #[test]
    fn test_disease_chance() {
        let mut rng = test_rng();
        let mut infected = 0;
        for _ in 0..800 {
            if disease_check(false, false, &mut rng) {
                infected += 1;
            }
        }
        // ~12.5% (1/8)
        assert!(infected > 60 && infected < 160, "감염: {}", infected);
    }

    // --- u_slip_free_check ---

    #[test]
    fn test_slip_amorphous() {
        let mut rng = test_rng();
        assert!(u_slip_free_check(true, false, false, 10, &mut rng));
    }

    #[test]
    fn test_slip_greased() {
        let mut rng = test_rng();
        assert!(u_slip_free_check(false, false, true, 10, &mut rng));
    }

    #[test]
    fn test_slip_normal() {
        let mut rng = test_rng();
        // 민첩 10, 그리스/비정형 없음 — 풀려남 확률 낮음
        let mut slips = 0;
        for _ in 0..200 {
            if u_slip_free_check(false, false, false, 10, &mut rng) {
                slips += 1;
            }
        }
        assert!(slips < 100, "일반 풀려남 낮음: {}", slips);
    }
}
