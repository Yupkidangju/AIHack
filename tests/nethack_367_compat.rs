use std::{env, fs};

use aihack::{
    core::{
        save::ArtifactStore, CommandIntent, Direction, EntityId, GameEvent, GameSession, Pos,
        RunState,
    },
    domain::{
        combat::DeathCause,
        entity::EntityLocation,
        inventory::InventoryLetter,
        level::{
            PHASE5_LEVEL1_ID, PHASE5_LEVEL1_STAIRS_DOWN, PHASE5_LEVEL2_ID,
            PHASE5_LEVEL2_STAIRS_UP_POS,
        },
        status::{HungerState, Status},
        tile::{DoorState, TileKind, TrapKind},
    },
    systems::vision::has_line_of_sight,
};

const ARCHIVE_SHA256: &str = "98cf67df6debf9668a61745aa84c09bcab362e5d33f5b944ec5155d44d2aacb2";
const C001: &str = include_str!("../docs/compatibility/NH367-C001-wall-movement.md");
const C002: &str = include_str!("../docs/compatibility/NH367-C002-closed-door.md");
const C003: &str = include_str!("../docs/compatibility/NH367-C003-bump-attack.md");
const C004: &str = include_str!("../docs/compatibility/NH367-C004-item-actions.md");
const C005: &str = include_str!("../docs/compatibility/NH367-C005-stairs.md");
const C006: &str = include_str!("../docs/compatibility/NH367-C006-search.md");
const C007: &str = include_str!("../docs/compatibility/NH367-C007-projectiles.md");
const C008: &str = include_str!("../docs/compatibility/NH367-C008-hunger-status.md");
const C009: &str = include_str!("../docs/compatibility/NH367-C009-save-continuation.md");
const C010: &str = include_str!("../docs/compatibility/NH367-C010-game-over.md");

fn assert_record(record: &str, id: &str, function: &str) {
    assert!(record.contains(&format!("id: {id}")));
    assert!(record.contains("status: Implemented"));
    assert!(record.contains(ARCHIVE_SHA256));
    assert!(record.contains("locator:"));
    assert!(record.contains("provenance_status:"));
    assert!(record.contains("commands:"));
    assert!(record.contains("events:"));
    assert!(record.contains("hash_fields:"));
    assert!(record.contains(&format!("function: {function}")));
}

fn clear_monsters(session: &mut GameSession) {
    aihack::testing::SessionBuilder::mutate(session, |world| {
        world.saved().entities.clear_monsters()
    });
}

#[test]
fn nh367_c001_wall_movement_preserves_position_turn_and_hash() {
    assert_record(
        C001,
        "NH367-C001",
        "nh367_c001_wall_movement_preserves_position_turn_and_hash",
    );
    let mut session = GameSession::new_for_playing(42);
    clear_monsters(&mut session);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 1, y: 1 })
    });
    let before = session.snapshot().stable_hash();

    let outcome = session.submit(CommandIntent::Move(Direction::North));

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert_eq!(session.world().player_pos(), Pos { x: 1, y: 1 });
    assert_eq!(outcome.snapshot_hash, before);
}

#[test]
fn nh367_c002_closed_door_blocks_then_open_transitions_state() {
    assert_record(
        C002,
        "NH367-C002",
        "nh367_c002_closed_door_blocks_then_open_transitions_state",
    );
    let mut session = GameSession::new_for_playing(42);
    clear_monsters(&mut session);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 9, y: 5 })
    });
    let behind_door = Pos { x: 11, y: 5 };

    assert!(!has_line_of_sight(
        session.world(),
        session.world().player_pos(),
        behind_door
    ));
    assert!(
        !session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    let opened = session.submit(CommandIntent::Open(Direction::East));

    assert!(opened.accepted);
    assert!(opened.events.iter().any(|event| matches!(
        event,
        GameEvent::DoorChanged {
            from: DoorState::Closed,
            to: DoorState::Open,
            ..
        }
    )));
    assert_eq!(
        session.world().current_map().tile(Pos { x: 10, y: 5 }),
        Ok(TileKind::Door(DoorState::Open))
    );
    assert!(has_line_of_sight(
        session.world(),
        session.world().player_pos(),
        behind_door
    ));
}

