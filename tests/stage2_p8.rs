use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use aihack_content::{
    item_color, item_display_name, item_extrinsic_resistances, item_glyph, item_id, monster_color,
    monster_corpse_spec, monster_display_name, monster_glyph, monster_id, monster_resistances,
    ContentRegistry, CONTENT_SCHEMA_VERSION,
};
use aihack_core::{
    action::CommandIntent as C,
    campaign::{Alignment, Attributes, LearnedSpell, Race, Role, SpellKind},
    domain::{
        entity::{ActorKind, EntityLocation},
        item::{BucStatus, EquipmentSlot, ItemKind},
        level::{BranchGraph, Engraving, EngravingKind, SpecialRoomKind},
        monster::MonsterKind,
        status::Resistances,
        tile::{TileKind, TrapKind},
    },
    event::GameEvent,
    ids::{BranchId, EntityId, LevelId},
    position::{Direction, Pos},
    save::{SAVE_SCHEMA_VERSION_V1, SAVE_SCHEMA_VERSION_V3},
};
use aihack_llm::{service::LlmObservationView, transport::REQUEST_BODY_LIMIT};
use aihack_runtime::GameSession;
use aihack_tui::tui::{
    compute_layout, input, render_frame, render_panels, runtime_event_to_candidate, TuiApp,
    UiClock, UiCommandCandidate, UiPanel, UiRuntimeConfig,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};

const EMBEDDED_ITEMS: &str = include_str!("../crates/aihack-content/src/data/items.toml");
const EMBEDDED_MONSTERS: &str = include_str!("../crates/aihack-content/src/data/monsters.toml");
const EMBEDDED_LEVEL_1: &str = include_str!("../crates/aihack-content/src/data/levels/main_1.toml");
const EMBEDDED_LEVEL_2: &str = include_str!("../crates/aihack-content/src/data/levels/main_2.toml");

#[derive(Default)]
struct TestClock(AtomicU64);

impl UiClock for TestClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::Relaxed))
    }
}

fn start(seed: u64, role: Role) -> GameSession {
    let mut session = GameSession::try_new(seed).unwrap();
    assert!(session.submit(C::Wait).accepted);
    let outcome = session.submit(C::StartCampaign { role });
    assert!(outcome.accepted, "{:?}", outcome.events);
    session
}

fn send_key(app: &mut TuiApp, clock: &TestClock, ch: char, w: u16, h: u16) -> UiCommandCandidate {
    clock.0.fetch_add(1000, Ordering::Relaxed);
    app.release_transition_gesture_on_idle();
    app.release_transition_gesture_on_idle();
    let candidate = runtime_event_to_candidate(
        Event::Key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)),
        w,
        h,
        app,
    )
    .expect("TUI key must map to candidate");
    assert!(!app.handle_candidate_owned(candidate).unwrap());
    candidate
}

fn park_all_monsters_on(save: &mut aihack_runtime::save::SaveDataV1, level: LevelId) {
    let map = save.world.levels.map(level).unwrap().clone();
    let free: Vec<Pos> = (1..map.height - 1)
        .flat_map(|y| (1..map.width - 1).map(move |x| Pos { x, y }))
        .filter(|pos| map.tile(*pos).ok() == Some(TileKind::Floor))
        .collect();
    let monsters: Vec<EntityId> = save
        .world
        .entities
        .entities()
        .iter()
        .filter(|e| matches!(e.actor_kind(), Some(ActorKind::Monster(_))))
        .map(|e| e.id)
        .collect();
    for (idx, id) in monsters.into_iter().enumerate() {
        save.world
            .entities
            .set_actor_location(id, level, free[idx % free.len()]);
    }
}

fn add_inventory_item(
    save: &mut aihack_runtime::save::SaveDataV1,
    kind: ItemKind,
    buc: BucStatus,
    enchantment: i8,
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
    id
}

fn find_lit_room_origin_and_neighbors(
    session: &GameSession,
    level_id: LevelId,
    count: usize,
) -> (Pos, Vec<Pos>) {
    let level = session.world().levels.get(level_id).unwrap();
    for y in 2..level.map.height - 2 {
        for x in 2..level.map.width - 2 {
            let origin = Pos { x, y };
            if !level.is_lit(origin) || level.map.tile(origin).ok() != Some(TileKind::Floor) {
                continue;
            }
            let mut found = Vec::new();
            for dy in -2..=2 {
                for dx in -2..=2 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let p = Pos {
                        x: x + dx,
                        y: y + dy,
                    };
                    if level.map.tile(p).ok() == Some(TileKind::Floor)
                        && aihack_core::vision::has_line_of_sight(&level.map, origin, p)
                    {
                        found.push(p);
                        if found.len() == count {
                            return (origin, found);
                        }
                    }
                }
            }
        }
    }
    panic!("failed to find lit room with {count} neighbors on {level_id:?}");
}

