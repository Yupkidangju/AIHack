use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use aihack_ai_contract::{ActionSpace, RunState};
use aihack_content::{
    item_id, item_kind_from_id, level_spawns, level_tile_overrides, monster_id,
    monster_kind_from_id, ContentRegistry, LevelSpawn, CONTENT_SCHEMA_VERSION,
};
use aihack_core::{
    action::{ActionIntent, CommandIntent as C},
    campaign::Role,
    domain::{
        combat::{attack_roll_value, DeathCause},
        entity::{ActorKind, EntityLocation},
        inventory::{Inventory, InventoryLetter, MAX_INVENTORY_SLOTS},
        item::{BucStatus, ItemKind, MAX_ITEM_ENCHANTMENT},
        monster::MonsterKind,
        player::adventurer_template,
        status::STARVATION_THRESHOLD,
        tile::{DoorState, TileKind, TrapKind},
    },
    event::GameEvent,
    ids::{BranchId, EntityId, LevelId},
    position::{Direction, Pos},
};
use aihack_headless::{run_to_turn, HeadlessPolicy};
use aihack_llm::{
    decision::parse_decision_payload_json, service::LlmObservationView,
    transport::REQUEST_BODY_LIMIT,
};
use aihack_runtime::{
    appearance::appearance_for_kind, domain::monster::monster_template, shop_buy_price, GameSession,
};
use aihack_tui::tui::{
    compute_layout, input, render_frame, render_panels, runtime_event_to_candidate, TuiApp,
    UiClock, UiCommandCandidate, UiRuntimeConfig, UiTheme,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};

const ALL_MONSTERS: [(MonsterKind, i16); 12] = [
    (MonsterKind::Jackal, 10),
    (MonsterKind::Goblin, 9),
    (MonsterKind::FloatingEye, 8),
    (MonsterKind::Kobold, 9),
    (MonsterKind::Orc, 8),
    (MonsterKind::Ogre, 7),
    (MonsterKind::GiantSpider, 8),
    (MonsterKind::GnollArcher, 8),
    (MonsterKind::Troll, 7),
    (MonsterKind::FireAnt, 7),
    (MonsterKind::Minotaur, 6),
    (MonsterKind::Dragon, 5),
];

const ALL_ITEMS: [ItemKind; 24] = [
    ItemKind::Dagger,
    ItemKind::ShortSword,
    ItemKind::Mace,
    ItemKind::FoodRation,
    ItemKind::PotionHealing,
    ItemKind::PotionExtraHealing,
    ItemKind::PotionCure,
    ItemKind::WandMagicMissile,
    ItemKind::WandFire,
    ItemKind::ScrollReveal,
    ItemKind::ScrollIdentify,
    ItemKind::ScrollLevelTeleport,
    ItemKind::ScrollCharging,
    ItemKind::ScrollEnchantWeapon,
    ItemKind::ScrollEnchantArmor,
    ItemKind::ScrollRemoveCurse,
    ItemKind::Rock,
    ItemKind::ArmorLeather,
    ItemKind::ArmorChainMail,
    ItemKind::RingPoisonResistance,
    ItemKind::RingFireResistance,
    ItemKind::RingProtection,
    ItemKind::CorpseJackal,
    ItemKind::AmuletAscension,
];

#[derive(Default)]
struct TestClock(AtomicU64);

impl UiClock for TestClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::Relaxed))
    }
}

fn send_key(app: &mut TuiApp, clock: &TestClock, ch: char) -> UiCommandCandidate {
    clock.0.fetch_add(1000, Ordering::Relaxed);
    app.release_transition_gesture_on_idle();
    app.release_transition_gesture_on_idle();
    let candidate = runtime_event_to_candidate(
        Event::Key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)),
        100,
        32,
        app,
    )
    .expect("TUI key must map to candidate");
    assert!(!app.handle_candidate_owned(candidate).unwrap());
    candidate
}

fn start_campaign(seed: u64, role: Role) -> GameSession {
    let mut session = GameSession::try_new(seed).unwrap();
    assert!(session.submit(C::Wait).accepted);
    assert!(session.submit(C::StartCampaign { role }).accepted);
    session
}

fn mines_level(depth: i16) -> LevelId {
    LevelId {
        branch: BranchId::Mines,
        depth,
    }
}

