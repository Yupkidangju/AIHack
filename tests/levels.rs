use aihack::{
    core::{BranchId, CommandIntent, EntityId, GameSession, LevelId, Pos},
    domain::{
        entity::{EntityLocation, EntityPayload},
        level::{PHASE5_LEVEL1_ID, PHASE5_LEVEL1_STAIRS_DOWN, PHASE5_LEVEL2_ID},
        tile::{DoorState, TileKind},
    },
};

#[test]
fn level_registry_contains_two_fixed_levels() {
    let session = GameSession::new(42);

    assert_eq!(session.world.levels.len(), 2);
    assert!(session.world.levels.contains(LevelId {
        branch: BranchId::Main,
        depth: 1,
    }));
    assert!(session.world.levels.contains(LevelId {
        branch: BranchId::Main,
        depth: 2,
    }));
    assert_eq!(session.world.current_level(), PHASE5_LEVEL1_ID);
}

#[test]
fn fixed_level_stairs_match_spec() {
    let session = GameSession::new(42);

    assert_eq!(
        session
            .world
            .map(PHASE5_LEVEL1_ID)
            .tile(PHASE5_LEVEL1_STAIRS_DOWN),
        Ok(TileKind::StairsDown)
    );
    assert_eq!(
        session.world.map(PHASE5_LEVEL2_ID).tile(Pos { x: 5, y: 5 }),
        Ok(TileKind::StairsUp)
    );
    assert!(!session
        .world
        .map(PHASE5_LEVEL2_ID)
        .tiles()
        .contains(&TileKind::StairsDown));
    assert_eq!(
        session.world.entities.actor_location(EntityId(2)),
        Some((PHASE5_LEVEL1_ID, Pos { x: 6, y: 5 }))
    );
    assert_eq!(
        session.world.entities.actor_location(EntityId(3)),
        Some((PHASE5_LEVEL1_ID, Pos { x: 20, y: 12 }))
    );
}

#[test]
fn actor_and_item_locations_are_level_aware() {
    let session = GameSession::new(42);

    assert_eq!(
        session
            .world
            .entities
            .actor_location(session.world.player_id),
        Some((PHASE5_LEVEL1_ID, Pos { x: 5, y: 5 }))
    );
    assert!(matches!(
        &session
            .world
            .entities
            .get(session.world.player_id)
            .unwrap()
            .payload,
        EntityPayload::Actor {
            location: EntityLocation::OnMap {
                level: PHASE5_LEVEL1_ID,
                pos: Pos { x: 5, y: 5 }
            },
            ..
        }
    ));
    assert_eq!(
        session.world.entities.item_location(EntityId(4)),
        Some(EntityLocation::OnMap {
            level: PHASE5_LEVEL1_ID,
            pos: Pos { x: 8, y: 5 },
        })
    );
    assert_eq!(
        session
            .world
            .entities
            .item_at(PHASE5_LEVEL1_ID, Pos { x: 8, y: 5 }),
        Some(EntityId(4))
    );
    assert_eq!(
        session
            .world
            .entities
            .alive_hostile_at(PHASE5_LEVEL1_ID, Pos { x: 6, y: 5 }),
        Some(EntityId(2))
    );

    let entity_source = include_str!("../src/domain/entity.rs");
    assert!(entity_source.contains("location: EntityLocation"));
    assert!(!entity_source.contains(
        "Actor {\n        kind: ActorKind,\n        faction: Faction,\n        pos: Pos"
    ));
}

#[test]
fn level1_state_survives_round_trip() {
    let mut session = GameSession::new(42);

    session.world.set_player_pos(Pos { x: 9, y: 5 });
    assert!(
        session
            .submit(CommandIntent::Open(aihack::core::Direction::East))
            .accepted
    );
    session.world.set_player_pos(Pos { x: 8, y: 5 });
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let potion_location_before = session.world.entities.item_location(EntityId(4));
    let inventory_before = session.world.inventory.clone();

    session.world.set_player_pos(PHASE5_LEVEL1_STAIRS_DOWN);
    assert!(session.submit(CommandIntent::Descend).accepted);
    assert!(session.submit(CommandIntent::Ascend).accepted);

    assert_eq!(
        session
            .world
            .map(PHASE5_LEVEL1_ID)
            .tile(Pos { x: 10, y: 5 }),
        Ok(TileKind::Door(DoorState::Open))
    );
    assert_eq!(
        session.world.entities.item_location(EntityId(4)),
        potion_location_before
    );
    assert_eq!(session.world.inventory, inventory_before);
    assert_eq!(
        session.world.entities.actor_location(EntityId(2)),
        Some((PHASE5_LEVEL1_ID, Pos { x: 6, y: 5 }))
    );
    assert_eq!(session.world.current_level(), PHASE5_LEVEL1_ID);
    assert_eq!(session.world.player_pos(), PHASE5_LEVEL1_STAIRS_DOWN);
}

#[test]
fn current_level_affects_snapshot_hash() {
    let mut a = GameSession::new(42);
    let mut b = GameSession::new(42);
    b.world
        .set_player_location(PHASE5_LEVEL2_ID, Pos { x: 5, y: 5 });

    assert_ne!(a.snapshot().stable_hash(), b.snapshot().stable_hash());
    a.world
        .set_player_location(PHASE5_LEVEL2_ID, Pos { x: 5, y: 5 });
    assert_eq!(a.snapshot().stable_hash(), b.snapshot().stable_hash());
}

#[test]
fn level_map_state_affects_snapshot_hash() {
    let a = GameSession::new(42);
    let mut b = GameSession::new(42);
    b.world
        .map_mut(PHASE5_LEVEL2_ID)
        .set_tile(Pos { x: 8, y: 5 }, TileKind::Door(DoorState::Open))
        .unwrap();

    assert_eq!(a.snapshot().levels[0].id, PHASE5_LEVEL1_ID);
    assert_eq!(a.snapshot().levels[1].id, PHASE5_LEVEL2_ID);
    assert_ne!(a.snapshot().stable_hash(), b.snapshot().stable_hash());
}

#[test]
fn current_level_invariant_survives_session_commands() {
    let mut session = GameSession::new(42);

    assert_eq!(
        session.world.current_level(),
        session.world.player_location().0
    );
    assert!(session.submit(CommandIntent::Wait).accepted);
    assert_eq!(
        session.world.current_level(),
        session.world.player_location().0
    );
    assert!(
        session
            .submit(CommandIntent::Move(aihack::core::Direction::South))
            .accepted
    );
    assert_eq!(
        session.world.current_level(),
        session.world.player_location().0
    );

    session.world.set_player_pos(PHASE5_LEVEL1_STAIRS_DOWN);
    assert!(session.submit(CommandIntent::Descend).accepted);
    assert_eq!(
        session.world.current_level(),
        session.world.player_location().0
    );
    assert!(session.submit(CommandIntent::Ascend).accepted);
    assert_eq!(
        session.world.current_level(),
        session.world.player_location().0
    );
}
