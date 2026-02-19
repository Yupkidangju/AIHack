// =============================================================================
// AIHack - explode_ext.rs
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// [v2.11.0] explode.c 미이식 핵심 로직 완전 이식 (순수 결과 패턴)
// 원본: nethack-3.6.7/src/explode.c (820줄)
//
// 이 모듈은 기존 explode.rs에서 누락된 원본 C 함수 로직을
// 순수 결과(pure result) 패턴으로 이식합니다.
//
// 이식된 로직:
//   1. retributive_damage_reduction — 역할별 보복 공격 감소 (explode.c:68-80)
//   2. explosion_adtype — 폭발 타입 → 공격 타입 결정 (explode.c:125-160)
//   3. resistance_mask_result — 3x3 저항 마스크 계산 (explode.c:162-255)
//   4. opposite_resistance_bonus — 반대 속성 2배 데미지 (explode.c:440-443)
//   5. grab_damage_bonus — 잡기 2배 데미지 (explode.c:434-435)
//   6. half_physical_damage — 물리 절반 감소 (explode.c:501-502)
//   7. scatter_direction_result — 파편 방향/범위 결정 (explode.c:600-704)
//   8. scatter_fracture_chance — 바위/석상 파쇄 확률 (explode.c:644-673)
//   9. scatter_destroy_chance — 유리/달걀 파괴 확률 (explode.c:675-679)
//  10. splatter_oil_damage — 기름 폭발 데미지 (explode.c:792-802)
//  11. wake_range — 폭발 소음 범위 (explode.c:570-575)
//  12. explosion_description — 폭발 설명 문자열 (explode.c:127-160)
// =============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// 폭발 공격 타입 (원본 adtyp)
// =============================================================================

/// 폭발 공격 타입 (원본 AD_* 상수)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackDamageType {
    /// 물리 (AD_PHYS)
    Physical,
    /// 마법 (AD_MAGM)
    MagicMissile,
    /// 화염 (AD_FIRE)
    Fire,
    /// 냉기 (AD_COLD)
    Cold,
    /// 분해 (AD_DISN)
    Disintegration,
    /// 전기 (AD_ELEC)
    Electricity,
    /// 독 (AD_DRST)
    Poison,
    /// 산성 (AD_ACID)
    Acid,
}

/// 폭발 소스 타입 (원본 olet)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionSource {
    /// 지팡이 (WAND_CLASS)
    Wand,
    /// 몬스터 자폭 (MON_EXPLODE)
    MonsterExplode,
    /// 두루마리 (SCROLL_CLASS)
    Scroll,
    /// 기름 (BURNING_OIL)
    BurningOil,
    /// 기타
    Other,
}

// =============================================================================
// 1. retributive_damage_reduction — 역할별 보복 공격 감소
// [v2.11.0] explode.c:68-80 이식
// =============================================================================

/// 보복 공격(지팡이 파괴) 시 역할별 데미지 감소 (원본 explode.c:68-80)
/// Priest/Monk/Wizard: 1/5, Healer/Knight: 1/2, 기타: 감소 없음
pub fn retributive_damage_reduction(base_damage: i32, role: &str) -> i32 {
    match role {
        "Priest" | "Monk" | "Wizard" => base_damage / 5,
        "Healer" | "Knight" => base_damage / 2,
        _ => base_damage,
    }
}

// =============================================================================
// 2. explosion_adtype — 폭발 타입 → 공격 타입 결정
// [v2.11.0] explode.c:125-160 이식
// =============================================================================

/// 폭발 zap 타입 → 공격 타입 결정 (원본 explode.c:125-160)
/// zap_type: abs(type) % 10 값
pub fn explosion_adtype(zap_type: i32, source: ExplosionSource) -> Option<AttackDamageType> {
    if source == ExplosionSource::MonsterExplode {
        return Some(AttackDamageType::Physical);
    }

    match zap_type % 10 {
        0 => Some(AttackDamageType::MagicMissile),
        1 => Some(AttackDamageType::Fire),
        2 => Some(AttackDamageType::Cold),
        4 => Some(AttackDamageType::Disintegration),
        5 => Some(AttackDamageType::Electricity),
        6 => Some(AttackDamageType::Poison),
        7 => Some(AttackDamageType::Acid),
        _ => None, // 유효하지 않은 타입
    }
}

