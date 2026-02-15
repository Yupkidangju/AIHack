// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    ///
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct StatusFlags: u64 {
        const BLIND        = 0x00000001;
        const CONFUSED     = 0x00000002;
        const STUNNED      = 0x00000004;
        const HALLUCINATING = 0x00000008;
        const LEVITATING   = 0x00000010;
        const SLOW         = 0x00000020;
        const FAST         = 0x00000040;
        const POISONED     = 0x00000080;
        const SICK         = 0x00000100;
        const STONING      = 0x00000200;
        const STRANGLED    = 0x00000400;
        const SLIMED       = 0x00000800;
        const LYCANTHROPY  = 0x00001000;
        const SLEEPING     = 0x00002000;
        const PARALYZED    = 0x00004000;
        const REFLECTING   = 0x00008000;
        const VOMITING     = 0x00010000;
        const CHOKING      = 0x00020000;
        const PHASING      = 0x00040000;
        const SEARCHING    = 0x00080000;
        const HALLUC_RES   = 0x00100000;
        const SHOCK_RES    = 0x00200000;
        const FIRE_RES     = 0x00400000;
        const COLD_RES     = 0x00800000;
        const SLEEP_RES    = 0x01000000;
        const DISINT_RES   = 0x02000000;
        const POISON_RES   = 0x04000000;
        const ACID_RES     = 0x08000000;
        const DRAIN_RES    = 0x10000000;
        const DISPLACED    = 0x20000000;
        const HALF_DMG     = 0x40000000;
        const PROTECTION   = 0x80000000;
        const FOOD_POISONING = 0x0000000100000000;
        const FLYING       = 0x0000000200000000;
        const WATERWALKING = 0x0000000400000000;
        const SWIMMING     = 0x0000000800000000;
        const INFRAVISION  = 0x0000001000000000;
        const NIGHT_VISION = 0x0000002000000000;
        const BURDENED     = 0x0000004000000000;
        const STRESSED     = 0x0000008000000000;
        const STRAINED     = 0x0000010000000000;
        const OVERTAXED    = 0x0000020000000000;
        const OVERLOADED   = 0x0000040000000000;
    }
}

// =============================================================================
// [v2.0.0
// =============================================================================
//
//
//
//
//
// =============================================================================

///
///
///
///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusEffect {
    //
    ///
    Blind,
    ///
    Confused,
    ///
    Stunned,
    ///
    Hallucinating,
    ///
    Slow,
    ///
    Poisoned,
    /// 吏덈퀝
    Sick,
    ///
    Sleeping,
    /// 留덈퉬
    Paralyzed,
    ///
    Lycanthropy,
    ///
    Vomiting,
    ///
    FoodPoisoning,

    //
    ///
    Fast,
    ///
    Searching,

    //
    ///
    Reflecting,
    ///
    HallucResistance,
    ///
    ShockResistance,
    ///
    FireResistance,
    ///
    ColdResistance,
    ///
    SleepResistance,
    ///
    DisintegrationResistance,
    ///
    PoisonResistance,
    ///
    AcidResistance,
    ///
    DrainResistance,
    ///
    Displaced,
    ///
    HalfDamage,
    ///
    Protection,
    ///
    Infravision,
    ///
    NightVision,

    //
    ///
    Levitating,
    ///
    Flying,
    ///
    WaterWalking,
    ///
    Swimming,
    ///
    Phasing,

    //
    ///
    Burdened,
    ///
    Stressed,
    ///
    Strained,
    ///
    Overtaxed,
    ///
    Overloaded,

    //
    ///
    Stoning,
    ///
    Strangled,
    ///
    Slimed,
    ///
    Choking,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectCategory {
    ///
    TemporaryDebuff,
    ///
    TemporaryBuff,
    ///
    IntrinsicResistance,
    ///
    MovementMode,
    ///
    Encumbrance,
    ///
    LethalCondition,
}

impl StatusEffect {
    ///
    pub const fn to_flag(self) -> StatusFlags {
        //
        //
        StatusFlags::from_bits_truncate(self.to_bits())
    }

