// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original: Copyright (c) Stichting Mathematisch Centrum, Amsterdam, 1985.
// NetHack may be freely redistributed. See license for details.
//
// [v2.22.0 R34-3] 아티팩트 확장 모듈 (artifact_ext.rs)
// 원본: NetHack 3.6.7 artifact.c (순수 계산 로직)
// ============================================================================

use crate::util::rng::NetHackRng;
use serde::{Deserialize, Serialize};

// =============================================================================
// [1] 상수 및 열거형 (원본: artifact.h, artilist.h 매크로)
// =============================================================================

/// 아티팩트 특수 플래그 (원본: SPFX_* 매크로)
pub mod spfx {
    pub const NONE: u64 = 0x00000000;
    /// 탐색 부여 (Searching)
    pub const SEARCH: u64 = 0x00000001;
    /// 환각 저항
    pub const HALRES: u64 = 0x00000002;
    /// 텔레파시 (ESP)
    pub const ESP: u64 = 0x00000004;
    /// 은신 (Stealth)
    pub const STLTH: u64 = 0x00000008;
    /// 재생 (Regeneration)
    pub const REGEN: u64 = 0x00000010;
    /// 순간이동 제어 (Teleport Control)
    pub const TCTRL: u64 = 0x00000020;
    /// 에너지 재생 (Energy Regeneration)
    pub const EREGEN: u64 = 0x00000040;
    /// 반마법 피해 (Half Spell Damage)
    pub const HSPDAM: u64 = 0x00000080;
    /// 반물리 피해 (Half Physical Damage)
    pub const HPHDAM: u64 = 0x00000100;
    /// 맞은 대상 경고 (Warning against specific monster)
    pub const WARN: u64 = 0x00000200;
    /// 대상 관통 (DBONUS적용 대상)
    pub const DBONUS: u64 = 0x00000400;
    /// 참수 (Vorpal/Behead)
    pub const BEHEAD: u64 = 0x00000800;
    /// 공격 부여 (special attack)
    pub const ATTK: u64 = 0x00001000;
    /// 지능 (self-willed)
    pub const INTEL: u64 = 0x00002000;
    /// 정렬 제한 (alignment restricted)
    pub const RESTR: u64 = 0x00004000;
    /// 반사 (Reflect)
    pub const REFLECT: u64 = 0x00008000;
    /// 보호 (Protection)
    pub const PROTECT: u64 = 0x00010000;
    /// X-레이 시야
    pub const XRAY: u64 = 0x00020000;
    /// 생성 불가 (No-gen)
    pub const NOGEN: u64 = 0x00040000;
    /// 말하는 아티팩트
    pub const SPEAK: u64 = 0x00080000;
    /// 행운 부여
    pub const LUCK: u64 = 0x00100000;
    /// 대상 특정 몬스터
    pub const DMONS: u64 = 0x00200000;
    /// 대상 특정 클래스
    pub const DCLAS: u64 = 0x00400000;
    /// 대상 MF1 플래그
    pub const DFLAG1: u64 = 0x00800000;
    /// 대상 MF2 플래그
    pub const DFLAG2: u64 = 0x01000000;
    /// 대상 정렬
    pub const DALIGN: u64 = 0x02000000;
}

/// 공격 속성 형태 (원본: AD_* 매크로)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackType {
    Physical,
    Fire,
    Cold,
    Shock,
    MagicMissile,
    Disintegrate,
    Poison,
    DrainLife,
    Stun,
    Petrify,
    None,
}

// =============================================================================
// [2] 아티팩트 정의 구조체 (원본: struct artifact, artilist.h 테이블)
// =============================================================================

/// [v2.22.0 R34-3] 아티팩트 공격/방어 정의
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactAttack {
    /// 공격 속성
    pub adtyp: AttackType,
    /// 명중 다이스 수 (damn)
    pub to_hit_dice: i32,
    /// 피해 다이스 수 (damd)
    pub damage_dice: i32,
}

impl Default for ArtifactAttack {
    fn default() -> Self {
        Self {
            adtyp: AttackType::None,
            to_hit_dice: 0,
            damage_dice: 0,
        }
    }
}

