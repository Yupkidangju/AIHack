// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::tile::TileType;
// use crate::core::dungeon::Grid;
use crate::core::entity::player::Player;
use crate::core::entity::{Health, PlayerTag, Position};
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

///
///
pub fn try_sit(
    world: &mut SubWorld,
    grid: &crate::core::dungeon::Grid,
    log: &mut GameLog,
    turn: u64,
    rng: &mut NetHackRng,
) -> bool {
    //
    let mut p_query =
        <(&mut Player, &mut Health, &mut Position)>::query().filter(component::<PlayerTag>());
    let mut player_packet = None;

    //
    for (p, h, pos) in p_query.iter_mut(world) {
        player_packet = Some((p, h, pos));
        break;
    }

    if let Some((p_stats, p_health, p_pos)) = player_packet {
        //
        if let Some(tile) = grid.get_tile(p_pos.x as usize, p_pos.y as usize) {
            match tile.typ {
                TileType::Throne => {
                    log.add("You sit on the throne.", turn);

                    //
                    //
                    //
                    // 2. ??(Gold)
                    //
                    //
                    //
                    //

                    let roll = rng.rn2(100);

                    if roll < 5 {
                        //
                        log.add_colored(
                            "You feel lucky! (Wish implemented partially)",
                            [255, 215, 0],
                            turn,
                        );
                        // TODO: Wish UI
                        p_stats.gold += 5000;
                    } else if roll < 15 {
                        // 10% ??
                        let gold = rng.rn1(500, 200) as u64;
                        p_stats.gold += gold;
                        log.add(
                            format!("You find {} gold pieces in the cushions.", gold),
                            turn,
                        );
                    } else if roll < 25 {
                        //
                        log.add("You feel restored.", turn);
                        p_health.current = p_health.max;
                        p_stats.str.base += 1;
                        if p_stats.str.base > 18 {
                            p_stats.str.base = 18;
                        }
                    } else if roll < 35 {
                        //
                        log.add("The world spins around you!", turn);
                        //
                        //
                        //
                        //
                        //
                    } else if roll < 50 {
                        //
                        log.add_colored("Monsters appear from nowhere!", [255, 0, 0], turn);
                        //
                    } else if roll < 60 {
                        //
                        log.add("A shock runs through your body!", turn);
                        p_health.current -= rng.rn1(10, 6);
                        if p_health.current <= 0 {
                            log.add_colored(
                                "You die from the electric shock...",
                                [255, 0, 0],
                                turn,
                            );
                            //
                        }
                    } else if roll < 70 {
                        //
                        log.add("Your vision blurs.", turn);
                        //
                    } else {
                        log.add("You feel very comfortable.", turn);
                    }
                }
                _ => {
                    log.add("You sit on the floor.", turn);
                    if p_health.current < p_health.max {
                        p_health.current += 1;
                        log.add("You feel slightly rested.", turn);
                    }
                }
            }
            return true;
        }
    }

    //
    //
    //
    false
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThroneEffect {
    Nothing,
    Wish,
    GoldDrop,
    Genocide,
    Identify,
    Heal,
    StatIncrease,
    StatDecrease,
    Teleport,
    MonsterSummon,
    ElectricShock,
    Blind,
    Confuse,
    Poison,        // ??
    Polymorph,
    Alignment,
}

///
pub fn throne_effect(rng: &mut NetHackRng) -> ThroneEffect {
    let roll = rng.rn2(100);
    match roll {
        0..=2 => ThroneEffect::Wish,            // 3%
        3..=7 => ThroneEffect::GoldDrop,        // 5%
        8..=9 => ThroneEffect::Genocide,        // 2%
        10..=14 => ThroneEffect::Identify,      // 5%
        15..=24 => ThroneEffect::Heal,          // 10%
        25..=32 => ThroneEffect::StatIncrease,  // 8%
        33..=37 => ThroneEffect::StatDecrease,  // 5%
        38..=47 => ThroneEffect::Teleport,      // 10%
        48..=57 => ThroneEffect::MonsterSummon, // 10%
        58..=65 => ThroneEffect::ElectricShock, // 8%
        66..=72 => ThroneEffect::Blind,         // 7%
        73..=78 => ThroneEffect::Confuse,       // 6%
        79..=83 => ThroneEffect::Poison,        // 5%
        84..=88 => ThroneEffect::Polymorph,     // 5%
        89..=93 => ThroneEffect::Alignment,     // 5%
        _ => ThroneEffect::Nothing,             // 6%
    }
}

///
pub fn throne_effect_message(effect: &ThroneEffect) -> &'static str {
    match effect {
        ThroneEffect::Nothing => "You feel very comfortable.",
        ThroneEffect::Wish => "You feel telepathic... A voice booms: \"Thy wish is granted!\"",
        ThroneEffect::GoldDrop => "You find gold coins in the cushion!",
        ThroneEffect::Genocide => "A voice whispers: \"Choose a species to genocide.\"",
        ThroneEffect::Identify => "You feel more knowledgeable.",
        ThroneEffect::Heal => "You feel restored to health!",
        ThroneEffect::StatIncrease => "You feel your abilities increasing!",
        ThroneEffect::StatDecrease => "You feel your abilities diminishing!",
        ThroneEffect::Teleport => "The world spins around you!",
        ThroneEffect::MonsterSummon => "Monsters appear from nowhere!",
        ThroneEffect::ElectricShock => "A shock runs through your body!",
        ThroneEffect::Blind => "Your vision goes dark!",
        ThroneEffect::Confuse => "Your head spins!",
        ThroneEffect::Poison => "Something sharp pricks you from the cushion!",
        ThroneEffect::Polymorph => "You feel different!",
        ThroneEffect::Alignment => "You feel a change in your moral compass.",
    }
}