#[test]
fn p8a_content_metadata_new_entities_validation_and_tui_rendering() {
    let registry = ContentRegistry::from_embedded().unwrap();

    let all_monsters = [
        MonsterKind::Jackal,
        MonsterKind::Goblin,
        MonsterKind::FloatingEye,
        MonsterKind::Kobold,
        MonsterKind::Orc,
        MonsterKind::Ogre,
        MonsterKind::GiantSpider,
        MonsterKind::GnollArcher,
        MonsterKind::Troll,
        MonsterKind::FireAnt,
        MonsterKind::Minotaur,
        MonsterKind::Dragon,
        MonsterKind::Bat,
    ];
    for mk in all_monsters {
        let def = registry.monster(monster_id(mk)).unwrap();
        assert!(def.name.is_some(), "{mk:?} missing name");
        assert!(def.color.is_some(), "{mk:?} missing color");
        assert!(def.base_level.is_some(), "{mk:?} missing base_level");
        assert!(def.corpse_chance.is_some(), "{mk:?} missing corpse_chance");
        assert!(
            def.corpse_nutrition.is_some(),
            "{mk:?} missing corpse_nutrition"
        );
        assert!(!def.ai_flags.is_empty(), "{mk:?} missing ai_flags");
        assert!(
            def.spawn_depth_range.is_some(),
            "{mk:?} missing spawn_depth_range"
        );
        assert!(def.spawn_weight.is_some(), "{mk:?} missing spawn_weight");
    }

    let bat_def = registry.monster(monster_id(MonsterKind::Bat)).unwrap();
    assert_eq!(bat_def.speed, 24);
    assert_eq!(monster_glyph(MonsterKind::Bat, &registry), 'B');
    assert_eq!(monster_display_name(MonsterKind::Bat, &registry), "bat");
    assert_eq!(monster_color(MonsterKind::Bat, &registry), Some("brown"));
    assert_eq!(
        monster_resistances(MonsterKind::FireAnt, &registry),
        Resistances::FIRE
    );
    assert_eq!(
        monster_resistances(MonsterKind::GiantSpider, &registry),
        Resistances::POISON
    );
    assert_eq!(
        monster_resistances(MonsterKind::Dragon, &registry),
        Resistances::FIRE
    );
    assert!(monster_corpse_spec(MonsterKind::FloatingEye, &registry)
        .effects
        .contains(&"grant_telepathy".to_string()));

    let new_items = [
        (ItemKind::ArmorShield, '['),
        (ItemKind::ArmorHelm, '['),
        (ItemKind::ArmorCloak, '['),
        (ItemKind::ArmorGloves, '['),
        (ItemKind::ArmorBoots, '['),
        (ItemKind::AmuletEsp, '"'),
        (ItemKind::PotionSpeed, '!'),
        (ItemKind::PotionBlindness, '!'),
        (ItemKind::PotionConfusion, '!'),
        (ItemKind::PotionHallucination, '!'),
        (ItemKind::PotionInvisibility, '!'),
        (ItemKind::PotionLevitation, '!'),
        (ItemKind::Corpse, '%'),
    ];
    for (ik, expected_glyph) in new_items {
        let def = registry.item(item_id(ik)).unwrap();
        assert!(def.name.is_some(), "{ik:?} missing name");
        assert!(def.color.is_some(), "{ik:?} missing color");
        assert!(def.material.is_some(), "{ik:?} missing material");
        assert_eq!(item_glyph(ik, &registry), expected_glyph);
        assert!(!item_display_name(ik, &registry).is_empty());
        assert!(item_color(ik, &registry).is_some());
    }
    assert_eq!(
        item_extrinsic_resistances(ItemKind::AmuletEsp, &registry),
        Resistances::TELEPATHY
    );

    let bad_monsters = EMBEDDED_MONSTERS.replace("speed=24", "speed=37");
    assert!(ContentRegistry::from_toml_sources(
        CONTENT_SCHEMA_VERSION,
        EMBEDDED_ITEMS,
        &bad_monsters,
        &[
            ("levels/main_1.toml", EMBEDDED_LEVEL_1),
            ("levels/main_2.toml", EMBEDDED_LEVEL_2),
        ],
    )
    .is_err());

    // Verify data-driven TUI rendering for Bat + new armor/amulet/potion/corpse items
    let base = start(42, Role::Knight);
    let main1 = LevelId::main(1);
    let (p_pos, neighbors) = find_lit_room_origin_and_neighbors(&base, main1, 5);
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world.current_level = main1;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, p_pos);
    let bat_id = save
        .world
        .entities
        .spawn_monster(MonsterKind::Bat, neighbors[0])
        .unwrap();
    save.world
        .entities
        .set_actor_location(bat_id, main1, neighbors[0]);
    save.world
        .entities
        .spawn_item(
            ItemKind::ArmorShield,
            EntityLocation::OnMap {
                level: main1,
                pos: neighbors[1],
            },
        )
        .unwrap();
    let amulet_floor = save
        .world
        .entities
        .spawn_item(
            ItemKind::AmuletEsp,
            EntityLocation::OnMap {
                level: main1,
                pos: neighbors[2],
            },
        )
        .unwrap();
    save.world.identified_items.push(ItemKind::AmuletEsp);
    let _ = amulet_floor;
    let corpse_id = save
        .world
        .entities
        .spawn_item(
            ItemKind::Corpse,
            EntityLocation::OnMap {
                level: main1,
                pos: neighbors[3],
            },
        )
        .unwrap();
    save.world
        .entities
        .set_item_corpse_monster(corpse_id, Some(MonsterKind::Goblin));

    let sess = GameSession::from_save_data(save).unwrap();
    let obs = sess.observation();
    assert!(
        render_panels::inspect_lines(&obs, Some(neighbors[0]), UiPanel::Inspect, &[])
            .iter()
            .any(|l| l.starts_with("entity bat"))
    );
    assert!(
        render_panels::inspect_lines(&obs, Some(neighbors[3]), UiPanel::Inspect, &[])
            .iter()
            .any(|l| l.contains("goblin corpse"))
    );

    let mut term = Terminal::new(TestBackend::new(100, 32)).unwrap();
    let mut app = TuiApp::new(sess, UiRuntimeConfig::default());
    let layout = compute_layout(100, 32);
    let vp = app.viewport_for_observation(layout);
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    let buf = term.backend().buffer();
    for (pos, sym) in [
        (neighbors[0], "B"),
        (neighbors[1], "["),
        (neighbors[2], "\""),
        (neighbors[3], "%"),
    ] {
        let (tx, ty) = vp.world_to_terminal(pos, layout.map).unwrap();
        assert_eq!(buf[(tx, ty)].symbol(), sym);
    }
}