/// [v2.22.0 R34-3] 아티팩트 정의 (원본: struct artifact)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDef {
    /// 아티팩트 이름
    pub name: String,
    /// 기반 아이템 타입명
    pub base_item: String,
    /// 특수 플래그 (착용/장비 시)
    pub spfx: u64,
    /// 소지 시 특수 플래그
    pub cspfx: u64,
    /// 공격 정의
    pub attk: ArtifactAttack,
    /// 방어 정의
    pub defn: ArtifactAttack,
    /// 소지 시 방어
    pub cary: ArtifactAttack,
    /// 대상 몬스터 타입 / M2 플래그
    pub mtype: u64,
    /// 정렬 (0=없음, 1=질서, -1=혼돈, 2=-2=중립)
    pub alignment: i32,
    /// 전용 직업 (빈 문자열이면 제한 없음)
    pub role: String,
    /// 전용 종족
    pub race: String,
    /// 기본 가격 (0이면 기반 아이템 × 100)
    pub cost: i64,
    /// 발동 속성 ID
    pub inv_prop: i32,
    /// 색상 인덱스
    pub acolor: i32,
}

// =============================================================================
// [3] 아티팩트 피해/보너스 순수 계산 (원본: spec_abon, spec_dbon)
// =============================================================================

/// [v2.22.0 R34-3] 아티팩트 특수 명중 보너스 (원본: artifact.c:824 spec_abon)
/// `applies`: spec_applies 결과 (대상에 특수공격이 적용되는지)
pub fn calc_spec_abon(attk: &ArtifactAttack, applies: bool, rng: &mut NetHackRng) -> i32 {
    if attk.to_hit_dice > 0 && applies {
        rng.rnd(attk.to_hit_dice)
    } else {
        0
    }
}

/// [v2.22.0 R34-3] 아티팩트 특수 피해 보너스 (원본: artifact.c:840 spec_dbon)
/// `base_damage`: 원래 피해 (max(tmp, 1)로 최소 1)
/// `is_grimtooth`: Grimtooth는 대상 제한 무시
/// `applies`: spec_applies 결과
pub fn calc_spec_dbon(
    attk: &ArtifactAttack,
    base_damage: i32,
    applies: bool,
    rng: &mut NetHackRng,
) -> (i32, bool) {
    // NO_ATTK 체크: 물리 + 주사위 없음 = 특수 공격 없음
    if attk.adtyp == AttackType::Physical && attk.to_hit_dice == 0 && attk.damage_dice == 0 {
        return (0, false);
    }

    if applies {
        let dmg = if attk.damage_dice > 0 {
            rng.rnd(attk.damage_dice)
        } else {
            base_damage.max(1)
        };
        (dmg, true)
    } else {
        (0, false)
    }
}

// =============================================================================
// [4] Magicbane 특수 효과 순수 계산 (원본: artifact.c:961 Mb_hit)
// =============================================================================

/// Magicbane 효과 인덱스 (원본: enum mb_effect_indices)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbEffect {
    Probe = 0,
    Stun = 1,
    Scare = 2,
    Cancel = 3,
}

/// Magicbane 동사 테이블 (원본: mb_verb[][])
pub const MB_VERB_NORMAL: [&str; 4] = ["probe", "stun", "scare", "cancel"];
pub const MB_VERB_HALLUC: [&str; 4] = ["prod", "amaze", "tickle", "purge"];
/// 최대 마법 주사위 (원본: MB_MAX_DIEROLL = 8)
pub const MB_MAX_DIEROLL: i32 = 8;

/// [v2.22.0 R34-3] Magicbane 효과 및 추가 피해 계산 결과
#[derive(Debug, Clone)]
pub struct MbHitResult {
    /// 선택된 효과
    pub effect: MbEffect,
    /// 추가 피해량
    pub extra_damage: i32,
    /// 스턴 적용 여부
    pub do_stun: bool,
    /// 혼란 적용 여부
    pub do_confuse: bool,
    /// 효과 적용 대상이 저항했는지
    pub resisted: bool,
}

