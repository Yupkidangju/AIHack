use aihack::core::session::GameSession;

/// load_items가 비어있지 않은 아이템 목록을 반환하는 콘텐츠 회귀다.
#[test]
fn load_items_returns_non_empty_list() {
    let items = aihack::data::load_items().expect("embedded content must validate");
    assert!(!items.is_empty(), "items.toml에서 아이템을 읽어와야 한다.");
}

/// load_items에서 dagger 아이템을 찾을 수 있어야 한다.
#[test]
fn load_items_includes_dagger() {
    let items = aihack::data::load_items().expect("embedded content must validate");
    let dagger = items.iter().find(|i| i.id == "item.weapon.dagger");
    assert!(
        dagger.is_some(),
        "dagger 아이템이 items.toml에 있어야 한다."
    );
    let dagger = dagger.unwrap();
    assert_eq!(dagger.glyph, ")");
    assert_eq!(dagger.weight, 10);
}

/// load_items에서 healing potion을 찾을 수 있어야 한다.
#[test]
fn load_items_includes_healing_potion() {
    let items = aihack::data::load_items().expect("embedded content must validate");
    let potion = items.iter().find(|i| i.id == "item.potion.healing");
    assert!(
        potion.is_some(),
        "healing potion이 items.toml에 있어야 한다."
    );
    let potion = potion.unwrap();
    assert_eq!(potion.effect.as_deref(), Some("heal_1d8_plus_4"));
}

/// load_monsters가 비어있지 않은 몬스터 목록을 반환해야 한다.
#[test]
fn load_monsters_returns_non_empty_list() {
    let monsters = aihack::data::load_monsters().expect("embedded content must validate");
    assert!(
        !monsters.is_empty(),
        "monsters.toml에서 몬스터를 읽어와야 한다."
    );
}

/// load_monsters에서 jackal을 찾을 수 있어야 한다.
#[test]
fn load_monsters_includes_jackal() {
    let monsters = aihack::data::load_monsters().expect("embedded content must validate");
    let jackal = monsters.iter().find(|m| m.id == "monster.jackal");
    assert!(jackal.is_some(), "jackal이 monsters.toml에 있어야 한다.");
    let jackal = jackal.unwrap();
    assert_eq!(jackal.hp, 4);
    assert_eq!(jackal.damage, "1d2");
}

/// load_level이 main:1 레벨 데이터를 반환해야 한다.
#[test]
fn load_level_main_1_returns_data() {
    let level = aihack::data::load_level("main:1").expect("main:1 must exist");
    assert_eq!(level.level_id, "main:1");
    assert_eq!(level.width, 40);
    assert_eq!(level.height, 20);
    assert_eq!(level.player_start, vec![5, 5]);
    assert_eq!(level.stairs_down, Some(vec![34, 15]));
}

/// GameWorld.status()가 현재 Status projection을 반환해야 한다.
#[test]
fn game_world_status_returns_status() {
    let session = GameSession::new_for_playing(42);
    let status = session.world().status();
    assert_eq!(status.nutrition, 900);
    assert_eq!(status.luck, 0);
    assert_eq!(status.prayer_cooldown, 0);
    assert_eq!(status.paralysis_turns, 0);
    assert!(!status.hallucinating);
}

/// Status::hunger_state가 동결된 영양 경계대로 계산되어야 한다.
#[test]
fn status_hunger_state_calculation() {
    use aihack::domain::status::{HungerState, Status};

    let fainting = Status {
        nutrition: 0,
        ..Status::default_adventurer()
    };
    assert!(matches!(fainting.hunger_state(), HungerState::Fainting));

    let hungry = Status {
        nutrition: 100,
        ..Status::default_adventurer()
    };
    assert!(matches!(hungry.hunger_state(), HungerState::Hungry));

    let satiated = Status {
        nutrition: 1001,
        ..Status::default_adventurer()
    };
    assert!(matches!(satiated.hunger_state(), HungerState::Satiated));
}

/// GameWorld.hunger_state()가 Status 진실원을 사용해야 한다.
#[test]
fn game_world_hunger_state_delegates_to_status() {
    let session = GameSession::new_for_playing(42);
    let hunger = session.world().hunger_state();
    assert!(matches!(
        hunger,
        aihack::domain::status::HungerState::NotHungry
    ));
}

#[test]
fn game_world_score_accessors_replace_direct_score_mutation() {
    let mut session = GameSession::new_for_playing(42);

    aihack::testing::SessionBuilder::mutate(&mut session, |world| {
        world.set_gold(123);
        world.set_kill_count(4);
    });

    assert_eq!(session.world().gold(), 123);
    assert_eq!(session.world().kill_count(), 4);
}

#[test]
fn game_world_exposes_player_identity_without_a_public_field() {
    let session = GameSession::new_for_playing(42);

    assert_eq!(session.world().player_id(), aihack::core::EntityId(1));
}
