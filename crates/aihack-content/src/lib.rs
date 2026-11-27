//! Embedded content registry다.
//!
//! content는 core에만 의존하며 UI와 adapter dependency를 추가하지 않는다.

pub mod core {
    pub mod error {
        pub use aihack_core::error::*;
    }
}

pub mod schema;

use std::sync::OnceLock;

pub use schema::{
    ContentRegistry, DoorData, HiddenDoorData, HiddenTrapData, ItemData, LevelData, LevelItemData,
    LevelMonsterData, MonsterAttackData, MonsterData, WallData, CONTENT_SCHEMA_VERSION,
    MAX_CONTENT_ARMOR_AC_BONUS, MAX_CONTENT_BASE_PRICE, MAX_CONTENT_MONSTER_HP,
    MAX_CONTENT_MONSTER_SPEED, MAX_CONTENT_NUTRITION,
};

use aihack_core::domain::map::MapLayout;
use aihack_core::domain::{
    combat::{AttackProfile, DamageRoll},
    item::{
        ConsumableEffect, EquipmentSlot, ItemClass, ItemData as CoreItemData, ItemKind, RingEffect,
        StatusEffectKind, ToolAction, WandEffect,
    },
    monster::{MonsterAiKind, MonsterKind, MonsterPassive, MonsterTemplate},
    status::Resistances,
    tile::{DoorState, TileKind, TrapKind},
};
use aihack_core::error::ContentError;
use aihack_core::position::Pos;
use serde::{Deserialize, Serialize};

static EMBEDDED_REGISTRY: OnceLock<Result<ContentRegistry, ContentError>> = OnceLock::new();

pub fn registry() -> Result<&'static ContentRegistry, ContentError> {
    EMBEDDED_REGISTRY
        .get_or_init(|| {
            ContentRegistry::from_toml_sources(
                CONTENT_SCHEMA_VERSION,
                include_str!("data/items.toml"),
                include_str!("data/monsters.toml"),
                &[
                    (
                        "levels/main_1.toml",
                        include_str!("data/levels/main_1.toml"),
                    ),
                    (
                        "levels/main_2.toml",
                        include_str!("data/levels/main_2.toml"),
                    ),
                ],
            )
        })
        .as_ref()
        .map_err(Clone::clone)
}

pub fn load_items() -> Result<Vec<ItemData>, ContentError> {
    Ok(registry()?.items().cloned().collect())
}

pub fn load_monsters() -> Result<Vec<MonsterData>, ContentError> {
    Ok(registry()?.monsters().cloned().collect())
}

pub fn load_level(level_id: &str) -> Result<LevelData, ContentError> {
    registry()?
        .level(level_id)
        .cloned()
        .ok_or_else(|| ContentError::UnknownReference {
            owner: "load_level".to_owned(),
            target: level_id.to_owned(),
        })
}

/// MonsterKind -> canonical TOML id.
pub const fn monster_id(kind: MonsterKind) -> &'static str {
    match kind {
        MonsterKind::Jackal => "monster.jackal",
        MonsterKind::Goblin => "monster.goblin",
        MonsterKind::FloatingEye => "monster.floating_eye",
        MonsterKind::Kobold => "monster.kobold",
        MonsterKind::Orc => "monster.orc",
        MonsterKind::Ogre => "monster.ogre",
        MonsterKind::GiantSpider => "monster.giant_spider",
        MonsterKind::GnollArcher => "monster.gnoll_archer",
        MonsterKind::Troll => "monster.troll",
        MonsterKind::FireAnt => "monster.fire_ant",
        MonsterKind::Minotaur => "monster.minotaur",
        MonsterKind::Dragon => "monster.dragon",
        MonsterKind::Bat => "monster.bat",
        MonsterKind::Kitten => "monster.kitten",
        MonsterKind::Housecat => "monster.housecat",
        MonsterKind::LittleDog => "monster.little_dog",
        MonsterKind::Dog => "monster.dog",
        MonsterKind::Pony => "monster.pony",
        MonsterKind::Warhorse => "monster.warhorse",
        MonsterKind::Shopkeeper => "monster.shopkeeper",
        MonsterKind::TemplePriest => "monster.temple_priest",
        MonsterKind::Oracle => "monster.oracle",
        MonsterKind::Watchman => "monster.watchman",
        MonsterKind::Leprechaun => "monster.leprechaun",
        MonsterKind::Nymph => "monster.nymph",
        MonsterKind::RustMonster => "monster.rust_monster",
        MonsterKind::BlackPudding => "monster.black_pudding",
        MonsterKind::Dwarf => "monster.dwarf",
        MonsterKind::Medusa => "monster.medusa",
        MonsterKind::OrcCaptain => "monster.orc_captain",
    }
}