/// [v2.22.0 R34-3] Magicbane 효과 계산 (원본: artifact.c:961 Mb_hit 핵심 로직)
/// `spe`: Magicbane의 인챈트 레벨
/// `dieroll`: d20 명중 주사위
/// `spec_dbon_applies`: 특수 피해 보너스 적용 여부
/// `defender_mr`: 방어자의 마법 저항률 (0-100)
pub fn calc_mb_hit(
    spe: i32,
    dieroll: i32,
    spec_dbon_applies: bool,
    defender_mr: i32,
    rng: &mut NetHackRng,
) -> MbHitResult {
    // [원본: artifact.c:979] 높은 인챈트일수록 심각한 효과가 줄어듦
    let mut scare_dieroll = MB_MAX_DIEROLL / 2;
    if spe >= 3 {
        scare_dieroll /= 1 << (spe / 3);
    }

    // 특수 피해가 미적용이면 dieroll 불이익
    let effective_dieroll = if !spec_dbon_applies {
        dieroll + 1
    } else {
        dieroll
    };

    // [원본: artifact.c:990] 스턴 여부: spe가 낮을수록 높은 확률
    let stun_threshold = if spec_dbon_applies { 11 } else { 7 };
    let do_stun = spe.max(0) < rng.rn2(stun_threshold);

    // [원본: artifact.c:999-1012] 효과 및 추가 피해 판정
    let mut extra_damage = rng.rnd(4); // 기본 +1d4
    let mut attack_indx = MbEffect::Probe;

    if do_stun {
        attack_indx = MbEffect::Stun;
        extra_damage += rng.rnd(4);
    }
    if effective_dieroll <= scare_dieroll {
        attack_indx = MbEffect::Scare;
        extra_damage += rng.rnd(4);
    }
    if effective_dieroll <= (scare_dieroll / 2) {
        attack_indx = MbEffect::Cancel;
        extra_damage += rng.rnd(4);
    }

    // [원본: artifact.c:1107] 12분의 1 확률로 혼란
    let do_confuse = rng.rn2(12) == 0;

    // 방어자 MR에 따른 저항 (scare/cancel에서 mr 체크)
    let resisted = match attack_indx {
        MbEffect::Scare => rng.rn2(2) != 0 && rng.rn2(100) < defender_mr,
        MbEffect::Cancel => rng.rn2(100) < defender_mr,
        _ => false,
    };

    MbHitResult {
        effect: attack_indx,
        extra_damage,
        do_stun: do_stun && attack_indx != MbEffect::Stun, // 이미 stun이면 별도 stun 불필요
        do_confuse,
        resisted,
    }
}

// =============================================================================
// [5] 아티팩트 가격/빛/면역 순수 계산
// =============================================================================

/// [v2.22.0 R34-3] 아티팩트 가격 (원본: artifact.c:1729 arti_cost)
/// `artifact_cost`: 아티팩트 고유 가격 (0이면 기반 아이템 × 100)
/// `base_item_cost`: 기반 아이템의 기본 가격
pub fn calc_arti_cost(artifact_cost: i64, base_item_cost: i64) -> i64 {
    if artifact_cost > 0 {
        artifact_cost
    } else {
        100 * base_item_cost
    }
}

/// [v2.22.0 R34-3] 아티팩트가 항상 빛나는지 (원본: artifact.c:1690 artifact_light)
pub fn artifact_always_lit(spfx: u64) -> bool {
    // 원본 사용: oart->inv_prop == LIGHT_AREA || oart->inv_prop == LIGHTNING...
    // 여기서는 SPFX에서 유추 (Sunsword 등)
    // 실제로는 inv_prop 기반이지만, 순수 플래그로 근사
    false // 호출자가 inv_prop으로 판단하도록 위임
}

/// [v2.22.0 R34-3] 아티팩트 침식 면역 (원본: artifact.c:718 arti_immune)
/// 아티팩트 자체가 특정 속성 피해에 면역인지 반환
pub fn calc_arti_immune(
    attk_type: AttackType,
    defn_type: AttackType,
    cary_type: AttackType,
    damage_type: AttackType,
) -> bool {
    if damage_type == AttackType::Physical {
        return false; // 물리 피해에는 아무것도 면역이 아님
    }
    attk_type == damage_type || defn_type == damage_type || cary_type == damage_type
}

/// [v2.22.0 R34-3] 빛 강도에 따른 동사 선택
/// (원본: artifact.c:1863 glow_strength, glow_verb)
pub fn calc_glow_verb(monster_count: i32, hallucinating: bool) -> &'static str {
    let strength = if monster_count > 12 {
        3 // "gleam"
    } else if monster_count > 4 {
        2 // "glimmer"
    } else if monster_count > 0 {
        1 // "flicker"
    } else {
        0 // "quiver" (맹인일 때)
    };

    let verbs_normal = ["quiver", "flicker", "glimmer", "gleam"];
    let verbs_halluc = ["quiver", "flicker", "glimmer", "gleam"]; // 환각 없이 동일

    if hallucinating {
        verbs_halluc[strength as usize]
    } else {
        verbs_normal[strength as usize]
    }
}

// =============================================================================
// [6] 터치 판정 (원본: touch_artifact 핵심 계산)
// =============================================================================

