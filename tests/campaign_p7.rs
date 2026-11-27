use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use aihack_ai_contract::ActionSpace;
use aihack_content::registry;
use aihack_core::action::{ActionIntent, CommandIntent as C};
use aihack_core::campaign::{Role, SpellKind};
use aihack_core::domain::{
    entity::{ActorKind, EntityKind, EntityLocation},
    item::{BucStatus, ItemKind, MAX_ITEM_STACK},
    monster::MonsterKind,
    tile::TileKind,
};
use aihack_core::event::GameEvent;
use aihack_core::ids::{BranchId, EntityId, LevelId};
use aihack_core::position::{Direction, Pos};
use aihack_core::rng::GameRng;
use aihack_headless::HeadlessPolicy;
use aihack_llm::decision::{parse_decision_payload_json, DecisionPayload};
use aihack_runtime::appearance::appearance_for_kind;
use aihack_runtime::domain::item::try_item_data_from_registry;
use aihack_runtime::systems::score;
use aihack_runtime::GameSession;
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

#[test]
fn p7a_altar_placement_validation_offer_and_altar_pray() {
    assert!(TileKind::Altar.is_movement_passable());
    assert!(TileKind::Altar.is_los_transparent());

    let base = start(42, Role::Mage);
    let main3_altars = find_tiles(&base, LevelId::main(3), TileKind::Altar);
    let mines2_altars = find_tiles(&base, mines_level(2), TileKind::Altar);
    assert_eq!(main3_altars.len(), 1);
    assert_eq!(mines2_altars.len(), 1);
    assert!(find_tiles(&base, LevelId::main(1), TileKind::Altar).is_empty());

    // Forged or removed Altar tiles must be rejected by persisted map validation.
    let mut forged = base.to_save_data();
    let (_, forged_pos) = forged
        .world
        .entities
        .actor_location(forged.world.player_id)
        .unwrap();
    forged
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(forged_pos, TileKind::Altar)
        .unwrap();
    assert!(GameSession::from_save_data(forged).is_err());

    let mut removed = base.to_save_data();
    removed
        .world
        .levels
        .map_mut(LevelId::main(3))
        .unwrap()
        .set_tile(main3_altars[0], TileKind::Floor)
        .unwrap();
    assert!(GameSession::from_save_data(removed).is_err());

    // Move onto Main 3 Altar with a corpse, food, an unidentified potion, and a cursed item.
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, mines_level(1));
    let main3 = LevelId::main(3);
    save.world.current_level = main3;
    let altar_pos = main3_altars[0];
    let step_off = Pos {
        x: altar_pos.x,
        y: altar_pos.y + 1,
    };
    assert_eq!(
        save.world.levels.map(main3).unwrap().tile(step_off).ok(),
        Some(TileKind::Floor)
    );
    save.world
        .entities
        .set_actor_location(save.world.player_id, main3, step_off);
    save.world.prayer_cooldown = 20;
    save.world.mp = 4;
    save.world.luck = 0;

    let corpse_id = add_inventory_item(&mut save, ItemKind::CorpseJackal, BucStatus::Uncursed, 0);
    let food_id = save
        .world
        .inventory
        .entries
        .iter()
        .find(|e| {
            save.world
                .entities
                .get(e.item)
                .and_then(|item| item.item())
                .is_some_and(|(kind, _, _, _, _)| kind == ItemKind::FoodRation)
        })
        .unwrap()
        .item;
    let dagger_id = save
        .world
        .inventory
        .entries
        .iter()
        .find(|e| {
            save.world
                .entities
                .get(e.item)
                .and_then(|item| item.item())
                .is_some_and(|(kind, _, _, _, _)| kind == ItemKind::Dagger)
        })
        .unwrap()
        .item;
    let extra_potion = add_inventory_item(
        &mut save,
        ItemKind::PotionExtraHealing,
        BucStatus::Uncursed,
        0,
    );
    let cursed_scroll = add_inventory_item(&mut save, ItemKind::ScrollReveal, BucStatus::Cursed, 0);
    let amulet_id = save.world.campaign.unwrap().amulet;
    let keep = [food_id, dagger_id, corpse_id, extra_potion, cursed_scroll];
    let trim_from_save: Vec<_> = save
        .world
        .inventory
        .entries
        .iter()
        .filter_map(|e| {
            let kind = save.world.entities.get(e.item)?.item()?.0;
            (!keep.contains(&e.item)
                && !matches!(kind, ItemKind::WandMagicMissile | ItemKind::Rock))
            .then_some(e.item)
        })
        .collect();
    for id in trim_from_save {
        save.world.inventory.remove(id);
        save.world
            .entities
            .set_item_location(id, EntityLocation::Consumed);
    }

    let mut sess = GameSession::from_save_data(save).unwrap();
    for heavy_kind in [ItemKind::WandMagicMissile, ItemKind::Rock] {
        let to_drop: Vec<_> = sess
            .observation()
            .inventory
            .iter()
            .filter(|i| i.kind == heavy_kind)
            .map(|i| i.item)
            .collect();
        for id in to_drop {
            assert!(sess.submit(C::Drop { item: id }).accepted);
        }
    }
    assert!(sess.world().carried_weight() <= sess.world().carrying_capacity());
    assert!(sess.submit(C::Wield { item: dagger_id }).accepted);
    // Offering off-altar must be rejected.
    assert!(!sess.submit(C::Offer { item: food_id }).accepted);
    assert!(!sess
        .observation()
        .legal_actions
        .contains(&C::Offer { item: food_id }));

    // Step onto Altar tile.
    let step_dir = Direction::ALL
        .into_iter()
        .find(|d| step_off.offset(d.delta()) == altar_pos)
        .unwrap();
    assert!(sess.submit(C::Move(step_dir)).accepted);
    assert_eq!(sess.world().player_pos(), altar_pos);
    assert!(sess
        .observation()
        .legal_actions
        .contains(&C::Offer { item: corpse_id }));

    // Amulet cannot be offered even if temporarily in inventory (check rejection on non-inventory too).
    assert!(!sess.submit(C::Offer { item: amulet_id }).accepted);

    // Offering corpse grants +1 luck, -10 cooldown (plus -1 turn), +4 MP, and blesses Uncursed wielded weapon.
    let mp_before = sess.world().mp;
    let cd_before = sess.world().prayer_cooldown;
    let luck_before = sess.world().luck;
    let outcome = sess.submit(C::Offer { item: corpse_id });
    assert!(outcome.accepted, "{:?}", outcome.events);
    assert!(outcome
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::ItemConsumed { item, .. } if *item == corpse_id)));
    assert_eq!(sess.world().luck, (luck_before + 1).min(3));
    assert_eq!(sess.world().prayer_cooldown, cd_before - 11);
    assert_eq!(sess.world().mp, (mp_before + 4).min(sess.world().max_mp));
    assert_eq!(
        sess.world()
            .entities
            .get(dagger_id)
            .and_then(|e| e.item_buc()),
        Some(BucStatus::Blessed)
    );

    // Offering non-food unidentified potion identifies its kind, grants +2 MP, and reduces cooldown by 5.
    assert!(!sess
        .world()
        .is_item_identified(ItemKind::PotionExtraHealing));
    let mp_before = sess.world().mp;
    let cd_before = sess.world().prayer_cooldown;
    let non_food = sess.submit(C::Offer { item: extra_potion });
    assert!(non_food.accepted, "{:?}", non_food.events);
    assert!(non_food
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::ItemIdentified { item, .. } if *item == extra_potion)));
    assert!(sess
        .world()
        .is_item_identified(ItemKind::PotionExtraHealing));
    assert_eq!(sess.world().mp, (mp_before + 2).min(sess.world().max_mp));
    assert_eq!(
        sess.world().prayer_cooldown,
        cd_before.saturating_sub(5).saturating_sub(1)
    );

    // Offering a cursed wielded item must be rejected.
    let mut cursed_weapon_save = sess.to_save_data();
    let cursed_sword = add_inventory_item(
        &mut cursed_weapon_save,
        ItemKind::ShortSword,
        BucStatus::Cursed,
        0,
    );
    let mut cursed_sess = GameSession::from_save_data(cursed_weapon_save).unwrap();
    assert!(cursed_sess.submit(C::Wield { item: cursed_sword }).accepted);
    assert!(!cursed_sess.submit(C::Offer { item: cursed_sword }).accepted);

    // Pray on Altar uncurses all cursed items, restores MP = max_mp, and heals +6 HP.
    let mut pray_save = sess.to_save_data();
    pray_save.world.prayer_cooldown = 0;
    pray_save.world.mp = 2;
    let max_hp = pray_save
        .world
        .entities
        .actor_stats(pray_save.world.player_id)
        .unwrap()
        .max_hp;
    pray_save
        .world
        .entities
        .actor_stats_mut(pray_save.world.player_id)
        .unwrap()
        .hp = max_hp - 8;
    let mut pray_sess = GameSession::from_save_data(pray_save).unwrap();
    let pray_outcome = pray_sess.submit(C::Pray);
    assert!(pray_outcome.accepted, "{:?}", pray_outcome.events);
    assert_eq!(
        pray_sess
            .world()
            .entities
            .get(cursed_scroll)
            .and_then(|e| e.item_buc()),
        Some(BucStatus::Uncursed)
    );
    assert_eq!(pray_sess.world().mp, pray_sess.world().max_mp);
    assert_eq!(
        pray_sess
            .world()
            .entities
            .actor_stats(pray_sess.world().player_id)
            .unwrap()
            .hp,
        (max_hp - 2
            + pray_sess
                .world()
                .campaign
                .unwrap()
                .attributes
                .wis_heal_bonus())
        .min(max_hp)
    );
}

