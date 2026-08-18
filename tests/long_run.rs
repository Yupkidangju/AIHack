use aihack::core::{
    causal::{CausalProjection, CausalSummary, CausalWitness, REQUIRED_CAUSAL_WITNESSES},
    policy::{run_to_turn, HeadlessPolicy},
    CommandIntent, Direction, EntityId, GameEvent, GameSession, RunState, SnapshotHash,
    TurnOutcome,
};
use aihack::domain::{
    entity::{EntityKind, EntityLocation},
    item::ItemKind,
    monster::{MonsterAiKind, MonsterKind},
};

const SEEDS: [u64; 3] = [42, 7, 1234];
const TARGET_TURN: u64 = 1000;
const EXPECTED_CAUSAL_HASHES: [&str; 3] =
    ["5cde4a5f145ff3af", "942403c665e19ad9", "01a8631d0ad95d96"];

fn semantic_state(session: &GameSession) -> serde_json::Value {
    let mut value = serde_json::to_value(session.snapshot()).unwrap();
    let object = value.as_object_mut().unwrap();
    for metadata in ["seed", "turn", "event_count", "last_event"] {
        object.remove(metadata);
    }
    value
}

#[test]
fn survival_policy_reaches_one_thousand_accepted_turns_for_required_seeds() {
    for seed in SEEDS {
        let mut session = GameSession::new_for_playing(seed);
        let initial_semantic_state = semantic_state(&session);
        let initial_nutrition = session.snapshot().nutrition;
        let report = run_to_turn(&mut session, TARGET_TURN, HeadlessPolicy::survival_v1()).unwrap();

        assert_eq!(report.accepted_turns, TARGET_TURN, "seed={seed}");
        assert_eq!(report.final_state, RunState::Playing, "seed={seed}");
        assert!((report.accepted_turns..=report.accepted_turns * 16)
            .contains(&report.submitted_commands));
        assert_ne!(
            semantic_state(&session),
            initial_semantic_state,
            "seed={seed}"
        );
        assert!(
            session.snapshot().nutrition < initial_nutrition,
            "seed={seed}"
        );
    }
}

#[test]
fn survival_policy_hash_is_stable_across_three_runs_per_seed() {
    for seed in SEEDS {
        let hashes = (0..3)
            .map(|_| {
                let mut session = GameSession::new_for_playing(seed);
                run_to_turn(&mut session, TARGET_TURN, HeadlessPolicy::survival_v1())
                    .unwrap()
                    .final_hash
            })
            .collect::<Vec<_>>();

        assert!(
            hashes.windows(2).all(|pair| pair[0] == pair[1]),
            "seed={seed}"
        );
    }
}

#[test]
fn causal_fixture_covers_every_required_witness_for_each_seed() {
    for (seed, expected_hash) in SEEDS.into_iter().zip(EXPECTED_CAUSAL_HASHES) {
        let (summary, hash, turn) = run_causal_fixture(seed);

        assert!(turn >= TARGET_TURN, "seed={seed}");
        assert_eq!(hash.0, expected_hash, "seed={seed}");
        assert_eq!(summary.validate_required(), Ok(()), "seed={seed}");
        for witness in REQUIRED_CAUSAL_WITNESSES {
            assert!(
                summary.count(witness) > 0,
                "seed={seed} witness={witness:?}"
            );
        }
    }
}

#[test]
fn causal_witness_multiset_and_final_hash_are_stable_across_three_runs() {
    for seed in SEEDS {
        let runs = (0..3)
            .map(|_| {
                let (summary, hash, _) = run_causal_fixture(seed);
                (summary, hash)
            })
            .collect::<Vec<_>>();

        assert!(
            runs.windows(2).all(|pair| pair[0] == pair[1]),
            "seed={seed}"
        );
    }
}

#[test]
fn causal_validator_rejects_event_only_turn_only_and_missing_witnesses() {
    let session = GameSession::new_for_playing(42);
    let projection = CausalProjection::from_session(&session);
    let mut summary = CausalSummary::default();
    let event_only = TurnOutcome {
        accepted: true,
        turn_advanced: true,
        events: vec![GameEvent::PrayerOffered {
            entity: session.world().player_id(),
            cooldown_after: 100,
        }],
        snapshot_hash: SnapshotHash("event-only".to_string()),
        next_state: RunState::Playing,
    };

    summary.observe(&projection, CommandIntent::Pray, &event_only, &projection);
    assert_eq!(summary.total_count(), 0);
    assert!(summary.validate_required().is_err());

    let (complete, _, _) = run_causal_fixture(42);
    assert!(complete
        .without(CausalWitness::ArmorDefense)
        .validate_required()
        .is_err());
}

