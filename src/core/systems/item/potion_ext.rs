// potion_ext.rs — potion.c 핵심 로직 순수 결과 패턴 이식
// [v2.12.0] 신규 생성: 물약 효과/혼합/충전/회복 등 12개 함수
// 원본: NetHack 3.6.7 src/potion.c (2,413줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수 정의
// ============================================================

/// 내재 능력 타임아웃 최대값 (원본 TIMEOUT 매크로)
const TIMEOUT_MAX: i64 = 100_000_000;

/// 물약 혼합 실패 시 반환값
pub const STRANGE_OBJECT: i32 = -1;

// ============================================================
// 물약 타입 열거형 (혼합 판정에 필요)
// ============================================================

/// 물약 종류 — 혼합 계산에 사용
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionType {
    Healing,
    ExtraHealing,
    FullHealing,
    Speed,
    GainLevel,
    GainEnergy,
    GainAbility,
    Sickness,
    Hallucination,
    Blindness,
    Confusion,
    FruitJuice,
    SeeInvisible,
    Booze,
    Enlightenment,
    Levitation,
    Water,
    Acid,
    Oil,
    Polymorph,
    Paralysis,
    Sleeping,
    Invisibility,
    RestoreAbility,
    MonsterDetection,
    ObjectDetection,
    Other(i32),
}

/// 담금(dip) 대상 아이템 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DipItemType {
    UnicornHorn,
    Amethyst,
    Poisonable,
    Towel,
    OilLamp,
    MagicLamp,
    LichenCorpse,
    Other,
}

/// 축복/저주 상태 (BCS = Blessed/Cursed/Status)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BcsStatus {
    Blessed,
    Uncursed,
    Cursed,
}

impl BcsStatus {
    /// bcsign 유틸리티: blessed=+1, uncursed=0, cursed=-1
    pub fn sign(&self) -> i32 {
        match self {
            BcsStatus::Blessed => 1,
            BcsStatus::Uncursed => 0,
            BcsStatus::Cursed => -1,
        }
    }
}

// ============================================================
// 1. itimeout — 내재 타임아웃 값 클램프
// ============================================================

/// 내재 능력 타임아웃 값을 유효 범위로 제한
/// 원본: potion.c itimeout()
pub fn itimeout(val: i64) -> i64 {
    if val >= TIMEOUT_MAX {
        TIMEOUT_MAX
    } else if val < 1 {
        0
    } else {
        val
    }
}

/// 기존 타임아웃에 증분을 더한 후 유효 범위로 제한
/// 원본: potion.c itimeout_incr()
pub fn itimeout_incr(old: i64, incr: i32) -> i64 {
    itimeout(old + incr as i64)
}

// ============================================================
// 2. healup_calc — 회복량 계산 (순수 결과만, 부작용 없음)
// ============================================================

/// 회복 결과 구조체
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealupResult {
    /// 적용할 HP 회복량
    pub hp_heal: i32,
    /// 최대 HP 증가량 (nxtra)
    pub max_hp_bonus: i32,
    /// 질병 치료 여부
    pub cure_sick: bool,
    /// 실명/청각장애 치료 여부
    pub cure_blind: bool,
}

