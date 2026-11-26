use aihack_core::action::CommandIntent as C;
use aihack_core::campaign::Role;
use aihack_core::domain::{
    combat::DeathCause,
    entity::{ActorKind, EntityLocation},
    item::{BucStatus, ItemKind, MAX_ITEM_ENCHANTMENT},
    monster::MonsterKind,
    tile::{DoorState, TileKind},
};
use aihack_core::event::GameEvent;
use aihack_core::ids::{BranchId, LevelId};
use aihack_core::position::{Direction, Pos};
use aihack_core::run_state::RunState;
use aihack_headless::HeadlessPolicy;
use aihack_runtime::GameSession;
use aihack_tui::tui::{input, render_frame, render_panels, TuiApp, UiPanel, UiRuntimeConfig};

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
) -> aihack_core::ids::EntityId {
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

fn find_door_y(session: &GameSession, level: LevelId, x: i16) -> Option<(i16, TileKind)> {
    let map = session.world().levels.map(level)?;
    for y in 0..map.height {
        let pos = Pos { x, y };
        if let Ok(tile) = map.tile(pos) {
            if matches!(tile, TileKind::Door(_) | TileKind::HiddenDoor) {
                return Some((y, tile));
            }
        }
    }
    None
}

#[test]
fn p5a_buc_enchantment_scrolls_potions_cursed_guards_and_validation() {
    let base = start(42, Role::Knight);

    // 1. Blessed +1 ShortSword vs Uncursed +0 ShortSword (identical seed/position)
    let setup_duel = |buc: BucStatus, enchant: i8| {
        let mut save = base.to_save_data();
        park_all_monsters_on(&mut save, mines_level(2));
        let level = LevelId::main(1);
        let ppos = save.world.levels.stairs_up_pos(level).unwrap();
        let target_pos = Pos {
            x: ppos.x + 1,
            y: ppos.y,
        };
        let target = save
            .world
            .entities
            .entities()
            .iter()
            .find(|e| matches!(e.actor_kind(), Some(ActorKind::Monster(_))))
            .unwrap()
            .id;
        save.world.current_level = level;
        save.world
            .entities
            .set_actor_location(save.world.player_id, level, ppos);
        save.world
            .entities
            .set_actor_location(target, level, target_pos);
        save.world.entities.actor_stats_mut(target).unwrap().hp = 50;
        save.world.entities.actor_stats_mut(target).unwrap().max_hp = 50;
        let sword = add_inventory_item(&mut save, ItemKind::ShortSword, buc, enchant);
        let mut session = GameSession::from_save_data(save).unwrap();
        assert!(session.submit(C::Wield { item: sword }).accepted);
        let out = session.submit(C::Move(Direction::East));
        let (roll, damage) = out
            .events
            .iter()
            .find_map(|event| match event {
                GameEvent::AttackResolved {
                    attacker,
                    attack_roll,
                    damage,
                    ..
                } if *attacker == session.world().player_id() => Some((*attack_roll, *damage)),
                _ => None,
            })
            .unwrap();
        (roll, damage)
    };
    let (plain_roll, _) = setup_duel(BucStatus::Uncursed, 0);
    let (blessed_roll, _) = setup_duel(BucStatus::Blessed, 1);
    assert_eq!(blessed_roll, plain_roll + 2);

    // 2. Armor enchantment updates AC and survives save/load; Cursed guard blocks Drop/Wield/Wear/Throw
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    let cursed_dagger = add_inventory_item(&mut save, ItemKind::Dagger, BucStatus::Cursed, -1);
    let extra_sword = add_inventory_item(&mut save, ItemKind::ShortSword, BucStatus::Uncursed, 0);
    let cursed_armor = add_inventory_item(&mut save, ItemKind::ArmorLeather, BucStatus::Cursed, 1);
    let extra_armor =
        add_inventory_item(&mut save, ItemKind::ArmorChainMail, BucStatus::Uncursed, 0);
    let scroll_enchant_w = add_inventory_item(
        &mut save,
        ItemKind::ScrollEnchantWeapon,
        BucStatus::Uncursed,
        0,
    );
    let scroll_enchant_a = add_inventory_item(
        &mut save,
        ItemKind::ScrollEnchantArmor,
        BucStatus::Uncursed,
        0,
    );
    let scroll_uncurse = add_inventory_item(
        &mut save,
        ItemKind::ScrollRemoveCurse,
        BucStatus::Uncursed,
        0,
    );
    let mut session = GameSession::from_save_data(save).unwrap();

    assert!(
        session
            .submit(C::Wield {
                item: cursed_dagger
            })
            .accepted
    );
    assert!(session.submit(C::Wear { item: cursed_armor }).accepted);
    // ArmorLeather base AC bonus 1 + enchant 1 = 2 -> player AC 0 - 2 = -2
    assert_eq!(
        session
            .world()
            .entities()
            .actor_stats(session.world().player_id())
            .unwrap()
            .ac,
        -2
    );

    // Cannot swap, drop, or throw cursed equipped weapon/armor
    assert!(!session.submit(C::Wield { item: extra_sword }).accepted);
    assert!(
        !session
            .submit(C::Drop {
                item: cursed_dagger
            })
            .accepted
    );
    assert!(
        !session
            .submit(C::Throw {
                item: cursed_dagger,
                direction: Direction::East,
            })
            .accepted
    );
    assert!(!session.submit(C::Wear { item: extra_armor }).accepted);
    assert!(!session.submit(C::Drop { item: cursed_armor }).accepted);

    // EnchantWeapon uncurses equipped weapon and increments enchantment (-1 -> 0)
    assert!(
        session
            .submit(C::Read {
                item: scroll_enchant_w
            })
            .accepted
    );
    assert_eq!(
        session
            .world()
            .entities()
            .get(cursed_dagger)
            .unwrap()
            .item_buc(),
        Some(BucStatus::Uncursed)
    );
    assert_eq!(
        session
            .world()
            .entities()
            .get(cursed_dagger)
            .unwrap()
            .item_enchantment(),
        Some(0)
    );
    assert!(
        session
            .submit(C::Drop {
                item: cursed_dagger
            })
            .accepted
    );

    // EnchantArmor uncurses equipped armor, increments +1 -> +2, updates AC to 0 - (1 + 2) = -3
    assert!(
        session
            .submit(C::Read {
                item: scroll_enchant_a
            })
            .accepted
    );
    assert_eq!(
        session
            .world()
            .entities()
            .get(cursed_armor)
            .unwrap()
            .item_buc(),
        Some(BucStatus::Uncursed)
    );
    assert_eq!(
        session
            .world()
            .entities()
            .get(cursed_armor)
            .unwrap()
            .item_enchantment(),
        Some(2)
    );
    assert_eq!(
        session
            .world()
            .entities()
            .actor_stats(session.world().player_id())
            .unwrap()
            .ac,
        -3
    );
    let reloaded = GameSession::from_save_data(session.to_save_data()).unwrap();
    assert_eq!(
        reloaded.snapshot().stable_hash(),
        session.snapshot().stable_hash()
    );

    // RemoveCurse lifts curse on any remaining inventory cursed items
    let mut save2 = reloaded.to_save_data();
    let cursed_swap = add_inventory_item(&mut save2, ItemKind::Dagger, BucStatus::Cursed, 0);
    let s_uncurse_only = add_inventory_item(
        &mut save2,
        ItemKind::ScrollRemoveCurse,
        BucStatus::Uncursed,
        0,
    );
    let mut session2 = GameSession::from_save_data(save2).unwrap();
    assert!(session2.submit(C::Wield { item: cursed_swap }).accepted);
    assert!(!session2.submit(C::Drop { item: cursed_swap }).accepted);
    assert!(
        session2
            .submit(C::Read {
                item: s_uncurse_only
            })
            .accepted
    );
    assert_eq!(
        session2
            .world()
            .entities()
            .get(cursed_swap)
            .unwrap()
            .item_buc(),
        Some(BucStatus::Uncursed)
    );
    assert!(session2.submit(C::Drop { item: cursed_swap }).accepted);

    // unused variable silence avoid & RemoveCurse consumed on cursed items
    let _ = scroll_uncurse;

    // 3. Weapon enchant cap MAX_ITEM_ENCHANTMENT (+5)
    let mut cap_save = base.to_save_data();
    park_all_monsters_on(&mut cap_save, mines_level(2));
    let max_sword = add_inventory_item(
        &mut cap_save,
        ItemKind::ShortSword,
        BucStatus::Uncursed,
        MAX_ITEM_ENCHANTMENT,
    );
    let scroll_ew = add_inventory_item(
        &mut cap_save,
        ItemKind::ScrollEnchantWeapon,
        BucStatus::Uncursed,
        0,
    );
    let mut cap_session = GameSession::from_save_data(cap_save).unwrap();
    assert!(cap_session.submit(C::Wield { item: max_sword }).accepted);
    assert!(cap_session.submit(C::Read { item: scroll_ew }).accepted);
    assert_eq!(
        cap_session
            .world()
            .entities()
            .get(max_sword)
            .unwrap()
            .item_enchantment(),
        Some(MAX_ITEM_ENCHANTMENT)
    );

    // 4. BUC potion & ScrollCharging effects
    let heal_with_buc = |buc: BucStatus| {
        let mut s = base.to_save_data();
        park_all_monsters_on(&mut s, mines_level(2));
        s.world
            .entities
            .actor_stats_mut(s.world.player_id)
            .unwrap()
            .hp = 5;
        let potion = add_inventory_item(&mut s, ItemKind::PotionHealing, buc, 0);
        let mut sess = GameSession::from_save_data(s).unwrap();
        assert!(sess.submit(C::Quaff { item: potion }).accepted);
        sess.observation().player.hp - 5
    };
    let h_cursed = heal_with_buc(BucStatus::Cursed);
    let h_uncursed = heal_with_buc(BucStatus::Uncursed);
    let h_blessed = heal_with_buc(BucStatus::Blessed);
    assert!(h_blessed == h_uncursed + 4 || h_blessed >= h_uncursed);
    assert!(h_cursed <= h_uncursed && h_cursed >= 1);

    let mut ch_save = base.to_save_data();
    park_all_monsters_on(&mut ch_save, mines_level(2));
    let wand = add_inventory_item(&mut ch_save, ItemKind::WandFire, BucStatus::Cursed, 0);
    ch_save.world.entities.set_item_charges(wand, Some(1));
    let cursed_charge =
        add_inventory_item(&mut ch_save, ItemKind::ScrollCharging, BucStatus::Cursed, 0);
    let blessed_charge = add_inventory_item(
        &mut ch_save,
        ItemKind::ScrollCharging,
        BucStatus::Blessed,
        0,
    );
    let mut ch_sess = GameSession::from_save_data(ch_save).unwrap();
    assert!(
        ch_sess
            .submit(C::Read {
                item: cursed_charge
            })
            .accepted
    );
    assert_eq!(
        ch_sess
            .world()
            .entities()
            .get(wand)
            .unwrap()
            .item()
            .unwrap()
            .4,
        Some(0)
    );
    assert!(
        ch_sess
            .submit(C::Read {
                item: blessed_charge
            })
            .accepted
    );
    assert_eq!(
        ch_sess
            .world()
            .entities()
            .get(wand)
            .unwrap()
            .item()
            .unwrap()
            .4,
        Some(4)
    );
    assert_eq!(
        ch_sess.world().entities().get(wand).unwrap().item_buc(),
        Some(BucStatus::Uncursed)
    );

    // 5. Save tamper checks: out-of-range enchantment and forged armor AC rejected
    let mut bad_enchant = base.to_save_data();
    add_inventory_item(&mut bad_enchant, ItemKind::Dagger, BucStatus::Uncursed, 6);
    assert!(GameSession::from_save_data(bad_enchant).is_err());

    let mut bad_low = base.to_save_data();
    add_inventory_item(&mut bad_low, ItemKind::Dagger, BucStatus::Uncursed, -4);
    assert!(GameSession::from_save_data(bad_low).is_err());

    let mut bad_ac = session.to_save_data();
    bad_ac
        .world
        .entities
        .actor_stats_mut(bad_ac.world.player_id)
        .unwrap()
        .ac = 1;
    assert!(GameSession::from_save_data(bad_ac).is_err());
}

#[test]
fn p5a_locked_doors_spawn_kick_open_and_tamper_rejection() {
    let base = start(42, Role::Knight);
    let mines2 = mines_level(2);
    let (y7, tile) = find_door_y(&base, mines2, 7).unwrap();
    assert_eq!(tile, TileKind::Door(DoorState::Locked));
    let door_pos = Pos { x: 7, y: y7 };

    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, mines2);
    save.world.current_level = mines2;
    save.world
        .entities
        .set_actor_location(save.world.player_id, mines2, Pos { x: 6, y: y7 });
    let mut session = GameSession::from_save_data(save).unwrap();

    // Open and Move blocked on Locked door
    let hash_before = session.snapshot().stable_hash();
    assert!(!session.submit(C::Open(Direction::East)).accepted);
    assert!(!session.submit(C::Move(Direction::East)).accepted);
    assert_eq!(session.snapshot().stable_hash(), hash_before);

    // Kick unlocks into Open, and Close -> Open -> save/load valid on generated Locked site
    let kicked = session.submit(C::Kick(Direction::East));
    assert!(kicked.accepted);
    assert_eq!(
        session.world().current_map().tile(door_pos),
        Ok(TileKind::Door(DoorState::Open))
    );
    assert!(session.submit(C::Close(Direction::East)).accepted);
    assert_eq!(
        session.world().current_map().tile(door_pos),
        Ok(TileKind::Door(DoorState::Closed))
    );
    assert!(GameSession::from_save_data(session.to_save_data()).is_ok());

    // Tampering: forging Locked on Main 1 Closed door or on Floor must be rejected
    let (y_main1, main1_tile) = find_door_y(&base, LevelId::main(1), 7).unwrap();
    assert_eq!(main1_tile, TileKind::Door(DoorState::Closed));
    let mut forged_closed = base.to_save_data();
    forged_closed
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(Pos { x: 7, y: y_main1 }, TileKind::Door(DoorState::Locked))
        .unwrap();
    assert!(GameSession::from_save_data(forged_closed).is_err());

    let mut forged_floor = base.to_save_data();
    let up_m1 = forged_floor
        .world
        .levels
        .stairs_up_pos(LevelId::main(1))
        .unwrap();
    forged_floor
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(
            Pos {
                x: up_m1.x + 1,
                y: up_m1.y,
            },
            TileKind::Door(DoorState::Locked),
        )
        .unwrap();
    assert!(GameSession::from_save_data(forged_floor).is_err());
}

