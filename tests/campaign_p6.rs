use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use aihack_content::registry;
use aihack_core::action::CommandIntent as C;
use aihack_core::campaign::Role;
use aihack_core::domain::{
    entity::{ActorKind, EntityLocation},
    item::{BucStatus, EquipmentSlot, ItemKind},
    monster::MonsterKind,
    tile::TileKind,
};
use aihack_core::event::{GameEvent, MessagePriority};
use aihack_core::ids::{BranchId, EntityId, LevelId};
use aihack_core::position::{Direction, Pos};
use aihack_headless::HeadlessPolicy;
use aihack_runtime::{domain::item::try_item_data_from_registry, GameSession};
use aihack_tui::tui::{
    input, render_frame, render_panels, runtime_event_to_candidate, TuiApp, UiClock,
    UiCommandCandidate, UiPanel, UiRuntimeConfig,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};

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

const EXPECTED_SHOP_CATALOG: [(ItemKind, &str); 9] = [
    (ItemKind::FoodRation, "item.food.ration"),
    (ItemKind::PotionHealing, "item.potion.healing"),
    (ItemKind::PotionCure, "item.potion.cure"),
    (ItemKind::ScrollEnchantWeapon, "item.scroll.enchant_weapon"),
    (ItemKind::ScrollEnchantArmor, "item.scroll.enchant_armor"),
    (ItemKind::ScrollRemoveCurse, "item.scroll.remove_curse"),
    (ItemKind::RingProtection, "item.ring.protection"),
    (
        ItemKind::RingPoisonResistance,
        "item.ring.poison_resistance",
    ),
    (ItemKind::RingFireResistance, "item.ring.fire_resistance"),
];

const fn mines_level(depth: i16) -> LevelId {
    LevelId {
        branch: BranchId::Mines,
        depth,
    }
}

fn start(seed: u64, role: Role) -> GameSession {
    let mut session = GameSession::try_new(seed).unwrap();
    assert!(session.submit(C::Wait).accepted);
    let outcome = session.submit(C::StartCampaign { role });
    assert!(outcome.accepted, "{:?}", outcome.events);
    session
}

fn find_tiles(session: &GameSession, level: LevelId, target: TileKind) -> Vec<Pos> {
    let map = session.world().levels.map(level).unwrap();
    (0..map.height)
        .flat_map(|y| (0..map.width).map(move |x| Pos { x, y }))
        .filter(|&pos| map.tile(pos).ok() == Some(target))
        .collect()
}