/// [v2.22.0 R34-3] 아티팩트 터치 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TouchResult {
    /// 터치 가능
    Ok,
    /// 반발 피해만 입음 (터치는 성공)
    Blasted { damage: i32 },
    /// 완전 거부 (터치 불가)
    Evaded,
}

/// [v2.22.0 R34-3] 아티팩트 터치 판정 (원본: artifact.c:640 touch_artifact 핵심)
/// `self_willed`: SPFX_INTEL 플래그
/// `badclass`: 직업/종족 불일치
/// `badalign`: 정렬 불일치 또는 bane_applies
/// `has_antimagic`: 반마법 보유
/// `is_silver_hater`: 은 혐오 여부
/// `is_silver_material`: 아이템 은 재질
pub fn calc_touch_artifact(
    self_willed: bool,
    badclass: bool,
    badalign: bool,
    has_antimagic: bool,
    is_silver_material: bool,
    is_silver_hater: bool,
    rng: &mut NetHackRng,
) -> TouchResult {
    let needs_blast = (badclass || badalign) && self_willed;
    let alignment_blast = badalign && rng.rn2(4) == 0;

    if needs_blast || alignment_blast {
        // [원본: artifact.c:695] 피해 계산
        let main_dice = if has_antimagic { 2 } else { 4 };
        let side_dice = if self_willed { 10 } else { 4 };
        let mut dmg = rng.d(main_dice, side_dice);

        // 은 재질 보너스
        if is_silver_material && is_silver_hater {
            let silver_bonus = rng.rnd(10);
            // Maybe_Half_Phys 근사: 절반 적용
            dmg += silver_bonus / 2;
        }

        if badclass && badalign && self_willed {
            return TouchResult::Evaded;
        }

        TouchResult::Blasted { damage: dmg }
    } else {
        TouchResult::Ok
    }
}

// =============================================================================
// [7] spec_applies 순수 계산 (원본: artifact.c:752)
// =============================================================================

/// [v2.22.0 R34-3] 아티팩트 특수 공격이 대상에 적용되는지 계산
/// (원본: artifact.c:752 spec_applies)
#[derive(Debug, Clone)]
pub struct SpecAppliesInput {
    /// 아티팩트 spfx
    pub spfx: u64,
    /// 공격 속성
    pub attk_type: AttackType,
    /// 대상 mtype / M2 플래그
    pub mtype: u64,
    /// 아티팩트의 정렬
    pub alignment: i32,
}

/// [v2.22.0 R34-3] 대상 몬스터의 정보
#[derive(Debug, Clone)]
pub struct TargetInfo {
    /// 몬스터의 M1 플래그
    pub mflags1: u64,
    /// 몬스터의 M2 플래그
    pub mflags2: u64,
    /// 몬스터의 정렬 부호 (-1/0/1)
    pub alignment_sign: i32,
    /// 해당 저항 보유 여부
    pub has_relevant_resistance: bool,
    /// 방어자가 해당 속성 방어 아티팩트를 가지고 있는지
    pub has_defending_artifact: bool,
}

/// [v2.22.0 R34-3] spec_applies 계산
pub fn calc_spec_applies(input: &SpecAppliesInput, target: &TargetInfo) -> bool {
    if input.spfx & (spfx::DBONUS | spfx::ATTK) == 0 {
        return input.attk_type == AttackType::Physical;
    }

    // 대상 특정 M2 플래그
    if input.spfx & spfx::DFLAG2 != 0 {
        return target.mflags2 & input.mtype != 0;
    }

    // 대상 특정 M1 플래그
    if input.spfx & spfx::DFLAG1 != 0 {
        return target.mflags1 & input.mtype != 0;
    }

    // 대상 정렬 불일치
    if input.spfx & spfx::DALIGN != 0 {
        return target.alignment_sign != input.alignment;
    }

    // 공격 속성 (저항 체크)
    if input.spfx & spfx::ATTK != 0 {
        if target.has_defending_artifact {
            return false;
        }
        return !target.has_relevant_resistance;
    }

    false
}

// =============================================================================
// [8] 참수(Vorpal) 판정 (원본: artifact_hit 내 behead 부분)
// =============================================================================

/// [v2.22.0 R34-3] 참수 판정 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeheadResult {
    /// 참수 발생하지 않음
    NoEffect,
    /// 신체 절단 (Tsurugi: dieroll == 1)
    Bisect,
    /// 참수 (Vorpal Blade: dieroll == 1)
    Behead,
    /// 참수 면역 (목 없는 몬스터)
    NeckImmune,
    /// 참수 면역 (아모포스 몬스터)
    AmorphousImmune,
}