/// 물약/주문 별 회복량 계산
/// 원본: potion.c healup() 호출부의 인자값 결정 로직
pub fn healup_calc(potion: PotionType, bcs: BcsStatus, rng: &mut NetHackRng) -> HealupResult {
    match potion {
        PotionType::Healing => {
            // d(4 + 2*bcsign, 4)
            let dice = (4 + 2 * bcs.sign()).max(1);
            let hp_heal = rng.d(dice, 4);
            let max_hp_bonus = if bcs == BcsStatus::Blessed { 1 } else { 0 };
            HealupResult {
                hp_heal,
                max_hp_bonus,
                cure_sick: false,
                cure_blind: false,
            }
        }
        PotionType::ExtraHealing => {
            // d(6 + 2*bcsign, 8)
            let dice = (6 + 2 * bcs.sign()).max(1);
            let hp_heal = rng.d(dice, 8);
            let max_hp_bonus = match bcs {
                BcsStatus::Blessed => 5,
                BcsStatus::Cursed => 0,
                BcsStatus::Uncursed => 2,
            };
            HealupResult {
                hp_heal,
                max_hp_bonus,
                cure_sick: bcs != BcsStatus::Cursed,
                cure_blind: true,
            }
        }
        PotionType::FullHealing => HealupResult {
            hp_heal: 400,
            max_hp_bonus: (4 + 4 * bcs.sign()).max(0),
            cure_sick: bcs != BcsStatus::Cursed,
            cure_blind: true,
        },
        _ => HealupResult {
            hp_heal: 0,
            max_hp_bonus: 0,
            cure_sick: false,
            cure_blind: false,
        },
    }
}

// ============================================================
// 3. potion_nutrition — 물약 음용 시 영양 계산
// ============================================================

/// 물약 음용 시 영양 증가량 계산
/// 원본: peffects() 내 u.uhunger += 계산 로직
pub fn potion_nutrition(
    potion: PotionType,
    bcs: BcsStatus,
    diluted: bool,
    rng: &mut NetHackRng,
) -> i32 {
    match potion {
        PotionType::Water => {
            if bcs == BcsStatus::Uncursed {
                rng.rnd(10)
            } else {
                0
            }
        }
        PotionType::Booze => 10 * (2 + bcs.sign()),
        PotionType::FruitJuice => {
            let base = if diluted { 5 } else { 10 };
            base * (2 + bcs.sign())
        }
        _ => 0,
    }
}

// ============================================================
// 4. gain_energy_calc — 마법 에너지 물약 계산
// ============================================================

/// 에너지 물약 효과 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GainEnergyResult {
    /// 최대 에너지 변화량 (음수일 수 있음)
    pub max_energy_delta: i32,
    /// 현재 에너지 변화량
    pub cur_energy_delta: i32,
}

/// 에너지 물약의 최대/현재 에너지 변화량 계산
/// 원본: peffects() POT_GAIN_ENERGY 분기
pub fn gain_energy_calc(bcs: BcsStatus, rng: &mut NetHackRng) -> GainEnergyResult {
    let dice_count = match bcs {
        BcsStatus::Blessed => 3,
        BcsStatus::Uncursed => 2,
        BcsStatus::Cursed => 1,
    };
    let mut num = rng.d(dice_count, 6);
    if bcs == BcsStatus::Cursed {
        num = -num;
    }
    GainEnergyResult {
        max_energy_delta: num,
        cur_energy_delta: 3 * num,
    }
}

// ============================================================
// 5. acid_damage_calc — 산성 물약 데미지 계산
// ============================================================

/// 산성 물약 데미지 계산
/// 원본: peffects() POT_ACID 분기 dmg = d(cursed?2:1, blessed?4:8)
pub fn acid_damage_calc(bcs: BcsStatus, has_acid_resistance: bool, rng: &mut NetHackRng) -> i32 {
    if has_acid_resistance {
        return 0;
    }
    let dice = if bcs == BcsStatus::Cursed { 2 } else { 1 };
    let sides = if bcs == BcsStatus::Blessed { 4 } else { 8 };
    rng.d(dice, sides)
}

// ============================================================
// 6. mixtype — 물약 혼합 결과 결정 (핵심 연금술 테이블)
// ============================================================