fn park_all_monsters_on(save: &mut aihack_runtime::save::SaveDataV1, park_level: LevelId) {
    let free_tiles: Vec<Pos> = {
        let map = save.world.levels.map(park_level).unwrap();
        let player_loc = save
            .world
            .entities
            .get(save.world.player_id)
            .and_then(|e| e.actor())
            .map(|a| (a.2, a.3));
        (0..map.height)
            .flat_map(|y| (0..map.width).map(move |x| Pos { x, y }))
            .filter(|&pos| {
                map.tile(pos).ok() == Some(TileKind::Floor) && player_loc != Some((park_level, pos))
            })
            .collect()
    };
    let monsters: Vec<_> = save
        .world
        .entities
        .entities()
        .iter()
        .filter(|entity| matches!(entity.actor_kind(), Some(ActorKind::Monster(_))))
        .map(|entity| entity.id)
        .collect();
    for (idx, id) in monsters.into_iter().enumerate() {
        save.world
            .entities
            .set_actor_location(id, park_level, free_tiles[idx]);
    }
}

fn find_tiles(session: &GameSession, level: LevelId, target: TileKind) -> Vec<Pos> {
    let map = session.world().levels.map(level).unwrap();
    (0..map.height)
        .flat_map(|y| (0..map.width).map(move |x| Pos { x, y }))
        .filter(|&pos| map.tile(pos).ok() == Some(target))
        .collect()
}

fn add_inventory_item(
    save: &mut aihack_runtime::save::SaveDataV1,
    kind: ItemKind,
    buc: BucStatus,
    enchantment: i8,
    known: bool,
) -> EntityId {
    let owner = save.world.player_id;
    let id = save
        .world
        .entities
        .spawn_item(kind, EntityLocation::Inventory { owner })
        .unwrap();
    let letter = save
        .world
        .inventory
        .add_existing_with_next_letter(id)
        .unwrap();
    save.world.entities.set_item_letter(id, letter);
    save.world.entities.set_item_buc(id, buc);
    save.world.entities.set_item_enchantment(id, enchantment);
    save.world.entities.set_item_buc_known(id, known);
    save.world.entities.set_item_enchant_known(id, known);
    id
}

#[test]
fn def01_descending_ac_unified_combat_and_tui() {
    assert_eq!(adventurer_template().ac, 10);
    for (kind, expected_ac) in ALL_MONSTERS {
        assert_eq!(
            monster_template(kind).ac,
            expected_ac,
            "monster {kind:?} must use Descending AC"
        );
    }

    assert_eq!(attack_roll_value(2, 10, 0, 8), (10, 10, true));
    assert_eq!(attack_roll_value(2, 4, 0, 8), (10, 16, false));

    let hits_ac_10 = (1..=20)
        .filter(|&roll| attack_roll_value(2, 10, 0, roll).2)
        .count();
    let hits_ac_4 = (1..=20)
        .filter(|&roll| attack_roll_value(2, 4, 0, roll).2)
        .count();
    assert!(hits_ac_10 > hits_ac_4);

    let mut session = start_campaign(42, Role::Knight);
    let obs = session.observation();
    assert_eq!(obs.player.ac, 10);
    let status = render_panels::status_lines(&obs);
    assert!(status.iter().any(|line| line.contains("AC:10")));
    let armor = obs
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::ArmorLeather)
        .unwrap()
        .item;
    assert!(session.submit(C::Wear { item: armor }).accepted);
    let worn_obs = session.observation();
    assert_eq!(worn_obs.player.ac, 9);
    let worn_status = render_panels::status_lines(&worn_obs);
    assert!(worn_status.iter().any(|line| line.contains("AC:9")));
    let creation = render_panels::character_creation_lines();
    assert!(creation.iter().any(|line| line.contains("AC: 10")));
}