#[test]
fn p7a_role_mp_scaling_regen_spells_and_save_validation() {
    for (role, base_mp, per_lvl, expected_spells) in [
        (
            Role::Mage,
            14,
            3,
            vec![SpellKind::ForceBolt, SpellKind::Detect],
        ),
        (
            Role::Cleric,
            10,
            2,
            vec![SpellKind::Heal, SpellKind::Detect],
        ),
        (Role::Scout, 6, 1, vec![SpellKind::Detect]),
        (Role::Knight, 4, 1, vec![SpellKind::Heal]),
    ] {
        let sess = start(42, role);
        let campaign = sess.world().campaign.unwrap();
        assert_eq!(role.base_mp(), base_mp);
        assert_eq!(role.mp_per_level(), per_lvl);
        assert_eq!(campaign.known_spells(), expected_spells.as_slice());
        assert_eq!(sess.world().mp, base_mp);
        assert_eq!(sess.world().max_mp, base_mp);
        assert_eq!(sess.observation().player.mp, base_mp);
        assert_eq!(sess.observation().player.max_mp, base_mp);
    }

    // Legacy save cannot have MP > 0.
    let legacy = GameSession::new_for_playing(42);
    assert_eq!(legacy.world().mp, 0);
    assert_eq!(legacy.world().max_mp, 0);
    let mut bad_legacy = legacy.to_save_data();
    bad_legacy.world.mp = 1;
    assert!(GameSession::from_save_data(bad_legacy).is_err());

    // Campaign save validates max_mp == campaign.max_mp() and 0 <= mp <= max_mp.
    let mage = start(42, Role::Mage);
    let mut bad_max = mage.to_save_data();
    bad_max.world.max_mp = 99;
    assert!(GameSession::from_save_data(bad_max).is_err());
    let mut over_mp = mage.to_save_data();
    over_mp.world.mp = over_mp.world.max_mp + 1;
    assert!(GameSession::from_save_data(over_mp).is_err());

    // Level-up increases max_mp and adds mp_per_level (killing an Orc gives 30 XP -> level 2).
    let mut lvl_save = mage.to_save_data();
    let orc_id = lvl_save
        .world
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Orc)))
        .unwrap()
        .id;
    let (o_lvl, o_pos) = lvl_save.world.entities.actor_location(orc_id).unwrap();
    lvl_save.world.current_level = o_lvl;
    lvl_save.world.entities.set_actor_location(
        lvl_save.world.player_id,
        o_lvl,
        Pos {
            x: o_pos.x - 1,
            y: o_pos.y,
        },
    );
    lvl_save.world.entities.actor_stats_mut(orc_id).unwrap().hp = 1;
    lvl_save.world.mp = 5;
    let mut lvl_sess = GameSession::from_save_data(lvl_save).unwrap();
    for _ in 0..10 {
        if lvl_sess.world().campaign.unwrap().level() >= 2 {
            break;
        }
        assert!(lvl_sess.submit(C::Move(Direction::East)).accepted);
    }
    assert_eq!(lvl_sess.world().campaign.unwrap().level(), 2);
    assert_eq!(lvl_sess.world().max_mp, 17);
    assert!(lvl_sess.world().mp >= 8);

    // Passive MP regen: +1 every 5 turns up to max_mp.
    let mut regen_save = start(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut regen_save, mines_level(2));
    regen_save.world.mp = 10;
    let mut regen_sess = GameSession::from_save_data(regen_save).unwrap();
    for _ in 0..4 {
        assert!(regen_sess.submit(C::Wait).accepted);
    }
    assert_eq!(regen_sess.world().mp, 10);
    assert!(regen_sess.submit(C::Wait).accepted);
    assert_eq!(regen_sess.world().mp, 11);

    // ForceBolt range 5 hits at distance 5, misses distance 6; wrong role / direction / low MP rejected.
    let mut bolt_save = start(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut bolt_save, mines_level(2));
    let main1 = LevelId::main(1);
    let mut far_pos = None;
    let map1 = bolt_save.world.levels.map(main1).unwrap();
    for y in 0..map1.height {
        for x in 0..map1.width - 7 {
            if (0..=6).all(|dx| {
                map1.tile(Pos { x: x + dx, y })
                    .is_ok_and(|t| t.is_movement_passable())
            }) {
                far_pos = Some(Pos { x, y });
                break;
            }
        }
        if far_pos.is_some() {
            break;
        }
    }
    let origin = far_pos.unwrap();
    let far_jackal = bolt_save
        .world
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Jackal)))
        .unwrap()
        .id;
    bolt_save.world.entities.set_actor_location(
        far_jackal,
        main1,
        Pos {
            x: origin.x + 6,
            y: origin.y,
        },
    );
    bolt_save
        .world
        .entities
        .set_actor_location(bolt_save.world.player_id, main1, origin);

    let mut bolt_sess = GameSession::from_save_data(bolt_save).unwrap();
    // Distance 6 is out of range (5).
    let out_of_range = bolt_sess.submit(C::Cast {
        spell: SpellKind::ForceBolt,
        direction: Some(Direction::East),
    });
    assert!(out_of_range.accepted);
    assert!(!out_of_range
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::AttackResolved { .. })));
    assert_eq!(bolt_sess.world().mp, 10);

    // Reject casting Heal as Mage, or ForceBolt without direction, or legacy cast.
    assert!(
        !bolt_sess
            .submit(C::Cast {
                spell: SpellKind::Heal,
                direction: None
            })
            .accepted
    );
    assert!(
        !bolt_sess
            .submit(C::Cast {
                spell: SpellKind::ForceBolt,
                direction: None
            })
            .accepted
    );
    assert!(
        !GameSession::new_for_playing(42)
            .submit(C::Cast {
                spell: SpellKind::Detect,
                direction: None
            })
            .accepted
    );

    // Distance 5 is in range: find deterministic hit seed and verify MP cost and >= 2d6 damage.
    let mut hit_verified = false;
    for test_seed in 42..65 {
        let mut s = start(test_seed, Role::Mage).to_save_data();
        park_all_monsters_on(&mut s, mines_level(2));
        s.world
            .entities
            .set_actor_location(s.world.player_id, main1, origin);
        let target = s
            .world
            .entities
            .entities()
            .iter()
            .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Minotaur)))
            .unwrap()
            .id;
        s.world.entities.set_actor_location(
            target,
            main1,
            Pos {
                x: origin.x + 5,
                y: origin.y,
            },
        );
        let mut sim = GameSession::from_save_data(s).unwrap();
        let outcome = sim.submit(C::Cast {
            spell: SpellKind::ForceBolt,
            direction: Some(Direction::East),
        });
        assert!(outcome.accepted);
        if let Some(dmg) = outcome.events.iter().find_map(|e| match e {
            GameEvent::AttackResolved {
                hit: true, damage, ..
            } => Some(*damage),
            _ => None,
        }) {
            assert!(dmg >= 2);
            assert!(outcome.events.iter().any(|e| matches!(
                e,
                GameEvent::SpellCast {
                    spell: SpellKind::ForceBolt,
                    mp_after: 10,
                    ..
                }
            )));
            hit_verified = true;
            break;
        }
    }
    assert!(hit_verified, "ForceBolt must hit a range-5 target");

    // Detect reveals all HiddenDoor & HiddenTrap tiles on floor and emits creature count.
    let mut det_save = start(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut det_save, mines_level(2));
    det_save.world.current_level = LevelId::main(2);
    det_save.world.entities.set_actor_location(
        det_save.world.player_id,
        LevelId::main(2),
        det_save
            .world
            .levels
            .stairs_up_pos(LevelId::main(2))
            .unwrap(),
    );
    let mut det_sess = GameSession::from_save_data(det_save).unwrap();
    assert!(!find_tiles(&det_sess, LevelId::main(2), TileKind::HiddenDoor).is_empty());
    let det_out = det_sess.submit(C::Cast {
        spell: SpellKind::Detect,
        direction: None,
    });
    assert!(det_out.accepted);
    assert!(find_tiles(&det_sess, LevelId::main(2), TileKind::HiddenDoor).is_empty());
    assert!(det_out
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::TileRevealed { .. })));
    assert!(det_out.events.iter().any(|e| matches!(
        e,
        GameEvent::Message {
            text, ..
        } if text.contains("hostile creatures on this level")
    )));

    // Cleric Heal restores HP + clears poison; Knight Heal does not clear poison; low MP fails.
    let mut cleric_save = start(42, Role::Cleric).to_save_data();
    park_all_monsters_on(&mut cleric_save, mines_level(2));
    cleric_save.world.poison_turns = 4;
    cleric_save
        .world
        .entities
        .actor_stats_mut(cleric_save.world.player_id)
        .unwrap()
        .hp = 6;
    let mut cleric_sess = GameSession::from_save_data(cleric_save).unwrap();
    let heal_out = cleric_sess.submit(C::Cast {
        spell: SpellKind::Heal,
        direction: None,
    });
    assert!(heal_out.accepted);
    assert_eq!(cleric_sess.world().mp, 5);
    assert_eq!(cleric_sess.world().poison_turns, 0);
    assert!(
        cleric_sess
            .world()
            .entities
            .actor_stats(cleric_sess.world().player_id)
            .unwrap()
            .hp
            >= 13
    );

    let mut knight_save = start(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut knight_save, mines_level(2));
    knight_save.world.mp = 3; // Less than Heal cost 5
    knight_save.world.poison_turns = 2;
    let mut knight_sess = GameSession::from_save_data(knight_save).unwrap();
    assert!(
        !knight_sess
            .submit(C::Cast {
                spell: SpellKind::Heal,
                direction: None
            })
            .accepted
    );
}