/// Canonical TOML id -> MonsterKind.
pub fn monster_kind_from_id(id: &str) -> Result<MonsterKind, ContentError> {
    match id {
        "monster.jackal" => Ok(MonsterKind::Jackal),
        "monster.goblin" => Ok(MonsterKind::Goblin),
        "monster.floating_eye" => Ok(MonsterKind::FloatingEye),
        "monster.kobold" => Ok(MonsterKind::Kobold),
        "monster.orc" => Ok(MonsterKind::Orc),
        "monster.ogre" => Ok(MonsterKind::Ogre),
        "monster.giant_spider" => Ok(MonsterKind::GiantSpider),
        "monster.gnoll_archer" => Ok(MonsterKind::GnollArcher),
        "monster.troll" => Ok(MonsterKind::Troll),
        "monster.fire_ant" => Ok(MonsterKind::FireAnt),
        "monster.minotaur" => Ok(MonsterKind::Minotaur),
        "monster.dragon" => Ok(MonsterKind::Dragon),
        "monster.bat" => Ok(MonsterKind::Bat),
        "monster.kitten" => Ok(MonsterKind::Kitten),
        "monster.housecat" => Ok(MonsterKind::Housecat),
        "monster.little_dog" => Ok(MonsterKind::LittleDog),
        "monster.dog" => Ok(MonsterKind::Dog),
        "monster.pony" => Ok(MonsterKind::Pony),
        "monster.warhorse" => Ok(MonsterKind::Warhorse),
        "monster.shopkeeper" => Ok(MonsterKind::Shopkeeper),
        "monster.temple_priest" => Ok(MonsterKind::TemplePriest),
        "monster.oracle" => Ok(MonsterKind::Oracle),
        "monster.watchman" => Ok(MonsterKind::Watchman),
        "monster.leprechaun" => Ok(MonsterKind::Leprechaun),
        "monster.nymph" => Ok(MonsterKind::Nymph),
        "monster.rust_monster" => Ok(MonsterKind::RustMonster),
        "monster.black_pudding" => Ok(MonsterKind::BlackPudding),
        "monster.dwarf" => Ok(MonsterKind::Dwarf),
        "monster.medusa" => Ok(MonsterKind::Medusa),
        "monster.orc_captain" => Ok(MonsterKind::OrcCaptain),
        _ => Err(ContentError::UnknownReference {
            owner: "monster kind".to_owned(),
            target: id.to_owned(),
        }),
    }
}