// =============================================================================
// 3. explosion_description — 폭발 설명 문자열
// [v2.11.0] explode.c:127-160 이식
// =============================================================================

/// 폭발 설명 문자열 결정 (원본 explode.c:127-160)
pub fn explosion_description(zap_type: i32, source: ExplosionSource) -> &'static str {
    if source == ExplosionSource::MonsterExplode {
        return "explosion";
    }

    match zap_type % 10 {
        0 => "magical blast",
        1 => match source {
            ExplosionSource::BurningOil => "burning oil",
            ExplosionSource::Scroll => "tower of flame",
            _ => "fireball",
        },
        2 => "ball of cold",
        4 => match source {
            ExplosionSource::Wand => "death field",
            _ => "disintegration field",
        },
        5 => "ball of lightning",
        6 => "poison gas cloud",
        7 => "splash of acid",
        _ => "explosion",
    }
}

// =============================================================================
// 4. resistance_mask_result — 저항 마스크 계산
// [v2.11.0] explode.c:162-255 이식
// =============================================================================

/// 폭발 셀 마스크 값
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplodeMask {
    /// 정상 폭발 (데미지 전량)
    Normal,
    /// 저항에 의한 방어 (방어 이펙트 표시)
    Shielded,
    /// 무효 (범위 밖 또는 이미 죽은 대상)
    Skip,
}

/// 특정 대상의 폭발 저항 마스크 계산 (원본 explode.c:172-246)
/// 속성별 저항 보유 여부에 따라 완전 방어/부분/무방어 결정
pub fn resistance_mask_result(
    adtyp: AttackDamageType,
    source: ExplosionSource,
    has_resistance: bool,
    is_nonliving: bool,
    is_demon: bool,
) -> ExplodeMask {
    match adtyp {
        AttackDamageType::Physical => ExplodeMask::Normal,
        AttackDamageType::Disintegration => {
            if source == ExplosionSource::Wand {
                // 지팡이의 죽음 필드: 비생물/악마만 방어
                if is_nonliving || is_demon {
                    ExplodeMask::Shielded
                } else {
                    ExplodeMask::Normal
                }
            } else {
                // 일반 분해: 분해 저항으로만 방어
                if has_resistance {
                    ExplodeMask::Shielded
                } else {
                    ExplodeMask::Normal
                }
            }
        }
        _ => {
            // 화염/냉기/전기/독/산성/마법 → 해당 저항으로 방어
            if has_resistance {
                ExplodeMask::Shielded
            } else {
                ExplodeMask::Normal
            }
        }
    }
}

// =============================================================================
// 5. opposite_resistance_bonus — 반대 속성 2배 데미지
// [v2.11.0] explode.c:440-443 이식
// =============================================================================

/// 반대 속성 저항 보유 시 2배 데미지 (원본 explode.c:440-443)
/// 냉기 저항 + 화염 폭발 → 2x, 화염 저항 + 냉기 폭발 → 2x
pub fn opposite_resistance_bonus(
    damage: i32,
    adtyp: AttackDamageType,
    resists_cold: bool,
    resists_fire: bool,
) -> i32 {
    if resists_cold && adtyp == AttackDamageType::Fire {
        damage * 2
    } else if resists_fire && adtyp == AttackDamageType::Cold {
        damage * 2
    } else {
        damage
    }
}

// =============================================================================
// 6. grab_damage_bonus — 잡기 2배 데미지
// [v2.11.0] explode.c:434-435 이식
// =============================================================================

/// 잡기 상태에서 폭발 범위 내면 2배 데미지 (원본 explode.c:434-435)
/// is_grabbed: 플레이어가 잡힌 상태
/// grabber_in_range: 잡은 자가 폭발 범위 내
pub fn grab_damage_bonus(damage: i32, is_grabbed: bool, grabber_in_range: bool) -> i32 {
    if is_grabbed && grabber_in_range {
        damage * 2
    } else {
        damage
    }
}

// =============================================================================
// 7. half_physical_damage — 물리 절반 감소
// [v2.11.0] explode.c:501-502 이식
// =============================================================================

/// 물리 데미지 절반 감소 (Half_physical_damage 효과)
/// 산성 폭발은 물리+화학 복합이므로 절반 적용
pub fn half_physical_damage(
    damage: i32,
    adtyp: AttackDamageType,
    has_half_physical: bool,
    is_invulnerable: bool,
) -> i32 {
    if is_invulnerable {
        return 0;
    }
    if has_half_physical && (adtyp == AttackDamageType::Physical || adtyp == AttackDamageType::Acid)
    {
        (damage + 1) / 2
    } else {
        damage
    }
}