/// [v2.22.0 R34-3] 참수 계산 (원본: artifact.c:1250-1395)
/// `is_tsurugi`: 무라마사의 태도인지
/// `is_vorpal`: Vorpal Blade인지
/// `dieroll`: d20 명중 주사위 (1이면 참수 트리거)
/// `defender_has_neck`: 방어자에게 목이 있는지
/// `defender_is_amorphous`: 방어자가 아모포스인지
/// `defender_is_unsolid`: 방어자가 비고체인지
pub fn calc_behead(
    is_tsurugi: bool,
    is_vorpal: bool,
    dieroll: i32,
    defender_has_neck: bool,
    defender_is_amorphous: bool,
    defender_is_unsolid: bool,
) -> BeheadResult {
    if dieroll != 1 {
        return BeheadResult::NoEffect;
    }

    if is_tsurugi {
        return BeheadResult::Bisect;
    }

    if is_vorpal {
        if defender_is_amorphous || defender_is_unsolid {
            return BeheadResult::AmorphousImmune;
        }
        if !defender_has_neck {
            return BeheadResult::NeckImmune;
        }
        return BeheadResult::Behead;
    }

    BeheadResult::NoEffect
}

// =============================================================================
// [9] Master Key of Thievery 판정 (원본: is_magic_key)
// =============================================================================

/// [v2.22.0 R34-3] 마스터 키 마법 판정 (원본: artifact.c:2172 is_magic_key)
/// `is_rogue`: 검사 대상이 도적 직업인지
/// `buc_blessed`: 축복 여부
/// `buc_cursed`: 저주 여부
pub fn is_magic_key(is_rogue: bool, buc_blessed: bool, buc_cursed: bool) -> bool {
    if is_rogue {
        !buc_cursed
    } else {
        buc_blessed
    }
}