    ///
    pub const fn to_bits(self) -> u64 {
        match self {
            //
            Self::Blind => 0x00000001,
            Self::Confused => 0x00000002,
            Self::Stunned => 0x00000004,
            Self::Hallucinating => 0x00000008,
            Self::Slow => 0x00000020,
            Self::Poisoned => 0x00000080,
            Self::Sick => 0x00000100,
            Self::Sleeping => 0x00002000,
            Self::Paralyzed => 0x00004000,
            Self::Lycanthropy => 0x00001000,
            Self::Vomiting => 0x00010000,
            Self::FoodPoisoning => 0x0000000100000000,

            //
            Self::Fast => 0x00000040,
            Self::Searching => 0x00080000,

            //
            Self::Reflecting => 0x00008000,
            Self::HallucResistance => 0x00100000,
            Self::ShockResistance => 0x00200000,
            Self::FireResistance => 0x00400000,
            Self::ColdResistance => 0x00800000,
            Self::SleepResistance => 0x01000000,
            Self::DisintegrationResistance => 0x02000000,
            Self::PoisonResistance => 0x04000000,
            Self::AcidResistance => 0x08000000,
            Self::DrainResistance => 0x10000000,
            Self::Displaced => 0x20000000,
            Self::HalfDamage => 0x40000000,
            Self::Protection => 0x80000000,
            Self::Infravision => 0x0000001000000000,
            Self::NightVision => 0x0000002000000000,

            //
            Self::Levitating => 0x00000010,
            Self::Flying => 0x0000000200000000,
            Self::WaterWalking => 0x0000000400000000,
            Self::Swimming => 0x0000000800000000,
            Self::Phasing => 0x00040000,

            //
            Self::Burdened => 0x0000004000000000,
            Self::Stressed => 0x0000008000000000,
            Self::Strained => 0x0000010000000000,
            Self::Overtaxed => 0x0000020000000000,
            Self::Overloaded => 0x0000040000000000,

            //
            Self::Stoning => 0x00000200,
            Self::Strangled => 0x00000400,
            Self::Slimed => 0x00000800,
            Self::Choking => 0x00020000,
        }
    }

    ///
    pub fn from_flag(flag: StatusFlags) -> Option<Self> {
        Self::from_bits(flag.bits())
    }

    ///
    pub fn from_bits(bits: u64) -> Option<Self> {
        match bits {
            0x00000001 => Some(Self::Blind),
            0x00000002 => Some(Self::Confused),
            0x00000004 => Some(Self::Stunned),
            0x00000008 => Some(Self::Hallucinating),
            0x00000010 => Some(Self::Levitating),
            0x00000020 => Some(Self::Slow),
            0x00000040 => Some(Self::Fast),
            0x00000080 => Some(Self::Poisoned),
            0x00000100 => Some(Self::Sick),
            0x00000200 => Some(Self::Stoning),
            0x00000400 => Some(Self::Strangled),
            0x00000800 => Some(Self::Slimed),
            0x00001000 => Some(Self::Lycanthropy),
            0x00002000 => Some(Self::Sleeping),
            0x00004000 => Some(Self::Paralyzed),
            0x00008000 => Some(Self::Reflecting),
            0x00010000 => Some(Self::Vomiting),
            0x00020000 => Some(Self::Choking),
            0x00040000 => Some(Self::Phasing),
            0x00080000 => Some(Self::Searching),
            0x00100000 => Some(Self::HallucResistance),
            0x00200000 => Some(Self::ShockResistance),
            0x00400000 => Some(Self::FireResistance),
            0x00800000 => Some(Self::ColdResistance),
            0x01000000 => Some(Self::SleepResistance),
            0x02000000 => Some(Self::DisintegrationResistance),
            0x04000000 => Some(Self::PoisonResistance),
            0x08000000 => Some(Self::AcidResistance),
            0x10000000 => Some(Self::DrainResistance),
            0x20000000 => Some(Self::Displaced),
            0x40000000 => Some(Self::HalfDamage),
            0x80000000 => Some(Self::Protection),
            0x0000000100000000 => Some(Self::FoodPoisoning),
            0x0000000200000000 => Some(Self::Flying),
            0x0000000400000000 => Some(Self::WaterWalking),
            0x0000000800000000 => Some(Self::Swimming),
            0x0000001000000000 => Some(Self::Infravision),
            0x0000002000000000 => Some(Self::NightVision),
            0x0000004000000000 => Some(Self::Burdened),
            0x0000008000000000 => Some(Self::Stressed),
            0x0000010000000000 => Some(Self::Strained),
            0x0000020000000000 => Some(Self::Overtaxed),
            0x0000040000000000 => Some(Self::Overloaded),
            _ => None,
        }
    }