#[test]
fn nh367_c003_bump_attack_emits_combat_without_player_movement() {
    assert_record(
        C003,
        "NH367-C003",
        "nh367_c003_bump_attack_emits_combat_without_player_movement",
    );
    let mut session = GameSession::new_for_playing(42);
    let before = session.world().player_pos();
    let defender_hp_before = session
        .world()
        .entities()
        .actor_stats(EntityId(2))
        .unwrap()
        .hp;
    let rng_before = session.to_save_data().rng_state;

    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert_eq!(session.turn(), 1);
    assert_eq!(session.world().player_pos(), before);
    let (hit, damage) = outcome
        .events
        .iter()
        .find_map(|event| match event {
            GameEvent::AttackResolved {
                attacker: EntityId(1),
                defender: EntityId(2),
                hit,
                damage,
                ..
            } => Some((*hit, *damage)),
            _ => None,
        })
        .expect("C003 AttackResolved event가 필요하다");
    assert_eq!(hit, damage > 0);
    let defender = session.world().entities().actor_stats(EntityId(2)).unwrap();
    assert_eq!(defender.hp, defender_hp_before - damage);
    assert_eq!(
        outcome.events.iter().any(|event| matches!(
            event,
            GameEvent::EntityDied {
                entity: EntityId(2),
                ..
            }
        )),
        !session
            .world()
            .entities()
            .get(EntityId(2))
            .unwrap()
            .is_alive_actor()
    );
    assert!(session.to_save_data().rng_state.draws > rng_before.draws);
}

#[test]
fn nh367_c004_pickup_wield_and_quaff_update_owned_item_state() {
    assert_record(
        C004,
        "NH367-C004",
        "nh367_c004_pickup_wield_and_quaff_update_owned_item_state",
    );
    let mut session = GameSession::new_for_playing(42);
    clear_monsters(&mut session);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 8, y: 5 })
    });

    assert!(session.submit(CommandIntent::Pickup).accepted);
    assert_eq!(
        session.world().inventory().letter_for(EntityId(4)),
        Some(InventoryLetter('f'))
    );
    let wielded = session.submit(CommandIntent::Wield { item: EntityId(5) });
    assert!(wielded.accepted);
    assert!(wielded.events.iter().any(|event| matches!(
        event,
        GameEvent::ItemEquipped {
            item: EntityId(5),
            ..
        }
    )));
    assert_eq!(
        session.world().inventory().equipped_melee,
        Some(EntityId(5))
    );
    let player = session.world().player_id();
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.actor_stats_mut(player).unwrap().hp = 5;
    });
    let quaffed = session.submit(CommandIntent::Quaff { item: EntityId(4) });
    assert!(quaffed.accepted);
    assert!(quaffed
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::EntityHealed { hp_after, .. } if *hp_after > 5)));
    assert_eq!(
        session.world().entities().item_location(EntityId(4)),
        Some(EntityLocation::Consumed)
    );
}

#[test]
fn nh367_c005_stairs_roundtrip_preserves_level_landing_contract() {
    assert_record(
        C005,
        "NH367-C005",
        "nh367_c005_stairs_roundtrip_preserves_level_landing_contract",
    );
    let mut session = GameSession::new_for_playing(42);
    clear_monsters(&mut session);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(PHASE5_LEVEL1_STAIRS_DOWN);
        world
            .current_map_mut()
            .set_tile(Pos { x: 2, y: 2 }, TileKind::Floor)
            .unwrap();
    });

    assert!(session.submit(CommandIntent::Descend).accepted);
    assert_eq!(session.world().current_level(), PHASE5_LEVEL2_ID);
    assert_eq!(session.world().player_pos(), PHASE5_LEVEL2_STAIRS_UP_POS);
    assert!(session.submit(CommandIntent::Ascend).accepted);
    assert_eq!(session.world().current_level(), PHASE5_LEVEL1_ID);
    assert_eq!(session.world().player_pos(), PHASE5_LEVEL1_STAIRS_DOWN);
    assert_eq!(
        session.world().current_map().tile(Pos { x: 2, y: 2 }),
        Ok(TileKind::Floor)
    );
}

