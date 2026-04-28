// eat_ext.rs — eat.c 핵심 로직 순수 결과 패턴 이식
// [v2.16.0] 신규 생성: 식사 영양/내성/시체 효과/통조림/질식 등 12개 함수
// 원본: NetHack 3.6.7 src/eat.c (3,353줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 상수
// ============================================================

/// 통조림 종류별 영양 (TTSZ)
/// 원본: eat.c tintxts[] L127-148
const TIN_NUTRITION: [i32; 15] = [
    -50, // 썩은
    50,  // 수제
    20,  // 수프
    40,  // 프렌치 프라이
    40,  // 절임
    50,  // 삶은
    50,  // 훈제
    55,  // 건조
    60,  // 딥 프라이
    70,  // 사천식
    80,  // 구운
    80,  // 볶음
    95,  // 소테
    100, // 설탕 절임
    500, // 퓌레
];

/// 시금치 통조림 영양
const SPINACH_NUTRITION: i32 = 600;

// ============================================================
// 열거형
// ============================================================

/// 통조림 종류
/// 원본: eat.c tintxts[] L127-148
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TinVariety {
    Rotten = 0,
    Homemade = 1,
    Soup = 2,
    FrenchFried = 3,
    Pickled = 4,
    Boiled = 5,
    Smoked = 6,
    Dried = 7,
    DeepFried = 8,
    Szechuan = 9,
    Broiled = 10,
    StirFried = 11,
    Sauteed = 12,
    Candied = 13,
    Pureed = 14,
}

/// 시체 식사 전 효과 (cprefx)
/// 원본: eat.c cprefx() L676-742
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpsePrefixEffect {
    /// 안전 (특별 효과 없음)
    Safe,
    /// 석화 — 돌이 되는 고기
    Petrification,
    /// 반려 동물 먹기 — 몬스터 악화
    BadPetEating,
    /// 식인 — 운 감소 + 몬스터 악화
    Cannibalism,
    /// 즉사 — 죽음의 기사 시체
    InstantDeath,
    /// 슬라임 감염
    SlimeInfection,
    /// 석화 치료 (도마뱀/산성 몬스터)
    CurePetrification,
}

/// 시체 식사 후 특수 효과 (cpostfx)
/// 원본: eat.c cpostfx() L944-1156
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpsePostEffect {
    /// 특별 효과 없음
    None,
    /// 뉴트 — 마력 회복
    NewtEnergy,
    /// 레이스 — 레벨 업
    WraithLevelUp,
    /// 수인 감염
    Lycanthropy,
    /// 간호사 — HP 완전 회복
    NurseHeal,
    /// 스토커 — 투명화
    Invisibility,
    /// 박쥐/노란 빛 — 기절
    Stun,
    /// 미믹 — 행동 불능 (변장)
    MimicPretend,
    /// 양자역학자 — 속도 토글
    QuantumSpeed,
    /// 도마뱀 — 기절/혼란 감소
    LizardCure,
    /// 카멜레온/도플갱어 — 변신
    Polymorph,
    /// 마법 해제자 — 내성 손실
    DisenchantStrip,
    /// 마인드 플레이어 — 지능 증가
    MindFlayerInt,
    /// 일반 내성 확인
    CheckIntrinsics,
}

/// 내성 유형
/// 원본: eat.c intrinsic_possible()/givit() L766-941
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicType {
    FireResistance,
    SleepResistance,
    ColdResistance,
    DisintegrationResistance,
    ShockResistance,
    PoisonResistance,
    Teleport,
    TeleportControl,
    Telepathy,
    Strength, // 거인에서 획득
}

/// 질식 결과
/// 원본: eat.c choke() L237-282
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChokeResult {
    /// 구토로 회복
    Vomit,
    /// 질식사
    Choke,
    /// 호흡 불필요로 회피
    Breathless,
}

/// 영양 수준
/// 원본: hack.h 허기 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HungerState {
    Satiated = 0,
    NotHungry = 1,
    Hungry = 2,
    Weak = 3,
    Fainting = 4,
    Fainted = 5,
    Starved = 6,
}

// ============================================================
// 1. tin_nutrition — 통조림 영양 계산
// ============================================================

/// 통조림 종류별 영양 반환
/// 원본: eat.c tintxts[] L127-148
pub fn tin_nutrition(variety: TinVariety) -> i32 {
    TIN_NUTRITION[variety as usize]
}