#[test]
fn def02_inventory_52_slots_slot_reuse_and_full_stack_merge() {
    let mut inv = Inventory::new(EntityId(1));
    for idx in 0..MAX_INVENTORY_SLOTS {
        let letter = inv.add_existing_with_next_letter(EntityId(100 + idx as u32));
        assert_eq!(
            letter,
            Inventory::letter_for_index(idx as u8),
            "slot {idx} should match a-z / A-Z mapping"
        );
    }
    assert!(!inv.has_free_slot());
    assert!(inv.add_existing_with_next_letter(EntityId(999)).is_none());
    inv.remove(EntityId(102));
    assert!(inv.has_free_slot());
    assert_eq!(
        inv.add_existing_with_next_letter(EntityId(1000)),
        Some(InventoryLetter('c'))
    );

    let base_sess = start_campaign(42, Role::Scout);
    let main2 = LevelId::main(2);
    let shop_pos = find_tiles(&base_sess, main2, TileKind::Shop)[0];
    let mut save = base_sess.to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    save.world.current_level = main2;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main2, shop_pos);
    save.world.gold = 1000;
    add_inventory_item(
        &mut save,
        ItemKind::PotionHealing,
        BucStatus::Uncursed,
        0,
        true,
    );
    while save.world.inventory.entries.len() < MAX_INVENTORY_SLOTS {
        add_inventory_item(&mut save, ItemKind::Rock, BucStatus::Uncursed, 0, true);
    }
    save.world
        .entities
        .spawn_item(
            ItemKind::Rock,
            EntityLocation::OnMap {
                level: main2,
                pos: shop_pos,
            },
        )
        .unwrap();
    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(
        !sess
            .submit(C::Buy {
                kind: ItemKind::RingProtection
            })
            .accepted
    );
    assert!(
        sess.submit(C::Buy {
            kind: ItemKind::PotionHealing
        })
        .accepted
    );
    assert!(sess.submit(C::Pickup).accepted);

    let mut overfull = sess.to_save_data();
    overfull
        .world
        .inventory
        .entries
        .push(aihack_core::domain::inventory::InventoryEntry {
            item: EntityId(99999),
            letter: InventoryLetter('1'),
        });
    assert!(GameSession::from_save_data(overfull).is_err());
}

#[test]
fn def03_level_teleport_amulet_block_branch_destination_and_legacy() {
    let mut save = start_campaign(42, Role::Mage).to_save_data();
    let scroll_a = add_inventory_item(
        &mut save,
        ItemKind::ScrollLevelTeleport,
        BucStatus::Uncursed,
        0,
        true,
    );
    let mut camp = GameSession::from_save_data(save).unwrap();
    let from = camp.world().current_level();
    let outcome = camp.submit(C::Read { item: scroll_a });
    assert!(outcome.accepted);
    let to = camp.world().current_level();
    assert_ne!(to, from);
    assert_eq!(to.branch, from.branch);
    assert_eq!(
        camp.world().player_pos(),
        camp.world().levels.stairs_up_pos(to).unwrap()
    );

    let mut amu_save = start_campaign(42, Role::Knight).to_save_data();
    let amulet_id = amu_save.world.campaign.unwrap().amulet;
    let owner = amu_save.world.player_id;
    amu_save
        .world
        .entities
        .set_item_location(amulet_id, EntityLocation::Inventory { owner });
    let a_letter = amu_save
        .world
        .inventory
        .add_existing_with_next_letter(amulet_id)
        .unwrap();
    amu_save.world.entities.set_item_letter(amulet_id, a_letter);
    let scroll_b = add_inventory_item(
        &mut amu_save,
        ItemKind::ScrollLevelTeleport,
        BucStatus::Uncursed,
        0,
        true,
    );
    let mut amu_sess = GameSession::from_save_data(amu_save).unwrap();
    let stay_level = amu_sess.world().current_level();
    let stay_pos = amu_sess.world().player_pos();
    let blocked = amu_sess.submit(C::Read { item: scroll_b });
    assert!(blocked.accepted);
    assert_eq!(amu_sess.world().current_level(), stay_level);
    assert_eq!(amu_sess.world().player_pos(), stay_pos);
    assert!(!amu_sess.world().inventory().contains(scroll_b));
    assert!(blocked
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::Message { .. })));

    let legacy = GameSession::new_for_playing(42);
    let leg_scroll = legacy
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| {
            e.item()
                .is_some_and(|(k, ..)| k == ItemKind::ScrollLevelTeleport)
        })
        .unwrap()
        .id;
    let mut leg_save = legacy.to_save_data();
    let leg_pos = leg_save
        .world
        .entities
        .get(leg_scroll)
        .and_then(|e| e.item())
        .map(|(_, _, loc, ..)| loc);
    assert!(leg_pos.is_some());
    leg_save.world.entities.set_item_location(
        leg_scroll,
        EntityLocation::Inventory {
            owner: leg_save.world.player_id,
        },
    );
    let ll = leg_save
        .world
        .inventory
        .add_existing_with_next_letter(leg_scroll)
        .unwrap();
    leg_save.world.entities.set_item_letter(leg_scroll, ll);
    let mut leg_sess = GameSession::from_save_data(leg_save).unwrap();
    assert!(leg_sess.submit(C::Read { item: leg_scroll }).accepted);
    assert_eq!(leg_sess.world().current_level(), LevelId::main(2));
}