fn park_all_monsters_on(save: &mut aihack_runtime::save::SaveDataV1, park_level: LevelId) {
    let player_loc = save.world.entities.actor_location(save.world.player_id);
    let free_tiles: Vec<Pos> = {
        let map = save.world.levels.map(park_level).unwrap();
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

fn adjacent_floor_pair(session: &GameSession, level: LevelId) -> (Pos, Pos) {
    let map = session.world().levels.map(level).unwrap();
    let is_occupied = |pos: Pos| {
        session
            .world()
            .entities
            .alive_hostile_at(level, pos)
            .is_some()
            || (session.world().current_level() == level && session.world().player_pos() == pos)
    };
    for y in 1..map.height - 1 {
        for x in 1..map.width - 2 {
            let p = Pos { x, y };
            let q = p.offset(Direction::East.delta());
            if map.tile(p).ok() == Some(TileKind::Floor)
                && map.tile(q).ok() == Some(TileKind::Floor)
                && !is_occupied(p)
                && !is_occupied(q)
            {
                return (p, q);
            }
        }
    }
    panic!("no adjacent floor pair found on {level:?}");
}

#[test]
fn p6a_ring_slot_ac_stacking_cursed_guards_and_save_validation() {
    let base = start(42, Role::Knight);
    let main2 = LevelId::main(2);
    let shop_pos = *find_tiles(&base, main2, TileKind::Shop)
        .first()
        .expect("Main 2 must contain Shop");

    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world.current_level = main2;
    let player_id = save.world.player_id;
    save.world
        .entities
        .set_actor_location(player_id, main2, shop_pos);
    let base_ac = save.world.entities.actor_stats(player_id).unwrap().ac;

    let armor_id = add_inventory_item(&mut save, ItemKind::ArmorLeather, BucStatus::Uncursed, 0);
    let ring_prot = add_inventory_item(&mut save, ItemKind::RingProtection, BucStatus::Uncursed, 1);
    let scroll_enc = add_inventory_item(
        &mut save,
        ItemKind::ScrollEnchantArmor,
        BucStatus::Uncursed,
        0,
    );
    let cursed_fire = add_inventory_item(
        &mut save,
        ItemKind::RingFireResistance,
        BucStatus::Cursed,
        0,
    );
    let scroll_rc = add_inventory_item(
        &mut save,
        ItemKind::ScrollRemoveCurse,
        BucStatus::Uncursed,
        0,
    );

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::Wear { item: armor_id }).accepted);
    assert_eq!(
        sess.world().entities().actor_stats(player_id).unwrap().ac,
        base_ac - 1
    );

    assert!(sess.submit(C::PutOn { item: ring_prot }).accepted);
    assert_eq!(
        sess.world().entities().actor_stats(player_id).unwrap().ac,
        base_ac - 3
    );
    assert!(sess
        .observation()
        .inventory
        .iter()
        .any(|i| i.item == ring_prot && i.equipped_slot == Some(EquipmentSlot::Ring)));

    assert!(sess.submit(C::Read { item: scroll_enc }).accepted);
    assert_eq!(
        sess.world().entities().actor_stats(player_id).unwrap().ac,
        base_ac - 4
    );

    // Save/load roundtrip preserves equipped_ring and AC, and tampering fails.
    let encoded = serde_json::to_string(&sess.to_save_data()).unwrap();
    let restored = GameSession::from_save_data(serde_json::from_str(&encoded).unwrap()).unwrap();
    assert_eq!(
        restored
            .world()
            .entities()
            .actor_stats(player_id)
            .unwrap()
            .ac,
        base_ac - 4
    );
    let mut bad_save = restored.to_save_data();
    bad_save
        .world
        .entities
        .actor_stats_mut(player_id)
        .unwrap()
        .ac += 1;
    assert!(GameSession::from_save_data(bad_save).is_err());

    assert!(sess.submit(C::RemoveRing).accepted);
    assert_eq!(
        sess.world().entities().actor_stats(player_id).unwrap().ac,
        base_ac - 2
    );

    // Cursed ring guards RemoveRing, PutOn swap, Drop, and Sell until ScrollRemoveCurse.
    assert!(sess.submit(C::PutOn { item: cursed_fire }).accepted);
    assert!(sess.observation().player.fire_resistant);
    assert!(!sess.submit(C::RemoveRing).accepted);
    assert!(!sess.submit(C::PutOn { item: ring_prot }).accepted);
    assert!(!sess.submit(C::Drop { item: cursed_fire }).accepted);
    assert!(!sess.submit(C::Sell { item: cursed_fire }).accepted);

    assert!(sess.submit(C::Read { item: scroll_rc }).accepted);
    assert!(sess.submit(C::RemoveRing).accepted);
    assert!(!sess.observation().player.fire_resistant);
    assert!(sess.submit(C::Sell { item: cursed_fire }).accepted);
}

