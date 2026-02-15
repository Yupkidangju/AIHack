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
