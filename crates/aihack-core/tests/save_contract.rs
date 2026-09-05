use aihack_core::{
    domain::{combat::DeathCause, entity::EntityStore, inventory::Inventory, level::LevelRegistry},
    ids::EntityId,
    save::SavedWorldV1,
    world::WorldState,
};

#[test]
fn saved_world_roundtrip_preserves_persisted_state_and_resets_runtime_death_cause() {
    let world = WorldState {
        campaign: None,
        levels: LevelRegistry::fixture_phase5(),
        current_level: aihack_core::ids::LevelId::main(1),
        entities: EntityStore::new(),
        player_id: EntityId(1),
        inventory: Inventory::new(EntityId(1)),
        nutrition: 900,
        luck: -1,
        prayer_cooldown: 3,
        paralysis_turns: 2,
        hallucinating: true,
        kill_count: 7,
        gold: 42,
        identified_items: Vec::new(),
        last_death_cause: Some(DeathCause::Combat {
            attacker: EntityId(9),
        }),
    };

    let restored = WorldState::from(SavedWorldV1::from(&world));

    assert_eq!(restored.levels, world.levels);
    assert_eq!(restored.current_level, world.current_level);
    assert_eq!(restored.nutrition, world.nutrition);
    assert_eq!(restored.luck, world.luck);
    assert_eq!(restored.gold, world.gold);
    assert_eq!(restored.last_death_cause, None);
}
