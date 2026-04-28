use serde::{Deserialize, Serialize};

use crate::{
    core::{
        action::CommandIntent,
        ids::{EntityId, LevelId},
        position::Delta,
        position::Direction,
        position::Pos,
        world::GameWorld,
    },
    domain::{
        entity::EntityLocation,
        inventory::InventoryLetter,
        item::{EquipmentSlot, ItemClass, ItemKind},
        tile::{DoorState, TileKind},
    },
    systems::{
        doors::door_state_in_direction,
        movement::{is_bump_attack_for_legal_action, is_passable_for_legal_action},
        vision::visible_positions,
    },
};

pub const OBSERVATION_SCHEMA_VERSION: u16 = 1;

/// [v0.1.0] Phase 4 최소 관찰 스키마다. inventory를 typed data로 노출한다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub current_level: LevelId,
    pub player_pos: Pos,
    pub visible_tiles: Vec<TileObservation>,
    pub inventory: Vec<ItemObservation>,
    pub legal_actions: Vec<CommandIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemObservation {
    pub item: EntityId,
    pub kind: ItemKind,
    pub letter: InventoryLetter,
    pub equipped_slot: Option<EquipmentSlot>,
}

/// [v0.1.0] UI/AI가 glyph 대신 typed tile 정보를 읽게 하는 최소 tile 관찰값이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TileObservation {
    pub pos: Pos,
    pub rel: Delta,
    pub tile: TileKind,
    pub visible: bool,
}

impl Observation {
    pub fn from_world(seed: u64, turn: u64, world: &GameWorld) -> Self {
        let mut visible_tiles = visible_positions(world)
            .into_iter()
            .filter_map(|pos| {
                world
                    .current_map()
                    .tile(pos)
                    .ok()
                    .map(|tile| TileObservation {
                        pos,
                        rel: world.player_pos().delta_to(pos),
                        tile,
                        visible: true,
                    })
            })
            .collect::<Vec<_>>();
        visible_tiles.sort_by_key(|tile| (tile.pos.y, tile.pos.x));

        Self {
            schema_version: OBSERVATION_SCHEMA_VERSION,
            seed,
            turn,
            current_level: world.current_level(),
            player_pos: world.player_pos(),
            visible_tiles,
            inventory: inventory_observations(world),
            legal_actions: legal_actions(world),
        }
    }
}

fn inventory_observations(world: &GameWorld) -> Vec<ItemObservation> {
    world
        .inventory
        .entries
        .iter()
        .filter_map(|entry| {
            let (kind, _, location, _) = world.entities.get(entry.item)?.item()?;
            if location
                != (EntityLocation::Inventory {
                    owner: world.player_id,
                })
            {
                return None;
            }
            Some(ItemObservation {
                item: entry.item,
                kind,
                letter: entry.letter,
                equipped_slot: (world.inventory.equipped_melee == Some(entry.item))
                    .then_some(EquipmentSlot::Melee),
            })
        })
        .collect()
}

fn legal_actions(world: &GameWorld) -> Vec<CommandIntent> {
    let mut actions = vec![CommandIntent::Wait, CommandIntent::ShowInventory];
    if world
        .entities
        .item_at(world.current_level(), world.player_pos())
        .is_some()
    {
        actions.push(CommandIntent::Pickup);
    }
    match world.current_map().tile(world.player_pos()) {
        Ok(TileKind::StairsDown) => actions.push(CommandIntent::Descend),
        Ok(TileKind::StairsUp) => actions.push(CommandIntent::Ascend),
        _ => {}
    }
    for entry in &world.inventory.entries {
        if let Some(data) = world.entities.item_data(entry.item) {
            if data.class == ItemClass::Weapon {
                actions.push(CommandIntent::Wield { item: entry.item });
            }
            if data.class == ItemClass::Potion {
                actions.push(CommandIntent::Quaff { item: entry.item });
            }
        }
    }
    for direction in Direction::ALL {
        if is_passable_for_legal_action(world, direction)
            || is_bump_attack_for_legal_action(world, direction)
        {
            actions.push(CommandIntent::Move(direction));
        }
        match door_state_in_direction(world, direction) {
            Some(DoorState::Closed) => actions.push(CommandIntent::Open(direction)),
            Some(DoorState::Open) => actions.push(CommandIntent::Close(direction)),
            None => {}
        }
    }
    actions
}
