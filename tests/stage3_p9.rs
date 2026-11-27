use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use aihack_core::{
    action::{CommandIntent as C, DipTarget, EngravingMethod, EngravingText},
    campaign::Role,
    domain::{
        combat::DeathCause,
        entity::{ActorKind, ActorStats, EntityKind, EntityLocation, Faction},
        item::{BucStatus, ItemKind},
        monster::MonsterKind,
        status::Resistances,
        tile::{DoorState, TileKind, TrapKind},
    },
    event::GameEvent,
    ids::{BranchId, EntityId, LevelId},
    position::{Direction, Pos},
    run_state::RunState,
    save::{SAVE_SCHEMA_VERSION_V1, SAVE_SCHEMA_VERSION_V3},
};
use aihack_headless::{run_to_turn, HeadlessPolicy};
use aihack_llm::{service::LlmObservationView, transport::REQUEST_BODY_LIMIT};
use aihack_runtime::GameSession;
use aihack_tui::tui::{
    render_frame, render_panels, runtime_event_to_candidate, TuiApp, UiClock, UiCommandCandidate,
    UiRuntimeConfig,
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

fn find_floor_with_neighbors(
    session: &GameSession,
    level_id: LevelId,
    count: usize,
) -> (Pos, Vec<Pos>) {
    let level = session.world().levels.get(level_id).unwrap();
    for y in 2..level.map.height - 2 {
        for x in 2..level.map.width - 2 {
            let origin = Pos { x, y };
            if level.map.tile(origin).ok() != Some(TileKind::Floor) {
                continue;
            }
            let mut found = Vec::new();
            for dir in Direction::ALL {
                let p = origin.offset(dir.delta());
                if level.map.tile(p).ok() == Some(TileKind::Floor) {
                    found.push(p);
                }
            }
            if found.len() >= count {
                return (origin, found);
            }
        }
    }
    panic!("failed to find floor with {count} neighbors on {level_id:?}");
}

fn find_tile(
    save: &aihack_runtime::save::SaveDataV1,
    level_id: LevelId,
    pred: impl Fn(TileKind) -> bool,
) -> Pos {
    let map = save.world.levels.map(level_id).unwrap();
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let p = Pos { x, y };
            if map.tile(p).ok().is_some_and(&pred) {
                return p;
            }
        }
    }
    panic!("failed to find matching tile on {level_id:?}");
}

fn find_adjacent_floor(
    save: &aihack_runtime::save::SaveDataV1,
    level_id: LevelId,
    target: Pos,
) -> (Pos, Direction) {
    let map = save.world.levels.map(level_id).unwrap();
    for dir in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
        Direction::NorthWest,
        Direction::NorthEast,
        Direction::SouthEast,
        Direction::SouthWest,
    ] {
        let p = Pos {
            x: target.x - dir.delta().dx,
            y: target.y - dir.delta().dy,
        };
        if map.contains(p) && map.tile(p).ok() == Some(TileKind::Floor) {
            return (p, dir);
        }
    }
    panic!("failed to find adjacent floor next to {target:?} on {level_id:?}");
}

fn find_dark_room_spot_with_dist3_floor(session: &GameSession, level_id: LevelId) -> Pos {
    let level = session.world().levels.get(level_id).unwrap();
    for y in 2..level.map.height - 2 {
        for x in 2..level.map.width - 2 {
            let p = Pos { x, y };
            if level.is_lit(p) || level.map.tile(p).ok() != Some(TileKind::Floor) {
                continue;
            }
            let has_dist3 = aihack_core::vision::visible_positions(&level.map, p)
                .into_iter()
                .any(|other| p.chebyshev_distance(other) >= 3);
            if has_dist3 {
                return p;
            }
        }
    }
    panic!("no dark floor with dist >= 3 visibility found on {level_id:?}");
}

