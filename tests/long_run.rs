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
const ITEMS_TOML: &str = include_str!("../crates/aihack-content/src/data/items.toml");
const MONSTERS_TOML: &str = include_str!("../crates/aihack-content/src/data/monsters.toml");
const LEVEL_1_TOML: &str = include_str!("../crates/aihack-content/src/data/levels/main_1.toml");
const LEVEL_2_TOML: &str = include_str!("../crates/aihack-content/src/data/levels/main_2.toml");
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
fn causal_field_only_ab_loses_exactly_one_witness_and_preserves_other_records() {
    let (complete, complete_trace) = run_field_only_causal_matrix(42, None);
    assert_eq!(
        complete.total_count(),
        REQUIRED_CAUSAL_WITNESSES.len() as u64,
        "complete records={:?}",
        complete.records()
    );
    for omitted in REQUIRED_CAUSAL_WITNESSES {
        let (summary, trace) = run_field_only_causal_matrix(42, Some(omitted));
        let repeated = run_field_only_causal_matrix(42, Some(omitted));
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
        assert_eq!(trace, complete_trace, "omitted={omitted:?}");
        let expected_records = complete
            .records()
            .iter()
            .filter(|record| record.witness != omitted)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(summary.records(), expected_records, "omitted={omitted:?}");
        assert_eq!((summary, trace), repeated, "omitted={omitted:?}");
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
    let mut session = GameSession::new_for_playing(seed);
    let mut summary = CausalSummary::default();
    record_independent_monster_attribution(seed, &mut summary);
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
            .unwrap()
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
            let letter = world
                .saved()
                .inventory
                .add_existing_with_next_letter(armor)
                .expect("armor must receive an inventory letter");
            assert!(world.saved().entities.set_item_letter(armor, letter));
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

fn session_with_content(
    seed: u64,
    items: &str,
    monsters: &str,
) -> (GameSession, aihack::data::ContentRegistry) {
    let registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        items,
        monsters,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    (
        GameSession::try_new_for_playing_with_registry(seed, &registry).unwrap(),
        registry,
    )
}

fn mutate_content_session<T>(
    session: &mut GameSession,
    registry: &aihack::data::ContentRegistry,
    configure: impl FnOnce(&mut aihack::core::save::SavedWorldV1) -> T,
) -> T {
    let mut save = session.to_save_data();
    let result = configure(&mut save.world);
    *session = GameSession::from_save_data_with_registry(save, registry).unwrap();
    result
}

fn run_field_only_causal_matrix(
    seed: u64,
    neutralized: Option<aihack::core::causal::CausalWitness>,
) -> (CausalSummary, Vec<&'static str>) {
    use aihack::core::causal::CausalWitness;

    let mut summary = CausalSummary::default();
    let mut trace = Vec::new();

    let food_control_items = ITEMS_TOML.replacen("nutrition=800", "nutrition=1", 1);
    let food_active_items = if neutralized == Some(CausalWitness::FoodNutrition) {
        food_control_items.as_str()
    } else {
        ITEMS_TOML
    };
    let (mut food_active, food_active_registry) =
        session_with_content(seed, food_active_items, MONSTERS_TOML);
    let (mut food_control, food_control_registry) =
        session_with_content(seed, &food_control_items, MONSTERS_TOML);
    for (session, registry) in [
        (&mut food_active, &food_active_registry),
        (&mut food_control, &food_control_registry),
    ] {
        mutate_content_session(session, registry, |world| {
            world.entities.clear_monsters();
            world.nutrition = 100;
        });
    }
    let food = inventory_item(&food_active, ItemKind::FoodRation);
    assert_eq!(food, inventory_item(&food_control, ItemKind::FoodRation));
    let active_before = CausalProjection::from_session(&food_active);
    let control_before = CausalProjection::from_session(&food_control);
    assert!(
        food_active
            .submit(CommandIntent::Eat { item: food })
            .accepted
    );
    assert!(
        food_control
            .submit(CommandIntent::Eat { item: food })
            .accepted
    );
    trace.extend(["food:eat-active", "food:eat-control", "food:observe-pair"]);
    summary.observe_item_nutrition_pair(
        &active_before,
        &CausalProjection::from_session(&food_active),
        &control_before,
        &CausalProjection::from_session(&food_control),
        food,
    );

    let corpse_control_items = ITEMS_TOML.replacen("nutrition=50", "nutrition=1", 1);
    let corpse_active_items = if neutralized == Some(CausalWitness::CorpseNutrition) {
        corpse_control_items.as_str()
    } else {
        ITEMS_TOML
    };
    let (mut corpse_active, corpse_active_registry) =
        session_with_content(seed, corpse_active_items, MONSTERS_TOML);
    let (mut corpse_control, corpse_control_registry) =
        session_with_content(seed, &corpse_control_items, MONSTERS_TOML);
    for (session, registry) in [
        (&mut corpse_active, &corpse_active_registry),
        (&mut corpse_control, &corpse_control_registry),
    ] {
        mutate_content_session(session, registry, |world| {
            world.entities.set_alive(EntityId(3), false);
            let jackal = world.entities.actor_stats_mut(EntityId(2)).unwrap();
            jackal.hp = 1;
            jackal.ai_kind = Some(MonsterAiKind::Stationary);
            let player = world.entities.actor_stats_mut(world.player_id).unwrap();
            player.hit_bonus = 100;
        });
    }
    let corpse_active_before_kill = CausalProjection::from_session(&corpse_active);
    let active_kill = corpse_active.submit(CommandIntent::Move(Direction::East));
    let control_kill = corpse_control.submit(CommandIntent::Move(Direction::East));
    assert!(active_kill.accepted && control_kill.accepted);
    summary.observe(
        &corpse_active_before_kill,
        CommandIntent::Move(Direction::East),
        &active_kill,
        &CausalProjection::from_session(&corpse_active),
    );
    trace.extend([
        "corpse:kill-active",
        "corpse:kill-control",
        "corpse:producer-observer",
    ]);
    let corpse = world_item(&corpse_active, ItemKind::CorpseJackal);
    assert_eq!(corpse, world_item(&corpse_control, ItemKind::CorpseJackal));
    let corpse_pos = match corpse_active.world().entities().item_location(corpse) {
        Some(EntityLocation::OnMap { pos, .. }) => pos,
        other => panic!("corpse must be produced on map: {other:?}"),
    };
    for (session, registry) in [
        (&mut corpse_active, &corpse_active_registry),
        (&mut corpse_control, &corpse_control_registry),
    ] {
        mutate_content_session(session, registry, |world| {
            world
                .entities
                .set_actor_location(world.player_id, world.current_level, corpse_pos);
        });
        assert!(session.submit(CommandIntent::Pickup).accepted);
    }
    trace.extend(["corpse:pickup-active", "corpse:pickup-control"]);
    let active_before = CausalProjection::from_session(&corpse_active);
    let control_before = CausalProjection::from_session(&corpse_control);
    assert!(
        corpse_active
            .submit(CommandIntent::Eat { item: corpse })
            .accepted
    );
    assert!(
        corpse_control
            .submit(CommandIntent::Eat { item: corpse })
            .accepted
    );
    trace.extend([
        "corpse:eat-active",
        "corpse:eat-control",
        "corpse:observe-pair",
    ]);
    summary.observe_item_nutrition_pair(
        &active_before,
        &CausalProjection::from_session(&corpse_active),
        &control_before,
        &CausalProjection::from_session(&corpse_control),
        corpse,
    );

    let armor_control_items = ITEMS_TOML.replacen("ac_bonus=1", "ac_bonus=0", 1);
    let armor_active_items = if neutralized == Some(CausalWitness::ArmorDefense) {
        armor_control_items.as_str()
    } else {
        ITEMS_TOML
    };
    let (mut armor_active, armor_active_registry) =
        session_with_content(seed, armor_active_items, MONSTERS_TOML);
    let (mut armor_control, armor_control_registry) =
        session_with_content(seed, &armor_control_items, MONSTERS_TOML);
    let armor = world_item(&armor_active, ItemKind::ArmorLeather);
    assert_eq!(armor, world_item(&armor_control, ItemKind::ArmorLeather));
    for (session, registry) in [
        (&mut armor_active, &armor_active_registry),
        (&mut armor_control, &armor_control_registry),
    ] {
        mutate_content_session(session, registry, |world| {
            world.entities.clear_monsters();
            let owner = world.player_id;
            assert!(world
                .entities
                .set_item_location(armor, EntityLocation::Inventory { owner }));
            let letter = world
                .inventory
                .add_existing_with_next_letter(armor)
                .unwrap();
            assert!(world.entities.set_item_letter(armor, letter));
        });
    }
    let active_before = CausalProjection::from_session(&armor_active);
    let control_before = CausalProjection::from_session(&armor_control);
    assert!(
        armor_active
            .submit(CommandIntent::Wear { item: armor })
            .accepted
    );
    assert!(
        armor_control
            .submit(CommandIntent::Wear { item: armor })
            .accepted
    );
    trace.extend([
        "armor:wear-active",
        "armor:wear-control",
        "armor:observe-pair",
    ]);
    summary.observe_armor_defense_pair(
        &active_before,
        &CausalProjection::from_session(&armor_active),
        &control_before,
        &CausalProjection::from_session(&armor_control),
        armor,
    );

    let speed_control_monsters = MONSTERS_TOML.replacen("speed=12", "speed=0", 1);
    let speed_active_monsters = if neutralized == Some(CausalWitness::MonsterSpeed) {
        speed_control_monsters.as_str()
    } else {
        MONSTERS_TOML
    };
    let (mut speed_active, speed_active_registry) =
        session_with_content(seed, ITEMS_TOML, speed_active_monsters);
    let (mut speed_control, speed_control_registry) =
        session_with_content(seed, ITEMS_TOML, &speed_control_monsters);
    for (session, registry) in [
        (&mut speed_active, &speed_active_registry),
        (&mut speed_control, &speed_control_registry),
    ] {
        mutate_content_session(session, registry, |world| {
            world.entities.set_actor_location(
                world.player_id,
                world.current_level,
                aihack::core::Pos { x: 5, y: 10 },
            );
            world.entities.set_alive(EntityId(3), false);
            let jackal = world.entities.actor_stats_mut(EntityId(2)).unwrap();
            jackal.hp = 100;
            jackal.max_hp = 100;
            jackal.ai_kind = Some(MonsterAiKind::ChaseVisiblePlayer);
        });
    }
    let active_before = CausalProjection::from_session(&speed_active);
    let control_before = CausalProjection::from_session(&speed_control);
    assert!(speed_active.submit(CommandIntent::Wait).accepted);
    assert!(speed_control.submit(CommandIntent::Wait).accepted);
    trace.extend([
        "speed:wait-active",
        "speed:wait-control",
        "speed:observe-pair",
    ]);
    summary.observe_monster_speed_pair(
        &active_before,
        &CausalProjection::from_session(&speed_active),
        &control_before,
        &CausalProjection::from_session(&speed_control),
        EntityId(2),
    );

    let ai_control_monsters = MONSTERS_TOML.replacen("ai=\"wander\"", "ai=\"stationary\"", 1);
    let ai_active_monsters = if neutralized == Some(CausalWitness::MonsterAi) {
        ai_control_monsters.as_str()
    } else {
        MONSTERS_TOML
    };
    let (mut ai_active, ai_active_registry) =
        session_with_content(seed, ITEMS_TOML, ai_active_monsters);
    let (mut ai_control, ai_control_registry) =
        session_with_content(seed, ITEMS_TOML, &ai_control_monsters);
    for (session, registry) in [
        (&mut ai_active, &ai_active_registry),
        (&mut ai_control, &ai_control_registry),
    ] {
        mutate_content_session(session, registry, |world| {
            world.entities.set_actor_location(
                world.player_id,
                world.current_level,
                aihack::core::Pos { x: 5, y: 10 },
            );
            world.entities.set_alive(EntityId(3), false);
        });
    }
    let active_before = CausalProjection::from_session(&ai_active);
    let control_before = CausalProjection::from_session(&ai_control);
    assert!(ai_active.submit(CommandIntent::Wait).accepted);
    assert!(ai_control.submit(CommandIntent::Wait).accepted);
    trace.extend(["ai:wait-active", "ai:wait-control", "ai:observe-pair"]);
    summary.observe_monster_ai_pair(
        &active_before,
        &CausalProjection::from_session(&ai_active),
        &control_before,
        &CausalProjection::from_session(&ai_control),
        EntityId(2),
    );

    let passive_enabled_monsters = MONSTERS_TOML.replacen(
        "difficulty=2",
        "difficulty=2\npassive=\"paralyze_on_melee\"",
        1,
    );
    let passive_active_monsters = if neutralized == Some(CausalWitness::MonsterPassive) {
        MONSTERS_TOML
    } else {
        passive_enabled_monsters.as_str()
    };
    let (mut passive_active, passive_active_registry) =
        session_with_content(seed, ITEMS_TOML, passive_active_monsters);
    let (mut passive_control, passive_control_registry) =
        session_with_content(seed, ITEMS_TOML, MONSTERS_TOML);
    for (session, registry) in [
        (&mut passive_active, &passive_active_registry),
        (&mut passive_control, &passive_control_registry),
    ] {
        mutate_content_session(session, registry, |world| {
            world.entities.set_alive(EntityId(2), false);
            world.entities.set_actor_location(
                world.player_id,
                world.current_level,
                aihack::core::Pos { x: 5, y: 5 },
            );
            world.entities.set_actor_location(
                EntityId(3),
                world.current_level,
                aihack::core::Pos { x: 6, y: 5 },
            );
            let eye = world.entities.actor_stats_mut(EntityId(3)).unwrap();
            eye.hp = 100;
            eye.max_hp = 100;
            let player = world.entities.actor_stats_mut(world.player_id).unwrap();
            player.hit_bonus = 100;
        });
    }
    let active_before = CausalProjection::from_session(&passive_active);
    let control_before = CausalProjection::from_session(&passive_control);
    assert!(
        passive_active
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    assert!(
        passive_control
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    trace.extend([
        "passive:attack-active",
        "passive:attack-control",
        "passive:observe-pair",
    ]);
    summary.observe_monster_passive_pair(
        &active_before,
        &CausalProjection::from_session(&passive_active),
        &control_before,
        &CausalProjection::from_session(&passive_control),
        EntityId(3),
    );

    let difficulty_control_monsters = MONSTERS_TOML.replacen("difficulty=1", "difficulty=2", 1);
    let difficulty_active_monsters = if neutralized == Some(CausalWitness::MonsterDifficultyEconomy)
    {
        difficulty_control_monsters.as_str()
    } else {
        MONSTERS_TOML
    };
    let (mut difficulty_active, difficulty_active_registry) =
        session_with_content(seed, ITEMS_TOML, difficulty_active_monsters);
    let (mut difficulty_control, difficulty_control_registry) =
        session_with_content(seed, ITEMS_TOML, &difficulty_control_monsters);
    for (session, registry) in [
        (&mut difficulty_active, &difficulty_active_registry),
        (&mut difficulty_control, &difficulty_control_registry),
    ] {
        mutate_content_session(session, registry, |world| {
            world.entities.set_alive(EntityId(3), false);
            let jackal = world.entities.actor_stats_mut(EntityId(2)).unwrap();
            jackal.hp = 1;
            jackal.ai_kind = Some(MonsterAiKind::Stationary);
            let player = world.entities.actor_stats_mut(world.player_id).unwrap();
            player.hit_bonus = 100;
        });
    }
    let active_before = CausalProjection::from_session(&difficulty_active);
    let control_before = CausalProjection::from_session(&difficulty_control);
    assert!(
        difficulty_active
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    assert!(
        difficulty_control
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    trace.extend([
        "difficulty:kill-active",
        "difficulty:kill-control",
        "difficulty:observe-pair",
    ]);
    summary.observe_monster_difficulty_pair(
        &active_before,
        &CausalProjection::from_session(&difficulty_active),
        &control_before,
        &CausalProjection::from_session(&difficulty_control),
        EntityId(2),
    );

    let mut luck_active = GameSession::new_for_playing(seed);
    let mut luck_control = GameSession::new_for_playing(seed);
    for session in [&mut luck_active, &mut luck_control] {
        aihack::testing::SessionBuilder::mutate(session, |world| {
            world.saved().entities.set_alive(EntityId(3), false);
            let jackal = world.saved().entities.actor_stats_mut(EntityId(2)).unwrap();
            jackal.hp = 100;
            jackal.max_hp = 100;
            jackal.ai_kind = Some(MonsterAiKind::Stationary);
        });
        assert!(session.submit(CommandIntent::Pray).accepted);
    }
    aihack::testing::SessionBuilder::mutate(&mut luck_control, |world| world.saved().luck = 0);
    if neutralized == Some(CausalWitness::PrayerLuckCombat) {
        aihack::testing::SessionBuilder::mutate(&mut luck_active, |world| world.saved().luck = 0);
    }
    trace.extend(["luck:pray-active", "luck:pray-control"]);
    let active_before = CausalProjection::from_session(&luck_active);
    let control_before = CausalProjection::from_session(&luck_control);
    let active_outcome = luck_active.submit(CommandIntent::Move(Direction::East));
    let control_outcome = luck_control.submit(CommandIntent::Move(Direction::East));
    trace.extend([
        "luck:attack-active",
        "luck:attack-control",
        "luck:observe-pair",
    ]);
    summary.observe_prayer_luck_pair(
        &active_before,
        &active_outcome,
        &CausalProjection::from_session(&luck_active),
        &control_before,
        &control_outcome,
        &CausalProjection::from_session(&luck_control),
    );

    let mut gold_active = GameSession::new_for_playing(seed);
    let mut gold_control = GameSession::new_for_playing(seed);
    for session in [&mut gold_active, &mut gold_control] {
        aihack::testing::SessionBuilder::mutate(session, |world| {
            world.saved().entities.clear_monsters();
            world.saved().gold = 0;
        });
    }
    if neutralized != Some(CausalWitness::GoldScore) {
        aihack::testing::SessionBuilder::mutate(&mut gold_active, |world| world.saved().gold = 10);
    }
    let active_before = CausalProjection::from_session(&gold_active);
    let control_before = CausalProjection::from_session(&gold_control);
    assert!(gold_active.submit(CommandIntent::Quit).accepted);
    assert!(gold_control.submit(CommandIntent::Quit).accepted);
    trace.extend(["gold:quit-active", "gold:quit-control", "gold:observe-pair"]);
    summary.observe_gold_score_pair(
        &active_before,
        &CausalProjection::from_session(&gold_active),
        &control_before,
        &CausalProjection::from_session(&gold_control),
    );

    (summary, trace)
}

fn record_independent_monster_attribution(seed: u64, summary: &mut CausalSummary) {
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

    let prepare_economy = |difficulty| {
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
            jackal.difficulty = difficulty;
            let player = saved.entities.actor_stats_mut(saved.player_id).unwrap();
            player.hit_bonus = 100;
        });
        session
    };
    let mut active = prepare_economy(1);
    let mut control = prepare_economy(2);
    let active_before = CausalProjection::from_session(&active);
    let control_before = CausalProjection::from_session(&control);
    assert!(active.submit(CommandIntent::Move(Direction::East)).accepted);
    assert!(
        control
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    summary.observe_monster_difficulty_pair(
        &active_before,
        &CausalProjection::from_session(&active),
        &control_before,
        &CausalProjection::from_session(&control),
        EntityId(2),
    );
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