/// ItemKind -> canonical TOML id.
pub const fn item_id(kind: ItemKind) -> &'static str {
    match kind {
        ItemKind::Dagger => "item.weapon.dagger",
        ItemKind::ShortSword => "item.weapon.short_sword",
        ItemKind::Mace => "item.weapon.mace",
        ItemKind::FoodRation => "item.food.ration",
        ItemKind::PotionHealing => "item.potion.healing",
        ItemKind::PotionExtraHealing => "item.potion.extra_healing",
        ItemKind::PotionCure => "item.potion.cure",
        ItemKind::PotionSpeed => "item.potion.speed",
        ItemKind::PotionBlindness => "item.potion.blindness",
        ItemKind::PotionConfusion => "item.potion.confusion",
        ItemKind::PotionHallucination => "item.potion.hallucination",
        ItemKind::PotionInvisibility => "item.potion.invisibility",
        ItemKind::PotionLevitation => "item.potion.levitation",
        ItemKind::WandMagicMissile => "item.wand.magic_missile",
        ItemKind::WandFire => "item.wand.fire",
        ItemKind::ScrollReveal => "item.scroll.reveal",
        ItemKind::ScrollIdentify => "item.scroll.identify",
        ItemKind::ScrollLevelTeleport => "item.scroll.teleport",
        ItemKind::ScrollCharging => "item.scroll.charging",
        ItemKind::ScrollEnchantWeapon => "item.scroll.enchant_weapon",
        ItemKind::ScrollEnchantArmor => "item.scroll.enchant_armor",
        ItemKind::ScrollRemoveCurse => "item.scroll.remove_curse",
        ItemKind::Rock => "item.weapon.rock",
        ItemKind::ArmorLeather => "item.armor.leather",
        ItemKind::ArmorChainMail => "item.armor.chain_mail",
        ItemKind::ArmorShield => "item.armor.small_shield",
        ItemKind::ArmorHelm => "item.armor.iron_helm",
        ItemKind::ArmorCloak => "item.armor.cloak_protection",
        ItemKind::ArmorGloves => "item.armor.leather_gloves",
        ItemKind::ArmorBoots => "item.armor.low_boots",
        ItemKind::RingPoisonResistance => "item.ring.poison_resistance",
        ItemKind::RingFireResistance => "item.ring.fire_resistance",
        ItemKind::RingProtection => "item.ring.protection",
        ItemKind::AmuletEsp => "item.amulet.esp",
        ItemKind::CorpseJackal => "item.corpse.jackal",
        ItemKind::Corpse => "item.corpse.generic",
        ItemKind::AmuletAscension => "item.quest.ascension",
        ItemKind::SkeletonKey => "item.tool.skeleton_key",
        ItemKind::LockPick => "item.tool.lock_pick",
        ItemKind::PickAxe => "item.tool.pick_axe",
        ItemKind::DwarvishMattock => "item.tool.dwarvish_mattock",
        ItemKind::TinWhistle => "item.tool.tin_whistle",
        ItemKind::OilLamp => "item.tool.oil_lamp",
        ItemKind::Blindfold => "item.tool.blindfold",
        ItemKind::UnicornHorn => "item.tool.unicorn_horn",
        ItemKind::Sack => "item.tool.sack",
        ItemKind::BagOfHolding => "item.tool.bag_of_holding",
        ItemKind::Chest => "item.tool.chest",
        ItemKind::PotionWater => "item.potion.water",
        ItemKind::ScrollBlank => "item.scroll.blank",
        ItemKind::LongSword => "item.weapon.long_sword",
        ItemKind::Katana => "item.weapon.katana",
        ItemKind::BattleAxe => "item.weapon.battle_axe",
        ItemKind::Quarterstaff => "item.weapon.quarterstaff",
        ItemKind::BroadSword => "item.weapon.broad_sword",
    }
}