#[test]
fn nh367_c006_search_reveals_hidden_door_and_trap() {
    assert_record(
        C006,
        "NH367-C006",
        "nh367_c006_search_reveals_hidden_door_and_trap",
    );
    for (player, target) in [
        (Pos { x: 11, y: 5 }, Pos { x: 12, y: 5 }),
        (Pos { x: 15, y: 5 }, Pos { x: 16, y: 5 }),
    ] {
        let mut session = GameSession::new_for_playing(42);
        clear_monsters(&mut session);
        aihack::testing::SessionBuilder::mutate(&mut session, |world| world.set_player_pos(player));
        let outcome = session.submit(CommandIntent::Search);
        assert!(outcome.events.iter().any(|event| matches!(
            event,
            GameEvent::TileRevealed { pos, .. } if *pos == target
        )));
    }
}

#[test]
fn nh367_c007_throw_zap_and_read_consume_bounded_resources() {
    assert_record(
        C007,
        "NH367-C007",
        "nh367_c007_throw_zap_and_read_consume_bounded_resources",
    );
    let mut thrown = GameSession::new_for_playing(42);
    clear_monsters(&mut thrown);
    let thrown_rng_before = thrown.to_save_data().rng_state;
    let thrown_map_before = thrown.world().current_map().tiles().to_vec();
    let thrown_outcome = thrown.submit(CommandIntent::Throw {
        item: EntityId(9),
        direction: Direction::East,
    });
    assert!(thrown_outcome.accepted && thrown_outcome.turn_advanced);
    assert!(thrown_outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::ItemThrown {
            to: Pos { x: 9, y: 5 },
            ..
        }
    )));
    assert_eq!(thrown.turn(), 1);
    assert_eq!(
        thrown.world().entities().item_location(EntityId(9)),
        Some(EntityLocation::OnMap {
            level: PHASE5_LEVEL1_ID,
            pos: Pos { x: 9, y: 5 },
        })
    );
    assert_eq!(thrown.world().current_map().tiles(), thrown_map_before);
    assert_eq!(thrown.to_save_data().rng_state, thrown_rng_before);

    let mut zapped = GameSession::new_for_playing(42);
    clear_monsters(&mut zapped);
    let zapped_rng_before = zapped.to_save_data().rng_state;
    let zapped_map_before = zapped.world().current_map().tiles().to_vec();
    let zapped_outcome = zapped.submit(CommandIntent::Zap {
        item: EntityId(7),
        direction: Direction::East,
    });
    assert!(zapped_outcome.accepted && zapped_outcome.turn_advanced);
    assert!(zapped_outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::WandZapped {
            charges_after: 2,
            ..
        }
    )));
    assert_eq!(zapped.turn(), 1);
    assert_eq!(zapped.world().entities().item_charges(EntityId(7)), Some(2));
    assert_eq!(zapped.world().current_map().tiles(), zapped_map_before);
    assert_eq!(zapped.to_save_data().rng_state, zapped_rng_before);

    let mut read = GameSession::new_for_playing(42);
    clear_monsters(&mut read);
    let read_rng_before = read.to_save_data().rng_state;
    let read_map_before = read.world().current_map().tiles().to_vec();
    assert_eq!(
        read.world().current_map().tile(Pos { x: 12, y: 5 }),
        Ok(TileKind::HiddenDoor)
    );
    assert_eq!(
        read.world().current_map().tile(Pos { x: 16, y: 5 }),
        Ok(TileKind::HiddenTrap(TrapKind::Pit))
    );
    let read_outcome = read.submit(CommandIntent::Read { item: EntityId(8) });
    assert!(read_outcome.accepted && read_outcome.turn_advanced);
    assert!(read_outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::ScrollRead {
            item: EntityId(8),
            ..
        }
    )));
    assert_eq!(read.turn(), 1);
    assert_eq!(
        read.world().entities().item_location(EntityId(8)),
        Some(EntityLocation::Consumed)
    );
    assert_ne!(read.world().current_map().tiles(), read_map_before);
    assert_eq!(
        read.world().current_map().tile(Pos { x: 12, y: 5 }),
        Ok(TileKind::Door(DoorState::Closed))
    );
    assert_eq!(
        read.world().current_map().tile(Pos { x: 16, y: 5 }),
        Ok(TileKind::Trap(TrapKind::Pit))
    );
    assert_eq!(read.to_save_data().rng_state, read_rng_before);
}

