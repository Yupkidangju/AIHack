use aihack_core::{
    action::{
        ActionIntent, CommandIntent, DipTarget, EngravingMethod, EngravingText, InventoryAction,
    },
    domain::{
        entity::EntityLocation,
        item::{BucStatus, EquipmentSlot, ItemClass, ItemKind, ToolAction, WandEffect},
        tile::{DoorState, TileKind},
    },
    event::GameEvent,
    ids::EntityId,
    position::{Direction, Pos},
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
    ActionSpace, ContainerItemObservation, EntityObservation, ItemObservation, Observation,
    PlayerObservation, RunStateSummary, TileObservation, OBSERVATION_SCHEMA_VERSION,
};

fn tile_engraving(world: &GameWorld, pos: Pos) -> Option<String> {
    world.campaign?;
    world.levels.get(world.current_level()).and_then(|level| {
        level
            .engravings
            .iter()
            .find(|entry| entry.pos == pos)
            .map(|entry| entry.text.clone())
    })
}

pub fn from_world(
    seed: u64,
    turn: u64,
    run_state: RunState,
    event_log: &[GameEvent],
    world: &GameWorld,
) -> Observation {
    let visible_list = visible_positions(world);
    let visible_set: std::collections::HashSet<_> = visible_list.iter().copied().collect();
    let mut visible_tiles = visible_list
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
                    engraving: tile_engraving(world, pos),
                })
        })
        .collect::<Vec<_>>();
    if let Some(level) = world.levels.get(world.current_level()) {
        for pos in level.explored_positions() {
            if !visible_set.contains(&pos) {
                if let Ok(tile) = level.map.tile(pos) {
                    visible_tiles.push(TileObservation {
                        pos,
                        rel: world.player_pos().delta_to(pos),
                        tile: tile.observation_equivalent(),
                        visible: false,
                        engraving: tile_engraving(world, pos),
                    });
                }
            }
        }
    }
    visible_tiles.sort_by_key(|tile| (tile.pos.y, tile.pos.x));

    let legal_actions = legal_actions(world, run_state);
    Observation {
        campaign: world
            .campaign
            .map(|campaign| aihack_ai_contract::CampaignObservation {
                carried_weight: world.carried_weight(),
                carrying_capacity: world.carrying_capacity(),
                role: campaign.role,
                xp: campaign.xp,
                level: campaign.level(),
                has_amulet: crate::campaign::has_amulet(world),
                identified_items: world.identified_items.clone(),
                attributes: Some(campaign.attributes),
                alignment: Some(campaign.alignment),
                race: Some(campaign.race),
            }),
        schema_version: OBSERVATION_SCHEMA_VERSION,
        seed,
        turn,
        current_level: world.current_level(),
        run_state: run_state_summary(run_state),
        player: player_observation(world),
        player_pos: world.player_pos(),
        visible_tiles,
        visible_entities: visible_entities(seed, world),
        inventory: inventory_observations(seed, world),
        container_items: container_observations(seed, world),
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
                .take(64)
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
        RunState::Victory { .. } => RunStateSummary::Victory,
    }
}

fn player_observation(world: &GameWorld) -> PlayerObservation {
    let stats = world
        .entities
        .actor_stats(world.player_id)
        .expect("player stats must exist");
    let resistances = if world.campaign.is_some() {
        world.effective_player_resistances().0
    } else {
        0
    };
    PlayerObservation {
        entity: world.player_id,
        pos: world.player_pos(),
        hp: stats.hp,
        max_hp: stats.max_hp,
        ac: stats.ac,
        current_level: world.current_level(),
        hunger: world.nutrition,
        luck: world.luck,
        prayer_cooldown: world.prayer_cooldown,
        paralysis_turns: world.paralysis_turns,
        hallucinating: world.hallucinating,
        poison_turns: world.poison_turns,
        gold: world.gold,
        fire_resistant: crate::systems::items::has_fire_resistance(world),
        poison_resistant: crate::systems::items::has_poison_resistance(world),
        mp: world.mp,
        max_mp: world.max_mp,
        resistances,
        blind_turns: world.status_timers.blind,
        confused_turns: world.status_timers.confused,
        stunned_turns: world.status_timers.stunned,
        fast_turns: world.status_timers.fast,
        levitation_turns: world.status_timers.levitation,
        invisible_turns: world.status_timers.invisible,
        blindfolded: world.campaign.is_some()
            && crate::systems::tools::is_player_blindfolded(world),
    }
}