/// Canonical TOML id -> ItemKind.
pub fn item_kind_from_id(id: &str) -> Result<ItemKind, ContentError> {
    match id {
        "item.weapon.dagger" => Ok(ItemKind::Dagger),
        "item.weapon.short_sword" => Ok(ItemKind::ShortSword),
        "item.weapon.long_sword" => Ok(ItemKind::LongSword),
        "item.weapon.katana" => Ok(ItemKind::Katana),
        "item.weapon.battle_axe" => Ok(ItemKind::BattleAxe),
        "item.weapon.quarterstaff" => Ok(ItemKind::Quarterstaff),
        "item.weapon.broad_sword" => Ok(ItemKind::BroadSword),
        "item.weapon.mace" => Ok(ItemKind::Mace),
        "item.food.ration" => Ok(ItemKind::FoodRation),
        "item.potion.healing" => Ok(ItemKind::PotionHealing),
        "item.potion.extra_healing" => Ok(ItemKind::PotionExtraHealing),
        "item.potion.cure" => Ok(ItemKind::PotionCure),
        "item.potion.speed" => Ok(ItemKind::PotionSpeed),
        "item.potion.blindness" => Ok(ItemKind::PotionBlindness),
        "item.potion.confusion" => Ok(ItemKind::PotionConfusion),
        "item.potion.hallucination" => Ok(ItemKind::PotionHallucination),
        "item.potion.invisibility" => Ok(ItemKind::PotionInvisibility),
        "item.potion.levitation" => Ok(ItemKind::PotionLevitation),
        "item.potion.water" => Ok(ItemKind::PotionWater),
        "item.wand.magic_missile" => Ok(ItemKind::WandMagicMissile),
        "item.wand.fire" => Ok(ItemKind::WandFire),
        "item.scroll.reveal" => Ok(ItemKind::ScrollReveal),
        "item.scroll.identify" => Ok(ItemKind::ScrollIdentify),
        "item.scroll.teleport" => Ok(ItemKind::ScrollLevelTeleport),
        "item.scroll.charging" => Ok(ItemKind::ScrollCharging),
        "item.scroll.enchant_weapon" => Ok(ItemKind::ScrollEnchantWeapon),
        "item.scroll.enchant_armor" => Ok(ItemKind::ScrollEnchantArmor),
        "item.scroll.remove_curse" => Ok(ItemKind::ScrollRemoveCurse),
        "item.scroll.blank" => Ok(ItemKind::ScrollBlank),
        "item.weapon.rock" => Ok(ItemKind::Rock),
        "item.armor.leather" => Ok(ItemKind::ArmorLeather),
        "item.armor.chain_mail" => Ok(ItemKind::ArmorChainMail),
        "item.armor.small_shield" => Ok(ItemKind::ArmorShield),
        "item.armor.iron_helm" => Ok(ItemKind::ArmorHelm),
        "item.armor.cloak_protection" => Ok(ItemKind::ArmorCloak),
        "item.armor.leather_gloves" => Ok(ItemKind::ArmorGloves),
        "item.armor.low_boots" => Ok(ItemKind::ArmorBoots),
        "item.ring.poison_resistance" => Ok(ItemKind::RingPoisonResistance),
        "item.ring.fire_resistance" => Ok(ItemKind::RingFireResistance),
        "item.ring.protection" => Ok(ItemKind::RingProtection),
        "item.amulet.esp" => Ok(ItemKind::AmuletEsp),
        "item.corpse.jackal" => Ok(ItemKind::CorpseJackal),
        "item.corpse.generic" => Ok(ItemKind::Corpse),
        "item.quest.ascension" => Ok(ItemKind::AmuletAscension),
        "item.tool.skeleton_key" => Ok(ItemKind::SkeletonKey),
        "item.tool.lock_pick" => Ok(ItemKind::LockPick),
        "item.tool.pick_axe" => Ok(ItemKind::PickAxe),
        "item.tool.dwarvish_mattock" => Ok(ItemKind::DwarvishMattock),
        "item.tool.tin_whistle" => Ok(ItemKind::TinWhistle),
        "item.tool.oil_lamp" => Ok(ItemKind::OilLamp),
        "item.tool.blindfold" => Ok(ItemKind::Blindfold),
        "item.tool.unicorn_horn" => Ok(ItemKind::UnicornHorn),
        "item.tool.sack" => Ok(ItemKind::Sack),
        "item.tool.bag_of_holding" => Ok(ItemKind::BagOfHolding),
        "item.tool.chest" => Ok(ItemKind::Chest),
        _ => Err(ContentError::UnknownReference {
            owner: "item kind".to_owned(),
            target: id.to_owned(),
        }),
    }
}

