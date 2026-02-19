// fountain_ext.rs — fountain.c + sit.c 핵심 로직 순수 결과 패턴 이식
// [v2.14.0] 신규 생성: 분수/싱크대/왕좌 효과 결정 14개 함수
// 원본: NetHack 3.6.7 src/fountain.c (629줄) + src/sit.c (492줄)

use crate::util::rng::NetHackRng;

// ============================================================
// 열거형
// ============================================================

/// 분수 음용 결과 (rnd(30) 기반)
/// 원본: fountain.c drinkfountain() L222-358
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrinkFountainResult {
    /// 마법 분수 + 행운 ≥ 0 + fate ≥ 10 → 능력치 회복/증가
    MagicBless,
    /// fate < 10 → 시원한 물 (배고픔 +rnd(10))
    CoolDraught { hunger_gain: i32 },
    /// 19: 자기 인식
    SelfKnowledge,
    /// 20: 구역질 (배고픔 +rn1(20,11))
    Foul { hunger_loss: i32 },
    /// 21: 오염 (독 저항 여부에 따라 데미지/STR 손실)
    Poison {
        has_resistance: bool,
        damage: i32,
        str_loss: i32,
    },
    /// 22: 뱀 떼 소환 (rn1(5,2)마리)
    WaterSnakes { count: i32 },
    /// 23: 물 악마 소환 (소원 가능)
    WaterDemon { grants_wish: bool },
    /// 24: 아이템 저주 (1/5 확률)
    CurseItems { hunger_loss: i32 },
    /// 25: 투명 보기
    SeeInvisible,
    /// 26: 몬스터 탐지
    SeeMonsters,
    /// 27: 보석 발견 (약탈 안 됨)
    FindGem,
    /// 28: 물 님프 소환
    WaterNymph,
    /// 29: 냄새 (몬스터 도주)
    Scare,
    /// 30: 분출
    Gush,
    /// 기본: 미지근한 물
    Tepid,
}

/// 분수 담금 결과 (rnd(30) 기반)
/// 원본: fountain.c dipfountain() L362-503
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DipFountainResult {
    /// 엑스칼리버 생성 (질서 성향)
    Excalibur,
    /// 엑스칼리버 시도 실패 (비질서 → 저주+강화 감소)
    ExcaliburFail { spe_decrease: bool },
    /// 16: 저주
    Curse,
    /// 17-20: 저주 해제
    Uncurse,
    /// 21: 물 악마
    WaterDemon,
    /// 22: 물 님프
    WaterNymph,
    /// 23: 뱀 떼
    WaterSnakes,
    /// 24: 보석 발견
    FindGem,
    /// 25: 분출
    Gush,
    /// 26-27: 기묘한 느낌 (메시지만)
    StrangeTingling,
    /// 28: 목욕 충동 (금 손실)
    BathUrge,
    /// 29: 동전 발견 (깊이 기반)
    FindCoins { gold_amount: i32 },
    /// 기본: 물 데미지만
    WaterDamage,
}

/// 싱크대 음용 결과 (rn2(20) 기반)
/// 원본: fountain.c drinksink() L519-626
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrinkSinkResult {
    /// 0: 매우 시원한 물
    VeryCold,
    /// 1: 매우 따뜻한 물
    VeryWarm,
    /// 2: 끓는 물 (화염 저항 없으면 데미지)
    ScaldingHot { damage: i32 },
    /// 3: 하수 쥐 (멸종 시 더러운 싱크대)
    SewerRat { extinct: bool },
    /// 4: 랜덤 물약 효과
    RandomPotion,
    /// 5: 반지 발견 (약탈 안 됨)
    FindRing { already_looted: bool },
    /// 6: 싱크대 파괴 → 분수 변환
    BreakSink,
    /// 7: 물 정령 소환
    WaterElemental,
    /// 8: 역겨운 물 (경험치 +1)
    YukWater,
    /// 9: 하수 맛 → 구역질 (배고픔 소비)
    SewerTaste { hunger_loss: i32 },
    /// 10: 독성 폐수 → 변신
    ToxicWaste,
    /// 11: 파이프 소리
    ClankyPipes,
    /// 12: 노래 소리
    SewageSong,
    /// 19: 환각 → 손 메시지
    HallucinationHand,
    /// 기타: 일반 물 (온도 랜덤)
    NormalWater,
}