/// 시금치 통조림 영양
pub fn spinach_nutrition() -> i32 {
    SPINACH_NUTRITION
}

// ============================================================
// 2. corpse_nutrition — 시체 영양 계산 (종족 보정 포함)
// ============================================================

/// 시체/음식 영양 계산 (종족 보정 적용)
/// 원본: eat.c obj_nutrition() L315-338
pub fn corpse_nutrition(
    base_nutrition: u32,
    is_lembas: bool,
    is_cram: bool,
    is_elf: bool,
    is_orc: bool,
    is_dwarf: bool,
) -> u32 {
    let mut nut = base_nutrition;
    if is_lembas {
        if is_elf {
            nut += nut / 4; // 800 → 1000
        } else if is_orc {
            nut -= nut / 4; // 800 → 600
        }
    } else if is_cram && is_dwarf {
        nut += nut / 6; // 600 → 700
    }
    nut
}

// ============================================================
// 3. choke_check — 질식 판정
// ============================================================

/// 과식 시 질식 판정
/// 원본: eat.c choke() L237-282
pub fn choke_check(is_breathless: bool, is_strangled: bool, rng: &mut NetHackRng) -> ChokeResult {
    if is_breathless || (!is_strangled && rng.rn2(20) == 0) {
        ChokeResult::Vomit
    } else {
        ChokeResult::Choke
    }
}

// ============================================================
// 4. intrinsic_chance — 내성 획득 확률 계산
// ============================================================

/// 시체 섭취 시 내성 부여 확률 판정
/// 원본: eat.c givit() L828-941
/// monster_level > rn2(chance) 이면 성공
pub fn intrinsic_chance(
    intrinsic: IntrinsicType,
    monster_level: i32,
    is_killer_bee_or_scorpion: bool,
    rng: &mut NetHackRng,
) -> bool {
    let chance = match intrinsic {
        IntrinsicType::PoisonResistance => {
            if is_killer_bee_or_scorpion && rng.rn2(4) == 0 {
                1 // 독충에서 쉽게 획득
            } else {
                15
            }
        }
        IntrinsicType::Teleport => 10,
        IntrinsicType::TeleportControl => 12,
        IntrinsicType::Telepathy => 1,
        IntrinsicType::Strength => 15,
        _ => 15,
    };
    monster_level > rng.rn2(chance)
}

// ============================================================
// 5. intrinsic_pick — 내성 선택 (다중 후보 중)
// ============================================================

/// 여러 내성 후보 중 하나를 무작위 선택 (저수지 샘플링)
/// 원본: eat.c cpostfx() L1121-1148
pub fn intrinsic_pick(candidates: &[IntrinsicType], rng: &mut NetHackRng) -> Option<IntrinsicType> {
    if candidates.is_empty() {
        return None;
    }
    let mut chosen = candidates[0];
    let mut count = 1;
    for &candidate in &candidates[1..] {
        count += 1;
        if rng.rn2(count) == 0 {
            chosen = candidate;
        }
    }
    // 힘만 후보이면 50% 확률로 무효
    if candidates.len() == 1 && candidates[0] == IntrinsicType::Strength {
        if rng.rn2(2) != 0 {
            return None;
        }
    }
    Some(chosen)
}

// ============================================================
// 6. newt_energy — 뉴트 마력 회복량
// ============================================================

/// 뉴트 시체 섭취 시 마력 증가량 결정
/// 원본: eat.c cpostfx() L958-974
pub fn newt_energy(current_en: i32, max_en: i32, rng: &mut NetHackRng) -> (i32, bool) {
    // 2/3 확률 또는 마력이 낮으면 항상 발동
    if rng.rn2(3) != 0 || 3 * current_en <= 2 * max_en {
        let boost = rng.rnd(3);
        let mut new_en = current_en + boost;
        let mut max_increased = false;
        if new_en > max_en {
            if rng.rn2(3) == 0 {
                max_increased = true;
            }
            new_en = max_en + if max_increased { 1 } else { 0 };
        }
        (
            new_en.min(max_en + if max_increased { 1 } else { 0 }),
            max_increased,
        )
    } else {
        (current_en, false) // 효과 없음
    }
}

// ============================================================
// 7. mimic_delay — 미믹 변장 지속 턴
// ============================================================