#[test]
fn p9_1_tools_keys_dig_lamp_blindfold_whistle_and_unicorn_horn() {
    let base = start(42, Role::Knight);
    let main1 = LevelId::main(1);
    let mines2 = LevelId::new(BranchId::Mines, 2);

    // 1. SkeletonKey & LockPick unlock genuine generated locked door + chest (LockPick disarms chest trap)
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let door_pos = find_tile(&save, mines2, |t| t == TileKind::Door(DoorState::Locked));
    let (door_origin, door_dir) = find_adjacent_floor(&save, mines2, door_pos);
    save.world.current_level = mines2;
    save.world
        .entities
        .set_actor_location(save.world.player_id, mines2, door_origin);
    let key_id = add_inventory_item(&mut save, ItemKind::SkeletonKey, BucStatus::Uncursed, 0);
    let pick_id = add_inventory_item(&mut save, ItemKind::LockPick, BucStatus::Uncursed, 0);
    let chest_id = save
        .world
        .entities
        .spawn_item(
            ItemKind::Chest,
            EntityLocation::OnMap {
                level: mines2,
                pos: door_origin,
            },
        )
        .unwrap();
    save.world
        .entities
        .set_item_container_locked(chest_id, true);
    save.world
        .entities
        .set_item_container_trapped(chest_id, true);
    let chest_loot = save
        .world
        .entities
        .spawn_item(
            ItemKind::FoodRation,
            EntityLocation::InContainer {
                container: chest_id,
            },
        )
        .unwrap();
    save.world
        .entities
        .push_contained_item(chest_id, chest_loot);

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(
        sess.submit(C::Apply {
            item: key_id,
            direction: Some(door_dir)
        })
        .accepted
    );
    assert_eq!(
        sess.world().current_map().tile(door_pos).unwrap(),
        TileKind::Door(DoorState::Closed)
    );
    assert!(
        sess.submit(C::Apply {
            item: key_id,
            direction: Some(door_dir)
        })
        .accepted
    );
    assert_eq!(
        sess.world().current_map().tile(door_pos).unwrap(),
        TileKind::Door(DoorState::Locked)
    );
    assert!(
        sess.submit(C::Apply {
            item: pick_id,
            direction: None
        })
        .accepted
    );
    let chest_ref = sess.world().entities().get(chest_id).unwrap();
    assert_eq!(chest_ref.item_container_locked(), Some(false));
    assert_eq!(chest_ref.item_container_trapped(), Some(false));

    // 2. PickAxe & DwarvishMattock dig interior Wall and block outer perimeter Wall
    let mut dig_save = base.to_save_data();
    park_all_monsters_on(&mut dig_save, LevelId::main(6));
    let map1 = dig_save.world.levels.map(main1).unwrap().clone();
    let (wall_origin, dig_dir, target_wall) = (2..map1.height - 2)
        .flat_map(|y| (2..map1.width - 2).map(move |x| Pos { x, y }))
        .filter(|p| map1.tile(*p).ok() == Some(TileKind::Floor))
        .find_map(|p| {
            Direction::ALL.into_iter().find_map(|d| {
                let w = p.offset(d.delta());
                (w.x > 0
                    && w.x < map1.width - 1
                    && w.y > 0
                    && w.y < map1.height - 1
                    && map1.tile(w).ok() == Some(TileKind::Wall))
                .then_some((p, d, w))
            })
        })
        .unwrap();
    dig_save
        .world
        .entities
        .set_actor_location(dig_save.world.player_id, main1, wall_origin);
    let mattock = add_inventory_item(
        &mut dig_save,
        ItemKind::DwarvishMattock,
        BucStatus::Uncursed,
        0,
    );
    let mut dig_sess = GameSession::from_save_data(dig_save).unwrap();
    let dig_out = dig_sess.submit(C::Apply {
        item: mattock,
        direction: Some(dig_dir),
    });
    assert!(dig_out.accepted);
    assert_eq!(
        dig_sess.world().current_map().tile(target_wall).unwrap(),
        TileKind::Floor
    );
    let restored_dig = GameSession::from_save_data(dig_sess.to_save_data()).unwrap();
    assert_eq!(
        restored_dig
            .world()
            .current_map()
            .tile(target_wall)
            .unwrap(),
        TileKind::Floor
    );

    // Digging perimeter Wall (x == 0 / width-1 / y == 0 / height-1) is blocked
    let mut perim_save = restored_dig.to_save_data();
    if let Some((edge_p, edge_dir)) = (1..map1.height - 1)
        .flat_map(|y| (1..map1.width - 1).map(move |x| Pos { x, y }))
        .filter(|p| map1.tile(*p).ok() == Some(TileKind::Floor))
        .find_map(|p| {
            [
                Direction::North,
                Direction::South,
                Direction::West,
                Direction::East,
            ]
            .into_iter()
            .find_map(|d| {
                let b = p.offset(d.delta());
                (b.x == 0 || b.y == 0 || b.x == map1.width - 1 || b.y == map1.height - 1)
                    .then_some((p, d))
            })
        })
    {
        perim_save
            .world
            .entities
            .set_actor_location(perim_save.world.player_id, main1, edge_p);
        let mut perim_sess = GameSession::from_save_data(perim_save).unwrap();
        assert!(
            !perim_sess
                .submit(C::Apply {
                    item: mattock,
                    direction: Some(edge_dir)
                })
                .accepted
        );
    }

    // 3. OilLamp expands vision in dark room and burns fuel to extinction
    let dark_level = LevelId::main(2);
    let dark_pos = find_dark_room_spot_with_dist3_floor(&base, dark_level);
    let mut lamp_save = base.to_save_data();
    park_all_monsters_on(&mut lamp_save, LevelId::main(6));
    lamp_save.world.current_level = dark_level;
    lamp_save
        .world
        .entities
        .set_actor_location(lamp_save.world.player_id, dark_level, dark_pos);
    let lamp_id = add_inventory_item(&mut lamp_save, ItemKind::OilLamp, BucStatus::Uncursed, 0);
    lamp_save.world.entities.set_item_charges(lamp_id, Some(2));
    let mut lamp_sess = GameSession::from_save_data(lamp_save).unwrap();
    let dark_visible_count = lamp_sess
        .observation()
        .visible_tiles
        .iter()
        .filter(|t| t.visible)
        .count();
    assert!(
        lamp_sess
            .submit(C::Apply {
                item: lamp_id,
                direction: None
            })
            .accepted
    );
    let lit_visible_count = lamp_sess
        .observation()
        .visible_tiles
        .iter()
        .filter(|t| t.visible)
        .count();
    assert!(lit_visible_count > dark_visible_count);
    let out2 = lamp_sess.submit(C::Wait);
    assert!(out2.accepted);
    assert_eq!(
        lamp_sess
            .world()
            .entities()
            .get(lamp_id)
            .unwrap()
            .item_tool_active(),
        Some(false)
    );
    assert_eq!(lamp_sess.world().entities().item_charges(lamp_id), Some(0));
    assert!(out2
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::Message { text, .. } if text.contains("flickers out"))));
    assert!(
        !lamp_sess
            .submit(C::Apply {
                item: lamp_id,
                direction: None
            })
            .accepted
    );

    // 4. Blindfold toggles sight + Telepathy, cursed blindfold refuses removal
    let mut blind_save = base.to_save_data();
    park_all_monsters_on(&mut blind_save, LevelId::main(6));
    let (bp, bneighbors) = find_floor_with_neighbors(&base, main1, 1);
    blind_save
        .world
        .entities
        .set_actor_location(blind_save.world.player_id, main1, bp);
    let bat_id = blind_save
        .world
        .entities
        .spawn_monster(MonsterKind::Bat, bneighbors[0])
        .unwrap();
    blind_save
        .world
        .entities
        .set_actor_location(bat_id, main1, bneighbors[0]);
    let esp_id = add_inventory_item(&mut blind_save, ItemKind::AmuletEsp, BucStatus::Uncursed, 0);
    let bf_good = add_inventory_item(&mut blind_save, ItemKind::Blindfold, BucStatus::Uncursed, 0);
    let bf_cursed = add_inventory_item(&mut blind_save, ItemKind::Blindfold, BucStatus::Cursed, 0);
    let mut blind_sess = GameSession::from_save_data(blind_save).unwrap();
    assert!(blind_sess.submit(C::PutOn { item: esp_id }).accepted);
    assert!(
        blind_sess
            .submit(C::Apply {
                item: bf_good,
                direction: None
            })
            .accepted
    );
    let bobs = blind_sess.observation();
    assert!(bobs.player.blindfolded);
    assert_eq!(bobs.visible_tiles.iter().filter(|t| t.visible).count(), 1);
    assert!(bobs.visible_entities.iter().any(|e| e.entity == bat_id));
    assert!(
        blind_sess
            .submit(C::Apply {
                item: bf_good,
                direction: None
            })
            .accepted
    );
    assert!(!blind_sess.observation().player.blindfolded);
    assert!(
        blind_sess
            .submit(C::Apply {
                item: bf_cursed,
                direction: None
            })
            .accepted
    );
    assert!(
        !blind_sess
            .submit(C::Apply {
                item: bf_cursed,
                direction: None
            })
            .accepted
    );
    assert!(!blind_sess.submit(C::Drop { item: bf_cursed }).accepted);

    // 5. TinWhistle calls pet closer & UnicornHorn restores status/attributes (cursed harms)
    let mut wh_save = base.to_save_data();
    park_all_monsters_on(&mut wh_save, LevelId::main(6));
    let map = wh_save.world.levels.map(main1).unwrap().clone();
    let mut pet_pair = None;
    for y in 2..map.height - 2 {
        for x in 2..map.width - 4 {
            let p = Pos { x, y };
            let m = Pos { x: x + 1, y };
            let q = Pos { x: x + 2, y };
            if map.tile(p).ok() == Some(TileKind::Floor)
                && map.tile(m).ok() == Some(TileKind::Floor)
                && map.tile(q).ok() == Some(TileKind::Floor)
            {
                pet_pair = Some((p, q));
                break;
            }
        }
        if pet_pair.is_some() {
            break;
        }
    }
    let (wp, pq) = pet_pair.unwrap();
    wh_save
        .world
        .entities
        .set_actor_location(wh_save.world.player_id, main1, wp);
    let tmpl = aihack_runtime::domain::monster::monster_template(MonsterKind::FloatingEye);
    let pet_stats = ActorStats {
        hp: tmpl.hp,
        max_hp: tmpl.hp,
        ac: tmpl.ac,
        hit_bonus: tmpl.hit_bonus,
        damage_bonus: tmpl.damage_bonus,
        damage_reduction: 0,
        damage: tmpl.attack_profile.damage,
        weapon_hit_bonus: 0,
        speed: tmpl.speed,
        energy: 0,
        resistances: Resistances::default(),
        ai_kind: Some(tmpl.ai_kind),
        passive: tmpl.passive,
        difficulty: tmpl.difficulty,
    };
    let pet_id = wh_save
        .world
        .entities
        .spawn(
            EntityKind::Monster(MonsterKind::FloatingEye),
            Faction::Pet,
            pq,
            pet_stats,
        )
        .unwrap();
    wh_save.world.entities.set_actor_location(pet_id, main1, pq);
    let whistle = add_inventory_item(&mut wh_save, ItemKind::TinWhistle, BucStatus::Uncursed, 0);
    let horn = add_inventory_item(&mut wh_save, ItemKind::UnicornHorn, BucStatus::Uncursed, 0);
    let cursed_horn = add_inventory_item(&mut wh_save, ItemKind::UnicornHorn, BucStatus::Cursed, 0);
    wh_save.world.poison_turns = 4;
    wh_save.world.status_timers.confused = 4;
    wh_save.world.status_timers.blind = 3;
    let mut c = wh_save.world.campaign.unwrap();
    c.attributes.str_ = 10;
    wh_save.world.campaign = Some(c);
    wh_save
        .world
        .entities
        .actor_stats_mut(wh_save.world.player_id)
        .unwrap()
        .damage_bonus = c.damage_bonus();

    let mut wh_sess = GameSession::from_save_data(wh_save).unwrap();
    let wh_out = wh_sess.submit(C::Apply {
        item: whistle,
        direction: None,
    });
    assert!(wh_out.accepted);
    let pet_after = wh_sess.world().entities().actor_location(pet_id).unwrap().1;
    assert!(pet_after.chebyshev_distance(wp) < pq.chebyshev_distance(wp));

    assert!(
        wh_sess
            .submit(C::Apply {
                item: horn,
                direction: None
            })
            .accepted
    );
    let after_horn = wh_sess.observation();
    assert_eq!(after_horn.player.poison_turns, 0);
    assert_eq!(after_horn.player.blind_turns, 0);
    assert_eq!(after_horn.player.confused_turns, 0);
    assert!(
        wh_sess.world().campaign.unwrap().attributes.str_ >= Role::Knight.base_attributes().str_
    );

    assert!(
        wh_sess
            .submit(C::Apply {
                item: cursed_horn,
                direction: None
            })
            .accepted
    );
    let after_bad = wh_sess.observation();
    assert!(after_bad.player.confused_turns > 0);
    assert!(after_bad.player.stunned_turns > 0);
}