fn visible_entities(seed: u64, world: &GameWorld) -> Vec<EntityObservation> {
    let visible = visible_positions(world);
    let has_telepathy = world.campaign.is_some()
        && world
            .effective_player_resistances()
            .contains(aihack_core::domain::status::Resistances::TELEPATHY);
    let registry = world.content_registry();
    let mut entities = world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| {
            if entity.id == world.player_id {
                return None;
            }
            if let Some((actor_kind, _, level, pos, stats, alive)) = entity.actor() {
                if alive
                    && level == world.current_level()
                    && (visible.contains(&pos) || has_telepathy)
                {
                    let (glyph, name, color) = if world.campaign.is_some() {
                        match actor_kind {
                            aihack_core::domain::entity::ActorKind::Monster(mk) => (
                                Some(aihack_content::monster_glyph(mk, registry)),
                                Some(aihack_content::monster_display_name(mk, registry)),
                                aihack_content::monster_color(mk, registry).map(str::to_owned),
                            ),
                            aihack_core::domain::entity::ActorKind::Player => {
                                (Some('@'), Some("player".to_owned()), None)
                            }
                        }
                    } else {
                        (None, None, None)
                    };
                    return Some(EntityObservation {
                        entity: entity.id,
                        kind: entity.kind(),
                        pos,
                        hp: Some(stats.hp),
                        alive,
                        appearance: None,
                        glyph,
                        name,
                        color,
                    });
                }
            }
            if let Some((kind, _, EntityLocation::OnMap { level, pos }, _, _)) = entity.item() {
                if level == world.current_level() && visible.contains(&pos) {
                    let appearance = (world.campaign.is_some() && !world.is_item_identified(kind))
                        .then(|| {
                            crate::appearance::appearance_for_kind(seed, kind).map(str::to_owned)
                        })
                        .flatten();
                    let (glyph, name, color) = if world.campaign.is_some() {
                        let name = if let Some(ref app) = appearance {
                            app.clone()
                        } else if let Some(mk) = entity.item_corpse_monster() {
                            format!(
                                "{} corpse",
                                aihack_content::monster_display_name(mk, registry)
                            )
                        } else {
                            aihack_content::item_display_name(kind, registry)
                        };
                        (
                            Some(aihack_content::item_glyph(kind, registry)),
                            Some(name),
                            aihack_content::item_color(kind, registry).map(str::to_owned),
                        )
                    } else {
                        (None, None, None)
                    };
                    return Some(EntityObservation {
                        entity: entity.id,
                        kind: entity.kind(),
                        pos,
                        hp: None,
                        alive: true,
                        appearance,
                        glyph,
                        name,
                        color,
                    });
                }
            }
            None
        })
        .collect::<Vec<_>>();
    entities.sort_by_key(|entity| entity.entity.0);
    entities
}