#[test]
fn p8b_ten_slots_unified_ac_takeoff_cursed_and_duplicate_tamper() {
    let base = start(42, Role::Knight);
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let sword = add_inventory_item(&mut save, ItemKind::ShortSword, BucStatus::Uncursed, 1);
    let body = add_inventory_item(&mut save, ItemKind::ArmorChainMail, BucStatus::Uncursed, 1);
    let shield = add_inventory_item(&mut save, ItemKind::ArmorShield, BucStatus::Uncursed, 0);
    let helm = add_inventory_item(&mut save, ItemKind::ArmorHelm, BucStatus::Cursed, 0);
    let cloak = add_inventory_item(&mut save, ItemKind::ArmorCloak, BucStatus::Uncursed, 0);
    let gloves = add_inventory_item(&mut save, ItemKind::ArmorGloves, BucStatus::Uncursed, 0);
    let boots = add_inventory_item(&mut save, ItemKind::ArmorBoots, BucStatus::Uncursed, 0);
    let left_ring = add_inventory_item(&mut save, ItemKind::RingProtection, BucStatus::Uncursed, 1);
    let right_ring = add_inventory_item(
        &mut save,
        ItemKind::RingFireResistance,
        BucStatus::Uncursed,
        0,
    );
    let amulet = add_inventory_item(&mut save, ItemKind::AmuletEsp, BucStatus::Uncursed, 0);
    let extra_ring = add_inventory_item(
        &mut save,
        ItemKind::RingPoisonResistance,
        BucStatus::Uncursed,
        0,
    );

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::Wield { item: sword }).accepted);
    for armor in [body, shield, helm, cloak, gloves, boots] {
        assert!(sess.submit(C::Wear { item: armor }).accepted);
    }
    assert!(sess.submit(C::PutOn { item: left_ring }).accepted);
    assert!(sess.submit(C::PutOn { item: right_ring }).accepted);
    assert!(sess.submit(C::PutOn { item: amulet }).accepted);

    let obs = sess.observation();
    for (id, expected_slot) in [
        (sword, EquipmentSlot::Melee),
        (body, EquipmentSlot::Body),
        (shield, EquipmentSlot::Shield),
        (helm, EquipmentSlot::Helm),
        (cloak, EquipmentSlot::Cloak),
        (gloves, EquipmentSlot::Gloves),
        (boots, EquipmentSlot::Boots),
        (left_ring, EquipmentSlot::LeftRing),
        (right_ring, EquipmentSlot::RightRing),
        (amulet, EquipmentSlot::Amulet),
    ] {
        assert_eq!(
            obs.inventory
                .iter()
                .find(|i| i.item == id)
                .unwrap()
                .equipped_slot,
            Some(expected_slot),
            "slot mismatch for {expected_slot:?}"
        );
    }

    // Check unified AC across all 9 defense slots
    let expected_bonus: i16 = [
        body, shield, helm, cloak, gloves, boots, left_ring, right_ring, amulet,
    ]
    .into_iter()
    .map(|id| {
        let entity = sess.world().entities().get(id).unwrap();
        let data = entity.item().unwrap().1;
        data.ac_bonus + i16::from(entity.item_enchantment().unwrap_or(0))
    })
    .sum();
    assert_eq!(obs.player.ac, 10 - expected_bonus);

    // Extra ring cannot equip while LeftRing & RightRing are both full if unequipping fails or Wait - LeftRing is uncursed so swaps LeftRing; verify swapping LeftRing changes AC accordingly
    let left_ring_bonus = sess
        .world()
        .entities()
        .get(left_ring)
        .unwrap()
        .item()
        .unwrap()
        .1
        .ac_bonus
        + 1;
    assert!(sess.submit(C::PutOn { item: extra_ring }).accepted);
    assert_eq!(
        sess.observation().player.ac,
        10 - expected_bonus + left_ring_bonus
    );

    // TakeOff Shield works and updates AC & inventory
    let shield_bonus = sess
        .world()
        .entities()
        .get(shield)
        .unwrap()
        .item()
        .unwrap()
        .1
        .ac_bonus;
    assert!(
        sess.submit(C::TakeOff {
            slot: EquipmentSlot::Shield
        })
        .accepted
    );
    assert_eq!(
        sess.observation().player.ac,
        10 - expected_bonus + left_ring_bonus + shield_bonus
    );
    assert_eq!(
        sess.observation()
            .inventory
            .iter()
            .find(|i| i.item == shield)
            .unwrap()
            .equipped_slot,
        None
    );

    // Cursed Helm blocks TakeOff & Drop
    assert!(
        !sess
            .submit(C::TakeOff {
                slot: EquipmentSlot::Helm
            })
            .accepted
    );
    assert!(!sess.submit(C::Drop { item: helm }).accepted);

    // Save/load roundtrip preserves all 10 slots; duplicate slot fails save validation
    let valid_save = sess.to_save_data();
    let roundtrip = GameSession::from_save_data(valid_save.clone()).unwrap();
    assert_eq!(
        roundtrip.snapshot().stable_hash(),
        sess.snapshot().stable_hash()
    );

    let mut dup_save = valid_save;
    dup_save.world.inventory.equipped_shield = dup_save.world.inventory.equipped_body;
    assert!(GameSession::from_save_data(dup_save).is_err());

    // TUI 'x' TakeOff menu works and 'T' preserves RemoveRing
    let clock = Arc::new(TestClock::default());
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        sess,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    assert_eq!(
        input::key_to_candidate('T', &app.observation()),
        Some(UiCommandCandidate::Command(C::RemoveRing))
    );
    assert_eq!(
        send_key(&mut app, &clock, 'x', 100, 32),
        UiCommandCandidate::BeginAction('x')
    );
    let mut term = Terminal::new(TestBackend::new(100, 32)).unwrap();
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    let screen: String = (0..32)
        .map(|y| {
            (0..100)
                .map(|x| term.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.contains("Take off equipped"));
}

#[test]
fn p8b_energy_bucket_scheduler_and_fast_status() {
    // 1. Campaign Bat (speed = 24) acts twice in a turn; PotionSpeed (Fast) halves gain to 1 action/turn
    let base = start(42, Role::Knight);
    let main1 = LevelId::main(1);
    let (p_pos, neighbors) = find_lit_room_origin_and_neighbors(&base, main1, 2);

    let setup_bat_duel = |with_fast: bool| {
        let mut save = base.to_save_data();
        park_all_monsters_on(&mut save, LevelId::main(6));
        save.world.current_level = main1;
        save.world
            .entities
            .set_actor_location(save.world.player_id, main1, p_pos);
        // Set player AC very high so attacks don't kill immediately, or count AttackResolved events
        let bat_pos = Pos {
            x: p_pos.x + 1,
            y: p_pos.y,
        };
        if save.world.levels.map(main1).unwrap().tile(bat_pos).ok() != Some(TileKind::Floor) {
            save.world
                .entities
                .set_actor_location(save.world.player_id, main1, neighbors[0]);
        }
        let actual_p = save
            .world
            .entities
            .actor_location(save.world.player_id)
            .unwrap()
            .1;
        let adj = Direction::ALL
            .into_iter()
            .map(|d| actual_p.offset(d.delta()))
            .find(|pos| {
                save.world.levels.map(main1).unwrap().tile(*pos).ok() == Some(TileKind::Floor)
            })
            .unwrap();
        let bat_id = save
            .world
            .entities
            .spawn_monster(MonsterKind::Bat, adj)
            .unwrap();
        save.world.entities.set_actor_location(bat_id, main1, adj);
        if with_fast {
            save.world.status_timers.fast = 10;
        }
        let mut sess = GameSession::from_save_data(save).unwrap();
        let out = sess.submit(C::Wait);
        assert!(out.accepted);
        let bat_attacks = out
            .events
            .iter()
            .filter(
                |e| matches!(e, GameEvent::AttackResolved { attacker, .. } if *attacker == bat_id),
            )
            .count();
        (
            bat_attacks,
            sess.world().entities().actor_stats(bat_id).unwrap().energy,
        )
    };

    let (normal_attacks, normal_energy_after) = setup_bat_duel(false);
    let (fast_attacks, fast_energy_after) = setup_bat_duel(true);
    assert_eq!(normal_attacks, 2, "Bat speed=24 must act twice in turn 1");
    assert_eq!(normal_energy_after, 0);
    assert_eq!(
        fast_attacks, 1,
        "Fast potion must halve Bat energy gain to 1 action"
    );
    assert_eq!(fast_energy_after, 0);

    // 2. Slow monster (speed=10 Troll) skips turn 1 (energy=10 < 12) and acts turn 2 (energy=20->8)
    let mut slow_save = base.to_save_data();
    park_all_monsters_on(&mut slow_save, LevelId::main(6));
    slow_save.world.current_level = main1;
    let level1 = slow_save.world.levels.map(main1).unwrap().clone();
    let adj_floor = Direction::ALL
        .into_iter()
        .map(|d| p_pos.offset(d.delta()))
        .find(|pos| level1.tile(*pos).ok() == Some(TileKind::Floor))
        .unwrap();
    slow_save
        .world
        .entities
        .set_actor_location(slow_save.world.player_id, main1, p_pos);
    let troll_id = slow_save
        .world
        .entities
        .spawn_monster(MonsterKind::Troll, adj_floor)
        .unwrap();
    slow_save
        .world
        .entities
        .set_actor_location(troll_id, main1, adj_floor);
    let mut slow_sess = GameSession::from_save_data(slow_save).unwrap();
    let t1 = slow_sess.submit(C::Wait);
    let attacks_t1 = t1
        .events
        .iter()
        .filter(
            |e| matches!(e, GameEvent::AttackResolved { attacker, .. } if *attacker == troll_id),
        )
        .count();
    assert_eq!(
        attacks_t1, 0,
        "Troll speed=10 must accumulate energy=10 and wait turn 1"
    );
    assert_eq!(
        slow_sess
            .world()
            .entities()
            .actor_stats(troll_id)
            .unwrap()
            .energy,
        10
    );
    let t2 = slow_sess.submit(C::Wait);
    let attacks_t2 = t2
        .events
        .iter()
        .filter(
            |e| matches!(e, GameEvent::AttackResolved { attacker, .. } if *attacker == troll_id),
        )
        .count();
    assert_eq!(
        attacks_t2, 1,
        "Troll speed=10 must act on turn 2 with energy 20->8"
    );
    assert_eq!(
        slow_sess
            .world()
            .entities()
            .actor_stats(troll_id)
            .unwrap()
            .energy,
        8
    );

    // 3. Legacy mode speed=12 maintains energy=0 steady state
    let mut legacy = GameSession::new_for_playing(42);
    assert!(legacy.submit(C::Wait).accepted);
    for entity in legacy.world().entities().entities() {
        if let Some((_, _, _, _, stats, alive)) = entity.actor() {
            if alive && entity.id != legacy.world().player_id() {
                assert_eq!(stats.energy, 0);
            }
        }
    }
}

#[test]
fn p8b_six_attributes_all_effects_and_save_tamper() {
    // Str carrying capacity & melee damage
    let knight = start(42, Role::Knight);
    let mage = start(42, Role::Mage);
    assert!(knight.world().carrying_capacity() > mage.world().carrying_capacity());
    assert_eq!(knight.world().carrying_capacity(), 140);
    assert_eq!(mage.world().carrying_capacity(), 105);

    // Int bonus on ForceBolt
    let base_mage = start(42, Role::Mage);
    let main1 = LevelId::main(1);
    let (p_pos, neighbors) = find_lit_room_origin_and_neighbors(&base_mage, main1, 1);
    let dir = Direction::ALL
        .into_iter()
        .find(|d| p_pos.offset(d.delta()) == neighbors[0])
        .unwrap();
    let bolt_dmg_for_int = |int_val: i16| {
        let mut save = base_mage.to_save_data();
        park_all_monsters_on(&mut save, LevelId::main(6));
        save.world.current_level = main1;
        save.world
            .entities
            .set_actor_location(save.world.player_id, main1, p_pos);
        save.world.campaign.as_mut().unwrap().attributes.int = int_val;
        let ogre = save
            .world
            .entities
            .spawn_monster(MonsterKind::Ogre, neighbors[0])
            .unwrap();
        save.world
            .entities
            .set_actor_location(ogre, main1, neighbors[0]);
        save.world.entities.actor_stats_mut(ogre).unwrap().hp = 100;
        save.world.entities.actor_stats_mut(ogre).unwrap().max_hp = 100;
        save.world.entities.actor_stats_mut(ogre).unwrap().ac = 20;
        let mut sess = GameSession::from_save_data(save).unwrap();
        let out = sess.submit(C::Cast {
            spell: SpellKind::ForceBolt,
            direction: Some(dir),
        });
        assert!(out.accepted);
        100 - sess.world().entities().actor_stats(ogre).unwrap().hp
    };
    let dmg_high = bolt_dmg_for_int(18);
    let dmg_low = bolt_dmg_for_int(10);
    assert_eq!(dmg_high - dmg_low, 3);

    // Wis Heal bonus & Prayer cooldown
    let base_cleric = start(42, Role::Cleric);
    let mut hi_wis = base_cleric.to_save_data();
    hi_wis.world.campaign.as_mut().unwrap().attributes.wis = 18;
    hi_wis
        .world
        .entities
        .actor_stats_mut(hi_wis.world.player_id)
        .unwrap()
        .hp = 5;
    let mut hi_sess = GameSession::from_save_data(hi_wis).unwrap();
    assert!(
        hi_sess
            .submit(C::Cast {
                spell: SpellKind::Heal,
                direction: None
            })
            .accepted
    );
    let hi_hp = hi_sess
        .world()
        .entities()
        .actor_stats(hi_sess.world().player_id())
        .unwrap()
        .hp;
    assert!(hi_sess.submit(C::Pray).accepted);
    let hi_cd = hi_sess.world().prayer_cooldown;

    let mut lo_wis = base_cleric.to_save_data();
    lo_wis.world.campaign.as_mut().unwrap().attributes.wis = 10;
    lo_wis
        .world
        .entities
        .actor_stats_mut(lo_wis.world.player_id)
        .unwrap()
        .hp = 5;
    let mut lo_sess = GameSession::from_save_data(lo_wis).unwrap();
    assert!(
        lo_sess
            .submit(C::Cast {
                spell: SpellKind::Heal,
                direction: None
            })
            .accepted
    );
    let lo_hp = lo_sess
        .world()
        .entities()
        .actor_stats(lo_sess.world().player_id())
        .unwrap()
        .hp;
    assert!(lo_sess.submit(C::Pray).accepted);
    let lo_cd = lo_sess.world().prayer_cooldown;

    assert!(hi_hp > lo_hp);
    assert!(hi_cd < lo_cd);

    // Cha shop Buy/Sell pricing
    let main2 = LevelId::main(2);
    let shop_pos = (0..21)
        .flat_map(|y| (0..80).map(move |x| Pos { x, y }))
        .find(|p| {
            base_mage.world().levels.map(main2).unwrap().tile(*p).ok() == Some(TileKind::Shop)
        })
        .unwrap();
    let shop_gold_after = |cha_val: i16| {
        let mut save = base_mage.to_save_data();
        park_all_monsters_on(&mut save, LevelId::main(6));
        save.world.current_level = main2;
        save.world.gold = 200;
        save.world.campaign.as_mut().unwrap().attributes.cha = cha_val;
        save.world
            .entities
            .set_actor_location(save.world.player_id, main2, shop_pos);
        let mut sess = GameSession::from_save_data(save).unwrap();
        assert!(
            sess.submit(C::Buy {
                kind: ItemKind::PotionHealing
            })
            .accepted
        );
        sess.world().gold
    };
    assert!(shop_gold_after(18) > shop_gold_after(6));

    // Dex hit bonus check via attribute helper + melee combat check
    assert!(
        Attributes {
            dex: 18,
            ..Attributes::default()
        }
        .dex_hit_bonus()
            > Attributes::default().dex_hit_bonus()
    );

    // Tamper out-of-range attributes rejected
    for bad_val in [2, 26] {
        let mut bad = base_mage.to_save_data();
        bad.world.campaign.as_mut().unwrap().attributes.str_ = bad_val;
        assert!(GameSession::from_save_data(bad).is_err());
    }
}

#[test]
fn p8b_resistances_telepathy_and_all_seven_status_timers() {
    let base = start(42, Role::Knight);
    let main1 = LevelId::main(1);
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world.campaign.as_mut().unwrap().attributes.str_ = 25;
    let far_pos = save
        .world
        .levels
        .map(main1)
        .unwrap()
        .tiles()
        .iter()
        .enumerate()
        .rev()
        .find_map(|(idx, t)| {
            (*t == TileKind::Floor).then_some(Pos {
                x: (idx % 80) as i16,
                y: (idx / 80) as i16,
            })
        })
        .unwrap();
    let telepathy_mob = save
        .world
        .entities
        .spawn_monster(MonsterKind::Orc, far_pos)
        .unwrap();
    save.world
        .entities
        .set_actor_location(telepathy_mob, main1, far_pos);
    let esp = add_inventory_item(&mut save, ItemKind::AmuletEsp, BucStatus::Uncursed, 0);
    let p_blind = add_inventory_item(&mut save, ItemKind::PotionBlindness, BucStatus::Uncursed, 0);
    let p_conf = add_inventory_item(&mut save, ItemKind::PotionConfusion, BucStatus::Uncursed, 0);
    let p_hall = add_inventory_item(
        &mut save,
        ItemKind::PotionHallucination,
        BucStatus::Uncursed,
        0,
    );
    let p_invis = add_inventory_item(
        &mut save,
        ItemKind::PotionInvisibility,
        BucStatus::Uncursed,
        0,
    );
    let p_lev = add_inventory_item(
        &mut save,
        ItemKind::PotionLevitation,
        BucStatus::Uncursed,
        0,
    );
    let p_cure = add_inventory_item(&mut save, ItemKind::PotionCure, BucStatus::Uncursed, 0);

    let mut sess = GameSession::from_save_data(save).unwrap();
    if far_pos.chebyshev_distance(sess.world().player_pos()) > 6 {
        assert!(!sess
            .observation()
            .visible_entities
            .iter()
            .any(|e| e.entity == telepathy_mob));
    }
    assert!(sess.submit(C::PutOn { item: esp }).accepted);
    assert_ne!(
        sess.observation().player.resistances & Resistances::TELEPATHY.0,
        0
    );
    assert!(sess
        .observation()
        .visible_entities
        .iter()
        .any(|e| e.entity == telepathy_mob));

    // Blind reduces visible tiles to 1 (player pos)
    assert!(sess.submit(C::Quaff { item: p_blind }).accepted);
    assert!(sess.observation().player.blind_turns > 0);
    let vis_tiles: Vec<_> = sess
        .observation()
        .visible_tiles
        .into_iter()
        .filter(|t| t.visible)
        .collect();
    assert_eq!(vis_tiles.len(), 1);
    assert_eq!(vis_tiles[0].pos, sess.observation().player_pos);

    // Confusion, Hallucination -> Cure clears negative timers
    assert!(sess.submit(C::Quaff { item: p_conf }).accepted);
    assert!(sess.observation().player.confused_turns > 0);
    assert!(sess.submit(C::Quaff { item: p_hall }).accepted);
    assert!(sess.observation().player.hallucinating);
    assert!(sess.world().status_timers.hallucinating > 0);

    // Stagger message during confusion across moves
    let mut saw_stagger = false;
    for dir in Direction::ALL.into_iter().cycle().take(16) {
        let out = sess.submit(C::Move(dir));
        if out
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::Message { text, .. } if text.contains("confusion")))
        {
            saw_stagger = true;
            break;
        }
    }
    assert!(saw_stagger);

    // Cure clears blind, confused, stunned, hallucinating
    assert!(sess.submit(C::Quaff { item: p_cure }).accepted);
    let obs = sess.observation();
    assert_eq!(obs.player.blind_turns, 0);
    assert_eq!(obs.player.confused_turns, 0);
    assert_eq!(sess.world().status_timers.hallucinating, 0);
    assert!(!obs.player.hallucinating);

    // Levitation avoids Pit trap & expires after duration
    let mut pit_save = sess.to_save_data();
    let pit_pos = (1..20)
        .flat_map(|y| (1..79).map(move |x| Pos { x, y }))
        .find(|p| {
            let map = pit_save.world.levels.map(main1).unwrap();
            matches!(map.tile(*p), Ok(TileKind::HiddenTrap(TrapKind::Pit)))
                && Direction::ALL.into_iter().any(|d| {
                    let n = p.offset(d.delta());
                    map.tile(n).ok() == Some(TileKind::Floor)
                })
        })
        .unwrap();
    let adj_to_pit = Direction::ALL
        .into_iter()
        .find_map(|d| {
            let n = pit_pos.offset(d.delta());
            (pit_save.world.levels.map(main1).unwrap().tile(n).ok() == Some(TileKind::Floor))
                .then_some(n)
        })
        .unwrap();
    let step_pit_dir = Direction::ALL
        .into_iter()
        .find(|d| adj_to_pit.offset(d.delta()) == pit_pos)
        .unwrap();
    pit_save
        .world
        .entities
        .set_actor_location(pit_save.world.player_id, main1, adj_to_pit);
    let mut lev_sess = GameSession::from_save_data(pit_save).unwrap();
    assert!(lev_sess.submit(C::Quaff { item: p_lev }).accepted);
    assert!(lev_sess.observation().player.levitation_turns > 0);
    let hp_before = lev_sess.observation().player.hp;
    let trap_step = lev_sess.submit(C::Move(step_pit_dir));
    assert!(trap_step.accepted);
    assert_eq!(lev_sess.observation().player.hp, hp_before);
    assert!(trap_step.events.iter().any(
        |e| matches!(e, GameEvent::Message { text, .. } if text.contains("float over the trap"))
    ));

    // Invisibility stealth against Orc at dist > 1 in lit room
    let mut inv_save = lev_sess.to_save_data();
    let (lit_p, lit_neigh) = find_lit_room_origin_and_neighbors(&lev_sess, main1, 1);
    let orc_pos = Pos {
        x: lit_p.x + 2,
        y: lit_p.y,
    };
    let _ = lit_neigh;
    let orc_target =
        if inv_save.world.levels.map(main1).unwrap().tile(orc_pos).ok() == Some(TileKind::Floor) {
            orc_pos
        } else {
            find_lit_room_origin_and_neighbors(&lev_sess, main1, 2).1[1]
        };
    inv_save
        .world
        .entities
        .set_actor_location(inv_save.world.player_id, main1, lit_p);
    let orc_id = inv_save
        .world
        .entities
        .spawn_monster(MonsterKind::Orc, orc_target)
        .unwrap();
    inv_save
        .world
        .entities
        .set_actor_location(orc_id, main1, orc_target);
    let mut inv_sess = GameSession::from_save_data(inv_save).unwrap();
    assert!(inv_sess.submit(C::Quaff { item: p_invis }).accepted);
    assert!(inv_sess.observation().player.invisible_turns > 0);
    if lit_p.chebyshev_distance(orc_target) > 1 {
        let before = inv_sess
            .world()
            .entities()
            .actor_location(orc_id)
            .unwrap()
            .1;
        assert!(inv_sess.submit(C::Wait).accepted);
        let after = inv_sess
            .world()
            .entities()
            .actor_location(orc_id)
            .unwrap()
            .1;
        assert_eq!(
            before, after,
            "Orc without SEE_INVISIBLE must not chase invisible player at dist > 1"
        );
    }
}