/// 두 물약/아이템을 혼합했을 때의 결과 물약 타입 반환
/// 원본: potion.c mixtype()
/// 반환값: Some(결과 물약) 또는 None (STRANGE_OBJECT)
pub fn mixtype(o1: PotionType, o2: PotionType, rng: &mut NetHackRng) -> Option<PotionType> {
    match o1 {
        PotionType::Healing => match o2 {
            PotionType::Speed => Some(PotionType::ExtraHealing),
            PotionType::GainLevel | PotionType::GainEnergy => Some(PotionType::ExtraHealing),
            PotionType::Sickness => Some(PotionType::FruitJuice),
            PotionType::Hallucination | PotionType::Blindness | PotionType::Confusion => {
                Some(PotionType::Water)
            }
            _ => None,
        },
        PotionType::ExtraHealing => match o2 {
            PotionType::GainLevel | PotionType::GainEnergy => Some(PotionType::FullHealing),
            PotionType::Sickness => Some(PotionType::FruitJuice),
            PotionType::Hallucination | PotionType::Blindness | PotionType::Confusion => {
                Some(PotionType::Water)
            }
            _ => None,
        },
        PotionType::FullHealing => match o2 {
            PotionType::GainLevel | PotionType::GainEnergy => Some(PotionType::GainAbility),
            PotionType::Sickness => Some(PotionType::FruitJuice),
            PotionType::Hallucination | PotionType::Blindness | PotionType::Confusion => {
                Some(PotionType::Water)
            }
            _ => None,
        },
        PotionType::GainLevel | PotionType::GainEnergy => match o2 {
            PotionType::Confusion => {
                if rng.rn2(3) != 0 {
                    Some(PotionType::Booze)
                } else {
                    Some(PotionType::Enlightenment)
                }
            }
            PotionType::Healing => Some(PotionType::ExtraHealing),
            PotionType::ExtraHealing => Some(PotionType::FullHealing),
            PotionType::FullHealing => Some(PotionType::GainAbility),
            PotionType::FruitJuice => Some(PotionType::SeeInvisible),
            PotionType::Booze => Some(PotionType::Hallucination),
            _ => None,
        },
        PotionType::FruitJuice => match o2 {
            PotionType::Sickness => Some(PotionType::Sickness),
            PotionType::Enlightenment | PotionType::Speed => Some(PotionType::Booze),
            PotionType::GainLevel | PotionType::GainEnergy => Some(PotionType::SeeInvisible),
            _ => None,
        },
        PotionType::Enlightenment => match o2 {
            PotionType::Levitation => {
                if rng.rn2(3) != 0 {
                    Some(PotionType::GainLevel)
                } else {
                    None
                }
            }
            PotionType::FruitJuice => Some(PotionType::Booze),
            PotionType::Booze => Some(PotionType::Confusion),
            _ => None,
        },
        _ => None,
    }
}

// ============================================================
// 7. djinni_chance — 진 병 열기 확률 결정
// ============================================================

/// 진(djinni) 등장 시 결과 유형
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DjinniResult {
    GrantWish,
    Tame,
    Peaceful,
    Vanish,
    Hostile,
}

/// 진 병 개봉 시 결과 결정
/// 원본: potion.c djinni_from_bottle()
/// blessed=80%소원, cursed=80%적대
pub fn djinni_chance(bcs: BcsStatus, rng: &mut NetHackRng) -> DjinniResult {
    let mut chance = rng.rn2(5);
    if bcs == BcsStatus::Blessed {
        chance = if chance == 4 { rng.rnd(4) } else { 0 };
    } else if bcs == BcsStatus::Cursed {
        chance = if chance == 0 { rng.rn2(4) } else { 4 };
    }
    match chance {
        0 => DjinniResult::GrantWish,
        1 => DjinniResult::Tame,
        2 => DjinniResult::Peaceful,
        3 => DjinniResult::Vanish,
        _ => DjinniResult::Hostile,
    }
}

// ============================================================
// 8. bottlename — 무작위 병 이름
// ============================================================

const BOTTLE_NAMES: &[&str] = &[
    "bottle", "phial", "flagon", "carafe", "flask", "jar", "vial",
];

