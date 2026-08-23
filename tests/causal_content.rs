use aihack::{
    core::{
        causal::{CausalField, CausalProjection, CausalScenario, CausalSummary, CausalWitness},
        CommandIntent, Direction, EntityId, GameEvent, GameSession, RunState,
    },
    domain::{
        entity::{EntityKind, EntityLocation},
        item::{item_data, ItemKind},
        monster::MonsterAiKind,
        status::HungerState,
    },
};

const ITEMS_TOML: &str = include_str!("../crates/aihack-content/src/data/items.toml");
const MONSTERS_TOML: &str = include_str!("../crates/aihack-content/src/data/monsters.toml");
const LEVEL_1_TOML: &str = include_str!("../crates/aihack-content/src/data/levels/main_1.toml");
const LEVEL_2_TOML: &str = include_str!("../crates/aihack-content/src/data/levels/main_2.toml");

fn inventory_item(session: &GameSession, kind: ItemKind) -> aihack::core::EntityId {
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

fn world_item(session: &GameSession, kind: ItemKind) -> aihack::core::EntityId {
    session
        .world()
        .entities()
        .entities()
        .iter()
        .find(|entity| entity.kind() == EntityKind::Item(kind))
        .map(|entity| entity.id)
        .expect("fixture world must contain the requested item")
}

#[test]
fn monster_speed_content_changes_actual_turn_movement() {
    let normal_registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        ITEMS_TOML,
        MONSTERS_TOML,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let stopped_monsters = MONSTERS_TOML.replacen("speed=12", "speed=0", 1);
    let stopped_registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        ITEMS_TOML,
        &stopped_monsters,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let mut normal =
        GameSession::try_new_for_playing_with_registry(1234, &normal_registry).unwrap();
    let mut stopped =
        GameSession::try_new_for_playing_with_registry(1234, &stopped_registry).unwrap();
    for session in [&mut normal, &mut stopped] {
        aihack::testing::SessionBuilder::mutate(session, |world| {
            world.set_player_pos(aihack::core::Pos { x: 5, y: 10 });
            world.saved().entities.set_alive(EntityId(3), false);
        });
    }
    let normal_projection_before = CausalProjection::from_session(&normal);
    let stopped_projection_before = CausalProjection::from_session(&stopped);
    let normal_before = normal
        .world()
        .entities()
        .actor_location(EntityId(2))
        .unwrap();
    let stopped_before = stopped
        .world()
        .entities()
        .actor_location(EntityId(2))
        .unwrap();

    assert!(normal.submit(CommandIntent::Wait).accepted);
    assert!(stopped.submit(CommandIntent::Wait).accepted);
    let normal_projection_after = CausalProjection::from_session(&normal);
    let stopped_projection_after = CausalProjection::from_session(&stopped);

    let normal_after = normal
        .world()
        .entities()
        .actor_location(EntityId(2))
        .unwrap();
    let stopped_after = stopped
        .world()
        .entities()
        .actor_location(EntityId(2))
        .unwrap();
    assert_ne!(normal_after, normal_before);
    assert_eq!(stopped_after, stopped_before);

    let mut summary = CausalSummary::default();
    summary.observe_monster_speed_pair(
        &normal_projection_before,
        &normal_projection_after,
        &stopped_projection_before,
        &stopped_projection_after,
        EntityId(2),
    );
    assert_eq!(summary.count(CausalWitness::MonsterSpeed), 1);
    assert_eq!(summary.count(CausalWitness::MonsterAi), 0);
    let record = summary.records().last().unwrap();
    assert_eq!(record.scenario, CausalScenario::MonsterSpeedPair);
    assert_eq!(record.field, CausalField::MonsterSpeed);
    assert_eq!(record.producer, Some(EntityId(2)));
}

#[test]
fn monster_ai_content_changes_actual_turn_intent() {
    let moving_registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        ITEMS_TOML,
        MONSTERS_TOML,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let stationary_monsters = MONSTERS_TOML.replacen("ai=\"wander\"", "ai=\"stationary\"", 1);
    let stationary_registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        ITEMS_TOML,
        &stationary_monsters,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let mut moving = GameSession::try_new_for_playing_with_registry(42, &moving_registry).unwrap();
    let mut stationary =
        GameSession::try_new_for_playing_with_registry(42, &stationary_registry).unwrap();
    for session in [&mut moving, &mut stationary] {
        aihack::testing::SessionBuilder::mutate(session, |world| {
            world.set_player_pos(aihack::core::Pos { x: 5, y: 10 });
            world.saved().entities.set_alive(EntityId(3), false);
        });
    }
    let moving_projection_before = CausalProjection::from_session(&moving);
    let stationary_projection_before = CausalProjection::from_session(&stationary);
    let moving_before = moving
        .world()
        .entities()
        .actor_location(EntityId(2))
        .unwrap();
    let stationary_before = stationary
        .world()
        .entities()
        .actor_location(EntityId(2))
        .unwrap();

    assert!(moving.submit(CommandIntent::Wait).accepted);
    assert!(stationary.submit(CommandIntent::Wait).accepted);
    let moving_projection_after = CausalProjection::from_session(&moving);
    let stationary_projection_after = CausalProjection::from_session(&stationary);

    assert_ne!(
        moving
            .world()
            .entities()
            .actor_location(EntityId(2))
            .unwrap(),
        moving_before
    );
    assert_eq!(
        stationary
            .world()
            .entities()
            .actor_location(EntityId(2))
            .unwrap(),
        stationary_before
    );

    let mut summary = CausalSummary::default();
    summary.observe_monster_ai_pair(
        &moving_projection_before,
        &moving_projection_after,
        &stationary_projection_before,
        &stationary_projection_after,
        EntityId(2),
    );
    assert_eq!(summary.count(CausalWitness::MonsterAi), 1);
    assert_eq!(summary.count(CausalWitness::MonsterSpeed), 0);
    let record = summary.records().last().unwrap();
    assert_eq!(record.scenario, CausalScenario::MonsterAiPair);
    assert_eq!(record.field, CausalField::MonsterAi);
    assert_eq!(record.producer, Some(EntityId(2)));
}

#[test]
fn monster_passive_content_changes_player_status() {
    let plain_registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        ITEMS_TOML,
        MONSTERS_TOML,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let passive_monsters = MONSTERS_TOML.replacen(
        "difficulty=1",
        "difficulty=1\npassive=\"paralyze_on_melee\"",
        1,
    );
    let passive_registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        ITEMS_TOML,
        &passive_monsters,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let mut plain = GameSession::try_new_for_playing_with_registry(7, &plain_registry).unwrap();
    let mut passive = GameSession::try_new_for_playing_with_registry(7, &passive_registry).unwrap();
    for session in [&mut plain, &mut passive] {
        aihack::testing::SessionBuilder::mutate(session, |world| {
            world.saved().entities.set_alive(EntityId(3), false);
            let jackal = world
                .saved()
                .entities
                .actor_stats_mut(EntityId(2))
                .expect("jackal stats must exist");
            jackal.hp = 100;
            jackal.max_hp = 100;
        });
    }

    assert!(plain.submit(CommandIntent::Move(Direction::East)).accepted);
    assert!(
        passive
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );

    assert_eq!(plain.snapshot().paralysis_turns, 0);
    assert_eq!(passive.snapshot().paralysis_turns, 1);
}

#[test]
fn item_base_price_changes_actual_game_over_score() {
    let normal_registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        ITEMS_TOML,
        MONSTERS_TOML,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let expensive_items = ITEMS_TOML.replacen("base_price=4", "base_price=404", 1);
    let expensive_registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        &expensive_items,
        MONSTERS_TOML,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let mut normal = GameSession::try_new_for_playing_with_registry(42, &normal_registry).unwrap();
    let mut expensive =
        GameSession::try_new_for_playing_with_registry(42, &expensive_registry).unwrap();

    assert!(normal.submit(CommandIntent::Quit).accepted);
    assert!(expensive.submit(CommandIntent::Quit).accepted);

    let score = |session: &GameSession| match session.run_state() {
        RunState::GameOver { final_score, .. } => final_score,
        state => panic!("expected game over, got {state:?}"),
    };
    assert_eq!(score(&expensive) - score(&normal), 400);
}

#[test]
fn prayer_created_luck_changes_the_next_attack_roll() {
    let mut plain = GameSession::new_for_playing(1234);
    let mut lucky = GameSession::new_for_playing(1234);
    for session in [&mut plain, &mut lucky] {
        aihack::testing::SessionBuilder::mutate(session, |world| {
            world.saved().entities.set_alive(EntityId(3), false);
            let jackal = world
                .saved()
                .entities
                .actor_stats_mut(EntityId(2))
                .expect("jackal stats must exist");
            jackal.hp = 100;
            jackal.max_hp = 100;
            jackal.ai_kind = Some(MonsterAiKind::Stationary);
        });
    }

    assert!(plain.submit(CommandIntent::Wait).accepted);
    assert!(lucky.submit(CommandIntent::Pray).accepted);
    assert_eq!(plain.snapshot().luck, 0);
    assert_eq!(lucky.snapshot().luck, 1);

    let plain_attack = plain.submit(CommandIntent::Move(Direction::East));
    let lucky_attack = lucky.submit(CommandIntent::Move(Direction::East));
    let attack_roll = |events: &[GameEvent]| {
        events
            .iter()
            .find_map(|event| match event {
                GameEvent::AttackResolved { attack_roll, .. } => Some(*attack_roll),
                _ => None,
            })
            .expect("bump attack must produce an attack resolution")
    };
    assert_eq!(
        attack_roll(&lucky_attack.events),
        attack_roll(&plain_attack.events) + 1
    );
}

#[test]
fn actual_trap_death_uses_content_value_in_game_over_score() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters();
        world.set_player_pos(aihack::core::Pos { x: 15, y: 5 });
        let player_id = world.saved().player_id;
        world
            .saved()
            .entities
            .actor_stats_mut(player_id)
            .expect("player stats must exist")
            .hp = 3;
    });

    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(outcome.accepted);
    match session.run_state() {
        RunState::GameOver { final_score, .. } => assert_eq!(final_score, 385),
        state => panic!("expected trap death, got {state:?}"),
    }
}