/// 왕좌 앉기 결과 (rnd(13) 기반, 2/3 확률로 발동)
/// 원본: sit.c dosit() L163-275
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThroneResult {
    /// 1: 능력치 감소 + 데미지
    CurseAttrib { damage: i32 },
    /// 2: 능력치 증가
    GainAttrib,
    /// 3: 전기 충격 (저항에 따라 데미지 차등)
    ElectricShock { damage: i32 },
    /// 4: 완전 회복
    FullHeal,
    /// 5: 금 손실
    LoseGold,
    /// 6: 행운 변경 또는 소원
    LuckOrWish { is_wish: bool },
    /// 7: 궁정 소환 (rnd(10)마리)
    Audience { count: i32 },
    /// 8: 제노사이드
    Genocide,
    /// 9: 저주 (실명+행운 감소 또는 rndcurse)
    ThroneBlind { blind_duration: i32 },
    /// 10: 지도 또는 투명 보기
    MapOrSeeInvisible { is_map: bool },
    /// 11: 위협 또는 텔레포트
    AggrOrTele { is_aggr: bool },
    /// 12: 통찰 (아이템 식별)
    Insight,
    /// 13: 혼란
    Pretzel { confuse_duration: i32 },
    /// 효과 없음 (1/3 확률)
    Comfortable,
}

/// 저주 제거 대상 고유 능력
/// 원본: sit.c attrcurse() L401-488
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicToRemove {
    FireResistance,
    Teleportation,
    PoisonResistance,
    Telepathy,
    ColdResistance,
    Invisibility,
    SeeInvisible,
    Speed,
    Stealth,
    Protection,
    AggravateMonster,
    None,
}

// ============================================================
// 1. drink_fountain_result — 분수 음용 결과 결정
// ============================================================

/// 분수 음용 결과 결정
/// 원본: fountain.c drinkfountain() L222-358
pub fn drink_fountain_result(
    is_magic_fountain: bool,
    luck: i32,
    fountain_looted: bool,
    level_difficulty: i32,
    rng: &mut NetHackRng,
) -> DrinkFountainResult {
    let fate = rng.rnd(30);

    // 마법 분수 + 행운 ≥ 0 + fate ≥ 10
    if is_magic_fountain && luck >= 0 && fate >= 10 {
        return DrinkFountainResult::MagicBless;
    }

    if fate < 10 {
        return DrinkFountainResult::CoolDraught {
            hunger_gain: rng.rnd(10),
        };
    }

    // 마법 분수가 아닌 경우의 효과
    match fate {
        19 => DrinkFountainResult::SelfKnowledge,
        20 => DrinkFountainResult::Foul {
            hunger_loss: rng.rn1(20, 11),
        },
        21 => {
            let has_res = false; // 호출자가 판정
            let dmg = if has_res { rng.rnd(4) } else { rng.rnd(10) };
            let str_loss = if has_res { 0 } else { rng.rn1(4, 3) };
            DrinkFountainResult::Poison {
                has_resistance: has_res,
                damage: dmg,
                str_loss,
            }
        }
        22 => DrinkFountainResult::WaterSnakes {
            count: rng.rn1(5, 2),
        },
        23 => {
            // 소원 확률: rnd(100) > 80 + level_difficulty
            let grants_wish = rng.rnd(100) > (80 + level_difficulty);
            DrinkFountainResult::WaterDemon { grants_wish }
        }
        24 => DrinkFountainResult::CurseItems {
            hunger_loss: rng.rn1(20, 11),
        },
        25 => DrinkFountainResult::SeeInvisible,
        26 => DrinkFountainResult::SeeMonsters,
        27 => {
            if !fountain_looted {
                DrinkFountainResult::FindGem
            } else {
                DrinkFountainResult::WaterNymph
            }
        }
        28 => DrinkFountainResult::WaterNymph,
        29 => DrinkFountainResult::Scare,
        30 => DrinkFountainResult::Gush,
        _ => DrinkFountainResult::Tepid,
    }
}

// ============================================================
// 2. dip_fountain_result — 분수 담금 결과 결정
// ============================================================

