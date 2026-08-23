use aihack::core::{
    causal::{CausalProjection, CausalSummary, REQUIRED_CAUSAL_WITNESSES},
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
// Report 25의 alive/HP/max_hp 관계와 armor fixture를 함께 만족하는 정합 상태의 기준 hash다.
const EXPECTED_CAUSAL_HASHES: [&str; 3] =
    ["e9737367c68c053d", "67074441a11c89da", "0c22555fc8344443"];

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
    let runs = SEEDS
        .into_iter()
        .map(|seed| (seed, run_causal_fixture(seed)))
        .collect::<Vec<_>>();
    for (seed, (_, hash, _)) in &runs {
        eprintln!("causal fixture seed={seed} hash={}", hash.0);
    }
    for ((seed, (summary, hash, turn)), expected_hash) in
        runs.into_iter().zip(EXPECTED_CAUSAL_HASHES)
    {
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
fn causal_validator_rejects_event_only_and_turn_only_changes() {
    let session = GameSession::new_for_playing(42);
    let projection = CausalProjection::from_session(&session);
    let mut summary = CausalSummary::default();
    let event_only = TurnOutcome {
        accepted: true,
        turn_advanced: false,
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

    let turn_only = TurnOutcome {
        accepted: true,
        turn_advanced: true,
        events: Vec::new(),
        snapshot_hash: SnapshotHash("turn-only".to_string()),
        next_state: RunState::Playing,
    };
    summary.observe(&projection, CommandIntent::Wait, &turn_only, &projection);
    assert_eq!(summary.total_count(), 0);
}

#[test]
fn causal_actual_producer_removal_loses_exactly_one_required_witness() {
    let (complete, _, complete_turn) = run_causal_fixture(42);
    assert_eq!(
        complete.total_count(),
        REQUIRED_CAUSAL_WITNESSES.len() as u64
    );
    for omitted in REQUIRED_CAUSAL_WITNESSES {
        let (summary, hash, turn) = run_causal_fixture_without(42, omitted);
        let repeated = run_causal_fixture_without(42, omitted);
        assert_eq!(
            summary.validate_required(),
            Err(vec![omitted]),
            "omitted={omitted:?} records={:?}",
            summary.records()
        );
        for witness in REQUIRED_CAUSAL_WITNESSES {
            assert_eq!(
                summary.count(witness) > 0,
                witness != omitted,
                "omitted={omitted:?} witness={witness:?}"
            );
        }
        assert_eq!(summary.total_count(), 8, "omitted={omitted:?}");
        assert_eq!(summary.records().len(), 8, "omitted={omitted:?}");
        assert_eq!(turn, complete_turn, "omitted={omitted:?}");
        assert_eq!((summary, hash, turn), repeated, "omitted={omitted:?}");
    }
}

#[test]
fn gold_score_witness_uses_a_paired_production_score() {
    let causal_source = include_str!("../crates/aihack-runtime/src/causal.rs");
    assert!(
        causal_source.contains("score::paired_gold_scores"),
        "GoldScore projection must call the production paired score path"
    );
    assert!(
        !causal_source.contains("let score_without_gold = i64::from(world.kill_count())"),
        "GoldScore projection must not duplicate the production score formula"
    );
}

fn run_causal_fixture(seed: u64) -> (CausalSummary, SnapshotHash, u64) {
    run_causal_fixture_with_omission(seed, None)
}

fn run_causal_fixture_without(
    seed: u64,
    omitted: aihack::core::causal::CausalWitness,
) -> (CausalSummary, SnapshotHash, u64) {
    run_causal_fixture_with_omission(seed, Some(omitted))
}

fn run_causal_fixture_with_omission(
    seed: u64,
    omitted: Option<aihack::core::causal::CausalWitness>,
) -> (CausalSummary, SnapshotHash, u64) {
    use aihack::core::causal::CausalWitness;

    let mut session = GameSession::new_for_playing(seed);
    let mut summary = CausalSummary::default();
    record_independent_monster_attribution(seed, &mut summary, omitted);
    let food = inventory_item(&session, ItemKind::FoodRation);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().nutrition = 100;
    });
    if omitted != Some(CausalWitness::FoodNutrition) {
        submit_and_observe(
            &mut session,
            &mut summary,
            CommandIntent::Eat { item: food },
        );
    }

    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(aihack::core::Pos { x: 5, y: 10 });
        world.saved().entities.set_alive(EntityId(3), false);
        let jackal = world
            .saved()
            .entities
            .actor_stats_mut(EntityId(2))
            .expect("jackal stats must exist");
        jackal.hp = 100;
        jackal.max_hp = 100;
    });
    for _ in 0..16 {
        let before_pos = session
            .world()
            .entities()
            .actor_location(EntityId(2))
            .unwrap();
        submit_and_observe(&mut session, &mut summary, CommandIntent::Wait);
        let after_pos = session
            .world()
            .entities()
            .actor_location(EntityId(2))
            .unwrap();
        if after_pos != before_pos {
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
        jackal.max_hp = 100;
        jackal.ai_kind = Some(MonsterAiKind::Stationary);
    });
    if omitted != Some(CausalWitness::PrayerLuckCombat) {
        submit_and_observe(&mut session, &mut summary, CommandIntent::Pray);
    }
    submit_and_observe(
        &mut session,
        &mut summary,
        CommandIntent::Move(Direction::East),
    );

    let floating_eye = if omitted == Some(CausalWitness::MonsterPassive) {
        None
    } else {
        let entity = aihack::testing::SessionBuilder::mutate(&mut session, |world| {
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
        Some(entity)
    };

    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        if let Some(floating_eye) = floating_eye {
            world.saved().entities.set_alive(floating_eye, false);
        }
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
        let player = saved
            .entities
            .actor_stats_mut(player_id)
            .expect("player stats must exist");
        player.hp = 100;
        player.max_hp = 100;
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
    if omitted != Some(CausalWitness::CorpseNutrition) {
        submit_and_observe(
            &mut session,
            &mut summary,
            CommandIntent::Eat { item: corpse },
        );
    }

    let armor = world_item(&session, ItemKind::ArmorLeather);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        let owner = world.saved().player_id;
        assert!(world
            .saved()
            .entities
            .set_item_location(armor, EntityLocation::Inventory { owner }));
        if !world.saved().inventory.contains(armor) {
            let letter = world
                .saved()
                .inventory
                .add_existing_with_next_letter(armor)
                .expect("armor must receive an inventory letter");
            assert!(world.saved().entities.set_item_letter(armor, letter));
        }
    });
    if omitted != Some(CausalWitness::ArmorDefense) {
        submit_and_observe(
            &mut session,
            &mut summary,
            CommandIntent::Wear { item: armor },
        );
    }

    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters();
        world.saved().paralysis_turns = 0;
    });
    while session.turn() < TARGET_TURN {
        submit_and_observe(&mut session, &mut summary, CommandIntent::Wait);
    }
    if omitted == Some(CausalWitness::GoldScore) {
        aihack::testing::SessionBuilder::mutate(&mut session, |world| {
            world.saved().gold = 0;
        });
    }
    submit_and_observe(&mut session, &mut summary, CommandIntent::Quit);

    (summary, session.snapshot().stable_hash(), session.turn())
}

