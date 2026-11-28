use serde::{Deserialize, Serialize};

use crate::domain::{combat::AttackProfile, status::Resistances};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterAiKind {
    Wander,
    ChaseVisiblePlayer,
    Stationary,
    RangedOnSight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MonsterPassive {
    ParalyzeOnMelee,
    PoisonOnMelee,
    Regenerate,
    FireOnMelee,
    FireBreath,
    StealGold,
    StealItem,
    RustMetal,
    SplitOnHit,
    GazeParalyze,
    SummonAllies,
    ColdBreath,
    LightningBreath,
    SleepOnMelee,
    DrainLife,
}

macro_rules! define_monster_kinds {
    (
        $(
            $variant:ident => {
                id: $toml_id:expr,
                ai: $ai:ident,
                diff: $diff:expr,
                atk: $atk:expr,
                v1: $v1:expr $(,)?
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum MonsterKind {
            $($variant,)*
        }

        impl MonsterKind {
            pub const ALL: &'static [Self] = &[$(Self::$variant,)*];

            pub const fn toml_id(self) -> &'static str {
                match self {
                    $(Self::$variant => $toml_id,)*
                }
            }

            pub fn from_toml_id(id: &str) -> Option<Self> {
                match id {
                    $($toml_id => Some(Self::$variant),)*
                    _ => None,
                }
            }

            pub const fn ai_kind(self) -> MonsterAiKind {
                match self {
                    $(Self::$variant => MonsterAiKind::$ai,)*
                }
            }

            pub const fn difficulty(self) -> u8 {
                match self {
                    $(Self::$variant => $diff,)*
                }
            }

            pub const fn natural_attack_name(self) -> &'static str {
                match self {
                    $(Self::$variant => $atk,)*
                }
            }

            pub const fn is_legacy_v1_allowed(self) -> bool {
                match self {
                    $(Self::$variant => $v1,)*
                }
            }
        }
    };
}

define_monster_kinds! {
    // --- Existing 30 monsters ---
    Jackal => { id: "monster.jackal", ai: Wander, diff: 1, atk: "bite", v1: true },
    Goblin => { id: "monster.goblin", ai: ChaseVisiblePlayer, diff: 2, atk: "short sword", v1: true },
    FloatingEye => { id: "monster.floating_eye", ai: Stationary, diff: 5, atk: "gaze", v1: true },
    Kobold => { id: "monster.kobold", ai: ChaseVisiblePlayer, diff: 2, atk: "spear", v1: true },
    Orc => { id: "monster.orc", ai: ChaseVisiblePlayer, diff: 3, atk: "scimitar", v1: true },
    Ogre => { id: "monster.ogre", ai: ChaseVisiblePlayer, diff: 5, atk: "club", v1: true },
    GiantSpider => { id: "monster.giant_spider", ai: ChaseVisiblePlayer, diff: 3, atk: "fangs", v1: true },
    GnollArcher => { id: "monster.gnoll_archer", ai: RangedOnSight, diff: 4, atk: "arrow", v1: true },
    Troll => { id: "monster.troll", ai: ChaseVisiblePlayer, diff: 6, atk: "claw", v1: true },
    FireAnt => { id: "monster.fire_ant", ai: ChaseVisiblePlayer, diff: 5, atk: "fire bite", v1: true },
    Minotaur => { id: "monster.minotaur", ai: ChaseVisiblePlayer, diff: 7, atk: "gore", v1: true },
    Dragon => { id: "monster.dragon", ai: RangedOnSight, diff: 8, atk: "bite", v1: true },
    Bat => { id: "monster.bat", ai: Wander, diff: 2, atk: "bite", v1: true },
    Kitten => { id: "monster.kitten", ai: ChaseVisiblePlayer, diff: 2, atk: "bite", v1: false },
    Housecat => { id: "monster.housecat", ai: ChaseVisiblePlayer, diff: 4, atk: "bite", v1: false },
    LittleDog => { id: "monster.little_dog", ai: ChaseVisiblePlayer, diff: 2, atk: "bite", v1: false },
    Dog => { id: "monster.dog", ai: ChaseVisiblePlayer, diff: 4, atk: "bite", v1: false },
    Pony => { id: "monster.pony", ai: ChaseVisiblePlayer, diff: 3, atk: "kick", v1: false },
    Warhorse => { id: "monster.warhorse", ai: ChaseVisiblePlayer, diff: 6, atk: "kick", v1: false },
    Shopkeeper => { id: "monster.shopkeeper", ai: ChaseVisiblePlayer, diff: 10, atk: "weapon", v1: false },
    TemplePriest => { id: "monster.temple_priest", ai: ChaseVisiblePlayer, diff: 8, atk: "weapon", v1: false },
    Oracle => { id: "monster.oracle", ai: Stationary, diff: 6, atk: "touch", v1: false },
    Watchman => { id: "monster.watchman", ai: ChaseVisiblePlayer, diff: 6, atk: "weapon", v1: false },
    Leprechaun => { id: "monster.leprechaun", ai: ChaseVisiblePlayer, diff: 4, atk: "claw", v1: false },
    Nymph => { id: "monster.nymph", ai: ChaseVisiblePlayer, diff: 5, atk: "touch", v1: false },
    RustMonster => { id: "monster.rust_monster", ai: ChaseVisiblePlayer, diff: 5, atk: "bite", v1: false },
    BlackPudding => { id: "monster.black_pudding", ai: ChaseVisiblePlayer, diff: 6, atk: "engulf", v1: false },
    Dwarf => { id: "monster.dwarf", ai: ChaseVisiblePlayer, diff: 4, atk: "axe", v1: false },
    Medusa => { id: "monster.medusa", ai: RangedOnSight, diff: 8, atk: "snakes", v1: false },
    OrcCaptain => { id: "monster.orc_captain", ai: ChaseVisiblePlayer, diff: 5, atk: "axe", v1: false },

    // --- P12 55 new monsters (85 total) ---
    GridBug => { id: "monster.grid_bug", ai: Wander, diff: 1, atk: "zap", v1: false },
    Newt => { id: "monster.newt", ai: Wander, diff: 1, atk: "bite", v1: false },
    Gecko => { id: "monster.gecko", ai: Wander, diff: 1, atk: "bite", v1: false },
    Lizard => { id: "monster.lizard", ai: Wander, diff: 2, atk: "bite", v1: false },
    SewerRat => { id: "monster.sewer_rat", ai: Wander, diff: 1, atk: "bite", v1: false },
    GiantRat => { id: "monster.giant_rat", ai: ChaseVisiblePlayer, diff: 2, atk: "bite", v1: false },
    CaveSpider => { id: "monster.cave_spider", ai: ChaseVisiblePlayer, diff: 2, atk: "fangs", v1: false },
    Centipede => { id: "monster.centipede", ai: ChaseVisiblePlayer, diff: 3, atk: "bite", v1: false },
    GiantBeetle => { id: "monster.giant_beetle", ai: ChaseVisiblePlayer, diff: 4, atk: "bite", v1: false },
    KillerBee => { id: "monster.killer_bee", ai: ChaseVisiblePlayer, diff: 4, atk: "sting", v1: false },
    QueenBee => { id: "monster.queen_bee", ai: ChaseVisiblePlayer, diff: 6, atk: "sting", v1: false },
    GiantBat => { id: "monster.giant_bat", ai: Wander, diff: 3, atk: "bite", v1: false },
    VampireBat => { id: "monster.vampire_bat", ai: ChaseVisiblePlayer, diff: 5, atk: "bite", v1: false },
    Fox => { id: "monster.fox", ai: ChaseVisiblePlayer, diff: 2, atk: "bite", v1: false },
    Wolf => { id: "monster.wolf", ai: ChaseVisiblePlayer, diff: 4, atk: "bite", v1: false },

    Gnome => { id: "monster.gnome", ai: ChaseVisiblePlayer, diff: 2, atk: "short sword", v1: false },
    GnomeLord => { id: "monster.gnome_lord", ai: ChaseVisiblePlayer, diff: 4, atk: "spear", v1: false },
    Hobgoblin => { id: "monster.hobgoblin", ai: ChaseVisiblePlayer, diff: 3, atk: "club", v1: false },
    Bugbear => { id: "monster.bugbear", ai: ChaseVisiblePlayer, diff: 5, atk: "club", v1: false },
    HillGiant => { id: "monster.hill_giant", ai: ChaseVisiblePlayer, diff: 7, atk: "club", v1: false },
    StoneGiant => { id: "monster.stone_giant", ai: RangedOnSight, diff: 8, atk: "boulder", v1: false },
    FrostGiant => { id: "monster.frost_giant", ai: RangedOnSight, diff: 9, atk: "axe", v1: false },
    FireGiant => { id: "monster.fire_giant", ai: RangedOnSight, diff: 9, atk: "sword", v1: false },
    Ettin => { id: "monster.ettin", ai: ChaseVisiblePlayer, diff: 9, atk: "club", v1: false },
    Centaur => { id: "monster.centaur", ai: RangedOnSight, diff: 5, atk: "kick", v1: false },
    Gargoyle => { id: "monster.gargoyle", ai: ChaseVisiblePlayer, diff: 6, atk: "claw", v1: false },
    TrollIce => { id: "monster.ice_troll", ai: ChaseVisiblePlayer, diff: 7, atk: "claw", v1: false },
    TrollRock => { id: "monster.rock_troll", ai: ChaseVisiblePlayer, diff: 7, atk: "claw", v1: false },

    Zombie => { id: "monster.zombie", ai: ChaseVisiblePlayer, diff: 2, atk: "claw", v1: false },
    Ghoul => { id: "monster.ghoul", ai: ChaseVisiblePlayer, diff: 4, atk: "claw", v1: false },
    Mummy => { id: "monster.mummy", ai: ChaseVisiblePlayer, diff: 5, atk: "claw", v1: false },
    Wight => { id: "monster.wight", ai: ChaseVisiblePlayer, diff: 6, atk: "claw", v1: false },
    Wraith => { id: "monster.wraith", ai: ChaseVisiblePlayer, diff: 7, atk: "touch", v1: false },
    Vampire => { id: "monster.vampire", ai: ChaseVisiblePlayer, diff: 8, atk: "bite", v1: false },
    VampireLord => { id: "monster.vampire_lord", ai: ChaseVisiblePlayer, diff: 10, atk: "bite", v1: false },
    Lich => { id: "monster.lich", ai: RangedOnSight, diff: 11, atk: "touch", v1: false },
    DemiLich => { id: "monster.demilich", ai: RangedOnSight, diff: 12, atk: "touch", v1: false },
    Skeleton => { id: "monster.skeleton", ai: ChaseVisiblePlayer, diff: 4, atk: "weapon", v1: false },
    BarrowWight => { id: "monster.barrow_wight", ai: ChaseVisiblePlayer, diff: 6, atk: "claw", v1: false },

    BlueJelly => { id: "monster.blue_jelly", ai: Stationary, diff: 3, atk: "touch", v1: false },
    SpottedJelly => { id: "monster.spotted_jelly", ai: Stationary, diff: 4, atk: "touch", v1: false },
    OchreJelly => { id: "monster.ochre_jelly", ai: ChaseVisiblePlayer, diff: 5, atk: "engulf", v1: false },
    GelatinousCube => { id: "monster.gelatinous_cube", ai: ChaseVisiblePlayer, diff: 5, atk: "engulf", v1: false },
    GrayOoze => { id: "monster.gray_ooze", ai: ChaseVisiblePlayer, diff: 3, atk: "engulf", v1: false },
    BrownMold => { id: "monster.brown_mold", ai: Stationary, diff: 2, atk: "touch", v1: false },
    RedMold => { id: "monster.red_mold", ai: Stationary, diff: 2, atk: "touch", v1: false },
    Homunculus => { id: "monster.homunculus", ai: ChaseVisiblePlayer, diff: 3, atk: "bite", v1: false },
    Imp => { id: "monster.imp", ai: ChaseVisiblePlayer, diff: 4, atk: "claw", v1: false },
    MindFlayer => { id: "monster.mind_flayer", ai: RangedOnSight, diff: 10, atk: "tentacles", v1: false },
    FireVortex => { id: "monster.fire_vortex", ai: ChaseVisiblePlayer, diff: 6, atk: "engulf", v1: false },
    IceVortex => { id: "monster.ice_vortex", ai: ChaseVisiblePlayer, diff: 6, atk: "engulf", v1: false },
    WhiteDragon => { id: "monster.white_dragon", ai: RangedOnSight, diff: 9, atk: "bite", v1: false },
    BlueDragon => { id: "monster.blue_dragon", ai: RangedOnSight, diff: 9, atk: "bite", v1: false },
    SilverDragon => { id: "monster.silver_dragon", ai: RangedOnSight, diff: 10, atk: "bite", v1: false },
    AirElemental => { id: "monster.air_elemental", ai: ChaseVisiblePlayer, diff: 8, atk: "engulf", v1: false },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MonsterTemplate {
    pub kind: MonsterKind,
    pub ai_kind: MonsterAiKind,
    pub hp: i16,
    pub ac: i16,
    pub hit_bonus: i16,
    pub damage_bonus: i16,
    pub attack_profile: AttackProfile,
    pub speed: i16,
    pub passive: Option<MonsterPassive>,
    pub difficulty: u16,
    pub resistances: Resistances,
}

impl MonsterKind {
    pub const fn is_p11_only(self) -> bool {
        !self.is_legacy_v1_allowed()
    }

    pub const fn is_pet_capable(self) -> bool {
        matches!(
            self,
            Self::Kitten
                | Self::Housecat
                | Self::LittleDog
                | Self::Dog
                | Self::Pony
                | Self::Warhorse
                | Self::Jackal
                | Self::Fox
                | Self::Wolf
        )
    }

    pub const fn is_herbivore_pet(self) -> bool {
        matches!(self, Self::Pony | Self::Warhorse)
    }

    pub const fn evolved_form(self) -> Option<Self> {
        match self {
            Self::Kitten => Some(Self::Housecat),
            Self::LittleDog => Some(Self::Dog),
            Self::Pony => Some(Self::Warhorse),
            _ => None,
        }
    }
}
