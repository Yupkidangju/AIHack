use aihack_core::{
    action::{ActionIntent, CommandIntent, InventoryAction},
    domain::{
        entity::EntityLocation,
        item::{EquipmentSlot, ItemClass},
        tile::{DoorState, TileKind},
    },
    event::GameEvent,
    position::Direction,
    run_state::RunState,
};

use crate::{
    systems::{
        doors::door_state_in_direction,
        movement::{is_bump_attack_for_legal_action, is_passable_for_legal_action},
        vision::visible_positions,
    },
    world::GameWorld,
};

pub use aihack_ai_contract::{
    ActionSpace, EntityObservation, ItemObservation, Observation, PlayerObservation,
    RunStateSummary, TileObservation, OBSERVATION_SCHEMA_VERSION,
};

pub fn from_world(
    seed: u64,
    turn: u64,
    run_state: RunState,
    event_log: &[GameEvent],
    world: &GameWorld,
) -> Observation {
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
                    tile: tile.observation_equivalent(),
                    visible: true,
                })
        })
        .collect::<Vec<_>>();
    visible_tiles.sort_by_key(|tile| (tile.pos.y, tile.pos.x));

    let legal_actions = legal_actions(world, run_state);
    Observation {
        schema_version: OBSERVATION_SCHEMA_VERSION,
        seed,
        turn,
        current_level: world.current_level(),
        run_state: run_state_summary(run_state),
        player: player_observation(world),
        player_pos: world.player_pos(),
        visible_tiles,
        visible_entities: visible_entities(world),
        inventory: inventory_observations(world),
        last_events: event_log
            .iter()
            .rev()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect(),
        action_space: ActionSpace {
            commands: legal_actions
                .iter()
                .copied()
                .map(ActionIntent::Command)
                .collect(),
        },
        legal_actions,
    }
}

fn run_state_summary(run_state: RunState) -> RunStateSummary {
    match run_state {
        RunState::Title => RunStateSummary::Title,
        RunState::CharacterCreation => RunStateSummary::CharacterCreation,
        RunState::Playing => RunStateSummary::Playing,
        RunState::AwaitingDirection { .. } => RunStateSummary::AwaitingDirection,
        RunState::AwaitingInventorySelection { .. } => RunStateSummary::AwaitingInventorySelection,
        RunState::MorePrompt => RunStateSummary::MorePrompt,
        RunState::GameOver { .. } => RunStateSummary::GameOver,
    }
}

fn player_observation(world: &GameWorld) -> PlayerObservation {
    let stats = world
        .entities
        .actor_stats(world.player_id)
        .expect("player stats must exist");
    PlayerObservation {
        entity: world.player_id,
        pos: world.player_pos(),
        hp: stats.hp,
        max_hp: stats.max_hp,
        current_level: world.current_level(),
        hunger: world.nutrition,
        luck: world.luck,
        prayer_cooldown: world.prayer_cooldown,
        paralysis_turns: world.paralysis_turns,
        hallucinating: world.hallucinating,
    }
}

fn visible_entities(world: &GameWorld) -> Vec<EntityObservation> {
    let visible = visible_positions(world);
    let mut entities = world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| {
            if entity.id == world.player_id {
                return None;
            }
            if let Some((_, _, level, pos, stats, alive)) = entity.actor() {
                if level == world.current_level() && visible.contains(&pos) {
                    return Some(EntityObservation {
                        entity: entity.id,
                        kind: entity.kind(),
                        pos,
                        hp: Some(stats.hp),
                        alive,
                    });
                }
            }
            None
        })
        .collect::<Vec<_>>();
    entities.sort_by_key(|entity| entity.entity.0);
    entities
}

fn inventory_observations(world: &GameWorld) -> Vec<ItemObservation> {
    world
        .inventory
        .entries
        .iter()
        .filter_map(|entry| {
            let (kind, _, location, _, _) = world.entities.get(entry.item)?.item()?;
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
                equipped_slot: if world.inventory.equipped_melee == Some(entry.item) {
                    Some(EquipmentSlot::Melee)
                } else if world.inventory.equipped_body == Some(entry.item) {
                    Some(EquipmentSlot::Body)
                } else {
                    None
                },
                identified: world.is_item_identified(kind),
            })
        })
        .collect()
}

fn legal_actions(world: &GameWorld, run_state: RunState) -> Vec<CommandIntent> {
    match run_state {
        RunState::Title | RunState::CharacterCreation => {
            vec![CommandIntent::Wait, CommandIntent::Quit]
        }
        RunState::MorePrompt => vec![CommandIntent::AcknowledgeMore],
        RunState::GameOver { .. } => vec![CommandIntent::Quit],
        RunState::AwaitingDirection { .. } => {
            let mut actions = Direction::ALL
                .iter()
                .copied()
                .map(CommandIntent::Move)
                .collect::<Vec<_>>();
            actions.push(CommandIntent::Quit);
            actions
        }
        RunState::AwaitingInventorySelection { action } => {
            let mut actions = Vec::new();
            for entry in &world.inventory.entries {
                let item = entry.item;
                match action {
                    InventoryAction::Drop => actions.push(CommandIntent::Drop { item }),
                    InventoryAction::Wield => {
                        if item_has_class(world, item, ItemClass::Weapon) {
                            actions.push(CommandIntent::Wield { item });
                        }
                    }
                    InventoryAction::Wear => {
                        if item_has_class(world, item, ItemClass::Armor) {
                            actions.push(CommandIntent::Wear { item });
                        }
                    }
                    InventoryAction::Quaff => {
                        if item_has_class(world, item, ItemClass::Potion) {
                            actions.push(CommandIntent::Quaff { item });
                        }
                    }
                    InventoryAction::Read => {
                        if item_has_class(world, item, ItemClass::Scroll) {
                            actions.push(CommandIntent::Read { item });
                        }
                    }
                }
            }
            actions.push(CommandIntent::Quit);
            actions
        }
        RunState::Playing => playing_actions(world),
    }
}

fn playing_actions(world: &GameWorld) -> Vec<CommandIntent> {
    let mut actions = vec![
        CommandIntent::Wait,
        CommandIntent::Search,
        CommandIntent::Pray,
        CommandIntent::ShowInventory,
    ];
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
            if data.class == ItemClass::Armor {
                actions.push(CommandIntent::Wear { item: entry.item });
            }
            if data.class == ItemClass::Potion {
                actions.push(CommandIntent::Quaff { item: entry.item });
            }
            if data.class == ItemClass::Scroll {
                actions.push(CommandIntent::Read { item: entry.item });
            }
            if matches!(data.class, ItemClass::Weapon | ItemClass::Rock) {
                for direction in Direction::ALL {
                    actions.push(CommandIntent::Throw {
                        item: entry.item,
                        direction,
                    });
                }
            }
            if data.class == ItemClass::Wand
                && world.entities.item_charges(entry.item).unwrap_or(0) > 0
            {
                for direction in Direction::ALL {
                    actions.push(CommandIntent::Zap {
                        item: entry.item,
                        direction,
                    });
                }
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
        actions.push(CommandIntent::Kick(direction));
    }
    for entry in &world.inventory.entries {
        actions.push(CommandIntent::Drop { item: entry.item });
    }
    actions
}

fn item_has_class(world: &GameWorld, item: aihack_core::ids::EntityId, class: ItemClass) -> bool {
    world
        .entities
        .item_data(item)
        .map(|data| data.class == class)
        .unwrap_or(false)
}