/// Registry definition을 core가 소비하는 monster template으로 변환한다.
pub fn monster_template_from_registry(
    kind: MonsterKind,
    registry: &ContentRegistry,
) -> Result<MonsterTemplate, ContentError> {
    let id = monster_id(kind);
    let definition = registry
        .monster(id)
        .ok_or_else(|| ContentError::UnknownReference {
            owner: "monster factory".to_owned(),
            target: id.to_owned(),
        })?;
    let ai_kind = match definition.ai.as_str() {
        "wander" => MonsterAiKind::Wander,
        "chase_on_sight" => MonsterAiKind::ChaseVisiblePlayer,
        "stationary" => MonsterAiKind::Stationary,
        "ranged_on_sight" => MonsterAiKind::RangedOnSight,
        ai => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: ai.to_owned(),
            })
        }
    };
    let damage = parse_damage(&definition.damage)?;
    let passive = match definition.passive.as_deref() {
        Some("paralyze_on_melee") => Some(MonsterPassive::ParalyzeOnMelee),
        Some("poison_on_melee") => Some(MonsterPassive::PoisonOnMelee),
        Some("regenerate_hp") => Some(MonsterPassive::Regenerate),
        Some("fire_on_melee") => Some(MonsterPassive::FireOnMelee),
        Some("fire_breath") => Some(MonsterPassive::FireBreath),
        Some("steal_gold") => Some(MonsterPassive::StealGold),
        Some("steal_item") => Some(MonsterPassive::StealItem),
        Some("rust_metal") => Some(MonsterPassive::RustMetal),
        Some("split_on_hit") => Some(MonsterPassive::SplitOnHit),
        Some("gaze_paralyze") => Some(MonsterPassive::GazeParalyze),
        Some("summon_allies") => Some(MonsterPassive::SummonAllies),
        None => None,
        Some(value) => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: value.to_owned(),
            })
        }
    };
    let name = match kind {
        MonsterKind::Jackal => "bite",
        MonsterKind::Goblin => "short sword",
        MonsterKind::FloatingEye => "gaze",
        MonsterKind::Kobold => "spear",
        MonsterKind::Orc => "scimitar",
        MonsterKind::Ogre => "club",
        MonsterKind::GiantSpider => "fangs",
        MonsterKind::GnollArcher => "arrow",
        MonsterKind::Troll => "claw",
        MonsterKind::FireAnt => "fire bite",
        MonsterKind::Minotaur => "gore",
        MonsterKind::Dragon => "bite",
        MonsterKind::Bat => "bite",
        MonsterKind::Kitten
        | MonsterKind::Housecat
        | MonsterKind::LittleDog
        | MonsterKind::Dog
        | MonsterKind::RustMonster => "bite",
        MonsterKind::Pony | MonsterKind::Warhorse => "kick",
        MonsterKind::Shopkeeper | MonsterKind::TemplePriest | MonsterKind::Watchman => "weapon",
        MonsterKind::Oracle | MonsterKind::Nymph => "touch",
        MonsterKind::Leprechaun => "claw",
        MonsterKind::BlackPudding => "engulf",
        MonsterKind::Dwarf | MonsterKind::OrcCaptain => "axe",
        MonsterKind::Medusa => "snakes",
    };
    Ok(MonsterTemplate {
        kind,
        ai_kind,
        hp: definition.hp,
        ac: definition.ac,
        hit_bonus: definition.hit_bonus,
        damage_bonus: 0,
        attack_profile: AttackProfile::natural(name, damage),
        speed: definition.speed,
        passive,
        difficulty: definition.difficulty as u16,
        resistances: monster_resistances(kind, registry),
    })
}

