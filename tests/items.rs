use aihack::{
    core::{CommandIntent, EntityId, GameEvent, GameSession, Pos},
    domain::{
        entity::{EntityKind, EntityLocation},
        inventory::InventoryLetter,
        item::{item_data, ConsumableEffect, ItemClass, ItemKind},
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
}

#[test]
fn item_entities_have_expected_ids_and_locations() {
    let session = GameSession::new(42);

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
}

#[test]
fn pickup_assigns_next_letter_and_event_order() {
    let mut session = GameSession::new(42);
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
            letter: InventoryLetter('c')
        }) if *entity == session.world.player_id
    ));
    assert_eq!(
        session.world.inventory.letter_for(EntityId(4)),
        Some(InventoryLetter('c'))
    );
    assert_eq!(session.world.inventory.next_letter_index, 3);
    assert_eq!(
        session.world.entities.item_location(EntityId(4)),
        Some(EntityLocation::Inventory {
            owner: session.world.player_id
        })
    );
}

#[test]
fn pickup_without_item_is_rejected_without_turn() {
    let mut session = GameSession::new(42);
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
    let mut session = GameSession::new(42);
    let before_hash = session.snapshot().stable_hash();

    let outcome = session.submit(CommandIntent::ShowInventory);

    assert!(outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert!(outcome.events.is_empty());
    assert_eq!(outcome.snapshot_hash, before_hash);
}

#[test]
fn quaff_healing_potion_heals_and_consumes_repeatably() {
    fn run() -> (i16, i16) {
        let mut session = GameSession::new(42);
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
            Some(InventoryLetter('c'))
        );
        assert!(!session.world.inventory.contains(EntityId(4)));
        (amount, hp_after)
    }

    assert_eq!(run(), run());
}

#[test]
fn healing_clamps_to_max_hp_with_effective_amount() {
    let mut session = GameSession::new(42);
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
    let mut session = GameSession::new(42);
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
    let mut session = GameSession::new(42);
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
