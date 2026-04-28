use aihack::{
    core::{CommandIntent, Direction, EntityId, GameEvent, GameRng, GameSession, Pos},
    domain::{
        combat::DamageRoll,
        entity::{ActorStats, EntityKind, EntityStore, Faction},
        monster::{monster_template, MonsterKind},
        player::adventurer_template,
    },
    systems::combat,
};

#[test]
fn entity_store_assigns_stable_nonzero_ids() {
    let mut store = EntityStore::new();
    let player = store.spawn_player(Pos { x: 5, y: 5 });
    let jackal = store.spawn_monster(MonsterKind::Jackal, Pos { x: 6, y: 5 });

    assert_eq!(player, EntityId(1));
    assert_eq!(jackal, EntityId(2));
    assert!(store.get(EntityId(0)).is_none());
    assert!(matches!(
        store.get(player).unwrap().kind(),
        EntityKind::Player
    ));
    assert!(matches!(
        store.get(jackal).unwrap().kind(),
        EntityKind::Monster(MonsterKind::Jackal)
    ));
}

#[test]
fn tombstone_does_not_compact_entity_ids() {
    let mut store = EntityStore::new();
    let player = store.spawn_player(Pos { x: 5, y: 5 });
    let jackal = store.spawn_monster(MonsterKind::Jackal, Pos { x: 6, y: 5 });
    let goblin = store.spawn_monster(MonsterKind::Goblin, Pos { x: 20, y: 12 });

    assert_eq!(player, EntityId(1));
    assert_eq!(jackal, EntityId(2));
    assert_eq!(goblin, EntityId(3));
    assert!(store.set_alive(jackal, false));
    assert!(!store.get(jackal).unwrap().actor().unwrap().5);
    assert_eq!(store.get(goblin).unwrap().id, EntityId(3));
    let next = store.spawn_monster(MonsterKind::FloatingEye, Pos { x: 8, y: 8 });
    assert_eq!(next, EntityId(4));
}

#[test]
fn player_factory_matches_phase3_spec() {
    let player = adventurer_template();

    assert_eq!(player.hp, 16);
    assert_eq!(player.ac, 0);
    assert_eq!(player.hit_bonus, 2);
    assert_eq!(player.damage_bonus, 0);
    assert_eq!(player.attack_profile.hit_bonus, 1);
    assert_eq!(
        player.attack_profile.damage,
        DamageRoll { dice: 1, sides: 4 }
    );
}

#[test]
fn monster_factories_match_phase3_spec() {
    let jackal = monster_template(MonsterKind::Jackal);
    let goblin = monster_template(MonsterKind::Goblin);
    let floating_eye = monster_template(MonsterKind::FloatingEye);

    assert_eq!(
        (
            jackal.hp,
            jackal.ac,
            jackal.hit_bonus,
            jackal.attack_profile.damage
        ),
        (4, 0, 0, DamageRoll { dice: 1, sides: 2 })
    );
    assert_eq!(
        (
            goblin.hp,
            goblin.ac,
            goblin.hit_bonus,
            goblin.attack_profile.damage
        ),
        (6, 1, 1, DamageRoll { dice: 1, sides: 4 })
    );
    assert_eq!(
        (
            floating_eye.hp,
            floating_eye.ac,
            floating_eye.hit_bonus,
            floating_eye.attack_profile.damage
        ),
        (8, 2, 0, DamageRoll { dice: 0, sides: 0 })
    );
}

#[test]
fn hit_formula_uses_d20_bonuses_and_defense() {
    let mut store = EntityStore::new();
    let attacker = store.spawn(
        EntityKind::Player,
        Faction::Player,
        Pos { x: 1, y: 1 },
        ActorStats {
            hp: 10,
            max_hp: 10,
            ac: 0,
            hit_bonus: 2,
            damage_bonus: 0,
            damage_reduction: 0,
            damage: DamageRoll { dice: 1, sides: 4 },
            weapon_hit_bonus: 1,
        },
    );
    let defender = store.spawn(
        EntityKind::Monster(MonsterKind::Goblin),
        Faction::Hostile,
        Pos { x: 2, y: 1 },
        ActorStats {
            hp: 10,
            max_hp: 10,
            ac: 5,
            hit_bonus: 0,
            damage_bonus: 0,
            damage_reduction: 0,
            damage: DamageRoll { dice: 1, sides: 2 },
            weapon_hit_bonus: 0,
        },
    );
    let a = store.get(attacker).unwrap();
    let d = store.get(defender).unwrap();

    assert_eq!(combat::attack_roll_value(a, d, 1, 12), (15, 15, true));
    assert_eq!(combat::attack_roll_value(a, d, 1, 11), (14, 15, false));
}

#[test]
fn damage_formula_rolls_dice_and_clamps_minimum_one() {
    let mut rng = GameRng::new(42);
    let damage = combat::roll_damage(&mut rng, DamageRoll { dice: 1, sides: 4 }, 0, 0);
    assert!((1..=4).contains(&damage));

    let mut rng = GameRng::new(42);
    assert_eq!(
        combat::roll_damage(&mut rng, DamageRoll { dice: 1, sides: 2 }, 0, 99),
        1
    );
}

#[test]
fn bump_attack_keeps_player_in_place_and_emits_attack_event() {
    let mut session = GameSession::new(42);
    let before = session.world.player_pos();
    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(outcome.accepted);
    assert!(outcome.turn_advanced);
    assert_eq!(session.world.player_pos(), before);
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved { attacker, defender, .. }
            if *attacker == session.world.player_id && *defender == EntityId(2)
    )));
}

#[test]
fn attack_event_damage_shape_is_consistent() {
    let mut session = GameSession::new(42);
    let outcome = session.submit(CommandIntent::Move(Direction::East));
    let event = outcome
        .events
        .iter()
        .find_map(|event| match event {
            GameEvent::AttackResolved { hit, damage, .. } => Some((*hit, *damage)),
            _ => None,
        })
        .expect("bump attack은 AttackResolved를 남겨야 한다");

    if event.0 {
        assert!(event.1 >= 1);
    } else {
        assert_eq!(event.1, 0);
    }
}

#[test]
fn legal_actions_include_adjacent_bump_attack() {
    let session = GameSession::new(42);
    let observation = session.observation();

    assert!(observation
        .legal_actions
        .contains(&CommandIntent::Move(Direction::East)));
}

#[test]
fn goblin_can_be_attacked_by_bump_combat() {
    let mut session = GameSession::new(42);
    let goblin = EntityId(3);
    let before_hp = session.world.entities.actor_stats(goblin).unwrap().hp;
    session.world.set_player_pos(Pos { x: 19, y: 12 });

    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(outcome.accepted);
    assert_eq!(session.world.player_pos(), Pos { x: 19, y: 12 });
    assert!(outcome.events.iter().any(|event| matches!(
        event,
        GameEvent::AttackResolved { defender, .. } if *defender == goblin
    )));
    assert!(session.world.entities.actor_stats(goblin).unwrap().hp <= before_hp);
}

#[test]
fn bump_attack_event_order_is_turn_started_then_attack_then_optional_death() {
    let mut session = GameSession::new(42);
    let outcome = session.submit(CommandIntent::Move(Direction::East));

    assert!(matches!(
        outcome.events.first(),
        Some(GameEvent::TurnStarted { .. })
    ));
    assert!(matches!(
        outcome.events.get(1),
        Some(GameEvent::AttackResolved { .. })
    ));
    if outcome.events.len() > 2 {
        assert!(matches!(
            outcome.events.get(2),
            Some(GameEvent::EntityDied { .. })
        ));
    }
}