    ///
    pub const fn category(self) -> EffectCategory {
        match self {
            //
            Self::Stoning | Self::Strangled | Self::Slimed | Self::Choking => {
                EffectCategory::LethalCondition
            }
            //
            Self::Burdened
            | Self::Stressed
            | Self::Strained
            | Self::Overtaxed
            | Self::Overloaded => EffectCategory::Encumbrance,
            //
            Self::Levitating
            | Self::Flying
            | Self::WaterWalking
            | Self::Swimming
            | Self::Phasing => EffectCategory::MovementMode,
            //
            Self::Reflecting
            | Self::HallucResistance
            | Self::ShockResistance
            | Self::FireResistance
            | Self::ColdResistance
            | Self::SleepResistance
            | Self::DisintegrationResistance
            | Self::PoisonResistance
            | Self::AcidResistance
            | Self::DrainResistance
            | Self::Displaced
            | Self::HalfDamage
            | Self::Protection
            | Self::Infravision
            | Self::NightVision => EffectCategory::IntrinsicResistance,
            //
            Self::Fast | Self::Searching => EffectCategory::TemporaryBuff,
            //
            _ => EffectCategory::TemporaryDebuff,
        }
    }

    ///
    pub const fn is_dangerous(self) -> bool {
        matches!(
            self.category(),
            EffectCategory::LethalCondition | EffectCategory::TemporaryDebuff
        )
    }

    ///
    pub const fn is_timed(self) -> bool {
        matches!(
            self.category(),
            EffectCategory::TemporaryDebuff
                | EffectCategory::TemporaryBuff
                | EffectCategory::LethalCondition
        )
    }

    ///
    pub fn all() -> &'static [StatusEffect] {
        &[
            //
            Self::Blind,
            Self::Confused,
            Self::Stunned,
            Self::Hallucinating,
            Self::Slow,
            Self::Poisoned,
            Self::Sick,
            Self::Sleeping,
            Self::Paralyzed,
            Self::Lycanthropy,
            Self::Vomiting,
            Self::FoodPoisoning,
            //
            Self::Fast,
            Self::Searching,
            //
            Self::Reflecting,
            Self::HallucResistance,
            Self::ShockResistance,
            Self::FireResistance,
            Self::ColdResistance,
            Self::SleepResistance,
            Self::DisintegrationResistance,
            Self::PoisonResistance,
            Self::AcidResistance,
            Self::DrainResistance,
            Self::Displaced,
            Self::HalfDamage,
            Self::Protection,
            Self::Infravision,
            Self::NightVision,
            //
            Self::Levitating,
            Self::Flying,
            Self::WaterWalking,
            Self::Swimming,
            Self::Phasing,
            //
            Self::Burdened,
            Self::Stressed,
            Self::Strained,
            Self::Overtaxed,
            Self::Overloaded,
            //
            Self::Stoning,
            Self::Strangled,
            Self::Slimed,
            Self::Choking,
        ]
    }
}

impl EffectCategory {
    ///
    pub fn is_dangerous(&self) -> bool {
        matches!(self, Self::LethalCondition | Self::TemporaryDebuff)
    }

