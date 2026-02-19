// artifact_ext.rs — artifact.c 핵심 로직 순수 결과 패턴 이식
// [v2.14.0] 신규 생성: 아티팩트 데미지/터치/Magicbane/글로우 등 10개 함수
// 원본: NetHack 3.6.7 src/artifact.c (2,206줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// Magicbane 최대 다이스 롤 (이 이상은 마법 효과 없음)
const MB_MAX_DIEROLL: i32 = 8;

/// 치명적 데미지 모디파이어
const FATAL_DAMAGE_MODIFIER: i32 = 200;

// ============================================================
// 열거형
// ============================================================

/// 아티팩트 공격 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactAttackType {
    Physical,
    Fire,
    Cold,
    Electric,
    Magic,
    Stun,
    Poison,
    DrainLife,
    Stone,
    None,
}

/// Magicbane 특수 효과 인덱스
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MbEffect {
    Probe,
    Stun,
    Scare,
    Cancel,
}

/// Magicbane 타격 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MbHitResult {
    /// 추가 데미지 (기본 d4 외)
    pub extra_damage: i32,
    /// 최종 효과
    pub effect: MbEffect,
    /// 기절 적용 여부
    pub do_stun: bool,
    /// 혼란 적용 여부 (1/12 확률)
    pub do_confuse: bool,
}

/// 아티팩트 터치 데미지 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchBlastResult {
    /// 데미지 양
    pub damage: i32,
    /// 터치 가능 여부 (false면 들 수 없음)
    pub can_pick: bool,
}

/// 글로우 강도 (무기 빛남)
/// 원본: artifact.c glow_strength() L1472-1483
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlowStrength {
    /// 0: 빛나지 않음
    None,
    /// 1-3: 약하게 빛남
    Faint,
    /// 4-6: 보통
    Moderate,
    /// 7+: 강하게 빛남
    Bright,
}

// ============================================================
// 1. spec_dbon_calc — 아티팩트 특수 데미지 보너스 계산
// ============================================================

/// 아티팩트 특수 데미지 보너스 계산
/// 원본: artifact.c spec_dbon() L841-862
/// tmp: 기존 데미지, damd: 아티팩트 다이스 면수
pub fn spec_dbon_calc(damd: i32, tmp: i32, spec_applies: bool, rng: &mut NetHackRng) -> i32 {
    if !spec_applies {
        return 0;
    }
    if damd > 0 {
        rng.rnd(damd)
    } else {
        tmp.max(1)
    }
}

// ============================================================
// 2. spec_abon_calc — 아티팩트 특수 공격 보너스
// ============================================================

/// 아티팩트 특수 공격 보너스
/// 원본: artifact.c spec_abon() L824-838
pub fn spec_abon_calc(damn: i32, spec_applies: bool, rng: &mut NetHackRng) -> i32 {
    if spec_applies && damn > 0 {
        rng.rnd(damn)
    } else {
        0
    }
}

// ============================================================
// 3. touch_artifact_damage — 아티팩트 터치 데미지 계산
// ============================================================

/// 부적절한 아티팩트 터치 시 데미지 계산
/// 원본: artifact.c touch_artifact() L695-700
pub fn touch_artifact_damage(
    has_antimagic: bool,
    is_self_willed: bool,
    is_silver: bool,
    hates_silver: bool,
    rng: &mut NetHackRng,
) -> TouchBlastResult {
    let dice = if has_antimagic { 2 } else { 4 };
    let sides = if is_self_willed { 10 } else { 4 };
    let mut damage = rng.d(dice, sides);

    // 은 소재 + 은 혐오 시 추가 데미지 (절반 적용)
    if is_silver && hates_silver {
        let silver_bonus = rng.rnd(10);
        damage += silver_bonus / 2; // Maybe_Half_Phys 간략화
    }

    // 나쁜 정렬 + 나쁜 클래스 + self-willed → 들 수 없음
    // 여기서는 단순히 데미지만 반환, can_pick은 호출자가 판단
    TouchBlastResult {
        damage,
        can_pick: true,
    }
}

// ============================================================
// 4. mb_hit_calc — Magicbane 타격 효과 계산
// ============================================================