#[test]
fn p9_2_containers_weights_boh_explosion_and_save_tamper_guards() {
    let base = start(42, Role::Knight);
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let sack = add_inventory_item(&mut save, ItemKind::Sack, BucStatus::Uncursed, 0);
    let boh_blessed = add_inventory_item(&mut save, ItemKind::BagOfHolding, BucStatus::Blessed, 0);
    let boh_uncursed =
        add_inventory_item(&mut save, ItemKind::BagOfHolding, BucStatus::Uncursed, 0);
    let boh_cursed = add_inventory_item(&mut save, ItemKind::BagOfHolding, BucStatus::Cursed, 0);
    let heavy_armor =
        add_inventory_item(&mut save, ItemKind::ArmorChainMail, BucStatus::Uncursed, 0);
    let ration1 = add_inventory_item(&mut save, ItemKind::FoodRation, BucStatus::Uncursed, 0);
    let ration2 = save
        .world
        .entities
        .spawn_item(
            ItemKind::FoodRation,
            EntityLocation::Inventory {
                owner: save.world.player_id,
            },
        )
        .unwrap();
    let letter2 = save
        .world
        .inventory
        .add_existing_with_next_letter(ration2)
        .unwrap();
    save.world.entities.set_item_letter(ration2, letter2);
    let ration3 = save
        .world
        .entities
        .spawn_item(
            ItemKind::FoodRation,
            EntityLocation::Inventory {
                owner: save.world.player_id,
            },
        )
        .unwrap();
    let letter3 = save
        .world
        .inventory
        .add_existing_with_next_letter(ration3)
        .unwrap();
    save.world.entities.set_item_letter(ration3, letter3);

    let mut sess = GameSession::from_save_data(save).unwrap();
    let w_before = sess.world().carried_weight();
    assert!(
        sess.submit(C::ContainerPut {
            container: sack,
            item: ration1
        })
        .accepted
    );
    assert_eq!(sess.world().carried_weight(), w_before);
    assert!(
        sess.submit(C::ContainerPut {
            container: boh_blessed,
            item: heavy_armor
        })
        .accepted
    );
    let w_blessed = sess.world().carried_weight();
    assert!(w_blessed < w_before);
    assert!(
        sess.submit(C::ContainerTake {
            container: boh_blessed,
            item: heavy_armor
        })
        .accepted
    );
    assert_eq!(sess.world().carried_weight(), w_before);

    let w0 = sess.world().carried_weight();
    assert!(
        sess.submit(C::ContainerPut {
            container: boh_uncursed,
            item: ration2
        })
        .accepted
    );
    assert_eq!(sess.world().carried_weight(), w0 - 10);

    let w1 = sess.world().carried_weight();
    assert!(
        sess.submit(C::ContainerPut {
            container: boh_cursed,
            item: ration3
        })
        .accepted
    );
    assert_eq!(sess.world().carried_weight(), w1 + 20);

    // Nesting depth > 4 rejected
    let mut dchain = base.to_save_data();
    park_all_monsters_on(&mut dchain, LevelId::main(6));
    let c0 = add_inventory_item(&mut dchain, ItemKind::Sack, BucStatus::Uncursed, 0);
    let c1 = add_inventory_item(&mut dchain, ItemKind::Sack, BucStatus::Uncursed, 0);
    let c2 = add_inventory_item(&mut dchain, ItemKind::Sack, BucStatus::Uncursed, 0);
    let c3 = add_inventory_item(&mut dchain, ItemKind::Sack, BucStatus::Uncursed, 0);
    let c4 = add_inventory_item(&mut dchain, ItemKind::Sack, BucStatus::Uncursed, 0);
    let c5 = add_inventory_item(&mut dchain, ItemKind::Sack, BucStatus::Uncursed, 0);
    let mut csess = GameSession::from_save_data(dchain).unwrap();
    assert!(
        csess
            .submit(C::ContainerPut {
                container: c3,
                item: c4
            })
            .accepted
    );
    assert!(
        csess
            .submit(C::ContainerPut {
                container: c2,
                item: c3
            })
            .accepted
    );
    assert!(
        csess
            .submit(C::ContainerPut {
                container: c1,
                item: c2
            })
            .accepted
    );
    assert!(
        csess
            .submit(C::ContainerPut {
                container: c0,
                item: c1
            })
            .accepted
    );
    assert!(
        !csess
            .submit(C::ContainerPut {
                container: c5,
                item: c0
            })
            .accepted
    );

    // AmuletAscension cannot be placed in container
    let mut amulet_save = base.to_save_data();
    park_all_monsters_on(&mut amulet_save, LevelId::main(6));
    let amulet_id = amulet_save.world.campaign.unwrap().amulet;
    amulet_save.world.entities.set_item_location(
        amulet_id,
        EntityLocation::Inventory {
            owner: amulet_save.world.player_id,
        },
    );
    let a_letter = amulet_save
        .world
        .inventory
        .add_existing_with_next_letter(amulet_id)
        .unwrap();
    amulet_save
        .world
        .entities
        .set_item_letter(amulet_id, a_letter);
    let asack = add_inventory_item(&mut amulet_save, ItemKind::Sack, BucStatus::Uncursed, 0);
    let mut asess = GameSession::from_save_data(amulet_save).unwrap();
    assert!(
        !asess
            .submit(C::ContainerPut {
                container: asack,
                item: amulet_id
            })
            .accepted
    );

    // BoH nesting explosion -> DeathCause::Explosion when HP low
    let mut boom_save = base.to_save_data();
    park_all_monsters_on(&mut boom_save, LevelId::main(6));
    let b1 = add_inventory_item(
        &mut boom_save,
        ItemKind::BagOfHolding,
        BucStatus::Uncursed,
        0,
    );
    let b2 = add_inventory_item(
        &mut boom_save,
        ItemKind::BagOfHolding,
        BucStatus::Uncursed,
        0,
    );
    boom_save
        .world
        .entities
        .actor_stats_mut(boom_save.world.player_id)
        .unwrap()
        .hp = 1;
    let mut boom_sess = GameSession::from_save_data(boom_save).unwrap();
    let boom_out = boom_sess.submit(C::ContainerPut {
        container: b1,
        item: b2,
    });
    assert!(boom_out.accepted);
    assert!(matches!(
        boom_out.next_state,
        RunState::GameOver {
            cause: DeathCause::Explosion,
            ..
        }
    ));

    // SaveV3 & V1 tamper tests
    let mut bad_orphan = sess.to_save_data();
    let _ = bad_orphan
        .world
        .entities
        .spawn_item(
            ItemKind::Dagger,
            EntityLocation::InContainer {
                container: EntityId(999_999),
            },
        )
        .unwrap();
    assert!(GameSession::from_save_data(bad_orphan).is_err());

    let mut bad_amulet = asess.to_save_data();
    bad_amulet.world.inventory.remove(amulet_id).unwrap();
    bad_amulet.world.entities.clear_item_letter(amulet_id);
    bad_amulet
        .world
        .entities
        .set_item_location(amulet_id, EntityLocation::InContainer { container: asack });
    bad_amulet
        .world
        .entities
        .push_contained_item(asack, amulet_id);
    assert!(GameSession::from_save_data(bad_amulet).is_err());

    let mut bad_erosion = base.to_save_data();
    let d_id = add_inventory_item(&mut bad_erosion, ItemKind::Dagger, BucStatus::Uncursed, 0);
    bad_erosion.world.entities.set_item_erosion(d_id, 4);
    assert!(GameSession::from_save_data(bad_erosion).is_err());

    // Forge Floor -> Lava or Floor -> Altar rejected in SaveV3 map validator
    let mut bad_altar = base.to_save_data();
    let main1_floor = find_floor_with_neighbors(&base, LevelId::main(1), 1).0;
    bad_altar
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(main1_floor, TileKind::Altar)
        .unwrap();
    assert!(GameSession::from_save_data(bad_altar).is_err());

    let legacy = GameSession::new_for_playing(42);
    let mut bad_legacy_tile = legacy.to_save_data();
    assert_eq!(bad_legacy_tile.schema_version, SAVE_SCHEMA_VERSION_V1);
    bad_legacy_tile.world.levels.levels[0]
        .map
        .set_tile(Pos { x: 2, y: 2 }, TileKind::Fountain)
        .unwrap();
    assert!(GameSession::from_save_data(bad_legacy_tile).is_err());

    let mut bad_legacy_trap = legacy.to_save_data();
    bad_legacy_trap.world.levels.levels[0]
        .map
        .set_tile(Pos { x: 2, y: 2 }, TileKind::Trap(TrapKind::Fire))
        .unwrap();
    assert!(GameSession::from_save_data(bad_legacy_trap).is_err());

    let mut bad_legacy_item = legacy.to_save_data();
    let _ = add_inventory_item(&mut bad_legacy_item, ItemKind::Sack, BucStatus::Uncursed, 0);
    assert!(GameSession::from_save_data(bad_legacy_item).is_err());
}

