use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameRng, GameSession, Pos},
    domain::{
        item::{shop_base_price, ItemKind},
        monster::MonsterKind,
    },
    systems::score::{apply_luck, death_score, hallucination_message},
};

#[test]
fn p8_g01_same_seed_same_dice_sequence() {
    let mut a = GameRng::new(42);
    let mut b = GameRng::new(42);
    assert_eq!(a.next_u64(), b.next_u64());
    assert_eq!(a.next_u64(), b.next_u64());
}

#[test]
fn p8_g02_monster_difficulty_snapshot() {
    assert_eq!(MonsterKind::Jackal.difficulty(), 1);
    assert_eq!(MonsterKind::Goblin.difficulty(), 2);
    assert_eq!(MonsterKind::FloatingEye.difficulty(), 5);
}

#[test]
fn p8_g03_inventory_letter_policy_not_reused_after_drop() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters()
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 8, y: 5 })
    });
    assert!(
        session
            .submit(CommandIntent::Drop { item: EntityId(5) })
            .accepted
    );
    assert!(
        session
            .submit(CommandIntent::Move(Direction::West))
            .accepted
    );
    assert!(
        session
            .submit(CommandIntent::Move(Direction::East))
            .accepted
    );
    let outcome = session.submit(CommandIntent::Pickup);
    assert!(outcome.accepted);
    assert!(matches!(
        outcome.events.iter().find(|event| matches!(event, GameEvent::ItemPickedUp { .. })),
        Some(GameEvent::ItemPickedUp { letter, .. }) if *letter == aihack::domain::inventory::InventoryLetter('f')
    ));
}

#[test]
fn p8_g04_potion_healing_rule_fidelity() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters()
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 8, y: 5 })
    });
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let player = session.world().player_id();
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.actor_stats_mut(player).unwrap().hp = 5;
    });
    let outcome = session.submit(CommandIntent::Quaff { item: EntityId(4) });
    assert!(outcome.accepted);
    assert!(outcome
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::EntityHealed { hp_after, .. } if *hp_after > 5)));
}

#[test]
fn p8_g05_weapon_damage_profile_fidelity() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.set_alive(EntityId(3), false)
    });
    assert!(
        session
            .submit(CommandIntent::Wield { item: EntityId(5) })
            .accepted
    );
    let outcome = session.submit(CommandIntent::Move(Direction::East));
    assert!(outcome
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::AttackResolved { damage, .. } if *damage <= 4)));
}

#[test]
fn p8_g06_trap_detection_via_search() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters()
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 15, y: 5 })
    });
    let outcome = session.submit(CommandIntent::Search);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::TileRevealed {
            pos: Pos { x: 16, y: 5 },
            ..
        }
    )));
}

#[test]
fn p8_g07_door_kicking_opens_hidden_or_closed_door() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters()
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 11, y: 5 })
    });
    let outcome = session.submit(CommandIntent::Kick(Direction::East));
    assert!(outcome.accepted);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::DoorKicked {
            pos: Pos { x: 12, y: 5 }
        }
    )));
}

#[test]
fn p8_g08_wand_beam_semantics() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.set_alive(EntityId(2), false)
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 17, y: 12 })
    });
    let outcome = session.submit(CommandIntent::Zap {
        item: EntityId(7),
        direction: Direction::East,
    });
    assert!(outcome
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::WandZapped { .. })));
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved {
            defender: EntityId(3),
            ..
        }
    )));
}

#[test]
fn p8_g09_scroll_identify_marks_item_identified() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters()
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 9, y: 5 })
    });
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let outcome = session.submit(CommandIntent::Read { item: EntityId(11) });
    assert!(outcome.accepted);
    assert!(session.world().is_item_identified(ItemKind::Dagger));
    assert!(outcome
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::ItemIdentified { .. })));
}

#[test]
fn p8_g10_level_teleport_relocates_between_levels() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters()
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 11, y: 5 })
    });
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let outcome = session.submit(CommandIntent::Read { item: EntityId(12) });
    assert!(outcome.accepted);
    assert!(outcome
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::LevelChanged { .. })));
    assert_eq!(
        session.world().current_level(),
        aihack::core::LevelId::main(2)
    );
}

