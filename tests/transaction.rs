use aihack::{
    core::{error::GameError, CommandIntent, Direction, EntityId, GameSession, Pos},
    domain::monster::MonsterAiKind,
};

fn save_with_inventory_owner(owner: u32) -> aihack::core::SaveDataV1 {
    let save = GameSession::new_for_playing(42).to_save_data();
    let mut encoded = serde_json::to_value(save).unwrap();
    encoded["world"]["inventory"]["owner"] = serde_json::json!(owner);
    serde_json::from_value(encoded).unwrap()
}

#[test]
fn invalid_persisted_invariant_is_rejected_before_session_creation() {
    assert!(matches!(
        GameSession::from_save_data(save_with_inventory_owner(2)),
        Err(GameError::InvalidSave(_))
    ));
}

#[test]
fn rejected_command_preserves_the_following_deterministic_turn() {
    let mut baseline = GameSession::new_for_playing(42);
    let mut candidate = GameSession::new_for_playing(42);

    let rejected = candidate.submit(CommandIntent::Open(aihack::core::Direction::West));
    assert!(!rejected.accepted);

    let expected = baseline.submit(CommandIntent::Wait);
    let actual = candidate.submit(CommandIntent::Wait);

    assert_eq!(actual.snapshot_hash, expected.snapshot_hash);
}

#[test]
fn invalid_persisted_invariant_cannot_materialize_an_rng() {
    let result = GameSession::from_save_data(save_with_inventory_owner(2));
    assert!(matches!(result, Err(GameError::InvalidSave(_))));
}

#[test]
fn accepted_turns_leave_a_six_check_valid_invariant_report() {
    let mut session = GameSession::new_for_playing(42);

    let outcome = session.submit(CommandIntent::Wait);
    let report = session.world().validate_invariants();

    assert!(outcome.accepted);
    assert_eq!(report.checked, 6);
    assert!(report.is_valid());
}

fn exact_successor_session(next_id: u32) -> GameSession {
    let mut value = serde_json::to_value(GameSession::new_for_playing(42).to_save_data()).unwrap();
    let jackal_kind = value["world"]["entities"]["entities"][1]["payload"]["Actor"]["kind"].clone();
    value["world"]["entities"]["entities"][2]["payload"]["Actor"]["kind"] = jackal_kind;
    let entities = value["world"]["entities"]["entities"]
        .as_array_mut()
        .unwrap();
    let last = entities.last_mut().unwrap();
    assert!(last["payload"]["Item"]["location"]["OnMap"].is_object());
    last["id"] = serde_json::json!(next_id - 1);
    value["world"]["entities"]["next_id"] = serde_json::json!(next_id);
    GameSession::from_save_data(serde_json::from_value(value).unwrap()).unwrap()
}

fn prepare_stationary_kill(session: &mut GameSession, target: EntityId, pos: Pos) {
    aihack::testing::SessionBuilder::mutate(session, |world| {
        let level = world.saved().current_level;
        for id in [EntityId(2), EntityId(3), EntityId(4)] {
            world.saved().entities.set_alive(id, id == target);
        }
        assert!(world
            .saved()
            .entities
            .set_actor_location(target, level, pos));
        let target_stats = world.saved().entities.actor_stats_mut(target).unwrap();
        target_stats.hp = 1;
        target_stats.max_hp = 1;
        target_stats.ai_kind = Some(MonsterAiKind::Stationary);
        let player = world.saved().player_id;
        world
            .saved()
            .entities
            .actor_stats_mut(player)
            .unwrap()
            .hit_bonus = 100;
    });
}

#[test]
fn production_valid_allocator_last_commit_and_exhaustion_are_atomic() {
    let mut session = exact_successor_session(u32::MAX - 1);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        let level = world.saved().current_level;
        let player = world.saved().player_id;
        world
            .saved()
            .entities
            .actor_stats_mut(player)
            .unwrap()
            .hit_bonus = 100;
        for (id, pos) in [
            (EntityId(2), Pos { x: 6, y: 5 }),
            (EntityId(3), Pos { x: 7, y: 5 }),
        ] {
            world.saved().entities.set_alive(id, true);
            assert!(world.saved().entities.set_actor_location(id, level, pos));
            let stats = world.saved().entities.actor_stats_mut(id).unwrap();
            stats.hp = 1;
            stats.max_hp = 1;
            stats.ai_kind = Some(MonsterAiKind::Stationary);
        }
        world.saved().entities.set_alive(EntityId(4), false);
    });

    let first = session.submit(CommandIntent::Move(Direction::East));
    assert!(first.accepted, "last allocatable corpse must commit");
    assert_eq!(session.to_save_data().world.entities.next_id(), u32::MAX);
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    let before = session.to_save_data();
    let hash_before = session.snapshot().stable_hash();

    let exhausted = session.submit(CommandIntent::Move(Direction::East));

    assert!(!exhausted.accepted, "events={:?}", exhausted.events);
    assert_eq!(session.to_save_data(), before);
    assert_eq!(session.snapshot().stable_hash(), hash_before);
    assert_eq!(exhausted.snapshot_hash, hash_before);
}

#[test]
fn production_valid_throw_and_zap_exhaustion_restore_item_charge_rng_and_hash() {
    for (intent, target) in [
        (
            CommandIntent::Throw {
                item: EntityId(9),
                direction: Direction::East,
            },
            EntityId(2),
        ),
        (
            CommandIntent::Zap {
                item: EntityId(7),
                direction: Direction::East,
            },
            EntityId(2),
        ),
    ] {
        let mut session = exact_successor_session(u32::MAX);
        aihack::testing::SessionBuilder::mutate(&mut session, |world| {
            world.set_player_pos(Pos { x: 17, y: 12 });
        });
        prepare_stationary_kill(&mut session, target, Pos { x: 20, y: 12 });
        let before = session.to_save_data();
        let hash_before = session.snapshot().stable_hash();

        let exhausted = session.submit(intent);

        assert!(!exhausted.accepted, "events={:?}", exhausted.events);
        assert_eq!(session.to_save_data(), before);
        assert_eq!(session.snapshot().stable_hash(), hash_before);
        assert_eq!(exhausted.snapshot_hash, hash_before);
    }
}