// =============================================================================
// [10] 테스트
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spec_abon_applies() {
        let mut rng = NetHackRng::new(42);
        let attk = ArtifactAttack {
            adtyp: AttackType::Fire,
            to_hit_dice: 5,
            damage_dice: 4,
        };
        let bonus = calc_spec_abon(&attk, true, &mut rng);
        assert!(bonus >= 1 && bonus <= 5);
    }

    #[test]
    fn test_spec_abon_not_applies() {
        let mut rng = NetHackRng::new(42);
        let attk = ArtifactAttack {
            adtyp: AttackType::Fire,
            to_hit_dice: 5,
            damage_dice: 4,
        };
        assert_eq!(calc_spec_abon(&attk, false, &mut rng), 0);
    }

    #[test]
    fn test_spec_dbon_no_attk() {
        let mut rng = NetHackRng::new(42);
        let attk = ArtifactAttack {
            adtyp: AttackType::Physical,
            to_hit_dice: 0,
            damage_dice: 0,
        };
        let (dmg, applies) = calc_spec_dbon(&attk, 5, true, &mut rng);
        assert_eq!(dmg, 0);
        assert!(!applies);
    }

    #[test]
    fn test_spec_dbon_with_dice() {
        let mut rng = NetHackRng::new(42);
        let attk = ArtifactAttack {
            adtyp: AttackType::Fire,
            to_hit_dice: 2,
            damage_dice: 6,
        };
        let (dmg, applies) = calc_spec_dbon(&attk, 5, true, &mut rng);
        assert!(dmg >= 1 && dmg <= 6);
        assert!(applies);
    }

    #[test]
    fn test_spec_dbon_no_dice() {
        let mut rng = NetHackRng::new(42);
        let attk = ArtifactAttack {
            adtyp: AttackType::Cold,
            to_hit_dice: 2,
            damage_dice: 0,
        };
        let (dmg, applies) = calc_spec_dbon(&attk, 5, true, &mut rng);
        assert_eq!(dmg, 5); // max(tmp, 1) = max(5, 1) = 5
        assert!(applies);
    }

    #[test]
    fn test_mb_hit_basic() {
        let mut rng = NetHackRng::new(42);
        let result = calc_mb_hit(0, 3, true, 0, &mut rng);
        assert!(result.extra_damage > 0);
    }

    #[test]
    fn test_mb_hit_high_spe() {
        let mut rng = NetHackRng::new(42);
        let result = calc_mb_hit(9, 2, true, 0, &mut rng);
        // 높은 spe에서 scare_dieroll이 크게 줄어들어 probe/stun 위주
        assert!(result.extra_damage > 0);
    }

    #[test]
    fn test_arti_cost() {
        // 고유 가격이 있는 경우
        assert_eq!(calc_arti_cost(4000, 50), 4000);
        // 고유 가격이 0인 경우: base × 100
        assert_eq!(calc_arti_cost(0, 50), 5000);
    }

    #[test]
    fn test_arti_immune() {
        // 화염 공격 아티팩트는 화염 침식에 면역
        assert!(calc_arti_immune(
            AttackType::Fire,
            AttackType::None,
            AttackType::None,
            AttackType::Fire
        ));
        // 물리에는 면역 없음
        assert!(!calc_arti_immune(
            AttackType::Fire,
            AttackType::None,
            AttackType::None,
            AttackType::Physical
        ));
        // 불일치 속성
        assert!(!calc_arti_immune(
            AttackType::Fire,
            AttackType::None,
            AttackType::None,
            AttackType::Cold
        ));
    }

    #[test]
    fn test_glow_verb() {
        assert_eq!(calc_glow_verb(0, false), "quiver");
        assert_eq!(calc_glow_verb(2, false), "flicker");
        assert_eq!(calc_glow_verb(8, false), "glimmer");
        assert_eq!(calc_glow_verb(20, false), "gleam");
    }

    #[test]
    fn test_touch_ok() {
        let mut rng = NetHackRng::new(42);
        let result = calc_touch_artifact(false, false, false, false, false, false, &mut rng);
        assert_eq!(result, TouchResult::Ok);
    }

    #[test]
    fn test_touch_evaded() {
        let mut rng = NetHackRng::new(99);
        let result = calc_touch_artifact(true, true, true, false, false, false, &mut rng);
        assert_eq!(result, TouchResult::Evaded);
    }

    #[test]
    fn test_behead_tsurugi() {
        assert_eq!(
            calc_behead(true, false, 1, true, false, false),
            BeheadResult::Bisect
        );
        assert_eq!(
            calc_behead(true, false, 2, true, false, false),
            BeheadResult::NoEffect
        );
    }

    #[test]
    fn test_behead_vorpal() {
        assert_eq!(
            calc_behead(false, true, 1, true, false, false),
            BeheadResult::Behead
        );
        assert_eq!(
            calc_behead(false, true, 1, false, false, false),
            BeheadResult::NeckImmune
        );
        assert_eq!(
            calc_behead(false, true, 1, true, true, false),
            BeheadResult::AmorphousImmune
        );
    }

    #[test]
    fn test_is_magic_key() {
        // 도적: 저주 안 됐으면 마법 키
        assert!(is_magic_key(true, false, false));
        assert!(is_magic_key(true, true, false));
        assert!(!is_magic_key(true, false, true));
        // 비도적: 축복이어야 마법 키
        assert!(is_magic_key(false, true, false));
        assert!(!is_magic_key(false, false, false));
    }

    #[test]
    fn test_spec_applies_dflag2() {
        let input = SpecAppliesInput {
            spfx: spfx::DBONUS | spfx::DFLAG2,
            attk_type: AttackType::Physical,
            mtype: 0x00040000, // M2_ORC
            alignment: 0,
        };
        let target_orc = TargetInfo {
            mflags1: 0,
            mflags2: 0x00040000,
            alignment_sign: -1,
            has_relevant_resistance: false,
            has_defending_artifact: false,
        };
        assert!(calc_spec_applies(&input, &target_orc));

        let target_human = TargetInfo {
            mflags1: 0,
            mflags2: 0x00000000,
            alignment_sign: 1,
            has_relevant_resistance: false,
            has_defending_artifact: false,
        };
        assert!(!calc_spec_applies(&input, &target_human));
    }

    #[test]
    fn test_spec_applies_attk_resisted() {
        let input = SpecAppliesInput {
            spfx: spfx::ATTK,
            attk_type: AttackType::Fire,
            mtype: 0,
            alignment: 0,
        };
        let resistant = TargetInfo {
            mflags1: 0,
            mflags2: 0,
            alignment_sign: 0,
            has_relevant_resistance: true,
            has_defending_artifact: false,
        };
        assert!(!calc_spec_applies(&input, &resistant));

        let not_resistant = TargetInfo {
            mflags1: 0,
            mflags2: 0,
            alignment_sign: 0,
            has_relevant_resistance: false,
            has_defending_artifact: false,
        };
        assert!(calc_spec_applies(&input, &not_resistant));
    }
}