#[test]
fn p8_g11_passive_attack_response() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.set_alive(EntityId(2), false)
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.set_alive(EntityId(3), false)
    });
    let eye = aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world
            .saved()
            .entities
            .spawn_monster(MonsterKind::FloatingEye, Pos { x: 6, y: 5 })
            .unwrap()
    });
    let outcome = session.submit(CommandIntent::Move(Direction::East));
    assert!(outcome.events.iter().any(
        |event| matches!(event, GameEvent::PassiveAttackTriggered { source, .. } if *source == eye)
    ));
    assert_eq!(session.world().status().paralysis_turns, 1);
}

#[test]
fn p8_g12_armor_ac_mitigation() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters()
    });
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_player_pos(Pos { x: 7, y: 5 })
    });
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let before = session
        .world()
        .entities()
        .actor_stats(session.world().player_id())
        .unwrap()
        .ac;
    assert!(
        session
            .submit(CommandIntent::Wear { item: EntityId(10) })
            .accepted
    );
    let after = session
        .world()
        .entities()
        .actor_stats(session.world().player_id())
        .unwrap()
        .ac;
    assert!(after < before);
}

#[test]
fn p8_g13_corpse_drop_on_jackal_death() {
    let mut session = GameSession::new_for_playing(42);
    for _ in 0..50 {
        let outcome = session.submit(CommandIntent::Move(Direction::East));
        if outcome.events.iter().any(|event| {
            matches!(
                event,
                GameEvent::EntityDied {
                    entity: EntityId(2),
                    ..
                }
            )
        }) {
            assert!(session
                .world()
                .entities()
                .entities()
                .iter()
                .any(|entity| matches!(
                    entity.kind(),
                    aihack::domain::entity::EntityKind::Item(ItemKind::CorpseJackal)
                )));
            return;
        }
    }
    panic!("jackal should die and drop corpse");
}

#[test]
fn p8_g14_death_score_computation() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_gold(123);
        world.set_kill_count(4);
    });
    let score = death_score(session.world(), 50);
    // 시작 인벤토리의 콘텐츠 가격 합계 285도 실제 종료 점수에 반영한다.
    assert_eq!(score, 123 + 40 + 100 + 285 - 5);
}

#[test]
fn p8_g15_luck_adjustment_effect() {
    assert_eq!(apply_luck(10, 2), 12);
    assert_eq!(apply_luck(10, -1), 9);
}

#[test]
fn p8_g16_hunger_tick_over_repeated_turns() {
    let mut session = GameSession::new_for_playing(42);
    let before = session.world().status().nutrition;
    assert!(session.submit(CommandIntent::Wait).accepted);
    assert_eq!(session.world().status().nutrition, before - 1);
}

#[test]
fn p8_g17_hallucination_message_changes_without_core_corruption() {
    let plain = hallucination_message("jackal appears", false);
    let hallu = hallucination_message("jackal appears", true);
    assert_eq!(plain, "jackal appears");
    assert_ne!(plain, hallu);
}

#[test]
fn p8_g18_encumbrance_blocks_movement_when_over_threshold() {
    let mut session = GameSession::new_for_playing(42);
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.saved().entities.clear_monsters()
    });
    // 무게 한도를 넘도록 무거운 rock을 inventory relation과 함께 추가한다.
    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        let owner = world.saved().player_id;
        for _ in 0..4 {
            let item = world
                .saved()
                .entities
                .spawn_item(
                    ItemKind::Rock,
                    aihack::domain::entity::EntityLocation::Inventory { owner },
                )
                .unwrap();
            let letter = world
                .saved()
                .inventory
                .add_existing_with_next_letter(item)
                .expect("fixture rock은 inventory letter를 받아야 한다");
            assert!(world.saved().entities.set_item_letter(item, letter));
        }
    });
    let outcome = session.submit(CommandIntent::Move(Direction::East));
    assert!(!outcome.accepted);
}

#[test]
fn p8_g19_shop_price_base_deterministic_math() {
    assert_eq!(shop_base_price(ItemKind::Rock), 1);
    assert!(shop_base_price(ItemKind::WandMagicMissile) > shop_base_price(ItemKind::Dagger));
}

#[test]
fn p8_g20_prayer_cooldown_enforced() {
    let mut session = GameSession::new_for_playing(42);
    let first = session.submit(CommandIntent::Pray);
    assert!(first.accepted);
    let second = session.submit(CommandIntent::Pray);
    assert!(!second.accepted);
}