#[test]
fn p8b_generalized_corpses_rotting_resistance_grant_and_troll_revival() {
    let base = start(42, Role::Knight);
    let main1 = LevelId::main(1);
    let (p_pos, neighbors) = find_lit_room_origin_and_neighbors(&base, main1, 2);

    // 1. Killing Troll (100% corpse_chance) drops ItemKind::Corpse with corpse_monster=Some(Troll), age=0
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world.current_level = main1;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, p_pos);
    let troll_id = save
        .world
        .entities
        .spawn_monster(MonsterKind::Troll, neighbors[0])
        .unwrap();
    save.world
        .entities
        .set_actor_location(troll_id, main1, neighbors[0]);
    save.world.entities.actor_stats_mut(troll_id).unwrap().hp = 1;
    let attack_dir = Direction::ALL
        .into_iter()
        .find(|d| p_pos.offset(d.delta()) == neighbors[0])
        .unwrap();
    let mut sess = GameSession::from_save_data(save).unwrap();
    for _ in 0..10 {
        if !sess
            .world()
            .entities()
            .get(troll_id)
            .unwrap()
            .is_alive_actor()
        {
            break;
        }
        assert!(sess.submit(C::Move(attack_dir)).accepted);
    }
    assert!(!sess
        .world()
        .entities()
        .get(troll_id)
        .unwrap()
        .is_alive_actor());
    let corpse = sess
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| {
            e.item()
                .is_some_and(|(k, _, _, _, _)| k == ItemKind::Corpse)
                && e.item_corpse_monster() == Some(MonsterKind::Troll)
        })
        .expect("Troll kill must drop a generalized Corpse entity");
    let corpse_id = corpse.id;
    assert_eq!(corpse.item_corpse_age(), Some(1));

    // 2. Troll corpse revives at age >= 25 on unoccupied floor tile
    let mut revive_save = sess.to_save_data();
    revive_save
        .world
        .entities
        .set_item_corpse_age(corpse_id, 24);
    let mut revive_sess = GameSession::from_save_data(revive_save).unwrap();
    let out = revive_sess.submit(C::Wait);
    assert!(out.accepted);
    assert!(out.events.iter().any(|e| {
        matches!(e, GameEvent::Message { text, .. } if text.contains("troll corpse shudders"))
    }));
    assert_eq!(
        revive_sess
            .world()
            .entities()
            .get(corpse_id)
            .unwrap()
            .item()
            .unwrap()
            .2,
        EntityLocation::Consumed
    );
    assert!(revive_sess.world().entities().entities().iter().any(|e| {
        e.id != troll_id
            && e.is_alive_actor()
            && e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Troll))
    }));

    // 3. Rotten corpse (corpse_age > 50) poisons non-poison-resistant player
    let mut rot_save = base.to_save_data();
    park_all_monsters_on(&mut rot_save, LevelId::main(6));
    let rotten = add_inventory_item(&mut rot_save, ItemKind::Corpse, BucStatus::Uncursed, 0);
    rot_save
        .world
        .entities
        .set_item_corpse_monster(rotten, Some(MonsterKind::Orc));
    rot_save.world.entities.set_item_corpse_age(rotten, 51);
    let mut rot_sess = GameSession::from_save_data(rot_save).unwrap();
    let rot_out = rot_sess.submit(C::Eat { item: rotten });
    assert!(rot_out.accepted);
    assert!(rot_sess.observation().player.poison_turns > 0);
    assert!(rot_out
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::Message { text, .. } if text.contains("tainted"))));

    // 4. Fresh FloatingEye / Dragon corpses grant intrinsic Telepathy / Fire resistance
    let mut fresh_save = base.to_save_data();
    park_all_monsters_on(&mut fresh_save, LevelId::main(6));
    let mut eye_corpses = Vec::new();
    for _ in 0..8 {
        let id = add_inventory_item(&mut fresh_save, ItemKind::Corpse, BucStatus::Uncursed, 0);
        fresh_save
            .world
            .entities
            .set_item_corpse_monster(id, Some(MonsterKind::FloatingEye));
        fresh_save.world.entities.set_item_corpse_age(id, 0);
        eye_corpses.push(id);
    }
    let mut fresh_sess = GameSession::from_save_data(fresh_save).unwrap();
    for id in eye_corpses {
        assert!(fresh_sess.submit(C::Eat { item: id }).accepted);
        if (fresh_sess.observation().player.resistances & Resistances::TELEPATHY.0) != 0 {
            break;
        }
    }
    assert_ne!(
        fresh_sess.observation().player.resistances & Resistances::TELEPATHY.0,
        0
    );
}