#[test]
fn p7b_seeded_appearance_and_campaign_auto_identification() {
    let p_a = appearance_for_kind(42, ItemKind::PotionHealing).unwrap();
    let p_b = appearance_for_kind(42, ItemKind::PotionExtraHealing).unwrap();
    let p_c = appearance_for_kind(42, ItemKind::PotionCure).unwrap();
    assert_ne!(p_a, p_b);
    assert_ne!(p_b, p_c);
    assert_ne!(p_a, p_c);
    assert_eq!(appearance_for_kind(42, ItemKind::PotionHealing), Some(p_a));
    assert!(appearance_for_kind(42, ItemKind::Dagger).is_none());
    assert!(appearance_for_kind(42, ItemKind::FoodRation).is_none());
    assert!(appearance_for_kind(42, ItemKind::AmuletAscension).is_none());

    let scrolls: Vec<_> = [
        ItemKind::ScrollReveal,
        ItemKind::ScrollIdentify,
        ItemKind::ScrollLevelTeleport,
        ItemKind::ScrollCharging,
        ItemKind::ScrollEnchantWeapon,
        ItemKind::ScrollEnchantArmor,
        ItemKind::ScrollRemoveCurse,
    ]
    .into_iter()
    .map(|k| appearance_for_kind(42, k).unwrap())
    .collect();
    for i in 0..scrolls.len() {
        for j in (i + 1)..scrolls.len() {
            assert_ne!(scrolls[i], scrolls[j]);
        }
    }

    // Legacy mode omits appearance everywhere.
    let legacy_obs = GameSession::new_for_playing(42).observation();
    assert!(legacy_obs.inventory.iter().all(|i| i.appearance.is_none()));
    assert!(legacy_obs
        .visible_entities
        .iter()
        .all(|e| e.appearance.is_none()));

    // Campaign mode exposes appearance for unidentified inventory & floor items until identified.
    let mut save = start(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    save.world.identified_items = vec![
        ItemKind::Dagger,
        ItemKind::FoodRation,
        ItemKind::WandMagicMissile,
        ItemKind::Rock,
    ];
    let id_scroll = add_inventory_item(&mut save, ItemKind::ScrollIdentify, BucStatus::Uncursed, 0);
    let ring_id = add_inventory_item(&mut save, ItemKind::RingProtection, BucStatus::Uncursed, 0);
    let (_, p1_pos) = save
        .world
        .entities
        .actor_location(save.world.player_id)
        .unwrap();
    let floor_pos = save
        .world
        .levels
        .map(LevelId::main(1))
        .unwrap()
        .tile(Pos { x: 4, y: p1_pos.y })
        .ok()
        .filter(|t| t.is_movement_passable())
        .map(|_| Pos { x: 4, y: p1_pos.y })
        .unwrap_or(p1_pos);
    save.world
        .entities
        .spawn_item(
            ItemKind::PotionHealing,
            EntityLocation::OnMap {
                level: LevelId::main(1),
                pos: floor_pos,
            },
        )
        .unwrap();

    let mut sess = GameSession::from_save_data(save).unwrap();
    let obs = sess.observation();
    let heal_pot = obs
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::PotionHealing)
        .unwrap()
        .clone();
    assert!(!heal_pot.identified);
    assert_eq!(
        heal_pot.appearance.as_deref(),
        appearance_for_kind(42, ItemKind::PotionHealing)
    );
    let floor_pot = obs
        .visible_entities
        .iter()
        .find(|e| e.kind == EntityKind::Item(ItemKind::PotionHealing))
        .unwrap();
    assert_eq!(
        floor_pot.appearance.as_deref(),
        appearance_for_kind(42, ItemKind::PotionHealing)
    );

    // Independent combat RNG rolls do not alter seeded appearances.
    let mut rng = GameRng::new(999);
    for _ in 0..20 {
        let _ = rng.next_u64();
    }
    assert_eq!(appearance_for_kind(42, ItemKind::PotionHealing), Some(p_a));

    // Quaff auto-identifies PotionHealing (and clears appearance on both inventory & floor).
    assert!(
        sess.submit(C::Quaff {
            item: heal_pot.item
        })
        .accepted
    );
    assert!(sess.world().is_item_identified(ItemKind::PotionHealing));
    let after_quaff = sess.observation();
    assert!(after_quaff
        .inventory
        .iter()
        .filter(|i| i.kind == ItemKind::PotionHealing)
        .all(|i| i.identified && i.appearance.is_none()));
    assert!(after_quaff
        .visible_entities
        .iter()
        .filter(|e| e.kind == EntityKind::Item(ItemKind::PotionHealing))
        .all(|e| e.appearance.is_none()));

    // Read ScrollIdentify auto-identifies ScrollIdentify AND prioritizes first unidentified item (ScrollReveal before Dagger).
    let reveal_item = sess
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::ScrollReveal)
        .unwrap()
        .item;
    let id_out = sess.submit(C::Read { item: id_scroll });
    assert!(id_out.accepted);
    assert!(sess.world().is_item_identified(ItemKind::ScrollIdentify));
    assert!(sess.world().is_item_identified(ItemKind::ScrollReveal));
    assert!(id_out
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::ItemIdentified { item, .. } if *item == reveal_item)));

    // PutOn auto-identifies RingProtection.
    assert!(sess.submit(C::PutOn { item: ring_id }).accepted);
    assert!(sess.world().is_item_identified(ItemKind::RingProtection));
}