    ///
    pub fn is_timed(&self) -> bool {
        matches!(
            self,
            Self::TemporaryDebuff | Self::TemporaryBuff | Self::LethalCondition
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveStatus {
    pub flag: StatusFlags,
    pub remaining_turns: u32,
}

///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBundle {
    pub active: Vec<ActiveStatus>,
    pub permanent: StatusFlags,
}

impl StatusBundle {
    pub fn new() -> Self {
        Self {
            active: Vec::new(),
            permanent: StatusFlags::empty(),
        }
    }

    pub fn flags(&self) -> StatusFlags {
        let mut f = self.permanent;
        for s in &self.active {
            f |= s.flag;
        }
        f
    }

    pub fn add(&mut self, flag: StatusFlags, turns: u32) {
        if let Some(s) = self.active.iter_mut().find(|s| s.flag == flag) {
            s.remaining_turns += turns;
        } else {
            self.active.push(ActiveStatus {
                flag,
                remaining_turns: turns,
            });
        }
    }

    pub fn has(&self, flag: StatusFlags) -> bool {
        self.permanent.contains(flag) || self.active.iter().any(|s| s.flag == flag)
    }

    pub fn remove(&mut self, flag: StatusFlags) {
        self.permanent.remove(flag);
        self.active.retain(|s| s.flag != flag);
    }

    pub fn tick(&mut self) -> Vec<StatusFlags> {
        let mut expired = Vec::new();
        for s in &mut self.active {
            if s.remaining_turns > 0 {
                s.remaining_turns -= 1;
                if s.remaining_turns == 0 {
                    expired.push(s.flag);
                }
            }
        }
        self.active.retain(|s| s.remaining_turns > 0);
        expired
    }

    //

    ///
    pub fn has_effect(&self, effect: StatusEffect) -> bool {
        self.has(effect.to_flag())
    }

    ///
    pub fn add_effect(&mut self, effect: StatusEffect, turns: u32) {
        self.add(effect.to_flag(), turns);
    }

    ///
    pub fn remove_effect(&mut self, effect: StatusEffect) {
        self.remove(effect.to_flag());
    }

    ///
    pub fn set_permanent(&mut self, effect: StatusEffect) {
        self.permanent |= effect.to_flag();
    }

    ///
    pub fn clear_permanent(&mut self, effect: StatusEffect) {
        self.permanent.remove(effect.to_flag());
    }

    ///
    pub fn active_effects(&self) -> Vec<StatusEffect> {
        let combined = self.flags();
        StatusEffect::all()
            .iter()
            .copied()
            .filter(|e| combined.contains(e.to_flag()))
            .collect()
    }

    ///
    pub fn effects_by_category(&self, cat: EffectCategory) -> Vec<StatusEffect> {
        self.active_effects()
            .into_iter()
            .filter(|e| e.category() == cat)
            .collect()
    }

    ///
    pub fn tick_effects(&mut self) -> Vec<StatusEffect> {
        self.tick()
            .into_iter()
            .filter_map(|f| StatusEffect::from_flag(f))
            .collect()
    }

    // =========================================================================
    // [v2.3.0
    //
    //
    //
    //
    // =========================================================================

    ///
    ///
    ///
    ///
    pub fn make_confused(&mut self, turns: u32) -> Option<&'static str> {
        let was_confused = self.has(StatusFlags::CONFUSED);
        if turns == 0 {
            //
            self.remove(StatusFlags::CONFUSED);
            if was_confused {
                return Some("You feel less confused now.");
            }
        } else {
            //
            if was_confused {
                //
                if let Some(s) = self
                    .active
                    .iter_mut()
                    .find(|s| s.flag == StatusFlags::CONFUSED)
                {
                    s.remaining_turns = s.remaining_turns.saturating_add(turns);
                }
            } else {
                self.add(StatusFlags::CONFUSED, turns);
            }
        }
        None
    }

    ///
    ///
    pub fn make_stunned(&mut self, turns: u32) -> Option<&'static str> {
        let was_stunned = self.has(StatusFlags::STUNNED);
        if turns == 0 {
            self.remove(StatusFlags::STUNNED);
            if was_stunned {
                return Some("You feel a bit steadier.");
            }
        } else {
            if !was_stunned {
                self.add(StatusFlags::STUNNED, turns);
                return Some("You stagger...");
            } else {
                if let Some(s) = self
                    .active
                    .iter_mut()
                    .find(|s| s.flag == StatusFlags::STUNNED)
                {
                    s.remaining_turns = s.remaining_turns.saturating_add(turns);
                }
            }
        }
        None
    }