#[test]
fn def04_hunger_weak_fainting_starvation_and_headless_eat() {
    let base_leg = GameSession::new_for_playing(42).to_save_data();
    let mut normal_save = base_leg.clone();
    normal_save.world.nutrition = 900;
    let mut weak_save = base_leg;
    weak_save.world.nutrition = 30;

    let mut normal = GameSession::from_save_data(normal_save).unwrap();
    let mut weak = GameSession::from_save_data(weak_save).unwrap();
    assert!(render_panels::status_lines(&weak.observation())
        .iter()
        .any(|l| l.contains("WEAK")));
    let n_out = normal.submit(C::Move(Direction::East));
    let w_out = weak.submit(C::Move(Direction::East));
    let n_roll = n_out
        .events
        .iter()
        .find_map(|e| match e {
            GameEvent::AttackResolved { attack_roll, .. } => Some(*attack_roll),
            _ => None,
        })
        .unwrap();
    let w_roll = w_out
        .events
        .iter()
        .find_map(|e| match e {
            GameEvent::AttackResolved { attack_roll, .. } => Some(*attack_roll),
            _ => None,
        })
        .unwrap();
    assert_eq!(w_roll, n_roll - 2);

    let mut faint_save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut faint_save, mines_level(2));
    faint_save.world.nutrition = 1;
    let mut faint = GameSession::from_save_data(faint_save).unwrap();
    let faint_out = faint.submit(C::Wait);
    assert_eq!(faint.world().nutrition, 0);
    assert!(faint.world().paralysis_turns >= 1);
    assert!(faint_out
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::Message { text, .. } if text.contains("faint"))));
    assert!(render_panels::status_lines(&faint.observation())
        .iter()
        .any(|l| l.contains("FAINT")));

    let mut starve_save = faint.to_save_data();
    starve_save.world.nutrition = STARVATION_THRESHOLD + 1;
    starve_save.world.paralysis_turns = 0;
    let mut starve = GameSession::from_save_data(starve_save).unwrap();
    let starved = starve.submit(C::Wait);
    assert!(matches!(
        starved.next_state,
        RunState::GameOver {
            cause: DeathCause::Starvation,
            ..
        }
    ));
    let mut term = Terminal::new(TestBackend::new(80, 28)).unwrap();
    let mut app = TuiApp::new(starve, UiRuntimeConfig::default());
    term.draw(|f| render_frame(f, &mut app)).unwrap();
    let screen: String = (0..28)
        .map(|y| {
            (0..80)
                .map(|x| term.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.contains("Died of starvation"));

    let mut hl_save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut hl_save, mines_level(2));
    hl_save.world.nutrition = 120;
    let mut hl_sess = GameSession::from_save_data(hl_save).unwrap();
    let candidates = HeadlessPolicy::survival_v1().candidates(&hl_sess);
    assert!(matches!(
        candidates.as_slice(),
        [C::Eat { item }] if hl_sess
            .observation()
            .inventory
            .iter()
            .any(|i| i.item == *item && i.kind == ItemKind::FoodRation)
    ));
    let next_t = hl_sess.turn() + 1;
    run_to_turn(&mut hl_sess, next_t, HeadlessPolicy::survival_v1()).unwrap();
    assert!(hl_sess.observation().player.hunger > 150);
}