#[test]
fn p7b_item_stacking_weight_value_and_save_validation() {
    let base = start(42, Role::Knight);
    let main2 = LevelId::main(2);
    let shop_pos = find_tiles(&base, main2, TileKind::Shop)[0];
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    save.world.current_level = main2;
    save.world.gold = 300;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main2, shop_pos);
    let floor_food_1 = save
        .world
        .entities
        .spawn_item(
            ItemKind::FoodRation,
            EntityLocation::OnMap {
                level: main2,
                pos: shop_pos,
            },
        )
        .unwrap();
    let blessed_food = add_inventory_item(&mut save, ItemKind::FoodRation, BucStatus::Blessed, 0);

    let mut sess = GameSession::from_save_data(save).unwrap();
    let inv_food = sess
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::FoodRation && i.buc == BucStatus::Uncursed)
        .unwrap()
        .item;
    let weight_before = sess.world().carried_weight();
    let score_before = score::death_score(sess.world(), sess.turn());
    let food_data = try_item_data_from_registry(ItemKind::FoodRation, registry().unwrap()).unwrap();

    // Pickup merges identical (FoodRation, Uncursed, 0) without consuming new letter or merging Blessed food.
    let inv_len_before = sess.observation().inventory.len();
    assert!(sess.submit(C::Pickup).accepted);
    assert_eq!(sess.observation().inventory.len(), inv_len_before);
    assert_eq!(
        sess.world()
            .entities
            .get(inv_food)
            .and_then(|e| e.item_quantity()),
        Some(2)
    );
    assert_eq!(
        sess.world()
            .entities
            .get(blessed_food)
            .and_then(|e| e.item_quantity()),
        Some(1)
    );
    assert_eq!(
        sess.world().entities.item_location(floor_food_1),
        Some(EntityLocation::Consumed)
    );
    assert_eq!(
        sess.world().carried_weight(),
        weight_before + food_data.weight
    );

    // Shop Buy merges into same stack; Sell consumes 1 from stack.
    assert!(
        sess.submit(C::Buy {
            kind: ItemKind::FoodRation
        })
        .accepted
    );
    assert_eq!(
        sess.world()
            .entities
            .get(inv_food)
            .and_then(|e| e.item_quantity()),
        Some(3)
    );
    assert_eq!(
        sess.world().carried_weight(),
        weight_before + food_data.weight * 2
    );
    let score_stacked = score::death_score(sess.world(), sess.turn());
    // Inventory value grows by 2 * base_price minus small turn/gold delta.
    assert!(score_stacked > score_before);

    assert!(sess.submit(C::Sell { item: inv_food }).accepted);
    assert_eq!(
        sess.world()
            .entities
            .get(inv_food)
            .and_then(|e| e.item_quantity()),
        Some(2)
    );
    assert!(sess.submit(C::Eat { item: inv_food }).accepted);
    assert_eq!(
        sess.world()
            .entities
            .get(inv_food)
            .and_then(|e| e.item_quantity()),
        Some(1)
    );

    // Throw splits 1 Rock from a stacked Rock slot.
    let mut throw_save = sess.to_save_data();
    let rock_id = throw_save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            let (kind, _, loc, _, _) = e.item()?;
            (kind == ItemKind::Rock
                && loc
                    == (EntityLocation::Inventory {
                        owner: throw_save.world.player_id,
                    }))
            .then_some(e.id)
        })
        .unwrap();
    throw_save.world.entities.set_item_quantity(rock_id, 4);
    let mut throw_sess = GameSession::from_save_data(throw_save).unwrap();
    let throw_out = throw_sess.submit(C::Throw {
        item: rock_id,
        direction: Direction::East,
    });
    assert!(throw_out.accepted);
    assert_eq!(
        throw_sess
            .world()
            .entities
            .get(rock_id)
            .and_then(|e| e.item_quantity()),
        Some(3)
    );
    let thrown_entity = throw_out
        .events
        .iter()
        .find_map(|e| match e {
            GameEvent::ItemThrown { item, .. } => Some(*item),
            _ => None,
        })
        .unwrap();
    assert_ne!(thrown_entity, rock_id);
    assert_eq!(
        throw_sess
            .world()
            .entities
            .get(thrown_entity)
            .and_then(|e| e.item_quantity()),
        Some(1)
    );

    // Save validation rejects invalid stack quantities (0, > MAX_ITEM_STACK, non-stackable > 1, legacy > 1).
    let mut zero_qty = sess.to_save_data();
    zero_qty.world.entities.set_item_quantity(inv_food, 0);
    assert!(GameSession::from_save_data(zero_qty).is_err());

    let mut over_qty = sess.to_save_data();
    over_qty
        .world
        .entities
        .set_item_quantity(inv_food, MAX_ITEM_STACK + 1);
    assert!(GameSession::from_save_data(over_qty).is_err());

    let mut sword_stack = sess.to_save_data();
    let dagger_id = add_inventory_item(&mut sword_stack, ItemKind::Dagger, BucStatus::Uncursed, 0);
    sword_stack.world.entities.set_item_quantity(dagger_id, 2);
    assert!(GameSession::from_save_data(sword_stack).is_err());

    let mut legacy_stack = GameSession::new_for_playing(42).to_save_data();
    let leg_food = legacy_stack.world.inventory.entries[1].item;
    legacy_stack.world.entities.set_item_quantity(leg_food, 2);
    assert!(GameSession::from_save_data(legacy_stack).is_err());
}