/// Magicbane 타격 시 특수 효과 및 추가 데미지 계산
/// 원본: artifact.c Mb_hit() L962-1141
pub fn mb_hit_calc(
    weapon_spe: i32,
    dieroll: i32,
    spec_dbon_applies: bool,
    rng: &mut NetHackRng,
) -> MbHitResult {
    // scare_dieroll: MB_MAX_DIEROLL/2 = 4, spe ≥ 3이면 나눔
    let mut scare_dieroll = MB_MAX_DIEROLL / 2;
    if weapon_spe >= 3 {
        scare_dieroll /= 1 << (weapon_spe / 3);
    }

    // spec_dbon 미적용 시 dieroll +1
    let adjusted_dieroll = if !spec_dbon_applies {
        dieroll + 1
    } else {
        dieroll
    };

    // 기절 판정
    let stun_chance_mod = if spec_dbon_applies { 11 } else { 7 };
    let do_stun = weapon_spe.max(0) < rng.rn2(stun_chance_mod);

    // 기본 추가 데미지 + 효과 결정
    let mut extra_damage = rng.rnd(4); // 기본 (2..3)d4
    let mut effect = MbEffect::Probe;

    if do_stun {
        effect = MbEffect::Stun;
        extra_damage += rng.rnd(4); // (3..4)d4
    }
    if adjusted_dieroll <= scare_dieroll {
        effect = MbEffect::Scare;
        extra_damage += rng.rnd(4); // (3..5)d4
    }
    if adjusted_dieroll <= scare_dieroll / 2 {
        effect = MbEffect::Cancel;
        extra_damage += rng.rnd(4); // (4..6)d4
    }

    // 혼란: 1/12 확률
    let do_confuse = rng.rn2(12) == 0;

    MbHitResult {
        extra_damage,
        effect,
        do_stun,
        do_confuse,
    }
}

// ============================================================
// 5. glow_strength — 글로우 강도 결정
// ============================================================

/// 아티팩트 무기의 글로우(빛남) 강도 결정
/// 원본: artifact.c glow_strength() L1472-1483
pub fn glow_strength(spe: i32) -> GlowStrength {
    if spe <= 0 {
        GlowStrength::None
    } else if spe <= 3 {
        GlowStrength::Faint
    } else if spe <= 6 {
        GlowStrength::Moderate
    } else {
        GlowStrength::Bright
    }
}

// ============================================================
// 6. artifact_fire_destroy — 화염 아티팩트 아이템 파괴 확률
// ============================================================

/// 화염 아티팩트 피격 시 아이템 파괴 확률
/// 원본: artifact.c artifact_hit() L1195-1200
pub fn artifact_fire_destroy_check(item_class: &str, rng: &mut NetHackRng) -> bool {
    match item_class {
        "potion" => rng.rn2(4) == 0,    // 1/4
        "scroll" => rng.rn2(4) == 0,    // 1/4
        "spellbook" => rng.rn2(7) == 0, // 1/7
        _ => false,
    }
}

// ============================================================
// 7. artifact_cold_destroy — 냉기 아티팩트 아이템 파괴 확률
// ============================================================

/// 냉기 아티팩트 피격 시 물약 파괴 확률
/// 원본: artifact.c artifact_hit() L1208
pub fn artifact_cold_destroy_check(rng: &mut NetHackRng) -> bool {
    rng.rn2(4) == 0 // 1/4
}

// ============================================================
// 8. vorpal_chance — 참수 확률 (Vorpal Blade/Tsurugi)
// ============================================================

/// Vorpal Blade 참수 확률
/// 원본: artifact.c artifact_hit() L1297 — dieroll == 1
pub fn vorpal_chance(dieroll: i32) -> bool {
    dieroll == 1
}

/// Tsurugi of Muramasa 이등분 확률
/// 원본: artifact.c artifact_hit() L1344 — dieroll == 1
pub fn bisect_chance(dieroll: i32) -> bool {
    dieroll == 1
}

// ============================================================
// 9. fatal_damage — 치명적 데미지 계산
// ============================================================

/// 즉사 판정용 치명적 데미지 계산
/// 원본: artifact.c L37-41
pub fn fatal_damage(target_hp: i32) -> i32 {
    2 * target_hp + FATAL_DAMAGE_MODIFIER
}

// ============================================================
// 10. count_surround_traps — 주변 함정 수
// ============================================================

/// Orcrist 보유 시 글로우 강도를 위한 주변 오크/함정 수 추정
/// 원본: artifact.c count_surround_traps()
/// 단순화: 반경 내 인접 셀 수 반환 (호출자가 카운트)
pub fn glow_intensity_from_count(trap_count: i32) -> GlowStrength {
    if trap_count == 0 {
        GlowStrength::None
    } else if trap_count <= 2 {
        GlowStrength::Faint
    } else if trap_count <= 4 {
        GlowStrength::Moderate
    } else {
        GlowStrength::Bright
    }
}

