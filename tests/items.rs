use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameSession, Pos},
    domain::{
        entity::{EntityKind, EntityLocation},
        inventory::InventoryLetter,
        item::{item_data, ConsumableEffect, ItemClass, ItemKind, WandEffect},
    },
};

fn stand_on_potion(session: &mut GameSession) {
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 8, y: 5 });
}

#[test]
fn item_factories_match_spec_data() {
    let dagger = item_data(ItemKind::Dagger);
    let food = item_data(ItemKind::FoodRation);
    let potion = item_data(ItemKind::PotionHealing);
    let wand = item_data(ItemKind::WandMagicMissile);
    let scroll = item_data(ItemKind::ScrollReveal);
    let rock = item_data(ItemKind::Rock);

    assert_eq!(dagger.class, ItemClass::Weapon);
    assert_eq!(dagger.glyph, ')');
    assert_eq!(dagger.weight, 10);
    assert_eq!(dagger.attack_profile.unwrap().hit_bonus, 1);
    assert_eq!(dagger.attack_profile.unwrap().damage.dice, 1);
    assert_eq!(dagger.attack_profile.unwrap().damage.sides, 4);

    assert_eq!(food.class, ItemClass::Food);
    assert_eq!(food.glyph, '%');
    assert_eq!(food.weight, 20);
    assert_eq!(food.nutrition, Some(800));

    assert_eq!(potion.class, ItemClass::Potion);
    assert_eq!(potion.glyph, '!');
    assert_eq!(potion.weight, 20);
    assert_eq!(
        potion.consumable_effect,
        Some(ConsumableEffect::Heal {
            dice: 1,
            sides: 8,
            bonus: 4,
        })
    );

    assert_eq!(wand.class, ItemClass::Wand);
    assert_eq!(wand.wand_effect, Some(WandEffect::MagicMissile));
    assert_eq!(wand.max_charges, Some(3));

    assert_eq!(scroll.class, ItemClass::Scroll);
    assert_eq!(
        scroll.consumable_effect,
        Some(ConsumableEffect::RevealLevel)
    );

    assert_eq!(rock.class, ItemClass::Rock);
    assert_eq!(rock.attack_profile.unwrap().damage.sides, 3);
}

#[test]
fn item_entities_have_expected_ids_and_locations() {
    let session = GameSession::new_for_playing(42);

    assert!(matches!(
        session.world.entities.get(EntityId(4)).unwrap().kind(),
        EntityKind::Item(ItemKind::PotionHealing)
    ));
    assert_eq!(
        session.world.entities.item_location(EntityId(4)),
        Some(EntityLocation::OnMap {
            level: session.world.current_level(),
            pos: Pos { x: 8, y: 5 },
        })
    );
    assert_eq!(
        session.world.entities.item_location(EntityId(5)),
        Some(EntityLocation::Inventory {
            owner: session.world.player_id
        })
    );
    assert_eq!(
        session.world.entities.item_location(EntityId(6)),
        Some(EntityLocation::Inventory {
            owner: session.world.player_id
        })
    );
    assert_eq!(session.world.entities.item_charges(EntityId(7)), Some(3));
    assert_eq!(
        session.world.entities.item_location(EntityId(8)),
        Some(EntityLocation::Inventory {
            owner: session.world.player_id
        })
    );
    assert_eq!(
        session.world.entities.item_location(EntityId(9)),
        Some(EntityLocation::Inventory {
            owner: session.world.player_id
        })
    );
}

#[test]
fn pickup_assigns_next_letter_and_event_order() {
    let mut session = GameSession::new_for_playing(42);
    stand_on_potion(&mut session);

    let outcome = session.submit(CommandIntent::Pickup);

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert!(matches!(
        outcome.events.first(),
        Some(GameEvent::TurnStarted { .. })
    ));
    assert!(matches!(
        outcome.events.get(1),
        Some(GameEvent::ItemPickedUp {
            entity,
            item: EntityId(4),
            letter: InventoryLetter('f')
        }) if *entity == session.world.player_id
    ));
    assert_eq!(
        session.world.inventory.letter_for(EntityId(4)),
        Some(InventoryLetter('f'))
    );
    assert_eq!(session.world.inventory.next_letter_index, 6);
    assert_eq!(
        session.world.entities.item_location(EntityId(4)),
        Some(EntityLocation::Inventory {
            owner: session.world.player_id
        })
    );
}

#[test]
fn pickup_without_item_is_rejected_without_turn() {
    let mut session = GameSession::new_for_playing(42);
    let before_hash = session.snapshot().stable_hash();

    let outcome = session.submit(CommandIntent::Pickup);

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert_eq!(session.turn, 0);
    assert_eq!(outcome.snapshot_hash, before_hash);
    assert!(matches!(
        outcome.events.as_slice(),
        [GameEvent::CommandRejected { reason }] if reason.contains("no item")
    ));
}