#[test]
fn p9_3_dip_holy_unholy_dilution_alchemy_and_erosion() {
    let base = start(42, Role::Cleric);
    let main3 = LevelId::main(3);
    let main2 = LevelId::main(2);
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let altar_pos = find_tile(&save, main3, |t| t == TileKind::Altar);
    let fountain_pos = find_tile(&save, main3, |t| t == TileKind::Fountain);
    let sink_pos = find_tile(&save, main2, |t| t == TileKind::Sink);

    save.world.current_level = main3;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main3, altar_pos);
    save.world.mp = 5;

    let water_unc = add_inventory_item(&mut save, ItemKind::PotionWater, BucStatus::Uncursed, 0);
    let cursed_dag = add_inventory_item(&mut save, ItemKind::Dagger, BucStatus::Cursed, 0);
    let unholy = add_inventory_item(&mut save, ItemKind::PotionWater, BucStatus::Cursed, 0);
    let mail = add_inventory_item(&mut save, ItemKind::ArmorChainMail, BucStatus::Uncursed, 0);
    let heal1 = add_inventory_item(&mut save, ItemKind::PotionHealing, BucStatus::Uncursed, 0);
    let heal2 = add_inventory_item(&mut save, ItemKind::PotionHealing, BucStatus::Uncursed, 0);
    let scroll_rev = add_inventory_item(&mut save, ItemKind::ScrollReveal, BucStatus::Uncursed, 0);

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(
        sess.submit(C::Dip {
            item: water_unc,
            target: DipTarget::Tile
        })
        .accepted
    );
    assert_eq!(sess.world().mp, 2);
    assert_eq!(
        sess.world().entities().get(water_unc).unwrap().item_buc(),
        Some(BucStatus::Blessed)
    );

    assert!(
        sess.submit(C::Dip {
            item: cursed_dag,
            target: DipTarget::Item(water_unc)
        })
        .accepted
    );
    assert_eq!(
        sess.world().entities().get(cursed_dag).unwrap().item_buc(),
        Some(BucStatus::Uncursed)
    );

    assert!(
        sess.submit(C::Dip {
            item: cursed_dag,
            target: DipTarget::Item(unholy)
        })
        .accepted
    );
    assert_eq!(
        sess.world().entities().get(cursed_dag).unwrap().item_buc(),
        Some(BucStatus::Cursed)
    );

    let extra_before: u16 = sess
        .observation()
        .inventory
        .iter()
        .filter(|i| i.kind == ItemKind::PotionExtraHealing)
        .map(|i| i.quantity)
        .sum();
    assert!(
        sess.submit(C::Dip {
            item: heal1,
            target: DipTarget::Item(heal2)
        })
        .accepted
    );
    let extra_after: u16 = sess
        .observation()
        .inventory
        .iter()
        .filter(|i| i.kind == ItemKind::PotionExtraHealing)
        .map(|i| i.quantity)
        .sum();
    assert_eq!(extra_after, extra_before + 1);

    assert!(sess.submit(C::Wear { item: mail }).accepted);
    let ac_before_rust = sess.observation().player.ac;
    let mut fsave = sess.to_save_data();
    fsave
        .world
        .entities
        .set_actor_location(fsave.world.player_id, main3, fountain_pos);
    let mut fsess = GameSession::from_save_data(fsave).unwrap();
    assert!(
        fsess
            .submit(C::Dip {
                item: mail,
                target: DipTarget::Tile
            })
            .accepted
    );
    assert_eq!(
        fsess.world().entities().get(mail).unwrap().item_erosion(),
        Some(1)
    );
    assert_eq!(fsess.observation().player.ac, ac_before_rust + 1);
    assert_eq!(fsess.to_save_data().schema_version, SAVE_SCHEMA_VERSION_V3);
    let reloaded = GameSession::from_save_data(fsess.to_save_data()).unwrap();
    assert_eq!(reloaded.observation().player.ac, ac_before_rust + 1);

    let mut ssave = reloaded.to_save_data();
    ssave.world.current_level = main2;
    ssave
        .world
        .entities
        .set_actor_location(ssave.world.player_id, main2, sink_pos);
    let mut ssess = GameSession::from_save_data(ssave).unwrap();
    assert!(
        ssess
            .submit(C::Dip {
                item: scroll_rev,
                target: DipTarget::Tile
            })
            .accepted
    );
    assert_eq!(
        ssess.world().entities().item_data(scroll_rev).unwrap().kind,
        ItemKind::ScrollBlank
    );
}