/// 미믹 시체 섭취 시 행동 불능 턴
/// 원본: eat.c cpostfx() L1016-1050
pub fn mimic_delay(is_giant: bool, is_large: bool) -> i32 {
    let mut tmp = 20; // small
    if is_large {
        tmp += 20;
    }
    if is_giant {
        tmp += 10;
    }
    tmp
}

// ============================================================
// 8. quantum_speed — 양자역학자 속도 토글
// ============================================================

/// 양자역학자 시체 섭취 시 속도 효과
/// 원본: eat.c cpostfx() L1052-1061
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantumEffect {
    Slower, // 기존 빠름 제거
    Faster, // 빠름 부여
}

pub fn quantum_speed(has_intrinsic_fast: bool) -> QuantumEffect {
    if has_intrinsic_fast {
        QuantumEffect::Slower
    } else {
        QuantumEffect::Faster
    }
}

// ============================================================
// 9. cannibal_penalty — 식인 패널티
// ============================================================

/// 식인 시 운 감소량
/// 원본: eat.c maybe_cannibal() L670 — -rn1(4,2) → -5~-2
pub fn cannibal_luck_penalty(rng: &mut NetHackRng) -> i32 {
    -(rng.rn1(4, 2)) // -2 ~ -5
}

// ============================================================
// 10. hallucination_from_eating — 환각 유발시 시간
// ============================================================

/// 스턴/환각 유발 시체 섭취 시 환각 추가 시간
/// 원본: eat.c cpostfx() L1107-1111
pub fn hallucination_extension() -> i64 {
    200
}

// ============================================================
// 11. stun_from_bat — 박쥐 기절 추가 시간
// ============================================================

/// 박쥐 시체 섭취 시 기절 추가 턴
/// 원본: eat.c cpostfx() L1011-1014
pub fn bat_stun_duration() -> i64 {
    30
}

/// Yellow light / stalker도 동일한 30턴 기절 추가
pub fn stalker_stun_duration() -> i64 {
    30
}

// ============================================================
// 12. hunger_state_from_nutrition — 영양 기반 허기 상태
// ============================================================