/// 분수 담금 결과 결정
/// 원본: fountain.c dipfountain() L362-503
pub fn dip_fountain_result(
    is_long_sword: bool,
    quantity: i32,
    player_level: i32,
    is_lawful: bool,
    no_artifact: bool,
    excalibur_exists: bool,
    fountain_looted: bool,
    dungeon_depth: i32,
    max_depth: i32,
    rng: &mut NetHackRng,
) -> DipFountainResult {
    // 엑스칼리버 판정: 장검, 수량 1, 레벨 ≥ 5, 1/6 확률, 아티팩트 없음
    if is_long_sword
        && quantity == 1
        && player_level >= 5
        && rng.rn2(6) == 0
        && no_artifact
        && !excalibur_exists
    {
        if is_lawful {
            return DipFountainResult::Excalibur;
        } else {
            let spe_decrease = rng.rn2(3) != 0;
            return DipFountainResult::ExcaliburFail { spe_decrease };
        }
    }

    match rng.rnd(30) {
        16 => DipFountainResult::Curse,
        17 | 18 | 19 | 20 => DipFountainResult::Uncurse,
        21 => DipFountainResult::WaterDemon,
        22 => DipFountainResult::WaterNymph,
        23 => DipFountainResult::WaterSnakes,
        24 => {
            if !fountain_looted {
                DipFountainResult::FindGem
            } else {
                DipFountainResult::Gush
            }
        }
        25 => DipFountainResult::Gush,
        26 | 27 => DipFountainResult::StrangeTingling,
        28 => DipFountainResult::BathUrge,
        29 => {
            if !fountain_looted {
                let gold_base = max_depth - dungeon_depth + 1;
                let gold = rng.rnd(gold_base.max(1) * 2) + 5;
                DipFountainResult::FindCoins { gold_amount: gold }
            } else {
                DipFountainResult::WaterDamage
            }
        }
        _ => DipFountainResult::WaterDamage,
    }
}

// ============================================================
// 3. drink_sink_result — 싱크대 음용 결과 결정
// ============================================================

/// 싱크대 음용 결과 결정
/// 원본: fountain.c drinksink() L519-626
pub fn drink_sink_result(
    constitution: i32,
    rat_extinct: bool,
    sink_looted_ring: bool,
    is_hallucinating: bool,
    rng: &mut NetHackRng,
) -> DrinkSinkResult {
    match rng.rn2(20) {
        0 => DrinkSinkResult::VeryCold,
        1 => DrinkSinkResult::VeryWarm,
        2 => DrinkSinkResult::ScaldingHot { damage: rng.rnd(6) },
        3 => DrinkSinkResult::SewerRat {
            extinct: rat_extinct,
        },
        4 => DrinkSinkResult::RandomPotion,
        5 => DrinkSinkResult::FindRing {
            already_looted: sink_looted_ring,
        },
        6 => DrinkSinkResult::BreakSink,
        7 => DrinkSinkResult::WaterElemental,
        8 => DrinkSinkResult::YukWater,
        9 => DrinkSinkResult::SewerTaste {
            hunger_loss: rng.rn1(30 - constitution, 11),
        },
        10 => DrinkSinkResult::ToxicWaste,
        11 => DrinkSinkResult::ClankyPipes,
        12 => DrinkSinkResult::SewageSong,
        19 => {
            if is_hallucinating {
                DrinkSinkResult::HallucinationHand
            } else {
                DrinkSinkResult::NormalWater
            }
        }
        _ => DrinkSinkResult::NormalWater,
    }
}

// ============================================================
// 4. throne_result — 왕좌 앉기 결과 결정
// ============================================================

/// 왕좌 앉기 결과 결정
/// 원본: sit.c dosit() L163-275
pub fn throne_result(luck: i32, rng: &mut NetHackRng) -> ThroneResult {
    // 2/3 확률로 효과 발생 (rnd(6) > 4)
    if rng.rnd(6) <= 4 {
        return ThroneResult::Comfortable;
    }

    match rng.rnd(13) {
        1 => ThroneResult::CurseAttrib {
            damage: rng.rnd(10),
        },
        2 => ThroneResult::GainAttrib,
        3 => ThroneResult::ElectricShock {
            // 호출자가 저항 여부 판정, 여기서는 비저항 데미지
            damage: rng.rnd(30),
        },
        4 => ThroneResult::FullHeal,
        5 => ThroneResult::LoseGold,
        6 => {
            let is_wish = luck + rng.rn2(5) >= 0;
            ThroneResult::LuckOrWish { is_wish }
        }
        7 => ThroneResult::Audience { count: rng.rnd(10) },
        8 => ThroneResult::Genocide,
        9 => {
            let blind_dur = rng.rn1(100, 250);
            ThroneResult::ThroneBlind {
                blind_duration: blind_dur,
            }
        }
        10 => {
            let is_map = luck < 0;
            ThroneResult::MapOrSeeInvisible { is_map }
        }
        11 => {
            let is_aggr = luck < 0;
            ThroneResult::AggrOrTele { is_aggr }
        }
        12 => ThroneResult::Insight,
        13 => ThroneResult::Pretzel {
            confuse_duration: rng.rn1(7, 16),
        },
        _ => ThroneResult::Comfortable,
    }
}