///
pub fn throne_gold_amount(luck: i32, rng: &mut NetHackRng) -> u64 {
    let base = rng.rn1(500, 100) as u64;
    if luck > 0 {
        base + (luck as u64 * 50)
    } else {
        base
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SitResult {
    Floor,
    Throne(ThroneEffect),
    Sink,
    Fountain,
    Altar,
    Trap(String),
    Lava,
    Water,
    Grave,
}

///
pub fn sit_result_message(result: &SitResult) -> String {
    match result {
        SitResult::Floor => "You sit on the floor. You feel slightly rested.".to_string(),
        SitResult::Throne(effect) => throne_effect_message(effect).to_string(),
        SitResult::Sink => "You sit on the sink. You get wet!".to_string(),
        SitResult::Fountain => "You sit in the fountain. You get soaked!".to_string(),
        SitResult::Altar => "You sit on the altar. You feel a holy presence.".to_string(),
        SitResult::Trap(name) => format!("You sit on a {} and it activates!", name),
        SitResult::Lava => "You sit in lava! Bad idea!".to_string(),
        SitResult::Water => "You sit in the water. Splosh!".to_string(),
        SitResult::Grave => "You sit on a grave. You feel disrespectful.".to_string(),
    }
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AltarSitEffect {
    Nothing,
    AlignmentBonus,
    AlignmentPenalty,
    Curse,
}

///
pub fn altar_sit_effect(
    player_alignment: i32,
    altar_alignment: i32,
    rng: &mut NetHackRng,
) -> AltarSitEffect {
    if player_alignment == altar_alignment {
        if rng.rn2(3) == 0 {
            AltarSitEffect::AlignmentBonus
        } else {
            AltarSitEffect::Nothing
        }
    } else {
        if rng.rn2(2) == 0 {
            AltarSitEffect::AlignmentPenalty
        } else {
            AltarSitEffect::Nothing
        }
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct SitStatistics {
    pub total_sits: u32,
    pub throne_sits: u32,
    pub wishes_from_throne: u32,
    pub gold_from_throne: u64,
    pub damage_from_sitting: i32,
    pub floor_sits: u32,
}

impl SitStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_sit(&mut self, result: &SitResult) {
        self.total_sits += 1;
        match result {
            SitResult::Throne(effect) => {
                self.throne_sits += 1;
                if *effect == ThroneEffect::Wish {
                    self.wishes_from_throne += 1;
                }
            }
            SitResult::Floor => self.floor_sits += 1,
            _ => {}
        }
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod sit_extended_tests {
    use super::*;

    #[test]
    fn test_throne_effect_variety() {
        let mut rng = NetHackRng::new(42);
        let mut effects = std::collections::HashSet::new();
        for _ in 0..200 {
            let e = throne_effect(&mut rng);
            effects.insert(format!("{:?}", e));
        }
        assert!(effects.len() >= 8);
    }

    #[test]
    fn test_throne_message() {
        assert!(throne_effect_message(&ThroneEffect::Wish).contains("wish"));
    }

    #[test]
    fn test_throne_gold() {
        let mut rng = NetHackRng::new(42);
        let g1 = throne_gold_amount(-5, &mut rng);
        let mut rng2 = NetHackRng::new(42);
        let g2 = throne_gold_amount(10, &mut rng2);
        assert!(g2 > g1);
    }

    #[test]
    fn test_sit_result_message() {
        let m = sit_result_message(&SitResult::Lava);
        assert!(m.contains("lava"));
    }

    #[test]
    fn test_altar_sit_same_alignment() {
        let mut rng = NetHackRng::new(42);
        //
        let mut bonus = 0;
        for _ in 0..30 {
            let e = altar_sit_effect(1, 1, &mut rng);
            if e == AltarSitEffect::AlignmentBonus {
                bonus += 1;
            }
        }
        assert!(bonus > 0);
    }

    #[test]
    fn test_sit_stats() {
        let mut stats = SitStatistics::new();
        stats.record_sit(&SitResult::Throne(ThroneEffect::Wish));
        stats.record_sit(&SitResult::Floor);
        assert_eq!(stats.total_sits, 2);
        assert_eq!(stats.throne_sits, 1);
        assert_eq!(stats.wishes_from_throne, 1);
        assert_eq!(stats.floor_sits, 1);
    }
}

// =============================================================================
// [v2.9.7] sit.c 미구현 함수 이식 — take_gold, rndcurse, attrcurse
// =============================================================================

/// [v2.9.7] take_gold 결과 (원본: sit.c L10-31)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TakeGoldResult {
    /// 돈이 없어서 "이상한 감각" 메시지
    NoMoney,
    /// 돈이 제거됨
    MoneyLost,
}

/// [v2.9.7] 돈 빼앗기 판정 (원본: take_gold L10-31)
pub fn take_gold_result(has_coins: bool) -> TakeGoldResult {
    if has_coins { TakeGoldResult::MoneyLost } else { TakeGoldResult::NoMoney }
}

/// [v2.9.7] rndcurse 결과 (원본: sit.c L330-398)
#[derive(Debug, Clone)]
pub struct RndCurseResult {
    /// Magicbane이 흡수했는지
    pub magicbane_absorbed: bool,
    /// 반마법 방어 발동 여부
    pub antimagic_shield: bool,
    /// 저주할 아이템 인덱스 목록
    pub items_to_curse: Vec<usize>,
    /// 이미 저주된 아이템 스킵 수
    pub already_cursed_skips: i32,
    /// 아티팩트 저항 수
    pub artifact_resists: i32,
    /// 탈것 안장 저주 여부
    pub saddle_cursed: bool,
}

/// [v2.9.7] 랜덤 저주 판정 (원본: rndcurse L330-398)
/// item_count: 저주 가능 아이템 수 (금화 제외)
/// item_cursed: 각 아이템의 현재 저주 상태
/// item_blessed: 각 아이템의 축복 상태
/// item_is_intelligent_artifact: 지능형 아티팩트 여부
pub fn rndcurse_result(
    has_magicbane_wielded: bool,
    has_antimagic: bool,
    has_half_spell_damage: bool,
    item_count: usize,
    item_cursed: &[bool],
    item_is_intelligent_artifact: &[bool],
    has_steed_with_saddle: bool,
    saddle_already_cursed: bool,
    rng: &mut NetHackRng,
) -> RndCurseResult {
    // Magicbane 흡수: 95% 확률
    if has_magicbane_wielded && rng.rn2(20) != 0 {
        return RndCurseResult {
            magicbane_absorbed: true,
            antimagic_shield: false,
            items_to_curse: vec![],
            already_cursed_skips: 0,
            artifact_resists: 0,
            saddle_cursed: false,
        };
    }

    // 저주할 아이템 수 계산 (원본: rnd(6 / (1 + antimagic + half_spell)))
    let divisor = 1 + (has_antimagic as i32) + (has_half_spell_damage as i32);
    let curse_count = rng.rnd(6 / divisor);

    let mut items_to_curse = Vec::new();
    let mut already_cursed = 0;
    let mut artifact_resists = 0;

    if item_count > 0 {
        for _ in 0..curse_count {
            let target_idx = rng.rnd(item_count as i32) as usize - 1;
            if target_idx >= item_count { continue; }

            // 이미 저주된 경우 스킵
            if target_idx < item_cursed.len() && item_cursed[target_idx] {
                already_cursed += 1;
                continue;
            }

            // 지능형 아티팩트는 80% 확률로 저항
            if target_idx < item_is_intelligent_artifact.len()
                && item_is_intelligent_artifact[target_idx]
                && rng.rn2(10) < 8
            {
                artifact_resists += 1;
                continue;
            }

            items_to_curse.push(target_idx);
        }
    }

    // 탈것 안장 저주: 25% 확률
    let saddle = has_steed_with_saddle && !saddle_already_cursed && rng.rn2(4) == 0;

    RndCurseResult {
        magicbane_absorbed: false,
        antimagic_shield: has_antimagic,
        items_to_curse,
        already_cursed_skips: already_cursed,
        artifact_resists,
        saddle_cursed: saddle,
    }
}

/// [v2.9.7] 제거 가능한 내재 능력 (원본: attrcurse L400-489)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicAbility {
    FireResistance,
    Teleportation,
    PoisonResistance,
    Telepathy,
    ColdResistance,
    Invisibility,
    SeeInvisible,
    Fast,
    Stealth,
    Protection,
    AggravateMonster,
}

/// [v2.9.7] attrcurse 결과
#[derive(Debug, Clone)]
pub struct AttrCurseResult {
    /// 제거할 내재 능력 (None이면 아무것도 제거 안됨)
    pub removed_ability: Option<IntrinsicAbility>,
    /// 메시지
    pub message: &'static str,
}

/// [v2.9.7] 내재 능력 제거 판정 (원본: attrcurse L400-489)
/// flags: 각 내재 능력 보유 여부 [11개 — FireRes부터 AggravateMonster까지]
pub fn attrcurse_result(
    intrinsic_flags: &[bool; 11],
    rng: &mut NetHackRng,
) -> AttrCurseResult {
    let abilities = [
        IntrinsicAbility::FireResistance,
        IntrinsicAbility::Teleportation,
        IntrinsicAbility::PoisonResistance,
        IntrinsicAbility::Telepathy,
        IntrinsicAbility::ColdResistance,
        IntrinsicAbility::Invisibility,
        IntrinsicAbility::SeeInvisible,
        IntrinsicAbility::Fast,
        IntrinsicAbility::Stealth,
        IntrinsicAbility::Protection,
        IntrinsicAbility::AggravateMonster,
    ];
    let messages = [
        "You feel warmer.",
        "You feel less jumpy.",
        "You feel a little sick!",
        "Your senses fail!",
        "You feel cooler.",
        "You feel paranoid.",
        "You thought you saw something!",
        "You feel slower.",
        "You feel clumsy.",
        "You feel vulnerable.",
        "You feel less attractive.",
    ];

    // 원본: rnd(11)로 시작, FALLTHRU로 계단식 체크 (보유한 첫 번째 능력 제거)
    let start = rng.rnd(11) as usize - 1; // 0~10

    for offset in 0..=10 {
        let idx = (start + offset) % 11;
        if intrinsic_flags[idx] {
            return AttrCurseResult {
                removed_ability: Some(abilities[idx]),
                message: messages[idx],
            };
        }
    }

    // 아무 내재 능력도 없는 경우
    AttrCurseResult {
        removed_ability: None,
        message: "",
    }
}

/// [v2.9.7] dosit에서의 왕좌 판정 확장 — 왕좌 소멸 여부 (원본: dosit L283-288)
pub fn throne_vanishes(rng: &mut NetHackRng) -> bool {
    rng.rn2(3) == 0 // 33% 확률
}

/// [v2.9.7] 알 낳기 가능 여부 (원본: dosit L289-323)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayEggResult {
    /// 수컷이라 불가
    MaleCantLay,
    /// 배고파서 불가
    TooHungry,
    /// 수중 종인데 물 밖
    NeedWater,
    /// 알 낳기 성공
    Success { nutrition_cost: i32 },
}

/// [v2.9.7] 알 낳기 판정
pub fn lay_egg_result(
    is_egg_layer: bool,
    is_female: bool,
    hunger: i32,
    egg_nutrition: i32,
    lays_in_water: bool,
    is_underwater: bool,
) -> LayEggResult {
    if !is_egg_layer {
        return LayEggResult::MaleCantLay; // 사실 호출 안 됨
    }
    if !is_female {
        return LayEggResult::MaleCantLay;
    }
    if hunger < egg_nutrition {
        return LayEggResult::TooHungry;
    }
    if lays_in_water && !is_underwater {
        return LayEggResult::NeedWater;
    }
    LayEggResult::Success { nutrition_cost: egg_nutrition }
}

// =============================================================================
// [v2.9.7] 테스트 — sit.c 추가 이식분
// =============================================================================
#[cfg(test)]
mod sit_phase2_tests {
    use super::*;

    #[test]
    fn test_take_gold() {
        assert_eq!(take_gold_result(true), TakeGoldResult::MoneyLost);
        assert_eq!(take_gold_result(false), TakeGoldResult::NoMoney);
    }

    #[test]
    fn test_rndcurse_magicbane() {
        let mut rng = NetHackRng::new(42);
        // Magicbane은 95% 확률로 흡수
        let mut absorbed = 0;
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            let result = rndcurse_result(true, false, false, 5, &[false; 5], &[false; 5], false, false, &mut r);
            if result.magicbane_absorbed { absorbed += 1; }
        }
        assert!(absorbed > 80); // ~95%
    }

    #[test]
    fn test_rndcurse_normal() {
        let mut rng = NetHackRng::new(42);
        let result = rndcurse_result(false, false, false, 5, &[false; 5], &[false; 5], false, false, &mut rng);
        assert!(!result.magicbane_absorbed);
        // 저주 대상이 있을 수 있음
    }

    #[test]
    fn test_rndcurse_saddle() {
        // 탈것 안장 저주 확률 테스트
        let mut saddle_cursed = 0;
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            let result = rndcurse_result(false, false, false, 0, &[], &[], true, false, &mut r);
            if result.saddle_cursed { saddle_cursed += 1; }
        }
        // 25% 기대값
        assert!(saddle_cursed > 10 && saddle_cursed < 45);
    }

    #[test]
    fn test_attrcurse_with_fire_res() {
        let mut flags = [false; 11];
        flags[0] = true; // 화염 저항
        let mut rng = NetHackRng::new(42);
        // 반복 시도하면 결국 화염 저항이 제거됨
        let mut found = false;
        for seed in 0..20u64 {
            let mut r = NetHackRng::new(seed);
            let result = attrcurse_result(&flags, &mut r);
            if result.removed_ability == Some(IntrinsicAbility::FireResistance) {
                found = true;
                assert_eq!(result.message, "You feel warmer.");
                break;
            }
        }
        assert!(found);
    }

    #[test]
    fn test_attrcurse_no_intrinsics() {
        let flags = [false; 11];
        let mut rng = NetHackRng::new(42);
        let result = attrcurse_result(&flags, &mut rng);
        assert!(result.removed_ability.is_none());
    }

    #[test]
    fn test_attrcurse_multiple() {
        let mut flags = [true; 11]; // 모든 능력 보유
        let mut rng = NetHackRng::new(42);
        let result = attrcurse_result(&flags, &mut rng);
        assert!(result.removed_ability.is_some());
    }

    #[test]
    fn test_throne_vanishes() {
        let mut vanished = 0;
        for seed in 0..100u64 {
            let mut r = NetHackRng::new(seed);
            if throne_vanishes(&mut r) { vanished += 1; }
        }
        assert!(vanished > 20 && vanished < 50); // ~33%
    }

    #[test]
    fn test_lay_egg_male() {
        assert_eq!(lay_egg_result(true, false, 100, 80, false, false), LayEggResult::MaleCantLay);
    }

    #[test]
    fn test_lay_egg_hungry() {
        assert_eq!(lay_egg_result(true, true, 50, 80, false, false), LayEggResult::TooHungry);
    }

    #[test]
    fn test_lay_egg_success() {
        let r = lay_egg_result(true, true, 100, 80, false, false);
        assert_eq!(r, LayEggResult::Success { nutrition_cost: 80 });
    }

    #[test]
    fn test_lay_egg_water() {
        assert_eq!(lay_egg_result(true, true, 100, 80, true, false), LayEggResult::NeedWater);
        let r = lay_egg_result(true, true, 100, 80, true, true);
        assert_eq!(r, LayEggResult::Success { nutrition_cost: 80 });
    }
}
