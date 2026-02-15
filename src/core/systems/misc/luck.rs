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
use crate::core::entity::player::Player;
use crate::ui::log::GameLog;
use legion::world::SubWorld;
use legion::IntoQuery;

///
///

///
#[legion::system]
#[write_component(Player)]
pub fn luck_maintenance(
    world: &mut SubWorld,
    #[resource] _log: &mut GameLog,
    #[resource] _turn: &u64,
) {
    let mut query = <&mut Player>::query();
    for player in query.iter_mut(world) {
        // 1. Luck Timeout 泥섎━ (u.uluckcnt)
        //
        if player.luck != 0 {
            player.luck_turns -= 1;
            if player.luck_turns <= 0 {
                if player.luck > 0 {
                    player.luck -= 1;
                } else if player.luck < 0 {
                    player.luck += 1;
                }
                player.luck_turns = 600;

                //
                // if player.luck == 0 {
                //
                // }
            }
        } else {
            //
            player.luck_turns = 600;
        }

        // 2. Base Luck vs Bonus Luck (u.uluck)
        //
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub const MINIMUM_LUCK: i32 = -13;
pub const MAXIMUM_LUCK: i32 = 13;

///
pub fn effective_luck(base_luck: i32, bonus: i32, has_luckstone: bool) -> i32 {
    let mut luck = base_luck + bonus;
    if has_luckstone {
        //
        if luck > 0 {
            luck = luck.max(1);
        } else if luck < 0 {
            luck = luck.min(-1);
        }
    }
    luck.clamp(MINIMUM_LUCK, MAXIMUM_LUCK)
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuckstoneType {
    None,
    Luckstone,
    Loadstone,
    Healthstone,
    Touchstone,
}

///
pub fn luckstone_effect(stone: LuckstoneType, current_luck: i32) -> i32 {
    match stone {
        LuckstoneType::None => 0,
        LuckstoneType::Luckstone => {
            if current_luck < 0 {
                1
            } else {
                0
            }
        }
        LuckstoneType::Loadstone => -1,
        LuckstoneType::Healthstone => 0,
        LuckstoneType::Touchstone => 0,
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuckCause {
    PrayerAnswered,
    PrayerFailed,
    Murder,
    Sacrifice,
    Theft,
    Charity,
    BreakMirror,
    CrossBlackCat,
    Unicorn,
    FountainDip,
    GodWrath,
    ArtifactGift,
}

///
pub fn luck_change_amount(cause: LuckCause) -> i32 {
    match cause {
        LuckCause::PrayerAnswered => 1,
        LuckCause::PrayerFailed => -1,
        LuckCause::Murder => -5,
        LuckCause::Sacrifice => 2,
        LuckCause::Theft => -1,
        LuckCause::Charity => 2,
        LuckCause::BreakMirror => -2,
        LuckCause::CrossBlackCat => -1,
        LuckCause::Unicorn => 1,
        LuckCause::FountainDip => 0,
        LuckCause::GodWrath => -3,
        LuckCause::ArtifactGift => 2,
    }
}

///
pub fn apply_luck_change(current: i32, cause: LuckCause) -> i32 {
    let new_luck = current + luck_change_amount(cause);
    new_luck.clamp(MINIMUM_LUCK, MAXIMUM_LUCK)
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
///
pub fn luck_adjusted_roll(n: i32, luck: i32, rng: &mut crate::util::rng::NetHackRng) -> i32 {
    if n <= 0 {
        return 0;
    }
    let mut result = rng.rn2(n);

    //
    if luck > 0 {
        for _ in 0..luck.min(3) {
            let reroll = rng.rn2(n);
            if reroll < result {
                result = reroll;
            }
        }
    }
    //
    if luck < 0 {
        for _ in 0..(-luck).min(3) {
            let reroll = rng.rn2(n);
            if reroll > result {
                result = reroll;
            }
        }
    }

    result
}

///
pub fn luck_success_bonus(base_chance: i32, luck: i32) -> i32 {
    let bonus = luck * 3;
    (base_chance + bonus).clamp(1, 99)
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct LuckStatistics {
    pub max_luck_reached: i32,
    pub min_luck_reached: i32,
    pub prayers_answered: u32,
    pub prayers_failed: u32,
    pub murders_committed: u32,
    pub sacrifices_made: u32,
    pub luckstone_found: bool,
}

impl LuckStatistics {
    pub fn new() -> Self {
        Self {
            max_luck_reached: 0,
            min_luck_reached: 0,
            ..Default::default()
        }
    }

    pub fn record_luck(&mut self, luck: i32) {
        if luck > self.max_luck_reached {
            self.max_luck_reached = luck;
        }
        if luck < self.min_luck_reached {
            self.min_luck_reached = luck;
        }
    }

    pub fn record_cause(&mut self, cause: LuckCause) {
        match cause {
            LuckCause::PrayerAnswered => self.prayers_answered += 1,
            LuckCause::PrayerFailed => self.prayers_failed += 1,
            LuckCause::Murder => self.murders_committed += 1,
            LuckCause::Sacrifice => self.sacrifices_made += 1,
            _ => {}
        }
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod luck_extended_tests {
    use super::*;

    #[test]
    fn test_effective_luck() {
        assert_eq!(effective_luck(3, 2, false), 5);
        assert_eq!(effective_luck(15, 0, false), MAXIMUM_LUCK);
        assert_eq!(effective_luck(-15, 0, false), MINIMUM_LUCK);
    }

    #[test]
    fn test_luckstone_protection() {
        //
        let e = effective_luck(0, 1, true);
        assert!(e >= 1);
    }

    #[test]
    fn test_luck_change() {
        assert_eq!(apply_luck_change(0, LuckCause::Murder), -5);
        assert_eq!(apply_luck_change(0, LuckCause::Charity), 2);
        assert_eq!(
            apply_luck_change(12, LuckCause::PrayerAnswered),
            MAXIMUM_LUCK
        );
    }

    #[test]
    fn test_luck_adjusted_roll() {
        let mut rng = crate::util::rng::NetHackRng::new(42);
        //
        let mut low_count = 0;
        for _ in 0..100 {
            let r = luck_adjusted_roll(20, 3, &mut rng);
            if r < 5 {
                low_count += 1;
            }
        }
        //
        assert!(low_count > 20);
    }

    #[test]
    fn test_luck_success_bonus() {
        assert_eq!(luck_success_bonus(50, 5), 65);
        assert_eq!(luck_success_bonus(50, -5), 35);
        assert_eq!(luck_success_bonus(99, 10), 99);
    }

    #[test]
    fn test_luck_stats() {
        let mut stats = LuckStatistics::new();
        stats.record_luck(5);
        stats.record_luck(-3);
        stats.record_cause(LuckCause::Murder);
        assert_eq!(stats.max_luck_reached, 5);
        assert_eq!(stats.min_luck_reached, -3);
        assert_eq!(stats.murders_committed, 1);
    }
}