// ============================================================
// 5. rndcurse_count — 저주할 아이템 수 계산
// ============================================================

/// rndcurse 시 저주할 아이템 수 결정
/// 원본: sit.c rndcurse() L356 — rnd(6 / divisor)
pub fn rndcurse_count(
    has_antimagic: bool,
    has_half_spell_damage: bool,
    rng: &mut NetHackRng,
) -> i32 {
    let divisor =
        1 + (if has_antimagic { 1 } else { 0 }) + (if has_half_spell_damage { 1 } else { 0 });
    rng.rnd(6 / divisor)
}

// ============================================================
// 6. attrcurse_pick — 제거할 고유 능력 선택
// ============================================================

/// attrcurse 시 제거할 고유 능력 결정
/// 원본: sit.c attrcurse() L401-488
/// FALLTHRU 패턴을 시뮬레이션: rnd(11) 결과부터 순서대로 보유 여부 확인
pub fn attrcurse_pick(
    has_fire_res: bool,
    has_teleportation: bool,
    has_poison_res: bool,
    has_telepathy: bool,
    has_cold_res: bool,
    has_invisibility: bool,
    has_see_invisible: bool,
    has_speed: bool,
    has_stealth: bool,
    has_protection: bool,
    has_aggravate: bool,
    rng: &mut NetHackRng,
) -> IntrinsicToRemove {
    let abilities = [
        (has_fire_res, IntrinsicToRemove::FireResistance),
        (has_teleportation, IntrinsicToRemove::Teleportation),
        (has_poison_res, IntrinsicToRemove::PoisonResistance),
        (has_telepathy, IntrinsicToRemove::Telepathy),
        (has_cold_res, IntrinsicToRemove::ColdResistance),
        (has_invisibility, IntrinsicToRemove::Invisibility),
        (has_see_invisible, IntrinsicToRemove::SeeInvisible),
        (has_speed, IntrinsicToRemove::Speed),
        (has_stealth, IntrinsicToRemove::Stealth),
        (has_protection, IntrinsicToRemove::Protection),
        (has_aggravate, IntrinsicToRemove::AggravateMonster),
    ];

    let start = (rng.rnd(11) - 1) as usize; // 0-indexed
                                            // FALLTHRU: start 위치부터 끝까지 검색
    for i in start..abilities.len() {
        if abilities[i].0 {
            return abilities[i].1;
        }
    }
    IntrinsicToRemove::None
}

// ============================================================
// 7. fountain_dryup_chance — 분수 말라붙기 확률
// ============================================================

/// 분수가 말라붙는지 판정
/// 원본: fountain.c dryup() L172 — !rn2(3) || warned
pub fn fountain_dryup_chance(is_warned: bool, rng: &mut NetHackRng) -> bool {
    rng.rn2(3) == 0 || is_warned
}

// ============================================================
// 8. water_snake_count — 뱀 떼 수량
// ============================================================

/// 뱀 떼 소환 수량
/// 원본: fountain.c dowatersnakes() L36 — rn1(5,2)
pub fn water_snake_count(rng: &mut NetHackRng) -> i32 {
    rng.rn1(5, 2) // 2~6
}

// ============================================================
// 9. water_demon_wish_chance — 물 악마 소원 확률
// ============================================================

/// 물 악마가 소원을 줄지 판정
/// 원본: fountain.c dowaterdemon() L70 — rnd(100) > 80 + level_difficulty
pub fn water_demon_wish_chance(level_difficulty: i32, rng: &mut NetHackRng) -> bool {
    rng.rnd(100) > (80 + level_difficulty)
}