#[test]
fn p9_4_engrave_elbereth_repels_monsters_and_erodes() {
    let base = start(42, Role::Mage);
    let main1 = LevelId::main(1);
    let (p_pos, neighbors) = find_floor_with_neighbors(&base, main1, 2);

    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, p_pos);
    let orc_id = save
        .world
        .entities
        .spawn_monster(MonsterKind::Orc, neighbors[0])
        .unwrap();
    save.world
        .entities
        .set_actor_location(orc_id, main1, neighbors[0]);
    let wfire = add_inventory_item(&mut save, ItemKind::WandFire, BucStatus::Uncursed, 0);

    let mut sess = GameSession::from_save_data(save).unwrap();
    let ch_before = sess.world().entities().item_charges(wfire).unwrap();
    let eng_out = sess.submit(C::Engrave {
        method: EngravingMethod::Item(wfire),
        text: EngravingText::Elbereth,
    });
    assert!(eng_out.accepted);
    assert_eq!(
        sess.world().entities().item_charges(wfire).unwrap(),
        ch_before - 1
    );
    let tile_obs = sess
        .observation()
        .visible_tiles
        .into_iter()
        .find(|t| t.pos == p_pos)
        .unwrap();
    assert_eq!(tile_obs.engraving.as_deref(), Some("Elbereth"));

    let hp_before = sess.observation().player.hp;
    for _ in 0..3 {
        assert!(sess.submit(C::Wait).accepted);
    }
    assert_eq!(sess.observation().player.hp, hp_before);

    let mut dust_save = base.to_save_data();
    park_all_monsters_on(&mut dust_save, LevelId::main(6));
    dust_save
        .world
        .entities
        .set_actor_location(dust_save.world.player_id, main1, p_pos);
    let target_pos = neighbors[0];
    let atk_dir = Direction::ALL
        .into_iter()
        .find(|d| p_pos.offset(d.delta()) == target_pos)
        .unwrap();
    let eye_id = dust_save
        .world
        .entities
        .spawn_monster(MonsterKind::FloatingEye, target_pos)
        .unwrap();
    dust_save
        .world
        .entities
        .set_actor_location(eye_id, main1, target_pos);
    let mut dsess = GameSession::from_save_data(dust_save).unwrap();
    assert!(
        dsess
            .submit(C::Engrave {
                method: EngravingMethod::Fingers,
                text: EngravingText::Elbereth
            })
            .accepted
    );
    let _ = dsess.submit(C::Move(atk_dir));
    let tile_after_atk = dsess
        .observation()
        .visible_tiles
        .into_iter()
        .find(|t| t.pos == p_pos)
        .unwrap();
    assert!(tile_after_atk.engraving.is_none());

    let mut conf_save = base.to_save_data();
    park_all_monsters_on(&mut conf_save, LevelId::main(6));
    conf_save.world.status_timers.confused = 3;
    let mut csess = GameSession::from_save_data(conf_save).unwrap();
    assert!(
        csess
            .submit(C::Engrave {
                method: EngravingMethod::Fingers,
                text: EngravingText::Elbereth
            })
            .accepted
    );
    let cpos = csess.world().player_pos();
    let garbled = csess
        .observation()
        .visible_tiles
        .into_iter()
        .find(|t| t.pos == cpos)
        .unwrap()
        .engraving;
    assert_ne!(garbled.as_deref(), Some("Elbereth"));
}

