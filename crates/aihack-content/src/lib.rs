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
    LevelMonsterData, MonsterData, WallData, CONTENT_SCHEMA_VERSION, MAX_CONTENT_ARMOR_AC_BONUS,
    MAX_CONTENT_BASE_PRICE, MAX_CONTENT_MONSTER_HP, MAX_CONTENT_NUTRITION,
};

use aihack_core::domain::map::MapLayout;
use aihack_core::domain::{
    combat::{AttackProfile, DamageRoll},
    item::{ConsumableEffect, ItemClass, ItemData as CoreItemData, ItemKind, WandEffect},
    monster::{MonsterAiKind, MonsterKind, MonsterPassive, MonsterTemplate},
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

/// Registry definition을 core가 소비하는 monster template으로 변환한다.
pub fn monster_template_from_registry(
    kind: MonsterKind,
    registry: &ContentRegistry,
) -> Result<MonsterTemplate, ContentError> {
    let id = match kind {
        MonsterKind::Jackal => "monster.jackal",
        MonsterKind::Goblin => "monster.goblin",
        MonsterKind::FloatingEye => "monster.floating_eye",
    };
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
    })
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
    let id = match kind {
        ItemKind::Dagger => "item.weapon.dagger",
        ItemKind::FoodRation => "item.food.ration",
        ItemKind::PotionHealing => "item.potion.healing",
        ItemKind::WandMagicMissile => "item.wand.magic_missile",
        ItemKind::ScrollReveal => "item.scroll.reveal",
        ItemKind::ScrollIdentify => "item.scroll.identify",
        ItemKind::ScrollLevelTeleport => "item.scroll.teleport",
        ItemKind::Rock => "item.weapon.rock",
        ItemKind::ArmorLeather => "item.armor.leather",
        ItemKind::CorpseJackal => "item.corpse.jackal",
        ItemKind::AmuletAscension => "item.quest.ascension",
    };
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
        "corpse" => ItemClass::Corpse,
        "quest" => ItemClass::Quest,
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
    let consumable_effect = match definition.effect.as_deref() {
        Some("heal_1d8_plus_4") => Some(ConsumableEffect::Heal {
            dice: 1,
            sides: 8,
            bonus: 4,
        }),
        Some("reveal") => Some(ConsumableEffect::RevealLevel),
        Some("identify") => Some(ConsumableEffect::IdentifySingle),
        Some("teleport") => Some(ConsumableEffect::LevelTeleport),
        Some("magic_missile") | None => None,
        Some(effect) => {
            return Err(ContentError::UnknownReference {
                owner: id.to_owned(),
                target: effect.to_owned(),
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
        wand_effect: (definition.effect.as_deref() == Some("magic_missile"))
            .then_some(WandEffect::MagicMissile),
        max_charges: definition.charges,
        nutrition: definition.nutrition,
    })
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
        if trap.trap != "pit" {
            return Err(ContentError::UnknownReference {
                owner: level.level_id.clone(),
                target: trap.trap.clone(),
            });
        }
        tiles.push((
            position(level, &trap.pos)?,
            TileKind::HiddenTrap(TrapKind::Pit),
        ));
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
        let kind = match monster.id.as_str() {
            "monster.jackal" => MonsterKind::Jackal,
            "monster.goblin" => MonsterKind::Goblin,
            "monster.floating_eye" => MonsterKind::FloatingEye,
            _ => {
                return Err(ContentError::UnknownReference {
                    owner: level.level_id.clone(),
                    target: monster.id.clone(),
                })
            }
        };
        spawns.push(LevelSpawn::Monster {
            kind,
            pos: position(level, &monster.pos)?,
        });
    }
    for item in level.item.as_deref().unwrap_or_default() {
        let kind = match item.id.as_str() {
            "item.weapon.dagger" => ItemKind::Dagger,
            "item.food.ration" => ItemKind::FoodRation,
            "item.potion.healing" => ItemKind::PotionHealing,
            "item.wand.magic_missile" => ItemKind::WandMagicMissile,
            "item.scroll.reveal" => ItemKind::ScrollReveal,
            "item.scroll.identify" => ItemKind::ScrollIdentify,
            "item.scroll.teleport" => ItemKind::ScrollLevelTeleport,
            "item.weapon.rock" => ItemKind::Rock,
            "item.armor.leather" => ItemKind::ArmorLeather,
            "item.corpse.jackal" => ItemKind::CorpseJackal,
            "item.quest.ascension" => ItemKind::AmuletAscension,
            _ => {
                return Err(ContentError::UnknownReference {
                    owner: level.level_id.clone(),
                    target: item.id.clone(),
                })
            }
        };
        spawns.push(LevelSpawn::Item {
            kind,
            pos: position(level, &item.pos)?,
        });
    }
    Ok(spawns)
}