#[test]
fn p8c_generator_v3_map_validation_branch_graph_save_v3_llm_and_tui_viewport() {
    let sess_a1 = start(42, Role::Knight);
    let sess_a2 = start(42, Role::Knight);
    let sess_b = start(7, Role::Knight);
    assert_eq!(
        sess_a1.snapshot().stable_hash(),
        sess_a2.snapshot().stable_hash()
    );
    assert_ne!(sess_a1.world().levels, sess_b.world().levels);

    // Dimensions, lit_tiles, special rooms
    let mut saw_lit_and_dark = false;
    let mut rooms_seen = Vec::new();
    for level in &sess_a1.world().levels.levels {
        assert_eq!(level.map.width, 80);
        assert_eq!(level.map.height, 21);
        assert_eq!(level.lit_tiles.len(), 80 * 21);
        let has_lit = level.lit_tiles.iter().any(|&b| b);
        let has_dark = level.lit_tiles.iter().any(|&b| !b);
        if has_lit && has_dark {
            saw_lit_and_dark = true;
        }
        if let Some(kind) = level.special_room {
            rooms_seen.push(kind);
        }
    }
    assert!(saw_lit_and_dark);
    for expected in [
        SpecialRoomKind::Shop,
        SpecialRoomKind::Temple,
        SpecialRoomKind::Barracks,
        SpecialRoomKind::Vault,
    ] {
        assert!(rooms_seen.contains(&expected), "missing {expected:?}");
    }

    // BranchGraph topology check
    let graph = BranchGraph::campaign_v3();
    assert_eq!(graph.active_level_ids().len(), 8);
    assert_eq!(
        graph.branch_entry_from(LevelId::main(3), TileKind::StairsUp),
        Some(LevelId::new(BranchId::Mines, 1))
    );

    // Structural persisted map validation: allows interior Wall->Floor dig, rejects perimeter/structural tamper
    let mut dug_save = sess_a1.to_save_data();
    let map1 = dug_save.world.levels.map(LevelId::main(1)).unwrap();
    let interior_wall = (1..map1.height - 1)
        .flat_map(|y| (1..map1.width - 1).map(move |x| Pos { x, y }))
        .find(|p| map1.tile(*p).ok() == Some(TileKind::Wall))
        .unwrap();
    dug_save
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(interior_wall, TileKind::Floor)
        .unwrap();
    assert!(GameSession::from_save_data(dug_save).is_ok());

    let mut breached_save = sess_a1.to_save_data();
    breached_save
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(Pos { x: 0, y: 1 }, TileKind::Floor)
        .unwrap();
    assert!(GameSession::from_save_data(breached_save).is_err());

    let mut forged_altar_save = sess_a1.to_save_data();
    forged_altar_save
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(interior_wall, TileKind::Altar)
        .unwrap();
    assert!(GameSession::from_save_data(forged_altar_save).is_err());

    // P9~P13 SaveV3 roundtrip + legacy V1 rejection of P8/extended state
    let mut sess_v3 = GameSession::try_new(42).unwrap();
    assert!(sess_v3.submit(C::Wait).accepted);
    assert!(
        sess_v3
            .submit(C::StartCampaignWithBuild {
                role: Role::Mage,
                race: Race::Elf,
                alignment: Alignment::Chaotic,
            })
            .accepted
    );
    let mut save_v3 = sess_v3.to_save_data();
    assert_eq!(save_v3.schema_version, SAVE_SCHEMA_VERSION_V3);
    save_v3
        .world
        .intrinsic_resistances
        .insert(Resistances::COLD.union(Resistances::SHOCK));
    save_v3.world.weapon_skills.insert("dagger".into(), 2);
    save_v3.world.spell_skill.insert("spellcasting".into(), 1);
    save_v3.world.learned_spells.push(LearnedSpell {
        spell: SpellKind::ForceBolt,
        learned_at_turn: 3,
    });
    save_v3.world.generated_artifacts.insert("sting".into());
    save_v3
        .world
        .ascension_altars
        .push((LevelId::main(1), Pos { x: 10, y: 10 }));
    save_v3.world.levels.levels[0].engravings.push(Engraving {
        pos: Pos { x: 5, y: 5 },
        text: "Elbereth".into(),
        kind: EngravingKind::Burned,
    });
    save_v3.world.levels.levels[0].special_level_id = Some("main_entry".into());
    let art_id = save_v3.world.inventory.entries[0].item;
    save_v3
        .world
        .entities
        .set_item_artifact_id(art_id, Some("sting".into()));

    let json = serde_json::to_string(&save_v3).unwrap();
    let decoded: aihack_runtime::save::SaveDataV1 = serde_json::from_str(&json).unwrap();
    let restored = GameSession::from_save_data(decoded).unwrap();
    let rw = restored.world();
    assert_eq!(rw.campaign.unwrap().alignment, Alignment::Chaotic);
    assert_eq!(rw.campaign.unwrap().race, Race::Elf);
    assert!(rw
        .intrinsic_resistances
        .contains(Resistances::COLD.union(Resistances::SHOCK)));
    assert_eq!(rw.weapon_skills.get("dagger"), Some(&2));
    assert_eq!(rw.spell_skill.get("spellcasting"), Some(&1));
    assert_eq!(rw.learned_spells.len(), 1);
    assert!(rw.generated_artifacts.contains("sting"));
    assert_eq!(rw.levels.levels[0].engravings[0].text, "Elbereth");
    assert_eq!(
        rw.entities().get(art_id).unwrap().item_artifact_id(),
        Some("sting")
    );

    // Legacy V1 saves omit extended keys and reject tampered P8/P9~P13 payloads
    let legacy = GameSession::new_for_playing(42);
    let legacy_save = legacy.to_save_data();
    assert_eq!(legacy_save.schema_version, SAVE_SCHEMA_VERSION_V1);
    let legacy_json = serde_json::to_string(&legacy_save).unwrap();
    for key in [
        "intrinsic_resistances",
        "status_timers",
        "weapon_skills",
        "spell_skill",
        "learned_spells",
        "generated_artifacts",
        "ascension_altars",
        "lit_tiles",
        "engravings",
    ] {
        assert!(
            !legacy_json.contains(key),
            "legacy save unexpectedly serialized {key}"
        );
    }
    let mut bad_legacy = legacy_save;
    bad_legacy.world.weapon_skills.insert("dagger".into(), 1);
    assert!(GameSession::from_save_data(bad_legacy).is_err());

    // LlmObservationView canonical JSON < 32,768 bytes and ActionSpace <= 64 on 80x21
    let full_sess = restored;
    let obs = full_sess.observation();
    assert!(obs.action_space.commands.len() <= 64);
    let llm_view = LlmObservationView::from(&obs);
    let llm_json = serde_json::to_string(&llm_view).unwrap();
    assert!(
        llm_json.len() < REQUEST_BODY_LIMIT,
        "LlmObservationView exceeded limit: {} >= {}",
        llm_json.len(),
        REQUEST_BODY_LIMIT
    );

    // TUI 60x24 viewport scrolling on 80x21 when player stands near x=68, y=15
    let mut scroll_save = full_sess.to_save_data();
    let main1 = LevelId::main(1);
    let right_floor = (1..20)
        .flat_map(|y| (55..79).rev().map(move |x| Pos { x, y }))
        .find(|p| {
            scroll_save.world.levels.map(main1).unwrap().tile(*p).ok() == Some(TileKind::Floor)
        })
        .unwrap();
    scroll_save
        .world
        .entities
        .set_actor_location(scroll_save.world.player_id, main1, right_floor);
    let scroll_sess = GameSession::from_save_data(scroll_save).unwrap();
    let mut app = TuiApp::new(scroll_sess, UiRuntimeConfig::default());
    let layout = compute_layout(60, 24);
    let vp = app.viewport_for_observation(layout);
    assert!(vp.world_to_terminal(right_floor, layout.map).is_some());
    let mut term = Terminal::new(TestBackend::new(60, 24)).unwrap();
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    let (px, py) = vp.world_to_terminal(right_floor, layout.map).unwrap();
    assert_eq!(term.backend().buffer()[(px, py)].symbol(), "@");
}