// =============================================================================
// 8. scatter_direction_result — 파편 방향/범위 결정
// [v2.11.0] explode.c:686-704 이식
// =============================================================================

/// 파편 이동 정보
#[derive(Debug, Clone)]
pub struct ScatterItem {
    /// 이동 방향 dx
    pub dx: i32,
    /// 이동 방향 dy
    pub dy: i32,
    /// 최대 이동 범위
    pub range: i32,
}

/// 8방향 오프셋 (원본 xdir/ydir 배열)
const DIRS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// 파편 방향 및 범위 결정 (원본 scatter 내부 로직)
/// blast_force: 폭발력
/// item_weight: 아이템 무게
pub fn scatter_direction_result(
    blast_force: i32,
    item_weight: i32,
    rng: &mut NetHackRng,
) -> ScatterItem {
    let dir_idx = rng.rn2(8) as usize;
    let (dx, dy) = DIRS[dir_idx];
    let range_max = (blast_force - item_weight / 40).max(1);
    let range = rng.rnd(range_max);
    ScatterItem { dx, dy, range }
}

// =============================================================================
// 9. scatter_fracture_chance — 바위/석상 파쇄 확률
// [v2.11.0] explode.c:644-673 이식
// =============================================================================

/// 파편 파쇄 판정 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractureResult {
    /// 파쇄 성공 (바위 → 돌, 석상 → 파편)
    Fractured,
    /// 파쇄 실패 (원형 유지)
    Intact,
}

/// 바위/석상 파쇄 확률 (9/10) (원본 explode.c:644-646)
/// may_fracture: MAY_FRACTURE 플래그 여부
pub fn scatter_fracture_chance(
    may_fracture: bool,
    is_boulder_or_statue: bool,
    rng: &mut NetHackRng,
) -> FractureResult {
    if may_fracture && is_boulder_or_statue && rng.rn2(10) != 0 {
        FractureResult::Fractured
    } else {
        FractureResult::Intact
    }
}

// =============================================================================
// 10. scatter_destroy_chance — 유리/달걀 파괴 확률
// [v2.11.0] explode.c:675-679 이식
// =============================================================================

/// 아이템 파괴 확률 (1/10 기본, 유리/달걀은 항상) (원본 explode.c:675-677)
/// may_destroy: MAY_DESTROY 플래그
/// is_glass: 유리 재질
/// is_egg: 달걀
pub fn scatter_destroy_chance(
    may_destroy: bool,
    is_glass: bool,
    is_egg: bool,
    rng: &mut NetHackRng,
) -> bool {
    if !may_destroy {
        return false;
    }
    // 유리/달걀은 항상 파괴, 기타는 1/10
    is_glass || is_egg || rng.rn2(10) == 0
}

// =============================================================================
// 11. splatter_oil_damage — 기름 폭발 데미지
// [v2.11.0] explode.c:792-802 이식
// =============================================================================

/// 기름 폭발 데미지 계산 (원본 splatter_burning_oil)
/// diluted: 희석된 기름 여부
pub fn splatter_oil_damage(diluted: bool, rng: &mut NetHackRng) -> i32 {
    if diluted {
        rng.d(3, 4) // 3d4
    } else {
        rng.d(4, 4) // 4d4
    }
}

// =============================================================================
// 12. wake_range — 폭발 소음 범위
// [v2.11.0] explode.c:570-575 이식
// =============================================================================

/// 폭발 소음 범위 (dam*dam, 최소 50) (원본 explode.c:570-575)
/// inside_engulfer: 삼켜진 상태이면 1/4
pub fn wake_range(damage: i32, inside_engulfer: bool) -> i32 {
    let mut i = damage * damage;
    if i < 50 {
        i = 50;
    }
    if inside_engulfer {
        i = (i + 3) / 4;
    }
    i
}

// =============================================================================
// 추가: 몬스터 폭발 피해 계산 종합
// [v2.11.0] explode.c:418-446 핵심 로직 이식
// =============================================================================