#[test]
fn nh367_c008_hunger_thresholds_map_to_stable_status_states() {
    assert_record(
        C008,
        "NH367-C008",
        "nh367_c008_hunger_thresholds_map_to_stable_status_states",
    );
    for (nutrition, expected) in [
        (-1, HungerState::Fainting),
        (0, HungerState::Fainting),
        (1, HungerState::Weak),
        (50, HungerState::Weak),
        (51, HungerState::Hungry),
        (150, HungerState::Hungry),
        (151, HungerState::NotHungry),
        (1000, HungerState::NotHungry),
        (1001, HungerState::Satiated),
    ] {
        let status = Status {
            nutrition,
            ..Status::default_adventurer()
        };
        assert_eq!(status.hunger_state(), expected);
    }
}

#[test]
fn nh367_c009_save_load_preserves_rng_command_continuation() {
    assert_record(
        C009,
        "NH367-C009",
        "nh367_c009_save_load_preserves_rng_command_continuation",
    );
    let root = env::temp_dir().join(format!("aihack-nh367-c009-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let store = ArtifactStore::open(&root).unwrap();
    let path = std::path::Path::new("save.json");
    let mut direct = GameSession::new_for_playing(42);
    assert!(direct.submit(CommandIntent::Wait).accepted);
    store.save_session(&direct, path).unwrap();
    let mut loaded = store.load_session(path).unwrap();

    for command in [CommandIntent::Search, CommandIntent::Move(Direction::East)] {
        let expected = direct.submit(command);
        let actual = loaded.submit(command);
        assert_eq!(actual.events, expected.events);
        assert_eq!(actual.snapshot_hash, expected.snapshot_hash);
    }
    drop(store);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn nh367_c010_player_death_records_cause_and_game_over_state() {
    assert_record(
        C010,
        "NH367-C010",
        "nh367_c010_player_death_records_cause_and_game_over_state",
    );
    let session = GameSession::new_for_playing(42);
    let player = session.world().player_id();
    let attacker = EntityId(3);
    let mut saved = session.to_save_data().world;
    saved.entities.actor_stats_mut(player).unwrap().hp = 0;
    let mut world = aihack::core::GameWorld::from_depleted_saved_world(saved).unwrap();

    let events =
        aihack::systems::death::collect_death_events_after_attack(&mut world, attacker, player);
    let state = aihack::systems::death::state_after_deaths(&world);

    assert!(events.iter().any(|event| matches!(
        event,
        GameEvent::EntityDied {
            entity,
            cause: DeathCause::Combat { attacker: source }
        } if *entity == player && *source == attacker
    )));
    assert!(matches!(
        state,
        RunState::GameOver {
            cause: DeathCause::Combat { attacker: source },
            final_score: 0,
        } if source == attacker
    ));
}
