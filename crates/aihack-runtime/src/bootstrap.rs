use aihack_content::ContentRegistry;
use aihack_core::{
    domain::{
        inventory::Inventory,
        level::{LevelRegistry, PHASE5_LEVEL1_ID},
    },
    error::ContentError,
    ids::EntityId,
    position::Pos,
    world::WorldState,
};

use crate::domain::{
    entity::{EntityLocation, EntityStore},
    item::{try_item_data_from_registry, ItemKind},
    monster::try_monster_template_from_registry,
};

fn allocation_content_error(error: aihack_core::error::EntityAllocationError) -> ContentError {
    ContentError::Parse {
        file: "world bootstrap".to_owned(),
        message: error.to_string(),
    }
}

pub(crate) fn campaign_world(
    registry: &ContentRegistry,
    seed: u64,
    role: aihack_core::campaign::Role,
) -> Result<WorldState<EntityStore>, ContentError> {
    use aihack_core::{
        campaign::{CampaignState, Role},
        domain::monster::MonsterKind,
        ids::LevelId,
    };
    let mut state = initial_world(registry)?;
    state.entities = EntityStore::new();
    state.levels.levels.clear();
    let (_, start) = crate::campaign_map::generate(seed, LevelId::main(1))?;
    state.player_id = state
        .entities
        .spawn_player(start[0])
        .map_err(allocation_content_error)?;
    state.inventory = Inventory::new(state.player_id);
    let mut amulet = None;
    for id in crate::campaign_map::level_ids() {
        let (level, centers) = crate::campaign_map::generate(seed, id)?;
        state.levels.levels.push(level);
        let kind = if id.depth < 3 {
            MonsterKind::Jackal
        } else {
            MonsterKind::Goblin
        };
        let monster = state
            .entities
            .spawn_monster_with_template(
                kind,
                try_monster_template_from_registry(kind, registry)?,
                centers[2],
            )
            .map_err(allocation_content_error)?;
        state.entities.set_actor_location(monster, id, centers[2]);
        for (kind, pos) in [
            (ItemKind::FoodRation, centers[1]),
            (
                ItemKind::PotionHealing,
                Pos {
                    x: centers[1].x + 1,
                    y: centers[1].y,
                },
            ),
        ] {
            spawn_item(
                &mut state.entities,
                registry,
                kind,
                EntityLocation::OnMap { level: id, pos },
            )?;
        }
        if id == LevelId::main(6) {
            amulet = Some(spawn_item(
                &mut state.entities,
                registry,
                ItemKind::AmuletAscension,
                EntityLocation::OnMap {
                    level: id,
                    pos: centers[3],
                },
            )?);
        }
    }
    let extras = match role {
        Role::Knight => vec![ItemKind::ArmorLeather, ItemKind::PotionHealing],
        Role::Scout => vec![ItemKind::FoodRation, ItemKind::FoodRation],
        Role::Mage => vec![
            ItemKind::WandMagicMissile,
            ItemKind::PotionHealing,
            ItemKind::PotionHealing,
        ],
    };
    for kind in [
        ItemKind::Dagger,
        ItemKind::FoodRation,
        ItemKind::WandMagicMissile,
        ItemKind::ScrollReveal,
        ItemKind::Rock,
    ]
    .into_iter()
    .chain(extras)
    {
        let item = spawn_item(
            &mut state.entities,
            registry,
            kind,
            EntityLocation::Inventory {
                owner: state.player_id,
            },
        )?;
        let letter = state
            .inventory
            .add_existing_with_next_letter(item)
            .expect("campaign starts with fewer than 52 items");
        state.entities.set_item_letter(item, letter);
    }
    let campaign = CampaignState {
        role,
        xp: 0,
        amulet: amulet.expect("Main 6 was generated"),
    };
    let stats = state
        .entities
        .actor_stats_mut(state.player_id)
        .expect("player was spawned");
    stats.hp = campaign.max_hp();
    stats.max_hp = campaign.max_hp();
    stats.hit_bonus = campaign.hit_bonus();
    stats.damage_bonus = campaign.damage_bonus();
    state.campaign = Some(campaign);
    Ok(state)
}

fn spawn_item(
    entities: &mut EntityStore,
    registry: &ContentRegistry,
    kind: ItemKind,
    location: EntityLocation,
) -> Result<EntityId, ContentError> {
    entities
        .spawn_item_with_data(kind, try_item_data_from_registry(kind, registry)?, location)
        .map_err(allocation_content_error)
}

/// Embedded content에서 초기 월드 상태를 만든다. session/UI는 이 경계에 관여하지 않는다.
pub fn initial_world(registry: &ContentRegistry) -> Result<WorldState<EntityStore>, ContentError> {
    let mut entities = EntityStore::new();
    let level = registry
        .level("main:1")
        .ok_or_else(|| ContentError::UnknownReference {
            owner: "world bootstrap".to_owned(),
            target: "main:1".to_owned(),
        })?;
    let player_id = entities
        .spawn_player(Pos {
            x: level.player_start[0],
            y: level.player_start[1],
        })
        .map_err(allocation_content_error)?;
    for spawn in aihack_content::level_spawns(level)? {
        match spawn {
            aihack_content::LevelSpawn::Monster { kind, pos } => {
                entities
                    .spawn_monster_with_template(
                        kind,
                        try_monster_template_from_registry(kind, registry)?,
                        pos,
                    )
                    .map_err(allocation_content_error)?;
            }
            aihack_content::LevelSpawn::Item { kind, pos } => {
                spawn_item(
                    &mut entities,
                    registry,
                    kind,
                    EntityLocation::OnMap {
                        level: PHASE5_LEVEL1_ID,
                        pos,
                    },
                )?;
            }
        }
    }
    let mut inventory = Inventory::new(player_id);
    for kind in [
        ItemKind::Dagger,
        ItemKind::FoodRation,
        ItemKind::WandMagicMissile,
        ItemKind::ScrollReveal,
        ItemKind::Rock,
    ] {
        let item = spawn_item(
            &mut entities,
            registry,
            kind,
            EntityLocation::Inventory { owner: player_id },
        )?;
        let letter = inventory
            .add_existing_with_next_letter(item)
            .ok_or_else(|| ContentError::UnknownReference {
                owner: "world bootstrap inventory".to_owned(),
                target: "item letter".to_owned(),
            })?;
        entities.set_item_letter(item, letter);
    }
    for (kind, pos) in [
        (ItemKind::ArmorLeather, Pos { x: 7, y: 5 }),
        (ItemKind::ScrollIdentify, Pos { x: 9, y: 5 }),
        (ItemKind::ScrollLevelTeleport, Pos { x: 11, y: 5 }),
    ] {
        spawn_item(
            &mut entities,
            registry,
            kind,
            EntityLocation::OnMap {
                level: PHASE5_LEVEL1_ID,
                pos,
            },
        )?;
    }
    Ok(WorldState {
        campaign: None,
        levels: LevelRegistry::from_layouts(registry.levels())?,
        current_level: PHASE5_LEVEL1_ID,
        entities,
        player_id,
        inventory,
        nutrition: 900,
        luck: 0,
        prayer_cooldown: 0,
        paralysis_turns: 0,
        hallucinating: false,
        kill_count: 0,
        gold: 0,
        identified_items: Vec::new(),
        last_death_cause: None,
    })
}