#[test]
fn def05_campaign_natural_hp_regen_only() {
    let mut save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    let pid = save.world.player_id;
    save.world.entities.actor_stats_mut(pid).unwrap().hp = 10;
    save.world.nutrition = 900;
    let mut camp = GameSession::from_save_data(save).unwrap();
    for _ in 0..14 {
        assert!(camp.submit(C::Wait).accepted);
    }
    assert_eq!(camp.observation().player.hp, 11);

    let mut leg_save = GameSession::new_for_playing(42).to_save_data();
    leg_save.world.entities.clear_monsters();
    let leg_pid = leg_save.world.player_id;
    leg_save.world.entities.actor_stats_mut(leg_pid).unwrap().hp = 10;
    let mut leg = GameSession::from_save_data(leg_save).unwrap();
    for _ in 0..20 {
        assert!(leg.submit(C::Wait).accepted);
    }
    assert_eq!(leg.observation().player.hp, 10);
}

#[test]
fn def06_unidentified_buc_enchant_and_shop_appearance_price() {
    let tmp = start_campaign(42, Role::Knight);
    let main2 = LevelId::main(2);
    let shop_pos = find_tiles(&tmp, main2, TileKind::Shop)[0];
    let mut save = tmp.to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    save.world.current_level = main2;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main2, shop_pos);
    save.world.gold = 500;
    let unappraised_sword = add_inventory_item(
        &mut save,
        ItemKind::ShortSword,
        BucStatus::Blessed,
        2,
        false,
    );
    let healing_id = add_inventory_item(
        &mut save,
        ItemKind::PotionHealing,
        BucStatus::Uncursed,
        0,
        false,
    );

    let mut sess = GameSession::from_save_data(save).unwrap();
    let obs = sess.observation();
    let sw_obs = obs
        .inventory
        .iter()
        .find(|i| i.item == unappraised_sword)
        .unwrap();
    assert!(!sw_obs.buc_known);
    assert!(!sw_obs.enchant_known);
    assert_eq!(sw_obs.buc, BucStatus::Uncursed);
    assert_eq!(sw_obs.enchantment, 0);
    let pot_obs = obs.inventory.iter().find(|i| i.item == healing_id).unwrap();
    assert!(!pot_obs.identified);
    assert!(pot_obs.appearance.is_some());

    let ctas = input::inventory_panel_ctas(&obs);
    assert!(!ctas
        .iter()
        .any(|c| c.label.contains("blessed") || c.label.contains("+2")));

    let buy_outcome = sess.submit(C::Buy {
        kind: ItemKind::PotionCure,
    });
    assert!(buy_outcome.accepted);
    assert!(!buy_outcome.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => text.contains("PotionCure"),
        _ => false,
    }));

    let clock = Arc::new(TestClock::default());
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        sess,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    let mut term = Terminal::new(TestBackend::new(100, 32)).unwrap();
    send_key(&mut app, &clock, '$');
    term.draw(|f| render_frame(f, &mut app)).unwrap();
    let screen: String = (0..32)
        .map(|y| {
            (0..100)
                .map(|x| term.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    let expected_app = appearance_for_kind(obs.seed, ItemKind::PotionCure).unwrap();
    let expected_price = shop_buy_price(ItemKind::PotionCure).unwrap();
    assert!(
        screen.contains(expected_app),
        "shop menu must show randomized appearance"
    );
    assert!(
        screen.contains(&format!("({expected_price}g)")),
        "shop menu must show buy price"
    );
    assert!(!screen.contains("cure potion"));

    let cur_sess =
        GameSession::from_save_data(start_campaign(42, Role::Knight).to_save_data()).unwrap();
    let mut cur_save = cur_sess.to_save_data();
    let ca = add_inventory_item(
        &mut cur_save,
        ItemKind::ArmorLeather,
        BucStatus::Cursed,
        -1,
        false,
    );
    let mut cur_sess = GameSession::from_save_data(cur_save).unwrap();
    assert!(cur_sess.submit(C::Wear { item: ca }).accepted);
    let ca_obs = cur_sess
        .observation()
        .inventory
        .into_iter()
        .find(|i| i.item == ca)
        .unwrap();
    assert!(ca_obs.buc_known);
    assert_eq!(ca_obs.buc, BucStatus::Cursed);
}

#[test]
fn def07_fog_of_war_explored_persistence_tui_muted_and_llm_los_filter() {
    let mut save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    let mut sess = GameSession::from_save_data(save).unwrap();
    let initial_pos = sess.world().player_pos();

    // Walk along corridors/steps to leave previously explored tiles out of active LOS.
    for dir in [
        Direction::North,
        Direction::North,
        Direction::East,
        Direction::East,
        Direction::SouthEast,
        Direction::SouthEast,
        Direction::South,
        Direction::South,
    ] {
        let _ = sess.submit(C::Move(dir));
    }
    let obs = sess.observation();
    assert!(
        obs.visible_tiles.iter().any(|t| !t.visible),
        "remembered out-of-LOS tiles must be emitted with visible=false"
    );

    let reloaded = GameSession::from_save_data(sess.to_save_data()).unwrap();
    let rel_obs = reloaded.observation();
    assert!(rel_obs.visible_tiles.iter().any(|t| !t.visible));
    let memory_tile = rel_obs
        .visible_tiles
        .iter()
        .find(|t| !t.visible && t.pos != initial_pos)
        .unwrap();

    let mut app = TuiApp::new(reloaded, UiRuntimeConfig::default());
    let layout = compute_layout(80, 28);
    let vp = app.viewport_for_observation(layout);
    let mut term = Terminal::new(TestBackend::new(80, 28)).unwrap();
    term.draw(|f| render_frame(f, &mut app)).unwrap();
    if let Some((mx, my)) = vp.world_to_terminal(memory_tile.pos, layout.map) {
        assert_eq!(
            term.backend().buffer()[(mx, my)].fg,
            UiTheme::standard().muted
        );
    }

    let llm_view = LlmObservationView::from(&rel_obs);
    assert!(llm_view.visible_tiles.iter().all(|t| t.visible));
    let llm_bytes = serde_json::to_vec(&llm_view).unwrap().len();
    assert!(llm_bytes < REQUEST_BODY_LIMIT / 2);
}

#[test]
fn def08_content_unified_item_mapping_and_level_overrides() {
    let mut seen_monsters = HashSet::new();
    for (kind, _) in ALL_MONSTERS {
        let id = monster_id(kind);
        assert!(seen_monsters.insert(id));
        assert_eq!(monster_kind_from_id(id).unwrap(), kind);
    }
    let mut seen_items = HashSet::new();
    for kind in ALL_ITEMS {
        let id = item_id(kind);
        assert!(seen_items.insert(id));
        assert_eq!(item_kind_from_id(id).unwrap(), kind);
    }

    let mut level_toml =
        include_str!("../crates/aihack-content/src/data/levels/main_1.toml").to_string();
    level_toml.push_str("\n[[door]]\npos=[11,6]\nstate=\"locked\"\n");
    level_toml.push_str("[[hidden_trap]]\npos=[17,5]\ntrap=\"dart\"\ntile=\"floor\"\n");
    level_toml.push_str("[[hidden_trap]]\npos=[18,5]\ntrap=\"bear\"\ntile=\"floor\"\n");
    for (idx, kind) in ALL_ITEMS.into_iter().enumerate() {
        let x = 20 + (idx % 12) as i16;
        let y = 2 + (idx / 12) as i16;
        level_toml.push_str(&format!(
            "[[item]]\nid=\"{}\"\npos=[{x},{y}]\n",
            item_id(kind)
        ));
    }

    let reg = ContentRegistry::from_toml_sources(
        CONTENT_SCHEMA_VERSION,
        include_str!("../crates/aihack-content/src/data/items.toml"),
        include_str!("../crates/aihack-content/src/data/monsters.toml"),
        &[
            ("levels/main_1.toml", &level_toml),
            (
                "levels/main_2.toml",
                include_str!("../crates/aihack-content/src/data/levels/main_2.toml"),
            ),
        ],
    )
    .unwrap();
    let lvl = reg.level("main:1").unwrap();
    let overrides = level_tile_overrides(lvl).unwrap();
    assert!(overrides.contains(&(Pos { x: 11, y: 6 }, TileKind::Door(DoorState::Locked))));
    assert!(overrides.contains(&(Pos { x: 17, y: 5 }, TileKind::HiddenTrap(TrapKind::Dart))));
    assert!(overrides.contains(&(Pos { x: 18, y: 5 }, TileKind::HiddenTrap(TrapKind::Bear))));

    let spawns = level_spawns(lvl).unwrap();
    for kind in ALL_ITEMS {
        assert!(
            spawns
                .iter()
                .any(|s| matches!(s, LevelSpawn::Item { kind: k, .. } if *k == kind)),
            "level_spawns must decode {kind:?}"
        );
    }
}

#[test]
fn def09_character_creation_cleanup_and_hallucination_log() {
    let creation = render_panels::character_creation_lines();
    assert!(!creation
        .iter()
        .any(|l| l.contains("Strength:") || l.contains("Dexterity:")));
    assert!(creation.iter().any(|l| l.contains("AC: 10")));

    let mut save = start_campaign(42, Role::Knight).to_save_data();
    save.world.hallucinating = true;
    let sess = GameSession::from_save_data(save).unwrap();
    let lines = render_panels::log_lines(&sess.observation(), &["You hear footsteps.".to_string()]);
    assert!(lines
        .iter()
        .any(|l| l.starts_with("Hallucination:") && l.contains("shimmers in impossible colors")));
}

#[test]
fn def10_scroll_buc_branching_and_targeted_identify_tui_llm() {
    let mut save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    let sword = add_inventory_item(
        &mut save,
        ItemKind::ShortSword,
        BucStatus::Uncursed,
        0,
        true,
    );
    let armor = add_inventory_item(
        &mut save,
        ItemKind::ArmorLeather,
        BucStatus::Uncursed,
        0,
        true,
    );
    let cur_ring = add_inventory_item(
        &mut save,
        ItemKind::RingProtection,
        BucStatus::Cursed,
        0,
        true,
    );
    let cur_bag = add_inventory_item(&mut save, ItemKind::Dagger, BucStatus::Cursed, 0, true);
    let s_ew_c = add_inventory_item(
        &mut save,
        ItemKind::ScrollEnchantWeapon,
        BucStatus::Cursed,
        0,
        true,
    );
    let s_ew_b = add_inventory_item(
        &mut save,
        ItemKind::ScrollEnchantWeapon,
        BucStatus::Blessed,
        0,
        true,
    );
    let s_ea_b = add_inventory_item(
        &mut save,
        ItemKind::ScrollEnchantArmor,
        BucStatus::Blessed,
        0,
        true,
    );
    let s_rc_c = add_inventory_item(
        &mut save,
        ItemKind::ScrollRemoveCurse,
        BucStatus::Cursed,
        0,
        true,
    );
    let s_rc_u = add_inventory_item(
        &mut save,
        ItemKind::ScrollRemoveCurse,
        BucStatus::Uncursed,
        0,
        true,
    );
    let s_rc_b = add_inventory_item(
        &mut save,
        ItemKind::ScrollRemoveCurse,
        BucStatus::Blessed,
        0,
        true,
    );
    let s_id = add_inventory_item(
        &mut save,
        ItemKind::ScrollIdentify,
        BucStatus::Uncursed,
        0,
        true,
    );
    let secret_pot = add_inventory_item(
        &mut save,
        ItemKind::PotionExtraHealing,
        BucStatus::Blessed,
        0,
        false,
    );

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::Wield { item: sword }).accepted);
    assert!(sess.submit(C::Wear { item: armor }).accepted);
    assert!(sess.submit(C::PutOn { item: cur_ring }).accepted);

    // Cursed EnchantWeapon reduces & curses
    assert!(sess.submit(C::Read { item: s_ew_c }).accepted);
    let sw_e = sess.world().entities().get(sword).unwrap();
    assert_eq!(sw_e.item_buc(), Some(BucStatus::Cursed));
    assert_eq!(sw_e.item_enchantment(), Some(-1));

    // Cursed RemoveCurse leaves equipped curse intact
    assert!(sess.submit(C::Read { item: s_rc_c }).accepted);
    assert_eq!(
        sess.world().entities().get(sword).unwrap().item_buc(),
        Some(BucStatus::Cursed)
    );

    // Uncursed RemoveCurse removes equipped curses ONLY (backpack cur_bag remains Cursed)
    assert!(sess.submit(C::Read { item: s_rc_u }).accepted);
    assert_eq!(
        sess.world().entities().get(sword).unwrap().item_buc(),
        Some(BucStatus::Uncursed)
    );
    assert_eq!(
        sess.world().entities().get(cur_ring).unwrap().item_buc(),
        Some(BucStatus::Uncursed)
    );
    assert_eq!(
        sess.world().entities().get(cur_bag).unwrap().item_buc(),
        Some(BucStatus::Cursed)
    );

    // Blessed RemoveCurse clears backpack curse too
    assert!(sess.submit(C::Read { item: s_rc_b }).accepted);
    assert_eq!(
        sess.world().entities().get(cur_bag).unwrap().item_buc(),
        Some(BucStatus::Uncursed)
    );

    // Blessed EnchantWeapon & EnchantArmor boost +1..=+2 capped at MAX_ITEM_ENCHANTMENT
    assert!(sess.submit(C::Read { item: s_ew_b }).accepted);
    let sw_ench = sess
        .world()
        .entities()
        .get(sword)
        .unwrap()
        .item_enchantment()
        .unwrap();
    assert!((0..=1).contains(&sw_ench) && sw_ench <= MAX_ITEM_ENCHANTMENT);
    assert!(sess.submit(C::Read { item: s_ea_b }).accepted);
    let ar_ench = sess
        .world()
        .entities()
        .get(armor)
        .unwrap()
        .item_enchantment()
        .unwrap();
    assert!((1..=2).contains(&ar_ench) && ar_ench <= MAX_ITEM_ENCHANTMENT);

    // Reading once or marking ScrollIdentify identified enables targeted Identify in TUI and legal_actions
    let mut id_save = sess.to_save_data();
    id_save
        .world
        .identified_items
        .push(ItemKind::ScrollIdentify);
    let id_sess = GameSession::from_save_data(id_save).unwrap();
    let id_obs = id_sess.observation();
    assert!(id_obs.action_space.commands.len() <= 64);
    let expected_cmd = C::Identify {
        scroll: s_id,
        target: secret_pot,
    };
    assert!(id_obs.legal_actions.contains(&expected_cmd));

    // Crowded 52-slot inventory keeps action_space <= 64 while legal_actions still offers Identify.
    let mut crowd_save = id_sess.to_save_data();
    while crowd_save.world.inventory.entries.len() < MAX_INVENTORY_SLOTS {
        add_inventory_item(
            &mut crowd_save,
            ItemKind::Rock,
            BucStatus::Uncursed,
            0,
            false,
        );
    }
    let crowd_obs = GameSession::from_save_data(crowd_save)
        .unwrap()
        .observation();
    assert!(crowd_obs.action_space.commands.len() <= 64);
    assert!(crowd_obs.legal_actions.contains(&expected_cmd));

    // LLM wire deserialization of IDENTIFY
    let space = ActionSpace {
        commands: vec![ActionIntent::Command(expected_cmd)],
    };
    let wire = format!(
        r#"{{"kind":"DECISION","action":{{"type":"IDENTIFY","scroll":{},"target":{}}},"rationale":"identify potion","confidence":0.95}}"#,
        s_id.0, secret_pot.0
    );
    assert_eq!(
        parse_decision_payload_json(&wire, &space).unwrap().action,
        ActionIntent::Command(expected_cmd)
    );

    // TUI 'r' -> scroll letter -> target letter dispatches CommandIntent::Identify
    let s_id_letter = id_obs
        .inventory
        .iter()
        .find(|i| i.item == s_id)
        .unwrap()
        .letter
        .0;
    let pot_letter = id_obs
        .inventory
        .iter()
        .find(|i| i.item == secret_pot)
        .unwrap()
        .letter
        .0;
    let clock = Arc::new(TestClock::default());
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        id_sess,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    assert_eq!(
        send_key(&mut app, &clock, 'r'),
        UiCommandCandidate::BeginAction('r')
    );
    assert_eq!(
        send_key(&mut app, &clock, s_id_letter),
        UiCommandCandidate::ChooseIdentifyTarget(s_id)
    );
    assert_eq!(
        send_key(&mut app, &clock, pot_letter),
        UiCommandCandidate::Command(expected_cmd)
    );
    let post_pot = app
        .observation()
        .inventory
        .into_iter()
        .find(|i| i.item == secret_pot)
        .unwrap();
    assert!(post_pot.identified);
    assert!(post_pot.buc_known);
    assert!(post_pot.enchant_known);
    assert_eq!(post_pot.buc, BucStatus::Blessed);
}