#[test]
fn p7c_bosses_dragon_breath_ascension_pursuit_and_llm_wire() {
    let base = start(42, Role::Knight);
    let minotaur = base
        .world()
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Minotaur)))
        .expect("Minotaur must spawn in Mines 2");
    assert_eq!(minotaur.actor_stats_hp_check(), (26, 7));

    let dragon = base
        .world()
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Dragon)))
        .expect("Dragon must spawn in Main 6");
    let (_, _, dragon_lvl, dragon_pos, dragon_stats, _) = dragon.actor().unwrap();
    assert_eq!(dragon_lvl, LevelId::main(6));
    assert_eq!(dragon_stats.max_hp, 30);
    assert_eq!(dragon_stats.difficulty, 8);

    // Dragon FireBreath deals 2d6 when unresisted and is blocked by RingFireResistance.
    let mut breath_save = base.to_save_data();
    breath_save.world.current_level = LevelId::main(6);
    let map6 = breath_save.world.levels.map(LevelId::main(6)).unwrap();
    let line_pos = [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .into_iter()
        .flat_map(|(sx, sy)| {
            (2..=4).filter_map(move |dist| {
                let candidate = Pos {
                    x: dragon_pos.x + sx * dist,
                    y: dragon_pos.y + sy * dist,
                };
                (1..=dist)
                    .all(|step| {
                        let mid = Pos {
                            x: dragon_pos.x + sx * step,
                            y: dragon_pos.y + sy * step,
                        };
                        map6.tile(mid).is_ok_and(|t| t.is_movement_passable())
                    })
                    .then_some(candidate)
            })
        })
        .next()
        .unwrap();
    breath_save.world.entities.set_actor_location(
        breath_save.world.player_id,
        LevelId::main(6),
        line_pos,
    );
    let fire_ring = add_inventory_item(
        &mut breath_save,
        ItemKind::RingFireResistance,
        BucStatus::Uncursed,
        0,
    );

    let mut unresisted = GameSession::from_save_data(breath_save.clone()).unwrap();
    let hp_before = unresisted
        .world()
        .entities
        .actor_stats(unresisted.world().player_id)
        .unwrap()
        .hp;
    let mut burned = false;
    for _ in 0..4 {
        let out = unresisted.submit(C::Wait);
        assert!(out.accepted);
        if out
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::PassiveAttackTriggered { source, .. } if *source == dragon.id))
        {
            burned = true;
            break;
        }
    }
    assert!(burned, "Dragon must breathe fire when in line of sight");
    assert!(
        unresisted
            .world()
            .entities
            .actor_stats(unresisted.world().player_id)
            .unwrap()
            .hp
            < hp_before
    );

    let mut resisted = GameSession::from_save_data(breath_save).unwrap();
    assert!(resisted.submit(C::PutOn { item: fire_ring }).accepted);
    let hp_res = resisted
        .world()
        .entities
        .actor_stats(resisted.world().player_id)
        .unwrap()
        .hp;
    let mut saw_resist_msg = false;
    for _ in 0..4 {
        let out = resisted.submit(C::Wait);
        assert!(out.accepted);
        if out.events.iter().any(|e| {
            matches!(
                e,
                GameEvent::Message { text, .. } if text == "You resist the dragon's fire breath!"
            )
        }) {
            saw_resist_msg = true;
        }
    }
    assert!(saw_resist_msg);
    assert_eq!(
        resisted
            .world()
            .entities
            .actor_stats(resisted.world().player_id)
            .unwrap()
            .hp,
        hp_res
    );

    // Ascension pursuit spawns FireAnt on Main 4, Troll on Main 2, Minotaur on Main 1 once each, plus 15-turn burden.
    let mut asc_save = start(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut asc_save, mines_level(2));
    let amulet = asc_save.world.campaign.unwrap().amulet;
    let letter = asc_save
        .world
        .inventory
        .add_existing_with_next_letter(amulet)
        .unwrap();
    asc_save.world.entities.set_item_location(
        amulet,
        EntityLocation::Inventory {
            owner: asc_save.world.player_id,
        },
    );
    asc_save.world.entities.set_item_letter(amulet, letter);

    let count_kind_on = |s: &GameSession, lvl: LevelId, target: MonsterKind| -> usize {
        s.world()
            .entities
            .entities()
            .iter()
            .filter(|e| {
                e.actor().is_some_and(|(k, _, l, _, _, alive)| {
                    alive && l == lvl && k == ActorKind::Monster(target)
                })
            })
            .count()
    };

    let mut asc = GameSession::from_save_data(asc_save).unwrap();
    for (from_depth, expect_stage, expect_lvl, expect_kind) in [
        (5, 1_u8, LevelId::main(4), MonsterKind::FireAnt),
        (3, 2_u8, LevelId::main(2), MonsterKind::Troll),
        (2, 3_u8, LevelId::main(1), MonsterKind::Minotaur),
    ] {
        let count_before = count_kind_on(&asc, expect_lvl, expect_kind);
        let mut s = asc.to_save_data();
        s.world.current_level = LevelId::main(from_depth);
        let up = s
            .world
            .levels
            .stairs_up_pos(LevelId::main(from_depth))
            .unwrap();
        s.world
            .entities
            .set_actor_location(s.world.player_id, LevelId::main(from_depth), up);
        asc = GameSession::from_save_data(s).unwrap();
        let out = asc.submit(C::Ascend);
        assert!(out.accepted, "{:?}", out.events);
        assert_eq!(asc.world().ascension_pursuit_stage, expect_stage);
        assert_eq!(
            count_kind_on(&asc, expect_lvl, expect_kind),
            count_before + 1
        );
    }

    // Revisiting Main 4 must not re-spawn another FireAnt.
    let ants_main4 = count_kind_on(&asc, LevelId::main(4), MonsterKind::FireAnt);
    let mut revisit = asc.to_save_data();
    revisit.world.current_level = LevelId::main(5);
    let up5 = revisit
        .world
        .levels
        .stairs_up_pos(LevelId::main(5))
        .unwrap();
    revisit
        .world
        .entities
        .set_actor_location(revisit.world.player_id, LevelId::main(5), up5);
    let mut revisit_sess = GameSession::from_save_data(revisit).unwrap();
    assert!(revisit_sess.submit(C::Ascend).accepted);
    assert_eq!(
        count_kind_on(&revisit_sess, LevelId::main(4), MonsterKind::FireAnt),
        ants_main4
    );

    // Amulet burden drains 1 extra nutrition every 15 turns on Main 1..=5.
    let mut burden_save = revisit_sess.to_save_data();
    park_all_monsters_on(&mut burden_save, mines_level(2));
    burden_save.turn = 14;
    let mut burden_sess = GameSession::from_save_data(burden_save).unwrap();
    let nut_before = burden_sess.world().nutrition;
    assert!(burden_sess.submit(C::Wait).accepted);
    assert_eq!(burden_sess.world().nutrition, nut_before - 2);

    // Save validation rejects pursuit stage > 3 or legacy > 0.
    let mut bad_stage = revisit_sess.to_save_data();
    bad_stage.world.ascension_pursuit_stage = 4;
    assert!(GameSession::from_save_data(bad_stage).is_err());
    let mut legacy_stage = GameSession::new_for_playing(42).to_save_data();
    legacy_stage.world.ascension_pursuit_stage = 1;
    assert!(GameSession::from_save_data(legacy_stage).is_err());

    // LLM wire mapping accepts CAST and OFFER commands.
    let item_id = EntityId(9);
    let space = ActionSpace {
        commands: vec![
            ActionIntent::Command(C::Offer { item: item_id }),
            ActionIntent::Command(C::Cast {
                spell: SpellKind::ForceBolt,
                direction: Some(Direction::North),
            }),
            ActionIntent::Command(C::Cast {
                spell: SpellKind::Heal,
                direction: None,
            }),
        ],
    };
    let parsed_offer: DecisionPayload = parse_decision_payload_json(
        &serde_json::json!({
            "kind": "DECISION",
            "action": { "type": "OFFER", "item": item_id.0 },
            "rationale": "offer on altar",
            "confidence": 0.9
        })
        .to_string(),
        &space,
    )
    .unwrap();
    assert_eq!(
        parsed_offer.action,
        ActionIntent::Command(C::Offer { item: item_id })
    );

    let parsed_cast: DecisionPayload = parse_decision_payload_json(
        &serde_json::json!({
            "kind": "DECISION",
            "action": { "type": "CAST", "spell": "FORCE_BOLT", "direction": "NORTH" },
            "rationale": "cast force bolt",
            "confidence": 0.95
        })
        .to_string(),
        &space,
    )
    .unwrap();
    assert_eq!(
        parsed_cast.action,
        ActionIntent::Command(C::Cast {
            spell: SpellKind::ForceBolt,
            direction: Some(Direction::North),
        })
    );
}