#[test]
fn p9_5_terrain_12_traps_adapters_and_llm_budgets() {
    let base = start(42, Role::Knight);
    let main1 = LevelId::main(1);
    let main3 = LevelId::main(3);
    let main5 = LevelId::main(5);

    // 1. Drink on genuine Fountain & DryFountain SaveV3 validation
    let mut f_save = base.to_save_data();
    park_all_monsters_on(&mut f_save, LevelId::main(6));
    let real_fountain = find_tile(&f_save, main1, |t| t == TileKind::Fountain);
    f_save
        .world
        .entities
        .set_actor_location(f_save.world.player_id, main1, real_fountain);
    let mut fsess = GameSession::from_save_data(f_save).unwrap();
    assert!(fsess.submit(C::Drink).accepted);
    let mut dry_base = base.to_save_data();
    dry_base
        .world
        .levels
        .map_mut(main1)
        .unwrap()
        .set_tile(real_fountain, TileKind::DryFountain)
        .unwrap();
    assert!(GameSession::from_save_data(dry_base).is_ok());

    // 2. Genuine Water: heavy pack -> DeathCause::Drowning; genuine Lava -> DeathCause::Lava
    let mut drown_save = base.to_save_data();
    park_all_monsters_on(&mut drown_save, LevelId::main(6));
    let water_pos = find_tile(&drown_save, main3, |t| t == TileKind::Water);
    let (w_adj, w_dir) = find_adjacent_floor(&drown_save, main3, water_pos);
    drown_save.world.current_level = main3;
    drown_save
        .world
        .entities
        .set_actor_location(drown_save.world.player_id, main3, w_adj);
    drown_save
        .world
        .entities
        .actor_stats_mut(drown_save.world.player_id)
        .unwrap()
        .hp = 1;
    let mut drown_sess = GameSession::from_save_data(drown_save).unwrap();
    let drown_out = drown_sess.submit(C::Move(w_dir));
    assert!(drown_out.accepted, "events={:?}", drown_out.events);
    assert!(matches!(
        drown_out.next_state,
        RunState::GameOver {
            cause: DeathCause::Drowning,
            ..
        }
    ));

    let mut lava_save = base.to_save_data();
    park_all_monsters_on(&mut lava_save, LevelId::main(6));
    let lava_pos = find_tile(&lava_save, main5, |t| t == TileKind::Lava);
    let (l_adj, l_dir) = find_adjacent_floor(&lava_save, main5, lava_pos);
    lava_save.world.current_level = main5;
    lava_save
        .world
        .entities
        .set_actor_location(lava_save.world.player_id, main5, l_adj);
    lava_save
        .world
        .entities
        .actor_stats_mut(lava_save.world.player_id)
        .unwrap()
        .hp = 2;
    let mut lava_sess = GameSession::from_save_data(lava_save).unwrap();
    let lava_out = lava_sess.submit(C::Move(l_dir));
    assert!(lava_out.accepted);
    assert!(matches!(
        lava_out.next_state,
        RunState::GameOver {
            cause: DeathCause::Lava,
            ..
        }
    ));

    // 3. All 9 genuine generated new TrapKind triggers & reveal
    let trap_locations = [
        (LevelId::main(1), TrapKind::SleepingGas),
        (LevelId::main(2), TrapKind::Rust),
        (LevelId::main(3), TrapKind::Teleport),
        (LevelId::main(3), TrapKind::AntiMagic),
        (LevelId::main(4), TrapKind::FallingRock),
        (LevelId::main(4), TrapKind::Hole),
        (LevelId::main(5), TrapKind::Fire),
        (LevelId::main(5), TrapKind::Polymorph),
        (LevelId::main(6), TrapKind::Magic),
    ];
    for (lvl, trap) in trap_locations {
        let mut tsave = base.to_save_data();
        let tpos = find_tile(&tsave, lvl, |t| t == TileKind::HiddenTrap(trap));
        let (tadj, tdir) = find_adjacent_floor(&tsave, lvl, tpos);
        park_all_monsters_on(&mut tsave, LevelId::main(6));
        tsave.world.current_level = lvl;
        tsave
            .world
            .entities
            .set_actor_location(tsave.world.player_id, lvl, tadj);
        let mut tsess = GameSession::from_save_data(tsave).unwrap();
        let tout = tsess.submit(C::Move(tdir));
        assert!(tout.accepted, "{trap:?} rejected on {lvl:?}");
        assert!(
            tout.events
                .iter()
                .any(|e| matches!(e, GameEvent::TrapTriggered { trap: t, .. } if *t == trap)),
            "missing TrapTriggered for {trap:?} on {lvl:?}"
        );
    }

    // 4. Headless survival-v1 prioritizes UnicornHorn in campaign when confused/poisoned
    let mut hsave = base.to_save_data();
    park_all_monsters_on(&mut hsave, LevelId::main(6));
    let horn = add_inventory_item(&mut hsave, ItemKind::UnicornHorn, BucStatus::Uncursed, 0);
    hsave.world.poison_turns = 3;
    hsave.world.status_timers.confused = 3;
    let mut hsess = GameSession::from_save_data(hsave).unwrap();
    let cands = HeadlessPolicy::SurvivalV1.candidates(&hsess);
    assert_eq!(
        cands,
        vec![C::Apply {
            item: horn,
            direction: None
        }]
    );
    let target_turn = hsess.turn() + 1;
    let report = run_to_turn(&mut hsess, target_turn, HeadlessPolicy::SurvivalV1).unwrap();
    assert_eq!(report.accepted_turns, 1);
    assert_eq!(hsess.observation().player.poison_turns, 0);

    // 5. TUI rendering & P9 keys + LLM budget & ActionSpace <= 64
    let mut ui_save = base.to_save_data();
    park_all_monsters_on(&mut ui_save, LevelId::main(6));
    let ui_fountain = find_tile(&ui_save, main3, |t| t == TileKind::Fountain);
    ui_save.world.current_level = main3;
    ui_save
        .world
        .entities
        .set_actor_location(ui_save.world.player_id, main3, ui_fountain);
    let ui_sack = add_inventory_item(&mut ui_save, ItemKind::Sack, BucStatus::Uncursed, 0);
    let _ui_water = add_inventory_item(&mut ui_save, ItemKind::PotionWater, BucStatus::Uncursed, 0);
    let ui_bf = add_inventory_item(&mut ui_save, ItemKind::Blindfold, BucStatus::Uncursed, 0);
    let ui_ration = add_inventory_item(&mut ui_save, ItemKind::FoodRation, BucStatus::Uncursed, 0);
    ui_save.world.entities.set_item_location(
        ui_ration,
        EntityLocation::InContainer { container: ui_sack },
    );
    ui_save
        .world
        .entities
        .push_contained_item(ui_sack, ui_ration);
    ui_save.world.inventory.remove(ui_ration).unwrap();
    ui_save.world.entities.clear_item_letter(ui_ration);
    let mut ui_sess = GameSession::from_save_data(ui_save).unwrap();
    assert!(
        ui_sess
            .submit(C::Apply {
                item: ui_bf,
                direction: None
            })
            .accepted
    );

    let obs = ui_sess.observation();
    assert!(obs.action_space.commands.len() <= 64);
    assert!(obs
        .action_space
        .commands
        .iter()
        .any(|c| matches!(c, aihack_ai_contract::ActionIntent::Command(C::Drink))));
    assert!(obs.action_space.commands.iter().any(|c| {
        matches!(
            c,
            aihack_ai_contract::ActionIntent::Command(C::Engrave {
                method: EngravingMethod::Fingers,
                text: EngravingText::Elbereth
            })
        )
    }));
    let llm_json = serde_json::to_string(&LlmObservationView::from(&obs)).unwrap();
    assert!(llm_json.len() < REQUEST_BODY_LIMIT);

    let status_txt = render_panels::status_lines(&obs).join("\n");
    assert!(status_txt.contains("BLINDFOLD"));

    let clock = Arc::new(TestClock::default());
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        ui_sess,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    for ch in ['a', 'C', 'v', 'E'] {
        assert!(matches!(
            send_key(&mut app, &clock, ch, 80, 24),
            UiCommandCandidate::BeginAction(k) if k == ch
        ));
        clock.0.fetch_add(1000, Ordering::Relaxed);
        app.release_transition_gesture_on_idle();
        app.release_transition_gesture_on_idle();
        let close = runtime_event_to_candidate(
            Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            80,
            24,
            &mut app,
        )
        .expect("Esc must close open menu");
        assert!(!app.handle_candidate_owned(close).unwrap());
    }
    assert_eq!(
        send_key(&mut app, &clock, 'g', 80, 24),
        UiCommandCandidate::Command(C::Drink)
    );
}