// ============================================================
// 10. dip_gold_amount — 분수 담금 시 동전 수량
// ============================================================

/// 분수 담금 시 발견하는 동전 수량
/// 원본: fountain.c dipfountain() L491-493
pub fn dip_gold_amount(max_depth: i32, current_depth: i32, rng: &mut NetHackRng) -> i32 {
    let depth_factor = (max_depth - current_depth + 1).max(1);
    rng.rnd(depth_factor * 2) + 5
}

// ============================================================
// 11. throne_vanish_check — 왕좌 소멸 확률
// ============================================================

/// 왕좌에 앉은 후 소멸할지 판정
/// 원본: sit.c dosit() L283 — !rn2(3)
pub fn throne_vanish_check(rng: &mut NetHackRng) -> bool {
    rng.rn2(3) == 0
}

// ============================================================
// 12. sink_scalding_damage — 끓는 물 데미지
// ============================================================

/// 싱크대 끓는 물 데미지
pub fn sink_scalding_damage(rng: &mut NetHackRng) -> i32 {
    rng.rnd(6)
}

// ============================================================
// 13. sink_sewer_hunger — 하수 구역질 배고픔
// ============================================================

/// 싱크대 하수 구역질 시 들어간 배고픔
pub fn sink_sewer_hunger(constitution: i32, rng: &mut NetHackRng) -> i32 {
    rng.rn1(30 - constitution, 11)
}

// ============================================================
// 14. sit_trap_effect — 함정 위 앉기 추가 턴
// ============================================================