fn record_independent_monster_attribution(
    seed: u64,
    summary: &mut CausalSummary,
    omitted: Option<aihack::core::causal::CausalWitness>,
) {
    use aihack::core::causal::CausalWitness;
    let prepare = |speed: i16, ai_kind: MonsterAiKind| {
        let mut session = GameSession::new_for_playing(seed);
        aihack::testing::SessionBuilder::mutate(&mut session, |world| {
            world.set_player_pos(aihack::core::Pos { x: 5, y: 10 });
            world.saved().entities.set_alive(EntityId(3), false);
            let jackal = world
                .saved()
                .entities
                .actor_stats_mut(EntityId(2))
                .expect("jackal stats must exist");
            jackal.hp = 100;
            jackal.max_hp = 100;
            jackal.speed = speed;
            jackal.ai_kind = Some(ai_kind);
        });
        session
    };

    if omitted != Some(CausalWitness::MonsterSpeed) {
        let mut speed_active = prepare(12, MonsterAiKind::ChaseVisiblePlayer);
        let mut speed_control = prepare(0, MonsterAiKind::ChaseVisiblePlayer);
        let speed_active_before = CausalProjection::from_session(&speed_active);
        let speed_control_before = CausalProjection::from_session(&speed_control);
        assert!(speed_active.submit(CommandIntent::Wait).accepted);
        assert!(speed_control.submit(CommandIntent::Wait).accepted);
        summary.observe_monster_speed_pair(
            &speed_active_before,
            &CausalProjection::from_session(&speed_active),
            &speed_control_before,
            &CausalProjection::from_session(&speed_control),
            EntityId(2),
        );
    }

    if omitted != Some(CausalWitness::MonsterAi) {
        let mut ai_active = prepare(12, MonsterAiKind::ChaseVisiblePlayer);
        let mut ai_control = prepare(12, MonsterAiKind::Stationary);
        let ai_active_before = CausalProjection::from_session(&ai_active);
        let ai_control_before = CausalProjection::from_session(&ai_control);
        assert!(ai_active.submit(CommandIntent::Wait).accepted);
        assert!(ai_control.submit(CommandIntent::Wait).accepted);
        summary.observe_monster_ai_pair(
            &ai_active_before,
            &CausalProjection::from_session(&ai_active),
            &ai_control_before,
            &CausalProjection::from_session(&ai_control),
            EntityId(2),
        );
    }

    if omitted != Some(CausalWitness::MonsterDifficultyEconomy) {
        let prepare_economy = || {
            let mut session = GameSession::new_for_playing(seed);
            aihack::testing::SessionBuilder::mutate(&mut session, |world| {
                world.saved().entities.set_alive(EntityId(3), false);
                world.set_player_pos(aihack::core::Pos { x: 5, y: 5 });
                let saved = world.saved();
                let current_level = saved.current_level;
                saved.entities.set_actor_location(
                    EntityId(2),
                    current_level,
                    aihack::core::Pos { x: 6, y: 5 },
                );
                let jackal = saved.entities.actor_stats_mut(EntityId(2)).unwrap();
                jackal.hp = 1;
                jackal.ai_kind = Some(MonsterAiKind::Stationary);
                let player = saved.entities.actor_stats_mut(saved.player_id).unwrap();
                player.hit_bonus = 100;
            });
            session
        };
        let mut active = prepare_economy();
        let mut control = prepare_economy();
        let active_before = CausalProjection::from_session(&active);
        let control_before = CausalProjection::from_session(&control);
        assert!(active.submit(CommandIntent::Move(Direction::East)).accepted);
        assert!(control.submit(CommandIntent::Wait).accepted);
        summary.observe_monster_difficulty_pair(
            &active_before,
            &CausalProjection::from_session(&active),
            &control_before,
            &CausalProjection::from_session(&control),
            EntityId(2),
        );
    }
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
