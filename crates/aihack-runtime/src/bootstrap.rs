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

fn spawn_item(
    entities: &mut EntityStore,
    registry: &ContentRegistry,
    kind: ItemKind,
    location: EntityLocation,
) -> Result<EntityId, ContentError> {
    Ok(entities.spawn_item_with_data(kind, try_item_data_from_registry(kind, registry)?, location))
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
    let player_id = entities.spawn_player(Pos {
        x: level.player_start[0],
        y: level.player_start[1],
    });
    for spawn in aihack_content::level_spawns(level)? {
        match spawn {
            aihack_content::LevelSpawn::Monster { kind, pos } => {
                entities.spawn_monster_with_template(
                    kind,
                    try_monster_template_from_registry(kind, registry)?,
                    pos,
                );
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
