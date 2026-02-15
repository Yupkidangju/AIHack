// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// =============================================================================
// [v2.0.0
// =============================================================================
//
//
//
//
//
//
//
//
//
//
//
// =============================================================================

use super::monster::{MonsterFlags1, MonsterFlags2, MonsterFlags3, MonsterTemplate};

///
///
///
///
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MonsterCapability {
    //
    ///
    Fly,
    ///
    Swim,
    ///
    WallWalk,
    ///
    Cling,
    ///
    Tunnel,
    ///
    NeedPick,
    ///
    Amphibious,

    //
    ///
    Amorphous,
    ///
    Unsolid,
    ///
    Humanoid,
    ///
    Animal,
    ///
    Slithy,
    ///
    ThickHide,
    ///
    Oviparous,
    ///
    BigMonst,

    //
    ///
    NoEyes,
    ///
    NoHands,
    ///
    NoLimbs,
    ///
    NoHead,
    ///
    Mindless,
    ///
    SeeInvis,
    ///
    Infravision,
    ///
    Infravisible,

    //
    ///
    Breathless,
    ///
    NoTake,
    ///
    Conceal,
    ///
    Hide,
    ///
    Regen,
    ///
    Teleport,
    ///
    TeleportControl,
    ///
    Displaces,
    ///
    Close,
    ///
    WaitForYou,

    //
    ///
    Carnivore,
    ///
    Herbivore,
    ///
    Omnivore,
    ///
    Metallivore,

    //
    ///
    Acid,
    ///
    Poisonous,

    //
    ///
    Undead,
    ///
    Were,
    ///
    Human,
    ///
    Elf,
    ///
    Dwarf,
    ///
    Gnome,
    ///
    Orc,
    ///
    Demon,
    ///
    Merc,
    ///
    Lord,
    ///
    Prince,
    ///
    Minion,
    ///
    Giant,
    ///
    Shapeshifter,

    //
    ///
    Male,
    ///
    Female,
    ///
    Neuter,

    //
    ///
    ProperName,
    ///
    Hostile,
    ///
    Peaceful,
    ///
    Domestic,
    ///
    Wander,
    ///
    Stalk,
    ///
    Nasty,
    ///
    Strong,
    ///
    RockThrow,
    ///
    Greedy,
    ///
    Jewels,
    ///
    Collect,
    ///
    MagicCollect,
    ///
    NoPoly,

    //
    ///
    WantsAmulet,
    ///
    WantsBell,
    ///
    WantsBook,
    ///
    WantsCandelabrum,
    ///
    WantsArtifact,
}