#[test]
fn p6a_poison_and_fire_resistance_and_fire_ant() {
    let base = start(42, Role::Knight);
    let main5 = LevelId::main(5);
    let main6 = LevelId::main(6);
    assert!(base.world().entities().entities().iter().any(|e| {
        e.actor_kind() == Some(ActorKind::Monster(MonsterKind::FireAnt))
            && e.actor().map(|(_, _, lvl, _, _, _)| lvl) == Some(main5)
    }));
    assert!(base.world().entities().entities().iter().any(|e| {
        e.actor_kind() == Some(ActorKind::Monster(MonsterKind::FireAnt))
            && e.actor().map(|(_, _, lvl, _, _, _)| lvl) == Some(main6)
    }));

    // 1. FireAnt passive FireOnMelee vs RingFireResistance
    let (p_pos, ant_pos) = adjacent_floor_pair(&base, main5);
    let fire_ant_id = base
        .world()
        .entities
        .entities()
        .iter()
        .find(|e| {
            e.actor_kind() == Some(ActorKind::Monster(MonsterKind::FireAnt))
                && e.actor().map(|(_, _, lvl, _, _, _)| lvl) == Some(main5)
        })
        .unwrap()
        .id;

    let setup_fire = |equip_fire_ring: bool| {
        let mut save = base.to_save_data();
        park_all_monsters_on(&mut save, mines_level(2));
        save.world.current_level = main5;
        let player_id = save.world.player_id;
        save.world
            .entities
            .set_actor_location(player_id, main5, p_pos);
        save.world
            .entities
            .set_actor_location(fire_ant_id, main5, ant_pos);
        let ring = add_inventory_item(
            &mut save,
            ItemKind::RingFireResistance,
            BucStatus::Uncursed,
            0,
        );
        let mut sess = GameSession::from_save_data(save).unwrap();
        if equip_fire_ring {
            assert!(sess.submit(C::PutOn { item: ring }).accepted);
        }
        sess
    };

    let mut no_res = setup_fire(false);
    let mut with_res = setup_fire(true);
    let mut saw_burn = false;
    for _ in 0..25 {
        let out = no_res.submit(C::Wait);
        assert!(out.accepted);
        if out.events.iter().any(|ev| {
            matches!(
                ev,
                GameEvent::Message {
                    priority: MessagePriority::Warning,
                    text
                } if text.contains("burns you for")
            )
        }) && out.events.iter().any(|ev| {
            matches!(
                ev,
                GameEvent::PassiveAttackTriggered { source, .. } if *source == fire_ant_id
            )
        }) {
            saw_burn = true;
            break;
        }
    }
    assert!(saw_burn, "FireAnt must trigger fire burn when unresisted");

    let mut saw_fire_resist = false;
    for _ in 0..25 {
        let out = with_res.submit(C::Wait);
        assert!(out.accepted);
        assert!(!out.events.iter().any(|ev| {
            matches!(
                ev,
                GameEvent::PassiveAttackTriggered { source, .. } if *source == fire_ant_id
            )
        }));
        if out.events.iter().any(|ev| {
            matches!(
                ev,
                GameEvent::Message {
                    priority: MessagePriority::Info,
                    text
                } if text == "You resist the fire!"
            )
        }) {
            saw_fire_resist = true;
            break;
        }
    }
    assert!(
        saw_fire_resist,
        "RingFireResistance must resist FireAnt passive fire"
    );

    // 2. GiantSpider PoisonOnMelee & poison tick vs RingPoisonResistance
    let mines2 = mines_level(2);
    let (sp_p_pos, sp_pos) = adjacent_floor_pair(&base, mines2);
    let spider_id = base
        .world()
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::GiantSpider)))
        .unwrap()
        .id;

    let setup_poison = |equip_poison_ring: bool, initial_poison: u8| {
        let mut save = base.to_save_data();
        park_all_monsters_on(&mut save, LevelId::main(6));
        save.world.current_level = mines2;
        save.world.poison_turns = initial_poison;
        let player_id = save.world.player_id;
        save.world
            .entities
            .set_actor_location(player_id, mines2, sp_p_pos);
        save.world
            .entities
            .set_actor_location(spider_id, mines2, sp_pos);
        let ring = add_inventory_item(
            &mut save,
            ItemKind::RingPoisonResistance,
            BucStatus::Uncursed,
            0,
        );
        let mut sess = GameSession::from_save_data(save).unwrap();
        if equip_poison_ring {
            assert!(sess.submit(C::PutOn { item: ring }).accepted);
        }
        sess
    };

    let mut unres_spider = setup_poison(false, 0);
    let mut got_poisoned = false;
    for _ in 0..25 {
        let out = unres_spider.submit(C::Wait);
        assert!(out.accepted);
        if unres_spider.observation().player.poison_turns > 0 {
            got_poisoned = true;
            break;
        }
    }
    assert!(got_poisoned);

    let mut res_spider = setup_poison(true, 0);
    let mut saw_poison_resist = false;
    for _ in 0..25 {
        let out = res_spider.submit(C::Wait);
        assert!(out.accepted);
        assert_eq!(res_spider.observation().player.poison_turns, 0);
        if out.events.iter().any(|ev| {
            matches!(
                ev,
                GameEvent::Message {
                    priority: MessagePriority::Info,
                    text
                } if text == "You resist the poison!"
            )
        }) {
            saw_poison_resist = true;
            break;
        }
    }
    assert!(saw_poison_resist);

    // Pre-existing poison_turns ticks down without HP loss while wearing RingPoisonResistance
    let mut save_tick = base.to_save_data();
    park_all_monsters_on(&mut save_tick, LevelId::main(6));
    save_tick.world.current_level = mines2;
    save_tick.world.poison_turns = 2;
    let player_id = save_tick.world.player_id;
    save_tick
        .world
        .entities
        .set_actor_location(player_id, mines2, sp_p_pos);
    let ring = add_inventory_item(
        &mut save_tick,
        ItemKind::RingPoisonResistance,
        BucStatus::Uncursed,
        0,
    );
    let mut tick_sess = GameSession::from_save_data(save_tick).unwrap();
    assert!(tick_sess.submit(C::PutOn { item: ring }).accepted);
    assert_eq!(tick_sess.observation().player.poison_turns, 1);
    let hp_after_equip = tick_sess.observation().player.hp;
    let out = tick_sess.submit(C::Wait);
    assert!(out.accepted);
    assert_eq!(tick_sess.observation().player.poison_turns, 0);
    assert_eq!(tick_sess.observation().player.hp, hp_after_equip);
}