/// 폭발에 의한 몬스터 최종 데미지 계산 (원본 explode.c:418-446)
pub fn monster_explosion_damage(
    base_damage: i32,
    mask: ExplodeMask,
    is_grabbed: bool,
    grabber_in_range: bool,
    resists_cold: bool,
    resists_fire: bool,
    adtyp: AttackDamageType,
    item_damage_res: i32,
    item_damage_nonres: i32,
    resists_attack: bool,
) -> i32 {
    match mask {
        ExplodeMask::Skip => 0,
        ExplodeMask::Shielded => {
            // 방어 성공 → 아이템 비저항 손상만 적용
            item_damage_nonres
        }
        ExplodeMask::Normal => {
            let mut mdam = base_damage;
            // 저항 보유 시 절반
            if resists_attack {
                mdam = (base_damage + 1) / 2;
            }
            // 잡기 2배
            mdam = grab_damage_bonus(mdam, is_grabbed, grabber_in_range);
            // 반대 속성 2배
            mdam = opposite_resistance_bonus(mdam, adtyp, resists_cold, resists_fire);
            // 아이템 손상 추가
            mdam + item_damage_res + item_damage_nonres
        }
    }
}

// =============================================================================
// 추가: 삼켜진 상태에서 폭발 형용사
// [v2.11.0] explode.c:344-402 이식
// =============================================================================

