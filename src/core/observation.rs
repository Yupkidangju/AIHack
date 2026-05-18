use serde::{Deserialize, Serialize};

use crate::{
    core::{
        action::{ActionIntent, CommandIntent},
        event::GameEvent,
        ids::{EntityId, LevelId},
        position::{Delta, Direction, Pos},
        session::RunState,
        world::GameWorld,
    },
    domain::{
        entity::{EntityKind, EntityLocation},
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

/// [v0.2.0] Phase 16: spec.md 8.2 계약과 일치하는 AI run-state 요약이다.
/// GameOver는 summary 레벨에서 cause/final_score를 생략하고 단순 상태만 표시한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStateSummary {
    Title,
    CharacterCreation,
    Playing,
    AwaitingDirection,
    AwaitingInventorySelection,
    MorePrompt,
    GameOver,
}

/// [v0.1.0] Phase 11 플레이어 요약 관찰값이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerObservation {
    pub entity: EntityId,
    pub pos: Pos,
    pub hp: i16,
    pub max_hp: i16,
    pub current_level: LevelId,
    pub hunger: i16,
    pub luck: i16,
    pub prayer_cooldown: u16,
    pub paralysis_turns: u8,
    pub hallucinating: bool,
}

/// [v0.1.0] Phase 11 AI visible entity 요약이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityObservation {
    pub entity: EntityId,
    pub kind: EntityKind,
    pub pos: Pos,
    pub hp: Option<i16>,
    pub alive: bool,
}

/// [v0.1.0] Phase 11 action space contract다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionSpace {
    pub commands: Vec<ActionIntent>,
}

/// [v0.1.0] Phase 11 canonical observation schema다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub schema_version: u16,
    pub seed: u64,
    pub turn: u64,
    pub current_level: LevelId,
    pub run_state: RunStateSummary,
    pub player: PlayerObservation,
    pub player_pos: Pos,
    pub visible_tiles: Vec<TileObservation>,
    pub visible_entities: Vec<EntityObservation>,
    pub inventory: Vec<ItemObservation>,
    pub last_events: Vec<GameEvent>,
    pub action_space: ActionSpace,
    pub legal_actions: Vec<CommandIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemObservation {
    pub item: EntityId,
    pub kind: ItemKind,
    pub letter: InventoryLetter,
    pub equipped_slot: Option<EquipmentSlot>,
    pub identified: bool,
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
    pub fn from_world(
        seed: u64,
        turn: u64,
        run_state: RunState,
        event_log: &[GameEvent],
        world: &GameWorld,
    ) -> Self {
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
        Self {
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

/// [v0.2.0] Phase 16: RunState에 따라 legal_actions를 생성한다.
fn legal_actions(world: &GameWorld, run_state: RunState) -> Vec<CommandIntent> {
    match run_state {
        RunState::Title => vec![CommandIntent::Wait, CommandIntent::Quit],
        RunState::CharacterCreation => vec![CommandIntent::Wait, CommandIntent::Quit],
        RunState::MorePrompt => vec![CommandIntent::AcknowledgeMore],
        RunState::GameOver { .. } => vec![CommandIntent::Quit],
        RunState::AwaitingDirection { .. } => {
            let mut actions: Vec<CommandIntent> = Direction::ALL
                .iter()
                .copied()
                .map(CommandIntent::Move)
                .collect();
            actions.push(CommandIntent::Quit);
            actions
        }
        RunState::AwaitingInventorySelection { action } => {
            let mut actions = Vec::new();
            for entry in &world.inventory.entries {
                let item = entry.item;
                match action {
                    crate::core::action::InventoryAction::Drop => {
                        actions.push(CommandIntent::Drop { item });
                    }
                    crate::core::action::InventoryAction::Wield => {
                        if world
                            .entities
                            .item_data(item)
                            .map(|d| d.class == ItemClass::Weapon)
                            .unwrap_or(false)
                        {
                            actions.push(CommandIntent::Wield { item });
                        }
                    }
                    crate::core::action::InventoryAction::Wear => {
                        if world
                            .entities
                            .item_data(item)
                            .map(|d| d.class == ItemClass::Armor)
                            .unwrap_or(false)
                        {
                            actions.push(CommandIntent::Wear { item });
                        }
                    }
                    crate::core::action::InventoryAction::Quaff => {
                        if world
                            .entities
                            .item_data(item)
                            .map(|d| d.class == ItemClass::Potion)
                            .unwrap_or(false)
                        {
                            actions.push(CommandIntent::Quaff { item });
                        }
                    }
                    crate::core::action::InventoryAction::Read => {
                        if world
                            .entities
                            .item_data(item)
                            .map(|d| d.class == ItemClass::Scroll)
                            .unwrap_or(false)
                        {
                            actions.push(CommandIntent::Read { item });
                        }
                    }
                }
            }
            actions.push(CommandIntent::Quit);
            actions
        }
        RunState::Playing => {
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
    }
}