#[test]
fn p6b_shop_tile_economy_catalog_and_tamper_rejection() {
    let base = start(42, Role::Knight);
    assert_eq!(base.observation().player.gold, 50);

    let main2 = LevelId::main(2);
    let mines1 = mines_level(1);
    let main2_shops = find_tiles(&base, main2, TileKind::Shop);
    let mines1_shops = find_tiles(&base, mines1, TileKind::Shop);
    assert_eq!(main2_shops.len(), 1);
    assert_eq!(mines1_shops.len(), 1);
    for depth in [1, 3, 4, 5, 6] {
        assert!(find_tiles(&base, LevelId::main(depth), TileKind::Shop).is_empty());
    }
    assert!(find_tiles(&base, mines_level(2), TileKind::Shop).is_empty());

    // Tampering a forged Shop tile into Main 1 must be rejected by validate_persisted_map
    let mut forged_save = base.to_save_data();
    let floor_pos = *find_tiles(&base, LevelId::main(1), TileKind::Floor)
        .first()
        .unwrap();
    forged_save
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(floor_pos, TileKind::Shop)
        .unwrap();
    assert!(GameSession::from_save_data(forged_save).is_err());

    // Legacy vs Campaign kill gold scaling
    let legacy = GameSession::new_for_playing(42);
    assert_eq!(legacy.observation().player.gold, 0);
    let mut leg_save = legacy.to_save_data();
    let leg_player = leg_save.world.player_id;
    let leg_jackal = EntityId(2);
    let l_jackal_diff = leg_save
        .world
        .entities
        .actor_stats(leg_jackal)
        .unwrap()
        .difficulty;
    leg_save
        .world
        .entities
        .actor_stats_mut(leg_jackal)
        .unwrap()
        .hp = 1;
    leg_save
        .world
        .entities
        .actor_stats_mut(leg_player)
        .unwrap()
        .hit_bonus = 100;
    let mut leg_sess = GameSession::from_save_data(leg_save).unwrap();
    assert!(leg_sess.submit(C::Move(Direction::East)).accepted);
    assert_eq!(leg_sess.observation().player.gold, u32::from(l_jackal_diff));

    let (camp_p, camp_m) = adjacent_floor_pair(&base, main2);
    let mut camp_kill_save = base.to_save_data();
    park_all_monsters_on(&mut camp_kill_save, LevelId::main(6));
    camp_kill_save.world.current_level = main2;
    let camp_player = camp_kill_save.world.player_id;
    camp_kill_save
        .world
        .entities
        .set_actor_location(camp_player, main2, camp_p);
    let fire_ant_id = camp_kill_save
        .world
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::FireAnt)))
        .unwrap()
        .id;
    let ant_diff = camp_kill_save
        .world
        .entities
        .actor_stats(fire_ant_id)
        .unwrap()
        .difficulty;
    camp_kill_save
        .world
        .entities
        .set_actor_location(fire_ant_id, main2, camp_m);
    camp_kill_save
        .world
        .entities
        .actor_stats_mut(fire_ant_id)
        .unwrap()
        .hp = 1;
    let mut camp_kill_sess = GameSession::from_save_data(camp_kill_save).unwrap();
    let dagger = camp_kill_sess
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::Dagger)
        .unwrap()
        .item;
    assert!(camp_kill_sess.submit(C::Wield { item: dagger }).accepted);
    let gold_before = camp_kill_sess.observation().player.gold;
    let is_ant_alive = |sess: &GameSession| {
        sess.world()
            .entities
            .get(fire_ant_id)
            .is_some_and(|e| e.actor().is_some_and(|(_, _, _, _, _, alive)| alive))
    };
    for _ in 0..30 {
        if !is_ant_alive(&camp_kill_sess) {
            break;
        }
        assert!(camp_kill_sess.submit(C::Move(Direction::East)).accepted);
    }
    assert!(!is_ant_alive(&camp_kill_sess));
    assert_eq!(
        camp_kill_sess.observation().player.gold,
        gold_before + u32::from(ant_diff) * 25
    );

    // Shop Buy/Sell contracts
    let off_shop_floor = *find_tiles(&base, main2, TileKind::Floor).first().unwrap();
    let mut shop_save = base.to_save_data();
    park_all_monsters_on(&mut shop_save, LevelId::main(6));
    shop_save.world.current_level = main2;
    let player_id = shop_save.world.player_id;
    shop_save
        .world
        .entities
        .set_actor_location(player_id, main2, off_shop_floor);
    shop_save.world.gold = 300;
    let off_item = shop_save.world.inventory.entries[0].item;
    let mut off_sess = GameSession::from_save_data(shop_save).unwrap();
    assert!(
        !off_sess
            .submit(C::Buy {
                kind: ItemKind::PotionHealing
            })
            .accepted
    );
    assert!(!off_sess.submit(C::Sell { item: off_item }).accepted);

    let mut on_save = off_sess.to_save_data();
    let shop_pos = main2_shops[0];
    on_save
        .world
        .entities
        .set_actor_location(player_id, main2, shop_pos);
    let amulet_id = on_save.world.campaign.unwrap().amulet;
    on_save
        .world
        .entities
        .set_item_location(amulet_id, EntityLocation::Inventory { owner: player_id });
    let amulet_letter = on_save
        .world
        .inventory
        .add_existing_with_next_letter(amulet_id)
        .unwrap();
    on_save
        .world
        .entities
        .set_item_letter(amulet_id, amulet_letter);
    let mut sess = GameSession::from_save_data(on_save).unwrap();

    let reg = registry().unwrap();
    for (kind, id) in EXPECTED_SHOP_CATALOG {
        assert!(reg.item(id).is_some());
        let expected_buy = try_item_data_from_registry(kind, reg)
            .unwrap()
            .base_price
            .max(10);
        assert!(expected_buy >= 10);
        assert!(sess.observation().legal_actions.contains(&C::Buy { kind }));
    }
    assert!(!sess
        .observation()
        .legal_actions
        .contains(&C::Sell { item: amulet_id }));
    assert!(!sess.submit(C::Sell { item: amulet_id }).accepted);
    assert!(
        !sess
            .submit(C::Buy {
                kind: ItemKind::Mace
            })
            .accepted
    );

    let ring_data = try_item_data_from_registry(ItemKind::RingFireResistance, reg).unwrap();
    let ring_price = ring_data.base_price.max(10);
    let g0 = sess.observation().player.gold;
    let inv_before = sess.observation().inventory.len();
    assert!(
        sess.submit(C::Buy {
            kind: ItemKind::RingFireResistance
        })
        .accepted
    );
    assert_eq!(sess.observation().player.gold, g0 - ring_price);
    assert_eq!(sess.observation().inventory.len(), inv_before + 1);
    let bought_ring = sess
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::RingFireResistance)
        .unwrap()
        .item;

    let expected_sell = (ring_data.base_price / 2).max(1);
    let g1 = sess.observation().player.gold;
    assert!(sess.submit(C::Sell { item: bought_ring }).accepted);
    assert_eq!(sess.observation().player.gold, g1 + expected_sell);
    assert!(!sess
        .observation()
        .inventory
        .iter()
        .any(|i| i.item == bought_ring));

    // Insufficient gold rejected & full inventory rejected
    let mut low_gold_save = sess.to_save_data();
    low_gold_save.world.gold = 5;
    let mut low_sess = GameSession::from_save_data(low_gold_save).unwrap();
    assert!(
        !low_sess
            .submit(C::Buy {
                kind: ItemKind::PotionHealing
            })
            .accepted
    );

    let mut full_save = sess.to_save_data();
    full_save.world.gold = 500;
    while full_save.world.inventory.next_letter_index < 26 {
        add_inventory_item(&mut full_save, ItemKind::Rock, BucStatus::Uncursed, 0);
    }
    let mut full_sess = GameSession::from_save_data(full_save).unwrap();
    assert!(
        !full_sess
            .submit(C::Buy {
                kind: ItemKind::PotionHealing
            })
            .accepted
    );
}