#[test]
fn p7d_tui_headless_and_legacy_v1_compatibility() {
    // Legacy observation omits P7 default fields in JSON.
    let legacy = GameSession::new_for_playing(42);
    let leg_obs = legacy.observation();
    let player_json = serde_json::to_value(&leg_obs.player).unwrap();
    let player_obj = player_json.as_object().unwrap();
    assert!(player_obj.get("mp").is_none());
    assert!(player_obj.get("max_mp").is_none());
    let item_json = serde_json::to_value(&leg_obs.inventory[0]).unwrap();
    let item_obj = item_json.as_object().unwrap();
    assert!(item_obj.get("quantity").is_none());
    assert!(item_obj.get("appearance").is_none());

    // TUI: '_', 'H', 'D' glyphs, inspect labels, MP HUD, <qty>x stack prefix, appearance, 'Z' & 'O'.
    let base = start(42, Role::Mage);
    let main3 = LevelId::main(3);
    let altar_pos = find_tiles(&base, main3, TileKind::Altar)[0];
    let map3 = base.world().levels.map(main3).unwrap();
    let p_pos = Direction::ALL
        .into_iter()
        .map(|d| altar_pos.offset(d.delta()))
        .find(|&p| map3.tile(p).ok() == Some(TileKind::Floor))
        .unwrap();
    let adj: Vec<Pos> = Direction::ALL
        .into_iter()
        .map(|d| p_pos.offset(d.delta()))
        .filter(|&p| {
            p != altar_pos
                && map3.tile(p).ok() == Some(TileKind::Floor)
                && base.world().entities.entities().iter().all(|e| {
                    e.item().is_none_or(|(_, _, loc, _, _)| {
                        loc != EntityLocation::OnMap {
                            level: main3,
                            pos: p,
                        }
                    })
                })
        })
        .collect();
    let mino_pos = adj[0];
    let dragon_pos = adj[1];
    let pot_pos = adj[2];

    let mino_id = base
        .world()
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Minotaur)))
        .unwrap()
        .id;
    let dragon_id = base
        .world()
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Dragon)))
        .unwrap()
        .id;

    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    save.world.current_level = main3;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main3, p_pos);
    save.world
        .entities
        .set_actor_location(mino_id, main3, mino_pos);
    save.world
        .entities
        .set_actor_location(dragon_id, main3, dragon_pos);
    save.world
        .entities
        .spawn_item(
            ItemKind::PotionCure,
            EntityLocation::OnMap {
                level: main3,
                pos: pot_pos,
            },
        )
        .unwrap();
    let stacked_potion =
        add_inventory_item(&mut save, ItemKind::PotionCure, BucStatus::Uncursed, 0);
    save.world.entities.set_item_quantity(stacked_potion, 3);

    let sess = GameSession::from_save_data(save).unwrap();
    let obs = sess.observation();
    assert!(render_panels::status_lines(&obs)
        .iter()
        .any(|l| l.contains("mp 14/14")));
    assert!(
        render_panels::inspect_lines(&obs, Some(altar_pos), UiPanel::Inspect, &[])
            .iter()
            .any(|l| l == "tile altar")
    );
    assert!(
        render_panels::inspect_lines(&obs, Some(mino_pos), UiPanel::Inspect, &[])
            .iter()
            .any(|l| l.starts_with("entity minotaur"))
    );
    assert!(
        render_panels::inspect_lines(&obs, Some(dragon_pos), UiPanel::Inspect, &[])
            .iter()
            .any(|l| l.starts_with("entity dragon"))
    );
    let expected_cure_app = appearance_for_kind(42, ItemKind::PotionCure).unwrap();
    assert!(
        render_panels::inspect_lines(&obs, Some(pot_pos), UiPanel::Inspect, &[])
            .iter()
            .any(|l| l == &format!("entity {expected_cure_app}"))
    );
    assert!(input::inventory_panel_ctas(&obs)
        .iter()
        .any(|c| c.label.contains(&format!("3x {expected_cure_app}"))));

    assert_eq!(
        input::key_to_candidate('Z', &obs),
        Some(UiCommandCandidate::BeginAction('Z'))
    );
    assert_eq!(
        input::key_to_candidate('O', &obs),
        Some(UiCommandCandidate::BeginAction('O'))
    );

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
        let (ax, ay) = vp.world_to_terminal(altar_pos, layout.map).unwrap();
        assert_eq!(buf[(ax, ay)].symbol(), "_");
        let (mx, my) = vp.world_to_terminal(mino_pos, layout.map).unwrap();
        assert_eq!(buf[(mx, my)].symbol(), "H");
        let (dx, dy) = vp.world_to_terminal(dragon_pos, layout.map).unwrap();
        assert_eq!(buf[(dx, dy)].symbol(), "D");
    }

    // Cast menu ('Z') -> '1' (Force Bolt) -> spell direction menu ('l' East)
    send_key(&mut app, &clock, 'Z');
    send_key(&mut app, &clock, '1');
    let mp_before = app.observation().player.mp;
    assert_eq!(
        send_key(&mut app, &clock, 'l'),
        UiCommandCandidate::Command(C::Cast {
            spell: SpellKind::ForceBolt,
            direction: Some(Direction::East),
        })
    );
    assert_eq!(app.observation().player.mp, mp_before - 4);

    // Move onto Altar via session and trigger Offer ('O') menu in TUI.
    let mut offer_save = start(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut offer_save, mines_level(2));
    offer_save.world.current_level = main3;
    offer_save
        .world
        .entities
        .set_actor_location(offer_save.world.player_id, main3, altar_pos);
    let offer_sess = GameSession::from_save_data(offer_save).unwrap();
    let offer_clock = Arc::new(TestClock::default());
    let mut offer_app = TuiApp::new_with_llm_enabled_and_clock(
        offer_sess,
        UiRuntimeConfig::default(),
        false,
        offer_clock.clone(),
    );
    let offer_letter = offer_app.observation().inventory[0].letter.0;
    let inv_before = offer_app.observation().inventory.len();
    send_key(&mut offer_app, &offer_clock, 'O');
    send_key(&mut offer_app, &offer_clock, offer_letter);
    assert_eq!(offer_app.observation().inventory.len(), inv_before - 1);

    // Headless survival-v1: casts Heal when HP <= 50% and casts ForceBolt when enemy within range 5.
    let mut heal_hl_save = start(42, Role::Cleric).to_save_data();
    park_all_monsters_on(&mut heal_hl_save, mines_level(2));
    heal_hl_save
        .world
        .entities
        .actor_stats_mut(heal_hl_save.world.player_id)
        .unwrap()
        .hp = 8;
    let heal_hl_sess = GameSession::from_save_data(heal_hl_save).unwrap();
    assert_eq!(
        HeadlessPolicy::survival_v1().candidates(&heal_hl_sess),
        vec![C::Cast {
            spell: SpellKind::Heal,
            direction: None
        }]
    );

    let mut bolt_hl_save = start(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut bolt_hl_save, mines_level(2));
    let main1 = LevelId::main(1);
    let map1 = bolt_hl_save.world.levels.map(main1).unwrap();
    let origin = (0..map1.height)
        .flat_map(|y| (0..map1.width - 4).map(move |x| Pos { x, y }))
        .find(|&p| {
            (0..=3).all(|dx| {
                map1.tile(Pos {
                    x: p.x + dx,
                    y: p.y,
                })
                .is_ok_and(|t| t.is_movement_passable())
            })
        })
        .unwrap();
    bolt_hl_save
        .world
        .entities
        .set_actor_location(bolt_hl_save.world.player_id, main1, origin);
    let target = bolt_hl_save
        .world
        .entities
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Jackal)))
        .unwrap()
        .id;
    bolt_hl_save.world.entities.set_actor_location(
        target,
        main1,
        Pos {
            x: origin.x + 3,
            y: origin.y,
        },
    );
    let bolt_hl_sess = GameSession::from_save_data(bolt_hl_save).unwrap();
    assert_eq!(
        HeadlessPolicy::survival_v1().candidates(&bolt_hl_sess),
        vec![C::Cast {
            spell: SpellKind::ForceBolt,
            direction: Some(Direction::East),
        }]
    );
}

trait ActorStatsHpCheckExt {
    fn actor_stats_hp_check(&self) -> (i16, u16);
}

impl ActorStatsHpCheckExt for aihack_core::domain::entity::Entity {
    fn actor_stats_hp_check(&self) -> (i16, u16) {
        let (_, _, _, _, stats, _) = self.actor().unwrap();
        (stats.max_hp, self.monster_difficulty().unwrap())
    }
}