impl MonsterCapability {
    ///
    pub fn is_set_in(&self, template: &MonsterTemplate) -> bool {
        match self {
            //
            Self::Fly => template.has_flag1(MonsterFlags1::FLY),
            Self::Swim => template.has_flag1(MonsterFlags1::SWIM),
            Self::Amorphous => template.has_flag1(MonsterFlags1::AMORPHOUS),
            Self::WallWalk => template.has_flag1(MonsterFlags1::WALLWALK),
            Self::Cling => template.has_flag1(MonsterFlags1::CLING),
            Self::Tunnel => template.has_flag1(MonsterFlags1::TUNNEL),
            Self::NeedPick => template.has_flag1(MonsterFlags1::NEEDPICK),
            Self::Conceal => template.has_flag1(MonsterFlags1::CONCEAL),
            Self::Hide => template.has_flag1(MonsterFlags1::HIDE),
            Self::Amphibious => template.has_flag1(MonsterFlags1::AMPHIBIOUS),
            Self::Breathless => template.has_flag1(MonsterFlags1::BREATHLESS),
            Self::NoTake => template.has_flag1(MonsterFlags1::NOTAKE),
            Self::NoEyes => template.has_flag1(MonsterFlags1::NOEYES),
            Self::NoHands => template.has_flag1(MonsterFlags1::NOHANDS),
            Self::NoLimbs => template.has_flag1(MonsterFlags1::NOLIMBS),
            Self::NoHead => template.has_flag1(MonsterFlags1::NOHEAD),
            Self::Mindless => template.has_flag1(MonsterFlags1::MINDLESS),
            Self::Humanoid => template.has_flag1(MonsterFlags1::HUMANOID),
            Self::Animal => template.has_flag1(MonsterFlags1::ANIMAL),
            Self::Slithy => template.has_flag1(MonsterFlags1::SLITHY),
            Self::Unsolid => template.has_flag1(MonsterFlags1::UNSOLID),
            Self::ThickHide => template.has_flag1(MonsterFlags1::THICK_HIDE),
            Self::Oviparous => template.has_flag1(MonsterFlags1::OVIPAROUS),
            Self::Regen => template.has_flag1(MonsterFlags1::REGEN),
            Self::SeeInvis => template.has_flag1(MonsterFlags1::SEE_INVIS),
            Self::Teleport => template.has_flag1(MonsterFlags1::TPORT),
            Self::TeleportControl => template.has_flag1(MonsterFlags1::TPORT_CNTRL),
            Self::Acid => template.has_flag1(MonsterFlags1::ACID),
            Self::Poisonous => template.has_flag1(MonsterFlags1::POIS),
            Self::Carnivore => template.has_flag1(MonsterFlags1::CARNIVORE),
            Self::Herbivore => template.has_flag1(MonsterFlags1::HERBIVORE),
            Self::Omnivore => template.has_flag1(MonsterFlags1::OMNIVORE),
            Self::Metallivore => template.has_flag1(MonsterFlags1::METALLIVORE),

            //
            Self::NoPoly => template.has_flag2(MonsterFlags2::NOPOLY),
            Self::Undead => template.has_flag2(MonsterFlags2::UNDEAD),
            Self::Were => template.has_flag2(MonsterFlags2::WERE),
            Self::Human => template.has_flag2(MonsterFlags2::HUMAN),
            Self::Elf => template.has_flag2(MonsterFlags2::ELF),
            Self::Dwarf => template.has_flag2(MonsterFlags2::DWARF),
            Self::Gnome => template.has_flag2(MonsterFlags2::GNOME),
            Self::Orc => template.has_flag2(MonsterFlags2::ORC),
            Self::Demon => template.has_flag2(MonsterFlags2::DEMON),
            Self::Merc => template.has_flag2(MonsterFlags2::MERC),
            Self::Lord => template.has_flag2(MonsterFlags2::LORD),
            Self::Prince => template.has_flag2(MonsterFlags2::PRINCE),
            Self::Minion => template.has_flag2(MonsterFlags2::MINION),
            Self::Giant => template.has_flag2(MonsterFlags2::GIANT),
            Self::Shapeshifter => template.has_flag2(MonsterFlags2::SHAPESHIFTER),
            Self::Male => template.has_flag2(MonsterFlags2::MALE),
            Self::Female => template.has_flag2(MonsterFlags2::FEMALE),
            Self::Neuter => template.has_flag2(MonsterFlags2::NEUTER),
            Self::ProperName => template.has_flag2(MonsterFlags2::PNAME),
            Self::Hostile => template.has_flag2(MonsterFlags2::HOSTILE),
            Self::Peaceful => template.has_flag2(MonsterFlags2::PEACEFUL),
            Self::Domestic => template.has_flag2(MonsterFlags2::DOMESTIC),
            Self::Wander => template.has_flag2(MonsterFlags2::WANDER),
            Self::Stalk => template.has_flag2(MonsterFlags2::STALK),
            Self::Nasty => template.has_flag2(MonsterFlags2::NASTY),
            Self::Strong => template.has_flag2(MonsterFlags2::STRONG),
            Self::RockThrow => template.has_flag2(MonsterFlags2::ROCKTHROW),
            Self::Greedy => template.has_flag2(MonsterFlags2::GREEDY),
            Self::Jewels => template.has_flag2(MonsterFlags2::JEWELS),
            Self::Collect => template.has_flag2(MonsterFlags2::COLLECT),
            Self::MagicCollect => template.has_flag2(MonsterFlags2::MAGIC),

            //
            Self::WantsAmulet => template.has_flag3(MonsterFlags3::WANTSAMUL),
            Self::WantsBell => template.has_flag3(MonsterFlags3::WANTSBELL),
            Self::WantsBook => template.has_flag3(MonsterFlags3::WANTSBOOK),
            Self::WantsCandelabrum => template.has_flag3(MonsterFlags3::WANTSCAND),
            Self::WantsArtifact => template.has_flag3(MonsterFlags3::WANTSARTI),
            Self::WaitForYou => template.has_flag3(MonsterFlags3::WAITFORU),
            Self::Close => template.has_flag3(MonsterFlags3::CLOSE),
            Self::Infravision => template.has_flag3(MonsterFlags3::INFRAVISION),
            Self::Infravisible => template.has_flag3(MonsterFlags3::INFRAVISIBLE),
            Self::Displaces => template.has_flag3(MonsterFlags3::DISPLACES),

            //
            Self::BigMonst => template.msize >= 4,
        }
    }