pub fn bottlename(rng: &mut NetHackRng) -> &'static str {
    BOTTLE_NAMES[rng.rn2(BOTTLE_NAMES.len() as i32) as usize]
}

// ============================================================
// 9. oil_lamp_fill — 기름 등잔 충전 계산
// ============================================================

/// 기름 등잔에 기름 물약을 부었을 때의 수명 변화 계산
/// 반환값: (새 수명, 이미 가득 참 여부)
pub fn oil_lamp_fill(current_age: i64, potion_age: i64, diluted: bool) -> (i64, bool) {
    if current_age > 1000 {
        return (current_age, true);
    }
    let multiplier = if diluted { 3 } else { 4 };
    let new_age = current_age + multiplier * potion_age / 2;
    let clamped = new_age.min(1500);
    (clamped, false)
}

// ============================================================
// 10. h2o_dip_result — 성수/저주수 담금 결과
// ============================================================

/// 성수/저주수 담금 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H2oDipResult {
    NoChange,
    Uncurse,
    Bless,
    Unbless,
    Curse,
    WaterDamage,
}

/// 물 물약으로 아이템을 담갔을 때의 결과 결정
/// 원본: potion.c H2Opotion_dip()
pub fn h2o_dip_result(
    water_bcs: BcsStatus,
    target_blessed: bool,
    target_cursed: bool,
) -> H2oDipResult {
    match water_bcs {
        BcsStatus::Blessed => {
            if target_cursed {
                H2oDipResult::Uncurse
            } else if !target_blessed {
                H2oDipResult::Bless
            } else {
                H2oDipResult::NoChange
            }
        }
        BcsStatus::Cursed => {
            if target_blessed {
                H2oDipResult::Unbless
            } else if !target_cursed {
                H2oDipResult::Curse
            } else {
                H2oDipResult::NoChange
            }
        }
        BcsStatus::Uncursed => H2oDipResult::WaterDamage,
    }
}

// ============================================================
// 11. levitation_head_damage — 저주 부양 물약 두부 데미지
// ============================================================

/// 저주 부양 물약 천장 충돌 데미지 계산
pub fn levitation_head_damage(has_helmet: bool, is_metallic: bool, rng: &mut NetHackRng) -> i32 {
    let sides = if !has_helmet {
        10
    } else if !is_metallic {
        6
    } else {
        3
    };
    rng.rnd(sides)
}

// ============================================================
// 12. potion_explode_damage — 물약 폭발(담금 사고) 데미지
// ============================================================

