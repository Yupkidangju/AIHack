// ============================================================================
// [v2.26.0 Phase 90-2] 플레이어→몬스터 전투 확장 (uhitm_phase90_ext.rs)
// 원본: NetHack 3.6.7 src/uhitm.c L1300-2800 핵심 미이식 함수 이식
// 순수 결과 패턴
// ============================================================================

use crate::util::rng::NetHackRng;

// =============================================================================
// [1] 무기 데미지 계산 — weapon_dmg_calc (uhitm.c L1300-1600)
// =============================================================================

/// [v2.26.0 90-2] 무기 데미지 계산 입력
#[derive(Debug, Clone)]
pub struct WeaponDamageInput {
    /// 기본 데미지 다이스 (NdM)
    pub dice_num: i32,
    pub dice_sides: i32,
    /// 강화치 (+enchant)
    pub enchantment: i32,
    /// 축복/저주 (1/-1/0)
    pub buc: i32,
    /// 양손무기 여부
    pub two_handed: bool,
    /// 크리티컬 히트 여부
    pub critical: bool,
    /// 대상 크기 (0=작음, 1=보통, 2=큼)
    pub target_size: i32,
    /// 은 무기 + 대상이 은 취약
    pub silver_vs_vulnerable: bool,
    /// 아티팩트 보너스
    pub artifact_bonus: i32,
    /// 플레이어 힘 보너스
    pub strength_bonus: i32,
}

/// [v2.26.0 90-2] 무기 데미지 계산 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponDamageResult {
    /// 최종 데미지
    pub damage: i32,
    /// 기본 주사위 결과
    pub base_roll: i32,
    /// 보너스 내역
    pub bonuses: Vec<(String, i32)>,
}

/// [v2.26.0 90-2] 무기 데미지 계산
/// 원본: uhitm.c dmgval() + 보너스 합산
pub fn weapon_damage_calc(input: &WeaponDamageInput, rng: &mut NetHackRng) -> WeaponDamageResult {
    let mut bonuses = Vec::new();

    // [1] 기본 주사위
    let mut base = 0;
    for _ in 0..input.dice_num {
        base += rng.rn2(input.dice_sides) + 1;
    }

    // [2] 크리티컬 히트 → 주사위 한 번 더
    if input.critical {
        let crit_bonus = rng.rn2(input.dice_sides) + 1;
        bonuses.push(("크리티컬".to_string(), crit_bonus));
        base += crit_bonus;
    }

    let base_roll = base;

    // [3] 강화치
    if input.enchantment != 0 {
        bonuses.push(("강화".to_string(), input.enchantment));
    }

    // [4] 힘 보너스
    if input.strength_bonus > 0 {
        bonuses.push(("힘".to_string(), input.strength_bonus));
    }

    // [5] BUC 보너스
    let buc_bonus = match input.buc {
        1 => 1,   // 축복
        -1 => -1, // 저주
        _ => 0,
    };
    if buc_bonus != 0 {
        bonuses.push(("축복/저주".to_string(), buc_bonus));
    }

    // [6] 은 무기 보너스
    let silver_bonus = if input.silver_vs_vulnerable {
        rng.rn2(20) + 1
    } else {
        0
    };
    if silver_bonus > 0 {
        bonuses.push(("은".to_string(), silver_bonus));
    }

    // [7] 아티팩트 보너스
    if input.artifact_bonus > 0 {
        bonuses.push(("아티팩트".to_string(), input.artifact_bonus));
    }

    // [8] 대상 크기 보너스 (양손무기 vs 큰 적)
    let size_bonus = if input.two_handed && input.target_size >= 2 {
        rng.rn2(input.dice_sides) + 1
    } else {
        0
    };
    if size_bonus > 0 {
        bonuses.push(("대형 적".to_string(), size_bonus));
    }

    let total_bonus: i32 = bonuses.iter().map(|(_, v)| v).sum();
    let damage = (base_roll + total_bonus).max(1);

    WeaponDamageResult {
        damage,
        base_roll,
        bonuses,
    }
}

// =============================================================================
// [2] 맨손 격투 — martial_arts (uhitm.c L2200-2400)
// =============================================================================

/// [v2.26.0 90-2] 맨손 격투 기술
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MartialTechnique {
    Punch,
    Kick,
    Elbow,
    KneeStrike,
    HeadButt,
    UpperCut,
    JumpKick,
}

/// [v2.26.0 90-2] 맨손 격투 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MartialResult {
    pub technique: MartialTechnique,
    pub damage: i32,
    pub stun_chance: bool,
    pub message: String,
}