    ///
    pub fn category(&self) -> &str {
        match self {
            Self::Fly
            | Self::Swim
            | Self::WallWalk
            | Self::Cling
            | Self::Tunnel
            | Self::NeedPick
            | Self::Amphibious => "",
            Self::Amorphous
            | Self::Unsolid
            | Self::Humanoid
            | Self::Animal
            | Self::Slithy
            | Self::ThickHide
            | Self::Oviparous
            | Self::BigMonst => "",
            Self::NoEyes
            | Self::NoHands
            | Self::NoLimbs
            | Self::NoHead
            | Self::Mindless
            | Self::SeeInvis
            | Self::Infravision
            | Self::Infravisible => "Sense",
            Self::Breathless
            | Self::NoTake
            | Self::Conceal
            | Self::Hide
            | Self::Regen
            | Self::Teleport
            | Self::TeleportControl
            | Self::Displaces
            | Self::Close
            | Self::WaitForYou => "",
            Self::Carnivore | Self::Herbivore | Self::Omnivore | Self::Metallivore => "",
            Self::Acid | Self::Poisonous => "",
            Self::Undead
            | Self::Were
            | Self::Human
            | Self::Elf
            | Self::Dwarf
            | Self::Gnome
            | Self::Orc
            | Self::Demon
            | Self::Merc
            | Self::Lord
            | Self::Prince
            | Self::Minion
            | Self::Giant
            | Self::Shapeshifter => "Race",
            Self::Male | Self::Female | Self::Neuter => "",
            Self::ProperName
            | Self::Hostile
            | Self::Peaceful
            | Self::Domestic
            | Self::Wander
            | Self::Stalk
            | Self::Nasty
            | Self::Strong
            | Self::RockThrow
            | Self::Greedy
            | Self::Jewels
            | Self::Collect
            | Self::MagicCollect
            | Self::NoPoly => "",
            Self::WantsAmulet
            | Self::WantsBell
            | Self::WantsBook
            | Self::WantsCandelabrum
            | Self::WantsArtifact => "원하는 것",
        }
    }
}

/// MonsterTemplate에 대한 편의 쿼리 메서드 추가
impl MonsterTemplate {
    /// 편의성 능력 쿼리 — has_flag1/2/3 대신 사용
    ///
    /// 예: `template.has_capability(MonsterCapability::Fly)`
    pub fn has_capability(&self, cap: MonsterCapability) -> bool {
        cap.is_set_in(self)
    }
}

// =============================================================================
//
// =============================================================================

use super::status::StatusFlags;

///
///
/// StatusFlags에는 영구 효과(내성)와 일시 효과(독, 혼란)가 혼재.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCategory {
    /// 일시적 상태 이상 (디버프, 일정 턴 후 해제)
    TemporaryDebuff,
    /// 일시적 버프 (디버프, 일정 턴 후 해제)
    TemporaryBuff,
    ///
    IntrinsicResistance,
    ///
    MovementMode,
    /// 하중 상태 — 무게 기반
    Encumbrance,
    /// 치명적 상태 — 석화, 교살, 질식 (즉사 위험)
    LethalCondition,
}

impl StatusCategory {
    ///
    pub fn of(flag: StatusFlags) -> Self {
        // 치명적 상태
        if flag.intersects(
            StatusFlags::STONING
                | StatusFlags::STRANGLED
                | StatusFlags::SLIMED
                | StatusFlags::CHOKING,
        ) {
            return Self::LethalCondition;
        }

        // 하중 상태
        if flag.intersects(
            StatusFlags::BURDENED
                | StatusFlags::STRESSED
                | StatusFlags::STRAINED
                | StatusFlags::OVERTAXED
                | StatusFlags::OVERLOADED,
        ) {
            return Self::Encumbrance;
        }

        // 이동 모드
        if flag.intersects(
            StatusFlags::LEVITATING
                | StatusFlags::FLYING
                | StatusFlags::WATERWALKING
                | StatusFlags::SWIMMING
                | StatusFlags::PHASING,
        ) {
            return Self::MovementMode;
        }

        // 내재 저항
        if flag.intersects(
            StatusFlags::REFLECTING
                | StatusFlags::HALLUC_RES
                | StatusFlags::SHOCK_RES
                | StatusFlags::FIRE_RES
                | StatusFlags::COLD_RES
                | StatusFlags::SLEEP_RES
                | StatusFlags::DISINT_RES
                | StatusFlags::POISON_RES
                | StatusFlags::ACID_RES
                | StatusFlags::DRAIN_RES
                | StatusFlags::DISPLACED
                | StatusFlags::HALF_DMG
                | StatusFlags::PROTECTION
                | StatusFlags::INFRAVISION
                | StatusFlags::NIGHT_VISION,
        ) {
            return Self::IntrinsicResistance;
        }

        // 일시적 버프
        if flag.intersects(StatusFlags::FAST | StatusFlags::SEARCHING) {
            return Self::TemporaryBuff;
        }

        //
        Self::TemporaryDebuff
    }

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
