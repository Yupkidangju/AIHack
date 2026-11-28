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
        movement::{
            is_bump_attack_for_legal_action, is_friendly_swap_for_legal_action,
            is_passable_for_legal_action,
        },
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

fn tile_altar_alignment(
    world: &GameWorld,
    pos: Pos,
    tile: TileKind,
) -> Option<aihack_core::campaign::Alignment> {
    world.campaign?;
    if tile != TileKind::Altar {
        return None;
    }
    world
        .levels
        .get(world.current_level())
        .and_then(|level| level.altar_alignment(pos))
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
                    altar_alignment: tile_altar_alignment(world, pos, tile),
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
                        altar_alignment: tile_altar_alignment(world, pos, tile),
                    });
                }
            }
        }
    }
    visible_tiles.sort_by_key(|tile| (tile.pos.y, tile.pos.x));

    let legal_actions = legal_actions(world, run_state, turn);
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
                god_anger: campaign.god_anger,
                prayer_timeout: campaign.prayer_timeout,
                artifact_invoke_cooldown: campaign.artifact_invoke_cooldown,
                skills: crate::systems::skills::skill_observations(world),
                learned_spells: world.learned_spells.clone(),
                spell_failure_rates: {
                    let mut spells: Vec<aihack_core::campaign::SpellKind> =
                        campaign.known_spells().to_vec();
                    for ls in &world.learned_spells {
                        if !spells.contains(&ls.spell) {
                            spells.push(ls.spell);
                        }
                    }
                    spells
                        .into_iter()
                        .map(|spell| {
                            (
                                spell,
                                crate::systems::spells::spell_failure_percent(world, spell),
                            )
                        })
                        .collect()
                },
                special_level_id: world
                    .levels
                    .get(world.current_level())
                    .and_then(|level| level.special_level_id.clone()),
                special_room: world
                    .levels
                    .get(world.current_level())
                    .and_then(|level| level.special_room),
                on_magic_portal: world.current_map().tile(world.player_pos()).ok()
                    == Some(TileKind::MagicPortal),
                on_high_altar: if world.current_level()
                    == aihack_core::ids::LevelId::new(aihack_core::ids::BranchId::Planes, 5)
                    && world
                        .ascension_altars
                        .contains(&(world.current_level(), world.player_pos()))
                {
                    crate::systems::altar::altar_alignment(world)
                } else {
                    None
                },
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
        fast_turns: world
            .status_timers
            .fast
            .max(u8::from(world.campaign.is_some() && world.is_fast())),
        levitation_turns: world
            .status_timers
            .levitation
            .max(u8::from(world.campaign.is_some() && world.is_levitating())),
        invisible_turns: world
            .status_timers
            .invisible
            .max(u8::from(world.campaign.is_some() && world.is_invisible())),
        blindfolded: world.campaign.is_some()
            && crate::systems::tools::is_player_blindfolded(world),
        shop_debt: world.shop_debt,
    }
}

fn visible_entities(seed: u64, world: &GameWorld) -> Vec<EntityObservation> {
    let visible = visible_positions(world);
    let has_telepathy = world.campaign.is_some()
        && world
            .effective_player_resistances()
            .contains(aihack_core::domain::status::Resistances::TELEPATHY);
    let has_infravision = world.campaign.is_some_and(|c| c.race.has_infravision())
        && world.status_timers.blind == 0
        && !crate::systems::tools::is_player_blindfolded(world);
    let registry = world.content_registry();
    let mut entities = world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| {
            if entity.id == world.player_id {
                return None;
            }
            if let Some((actor_kind, faction, level, pos, stats, alive)) = entity.actor() {
                let infravision_hit = has_infravision
                    && level == world.current_level()
                    && i32::from(world.player_pos().chebyshev_distance(pos))
                        <= aihack_core::campaign::INFRAVISION_RADIUS
                    && crate::systems::vision::has_line_of_sight(world, world.player_pos(), pos);
                if alive
                    && level == world.current_level()
                    && (visible.contains(&pos) || has_telepathy || infravision_hit)
                {
                    if world.campaign.is_some()
                        && entity.actor_invisible() == Some(true)
                        && !world.has_see_invisible()
                        && !has_telepathy
                    {
                        return None;
                    }
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
                        faction: world.campaign.is_some().then_some(faction),
                        unpaid: false,
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
                        let name = crate::appearance::display_name_for_world_item(
                            seed, world, entity.id, kind,
                        );
                        let color = if let Some(ref app) = appearance {
                            Some(crate::appearance::color_for_appearance(app).to_owned())
                        } else {
                            aihack_content::item_color(kind, registry).map(str::to_owned)
                        };
                        (
                            Some(aihack_content::item_glyph(kind, registry)),
                            Some(name),
                            color,
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
                        faction: None,
                        unpaid: world.campaign.is_some() && entity.item_unpaid().unwrap_or(false),
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
                let item_name =
                    crate::appearance::display_name_for_world_item(seed, world, entry.item, kind);
                let color = if let Some(ref app) = appearance {
                    Some(crate::appearance::color_for_appearance(app).to_owned())
                } else {
                    aihack_content::item_color(kind, registry).map(str::to_owned)
                };
                (
                    Some(aihack_content::item_glyph(kind, registry)),
                    Some(item_name),
                    color,
                    Some(data.class),
                )
            } else {
                (None, None, None, None)
            };
            let (contained_count, tool_active, container_locked, erosion, artifact_id) =
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
                        entity.item_artifact_id().map(str::to_owned),
                    )
                } else {
                    (0, false, false, 0, None)
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
                artifact_id,
                unpaid: world.campaign.is_some() && entity.item_unpaid().unwrap_or(false),
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
            let artifact_id = child.item_artifact_id().map(str::to_owned);
            let name = if let Some(def) = artifact_id
                .as_deref()
                .and_then(aihack_core::campaign::artifact_def)
            {
                def.name.to_owned()
            } else if let Some(app) = appearance {
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
                artifact_id,
            });
        }
    }
    out
}