/// [v2.26.0 90-2] 맨손 격투 결과 판정
/// 원본: uhitm.c kick_monster() 내 마셜아츠 분기
pub fn martial_arts_attack(
    player_level: i32,
    skill_level: i32, // 0=미숙, 1=기본, 2=숙련, 3=전문, 4=대가
    rng: &mut NetHackRng,
) -> MartialResult {
    let tech_roll = rng.rn2(7);
    let technique = match tech_roll {
        0 => MartialTechnique::Punch,
        1 => MartialTechnique::Kick,
        2 => MartialTechnique::Elbow,
        3 => MartialTechnique::KneeStrike,
        4 => MartialTechnique::HeadButt,
        5 => MartialTechnique::UpperCut,
        _ => MartialTechnique::JumpKick,
    };

    let base_damage = match &technique {
        MartialTechnique::Punch => rng.rn2(4) + 1,
        MartialTechnique::Kick => rng.rn2(6) + 2,
        MartialTechnique::Elbow => rng.rn2(4) + 2,
        MartialTechnique::KneeStrike => rng.rn2(6) + 1,
        MartialTechnique::HeadButt => rng.rn2(4) + 3,
        MartialTechnique::UpperCut => rng.rn2(8) + 2,
        MartialTechnique::JumpKick => rng.rn2(10) + 3,
    };

    // 스킬 보너스
    let skill_bonus = skill_level * 2;
    let level_bonus = player_level / 3;
    let damage = (base_damage + skill_bonus + level_bonus).max(1);

    let stun = matches!(
        technique,
        MartialTechnique::UpperCut | MartialTechnique::HeadButt
    ) && rng.rn2(4) == 0;

    let msg = match &technique {
        MartialTechnique::Punch => "주먹을 날렸다!",
        MartialTechnique::Kick => "발차기를 날렸다!",
        MartialTechnique::Elbow => "팔꿈치 타격!",
        MartialTechnique::KneeStrike => "무릎 타격!",
        MartialTechnique::HeadButt => "머리박치기!",
        MartialTechnique::UpperCut => "어퍼컷!",
        MartialTechnique::JumpKick => "점프 킥!",
    };

    MartialResult {
        technique,
        damage,
        stun_chance: stun,
        message: msg.to_string(),
    }
}

// =============================================================================
// [3] 치명타 판정 — critical_hit_check (uhitm.c L1800-1900)
// =============================================================================

/// [v2.26.0 90-2] 치명타 판정
/// 원본: uhitm.c 원거리/근접 크리티컬 계산
pub fn critical_hit_check(
    player_dex: i32,
    player_luck: i32,
    weapon_skill: i32,
    target_sleeping: bool,
    target_paralyzed: bool,
    rng: &mut NetHackRng,
) -> bool {
    // 행동 불가 대상 → 자동 크리티컬
    if target_sleeping || target_paralyzed {
        return true;
    }

    // 기본 크리티컬 확률: 5% + DEX 보너스 + 행운 + 스킬
    let base_chance = 5;
    let dex_bonus = ((player_dex - 10).max(0)) / 2;
    let luck_bonus = player_luck.max(0);
    let skill_bonus = weapon_skill;

    let total_chance = (base_chance + dex_bonus + luck_bonus + skill_bonus).min(30);

    rng.rn2(100) < total_chance
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

    // --- weapon_damage_calc ---

    #[test]
    fn test_basic_damage() {
        let mut rng = test_rng();
        let input = WeaponDamageInput {
            dice_num: 1,
            dice_sides: 8,
            enchantment: 0,
            buc: 0,
            two_handed: false,
            critical: false,
            target_size: 1,
            silver_vs_vulnerable: false,
            artifact_bonus: 0,
            strength_bonus: 0,
        };
        let result = weapon_damage_calc(&input, &mut rng);
        assert!(result.damage >= 1 && result.damage <= 8);
    }

    #[test]
    fn test_enchanted_damage() {
        let mut rng = test_rng();
        let input = WeaponDamageInput {
            dice_num: 1,
            dice_sides: 8,
            enchantment: 5,
            buc: 0,
            two_handed: false,
            critical: false,
            target_size: 1,
            silver_vs_vulnerable: false,
            artifact_bonus: 0,
            strength_bonus: 0,
        };
        let result = weapon_damage_calc(&input, &mut rng);
        assert!(result.damage >= 6); // 최소 1 + 5
    }

    #[test]
    fn test_critical_damage() {
        let mut rng = test_rng();
        let input = WeaponDamageInput {
            dice_num: 1,
            dice_sides: 8,
            enchantment: 0,
            buc: 0,
            two_handed: false,
            critical: true,
            target_size: 1,
            silver_vs_vulnerable: false,
            artifact_bonus: 0,
            strength_bonus: 0,
        };
        let result = weapon_damage_calc(&input, &mut rng);
        assert!(result.bonuses.iter().any(|(n, _)| n == "크리티컬"));
    }

    #[test]
    fn test_silver_bonus() {
        let mut rng = test_rng();
        let input = WeaponDamageInput {
            dice_num: 1,
            dice_sides: 6,
            enchantment: 0,
            buc: 0,
            two_handed: false,
            critical: false,
            target_size: 1,
            silver_vs_vulnerable: true,
            artifact_bonus: 0,
            strength_bonus: 0,
        };
        let result = weapon_damage_calc(&input, &mut rng);
        assert!(result.bonuses.iter().any(|(n, _)| n == "은"));
    }

    // --- martial_arts ---

    #[test]
    fn test_martial_arts() {
        let mut rng = test_rng();
        let result = martial_arts_attack(10, 3, &mut rng);
        assert!(result.damage >= 1);
        assert!(!result.message.is_empty());
    }

    #[test]
    fn test_martial_skill_bonus() {
        let low = {
            let mut rng = NetHackRng::new(42);
            martial_arts_attack(10, 0, &mut rng).damage
        };
        let high = {
            let mut rng = NetHackRng::new(42);
            martial_arts_attack(10, 4, &mut rng).damage
        };
        assert!(high >= low, "높은 스킬 → 더 높은 데미지");
    }

    // --- critical_hit ---

    #[test]
    fn test_crit_sleeping_target() {
        let mut rng = test_rng();
        assert!(critical_hit_check(14, 0, 0, true, false, &mut rng));
    }

    #[test]
    fn test_crit_normal() {
        let mut crits = 0;
        for seed in 0..100u64 {
            let mut rng = NetHackRng::new(seed);
            if critical_hit_check(14, 0, 0, false, false, &mut rng) {
                crits += 1;
            }
        }
        // 기본 7% 정도 (5 + DEX 2)
        assert!(crits > 0 && crits < 30, "크리티컬 확률 적정 범위");
    }
}
