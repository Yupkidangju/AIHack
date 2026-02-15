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
use crate::core::entity::status::{StatusBundle, StatusFlags};
use crate::core::entity::Health;
use crate::ui::log::GameLog;
use legion::world::SubWorld;
use legion::IntoQuery;

const STONED_TEXTS: [&str; 5] = [
    "You are slowing down.",
    "Your limbs are stiffening.",
    "Your limbs have turned to stone.",
    "You have turned to stone.",
    "You are a statue.",
];

const SLIME_TEXTS: [&str; 5] = [
    "You are turning a little green.",
    "Your limbs are getting oozy.",
    "Your skin begins to peel away.",
    "You are turning into green slime.",
    "You have become green slime.",
];

const CHOKE_TEXTS: [&str; 5] = [
    "You are choking!",
    "You are gasping for air!",
    "You can't breathe!",
    "Your face is turning blue!",
    "You die from choking.",
];

const VOMIT_TEXTS: [&str; 5] = [
    "You feel nauseous.",
    "You feel like throwing up.",
    "You are vomiting uncontrollably.",
    "You are losing consciousness from vomiting.",
    "You die from vomiting.",
];

const STRANGLE_TEXTS: [&str; 5] = [
    "You are being strangled!",
    "You are gasping for air!",
    "You can't breathe!",
    "Your face is turning blue!",
    "You die from strangulation.",
];