fn inventory_observations(seed: u64, world: &GameWorld) -> Vec<ItemObservation> {
    let registry = world.content_registry();
    world
        .inventory
        .entries
        .iter()
        .filter_map(|entry| {
            let entity = world.entities.get(entry.item)?;
            let (kind, data, location, _, _) = entity.item()?;
            if location
                != (EntityLocation::Inventory {
                    owner: world.player_id,
                })
            {
                return None;
            }
            let identified = world.is_item_identified(kind);
            let (buc_known, enchant_known) = if world.campaign.is_some() {
                (
                    entity.item_buc_known().unwrap_or(false),
                    entity.item_enchant_known().unwrap_or(false),
                )
            } else {
                (true, true)
            };
            let buc = if buc_known {
                entity.item_buc().unwrap_or(BucStatus::Uncursed)
            } else {
                BucStatus::Uncursed
            };
            let enchantment = if enchant_known {
                entity.item_enchantment().unwrap_or(0)
            } else {
                0
            };
            let appearance = (world.campaign.is_some() && !identified)
                .then(|| crate::appearance::appearance_for_kind(seed, kind).map(str::to_owned))
                .flatten();
            let (glyph, name, color, item_class) = if world.campaign.is_some() {
                let item_name = if let Some(ref app) = appearance {
                    app.clone()
                } else if let Some(mk) = entity.item_corpse_monster() {
                    format!(
                        "{} corpse",
                        aihack_content::monster_display_name(mk, registry)
                    )
                } else {
                    aihack_content::item_display_name(kind, registry)
                };
                (
                    Some(aihack_content::item_glyph(kind, registry)),
                    Some(item_name),
                    aihack_content::item_color(kind, registry).map(str::to_owned),
                    Some(data.class),
                )
            } else {
                (None, None, None, None)
            };
            let (contained_count, tool_active, container_locked, erosion) =
                if world.campaign.is_some() {
                    (
                        u16::try_from(
                            entity
                                .item_contained_items()
                                .map(<[EntityId]>::len)
                                .unwrap_or(0),
                        )
                        .unwrap_or(u16::MAX),
                        entity.item_tool_active().unwrap_or(false),
                        entity.item_container_locked().unwrap_or(false),
                        entity.item_erosion().unwrap_or(0),
                    )
                } else {
                    (0, false, false, 0)
                };
            Some(ItemObservation {
                item: entry.item,
                kind,
                letter: entry.letter,
                equipped_slot: world.inventory.slot_of(entry.item),
                identified,
                buc_known,
                enchant_known,
                buc,
                enchantment,
                quantity: entity.item_quantity().unwrap_or(1),
                appearance,
                glyph,
                name,
                color,
                item_class,
                contained_count,
                tool_active,
                container_locked,
                erosion,
            })
        })
        .collect()
}

fn container_observations(seed: u64, world: &GameWorld) -> Vec<ContainerItemObservation> {
    if world.campaign.is_none() {
        return Vec::new();
    }
    let registry = world.content_registry();
    let mut containers = Vec::new();
    for entry in &world.inventory.entries {
        if world
            .entities
            .item_data(entry.item)
            .is_some_and(|data| data.container_capacity.is_some())
        {
            containers.push(entry.item);
        }
    }
    for floor_id in world
        .entities
        .items_at(world.current_level(), world.player_pos())
    {
        if world
            .entities
            .item_data(floor_id)
            .is_some_and(|data| data.container_capacity.is_some())
            && !containers.contains(&floor_id)
        {
            containers.push(floor_id);
        }
    }
    let mut out = Vec::new();
    for container_id in containers {
        let Some(container_entity) = world.entities.get(container_id) else {
            continue;
        };
        if container_entity.item_container_locked() == Some(true) {
            continue;
        }
        for &child_id in container_entity.item_contained_items().unwrap_or_default() {
            let Some(child) = world.entities.get(child_id) else {
                continue;
            };
            let Some((kind, _, _, _, _)) = child.item() else {
                continue;
            };
            let identified = world.is_item_identified(kind);
            let buc_known = child.item_buc_known().unwrap_or(false);
            let enchant_known = child.item_enchant_known().unwrap_or(false);
            let buc = if buc_known {
                child.item_buc().unwrap_or(BucStatus::Uncursed)
            } else {
                BucStatus::Uncursed
            };
            let enchantment = if enchant_known {
                child.item_enchantment().unwrap_or(0)
            } else {
                0
            };
            let appearance = (!identified)
                .then(|| crate::appearance::appearance_for_kind(seed, kind).map(str::to_owned))
                .flatten();
            let name = if let Some(app) = appearance {
                app
            } else if let Some(mk) = child.item_corpse_monster() {
                format!(
                    "{} corpse",
                    aihack_content::monster_display_name(mk, registry)
                )
            } else {
                aihack_content::item_display_name(kind, registry)
            };
            out.push(ContainerItemObservation {
                container: container_id,
                item: child_id,
                kind,
                name,
                quantity: child.item_quantity().unwrap_or(1),
                buc_known,
                buc,
                enchant_known,
                enchantment,
                erosion: child.item_erosion().unwrap_or(0),
            });
        }
    }
    out
}