// ============================================================
// 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    fn test_rng() -> NetHackRng {
        NetHackRng::new(42)
    }

    // --- spec_dbon_calc ---
    #[test]
    fn test_dbon_no_apply() {
        let mut rng = test_rng();
        assert_eq!(spec_dbon_calc(6, 5, false, &mut rng), 0);
    }

    #[test]
    fn test_dbon_with_dice() {
        let mut rng = test_rng();
        let dmg = spec_dbon_calc(6, 5, true, &mut rng);
        assert!(dmg >= 1 && dmg <= 6, "데미지: {}", dmg);
    }

    #[test]
    fn test_dbon_zero_dice() {
        let mut rng = test_rng();
        // damd=0이면 max(tmp, 1) 반환
        assert_eq!(spec_dbon_calc(0, 5, true, &mut rng), 5);
        assert_eq!(spec_dbon_calc(0, 0, true, &mut rng), 1);
    }

    // --- spec_abon_calc ---
    #[test]
    fn test_abon_applies() {
        let mut rng = test_rng();
        let bonus = spec_abon_calc(4, true, &mut rng);
        assert!(bonus >= 1 && bonus <= 4, "보너스: {}", bonus);
    }

    #[test]
    fn test_abon_no_apply() {
        let mut rng = test_rng();
        assert_eq!(spec_abon_calc(4, false, &mut rng), 0);
    }

    // --- touch_artifact_damage ---
    #[test]
    fn test_touch_no_antimagic() {
        let mut rng = test_rng();
        let result = touch_artifact_damage(false, true, false, false, &mut rng);
        // 4d10: 4~40
        assert!(
            result.damage >= 4 && result.damage <= 40,
            "데미지: {}",
            result.damage
        );
    }

    #[test]
    fn test_touch_with_antimagic() {
        let mut rng = test_rng();
        let result = touch_artifact_damage(true, false, false, false, &mut rng);
        // 2d4: 2~8
        assert!(
            result.damage >= 2 && result.damage <= 8,
            "데미지: {}",
            result.damage
        );
    }

    #[test]
    fn test_touch_silver() {
        let mut rng = test_rng();
        let result = touch_artifact_damage(false, false, true, true, &mut rng);
        // 4d4 + rnd(10)/2: 4+1=5 ~ 16+5=21
        assert!(
            result.damage >= 5 && result.damage <= 21,
            "은 데미지: {}",
            result.damage
        );
    }

    // --- mb_hit_calc ---
    #[test]
    fn test_mb_low_dieroll() {
        let mut rng = test_rng();
        let result = mb_hit_calc(0, 1, true, &mut rng);
        // dieroll=1 ≤ scare_dieroll/2=2 → Cancel
        assert_eq!(result.effect, MbEffect::Cancel);
        assert!(result.extra_damage >= 4, "최소 4d4 추가 데미지");
    }

    #[test]
    fn test_mb_high_dieroll() {
        let mut rng = test_rng();
        let result = mb_hit_calc(0, 8, true, &mut rng);
        // dieroll=8 > scare_dieroll=4 → Probe 또는 Stun
        assert!(
            result.effect == MbEffect::Probe || result.effect == MbEffect::Stun,
            "높은 롤: {:?}",
            result.effect
        );
    }

    #[test]
    fn test_mb_high_spe() {
        let mut rng = test_rng();
        let result = mb_hit_calc(9, 3, true, &mut rng);
        // spe=9 → scare_dieroll = 4 / (1<<3) = 0
        // dieroll=3 > 0 → Stun 또는 Probe
        assert!(
            result.effect != MbEffect::Cancel && result.effect != MbEffect::Scare,
            "높은 강화에서 Cancel/Scare 안 됨"
        );
    }

    // --- glow_strength ---
    #[test]
    fn test_glow() {
        assert_eq!(glow_strength(0), GlowStrength::None);
        assert_eq!(glow_strength(2), GlowStrength::Faint);
        assert_eq!(glow_strength(5), GlowStrength::Moderate);
        assert_eq!(glow_strength(10), GlowStrength::Bright);
    }

    // --- artifact_fire_destroy_check ---
    #[test]
    fn test_fire_destroy_potion() {
        let mut rng = test_rng();
        let mut destroyed = 0;
        for _ in 0..400 {
            if artifact_fire_destroy_check("potion", &mut rng) {
                destroyed += 1;
            }
        }
        // ~25% 확률
        assert!(
            destroyed > 60 && destroyed < 140,
            "물약 파괴: {}",
            destroyed
        );
    }

    // --- vorpal/bisect ---
    #[test]
    fn test_vorpal() {
        assert!(vorpal_chance(1));
        assert!(!vorpal_chance(2));
    }

    #[test]
    fn test_bisect() {
        assert!(bisect_chance(1));
        assert!(!bisect_chance(5));
    }

    // --- fatal_damage ---
    #[test]
    fn test_fatal() {
        assert_eq!(fatal_damage(50), 300); // 2*50 + 200
        assert_eq!(fatal_damage(100), 400);
    }

    // --- glow_intensity_from_count ---
    #[test]
    fn test_glow_count() {
        assert_eq!(glow_intensity_from_count(0), GlowStrength::None);
        assert_eq!(glow_intensity_from_count(1), GlowStrength::Faint);
        assert_eq!(glow_intensity_from_count(3), GlowStrength::Moderate);
        assert_eq!(glow_intensity_from_count(5), GlowStrength::Bright);
    }
}