/// 물약 혼합 폭발 데미지 계산
pub fn potion_explode_damage(quantity: i32, rng: &mut NetHackRng) -> i32 {
    quantity + rng.rnd(9)
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

    // --- itimeout ---
    #[test]
    fn test_itimeout_clamp_high() {
        assert_eq!(itimeout(200_000_000), TIMEOUT_MAX);
    }

    #[test]
    fn test_itimeout_clamp_low() {
        assert_eq!(itimeout(-5), 0);
        assert_eq!(itimeout(0), 0);
    }

    #[test]
    fn test_itimeout_normal() {
        assert_eq!(itimeout(500), 500);
    }

    #[test]
    fn test_itimeout_incr() {
        assert_eq!(itimeout_incr(100, 50), 150);
        assert_eq!(itimeout_incr(TIMEOUT_MAX - 10, 20), TIMEOUT_MAX);
    }

    // --- healup_calc ---
    #[test]
    fn test_healup_healing_blessed() {
        let mut rng = test_rng();
        let result = healup_calc(PotionType::Healing, BcsStatus::Blessed, &mut rng);
        assert!(result.hp_heal >= 6 && result.hp_heal <= 30);
        assert_eq!(result.max_hp_bonus, 1);
        assert!(!result.cure_sick);
        assert!(!result.cure_blind);
    }

    #[test]
    fn test_healup_full_healing() {
        let mut rng = test_rng();
        let result = healup_calc(PotionType::FullHealing, BcsStatus::Blessed, &mut rng);
        assert_eq!(result.hp_heal, 400);
        assert_eq!(result.max_hp_bonus, 8);
        assert!(result.cure_sick);
        assert!(result.cure_blind);
    }

    #[test]
    fn test_healup_extra_healing_cursed() {
        let mut rng = test_rng();
        let result = healup_calc(PotionType::ExtraHealing, BcsStatus::Cursed, &mut rng);
        assert!(result.hp_heal >= 4 && result.hp_heal <= 40);
        assert_eq!(result.max_hp_bonus, 0);
        assert!(!result.cure_sick);
        assert!(result.cure_blind);
    }

    // --- potion_nutrition ---
    #[test]
    fn test_nutrition_water() {
        let mut rng = test_rng();
        let n = potion_nutrition(PotionType::Water, BcsStatus::Uncursed, false, &mut rng);
        assert!(n >= 1 && n <= 10);
    }

    #[test]
    fn test_nutrition_booze_blessed() {
        let mut rng = test_rng();
        let n = potion_nutrition(PotionType::Booze, BcsStatus::Blessed, false, &mut rng);
        assert_eq!(n, 30);
    }

    #[test]
    fn test_nutrition_fruit_juice_cursed() {
        let mut rng = test_rng();
        let n = potion_nutrition(PotionType::FruitJuice, BcsStatus::Cursed, false, &mut rng);
        assert_eq!(n, 10);
    }

    #[test]
    fn test_nutrition_fruit_juice_diluted() {
        let mut rng = test_rng();
        let n = potion_nutrition(PotionType::FruitJuice, BcsStatus::Uncursed, true, &mut rng);
        assert_eq!(n, 10);
    }

    // --- gain_energy_calc ---
    #[test]
    fn test_gain_energy_blessed() {
        let mut rng = test_rng();
        let result = gain_energy_calc(BcsStatus::Blessed, &mut rng);
        assert!(result.max_energy_delta >= 3 && result.max_energy_delta <= 21);
        assert_eq!(result.cur_energy_delta, 3 * result.max_energy_delta);
    }

    #[test]
    fn test_gain_energy_cursed() {
        let mut rng = test_rng();
        let result = gain_energy_calc(BcsStatus::Cursed, &mut rng);
        assert!(result.max_energy_delta <= -1 && result.max_energy_delta >= -7);
        assert_eq!(result.cur_energy_delta, 3 * result.max_energy_delta);
    }

    // --- acid_damage_calc ---
    #[test]
    fn test_acid_with_resistance() {
        let mut rng = test_rng();
        let dmg = acid_damage_calc(BcsStatus::Uncursed, true, &mut rng);
        assert_eq!(dmg, 0);
    }

    #[test]
    fn test_acid_cursed() {
        let mut rng = test_rng();
        let dmg = acid_damage_calc(BcsStatus::Cursed, false, &mut rng);
        assert!(dmg >= 2 && dmg <= 18);
    }

    #[test]
    fn test_acid_blessed() {
        let mut rng = test_rng();
        let dmg = acid_damage_calc(BcsStatus::Blessed, false, &mut rng);
        assert!(dmg >= 1 && dmg <= 5);
    }

    // --- mixtype ---
    #[test]
    fn test_mix_healing_speed() {
        let mut rng = test_rng();
        assert_eq!(
            mixtype(PotionType::Healing, PotionType::Speed, &mut rng),
            Some(PotionType::ExtraHealing)
        );
    }

    #[test]
    fn test_mix_healing_sickness() {
        let mut rng = test_rng();
        assert_eq!(
            mixtype(PotionType::Healing, PotionType::Sickness, &mut rng),
            Some(PotionType::FruitJuice)
        );
    }

    #[test]
    fn test_mix_healing_hallucination() {
        let mut rng = test_rng();
        assert_eq!(
            mixtype(PotionType::Healing, PotionType::Hallucination, &mut rng),
            Some(PotionType::Water)
        );
    }

    #[test]
    fn test_mix_full_healing_gain_level() {
        let mut rng = test_rng();
        assert_eq!(
            mixtype(PotionType::FullHealing, PotionType::GainLevel, &mut rng),
            Some(PotionType::GainAbility)
        );
    }

    #[test]
    fn test_mix_no_result() {
        let mut rng = test_rng();
        assert_eq!(mixtype(PotionType::Water, PotionType::Acid, &mut rng), None);
    }

    #[test]
    fn test_mix_fruit_juice_enlightenment() {
        let mut rng = test_rng();
        assert_eq!(
            mixtype(PotionType::FruitJuice, PotionType::Enlightenment, &mut rng),
            Some(PotionType::Booze)
        );
    }

    // --- djinni_chance ---
    #[test]
    fn test_djinni_blessed_mostly_wish() {
        let mut rng = test_rng();
        let mut wish_count = 0;
        for _ in 0..100 {
            if djinni_chance(BcsStatus::Blessed, &mut rng) == DjinniResult::GrantWish {
                wish_count += 1;
            }
        }
        assert!(wish_count >= 60, "소원 {}번, 80% 기대", wish_count);
    }

    #[test]
    fn test_djinni_cursed_mostly_hostile() {
        let mut rng = test_rng();
        let mut hostile_count = 0;
        for _ in 0..100 {
            if djinni_chance(BcsStatus::Cursed, &mut rng) == DjinniResult::Hostile {
                hostile_count += 1;
            }
        }
        assert!(hostile_count >= 60, "적대 {}번, 80% 기대", hostile_count);
    }

    // --- bottlename ---
    #[test]
    fn test_bottlename() {
        let mut rng = test_rng();
        let name = bottlename(&mut rng);
        assert!(BOTTLE_NAMES.contains(&name));
    }

    // --- oil_lamp_fill ---
    #[test]
    fn test_oil_lamp_fill_normal() {
        let (age, full) = oil_lamp_fill(500, 200, false);
        assert_eq!(age, 900);
        assert!(!full);
    }

    #[test]
    fn test_oil_lamp_fill_already_full() {
        let (age, full) = oil_lamp_fill(1200, 200, false);
        assert_eq!(age, 1200);
        assert!(full);
    }

    #[test]
    fn test_oil_lamp_fill_cap() {
        let (age, full) = oil_lamp_fill(1000, 500, false);
        assert_eq!(age, 1500);
        assert!(!full);
    }

    // --- h2o_dip_result ---
    #[test]
    fn test_h2o_bless_cursed_item() {
        assert_eq!(
            h2o_dip_result(BcsStatus::Blessed, false, true),
            H2oDipResult::Uncurse
        );
    }

    #[test]
    fn test_h2o_bless_normal_item() {
        assert_eq!(
            h2o_dip_result(BcsStatus::Blessed, false, false),
            H2oDipResult::Bless
        );
    }

    #[test]
    fn test_h2o_curse_blessed_item() {
        assert_eq!(
            h2o_dip_result(BcsStatus::Cursed, true, false),
            H2oDipResult::Unbless
        );
    }

    #[test]
    fn test_h2o_uncursed_water() {
        assert_eq!(
            h2o_dip_result(BcsStatus::Uncursed, false, false),
            H2oDipResult::WaterDamage
        );
    }

    // --- levitation_head_damage ---
    #[test]
    fn test_lev_damage_no_helmet() {
        let mut rng = test_rng();
        let dmg = levitation_head_damage(false, false, &mut rng);
        assert!(dmg >= 1 && dmg <= 10);
    }

    #[test]
    fn test_lev_damage_metallic_helmet() {
        let mut rng = test_rng();
        let dmg = levitation_head_damage(true, true, &mut rng);
        assert!(dmg >= 1 && dmg <= 3);
    }
}