#[legion::system]
#[write_component(StatusBundle)]
#[write_component(Health)]
pub fn timeout_dialogue(
    world: &mut SubWorld,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
) {
    let mut query = <(&mut StatusBundle, Option<&mut Health>)>::query();
    for (status, mut health) in query.iter_mut(world) {
        // Stoning dialogue
        if let Some(s) = status
            .active
            .iter()
            .find(|s| s.flag == StatusFlags::STONING)
        {
            let idx = (s.remaining_turns as usize).min(STONED_TEXTS.len()) - 1;
            if s.remaining_turns > 0 && s.remaining_turns <= 5 {
                log.add(
                    STONED_TEXTS[STONED_TEXTS.len() - 1 - idx].to_string(),
                    *turn,
                );
            }
            if s.remaining_turns == 1 {
                if let Some(h) = health.as_mut() {
                    h.current = 0; // Death by stoning
                }
            }
        }

        // Sliming dialogue
        if let Some(s) = status.active.iter().find(|s| s.flag == StatusFlags::SLIMED) {
            let idx = (s.remaining_turns as usize).min(SLIME_TEXTS.len()) - 1;
            if s.remaining_turns > 0 && s.remaining_turns <= 5 {
                log.add(SLIME_TEXTS[SLIME_TEXTS.len() - 1 - idx].to_string(), *turn);
            }
            if s.remaining_turns == 1 {
                if let Some(h) = health.as_mut() {
                    h.current = 0; // Death by sliming
                }
            }
        }

        // Choking dialogue
        if let Some(s) = status
            .active
            .iter()
            .find(|s| s.flag == StatusFlags::CHOKING)
        {
            let idx = (s.remaining_turns as usize).min(CHOKE_TEXTS.len()) - 1;
            if s.remaining_turns > 0 && s.remaining_turns <= 5 {
                log.add(CHOKE_TEXTS[CHOKE_TEXTS.len() - 1 - idx].to_string(), *turn);
            }
            if s.remaining_turns == 1 {
                if let Some(h) = health.as_mut() {
                    h.current = 0; // Death by choking
                }
            }
        }

        // Vomiting dialogue
        if let Some(s) = status
            .active
            .iter()
            .find(|s| s.flag == StatusFlags::VOMITING)
        {
            let idx = (s.remaining_turns as usize).min(VOMIT_TEXTS.len()) - 1;
            if s.remaining_turns > 0 && s.remaining_turns <= 5 {
                log.add(VOMIT_TEXTS[VOMIT_TEXTS.len() - 1 - idx].to_string(), *turn);
            }
            if s.remaining_turns == 1 {
                if let Some(h) = health.as_mut() {
                    h.current = 0; // Death by vomiting
                }
            }
        }
        // Strangling dialogue
        if let Some(s) = status
            .active
            .iter()
            .find(|s| s.flag == StatusFlags::STRANGLED)
        {
            let idx = (s.remaining_turns as usize).min(STRANGLE_TEXTS.len()) - 1;
            if s.remaining_turns > 0 && s.remaining_turns <= 5 {
                log.add(
                    STRANGLE_TEXTS[STRANGLE_TEXTS.len() - 1 - idx].to_string(),
                    *turn,
                );
            }
            if s.remaining_turns == 1 {
                if let Some(h) = health.as_mut() {
                    h.current = 0; // Death by strangulation
                }
            }
        }
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerCategory {
    Lethal,
    Transform,
    Debuff,
    Buff,
    Environmental,
}

///
pub fn timer_category(flag: StatusFlags) -> TimerCategory {
    match flag {
        StatusFlags::STONING | StatusFlags::CHOKING | StatusFlags::STRANGLED => {
            TimerCategory::Lethal
        }
        StatusFlags::SLIMED => TimerCategory::Transform,
        StatusFlags::POISONED
        | StatusFlags::BLIND
        | StatusFlags::CONFUSED
        | StatusFlags::STUNNED
        | StatusFlags::HALLUCINATING => TimerCategory::Debuff,
        StatusFlags::REFLECTING | StatusFlags::FAST | StatusFlags::INFRAVISION => {
            TimerCategory::Buff
        }
        _ => TimerCategory::Environmental,
    }
}

///
pub fn timer_priority(category: TimerCategory) -> i32 {
    match category {
        TimerCategory::Lethal => 100,
        TimerCategory::Transform => 80,
        TimerCategory::Debuff => 50,
        TimerCategory::Environmental => 30,
        TimerCategory::Buff => 10,
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
pub fn timer_warning_message(flag: StatusFlags, remaining: u32) -> Option<&'static str> {
    if remaining > 10 {
        return None;
    }

    //
    if flag == StatusFlags::STONING {
        return Some(match remaining {
            1 => "You are a statue.",
            2 => "You have turned to stone.",
            3 => "Your limbs have turned to stone.",
            4 => "Your limbs are stiffening.",
            5 => "You are slowing down.",
            _ => "You feel somewhat stiff.",
        });
    }

    if flag == StatusFlags::SLIMED {
        return Some(match remaining {
            1 => "You have become green slime.",
            2 => "You are turning into slime.",
            3 => "Your skin begins to peel.",
            4 => "Your limbs are getting oozy.",
            5 => "You are turning green.",
            _ => "You feel a bit slimy.",
        });
    }

    if flag == StatusFlags::CHOKING {
        return Some(match remaining {
            1 => "You die from choking.",
            2 => "Your face is turning blue!",
            3 => "You can't breathe!",
            4 => "You are gasping for air!",
            5 => "You are choking!",
            _ => "You feel a tightness in your throat.",
        });
    }

    //
    if remaining <= 2 {
        return Some(match flag {
            StatusFlags::BLIND => "Your vision is clearing.",
            StatusFlags::CONFUSED => "You feel less confused.",
            StatusFlags::STUNNED => "You feel steadier.",
            StatusFlags::HALLUCINATING => "Everything seems more real now.",
            StatusFlags::POISONED => "The poison seems to be wearing off.",
            _ => "You feel a change coming.",
        });
    }

    None
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CureMethod {
    Prayer,
    Unicorn,
    Stone2Flesh,
    FireCure,
    Potion,
    TimeExpiry,
}

///
pub fn can_cure_with(flag: StatusFlags, method: CureMethod) -> bool {
    match flag {
        StatusFlags::STONING => matches!(method, CureMethod::Stone2Flesh | CureMethod::Prayer),
        StatusFlags::SLIMED => matches!(method, CureMethod::FireCure | CureMethod::Prayer),
        StatusFlags::CHOKING => matches!(method, CureMethod::Prayer),
        StatusFlags::STRANGLED => matches!(method, CureMethod::Prayer),
        StatusFlags::POISONED => matches!(
            method,
            CureMethod::Unicorn | CureMethod::Potion | CureMethod::Prayer
        ),
        StatusFlags::BLIND => matches!(
            method,
            CureMethod::Unicorn | CureMethod::Potion | CureMethod::Prayer | CureMethod::TimeExpiry
        ),
        StatusFlags::CONFUSED => matches!(
            method,
            CureMethod::Unicorn | CureMethod::TimeExpiry | CureMethod::Prayer
        ),
        StatusFlags::STUNNED => matches!(method, CureMethod::TimeExpiry | CureMethod::Prayer),
        StatusFlags::HALLUCINATING => matches!(
            method,
            CureMethod::Unicorn | CureMethod::TimeExpiry | CureMethod::Prayer
        ),
        _ => matches!(method, CureMethod::Prayer | CureMethod::TimeExpiry),
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================

///
#[derive(Debug, Clone, Default)]
pub struct TimerStatistics {
    pub lethal_timers_started: u32,
    pub lethal_deaths: u32,
    pub debuffs_applied: u32,
    pub debuffs_cured: u32,
    pub buffs_applied: u32,
    pub transforms_started: u32,
    pub transforms_completed: u32,
    pub prayers_used: u32,
}

impl TimerStatistics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_timer_start(&mut self, category: TimerCategory) {
        match category {
            TimerCategory::Lethal => self.lethal_timers_started += 1,
            TimerCategory::Transform => self.transforms_started += 1,
            TimerCategory::Debuff => self.debuffs_applied += 1,
            TimerCategory::Buff => self.buffs_applied += 1,
            TimerCategory::Environmental => {}
        }
    }

    pub fn record_cure(&mut self, method: CureMethod) {
        self.debuffs_cured += 1;
        if matches!(method, CureMethod::Prayer) {
            self.prayers_used += 1;
        }
    }
}

// =============================================================================
// [v2.3.3
// =============================================================================
#[cfg(test)]
mod timeout_extended_tests {
    use super::*;

    #[test]
    fn test_timer_category() {
        assert_eq!(timer_category(StatusFlags::STONING), TimerCategory::Lethal);
        assert_eq!(
            timer_category(StatusFlags::SLIMED),
            TimerCategory::Transform
        );
        assert_eq!(timer_category(StatusFlags::BLIND), TimerCategory::Debuff);
    }

    #[test]
    fn test_timer_priority() {
        assert!(timer_priority(TimerCategory::Lethal) > timer_priority(TimerCategory::Debuff));
        assert!(timer_priority(TimerCategory::Debuff) > timer_priority(TimerCategory::Buff));
    }

    #[test]
    fn test_warning_stoning() {
        let msg = timer_warning_message(StatusFlags::STONING, 3);
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("stone"));
    }

    #[test]
    fn test_warning_none() {
        let msg = timer_warning_message(StatusFlags::STONING, 20);
        assert!(msg.is_none());
    }

    #[test]
    fn test_can_cure() {
        assert!(can_cure_with(StatusFlags::STONING, CureMethod::Stone2Flesh));
        assert!(!can_cure_with(StatusFlags::STONING, CureMethod::Potion));
        assert!(can_cure_with(StatusFlags::POISONED, CureMethod::Unicorn));
    }

    #[test]
    fn test_timer_stats() {
        let mut stats = TimerStatistics::new();
        stats.record_timer_start(TimerCategory::Lethal);
        stats.record_timer_start(TimerCategory::Debuff);
        stats.record_cure(CureMethod::Prayer);
        assert_eq!(stats.lethal_timers_started, 1);
        assert_eq!(stats.debuffs_applied, 1);
        assert_eq!(stats.prayers_used, 1);
    }
}