fn legal_actions(world: &GameWorld, run_state: RunState, turn: u64) -> Vec<CommandIntent> {
    match run_state {
        RunState::CharacterCreation => {
            let mut actions = vec![CommandIntent::Wait, CommandIntent::Quit];
            actions.extend(
                aihack_core::campaign::Role::ALL
                    .into_iter()
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
                        if item_has_class(world, item, ItemClass::Scroll)
                            || (world.campaign.is_some()
                                && item_has_class(world, item, ItemClass::Spellbook))
                        {
                            actions.push(CommandIntent::Read { item });
                        }
                    }
                    InventoryAction::PutOn => {
                        if item_has_class(world, item, ItemClass::Ring)
                            || item_has_class(world, item, ItemClass::Amulet)
                        {
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
                        let on_astral_high_altar = world.current_level()
                            == aihack_core::ids::LevelId::new(
                                aihack_core::ids::BranchId::Planes,
                                5,
                            )
                            && world
                                .ascension_altars
                                .contains(&(world.current_level(), world.player_pos()));
                        if crate::systems::altar::is_on_altar(world)
                            && world.entities.item_data(item).is_some_and(|data| {
                                on_astral_high_altar
                                    || data.kind
                                        != aihack_core::domain::item::ItemKind::AmuletAscension
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
        RunState::Playing => playing_actions(world, turn),
    }
}

fn playing_actions(world: &GameWorld, turn: u64) -> Vec<CommandIntent> {
    if world.paralysis_turns > 0 {
        return vec![CommandIntent::Wait, CommandIntent::Quit];
    }
    let mut actions = Vec::new();
    // Tier 1: 핵심 공간 이동·기본 생존·상호작용 명령을 action_space(<= 64) 앞쪽에 배치한다.
    actions.push(CommandIntent::Wait);
    for direction in Direction::ALL {
        if is_passable_for_legal_action(world, direction)
            || is_bump_attack_for_legal_action(world, direction)
            || is_friendly_swap_for_legal_action(world, direction)
            || crate::systems::movement::is_boulder_push_for_legal_action(world, direction)
        {
            actions.push(CommandIntent::Move(direction));
        }
    }
    if crate::systems::npc::can_pay(world) {
        actions.push(CommandIntent::Pay);
    }
    for direction in Direction::ALL {
        if crate::systems::npc::can_chat_in_direction(world, direction) {
            actions.push(CommandIntent::Chat(direction));
        }
    }
    actions.push(CommandIntent::Search);
    if world
        .entities
        .item_at(world.current_level(), world.player_pos())
        .is_some()
    {
        actions.push(CommandIntent::Pickup);
    }
    let on_astral_high_altar = world.current_level()
        == aihack_core::ids::LevelId::new(aihack_core::ids::BranchId::Planes, 5)
        && world
            .ascension_altars
            .contains(&(world.current_level(), world.player_pos()));
    if on_astral_high_altar && crate::systems::altar::is_on_altar(world) {
        if let Some(campaign) = world.campaign {
            if crate::campaign::has_amulet(world) {
                actions.push(CommandIntent::Offer {
                    item: campaign.amulet,
                });
            }
        }
    }
    match world.current_map().tile(world.player_pos()) {
        Ok(tile @ TileKind::StairsDown) => {
            actions.push(CommandIntent::Descend);
            if world.campaign.is_some()
                && aihack_core::domain::level::BranchGraph::campaign_v3()
                    .branch_entry_from(world.current_level(), tile)
                    .is_some()
            {
                actions.push(CommandIntent::EnterBranch);
            }
        }
        Ok(tile @ TileKind::StairsUp) => {
            let graph = aihack_core::domain::level::BranchGraph::campaign_v3();
            let can_ascend = match world.campaign {
                None => true,
                Some(_) if world.current_level() == aihack_core::ids::LevelId::main(1) => {
                    crate::campaign::has_amulet(world)
                }
                Some(_) => graph.ascend_target(world.current_level()).is_some(),
            };
            if can_ascend {
                actions.push(CommandIntent::Ascend);
            }
            if world.campaign.is_some()
                && graph
                    .branch_entry_from(world.current_level(), tile)
                    .is_some()
            {
                actions.push(CommandIntent::EnterBranch);
            }
        }
        Ok(TileKind::MagicPortal) if world.campaign.is_some() => {
            if aihack_core::domain::level::BranchGraph::campaign_v3()
                .portal_target(world.current_level())
                .is_some()
            {
                actions.push(CommandIntent::EnterBranch);
                actions.push(CommandIntent::Ascend);
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
    if world.prayer_cooldown == 0 {
        actions.push(CommandIntent::Pray);
    }
    actions.push(CommandIntent::ShowInventory);
    for direction in Direction::ALL {
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
    if let Some(campaign) = world.campaign {
        for cat in aihack_core::campaign::SkillCategory::ALL {
            if crate::systems::skills::can_enhance(world, cat) {
                actions.push(CommandIntent::Enhance { skill: cat });
            }
        }
        if campaign.artifact_invoke_cooldown == 0 {
            for entry in &world.inventory.entries {
                if crate::systems::artifacts::artifact_def_for_item(world, entry.item).is_some() {
                    actions.push(CommandIntent::Invoke { item: entry.item });
                }
            }
        }
    }

    // Tier 2: 단일 인벤토리·상점·제단·비방향 주문/도구 명령
    let on_shop = crate::systems::shop::is_on_shop(world);
    let on_altar = crate::systems::altar::is_on_altar(world);
    let allow_non_amulet_offer = on_altar
        && !(on_astral_high_altar
            && world.campaign.is_some_and(|c| {
                crate::systems::altar::altar_alignment(world).is_some_and(|a| a != c.alignment)
            }));
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
            if data.class == ItemClass::Scroll
                || (world.campaign.is_some() && data.class == ItemClass::Spellbook)
            {
                actions.push(CommandIntent::Read { item: entry.item });
            }
            if world.campaign.is_some()
                && data.class == ItemClass::Tool
                && match data.tool_action {
                    Some(
                        ToolAction::Light
                        | ToolAction::Blindfold
                        | ToolAction::Music
                        | ToolAction::Magic,
                    ) => true,
                    Some(ToolAction::Unlock) => world
                        .entities
                        .items_at(world.current_level(), world.player_pos())
                        .into_iter()
                        .chain(world.inventory.entries.iter().map(|e| e.item))
                        .any(|id| {
                            world
                                .entities
                                .item_data(id)
                                .is_some_and(|d| d.kind == ItemKind::Chest)
                        }),
                    _ => false,
                }
            {
                actions.push(CommandIntent::Apply {
                    item: entry.item,
                    direction: None,
                });
            }
            if on_shop
                && data.kind != aihack_core::domain::item::ItemKind::AmuletAscension
                && world.entities.get(entry.item).and_then(|e| e.item_unpaid()) != Some(true)
            {
                actions.push(CommandIntent::Sell { item: entry.item });
            }
            if allow_non_amulet_offer
                && data.kind != aihack_core::domain::item::ItemKind::AmuletAscension
            {
                actions.push(CommandIntent::Offer { item: entry.item });
            }
        }
    }
    let mut active_spells: Vec<aihack_core::campaign::SpellKind> = Vec::new();
    if let Some(campaign) = world.campaign {
        for &spell in campaign.known_spells() {
            active_spells.push(spell);
        }
        for ls in &world.learned_spells {
            if turn.saturating_sub(ls.learned_at_turn) < aihack_core::campaign::SPELL_MEMORY_TURNS
                && !active_spells.contains(&ls.spell)
            {
                active_spells.push(ls.spell);
            }
        }
    }
    for &spell in &active_spells {
        if world.mp >= spell.mana_cost() && !spell.is_directional() {
            actions.push(CommandIntent::Cast {
                spell,
                direction: None,
            });
        }
    }
    if on_shop {
        for kind in crate::systems::shop::SHOP_CATALOG {
            if crate::systems::shop::can_buy_slot(world, kind)
                && crate::systems::shop::buy_price_for_world(world, kind)
                    .is_ok_and(|price| world.gold >= price)
            {
                actions.push(CommandIntent::Buy { kind });
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

    // Tier 3: 8방향 조합 폭발 명령·Drop·각인·컨테이너·Dip·감정
    for &spell in &active_spells {
        if world.mp >= spell.mana_cost() && spell.is_directional() {
            for direction in Direction::ALL {
                actions.push(CommandIntent::Cast {
                    spell,
                    direction: Some(direction),
                });
            }
        }
    }
    for entry in &world.inventory.entries {
        let Some(data) = world.entities.item_data(entry.item) else {
            continue;
        };
        if world.campaign.is_some() && data.class == ItemClass::Tool {
            match data.tool_action {
                Some(ToolAction::Unlock) => {
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
                    let cur_level = world.current_level();
                    let can_dig_level = cur_level.branch != aihack_core::ids::BranchId::Sokoban
                        && !(cur_level.branch == aihack_core::ids::BranchId::Planes
                            && cur_level.depth >= 2);
                    if can_dig_level {
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
                }
                _ => {}
            }
        }
        if matches!(data.class, ItemClass::Weapon | ItemClass::Rock)
            || (world.campaign.is_some()
                && matches!(data.class, ItemClass::Food | ItemClass::Corpse))
        {
            for direction in Direction::ALL {
                actions.push(CommandIntent::Throw {
                    item: entry.item,
                    direction,
                });
            }
        }
        if data.class == ItemClass::Wand && world.entities.item_charges(entry.item).unwrap_or(0) > 0
        {
            if data.wand_effect == Some(WandEffect::Wishing)
                && world.campaign.is_some()
                && world.is_item_identified(data.kind)
            {
                for &(_, wish) in crate::systems::projectiles::CURATED_WISHES {
                    actions.push(CommandIntent::Wish {
                        item: entry.item,
                        wish,
                    });
                }
            }
            for direction in Direction::ALL {
                actions.push(CommandIntent::Zap {
                    item: entry.item,
                    direction,
                });
            }
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
                    || (data.wand_effect == Some(WandEffect::Fire)
                        && world.is_item_identified(data.kind)
                        && charges.unwrap_or(0) > 0);
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
                if kind == ItemKind::AmuletAscension || item_entity.item_unpaid() == Some(true) {
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
        } else if tile == Some(TileKind::Altar) {
            let cross_aligned = world.campaign.is_some_and(|c| {
                crate::systems::altar::altar_alignment(world)
                    .is_some_and(|align| align != c.alignment)
            });
            if cross_aligned {
                for entry in &world.inventory.entries {
                    actions.push(CommandIntent::Dip {
                        item: entry.item,
                        target: DipTarget::Tile,
                    });
                }
            } else if world.mp >= 3 {
                for entry in &world.inventory.entries {
                    if let Some(e) = world.entities.get(entry.item) {
                        if let Some((k, d, _, _, _)) = e.item() {
                            if d.class != ItemClass::Potion {
                                continue;
                            }
                            let known_kind = world.is_item_identified(k);
                            if known_kind && k != ItemKind::PotionWater {
                                continue;
                            }
                            let known_buc = e.item_buc_known().unwrap_or(false);
                            if known_buc && e.item_buc() != Some(BucStatus::Uncursed) {
                                continue;
                            }
                            actions.push(CommandIntent::Dip {
                                item: entry.item,
                                target: DipTarget::Tile,
                            });
                        }
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
            let target_known_kind = world.is_item_identified(target_kind);
            let target_known_buc = target_entity.item_buc_known().unwrap_or(false);
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
                let item_known_kind = world.is_item_identified(item_kind);
                let eligible = if target_known_kind && target_kind != ItemKind::PotionWater {
                    item_data.class == ItemClass::Potion
                        && !(item_known_kind && item_kind == ItemKind::PotionWater)
                } else {
                    !(target_known_kind
                        && target_kind == ItemKind::PotionWater
                        && target_known_buc
                        && target_entity.item_buc() == Some(BucStatus::Uncursed)
                        && item_known_kind
                        && item_kind == ItemKind::PotionWater)
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