#[test]
fn armor_content_bonus_changes_player_defense_state() {
    let mut session = GameSession::new_for_playing(7);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters();
    });
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let armor = inventory_item(&session, ItemKind::ArmorLeather);
    let content_bonus = item_data(ItemKind::ArmorLeather).ac_bonus;
    let before_ac = session
        .world()
        .entities()
        .actor_stats(session.world().player_id())
        .expect("player stats must exist")
        .ac;

    let outcome = session.submit(CommandIntent::Wear { item: armor });
    let after_ac = session
        .world()
        .entities()
        .actor_stats(session.world().player_id())
        .expect("player stats must exist")
        .ac;

    assert!(outcome.accepted);
    assert!(content_bonus > 0);
    assert_eq!(after_ac, before_ac - content_bonus);
    assert_eq!(session.world().inventory().equipped_body, Some(armor));
}

#[test]
fn armor_drop_restores_ac_and_rewear_does_not_stack_across_save_load() {
    let mut session = GameSession::new_for_playing(7);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters();
    });
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let armor = inventory_item(&session, ItemKind::ArmorLeather);
    let base_ac = session
        .world()
        .entities()
        .actor_stats(session.world().player_id())
        .unwrap()
        .ac;
    let bonus = session
        .world()
        .entities()
        .item_data(armor)
        .unwrap()
        .ac_bonus;

    assert!(session.submit(CommandIntent::Wear { item: armor }).accepted);
    assert_eq!(
        session
            .world()
            .entities()
            .actor_stats(session.world().player_id())
            .unwrap()
            .ac,
        base_ac - bonus
    );
    assert!(session.submit(CommandIntent::Drop { item: armor }).accepted);
    assert_eq!(session.world().inventory().equipped_body, None);
    assert_eq!(
        session
            .world()
            .entities()
            .actor_stats(session.world().player_id())
            .unwrap()
            .ac,
        base_ac
    );

    assert!(session.submit(CommandIntent::Pickup).accepted);
    assert!(session.submit(CommandIntent::Wear { item: armor }).accepted);
    assert_eq!(
        session
            .world()
            .entities()
            .actor_stats(session.world().player_id())
            .unwrap()
            .ac,
        base_ac - bonus
    );

    let loaded = GameSession::from_save_data(session.to_save_data()).unwrap();
    assert_eq!(loaded.world().inventory().equipped_body, Some(armor));
    assert_eq!(
        loaded
            .world()
            .entities()
            .actor_stats(loaded.world().player_id())
            .unwrap()
            .ac,
        base_ac - bonus
    );
}