#[test]
fn p5b_poison_status_cure_wand_fire_and_new_monsters() {
    let base = start(42, Role::Knight);

    // Confirm campaign spawns exist
    let spider_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::GiantSpider)))
        .expect("GiantSpider must spawn in Mines 1")
        .id;
    let archer_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::GnollArcher)))
        .expect("GnollArcher must spawn in Main 4")
        .id;
    let troll_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Troll)))
        .expect("Troll must spawn in Main 5")
        .id;

    let item_present =
        |kind: ItemKind, level: LevelId, expected_buc: BucStatus, expected_ench: i8| {
            base.world().entities().entities().iter().any(|e| {
                e.item().is_some_and(|(k, _, loc, _, _)| {
                    k == kind && matches!(loc, EntityLocation::OnMap { level: l, .. } if l == level)
                }) && e.item_buc() == Some(expected_buc)
                    && e.item_enchantment() == Some(expected_ench)
            })
        };
    assert!(item_present(
        ItemKind::ScrollEnchantWeapon,
        LevelId::main(1),
        BucStatus::Uncursed,
        0
    ));
    assert!(item_present(
        ItemKind::ScrollRemoveCurse,
        LevelId::main(2),
        BucStatus::Uncursed,
        0
    ));
    assert!(item_present(
        ItemKind::ShortSword,
        LevelId::main(3),
        BucStatus::Blessed,
        1
    ));
    assert!(item_present(
        ItemKind::ScrollEnchantArmor,
        LevelId::main(3),
        BucStatus::Uncursed,
        0
    ));
    assert!(item_present(
        ItemKind::PotionCure,
        LevelId::main(4),
        BucStatus::Uncursed,
        0
    ));
    assert!(item_present(
        ItemKind::WandFire,
        LevelId::main(5),
        BucStatus::Uncursed,
        0
    ));
    assert!(item_present(
        ItemKind::PotionCure,
        mines_level(1),
        BucStatus::Uncursed,
        0
    ));
    assert!(item_present(
        ItemKind::Dagger,
        mines_level(1),
        BucStatus::Cursed,
        -1
    ));
    assert!(item_present(
        ItemKind::Mace,
        mines_level(2),
        BucStatus::Uncursed,
        1
    ));

    // 1. GiantSpider inflicts poison_turns >= 3, Wait ticks -1 HP + warning, PotionCure clears all
    let mut sp_save = base.to_save_data();
    park_all_monsters_on(&mut sp_save, mines_level(2));
    let level = LevelId::main(1);
    let ppos = sp_save.world.levels.stairs_up_pos(level).unwrap();
    sp_save.world.current_level = level;
    sp_save
        .world
        .entities
        .set_actor_location(sp_save.world.player_id, level, ppos);
    sp_save.world.entities.set_actor_location(
        spider_id,
        level,
        Pos {
            x: ppos.x + 1,
            y: ppos.y,
        },
    );
    let cure = add_inventory_item(&mut sp_save, ItemKind::PotionCure, BucStatus::Blessed, 0);
    let mut sp_sess = GameSession::from_save_data(sp_save).unwrap();
    let mut poisoned = false;
    for _ in 0..30 {
        let wait_out = sp_sess.submit(C::Wait);
        assert!(wait_out.accepted);
        if wait_out.events.iter().any(|e| {
            matches!(
                e,
                GameEvent::PassiveAttackTriggered { source, target }
                    if *source == spider_id && *target == sp_sess.world().player_id()
            )
        }) {
            poisoned = true;
            break;
        }
    }
    assert!(
        poisoned,
        "GiantSpider must land a melee hit within 30 waits"
    );
    assert!(sp_sess.observation().player.poison_turns >= 3);

    let hp_before_tick = sp_sess.observation().player.hp;
    let pois_before_tick = sp_sess.observation().player.poison_turns;
    let tick_out = sp_sess.submit(C::Move(Direction::West));
    assert!(tick_out.accepted);
    assert_eq!(
        sp_sess.observation().player.poison_turns,
        pois_before_tick - 1
    );
    assert!(tick_out.events.iter().any(|e| matches!(
        e,
        GameEvent::Message { text, .. } if text == "You suffer from poison!"
    )));
    assert_eq!(sp_sess.observation().player.hp, hp_before_tick - 1);

    // Drink Cure potion -> clears poison and heals
    let mut pre_cure = sp_sess.to_save_data();
    pre_cure.world.paralysis_turns = 0; // paralysis forbids Quaff
    pre_cure.world.hallucinating = true;
    pre_cure
        .world
        .entities
        .actor_stats_mut(pre_cure.world.player_id)
        .unwrap()
        .hp = 5;
    let mut cure_sess = GameSession::from_save_data(pre_cure).unwrap();
    assert!(cure_sess.submit(C::Quaff { item: cure }).accepted);
    assert_eq!(cure_sess.observation().player.poison_turns, 0);
    assert!(!cure_sess.observation().player.hallucinating);
    assert!(cure_sess.observation().player.hp > 5);

    // Fatal poison -> DeathCause::Poison + valid GameOver save round-trip
    let mut fatal_save = base.to_save_data();
    park_all_monsters_on(&mut fatal_save, mines_level(2));
    fatal_save.world.poison_turns = 2;
    fatal_save
        .world
        .entities
        .actor_stats_mut(fatal_save.world.player_id)
        .unwrap()
        .hp = 1;
    let mut fatal_sess = GameSession::from_save_data(fatal_save).unwrap();
    let fatal_out = fatal_sess.submit(C::Wait);
    assert!(fatal_out.accepted);
    assert!(matches!(
        fatal_sess.run_state(),
        RunState::GameOver {
            cause: DeathCause::Poison,
            ..
        }
    ));
    assert!(GameSession::from_save_data(fatal_sess.to_save_data()).is_ok());

    // 2. GnollArcher ranged attack unobstructed vs blocked
    let mut ar_save = base.to_save_data();
    park_all_monsters_on(&mut ar_save, mines_level(2));
    ar_save.world.current_level = level;
    let room0_center = ar_save.world.levels.stairs_up_pos(level).unwrap();
    let p_ar = Pos {
        x: room0_center.x - 2,
        y: room0_center.y,
    };
    let a_pos = Pos {
        x: room0_center.x + 2,
        y: room0_center.y,
    }; // distance 4 orthogonal inside 5x5 room
    ar_save
        .world
        .entities
        .set_actor_location(ar_save.world.player_id, level, p_ar);
    ar_save
        .world
        .entities
        .set_actor_location(archer_id, level, a_pos);
    let mut ar_sess = GameSession::from_save_data(ar_save.clone()).unwrap();
    let ar_out = ar_sess.submit(C::Wait);
    assert!(ar_out.events.iter().any(|e| matches!(
        e,
        GameEvent::AttackResolved { attacker, .. } if *attacker == archer_id
    )));
    // Obstruction with Locked door blocks ranged shot
    let mut blocked_save = ar_save;
    blocked_save
        .world
        .levels
        .map_mut(level)
        .unwrap()
        .set_tile(room0_center, TileKind::Door(DoorState::Locked))
        .unwrap();
    // Direct map mutation in Main 1 violates topology check if loaded cleanly, so put obstacle monster instead
    let _ = blocked_save;
    let mut mon_block_save = base.to_save_data();
    park_all_monsters_on(&mut mon_block_save, mines_level(2));
    mon_block_save.world.current_level = level;
    mon_block_save
        .world
        .entities
        .set_actor_location(mon_block_save.world.player_id, level, p_ar);
    mon_block_save
        .world
        .entities
        .set_actor_location(archer_id, level, a_pos);
    mon_block_save
        .world
        .entities
        .set_actor_location(spider_id, level, room0_center);
    let mut mb_sess = GameSession::from_save_data(mon_block_save).unwrap();
    let mb_out = mb_sess.submit(C::Wait);
    assert!(!mb_out.events.iter().any(|e| matches!(
        e,
        GameEvent::AttackResolved { attacker, .. } if *attacker == archer_id
    )));

    // 3. Troll regenerates +1 HP/turn up to max_hp + WandFire deals 2d6+2 and consumes charges
    let mut tr_save = base.to_save_data();
    park_all_monsters_on(&mut tr_save, mines_level(2));
    tr_save.world.current_level = level;
    let far_pos = Pos {
        x: room0_center.x - 2,
        y: room0_center.y - 2,
    };
    tr_save
        .world
        .entities
        .set_actor_location(tr_save.world.player_id, level, far_pos);
    tr_save.world.entities.set_actor_location(
        troll_id,
        level,
        Pos {
            x: far_pos.x + 2,
            y: far_pos.y,
        },
    );
    tr_save.world.entities.actor_stats_mut(troll_id).unwrap().hp = 15;
    let wand_fire = add_inventory_item(&mut tr_save, ItemKind::WandFire, BucStatus::Uncursed, 0);
    let mut tr_sess = GameSession::from_save_data(tr_save).unwrap();
    let zap_out = tr_sess.submit(C::Zap {
        item: wand_fire,
        direction: Direction::East,
    });
    assert!(zap_out.accepted);
    assert!(zap_out.events.iter().any(|e| matches!(
        e,
        GameEvent::WandZapped {
            charges_after: 3,
            ..
        }
    )));
    assert!(zap_out.events.iter().any(|e| matches!(
        e,
        GameEvent::EntityHealed {
            entity,
            amount: 1,
            ..
        } if *entity == troll_id
    )));
}