#[test]
fn p6c_tui_headless_and_legacy_contract_compatibility() {
    // 1. Legacy V1 serde omission preserves V1 wire shape
    let legacy = GameSession::new_for_playing(42);
    let leg_obs = legacy.observation();
    assert_eq!(leg_obs.player.gold, 0);
    assert!(!leg_obs.player.fire_resistant);
    assert!(!leg_obs.player.poison_resistant);
    let player_json = serde_json::to_value(&leg_obs.player).unwrap();
    let player_obj = player_json.as_object().unwrap();
    assert!(player_obj.get("gold").is_none());
    assert!(player_obj.get("fire_resistant").is_none());
    assert!(player_obj.get("poison_resistant").is_none());

    // 2. TUI rendering, glyphs, inspect labels, status badges, and '$' Shop menu
    let base = start(42, Role::Knight);
    let main2 = LevelId::main(2);
    let shop_pos = *find_tiles(&base, main2, TileKind::Shop).first().unwrap();
    let map2 = base.world().levels.map(main2).unwrap();
    let (step_key, p_pos) = [
        ('j', Direction::North),
        ('k', Direction::South),
        ('l', Direction::West),
        ('h', Direction::East),
    ]
    .into_iter()
    .find_map(|(k, offset_dir)| {
        let candidate = shop_pos.offset(offset_dir.delta());
        (map2.tile(candidate).ok() == Some(TileKind::Floor)).then_some((k, candidate))
    })
    .expect("Shop must have an orthogonal adjacent Floor tile");

    let neighbors: Vec<Pos> = Direction::ALL
        .into_iter()
        .map(|d| p_pos.offset(d.delta()))
        .filter(|&pos| {
            pos != shop_pos
                && map2.tile(pos).ok() == Some(TileKind::Floor)
                && base.world().entities.entities().iter().all(|e| {
                    e.item().is_none_or(|(_, _, loc, _, _)| {
                        loc != EntityLocation::OnMap { level: main2, pos }
                    })
                })
        })
        .collect();
    let ant_pos = neighbors[0];
    let floor_ring_pos = neighbors[1];

    let fire_ant_id = base
        .world()
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::FireAnt)))
        .unwrap()
        .id;

    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world.current_level = main2;
    save.world.gold = 120;
    let player_id = save.world.player_id;
    save.world
        .entities
        .set_actor_location(player_id, main2, p_pos);
    save.world
        .entities
        .set_actor_location(fire_ant_id, main2, ant_pos);
    save.world
        .entities
        .spawn_item(
            ItemKind::RingFireResistance,
            EntityLocation::OnMap {
                level: main2,
                pos: floor_ring_pos,
            },
        )
        .unwrap();
    let inv_fire = add_inventory_item(
        &mut save,
        ItemKind::RingFireResistance,
        BucStatus::Uncursed,
        0,
    );

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::PutOn { item: inv_fire }).accepted);
    let obs = sess.observation();
    assert_eq!(
        input::key_to_candidate('P', &obs),
        Some(UiCommandCandidate::BeginAction('P'))
    );
    assert_eq!(
        input::key_to_candidate('$', &obs),
        Some(UiCommandCandidate::BeginAction('$'))
    );
    assert_eq!(
        input::key_to_candidate('T', &obs),
        Some(UiCommandCandidate::Command(C::RemoveRing))
    );

    let status = render_panels::status_lines(&obs);
    assert!(status
        .iter()
        .any(|l| l.contains("G120") && l.contains("RES:F")));
    let shop_inspect = render_panels::inspect_lines(&obs, Some(shop_pos), UiPanel::Inspect, &[]);
    assert!(shop_inspect.iter().any(|l| l == "tile shop"));
    let ant_inspect = render_panels::inspect_lines(&obs, Some(ant_pos), UiPanel::Inspect, &[]);
    assert!(ant_inspect.iter().any(|l| l.starts_with("entity fire ant")));
    let ring_inspect =
        render_panels::inspect_lines(&obs, Some(floor_ring_pos), UiPanel::Inspect, &[]);
    assert!(ring_inspect
        .iter()
        .any(|l| l == "entity ring of fire resistance"));

    // Verify '$', 'a', and '=' glyphs on map, then move onto Shop tile via TUI
    let mut term = Terminal::new(TestBackend::new(100, 32)).unwrap();
    let clock = Arc::new(TestClock::default());
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        sess,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    let layout = aihack_tui::tui::compute_layout(100, 32);
    let vp = app.viewport_for_observation(layout);
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    {
        let buf = term.backend().buffer();
        let (sx, sy) = vp.world_to_terminal(shop_pos, layout.map).unwrap();
        assert_eq!(buf[(sx, sy)].symbol(), "$");
        let (ax, ay) = vp.world_to_terminal(ant_pos, layout.map).unwrap();
        assert_eq!(buf[(ax, ay)].symbol(), "a");
        let (rx, ry) = vp.world_to_terminal(floor_ring_pos, layout.map).unwrap();
        assert_eq!(buf[(rx, ry)].symbol(), "=");
    }
    send_key(&mut app, &clock, step_key);
    assert_eq!(app.observation().player_pos, shop_pos);

    // Open Shop menu via '$' and buy first item via '1'
    assert_eq!(
        send_key(&mut app, &clock, '$'),
        UiCommandCandidate::BeginAction('$')
    );
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    let menu_screen: String = (0..32)
        .map(|y| {
            (0..100)
                .map(|x| term.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(menu_screen.contains("Shop (Gold: 120)"));
    assert!(menu_screen.contains("[1] Buy "));

    let gold_before_tui_buy = app.observation().player.gold;
    send_key(&mut app, &clock, '1');
    assert!(app.observation().player.gold < gold_before_tui_buy);

    // 3. Headless survival-v1 equips carried ring when ring slot is empty
    let mut hl_save = base.to_save_data();
    park_all_monsters_on(&mut hl_save, mines_level(2));
    hl_save.world.current_level = main2;
    let hl_player = hl_save.world.player_id;
    hl_save
        .world
        .entities
        .set_actor_location(hl_player, main2, p_pos);
    let hl_ring = add_inventory_item(
        &mut hl_save,
        ItemKind::RingProtection,
        BucStatus::Uncursed,
        1,
    );
    let mut hl_sess = GameSession::from_save_data(hl_save).unwrap();
    assert_eq!(
        HeadlessPolicy::survival_v1().candidates(&hl_sess),
        vec![C::PutOn { item: hl_ring }]
    );
    let t0 = hl_sess.turn();
    aihack_headless::run_to_turn(&mut hl_sess, t0 + 1, HeadlessPolicy::survival_v1()).unwrap();
    assert!(hl_sess
        .observation()
        .inventory
        .iter()
        .any(|i| i.item == hl_ring && i.equipped_slot == Some(EquipmentSlot::Ring)));
}