#[test]
fn injected_registry_survives_save_restore_and_drives_runtime_created_corpse() {
    let custom_items = ITEMS_TOML.replacen("nutrition=50", "nutrition=500", 1);
    let registry = aihack::data::ContentRegistry::from_toml_sources(
        1,
        &custom_items,
        MONSTERS_TOML,
        &[("main_1.toml", LEVEL_1_TOML), ("main_2.toml", LEVEL_2_TOML)],
    )
    .unwrap();
    let session = GameSession::try_new_for_playing_with_registry(42, &registry).unwrap();
    let mut save = session.to_save_data();
    save.world.nutrition = 100;
    save.world.entities.set_alive(EntityId(3), false);
    let jackal = save
        .world
        .entities
        .actor_stats_mut(EntityId(2))
        .expect("jackal stats must exist");
    jackal.hp = 1;
    jackal.ai_kind = Some(MonsterAiKind::Stationary);
    save.world.inventory.equipped_melee = Some(EntityId(5));
    let mut restored = GameSession::from_save_data_with_registry(save, &registry).unwrap();

    for _ in 0..20 {
        if !restored
            .world()
            .entities()
            .get(EntityId(2))
            .unwrap()
            .is_alive_actor()
        {
            break;
        }
        assert!(
            restored
                .submit(CommandIntent::Move(Direction::East))
                .accepted
        );
    }
    let corpse = world_item(&restored, ItemKind::CorpseJackal);
    assert_eq!(
        restored
            .world()
            .entities()
            .item_data(corpse)
            .unwrap()
            .nutrition,
        Some(500)
    );

    assert!(
        restored
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    assert!(restored.submit(CommandIntent::Pickup).accepted);
    let before = restored.snapshot().nutrition;
    assert!(
        restored
            .submit(CommandIntent::Eat { item: corpse })
            .accepted
    );
    assert_eq!(restored.snapshot().nutrition, before + 499);
}

#[test]
fn jackal_death_creates_an_edible_corpse_that_changes_hunger() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().nutrition = 120;
        world.saved().entities.set_alive(EntityId(3), false);
        world
            .saved()
            .entities
            .actor_stats_mut(EntityId(2))
            .expect("jackal stats must exist")
            .hp = 1;
        world.saved().inventory.equipped_melee = Some(EntityId(5));
    });

    for _ in 0..20 {
        if !session
            .world()
            .entities()
            .get(EntityId(2))
            .expect("jackal must exist")
            .is_alive_actor()
        {
            break;
        }
        assert!(
            session
                .submit(CommandIntent::Move(Direction::East))
                .accepted
        );
    }
    assert!(!session
        .world()
        .entities()
        .get(EntityId(2))
        .expect("jackal tombstone must remain")
        .is_alive_actor());
    assert_eq!(session.world().kill_count(), 1);

    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let corpse = inventory_item(&session, ItemKind::CorpseJackal);
    let before_eat = session.snapshot();
    assert_eq!(session.world().hunger_state(), HungerState::Hungry);

    let eaten = session.submit(CommandIntent::Eat { item: corpse });
    let after_eat = session.snapshot();

    assert!(eaten.accepted);
    assert_eq!(after_eat.nutrition, before_eat.nutrition + 49);
    assert_eq!(session.world().hunger_state(), HungerState::NotHungry);
    assert_eq!(
        session.world().entities().item_location(corpse),
        Some(EntityLocation::Consumed)
    );
    assert_ne!(before_eat.stable_hash(), after_eat.stable_hash());
}

#[test]
fn eating_food_changes_nutrition_hunger_and_item_lifecycle() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().nutrition = 100;
        world.saved().entities.clear_monsters();
    });
    let food = inventory_item(&session, ItemKind::FoodRation);
    let before = session.snapshot();

    let outcome = session.submit(CommandIntent::Eat { item: food });
    let after = session.snapshot();

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert_eq!(before.nutrition, 100);
    assert_eq!(after.nutrition, 899);
    assert_eq!(session.world().hunger_state(), HungerState::NotHungry);
    assert_eq!(
        session.world().entities().item_location(food),
        Some(EntityLocation::Consumed)
    );
    assert!(!session.world().inventory().contains(food));
    assert_ne!(before.stable_hash(), after.stable_hash());
}