    ///
    ///
    ///
    pub fn make_sick(&mut self, turns: u32, sick_type: u8) -> Option<&'static str> {
        let was_sick = self.has(StatusFlags::SICK);
        if turns == 0 {
            //
            self.remove(StatusFlags::SICK);
            self.remove(StatusFlags::FOOD_POISONING);
            if was_sick {
                return Some("You feel cured. What a relief!");
            }
        } else {
            //
            if sick_type == 2 {
                //
                self.add(StatusFlags::FOOD_POISONING, turns);
            }
            if !was_sick {
                self.add(StatusFlags::SICK, turns);
                return Some("You feel deathly sick.");
            } else {
                //
                if let Some(s) = self.active.iter_mut().find(|s| s.flag == StatusFlags::SICK) {
                    //
                    if turns < s.remaining_turns / 2 {
                        s.remaining_turns = turns;
                        return Some("You feel much worse.");
                    } else {
                        s.remaining_turns = turns;
                        return Some("You feel even worse.");
                    }
                }
            }
        }
        None
    }

    ///
    ///
    pub fn make_blinded(&mut self, turns: u32) -> Option<&'static str> {
        let was_blind = self.has(StatusFlags::BLIND);
        if turns == 0 {
            self.remove(StatusFlags::BLIND);
            if was_blind {
                return Some("You can see again.");
            }
        } else {
            if !was_blind {
                self.add(StatusFlags::BLIND, turns);
                return Some("A cloud of darkness falls upon you.");
            } else {
                //
                if let Some(s) = self
                    .active
                    .iter_mut()
                    .find(|s| s.flag == StatusFlags::BLIND)
                {
                    s.remaining_turns = s.remaining_turns.saturating_add(turns);
                }
            }
        }
        None
    }

    ///
    ///
    ///
    pub fn make_hallucinated(&mut self, turns: u32) -> (bool, Option<&'static str>) {
        let was_halluc = self.has(StatusFlags::HALLUCINATING);
        if turns == 0 {
            self.remove(StatusFlags::HALLUCINATING);
            if was_halluc {
                return (true, Some("Everything looks SO boring now."));
            }
        } else {
            if !was_halluc {
                self.add(StatusFlags::HALLUCINATING, turns);
                return (true, Some("Oh wow! Everything looks so cosmic!"));
            } else {
                //
                if let Some(s) = self
                    .active
                    .iter_mut()
                    .find(|s| s.flag == StatusFlags::HALLUCINATING)
                {
                    s.remaining_turns = s.remaining_turns.saturating_add(turns);
                }
            }
        }
        (false, None)
    }

    ///
    ///
    pub fn make_stoned(&mut self, turns: u32) -> Option<&'static str> {
        let was_stoned = self.has(StatusFlags::STONING);
        if turns == 0 {
            self.remove(StatusFlags::STONING);
            if was_stoned {
                return Some("You feel limber!");
            }
        } else {
            if !was_stoned {
                self.add(StatusFlags::STONING, turns);
                return Some("You are slowing down.");
            }
        }
        None
    }

    ///
    ///
    pub fn make_slimed(&mut self, turns: u32) -> Option<&'static str> {
        let was_slimed = self.has(StatusFlags::SLIMED);
        if turns == 0 {
            self.remove(StatusFlags::SLIMED);
            if was_slimed {
                return Some("The slime is gone.");
            }
        } else {
            if !was_slimed {
                self.add(StatusFlags::SLIMED, turns);
                return Some("You begin to feel slimy.");
            }
        }
        None
    }

    ///
    ///
    pub fn make_vomiting(&mut self, turns: u32) -> Option<&'static str> {
        let was_vomiting = self.has(StatusFlags::SICK);
        if turns == 0 {
            //
            //
            self.remove(StatusFlags::FOOD_POISONING);
            if was_vomiting {
                return Some("You feel much less nauseated now.");
            }
        } else {
            self.add(StatusFlags::FOOD_POISONING, turns);
        }
        None
    }

    ///
    ///
    pub fn make_deaf(&mut self, turns: u32) -> Option<&'static str> {
        //
        //
        //
        if turns == 0 {
            return Some("You can hear again.");
        } else {
            return Some("You are unable to hear anything.");
        }
    }

    ///
    ///
    pub fn make_glib(&mut self, turns: u32) {
        //
        //
        let _ = turns;
    }

    // =========================================================================
    // [v2.3.0
    // =========================================================================

    ///
    ///
    pub fn clamp_timeout(val: u32) -> u32 {
        val.min(0x00FF_FFFF)
    }

    ///
    pub fn timeout_incr(old: u32, incr: i32) -> u32 {
        let new_val = (old as i64 + incr as i64).max(0) as u32;
        Self::clamp_timeout(new_val)
    }

    ///
    pub fn remaining_turns(&self, flag: StatusFlags) -> u32 {
        self.active
            .iter()
            .find(|s| s.flag == flag)
            .map(|s| s.remaining_turns)
            .unwrap_or(0)
    }

    ///
    pub fn set_timeout(&mut self, flag: StatusFlags, turns: u32) {
        let clamped = Self::clamp_timeout(turns);
        if clamped == 0 {
            self.remove(flag);
        } else if let Some(s) = self.active.iter_mut().find(|s| s.flag == flag) {
            s.remaining_turns = clamped;
        } else {
            self.add(flag, clamped);
        }
    }

    ///
    pub fn incr_timeout(&mut self, flag: StatusFlags, incr: i32) {
        let old = self.remaining_turns(flag);
        let new_val = Self::timeout_incr(old, incr);
        self.set_timeout(flag, new_val);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Swallowed {
    pub by: legion::Entity,
    pub digestion_turns: u32,
}

// =============================================================================
//
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    //
    #[test]
    fn test_status_bundle_basic() {
        let mut sb = StatusBundle::new();
        sb.add(StatusFlags::BLIND, 10);
        assert!(sb.has(StatusFlags::BLIND));
        assert!(!sb.has(StatusFlags::CONFUSED));
        sb.remove(StatusFlags::BLIND);
        assert!(!sb.has(StatusFlags::BLIND));
    }

    #[test]
    fn test_status_bundle_tick() {
        let mut sb = StatusBundle::new();
        sb.add(StatusFlags::STUNNED, 2);
        let expired = sb.tick();
        assert!(expired.is_empty());
        let expired = sb.tick();
        assert!(expired.contains(&StatusFlags::STUNNED));
    }

    //
    #[test]
    fn test_status_effect_roundtrip() {
        //
        for &effect in StatusEffect::all() {
            let flag = effect.to_flag();
            let restored = StatusEffect::from_flag(flag);
            assert_eq!(restored, Some(effect), "roundtrip failed: {:?}", effect);
        }
    }

    #[test]
    fn test_status_effect_categories() {
        //
        assert_eq!(
            StatusEffect::Stoning.category(),
            EffectCategory::LethalCondition
        );
        assert_eq!(
            StatusEffect::Choking.category(),
            EffectCategory::LethalCondition
        );
        assert!(StatusEffect::Stoning.is_dangerous());
        assert!(StatusEffect::Stoning.is_timed());

        //
        assert_eq!(
            StatusEffect::Blind.category(),
            EffectCategory::TemporaryDebuff
        );
        assert!(StatusEffect::Blind.is_dangerous());

        //
        assert_eq!(StatusEffect::Fast.category(), EffectCategory::TemporaryBuff);
        assert!(!StatusEffect::Fast.is_dangerous());

        //
        assert_eq!(
            StatusEffect::FireResistance.category(),
            EffectCategory::IntrinsicResistance
        );
        assert!(!StatusEffect::FireResistance.is_dangerous());
        assert!(!StatusEffect::FireResistance.is_timed());

        //
        assert_eq!(
            StatusEffect::Flying.category(),
            EffectCategory::MovementMode
        );

        //
        assert_eq!(
            StatusEffect::Burdened.category(),
            EffectCategory::Encumbrance
        );
    }

    #[test]
    fn test_bundle_effect_api() {
        let mut sb = StatusBundle::new();
        sb.add_effect(StatusEffect::Blind, 5);
        sb.set_permanent(StatusEffect::FireResistance);

        assert!(sb.has_effect(StatusEffect::Blind));
        assert!(sb.has_effect(StatusEffect::FireResistance));
        assert!(!sb.has_effect(StatusEffect::Confused));

        let effects = sb.active_effects();
        assert!(effects.contains(&StatusEffect::Blind));
        assert!(effects.contains(&StatusEffect::FireResistance));

        let debuffs = sb.effects_by_category(EffectCategory::TemporaryDebuff);
        assert!(debuffs.contains(&StatusEffect::Blind));
        assert!(!debuffs.contains(&StatusEffect::FireResistance));

        sb.remove_effect(StatusEffect::Blind);
        assert!(!sb.has_effect(StatusEffect::Blind));
    }

    #[test]
    fn test_tick_effects() {
        let mut sb = StatusBundle::new();
        sb.add_effect(StatusEffect::Stunned, 1);
        let expired = sb.tick_effects();
        assert!(expired.contains(&StatusEffect::Stunned));
    }
}
