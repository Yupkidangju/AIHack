use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameSession, GameSnapshot, Pos},
    domain::{inventory::InventoryLetter, item::EquipmentSlot},
};

#[test]
fn starting_inventory_letters_are_stable() {
    let session = GameSession::new(42);

    assert_eq!(session.world.inventory.owner, session.world.player_id);
    assert_eq!(session.world.inventory.equipped_melee, None);
    assert_eq!(session.world.inventory.next_letter_index, 2);
    assert_eq!(session.world.inventory.entries[0].item, EntityId(5));
    assert_eq!(
        session.world.inventory.entries[0].letter,
        InventoryLetter('a')
    );
    assert_eq!(session.world.inventory.entries[1].item, EntityId(6));
    assert_eq!(
        session.world.inventory.entries[1].letter,
        InventoryLetter('b')
    );
    assert_eq!(
        session.world.entities.item_letter(EntityId(5)),
        Some(InventoryLetter('a'))
    );
    assert_eq!(
        session.world.entities.item_letter(EntityId(6)),
        Some(InventoryLetter('b'))
    );
}

#[test]
fn observation_exposes_inventory_and_legal_item_actions() {
    let session = GameSession::new(42);
    let observation = session.observation();

    assert!(observation.inventory.iter().any(|item| {
        item.item == EntityId(5)
            && item.letter == InventoryLetter('a')
            && item.equipped_slot.is_none()
    }));
    assert!(observation
        .legal_actions
        .contains(&CommandIntent::ShowInventory));
    assert!(observation
        .legal_actions
        .contains(&CommandIntent::Wield { item: EntityId(5) }));
    assert!(!observation
        .legal_actions
        .contains(&CommandIntent::Quaff { item: EntityId(6) }));
}

#[test]
fn wield_dagger_sets_melee_slot_and_second_wield_is_idempotent() {
    let mut session = GameSession::new(42);

    let first = session.submit(CommandIntent::Wield { item: EntityId(5) });

    assert!(first.accepted);
    assert!(first.turn_advanced);
    assert_eq!(session.world.inventory.equipped_melee, Some(EntityId(5)));
    assert!(matches!(
        first.events.as_slice(),
        [
            GameEvent::TurnStarted { .. },
            GameEvent::ItemEquipped {
                item: EntityId(5),
                slot: EquipmentSlot::Melee,
                ..
            }
        ]
    ));

    let before_hash = session.snapshot().stable_hash();
    let second = session.submit(CommandIntent::Wield { item: EntityId(5) });
    assert!(second.accepted);
    assert!(!second.turn_advanced);
    assert!(second.events.is_empty());
    assert_eq!(second.snapshot_hash, before_hash);
}

#[test]
fn wield_non_weapon_is_rejected() {
    let mut session = GameSession::new(42);
    let outcome = session.submit(CommandIntent::Wield { item: EntityId(6) });

    assert!(!outcome.accepted);
    assert!(!outcome.turn_advanced);
    assert_eq!(session.world.inventory.equipped_melee, None);
}

#[test]
fn equipped_dagger_drives_player_attack_profile() {
    let mut session = GameSession::new(42);
    assert!(
        session
            .submit(CommandIntent::Wield { item: EntityId(5) })
            .accepted
    );

    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(outcome.accepted);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved { attack_roll, defense: 10, .. } if *attack_roll >= 4
    )));
}

#[test]
fn inventory_state_affects_snapshot_hash() {
    let mut session = GameSession::new(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 8, y: 5 });
    let before = session.snapshot().stable_hash();
    assert!(session.submit(CommandIntent::Pickup).accepted);
    let after_pickup = session.snapshot().stable_hash();
    assert_ne!(before, after_pickup);
    assert!(
        session
            .submit(CommandIntent::Wield { item: EntityId(5) })
            .accepted
    );
    let after_wield = session.snapshot().stable_hash();
    assert_ne!(after_pickup, after_wield);
}

#[test]
fn inventory_roundtrip_preserves_letters() {
    let mut session = GameSession::new(42);
    session.world.entities.clear_monsters();
    session.world.set_player_pos(Pos { x: 8, y: 5 });
    assert!(session.submit(CommandIntent::Pickup).accepted);
    assert!(
        session
            .submit(CommandIntent::Wield { item: EntityId(5) })
            .accepted
    );
    let snapshot = session.snapshot();

    let json = serde_json::to_string(&snapshot).unwrap();
    let decoded: GameSnapshot = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.inventory.entries, snapshot.inventory.entries);
    assert_eq!(
        decoded.inventory.equipped_melee,
        snapshot.inventory.equipped_melee
    );
    assert_eq!(decoded.entities, snapshot.entities);
}