fn run_causal_fixture(seed: u64) -> (CausalSummary, SnapshotHash, u64) {
    let mut session = GameSession::new_for_playing(seed);
    let mut summary = CausalSummary::default();
    let food = inventory_item(&session, ItemKind::FoodRation);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().nutrition = 100;
    });
    submit_and_observe(
        &mut session,
        &mut summary,
        CommandIntent::Eat { item: food },
    );

    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(aihack::core::Pos { x: 5, y: 10 });
        world.saved().entities.set_alive(EntityId(3), false);
        world
            .saved()
            .entities
            .actor_stats_mut(EntityId(2))
            .expect("jackal stats must exist")
            .hp = 100;
    });
    for _ in 0..16 {
        submit_and_observe(&mut session, &mut summary, CommandIntent::Wait);
        if summary.count(CausalWitness::MonsterSpeed) > 0
            && summary.count(CausalWitness::MonsterAi) > 0
        {
            break;
        }
    }

    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(aihack::core::Pos { x: 5, y: 5 });
        let saved = world.saved();
        let current_level = saved.current_level;
        saved.entities.set_actor_location(
            EntityId(2),
            current_level,
            aihack::core::Pos { x: 6, y: 5 },
        );
        let jackal = saved
            .entities
            .actor_stats_mut(EntityId(2))
            .expect("jackal stats must exist");
        jackal.hp = 100;
        jackal.ai_kind = Some(MonsterAiKind::Stationary);
    });
    submit_and_observe(&mut session, &mut summary, CommandIntent::Pray);
    submit_and_observe(
        &mut session,
        &mut summary,
        CommandIntent::Move(Direction::East),
    );

    let floating_eye = aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.set_alive(EntityId(2), false);
        world.saved().entities.set_alive(EntityId(3), false);
        world.set_player_pos(aihack::core::Pos { x: 5, y: 5 });
        world
            .saved()
            .entities
            .spawn_monster(MonsterKind::FloatingEye, aihack::core::Pos { x: 6, y: 5 })
    });
    submit_and_observe(
        &mut session,
        &mut summary,
        CommandIntent::Move(Direction::East),
    );

    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.set_alive(floating_eye, false);
        world.saved().entities.set_alive(EntityId(3), false);
        world.saved().paralysis_turns = 0;
        world.set_player_pos(aihack::core::Pos { x: 5, y: 5 });
        let saved = world.saved();
        let current_level = saved.current_level;
        let player_id = saved.player_id;
        saved.entities.set_alive(EntityId(2), true);
        saved.entities.set_actor_location(
            EntityId(2),
            current_level,
            aihack::core::Pos { x: 6, y: 5 },
        );
        let jackal = saved
            .entities
            .actor_stats_mut(EntityId(2))
            .expect("jackal stats must exist");
        jackal.hp = 1;
        jackal.ai_kind = Some(MonsterAiKind::Stationary);
        saved
            .entities
            .actor_stats_mut(player_id)
            .expect("player stats must exist")
            .hp = 100;
    });
    for _ in 0..32 {
        if !session
            .world()
            .entities()
            .get(EntityId(2))
            .expect("jackal must exist")
            .is_alive_actor()
        {
            break;
        }
        submit_and_observe(
            &mut session,
            &mut summary,
            CommandIntent::Move(Direction::East),
        );
    }
    assert!(!session
        .world()
        .entities()
        .get(EntityId(2))
        .expect("jackal must exist")
        .is_alive_actor());

    let corpse = world_item(&session, ItemKind::CorpseJackal);
    let corpse_pos = match session.world().entities().item_location(corpse) {
        Some(EntityLocation::OnMap { pos, .. }) => pos,
        location => panic!("corpse must be on the map, got {location:?}"),
    };
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(corpse_pos);
    });
    submit_and_observe(&mut session, &mut summary, CommandIntent::Pickup);
    submit_and_observe(
        &mut session,
        &mut summary,
        CommandIntent::Eat { item: corpse },
    );

    let armor = world_item(&session, ItemKind::ArmorLeather);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        let owner = world.saved().player_id;
        assert!(world
            .saved()
            .entities
            .set_item_location(armor, EntityLocation::Inventory { owner }));
        if !world.saved().inventory.contains(armor) {
            world
                .saved()
                .inventory
                .add_existing_with_next_letter(armor)
                .expect("armor must receive an inventory letter");
        }
    });
    submit_and_observe(
        &mut session,
        &mut summary,
        CommandIntent::Wear { item: armor },
    );

    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters();
        world.saved().paralysis_turns = 0;
    });
    while session.turn() < TARGET_TURN {
        submit_and_observe(&mut session, &mut summary, CommandIntent::Wait);
    }
    submit_and_observe(&mut session, &mut summary, CommandIntent::Quit);

    (summary, session.snapshot().stable_hash(), session.turn())
}

fn submit_and_observe(
    session: &mut GameSession,
    summary: &mut CausalSummary,
    command: CommandIntent,
) {
    let before = CausalProjection::from_session(session);
    let outcome = session.submit(command);
    let after = CausalProjection::from_session(session);
    assert!(
        outcome.accepted,
        "command={command:?} outcome={outcome:?} pos={:?}",
        session.world().player_pos()
    );
    summary.observe(&before, command, &outcome, &after);
}

fn inventory_item(session: &GameSession, kind: ItemKind) -> EntityId {
    session
        .world()
        .entities()
        .entities()
        .iter()
        .find(|entity| {
            entity.kind() == EntityKind::Item(kind)
                && matches!(
                    entity.item(),
                    Some((_, _, EntityLocation::Inventory { owner }, _, _))
                        if owner == session.world().player_id()
                )
        })
        .map(|entity| entity.id)
        .expect("fixture inventory must contain the requested item")
}

fn world_item(session: &GameSession, kind: ItemKind) -> EntityId {
    session
        .world()
        .entities()
        .entities()
        .iter()
        .find(|entity| entity.kind() == EntityKind::Item(kind))
        .map(|entity| entity.id)
        .expect("fixture world must contain the requested item")
}