fn legal_actions(world: &GameWorld, run_state: RunState) -> Vec<CommandIntent> {
    match run_state {
        RunState::CharacterCreation => {
            let mut actions = vec![CommandIntent::Wait, CommandIntent::Quit];
            actions.extend(
                [
                    aihack_core::campaign::Role::Knight,
                    aihack_core::campaign::Role::Scout,
                    aihack_core::campaign::Role::Mage,
                    aihack_core::campaign::Role::Cleric,
                ]
                .map(|role| CommandIntent::StartCampaign { role }),
            );
            actions
        }
        RunState::Title => {
            vec![CommandIntent::Wait, CommandIntent::Quit]
        }
        RunState::MorePrompt => vec![CommandIntent::AcknowledgeMore],
        RunState::GameOver { .. } | RunState::Victory { .. } => vec![CommandIntent::Quit],
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
                    InventoryAction::Eat => {
                        if item_is_edible(world, item) {
                            actions.push(CommandIntent::Eat { item });
                        }
                    }
                    InventoryAction::Read => {
                        if item_has_class(world, item, ItemClass::Scroll) {
                            actions.push(CommandIntent::Read { item });
                        }
                    }
                    InventoryAction::PutOn => {
                        if item_has_class(world, item, ItemClass::Ring) {
                            actions.push(CommandIntent::PutOn { item });
                        }
                    }
                    InventoryAction::Sell => {
                        if crate::systems::shop::is_on_shop(world)
                            && world.entities.item_data(item).is_some_and(|data| {
                                data.kind != aihack_core::domain::item::ItemKind::AmuletAscension
                            })
                        {
                            actions.push(CommandIntent::Sell { item });
                        }
                    }
                    InventoryAction::Offer => {
                        if crate::systems::altar::is_on_altar(world)
                            && world.entities.item_data(item).is_some_and(|data| {
                                data.kind != aihack_core::domain::item::ItemKind::AmuletAscension
                            })
                        {
                            actions.push(CommandIntent::Offer { item });
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
    if world.paralysis_turns > 0 {
        return vec![CommandIntent::Wait, CommandIntent::Quit];
    }
    let mut actions = vec![
        CommandIntent::Wait,
        CommandIntent::Search,
        CommandIntent::ShowInventory,
    ];
    if world.prayer_cooldown == 0 {
        actions.push(CommandIntent::Pray);
    }
    if world
        .entities
        .item_at(world.current_level(), world.player_pos())
        .is_some()
    {
        actions.push(CommandIntent::Pickup);
    }
    match world.current_map().tile(world.player_pos()) {
        Ok(TileKind::StairsDown) => actions.push(CommandIntent::Descend),
        Ok(tile @ TileKind::StairsUp) => {
            if world.campaign.is_none()
                || world.current_level() != aihack_core::ids::LevelId::main(1)
                || crate::campaign::has_amulet(world)
            {
                actions.push(CommandIntent::Ascend);
            }
            if world.campaign.is_some()
                && aihack_core::domain::level::BranchGraph::campaign_v3()
                    .branch_entry_from(world.current_level(), tile)
                    .is_some()
            {
                actions.push(CommandIntent::EnterBranch);
            }
        }
        Ok(tile) if world.campaign.is_some() => {
            if matches!(tile, TileKind::Fountain | TileKind::Sink) {
                actions.push(CommandIntent::Drink);
            }
            if aihack_core::domain::level::BranchGraph::campaign_v3()
                .branch_entry_from(world.current_level(), tile)
                .is_some()
            {
                actions.push(CommandIntent::EnterBranch);
            }
        }
        _ => {}
    }
    if world.inventory.equipped_ring.is_some() || world.inventory.equipped_right_ring.is_some() {
        actions.push(CommandIntent::RemoveRing);
    }
    if world.campaign.is_some() {
        for (slot, item_opt) in world.inventory.equipped_slots() {
            if item_opt.is_some() && slot != EquipmentSlot::Melee {
                actions.push(CommandIntent::TakeOff { slot });
            }
        }
    }
    if crate::systems::shop::is_on_shop(world) {
        for kind in crate::systems::shop::SHOP_CATALOG {
            if crate::systems::shop::can_buy_slot(world, kind)
                && crate::systems::shop::buy_price_for_world(world, kind)
                    .is_ok_and(|price| world.gold >= price)
            {
                actions.push(CommandIntent::Buy { kind });
            }
        }
    }
    let on_shop = crate::systems::shop::is_on_shop(world);
    let on_altar = crate::systems::altar::is_on_altar(world);
    if let Some(campaign) = world.campaign {
        for &spell in campaign.known_spells() {
            if world.mp >= spell.mana_cost() {
                if spell.is_directional() {
                    for direction in Direction::ALL {
                        actions.push(CommandIntent::Cast {
                            spell,
                            direction: Some(direction),
                        });
                    }
                } else {
                    actions.push(CommandIntent::Cast {
                        spell,
                        direction: None,
                    });
                }
            }
        }
    }
    for entry in &world.inventory.entries {
        if let Some(data) = world.entities.item_data(entry.item) {
            if data.class == ItemClass::Weapon {
                actions.push(CommandIntent::Wield { item: entry.item });
            }
            if data.class == ItemClass::Armor {
                actions.push(CommandIntent::Wear { item: entry.item });
            }
            if matches!(data.class, ItemClass::Ring | ItemClass::Amulet) {
                actions.push(CommandIntent::PutOn { item: entry.item });
            }
            if data.class == ItemClass::Potion {
                actions.push(CommandIntent::Quaff { item: entry.item });
            }
            if matches!(data.class, ItemClass::Food | ItemClass::Corpse)
                && data.nutrition.is_some_and(|nutrition| nutrition > 0)
            {
                actions.push(CommandIntent::Eat { item: entry.item });
            }
            if data.class == ItemClass::Scroll {
                actions.push(CommandIntent::Read { item: entry.item });
            }
            if world.campaign.is_some() && data.class == ItemClass::Tool {
                match data.tool_action {
                    Some(
                        ToolAction::Light
                        | ToolAction::Blindfold
                        | ToolAction::Music
                        | ToolAction::Magic,
                    ) => {
                        actions.push(CommandIntent::Apply {
                            item: entry.item,
                            direction: None,
                        });
                    }
                    Some(ToolAction::Unlock) => {
                        let has_chest = world
                            .entities
                            .items_at(world.current_level(), world.player_pos())
                            .into_iter()
                            .chain(world.inventory.entries.iter().map(|e| e.item))
                            .any(|id| {
                                world
                                    .entities
                                    .item_data(id)
                                    .is_some_and(|d| d.kind == ItemKind::Chest)
                            });
                        if has_chest {
                            actions.push(CommandIntent::Apply {
                                item: entry.item,
                                direction: None,
                            });
                        }
                        for direction in Direction::ALL {
                            let target = world.player_pos().offset(direction.delta());
                            if let Ok(tile) = world.current_map().tile(target) {
                                if tile == TileKind::Door(DoorState::Locked)
                                    || (data.kind == ItemKind::SkeletonKey
                                        && tile == TileKind::Door(DoorState::Closed))
                                {
                                    actions.push(CommandIntent::Apply {
                                        item: entry.item,
                                        direction: Some(direction),
                                    });
                                }
                            }
                        }
                    }
                    Some(ToolAction::Dig) => {
                        let map = world.current_map();
                        for direction in Direction::ALL {
                            let target = world.player_pos().offset(direction.delta());
                            if target.x > 0
                                && target.x < map.width - 1
                                && target.y > 0
                                && target.y < map.height - 1
                                && map.tile(target).ok() == Some(TileKind::Wall)
                            {
                                actions.push(CommandIntent::Apply {
                                    item: entry.item,
                                    direction: Some(direction),
                                });
                            }
                        }
                    }
                    Some(ToolAction::Container) | None => {}
                }
            }
            if on_shop && data.kind != aihack_core::domain::item::ItemKind::AmuletAscension {
                actions.push(CommandIntent::Sell { item: entry.item });
            }
            if on_altar && data.kind != aihack_core::domain::item::ItemKind::AmuletAscension {
                actions.push(CommandIntent::Offer { item: entry.item });
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
    if world.campaign.is_some()
        && !matches!(
            world.current_map().tile(world.player_pos()),
            Ok(TileKind::Water | TileKind::Lava)
        )
    {
        actions.push(CommandIntent::Engrave {
            method: EngravingMethod::Fingers,
            text: EngravingText::Elbereth,
        });
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
            Some(DoorState::Locked) | None => {}
        }
        if matches!(
            world
                .current_map()
                .tile(world.player_pos().offset(direction.delta())),
            Ok(TileKind::Door(DoorState::Closed | DoorState::Locked) | TileKind::HiddenDoor)
        ) {
            actions.push(CommandIntent::Kick(direction));
        }
    }
    for entry in &world.inventory.entries {
        actions.push(CommandIntent::Drop { item: entry.item });
    }
    if world.campaign.is_some() {
        if !matches!(
            world.current_map().tile(world.player_pos()),
            Ok(TileKind::Water | TileKind::Lava)
        ) {
            for text in [
                EngravingText::Warning,
                EngravingText::Safe,
                EngravingText::XMark,
            ] {
                actions.push(CommandIntent::Engrave {
                    method: EngravingMethod::Fingers,
                    text,
                });
            }
            for entry in &world.inventory.entries {
                let Some(entity) = world.entities.get(entry.item) else {
                    continue;
                };
                let Some((_, data, _, _, charges)) = entity.item() else {
                    continue;
                };
                let can_engrave = data.class == ItemClass::Weapon
                    || data.tool_action == Some(ToolAction::Dig)
                    || (data.wand_effect == Some(WandEffect::Fire) && charges.unwrap_or(0) > 0);
                if can_engrave {
                    for text in [
                        EngravingText::Elbereth,
                        EngravingText::Warning,
                        EngravingText::Safe,
                        EngravingText::XMark,
                    ] {
                        actions.push(CommandIntent::Engrave {
                            method: EngravingMethod::Item(entry.item),
                            text,
                        });
                    }
                }
            }
        }
        let mut containers = Vec::new();
        for entry in &world.inventory.entries {
            if world
                .entities
                .item_data(entry.item)
                .is_some_and(|d| d.container_capacity.is_some())
            {
                containers.push(entry.item);
            }
        }
        for floor_id in world
            .entities
            .items_at(world.current_level(), world.player_pos())
        {
            if world
                .entities
                .item_data(floor_id)
                .is_some_and(|d| d.container_capacity.is_some())
                && !containers.contains(&floor_id)
            {
                containers.push(floor_id);
            }
        }
        for container_id in containers {
            let Some(container_entity) = world.entities.get(container_id) else {
                continue;
            };
            if container_entity.item_container_locked() == Some(true) {
                continue;
            }
            let cap = container_entity
                .item()
                .and_then(|(_, d, _, _, _)| d.container_capacity)
                .unwrap_or(0);
            for &child in container_entity.item_contained_items().unwrap_or_default() {
                actions.push(CommandIntent::ContainerTake {
                    container: container_id,
                    item: child,
                });
            }
            let used =
                crate::systems::containers::raw_contained_weight(&world.entities, container_id, 0);
            for entry in &world.inventory.entries {
                if entry.item == container_id || world.inventory.slot_of(entry.item).is_some() {
                    continue;
                }
                let Some(item_entity) = world.entities.get(entry.item) else {
                    continue;
                };
                let Some((kind, data, _, _, _)) = item_entity.item() else {
                    continue;
                };
                if kind == ItemKind::AmuletAscension {
                    continue;
                }
                let qty = i32::from(item_entity.item_quantity().unwrap_or(1));
                let item_weight = i32::from(data.weight).saturating_mul(qty);
                if i32::from(used).saturating_add(item_weight) <= i32::from(cap) {
                    actions.push(CommandIntent::ContainerPut {
                        container: container_id,
                        item: entry.item,
                    });
                }
            }
        }
        let tile = world.current_map().tile(world.player_pos()).ok();
        if matches!(
            tile,
            Some(TileKind::Fountain | TileKind::Sink | TileKind::Water)
        ) {
            for entry in &world.inventory.entries {
                actions.push(CommandIntent::Dip {
                    item: entry.item,
                    target: DipTarget::Tile,
                });
            }
        } else if tile == Some(TileKind::Altar) && world.mp >= 3 {
            for entry in &world.inventory.entries {
                if let Some(e) = world.entities.get(entry.item) {
                    if e.item()
                        .is_some_and(|(k, _, _, _, _)| k == ItemKind::PotionWater)
                        && e.item_buc() == Some(BucStatus::Uncursed)
                    {
                        actions.push(CommandIntent::Dip {
                            item: entry.item,
                            target: DipTarget::Tile,
                        });
                    }
                }
            }
        }
        for target_entry in &world.inventory.entries {
            let Some(target_entity) = world.entities.get(target_entry.item) else {
                continue;
            };
            let Some((target_kind, target_data, _, _, _)) = target_entity.item() else {
                continue;
            };
            if target_data.class != ItemClass::Potion {
                continue;
            }
            for item_entry in &world.inventory.entries {
                if item_entry.item == target_entry.item {
                    continue;
                }
                let Some(item_entity) = world.entities.get(item_entry.item) else {
                    continue;
                };
                let Some((item_kind, item_data, _, _, _)) = item_entity.item() else {
                    continue;
                };
                let eligible = if target_kind == ItemKind::PotionWater {
                    !(item_kind == ItemKind::PotionWater
                        && target_entity.item_buc() == Some(BucStatus::Uncursed))
                } else {
                    item_data.class == ItemClass::Potion && item_kind != ItemKind::PotionWater
                };
                if eligible {
                    actions.push(CommandIntent::Dip {
                        item: item_entry.item,
                        target: DipTarget::Item(target_entry.item),
                    });
                }
            }
        }
    }
    for scroll_entry in &world.inventory.entries {
        let Some(data) = world.entities.item_data(scroll_entry.item) else {
            continue;
        };
        if data.kind != aihack_core::domain::item::ItemKind::ScrollIdentify
            || (world.campaign.is_some() && !world.is_item_identified(data.kind))
        {
            continue;
        }
        for entry in &world.inventory.entries {
            if entry.item == scroll_entry.item {
                continue;
            }
            if let Some(entity) = world.entities.get(entry.item) {
                let Some((kind, _, _, _, _)) = entity.item() else {
                    continue;
                };
                let needs_identify = world.campaign.is_none()
                    || !world.is_item_identified(kind)
                    || !entity.item_buc_known().unwrap_or(false)
                    || !entity.item_enchant_known().unwrap_or(false);
                if needs_identify {
                    actions.push(CommandIntent::Identify {
                        scroll: scroll_entry.item,
                        target: entry.item,
                    });
                }
            }
        }
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

fn item_is_edible(world: &GameWorld, item: aihack_core::ids::EntityId) -> bool {
    world
        .entities
        .item_data(item)
        .map(|data| {
            matches!(data.class, ItemClass::Food | ItemClass::Corpse)
                && data.nutrition.is_some_and(|nutrition| nutrition > 0)
        })
        .unwrap_or(false)
}
