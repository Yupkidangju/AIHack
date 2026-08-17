use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameSession, RunState},
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
            world
                .saved()
                .entities
                .actor_stats_mut(EntityId(2))
                .expect("jackal stats must exist")
                .hp = 100;
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