pub fn weighted_monster_kinds_for_depth(
    depth: i16,
    registry: &ContentRegistry,
) -> Vec<(MonsterKind, u32)> {
    let mut entries: Vec<_> = registry
        .monsters()
        .filter_map(|def| {
            let weight = u32::from(def.spawn_weight.unwrap_or(0));
            if weight == 0 {
                return None;
            }
            if let Some([min_d, max_d]) = def.spawn_depth_range {
                if depth < min_d || depth > max_d {
                    return None;
                }
            }
            let kind = monster_kind_from_id(&def.id).ok()?;
            Some((def.id.as_str(), kind, weight))
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    entries.into_iter().map(|(_, k, w)| (k, w)).collect()
}

fn parse_damage(value: &str) -> Result<DamageRoll, ContentError> {
    if value == "0" {
        return Ok(DamageRoll::none());
    }
    let Some((dice, sides)) = value.split_once('d') else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    let (Ok(dice), Ok(sides)) = (dice.parse(), sides.parse()) else {
        return Err(ContentError::InvalidDice {
            value: value.to_owned(),
        });
    };
    Ok(DamageRoll::new(dice, sides))
}

/// Registry definition을 core가 소비하는 item data로 변환한다.
pub fn item_data_from_registry(
    kind: ItemKind,
    registry: &ContentRegistry,
) -> Result<CoreItemData, ContentError> {
    let id = item_id(kind);
    let definition = registry
        .item(id)
        .ok_or_else(|| ContentError::UnknownReference {
            owner: "item factory".to_owned(),
            target: id.to_owned(),
        })?;
    let class = match definition.kind.as_str() {
        "weapon" => ItemClass::Weapon,
        "food" => ItemClass::Food,
        "potion" => ItemClass::Potion,
        "wand" => ItemClass::Wand,
        "scroll" => ItemClass::Scroll,
        "armor" => ItemClass::Armor,
        "ring" => ItemClass::Ring,
        "amulet" => ItemClass::Amulet,
        "corpse" => ItemClass::Corpse,
        "quest" => ItemClass::Quest,
        "tool" => ItemClass::Tool,
        other => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: other.to_owned(),
            })
        }
    };
    let mut glyphs = definition.glyph.chars();
    let glyph = glyphs.next().ok_or_else(|| ContentError::Parse {
        file: id.to_owned(),
        message: "glyph must contain exactly one Unicode scalar".to_owned(),
    })?;
    if glyphs.next().is_some() {
        return Err(ContentError::Parse {
            file: id.to_owned(),
            message: "glyph must contain exactly one Unicode scalar".to_owned(),
        });
    }
    let attack_profile = definition
        .damage
        .as_deref()
        .map(|value| {
            let name = match kind {
                ItemKind::Dagger => "dagger",
                ItemKind::ShortSword => "short sword",
                ItemKind::LongSword => "long sword",
                ItemKind::Katana => "katana",
                ItemKind::BattleAxe => "battle-axe",
                ItemKind::Quarterstaff => "quarterstaff",
                ItemKind::BroadSword => "broadsword",
                ItemKind::Mace => "mace",
                ItemKind::Rock => "rock",
                _ => "weapon",
            };
            Ok(AttackProfile {
                name,
                hit_bonus: definition.hit_bonus.unwrap_or_default(),
                damage: parse_damage(value)?,
            })
        })
        .transpose()?;
    let ring_effect = match definition.effect.as_deref() {
        Some("poison_resistance") => Some(RingEffect::PoisonResistance),
        Some("fire_resistance") => Some(RingEffect::FireResistance),
        Some("protection") => Some(RingEffect::Protection),
        _ => None,
    };
    let armor_slot = match definition.slot.as_deref() {
        Some("shield") => Some(EquipmentSlot::Shield),
        Some("helm") => Some(EquipmentSlot::Helm),
        Some("cloak") => Some(EquipmentSlot::Cloak),
        Some("gloves") => Some(EquipmentSlot::Gloves),
        Some("boots") => Some(EquipmentSlot::Boots),
        _ => None,
    };
    let consumable_effect = match definition.effect.as_deref() {
        Some("heal_1d8_plus_4") => Some(ConsumableEffect::Heal {
            dice: 1,
            sides: 8,
            bonus: 4,
        }),
        Some("heal_2d8_plus_8") => Some(ConsumableEffect::Heal {
            dice: 2,
            sides: 8,
            bonus: 8,
        }),
        Some("reveal") => Some(ConsumableEffect::RevealLevel),
        Some("identify") => Some(ConsumableEffect::IdentifySingle),
        Some("teleport") => Some(ConsumableEffect::LevelTeleport),
        Some("charging") => Some(ConsumableEffect::RechargeWand),
        Some("enchant_weapon") => Some(ConsumableEffect::EnchantWeapon),
        Some("enchant_armor") => Some(ConsumableEffect::EnchantArmor),
        Some("remove_curse") => Some(ConsumableEffect::RemoveCurse),
        Some("cure_status") => Some(ConsumableEffect::CureStatus),
        Some("speed") => Some(ConsumableEffect::ApplyStatus {
            effect: StatusEffectKind::Fast,
            turns: 20,
        }),
        Some("blindness") => Some(ConsumableEffect::ApplyStatus {
            effect: StatusEffectKind::Blind,
            turns: 12,
        }),
        Some("confusion") => Some(ConsumableEffect::ApplyStatus {
            effect: StatusEffectKind::Confused,
            turns: 12,
        }),
        Some("hallucination") => Some(ConsumableEffect::ApplyStatus {
            effect: StatusEffectKind::Hallucinating,
            turns: 15,
        }),
        Some("invisibility") => Some(ConsumableEffect::ApplyStatus {
            effect: StatusEffectKind::Invisible,
            turns: 20,
        }),
        Some("levitation") => Some(ConsumableEffect::ApplyStatus {
            effect: StatusEffectKind::Levitation,
            turns: 20,
        }),
        Some("water") => Some(ConsumableEffect::Water),
        Some("blank") => Some(ConsumableEffect::BlankPaper),
        Some("magic_missile")
        | Some("fire")
        | Some("poison_resistance")
        | Some("fire_resistance")
        | Some("protection")
        | None => None,
        Some(effect) => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: effect.to_owned(),
            })
        }
    };
    let tool_action = match definition.tool_action.as_deref() {
        Some("light") => Some(ToolAction::Light),
        Some("unlock") => Some(ToolAction::Unlock),
        Some("dig") => Some(ToolAction::Dig),
        Some("container") => Some(ToolAction::Container),
        Some("music") => Some(ToolAction::Music),
        Some("magic") => Some(ToolAction::Magic),
        Some("blindfold") => Some(ToolAction::Blindfold),
        None => None,
        Some(other) => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: other.to_owned(),
            })
        }
    };
    Ok(CoreItemData {
        kind,
        class: if kind == ItemKind::Rock {
            ItemClass::Rock
        } else {
            class
        },
        glyph,
        weight: definition.weight,
        base_price: definition.base_price.unwrap_or_default() as u32,
        ac_bonus: definition.ac_bonus.unwrap_or_default(),
        attack_profile,
        consumable_effect,
        wand_effect: match definition.effect.as_deref() {
            Some("magic_missile") => Some(WandEffect::MagicMissile),
            Some("fire") => Some(WandEffect::Fire),
            _ => None,
        },
        ring_effect,
        armor_slot,
        tool_action,
        container_capacity: definition.container_capacity,
        max_charges: definition.charges,
        nutrition: definition.nutrition,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpseSpec {
    pub chance: u8,
    pub nutrition: i16,
    pub effects: Vec<String>,
}

pub fn monster_resistances(kind: MonsterKind, registry: &ContentRegistry) -> Resistances {
    let mut flags = Resistances::NONE;
    if let Some(def) = registry.monster(monster_id(kind)) {
        for name in &def.resistances {
            if let Some(r) = Resistances::from_name(name) {
                flags.insert(r);
            }
        }
    }
    flags
}

pub fn monster_corpse_spec(kind: MonsterKind, registry: &ContentRegistry) -> CorpseSpec {
    if let Some(def) = registry.monster(monster_id(kind)) {
        CorpseSpec {
            chance: def.corpse_chance.unwrap_or(50),
            nutrition: def.corpse_nutrition.unwrap_or(50),
            effects: def.corpse_effects.clone(),
        }
    } else {
        CorpseSpec {
            chance: 50,
            nutrition: 50,
            effects: Vec::new(),
        }
    }
}

pub fn item_extrinsic_resistances(kind: ItemKind, registry: &ContentRegistry) -> Resistances {
    let mut flags = Resistances::NONE;
    match kind {
        ItemKind::RingFireResistance => flags.insert(Resistances::FIRE),
        ItemKind::RingPoisonResistance => flags.insert(Resistances::POISON),
        ItemKind::AmuletEsp => flags.insert(Resistances::TELEPATHY),
        _ => {}
    }
    if let Some(def) = registry.item(item_id(kind)) {
        for name in &def.effects {
            if let Some(r) = Resistances::from_name(name) {
                flags.insert(r);
            }
        }
    }
    flags
}

pub fn item_display_name(kind: ItemKind, registry: &ContentRegistry) -> String {
    if let Some(name) = registry.item(item_id(kind)).and_then(|d| d.name.as_ref()) {
        return name.clone();
    }
    format!("{kind:?}")
}

pub fn monster_display_name(kind: MonsterKind, registry: &ContentRegistry) -> String {
    if let Some(name) = registry
        .monster(monster_id(kind))
        .and_then(|d| d.name.as_ref())
    {
        return name.clone();
    }
    format!("{kind:?}")
}

pub fn item_color(kind: ItemKind, registry: &ContentRegistry) -> Option<&str> {
    registry
        .item(item_id(kind))
        .and_then(|d| d.color.as_deref())
}

pub fn monster_color(kind: MonsterKind, registry: &ContentRegistry) -> Option<&str> {
    registry
        .monster(monster_id(kind))
        .and_then(|d| d.color.as_deref())
}

pub fn item_glyph(kind: ItemKind, registry: &ContentRegistry) -> char {
    registry
        .item(item_id(kind))
        .and_then(|d| d.glyph.chars().next())
        .unwrap_or('?')
}

pub fn monster_glyph(kind: MonsterKind, registry: &ContentRegistry) -> char {
    registry
        .monster(monster_id(kind))
        .and_then(|d| d.glyph.chars().next())
        .unwrap_or('?')
}

pub fn monster_has_ai_flag(kind: MonsterKind, flag: &str, registry: &ContentRegistry) -> bool {
    registry
        .monster(monster_id(kind))
        .is_some_and(|d| d.ai_flags.iter().any(|entry| entry == flag))
}

/// Content level definition을 core map에 적용할 tile override로 해석한다.
pub fn level_tile_overrides(level: &LevelData) -> Result<Vec<(Pos, TileKind)>, ContentError> {
    let mut tiles = Vec::new();
    for x in 0..level.width {
        tiles.extend([
            (Pos { x, y: 0 }, TileKind::Wall),
            (
                Pos {
                    x,
                    y: level.height - 1,
                },
                TileKind::Wall,
            ),
        ]);
    }
    for y in 0..level.height {
        tiles.extend([
            (Pos { x: 0, y }, TileKind::Wall),
            (
                Pos {
                    x: level.width - 1,
                    y,
                },
                TileKind::Wall,
            ),
        ]);
    }
    for wall in level.wall.as_deref().unwrap_or_default() {
        for y in wall.y_range[0]..=wall.y_range[1] {
            tiles.push((Pos { x: wall.x, y }, TileKind::Wall));
        }
    }
    for door in level.door.as_deref().unwrap_or_default() {
        let state = match door.state.as_str() {
            "closed" => DoorState::Closed,
            "open" => DoorState::Open,
            "locked" => DoorState::Locked,
            other => {
                return Err(ContentError::UnknownReference {
                    owner: level.level_id.clone(),
                    target: other.to_owned(),
                })
            }
        };
        tiles.push((position(level, &door.pos)?, TileKind::Door(state)));
    }
    for door in level.hidden_door.as_deref().unwrap_or_default() {
        tiles.push((position(level, &door.pos)?, TileKind::HiddenDoor));
    }
    for trap in level.hidden_trap.as_deref().unwrap_or_default() {
        let kind = match trap.trap.as_str() {
            "pit" => TrapKind::Pit,
            "dart" => TrapKind::Dart,
            "bear" => TrapKind::Bear,
            "teleport" => TrapKind::Teleport,
            "sleeping_gas" => TrapKind::SleepingGas,
            "rust" => TrapKind::Rust,
            "polymorph" => TrapKind::Polymorph,
            "hole" => TrapKind::Hole,
            "anti_magic" => TrapKind::AntiMagic,
            "falling_rock" => TrapKind::FallingRock,
            "magic" => TrapKind::Magic,
            "fire" => TrapKind::Fire,
            other => {
                return Err(ContentError::UnknownReference {
                    owner: level.level_id.clone(),
                    target: other.to_owned(),
                })
            }
        };
        tiles.push((position(level, &trap.pos)?, TileKind::HiddenTrap(kind)));
    }
    if let Some(pos) = &level.stairs_down {
        tiles.push((position(level, pos)?, TileKind::StairsDown));
    }
    if let Some(pos) = &level.stairs_up {
        tiles.push((position(level, pos)?, TileKind::StairsUp));
    }
    Ok(tiles)
}

impl MapLayout for LevelData {
    fn level_id(&self) -> &str {
        &self.level_id
    }
    fn depth(&self) -> i16 {
        self.depth
    }
    fn dimensions(&self) -> (i16, i16) {
        (self.width, self.height)
    }
    fn tile_overrides(&self) -> Result<Vec<(Pos, TileKind)>, ContentError> {
        level_tile_overrides(self)
    }
}

fn position(level: &LevelData, value: &[i16]) -> Result<Pos, ContentError> {
    let [x, y] = value else {
        return Err(ContentError::InvalidCoordinate {
            level: level.level_id.clone(),
            x: 0,
            y: 0,
        });
    };
    Ok(Pos { x: *x, y: *y })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelSpawn {
    Monster { kind: MonsterKind, pos: Pos },
    Item { kind: ItemKind, pos: Pos },
}

/// Validated level reference를 core가 소비하는 typed spawn plan으로 변환한다.
pub fn level_spawns(level: &LevelData) -> Result<Vec<LevelSpawn>, ContentError> {
    let mut spawns = Vec::new();
    for monster in level.monster.as_deref().unwrap_or_default() {
        let kind =
            monster_kind_from_id(&monster.id).map_err(|_| ContentError::UnknownReference {
                owner: level.level_id.clone(),
                target: monster.id.clone(),
            })?;
        spawns.push(LevelSpawn::Monster {
            kind,
            pos: position(level, &monster.pos)?,
        });
    }
    for item in level.item.as_deref().unwrap_or_default() {
        let kind = item_kind_from_id(&item.id).map_err(|_| ContentError::UnknownReference {
            owner: level.level_id.clone(),
            target: item.id.clone(),
        })?;
        spawns.push(LevelSpawn::Item {
            kind,
            pos: position(level, &item.pos)?,
        });
    }
    Ok(spawns)
}