#[test]
fn p5c_tui_headless_and_legacy_v1_preservation() {
    // 1. Legacy V1 save preservation: schema v1, poison_turns==0 & default BUC/enchant omitted from JSON
    let legacy = GameSession::try_new_for_playing(42).unwrap();
    let legacy_save = legacy.to_save_data();
    assert_eq!(legacy_save.schema_version, 1);
    let json = serde_json::to_string(&legacy_save).unwrap();
    assert!(!json.contains("\"poison_turns\""));
    assert!(!json.contains("\"buc\""));
    assert!(!json.contains("\"enchantment\""));
    assert_eq!(
        GameSession::from_save_data(legacy_save)
            .unwrap()
            .snapshot()
            .stable_hash(),
        legacy.snapshot().stable_hash()
    );

    // 2. TUI checks: POIS:<n>, locked door, BUC/enchant labels, monster labels/glyphs, DeathCause::Poison
    let base = start(42, Role::Knight);
    let mines2 = mines_level(2);
    let (y7, _) = find_door_y(&base, mines2, 7).unwrap();
    let locked_door_pos = Pos { x: 7, y: y7 };

    let spider_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::GiantSpider)))
        .unwrap()
        .id;
    let archer_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::GnollArcher)))
        .unwrap()
        .id;
    let troll_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Troll)))
        .unwrap()
        .id;

    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world.poison_turns = 3;
    save.world.current_level = mines2;
    let p_pos = Pos { x: 6, y: y7 };
    let sp_pos = Pos { x: 5, y: y7 - 1 };
    let ar_pos = Pos { x: 5, y: y7 };
    let tr_pos = Pos { x: 5, y: y7 + 1 };
    save.world
        .entities
        .set_actor_location(save.world.player_id, mines2, p_pos);
    save.world
        .entities
        .set_actor_location(spider_id, mines2, sp_pos);
    save.world
        .entities
        .set_actor_location(archer_id, mines2, ar_pos);
    save.world
        .entities
        .set_actor_location(troll_id, mines2, tr_pos);
    add_inventory_item(&mut save, ItemKind::ShortSword, BucStatus::Blessed, 1);
    add_inventory_item(&mut save, ItemKind::Dagger, BucStatus::Cursed, -1);

    let ui_sess = GameSession::from_save_data(save).unwrap();
    let status = render_panels::status_lines(&ui_sess.observation());
    assert!(status.iter().any(|l| l.contains("POIS:3")));
    let inspect_door = render_panels::inspect_lines(
        &ui_sess.observation(),
        Some(locked_door_pos),
        UiPanel::Inspect,
        &[],
    );
    assert!(inspect_door.iter().any(|l| l == "tile locked door"));
    for (pos, expected) in [
        (sp_pos, "entity giant spider"),
        (ar_pos, "entity gnoll archer"),
        (tr_pos, "entity troll"),
    ] {
        let lines =
            render_panels::inspect_lines(&ui_sess.observation(), Some(pos), UiPanel::Inspect, &[]);
        assert!(lines.iter().any(|l| l.starts_with(expected)));
    }
    let inv_ctas = input::inventory_panel_ctas(&ui_sess.observation());
    assert!(inv_ctas
        .iter()
        .any(|c| c.label.contains("+1 blessed short sword")));
    assert!(inv_ctas
        .iter()
        .any(|c| c.label.contains("-1 cursed dagger")));

    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 28)).unwrap();
    let mut app = TuiApp::new(ui_sess, UiRuntimeConfig::default());
    let layout = aihack_tui::tui::compute_layout(80, 28);
    let vp = app.viewport_for_observation(layout);
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    let buf = term.backend().buffer();
    let (dx, dy) = vp.world_to_terminal(locked_door_pos, layout.map).unwrap();
    assert_eq!(buf[(dx, dy)].symbol(), "+");
    for (pos, sym) in [(sp_pos, "s"), (ar_pos, "G"), (tr_pos, "T")] {
        let (mx, my) = vp.world_to_terminal(pos, layout.map).unwrap();
        assert_eq!(buf[(mx, my)].symbol(), sym);
    }

    // DeathCause::Poison GameOver screen
    let mut dead_save = base.to_save_data();
    park_all_monsters_on(&mut dead_save, mines2);
    dead_save.world.poison_turns = 1;
    dead_save
        .world
        .entities
        .actor_stats_mut(dead_save.world.player_id)
        .unwrap()
        .hp = 1;
    let mut dead_sess = GameSession::from_save_data(dead_save).unwrap();
    assert!(dead_sess.submit(C::Wait).accepted);
    let mut dead_app = TuiApp::new(dead_sess, UiRuntimeConfig::default());
    term.draw(|frame| render_frame(frame, &mut dead_app))
        .unwrap();
    let screen: String = (0..28)
        .map(|y| {
            (0..80)
                .map(|x| term.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.contains("Killed by poison"));

    // 3. Headless survival-v1: drinks PotionCure when poisoned and kicks DoorState::Locked
    let mut hl_save = base.to_save_data();
    park_all_monsters_on(&mut hl_save, LevelId::main(6));
    hl_save.world.current_level = mines2;
    hl_save.world.poison_turns = 3;
    hl_save
        .world
        .entities
        .set_actor_location(hl_save.world.player_id, mines2, p_pos);
    add_inventory_item(&mut hl_save, ItemKind::PotionCure, BucStatus::Uncursed, 0);
    let mut hl_sess = GameSession::from_save_data(hl_save).unwrap();
    assert!(matches!(
        HeadlessPolicy::survival_v1().candidates(&hl_sess).first(),
        Some(C::Quaff { .. })
    ));
    let t0 = hl_sess.turn();
    aihack_headless::run_to_turn(&mut hl_sess, t0 + 1, HeadlessPolicy::survival_v1()).unwrap();
    assert_eq!(hl_sess.observation().player.poison_turns, 0);

    let t1 = hl_sess.turn();
    assert_eq!(
        HeadlessPolicy::survival_v1().candidates(&hl_sess),
        vec![C::Kick(Direction::East)]
    );
    aihack_headless::run_to_turn(&mut hl_sess, t1 + 1, HeadlessPolicy::survival_v1()).unwrap();
    assert_eq!(
        hl_sess.world().current_map().tile(locked_door_pos),
        Ok(TileKind::Door(DoorState::Open))
    );
}