/// 삼켜진 상태에서 폭발 시 형용사 (원본 explode.c:344-402)
/// is_animal: 삼킨 대상이 동물인지
pub fn engulf_explosion_adjective(
    adtyp: AttackDamageType,
    source: ExplosionSource,
    is_animal: bool,
) -> &'static str {
    if is_animal {
        match adtyp {
            AttackDamageType::Fire => "heartburn",
            AttackDamageType::Cold => "chilly",
            AttackDamageType::Disintegration => {
                if source == ExplosionSource::Wand {
                    "irradiated by pure energy"
                } else {
                    "perforated"
                }
            }
            AttackDamageType::Electricity => "shocked",
            AttackDamageType::Poison => "poisoned",
            AttackDamageType::Acid => "an upset stomach",
            _ => "fried",
        }
    } else {
        match adtyp {
            AttackDamageType::Fire => "toasted",
            AttackDamageType::Cold => "chilly",
            AttackDamageType::Disintegration => {
                if source == ExplosionSource::Wand {
                    "overwhelmed by pure energy"
                } else {
                    "perforated"
                }
            }
            AttackDamageType::Electricity => "shocked",
            AttackDamageType::Poison => "intoxicated",
            AttackDamageType::Acid => "burned",
            _ => "fried",
        }
    }
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ─── retributive_damage_reduction ────────────────────────────
    #[test]
    fn test_retributive_priest() {
        // Priest/Monk/Wizard → 1/5
        assert_eq!(retributive_damage_reduction(100, "Priest"), 20);
        assert_eq!(retributive_damage_reduction(100, "Monk"), 20);
        assert_eq!(retributive_damage_reduction(100, "Wizard"), 20);
    }

    #[test]
    fn test_retributive_knight() {
        // Healer/Knight → 1/2
        assert_eq!(retributive_damage_reduction(100, "Healer"), 50);
        assert_eq!(retributive_damage_reduction(100, "Knight"), 50);
    }

    #[test]
    fn test_retributive_other() {
        // 기타 → 감소 없음
        assert_eq!(retributive_damage_reduction(100, "Valkyrie"), 100);
        assert_eq!(retributive_damage_reduction(100, "Barbarian"), 100);
    }

    // ─── explosion_adtype ────────────────────────────────────────
    #[test]
    fn test_explosion_adtype_fire() {
        assert_eq!(
            explosion_adtype(1, ExplosionSource::Other),
            Some(AttackDamageType::Fire)
        );
    }

    #[test]
    fn test_explosion_adtype_cold() {
        assert_eq!(
            explosion_adtype(2, ExplosionSource::Other),
            Some(AttackDamageType::Cold)
        );
    }

    #[test]
    fn test_explosion_adtype_monster() {
        assert_eq!(
            explosion_adtype(1, ExplosionSource::MonsterExplode),
            Some(AttackDamageType::Physical)
        );
    }

    #[test]
    fn test_explosion_adtype_invalid() {
        assert_eq!(explosion_adtype(3, ExplosionSource::Other), None);
    }

    // ─── explosion_description ───────────────────────────────────
    #[test]
    fn test_explosion_desc_fireball() {
        assert_eq!(explosion_description(1, ExplosionSource::Other), "fireball");
    }

    #[test]
    fn test_explosion_desc_burning_oil() {
        assert_eq!(
            explosion_description(1, ExplosionSource::BurningOil),
            "burning oil"
        );
    }

    #[test]
    fn test_explosion_desc_tower_of_flame() {
        assert_eq!(
            explosion_description(1, ExplosionSource::Scroll),
            "tower of flame"
        );
    }

    #[test]
    fn test_explosion_desc_death_field() {
        assert_eq!(
            explosion_description(4, ExplosionSource::Wand),
            "death field"
        );
    }

    // ─── resistance_mask_result ──────────────────────────────────
    #[test]
    fn test_mask_physical_always_normal() {
        assert_eq!(
            resistance_mask_result(
                AttackDamageType::Physical,
                ExplosionSource::Other,
                true,
                false,
                false
            ),
            ExplodeMask::Normal
        );
    }

    #[test]
    fn test_mask_fire_with_resistance() {
        assert_eq!(
            resistance_mask_result(
                AttackDamageType::Fire,
                ExplosionSource::Other,
                true,
                false,
                false
            ),
            ExplodeMask::Shielded
        );
    }

    #[test]
    fn test_mask_disintegration_wand_demon() {
        // 지팡이 죽음 필드 + 악마 → 방어
        assert_eq!(
            resistance_mask_result(
                AttackDamageType::Disintegration,
                ExplosionSource::Wand,
                false,
                false,
                true
            ),
            ExplodeMask::Shielded
        );
    }

    #[test]
    fn test_mask_disintegration_wand_living() {
        // 지팡이 죽음 필드 + 생물 + 비악마 → 무방어
        assert_eq!(
            resistance_mask_result(
                AttackDamageType::Disintegration,
                ExplosionSource::Wand,
                false,
                false,
                false
            ),
            ExplodeMask::Normal
        );
    }

    // ─── opposite_resistance_bonus ───────────────────────────────
    #[test]
    fn test_opposite_cold_vs_fire() {
        // 냉기 저항 + 화염 → 2배
        assert_eq!(
            opposite_resistance_bonus(50, AttackDamageType::Fire, true, false),
            100
        );
    }

    #[test]
    fn test_opposite_fire_vs_cold() {
        // 화염 저항 + 냉기 → 2배
        assert_eq!(
            opposite_resistance_bonus(50, AttackDamageType::Cold, false, true),
            100
        );
    }

    #[test]
    fn test_opposite_no_bonus() {
        // 관련 없는 조합 → 보너스 없음
        assert_eq!(
            opposite_resistance_bonus(50, AttackDamageType::Electricity, true, false),
            50
        );
    }

    // ─── grab_damage_bonus ────────────────────────────────────
    #[test]
    fn test_grab_bonus() {
        assert_eq!(grab_damage_bonus(50, true, true), 100);
        assert_eq!(grab_damage_bonus(50, true, false), 50);
        assert_eq!(grab_damage_bonus(50, false, true), 50);
    }

    // ─── half_physical_damage ─────────────────────────────────
    #[test]
    fn test_half_phys_invulnerable() {
        assert_eq!(
            half_physical_damage(100, AttackDamageType::Physical, false, true),
            0
        );
    }

    #[test]
    fn test_half_phys_physical() {
        assert_eq!(
            half_physical_damage(100, AttackDamageType::Physical, true, false),
            50
        );
    }

    #[test]
    fn test_half_phys_acid() {
        // 산성도 물리 절반 적용
        assert_eq!(
            half_physical_damage(100, AttackDamageType::Acid, true, false),
            50
        );
    }

    #[test]
    fn test_half_phys_fire_no_effect() {
        // 화염은 물리 절반 미적용
        assert_eq!(
            half_physical_damage(100, AttackDamageType::Fire, true, false),
            100
        );
    }

    // ─── scatter_direction_result ─────────────────────────────
    #[test]
    fn test_scatter_direction() {
        let mut rng = NetHackRng::new(42);
        let item = scatter_direction_result(10, 200, &mut rng);
        assert!(item.dx >= -1 && item.dx <= 1);
        assert!(item.dy >= -1 && item.dy <= 1);
        assert!(item.range >= 1);
    }

    // ─── scatter_fracture_chance ──────────────────────────────
    #[test]
    fn test_fracture_disabled() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            scatter_fracture_chance(false, true, &mut rng),
            FractureResult::Intact
        );
    }

    #[test]
    fn test_fracture_non_boulder() {
        let mut rng = NetHackRng::new(42);
        assert_eq!(
            scatter_fracture_chance(true, false, &mut rng),
            FractureResult::Intact
        );
    }

    // ─── scatter_destroy_chance ───────────────────────────────
    #[test]
    fn test_destroy_glass_always() {
        let mut rng = NetHackRng::new(42);
        assert!(scatter_destroy_chance(true, true, false, &mut rng));
    }

    #[test]
    fn test_destroy_egg_always() {
        let mut rng = NetHackRng::new(42);
        assert!(scatter_destroy_chance(true, false, true, &mut rng));
    }

    #[test]
    fn test_destroy_disabled() {
        let mut rng = NetHackRng::new(42);
        assert!(!scatter_destroy_chance(false, true, true, &mut rng));
    }

    // ─── splatter_oil_damage ──────────────────────────────────
    #[test]
    fn test_oil_damage_range() {
        let mut rng = NetHackRng::new(42);
        let dmg = splatter_oil_damage(false, &mut rng);
        assert!(dmg >= 4 && dmg <= 20); // 4d4: 4~20 + 4
    }

    #[test]
    fn test_oil_diluted_range() {
        let mut rng = NetHackRng::new(42);
        let dmg = splatter_oil_damage(true, &mut rng);
        assert!(dmg >= 3 && dmg <= 15); // 3d4: 3~15 + 3
    }

    // ─── wake_range ───────────────────────────────────────────
    #[test]
    fn test_wake_range_min() {
        assert_eq!(wake_range(3, false), 50); // 9 < 50 → 50
    }

    #[test]
    fn test_wake_range_normal() {
        assert_eq!(wake_range(10, false), 100); // 100 >= 50
    }

    #[test]
    fn test_wake_range_engulfer() {
        assert_eq!(wake_range(10, true), 25); // (100+3)/4 = 25
    }

    // ─── monster_explosion_damage ─────────────────────────────
    #[test]
    fn test_monster_damage_skip() {
        assert_eq!(
            monster_explosion_damage(
                100,
                ExplodeMask::Skip,
                false,
                false,
                false,
                false,
                AttackDamageType::Fire,
                0,
                0,
                false
            ),
            0
        );
    }

    #[test]
    fn test_monster_damage_shielded() {
        assert_eq!(
            monster_explosion_damage(
                100,
                ExplodeMask::Shielded,
                false,
                false,
                false,
                false,
                AttackDamageType::Fire,
                5,
                10,
                false
            ),
            10 // 비저항 아이템 손상만
        );
    }

    #[test]
    fn test_monster_damage_normal_with_opposite() {
        // 화염 폭발 + 냉기 저항 → 2배 + 아이템 손상
        assert_eq!(
            monster_explosion_damage(
                50,
                ExplodeMask::Normal,
                false,
                false,
                true,
                false,
                AttackDamageType::Fire,
                5,
                10,
                false
            ),
            50 * 2 + 5 + 10 // 100 + 15 = 115
        );
    }

    #[test]
    fn test_monster_damage_resist_half() {
        // 저항 → 절반 + 아이템
        assert_eq!(
            monster_explosion_damage(
                100,
                ExplodeMask::Normal,
                false,
                false,
                false,
                false,
                AttackDamageType::Fire,
                0,
                0,
                true
            ),
            50 // (100+1)/2 = 50
        );
    }

    // ─── engulf_explosion_adjective ────────────────────────────
    #[test]
    fn test_engulf_animal_fire() {
        assert_eq!(
            engulf_explosion_adjective(AttackDamageType::Fire, ExplosionSource::Other, true),
            "heartburn"
        );
    }

    #[test]
    fn test_engulf_nonanimal_fire() {
        assert_eq!(
            engulf_explosion_adjective(AttackDamageType::Fire, ExplosionSource::Other, false),
            "toasted"
        );
    }

    #[test]
    fn test_engulf_disintegration_wand_animal() {
        assert_eq!(
            engulf_explosion_adjective(
                AttackDamageType::Disintegration,
                ExplosionSource::Wand,
                true
            ),
            "irradiated by pure energy"
        );
    }
}