/// 함정 위에 앉으면 추가되는 함정 턴 계산
/// 원본: sit.c dosit() L84-114
pub fn sit_trap_additional_turns(trap_type: &str, rng: &mut NetHackRng) -> i32 {
    match trap_type {
        "bear_trap" => 1,
        "pit" => rng.rn2(5),
        "web" => rng.rn1(10, 5),
        "lava" => rng.rnd(4),
        "infloor" | "buried_ball" => 1,
        _ => 0,
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

    // --- drink_fountain_result ---
    #[test]
    fn test_magic_fountain_bless() {
        let mut rng = test_rng();
        let mut magic_seen = false;
        for _ in 0..100 {
            let result = drink_fountain_result(true, 5, false, 1, &mut rng);
            if result == DrinkFountainResult::MagicBless {
                magic_seen = true;
                break;
            }
        }
        assert!(magic_seen, "마법 분수 축복 발생");
    }

    #[test]
    fn test_fountain_varied_results() {
        let mut rng = test_rng();
        let mut cool = 0;
        let mut other = 0;
        for _ in 0..300 {
            match drink_fountain_result(false, 0, false, 5, &mut rng) {
                DrinkFountainResult::CoolDraught { .. } => cool += 1,
                _ => other += 1,
            }
        }
        // fate < 10 은 9/30 ≈ 30% 확률
        assert!(cool > 50 && cool < 150, "시원한 물 횟수: {}", cool);
        assert!(other > 0, "다른 효과 발생");
    }

    // --- dip_fountain_result ---
    #[test]
    fn test_dip_excalibur() {
        let mut rng = test_rng();
        let mut excal = false;
        for _ in 0..600 {
            let result =
                dip_fountain_result(true, 1, 10, true, true, false, false, 5, 20, &mut rng);
            if result == DipFountainResult::Excalibur {
                excal = true;
                break;
            }
        }
        assert!(excal, "엑스칼리버 생성 발생");
    }

    // --- drink_sink_result ---
    #[test]
    fn test_sink_all_results() {
        let mut rng = test_rng();
        let mut types_seen = std::collections::HashSet::new();
        for _ in 0..2000 {
            let result = drink_sink_result(14, false, false, false, &mut rng);
            types_seen.insert(format!("{:?}", result));
        }
        // 적어도 5가지 이상의 결과 유형이 나와야 함
        assert!(
            types_seen.len() >= 5,
            "싱크대 결과 다양성: {}",
            types_seen.len()
        );
    }

    // --- throne_result ---
    #[test]
    fn test_throne_comfortable_majority() {
        let mut rng = test_rng();
        let mut comfortable = 0;
        for _ in 0..300 {
            if let ThroneResult::Comfortable = throne_result(0, &mut rng) {
                comfortable += 1;
            }
        }
        // 4/6 ≈ 67% 확률로 Comfortable
        assert!(
            comfortable > 150 && comfortable < 250,
            "편안함 비율: {}/300",
            comfortable
        );
    }

    #[test]
    fn test_throne_effects() {
        let mut rng = test_rng();
        let mut effects_seen = 0;
        for _ in 0..300 {
            if let ThroneResult::Comfortable = throne_result(0, &mut rng) {
            } else {
                effects_seen += 1;
            }
        }
        assert!(effects_seen > 0, "왕좌 효과 발생");
    }

    // --- rndcurse_count ---
    #[test]
    fn test_rndcurse_count_normal() {
        let mut rng = test_rng();
        let cnt = rndcurse_count(false, false, &mut rng);
        assert!(cnt >= 1 && cnt <= 6, "저주 수: {}", cnt);
    }

    #[test]
    fn test_rndcurse_count_protected() {
        let mut rng = test_rng();
        let cnt = rndcurse_count(true, true, &mut rng);
        // 6/3=2, rnd(2) → 1~2
        assert!(cnt >= 1 && cnt <= 2, "보호 시 저주 수: {}", cnt);
    }

    // --- attrcurse_pick ---
    #[test]
    fn test_attrcurse_has_fire() {
        // 모든 능력 보유 시 반드시 어떤 능력이 제거됨 (None이 아님)
        for seed in 0..50u64 {
            let mut rng = NetHackRng::new(seed);
            let result = attrcurse_pick(
                true, true, true, true, true, true, true, true, true, true, true, &mut rng,
            );
            assert_ne!(
                result,
                IntrinsicToRemove::None,
                "시드 {}: None 반환 불가",
                seed
            );
        }
    }

    #[test]
    fn test_attrcurse_fire_reachable() {
        // 화염 저항만 보유 + 충분한 시드 → 반드시 FireResistance 제거
        let mut seen = false;
        for seed in 0..200u64 {
            let mut rng = NetHackRng::new(seed);
            let result = attrcurse_pick(
                true, false, false, false, false, false, false, false, false, false, false,
                &mut rng,
            );
            if result == IntrinsicToRemove::FireResistance {
                seen = true;
                break;
            }
        }
        assert!(seen, "200 seeds: FireResistance reachable");
    }

    #[test]
    fn test_attrcurse_none() {
        let mut rng = test_rng();
        let result = attrcurse_pick(
            false, false, false, false, false, false, false, false, false, false, false, &mut rng,
        );
        assert_eq!(result, IntrinsicToRemove::None);
    }

    // --- fountain_dryup_chance ---
    #[test]
    fn test_dryup_warned() {
        let mut rng = test_rng();
        assert!(fountain_dryup_chance(true, &mut rng));
    }

    // --- water_snake_count ---
    #[test]
    fn test_snake_count_range() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let cnt = water_snake_count(&mut rng);
            assert!(cnt >= 2 && cnt < 7, "뱀 수: {}", cnt);
        }
    }

    // --- dip_gold_amount ---
    #[test]
    fn test_gold_amount() {
        let mut rng = test_rng();
        for _ in 0..100 {
            let gold = dip_gold_amount(20, 5, &mut rng);
            // (20-5+1)*2=32, rnd(32)+5 → 6~37
            assert!(gold >= 6 && gold <= 37, "금액: {}", gold);
        }
    }

    // --- throne_vanish_check ---
    #[test]
    fn test_throne_vanish() {
        let mut rng = test_rng();
        let mut vanished = 0;
        for _ in 0..300 {
            if throne_vanish_check(&mut rng) {
                vanished += 1;
            }
        }
        // ~33% 확률
        assert!(vanished > 60 && vanished < 140, "왕좌 소멸: {}", vanished);
    }

    // --- sit_trap_additional_turns ---
    #[test]
    fn test_bear_trap_sit() {
        let mut rng = test_rng();
        assert_eq!(sit_trap_additional_turns("bear_trap", &mut rng), 1);
    }

    #[test]
    fn test_web_sit() {
        let mut rng = test_rng();
        let turns = sit_trap_additional_turns("web", &mut rng);
        assert!(turns >= 5 && turns < 15, "거미줄 추가 턴: {}", turns);
    }
}