/// 영양 수치에서 허기 상태 결정
/// 원본: hack.h/eat.c 기준값
pub fn hunger_state_from_nutrition(nutrition: i32) -> HungerState {
    if nutrition > 1000 {
        HungerState::Satiated
    } else if nutrition > 150 {
        HungerState::NotHungry
    } else if nutrition > 50 {
        HungerState::Hungry
    } else if nutrition > 0 {
        HungerState::Weak
    } else {
        HungerState::Fainting
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

    // --- tin_nutrition ---
    #[test]
    fn test_tin_rotten() {
        assert_eq!(tin_nutrition(TinVariety::Rotten), -50);
    }

    #[test]
    fn test_tin_pureed() {
        assert_eq!(tin_nutrition(TinVariety::Pureed), 500);
    }

    #[test]
    fn test_spinach() {
        assert_eq!(spinach_nutrition(), 600);
    }

    // --- corpse_nutrition ---
    #[test]
    fn test_lembas_elf() {
        assert_eq!(corpse_nutrition(800, true, false, true, false, false), 1000);
    }

    #[test]
    fn test_lembas_orc() {
        assert_eq!(corpse_nutrition(800, true, false, false, true, false), 600);
    }

    #[test]
    fn test_cram_dwarf() {
        assert_eq!(corpse_nutrition(600, false, true, false, false, true), 700);
    }

    #[test]
    fn test_normal_food() {
        assert_eq!(
            corpse_nutrition(400, false, false, false, false, false),
            400
        );
    }

    // --- choke ---
    #[test]
    fn test_choke_breathless() {
        let mut rng = test_rng();
        assert_eq!(choke_check(true, false, &mut rng), ChokeResult::Vomit);
    }

    #[test]
    fn test_choke_probability() {
        let mut rng = test_rng();
        let mut vomits = 0;
        for _ in 0..200 {
            if choke_check(false, false, &mut rng) == ChokeResult::Vomit {
                vomits += 1;
            }
        }
        // ~5% 확률 → 약 10개
        assert!(vomits > 2 && vomits < 30, "구토 횟수: {}", vomits);
    }

    // --- intrinsic_chance ---
    #[test]
    fn test_intrinsic_high_level() {
        let mut rng = test_rng();
        let mut success = 0;
        for _ in 0..100 {
            if intrinsic_chance(IntrinsicType::FireResistance, 20, false, &mut rng) {
                success += 1;
            }
        }
        // 레벨20 > rn2(15) → 높은 확률
        assert!(success > 80, "성공: {}", success);
    }

    #[test]
    fn test_intrinsic_low_level() {
        let mut rng = test_rng();
        let mut success = 0;
        for _ in 0..100 {
            if intrinsic_chance(IntrinsicType::FireResistance, 2, false, &mut rng) {
                success += 1;
            }
        }
        // 레벨2 > rn2(15) → 매우 낮은 확률
        assert!(success < 30, "성공: {}", success);
    }

    #[test]
    fn test_telepathy_easy() {
        let mut rng = test_rng();
        // chance=1이므로 rn2(1)=0 → level>0 항상 성공
        assert!(intrinsic_chance(
            IntrinsicType::Telepathy,
            1,
            false,
            &mut rng
        ));
    }

    // --- intrinsic_pick ---
    #[test]
    fn test_pick_empty() {
        let mut rng = test_rng();
        assert_eq!(intrinsic_pick(&[], &mut rng), None);
    }

    #[test]
    fn test_pick_single_non_str() {
        let mut rng = test_rng();
        let candidates = [IntrinsicType::FireResistance];
        assert_eq!(
            intrinsic_pick(&candidates, &mut rng),
            Some(IntrinsicType::FireResistance)
        );
    }

    #[test]
    fn test_pick_multiple() {
        let mut rng = test_rng();
        let candidates = [
            IntrinsicType::FireResistance,
            IntrinsicType::ColdResistance,
            IntrinsicType::PoisonResistance,
        ];
        let result = intrinsic_pick(&candidates, &mut rng);
        assert!(result.is_some(), "후보 존재 시 항상 선택");
    }

    // --- newt_energy ---
    #[test]
    fn test_newt_energy_boost() {
        let mut rng = test_rng();
        let mut any_boost = false;
        for _ in 0..50 {
            let (new_en, _) = newt_energy(5, 20, &mut rng);
            if new_en > 5 {
                any_boost = true;
            }
        }
        assert!(any_boost, "뉴트 마력 증가 발생해야 함");
    }

    #[test]
    fn test_newt_low_energy() {
        let mut rng = test_rng();
        // 마력이 낮으면 항상 발동 (3*1 <= 2*20)
        let (new_en, _) = newt_energy(1, 20, &mut rng);
        assert!(new_en >= 2, "낮은 마력: {}", new_en);
    }

    // --- mimic_delay ---
    #[test]
    fn test_mimic_small() {
        assert_eq!(mimic_delay(false, false), 20);
    }

    #[test]
    fn test_mimic_giant() {
        assert_eq!(mimic_delay(true, true), 50);
    }

    // --- quantum_speed ---
    #[test]
    fn test_quantum_faster() {
        assert_eq!(quantum_speed(false), QuantumEffect::Faster);
    }

    #[test]
    fn test_quantum_slower() {
        assert_eq!(quantum_speed(true), QuantumEffect::Slower);
    }

    // --- cannibal_penalty ---
    #[test]
    fn test_cannibal() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let penalty = cannibal_luck_penalty(&mut rng);
            assert!(penalty >= -5 && penalty <= -2, "패널티: {}", penalty);
        }
    }

    // --- constants ---
    #[test]
    fn test_hallucination_ext() {
        assert_eq!(hallucination_extension(), 200);
    }

    #[test]
    fn test_bat_stun() {
        assert_eq!(bat_stun_duration(), 30);
    }

    // --- hunger_state ---
    #[test]
    fn test_hunger_satiated() {
        assert_eq!(hunger_state_from_nutrition(1500), HungerState::Satiated);
    }

    #[test]
    fn test_hunger_not_hungry() {
        assert_eq!(hunger_state_from_nutrition(500), HungerState::NotHungry);
    }

    #[test]
    fn test_hunger_hungry() {
        assert_eq!(hunger_state_from_nutrition(100), HungerState::Hungry);
    }

    #[test]
    fn test_hunger_weak() {
        assert_eq!(hunger_state_from_nutrition(30), HungerState::Weak);
    }

    #[test]
    fn test_hunger_fainting() {
        assert_eq!(hunger_state_from_nutrition(0), HungerState::Fainting);
    }
}
