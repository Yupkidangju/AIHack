use serde::{Deserialize, Serialize};

use crate::{campaign::SpellKind, domain::combat::AttackProfile};

pub const MIN_ITEM_ENCHANTMENT: i8 = -3;
pub const MAX_ITEM_ENCHANTMENT: i8 = 5;
pub const MAX_ITEM_STACK: u16 = 99;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum BucStatus {
    Blessed,
    #[default]
    Uncursed,
    Cursed,
}

impl BucStatus {
    pub fn is_uncursed(&self) -> bool {
        matches!(self, Self::Uncursed)
    }
}

macro_rules! define_item_kinds {
    (
        $(
            $variant:ident => {
                id: $toml_id:expr,
                stackable: $stackable:expr,
                appearance: $appearance:expr,
                v1: $v1:expr $(,)?
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum ItemKind {
            $($variant,)*
        }

        impl ItemKind {
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

            pub const fn is_stackable(self) -> bool {
                match self {
                    $(Self::$variant => $stackable,)*
                }
            }

            pub const fn has_randomized_appearance(self) -> bool {
                match self {
                    $(Self::$variant => $appearance,)*
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

define_item_kinds! {
    // --- Existing 55 items (exact serde variant & TOML ID parity) ---
    Dagger => { id: "item.weapon.dagger", stackable: false, appearance: false, v1: true },
    ShortSword => { id: "item.weapon.short_sword", stackable: false, appearance: false, v1: true },
    Mace => { id: "item.weapon.mace", stackable: false, appearance: false, v1: true },
    FoodRation => { id: "item.food.ration", stackable: true, appearance: false, v1: true },
    PotionHealing => { id: "item.potion.healing", stackable: true, appearance: true, v1: true },
    PotionExtraHealing => { id: "item.potion.extra_healing", stackable: true, appearance: true, v1: true },
    PotionCure => { id: "item.potion.cure", stackable: true, appearance: true, v1: true },
    WandMagicMissile => { id: "item.wand.magic_missile", stackable: false, appearance: false, v1: true },
    WandFire => { id: "item.wand.fire", stackable: false, appearance: false, v1: true },
    ScrollReveal => { id: "item.scroll.reveal", stackable: true, appearance: true, v1: true },
    ScrollIdentify => { id: "item.scroll.identify", stackable: true, appearance: true, v1: true },
    ScrollLevelTeleport => { id: "item.scroll.teleport", stackable: true, appearance: true, v1: true },
    ScrollCharging => { id: "item.scroll.charging", stackable: true, appearance: true, v1: true },
    ScrollEnchantWeapon => { id: "item.scroll.enchant_weapon", stackable: true, appearance: true, v1: true },
    ScrollEnchantArmor => { id: "item.scroll.enchant_armor", stackable: true, appearance: true, v1: true },
    ScrollRemoveCurse => { id: "item.scroll.remove_curse", stackable: true, appearance: true, v1: true },
    Rock => { id: "item.weapon.rock", stackable: true, appearance: false, v1: true },
    ArmorLeather => { id: "item.armor.leather", stackable: false, appearance: false, v1: true },
    ArmorChainMail => { id: "item.armor.chain_mail", stackable: false, appearance: false, v1: true },
    RingPoisonResistance => { id: "item.ring.poison_resistance", stackable: false, appearance: true, v1: true },
    RingFireResistance => { id: "item.ring.fire_resistance", stackable: false, appearance: true, v1: true },
    RingProtection => { id: "item.ring.protection", stackable: false, appearance: true, v1: true },
    CorpseJackal => { id: "item.corpse.jackal", stackable: false, appearance: false, v1: true },
    AmuletAscension => { id: "item.quest.ascension", stackable: false, appearance: false, v1: true },
    ArmorShield => { id: "item.armor.small_shield", stackable: false, appearance: false, v1: true },
    ArmorHelm => { id: "item.armor.iron_helm", stackable: false, appearance: false, v1: true },
    ArmorCloak => { id: "item.armor.cloak_protection", stackable: false, appearance: false, v1: true },
    ArmorGloves => { id: "item.armor.leather_gloves", stackable: false, appearance: false, v1: true },
    ArmorBoots => { id: "item.armor.low_boots", stackable: false, appearance: false, v1: true },
    AmuletEsp => { id: "item.amulet.esp", stackable: false, appearance: true, v1: true },
    PotionSpeed => { id: "item.potion.speed", stackable: true, appearance: true, v1: true },
    PotionBlindness => { id: "item.potion.blindness", stackable: true, appearance: true, v1: true },
    PotionConfusion => { id: "item.potion.confusion", stackable: true, appearance: true, v1: true },
    PotionHallucination => { id: "item.potion.hallucination", stackable: true, appearance: true, v1: true },
    PotionInvisibility => { id: "item.potion.invisibility", stackable: true, appearance: true, v1: true },
    PotionLevitation => { id: "item.potion.levitation", stackable: true, appearance: true, v1: true },
    Corpse => { id: "item.corpse.generic", stackable: false, appearance: false, v1: true },
    SkeletonKey => { id: "item.tool.skeleton_key", stackable: false, appearance: false, v1: false },
    LockPick => { id: "item.tool.lock_pick", stackable: false, appearance: false, v1: false },
    PickAxe => { id: "item.tool.pick_axe", stackable: false, appearance: false, v1: false },
    DwarvishMattock => { id: "item.tool.dwarvish_mattock", stackable: false, appearance: false, v1: false },
    TinWhistle => { id: "item.tool.tin_whistle", stackable: false, appearance: false, v1: false },
    OilLamp => { id: "item.tool.oil_lamp", stackable: false, appearance: false, v1: false },
    Blindfold => { id: "item.tool.blindfold", stackable: false, appearance: false, v1: false },
    UnicornHorn => { id: "item.tool.unicorn_horn", stackable: false, appearance: false, v1: false },
    Sack => { id: "item.tool.sack", stackable: false, appearance: false, v1: false },
    BagOfHolding => { id: "item.tool.bag_of_holding", stackable: false, appearance: false, v1: false },
    Chest => { id: "item.tool.chest", stackable: false, appearance: false, v1: false },
    PotionWater => { id: "item.potion.water", stackable: true, appearance: false, v1: false },
    ScrollBlank => { id: "item.scroll.blank", stackable: true, appearance: false, v1: false },
    LongSword => { id: "item.weapon.long_sword", stackable: false, appearance: false, v1: false },
    Katana => { id: "item.weapon.katana", stackable: false, appearance: false, v1: false },
    BattleAxe => { id: "item.weapon.battle_axe", stackable: false, appearance: false, v1: false },
    Quarterstaff => { id: "item.weapon.quarterstaff", stackable: false, appearance: false, v1: false },
    BroadSword => { id: "item.weapon.broad_sword", stackable: false, appearance: false, v1: false },

    // --- P12 28 Spellbooks ---
    SpellbookForceBolt => { id: "item.spellbook.force_bolt", stackable: false, appearance: true, v1: false },
    SpellbookMagicMissile => { id: "item.spellbook.magic_missile", stackable: false, appearance: true, v1: false },
    SpellbookFireball => { id: "item.spellbook.fireball", stackable: false, appearance: true, v1: false },
    SpellbookConeOfCold => { id: "item.spellbook.cone_of_cold", stackable: false, appearance: true, v1: false },
    SpellbookLightningBolt => { id: "item.spellbook.lightning_bolt", stackable: false, appearance: true, v1: false },
    SpellbookFingerOfDeath => { id: "item.spellbook.finger_of_death", stackable: false, appearance: true, v1: false },
    SpellbookHeal => { id: "item.spellbook.heal", stackable: false, appearance: true, v1: false },
    SpellbookExtraHealing => { id: "item.spellbook.extra_healing", stackable: false, appearance: true, v1: false },
    SpellbookCureSickness => { id: "item.spellbook.cure_sickness", stackable: false, appearance: true, v1: false },
    SpellbookStoneToFlesh => { id: "item.spellbook.stone_to_flesh", stackable: false, appearance: true, v1: false },
    SpellbookDetect => { id: "item.spellbook.detect", stackable: false, appearance: true, v1: false },
    SpellbookIdentify => { id: "item.spellbook.identify", stackable: false, appearance: true, v1: false },
    SpellbookMagicMapping => { id: "item.spellbook.magic_mapping", stackable: false, appearance: true, v1: false },
    SpellbookDetectTreasure => { id: "item.spellbook.detect_treasure", stackable: false, appearance: true, v1: false },
    SpellbookSleep => { id: "item.spellbook.sleep", stackable: false, appearance: true, v1: false },
    SpellbookConfuseMonster => { id: "item.spellbook.confuse_monster", stackable: false, appearance: true, v1: false },
    SpellbookCharmMonster => { id: "item.spellbook.charm_monster", stackable: false, appearance: true, v1: false },
    SpellbookSlowMonster => { id: "item.spellbook.slow_monster", stackable: false, appearance: true, v1: false },
    SpellbookRemoveCurse => { id: "item.spellbook.remove_curse", stackable: false, appearance: true, v1: false },
    SpellbookProtection => { id: "item.spellbook.protection", stackable: false, appearance: true, v1: false },
    SpellbookTurnUndead => { id: "item.spellbook.turn_undead", stackable: false, appearance: true, v1: false },
    SpellbookCreateFamiliar => { id: "item.spellbook.create_familiar", stackable: false, appearance: true, v1: false },
    SpellbookJumping => { id: "item.spellbook.jumping", stackable: false, appearance: true, v1: false },
    SpellbookHasteSelf => { id: "item.spellbook.haste_self", stackable: false, appearance: true, v1: false },
    SpellbookTeleportAway => { id: "item.spellbook.teleport_away", stackable: false, appearance: true, v1: false },
    SpellbookKnock => { id: "item.spellbook.knock", stackable: false, appearance: true, v1: false },
    SpellbookWizardLock => { id: "item.spellbook.wizard_lock", stackable: false, appearance: true, v1: false },
    SpellbookDig => { id: "item.spellbook.dig", stackable: false, appearance: true, v1: false },

    // --- P12 16 Wands ---
    WandCold => { id: "item.wand.cold", stackable: false, appearance: true, v1: false },
    WandLightning => { id: "item.wand.lightning", stackable: false, appearance: true, v1: false },
    WandSleep => { id: "item.wand.sleep", stackable: false, appearance: true, v1: false },
    WandDigging => { id: "item.wand.digging", stackable: false, appearance: true, v1: false },
    WandTeleport => { id: "item.wand.teleport", stackable: false, appearance: true, v1: false },
    WandPolymorph => { id: "item.wand.polymorph", stackable: false, appearance: true, v1: false },
    WandCancellation => { id: "item.wand.cancellation", stackable: false, appearance: true, v1: false },
    WandDeath => { id: "item.wand.death", stackable: false, appearance: true, v1: false },
    WandWishing => { id: "item.wand.wishing", stackable: false, appearance: true, v1: false },
    WandStriking => { id: "item.wand.striking", stackable: false, appearance: true, v1: false },
    WandSlowMonster => { id: "item.wand.slow_monster", stackable: false, appearance: true, v1: false },
    WandSpeedMonster => { id: "item.wand.speed_monster", stackable: false, appearance: true, v1: false },
    WandOpening => { id: "item.wand.opening", stackable: false, appearance: true, v1: false },
    WandLocking => { id: "item.wand.locking", stackable: false, appearance: true, v1: false },
    WandMakeInvisible => { id: "item.wand.make_invisible", stackable: false, appearance: true, v1: false },
    WandCreateMonster => { id: "item.wand.create_monster", stackable: false, appearance: true, v1: false },

    // --- P12 9 Rings ---
    RingColdResistance => { id: "item.ring.cold_resistance", stackable: false, appearance: true, v1: false },
    RingShockResistance => { id: "item.ring.shock_resistance", stackable: false, appearance: true, v1: false },
    RingSleepResistance => { id: "item.ring.sleep_resistance", stackable: false, appearance: true, v1: false },
    RingInvisibility => { id: "item.ring.invisibility", stackable: false, appearance: true, v1: false },
    RingLevitation => { id: "item.ring.levitation", stackable: false, appearance: true, v1: false },
    RingConflict => { id: "item.ring.conflict", stackable: false, appearance: true, v1: false },
    RingSearching => { id: "item.ring.searching", stackable: false, appearance: true, v1: false },
    RingTeleportControl => { id: "item.ring.teleport_control", stackable: false, appearance: true, v1: false },
    RingSeeInvisible => { id: "item.ring.see_invisible", stackable: false, appearance: true, v1: false },

    // --- P12 5 Amulets ---
    AmuletLifeSaving => { id: "item.amulet.life_saving", stackable: false, appearance: true, v1: false },
    AmuletReflection => { id: "item.amulet.reflection", stackable: false, appearance: true, v1: false },
    AmuletMagicalBreathing => { id: "item.amulet.magical_breathing", stackable: false, appearance: true, v1: false },
    AmuletPoisonResistance => { id: "item.amulet.poison_resistance", stackable: false, appearance: true, v1: false },
    AmuletUnchanging => { id: "item.amulet.unchanging", stackable: false, appearance: true, v1: false },

    // --- P12 5 Weapons ---
    SilverSaber => { id: "item.weapon.silver_saber", stackable: false, appearance: false, v1: false },
    TwoHandedSword => { id: "item.weapon.two_handed_sword", stackable: false, appearance: false, v1: false },
    WarHammer => { id: "item.weapon.war_hammer", stackable: false, appearance: false, v1: false },
    ElvenDagger => { id: "item.weapon.elven_dagger", stackable: false, appearance: false, v1: false },
    Spear => { id: "item.weapon.spear", stackable: true, appearance: false, v1: false },

    // --- P12 5 Armor ---
    ArmorSilverDragonScale => { id: "item.armor.silver_dragon_scale", stackable: false, appearance: false, v1: false },
    ArmorGrayDragonScale => { id: "item.armor.gray_dragon_scale", stackable: false, appearance: false, v1: false },
    ArmorShieldReflection => { id: "item.armor.shield_reflection", stackable: false, appearance: false, v1: false },
    ArmorCloakMagicResistance => { id: "item.armor.cloak_magic_resistance", stackable: false, appearance: false, v1: false },
    ArmorSpeedBoots => { id: "item.armor.speed_boots", stackable: false, appearance: false, v1: false },

    // --- P12 4 Potions / Scrolls ---
    PotionFullHealing => { id: "item.potion.full_healing", stackable: true, appearance: true, v1: false },
    PotionGainLevel => { id: "item.potion.gain_level", stackable: true, appearance: true, v1: false },
    PotionPolymorph => { id: "item.potion.polymorph", stackable: true, appearance: true, v1: false },
    ScrollMagicMapping => { id: "item.scroll.magic_mapping", stackable: true, appearance: true, v1: false },

    // --- P12 3 Food / Gems ---
    FoodLembasWafer => { id: "item.food.lembas_wafer", stackable: true, appearance: false, v1: false },
    GemDiamond => { id: "item.gem.diamond", stackable: true, appearance: false, v1: false },
    GemRuby => { id: "item.gem.ruby", stackable: true, appearance: false, v1: false },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemClass {
    Weapon,
    Food,
    Potion,
    Wand,
    Scroll,
    Rock,
    Armor,
    Ring,
    Corpse,
    Quest,
    Amulet,
    Tool,
    Spellbook,
    Gem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
    Helm,
    Body,
    Cloak,
    Gloves,
    Boots,
    #[serde(alias = "Ring")]
    LeftRing,
    RightRing,
    Amulet,
}

impl EquipmentSlot {
    pub const ALL: [Self; 10] = [
        Self::Melee,
        Self::Shield,
        Self::Helm,
        Self::Body,
        Self::Cloak,
        Self::Gloves,
        Self::Boots,
        Self::LeftRing,
        Self::RightRing,
        Self::Amulet,
    ];

    pub const ALL_DEFENSE: [Self; 9] = [
        Self::Shield,
        Self::Helm,
        Self::Body,
        Self::Cloak,
        Self::Gloves,
        Self::Boots,
        Self::LeftRing,
        Self::RightRing,
        Self::Amulet,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Melee => "weapon",
            Self::Shield => "shield",
            Self::Helm => "helm",
            Self::Body => "body armor",
            Self::Cloak => "cloak",
            Self::Gloves => "gloves",
            Self::Boots => "boots",
            Self::LeftRing => "left ring",
            Self::RightRing => "right ring",
            Self::Amulet => "amulet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemMaterial {
    Iron,
    Steel,
    Leather,
    Cloth,
    Wood,
    Glass,
    Paper,
    Mineral,
    Mithril,
    Dragonhide,
    Gold,
    Silver,
    Organic,
    Gemstone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolAction {
    Light,
    Unlock,
    Dig,
    Container,
    Music,
    Magic,
    Blindfold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectKind {
    Blind,
    Confused,
    Stunned,
    Hallucinating,
    Fast,
    Levitation,
    Invisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RingEffect {
    PoisonResistance,
    FireResistance,
    Protection,
    ColdResistance,
    ShockResistance,
    SleepResistance,
    Invisibility,
    Levitation,
    Conflict,
    Searching,
    TeleportControl,
    SeeInvisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AmuletEffect {
    Esp,
    LifeSaving,
    Reflection,
    MagicalBreathing,
    PoisonResistance,
    Unchanging,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsumableEffect {
    Heal { dice: i16, sides: i16, bonus: i16 },
    RevealLevel,
    IdentifySingle,
    LevelTeleport,
    RechargeWand,
    EnchantWeapon,
    EnchantArmor,
    RemoveCurse,
    CureStatus,
    ApplyStatus { effect: StatusEffectKind, turns: u8 },
    Water,
    BlankPaper,
    GainLevel,
    PolymorphSelf,
    MagicMapping,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WandEffect {
    MagicMissile,
    Fire,
    Cold,
    Lightning,
    Sleep,
    Digging,
    Teleport,
    Polymorph,
    Cancellation,
    Death,
    Wishing,
    Striking,
    SlowMonster,
    SpeedMonster,
    Opening,
    Locking,
    MakeInvisible,
    CreateMonster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemData {
    pub kind: ItemKind,
    pub class: ItemClass,
    pub glyph: char,
    pub weight: i16,
    pub base_price: u32,
    #[serde(default)]
    pub ac_bonus: i16,
    pub attack_profile: Option<AttackProfile>,
    pub consumable_effect: Option<ConsumableEffect>,
    pub wand_effect: Option<WandEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_effect: Option<RingEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amulet_effect: Option<AmuletEffect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_slot: Option<EquipmentSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_action: Option<ToolAction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_capacity: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spellbook_spell: Option<SpellKind>,
    pub max_charges: Option<u8>,
    pub nutrition: Option<i16>,
}

impl ItemData {
    pub const fn effective_armor_slot(&self) -> Option<EquipmentSlot> {
        if let Some(slot) = self.armor_slot {
            return Some(slot);
        }
        if matches!(self.class, ItemClass::Armor) {
            return Some(EquipmentSlot::Body);
        }
        None
    }
}