#[test]
fn show_inventory_is_no_turn_and_eventless() {
    let mut session = GameSession::new_for_playing(42);
    let before_hash = session.snapshot().stable_hash();
    let before_hp = session
        .world
        .entities
        .actor_stats(session.world.player_id)
        .expect("player stats must exist")
        .hp;

    let outcome = session.submit(CommandIntent::ShowInventory);

    assert!(outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert!(outcome.events.is_empty());
    assert_eq!(outcome.snapshot_hash, before_hash);
    assert_eq!(
        session
            .world
            .entities
            .actor_stats(session.world.player_id)
            .expect("player stats must exist")
            .hp,
        before_hp
    );
}

#[test]
fn quaff_healing_potion_heals_and_consumes_repeatably() {
    fn run() -> (i16, i16) {
        let mut session = GameSession::new_for_playing(42);
        stand_on_potion(&mut session);
        assert!(session.submit(CommandIntent::Pickup).accepted);
        session
            .world
            .entities
            .actor_stats_mut(session.world.player_id)
            .unwrap()
            .hp = 5;
        let outcome = session.submit(CommandIntent::Quaff { item: EntityId(4) });
        assert!(outcome.accepted);
        assert!(outcome.turn_advanced);
        assert!(matches!(
            outcome.events.first(),
            Some(GameEvent::TurnStarted { .. })
        ));
        assert!(matches!(
            outcome.events.get(1),
            Some(GameEvent::ItemConsumed {
                item: EntityId(4),
                ..
            })
        ));
        let (amount, hp_after) = outcome
            .events
            .iter()
            .find_map(|event| match event {
                GameEvent::EntityHealed {
                    amount, hp_after, ..
                } => Some((*amount, *hp_after)),
                _ => None,
            })
            .unwrap();
        assert!((5..=12).contains(&amount));
        assert_eq!(
            session.world.entities.item_location(EntityId(4)),
            Some(EntityLocation::Consumed)
        );
        assert_eq!(
            session.world.entities.item_letter(EntityId(4)),
            Some(InventoryLetter('f'))
        );
        assert!(!session.world.inventory.contains(EntityId(4)));
        (amount, hp_after)
    }

    assert_eq!(run(), run());
}

#[test]
fn healing_clamps_to_max_hp_with_effective_amount() {
    let mut session = GameSession::new_for_playing(42);
    stand_on_potion(&mut session);
    assert!(session.submit(CommandIntent::Pickup).accepted);
    session
        .world
        .entities
        .actor_stats_mut(session.world.player_id)
        .unwrap()
        .hp = 15;

    let outcome = session.submit(CommandIntent::Quaff { item: EntityId(4) });

    let (amount, hp_after) = outcome
        .events
        .iter()
        .find_map(|event| match event {
            GameEvent::EntityHealed {
                amount, hp_after, ..
            } => Some((*amount, *hp_after)),
            _ => None,
        })
        .unwrap();
    assert_eq!(amount, 1);
    assert_eq!(hp_after, 16);
}

#[test]
fn invalid_quaffs_are_rejected_without_turn() {
    let mut session = GameSession::new_for_playing(42);
    let before_hash = session.snapshot().stable_hash();

    let food = session.submit(CommandIntent::Quaff { item: EntityId(6) });
    assert!(!food.accepted);
    assert!(!food.turn_advanced);

    let map_potion = session.submit(CommandIntent::Quaff { item: EntityId(4) });
    assert!(!map_potion.accepted);
    assert!(!map_potion.turn_advanced);
    assert_eq!(map_potion.snapshot_hash, before_hash);
}

#[test]
fn consumed_potion_is_not_legal_action() {
    let mut session = GameSession::new_for_playing(42);
    stand_on_potion(&mut session);
    assert!(session.submit(CommandIntent::Pickup).accepted);
    assert!(session
        .observation()
        .legal_actions
        .contains(&CommandIntent::Quaff { item: EntityId(4) }));
    assert!(
        session
            .submit(CommandIntent::Quaff { item: EntityId(4) })
            .accepted
    );

    assert!(!session
        .observation()
        .legal_actions
        .contains(&CommandIntent::Quaff { item: EntityId(4) }));
    assert!(!session
        .observation()
        .inventory
        .iter()
        .any(|item| item.item == EntityId(4)));
}

#[test]
fn read_scroll_reveals_hidden_tiles_and_consumes_scroll() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();

    let outcome = session.submit(CommandIntent::Read { item: EntityId(8) });

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::ScrollRead {
            item: EntityId(8),
            ..
        }
    )));
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::TileRevealed {
            pos: Pos { x: 12, y: 5 },
            ..
        }
    )));
    assert_eq!(
        session.world.entities.item_location(EntityId(8)),
        Some(EntityLocation::Consumed)
    );
}

#[test]
fn wand_zap_spends_charge() {
    let mut session = GameSession::new_for_playing(42);
    session.world.entities.clear_monsters();

    let outcome = session.submit(CommandIntent::Zap {
        item: EntityId(7),
        direction: Direction::East,
    });

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::WandZapped {
            item: EntityId(7),
            charges_after: 2,
            ..
        }
    )));
    assert_eq!(session.world.entities.item_charges(EntityId(7)), Some(2));
}
