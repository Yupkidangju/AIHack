use aihack_core::action::CommandIntent as C;
use aihack_core::campaign::Role;
use aihack_core::domain::{
    combat::DeathCause,
    entity::{ActorKind, EntityLocation},
    item::ItemKind,
    monster::MonsterKind,
    tile::{DoorState, TileKind, TrapKind},
};
use aihack_core::event::GameEvent;
use aihack_core::ids::{BranchId, LevelId};
use aihack_core::position::{Direction, Pos};
use aihack_core::run_state::RunState;
use aihack_runtime::GameSession;
use aihack_tui::tui::{render_frame, render_panels, TuiApp, UiPanel, UiRuntimeConfig};

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

fn find_door_matching(
    session: &GameSession,
    level: LevelId,
    pred: impl Fn(TileKind) -> bool,
) -> Option<(Pos, TileKind)> {
    let map = session.world().levels.map(level)?;
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let pos = Pos { x, y };
            let tile = map.tile(pos).ok()?;
            if pred(tile) {
                return Some((pos, tile));
            }
        }
    }
    None
}

fn adjacent_floor_toward(session: &GameSession, level: LevelId, target: Pos) -> (Pos, Direction) {
    let map = session.world().levels.map(level).unwrap();
    for dir in [
        Direction::East,
        Direction::West,
        Direction::North,
        Direction::South,
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::SouthWest,
    ] {
        let d = dir.delta();
        let candidate = Pos {
            x: target.x - d.dx,
            y: target.y - d.dy,
        };
        if matches!(
            map.tile(candidate),
            Ok(TileKind::Floor | TileKind::StairsUp | TileKind::StairsDown)
        ) {
            return (candidate, dir);
        }
    }
    panic!("no adjacent floor around {target:?} on {level:?}");
}

fn visible_floor_neighbors(
    session: &GameSession,
    level: LevelId,
    origin: Pos,
    exclude: &[Pos],
    needed: usize,
) -> Vec<Pos> {
    let map = session.world().levels.map(level).unwrap();
    let mut found = Vec::new();
    for dy in -2_i16..=2 {
        for dx in -2_i16..=2 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let candidate = Pos {
                x: origin.x + dx,
                y: origin.y + dy,
            };
            if exclude.contains(&candidate) {
                continue;
            }
            if map.tile(candidate).ok() == Some(TileKind::Floor)
                && aihack_core::vision::has_line_of_sight(map, origin, candidate)
            {
                found.push(candidate);
                if found.len() == needed {
                    return found;
                }
            }
        }
    }
    panic!(
        "needed {needed} visible floor tiles near {origin:?} on {level:?}, found {}",
        found.len()
    );
}

fn find_hidden_trap(session: &GameSession, level: LevelId, kind: TrapKind) -> Option<Pos> {
    let map = session.world().levels.map(level)?;
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = Pos { x, y };
            if map.tile(pos).ok() == Some(TileKind::HiddenTrap(kind)) {
                return Some(pos);
            }
        }
    }
    None
}

fn find_campaign_item(
    session: &GameSession,
    kind: ItemKind,
) -> (aihack_core::ids::EntityId, LevelId, Pos) {
    session
        .world()
        .entities()
        .entities()
        .iter()
        .find_map(|entity| {
            let (item_kind, _, location, _, _) = entity.item()?;
            if item_kind != kind {
                return None;
            }
            match location {
                EntityLocation::OnMap { level, pos } => Some((entity.id, level, pos)),
                EntityLocation::Inventory { .. }
                | EntityLocation::InContainer { .. }
                | EntityLocation::Consumed => None,
            }
        })
        .unwrap_or_else(|| panic!("missing campaign item {kind:?}"))
}

fn move_player_cleanly(
    session: &GameSession,
    level: LevelId,
    pos: Pos,
    park_monsters_level: LevelId,
) -> GameSession {
    let mut save = session.to_save_data();
    park_all_monsters_on(&mut save, park_monsters_level);
    save.world.current_level = level;
    save.world
        .entities
        .set_actor_location(save.world.player_id, level, pos);
    GameSession::from_save_data(save).unwrap()
}

#[test]
fn campaign_doors_and_hidden_doors_work_across_depths_and_save_roundtrip() {
    let session = start(42, Role::Knight);

    let (door_pos_1, tile1) = find_door_matching(&session, LevelId::main(1), |t| {
        t == TileKind::Door(DoorState::Closed)
    })
    .unwrap();
    assert_eq!(tile1, TileKind::Door(DoorState::Closed));

    let (_, tile2_closed) = find_door_matching(&session, LevelId::main(2), |t| {
        t == TileKind::Door(DoorState::Closed)
    })
    .unwrap();
    let (hidden_pos_2, tile2_hidden) =
        find_door_matching(&session, LevelId::main(2), |t| t == TileKind::HiddenDoor).unwrap();
    let mines2 = LevelId {
        branch: BranchId::Mines,
        depth: 2,
    };
    let (_, mines2_locked) =
        find_door_matching(&session, mines2, |t| t == TileKind::Door(DoorState::Locked)).unwrap();
    let (_, mines2_hidden) =
        find_door_matching(&session, mines2, |t| t == TileKind::HiddenDoor).unwrap();
    assert_eq!(tile2_closed, TileKind::Door(DoorState::Closed));
    assert_eq!(tile2_hidden, TileKind::HiddenDoor);
    assert_eq!(mines2_locked, TileKind::Door(DoorState::Locked));
    assert_eq!(mines2_hidden, TileKind::HiddenDoor);
    assert!(find_hidden_trap(&session, mines2, TrapKind::Bear).is_some());

    let (p1_pos, dir1) = adjacent_floor_toward(&session, LevelId::main(1), door_pos_1);
    let mut session = move_player_cleanly(&session, LevelId::main(1), p1_pos, mines_level(2));

    // 닫힌 문은 바로 지나갈 수 없고 Open -> Close -> Kick이 모두 동작한다.
    assert!(!session.submit(C::Move(dir1)).accepted);
    let opened = session.submit(C::Open(dir1));
    assert!(opened.accepted);
    assert!(opened.events.iter().any(|event| matches!(
        event,
        GameEvent::DoorChanged {
            from: DoorState::Closed,
            to: DoorState::Open,
            ..
        }
    )));
    assert_eq!(
        session.world().current_map().tile(door_pos_1),
        Ok(TileKind::Door(DoorState::Open))
    );

    let closed = session.submit(C::Close(dir1));
    assert!(closed.accepted);
    assert_eq!(
        session.world().current_map().tile(door_pos_1),
        Ok(TileKind::Door(DoorState::Closed))
    );

    let kicked = session.submit(C::Kick(dir1));
    assert!(kicked.accepted);
    assert_eq!(
        session.world().current_map().tile(door_pos_1),
        Ok(TileKind::Door(DoorState::Open))
    );
    assert!(session.submit(C::Move(dir1)).accepted);
    assert_eq!(session.world().player_pos(), door_pos_1);

    // Main 2 HiddenDoor는 관측상 벽으로 보이다가 Search 후 Open 가능하며 저장/로드로 유지된다.
    let (p2_pos, dir2) = adjacent_floor_toward(&session, LevelId::main(2), hidden_pos_2);
    let mut session = move_player_cleanly(&session, LevelId::main(2), p2_pos, mines_level(2));
    assert!(session
        .observation()
        .visible_tiles
        .iter()
        .any(|tile| tile.pos == hidden_pos_2 && tile.tile == TileKind::Wall));
    assert!(!session.submit(C::Move(dir2)).accepted);
    assert!(!session.submit(C::Open(dir2)).accepted);

    let searched = session.submit(C::Search);
    assert!(searched.accepted);
    assert!(searched.events.iter().any(|event| matches!(
        event,
        GameEvent::TileRevealed {
            pos,
            tile: TileKind::Door(DoorState::Closed),
        } if *pos == hidden_pos_2
    )));
    assert!(session.submit(C::Open(dir2)).accepted);
    assert_eq!(
        session.world().current_map().tile(hidden_pos_2),
        Ok(TileKind::Door(DoorState::Open))
    );

    let reloaded = GameSession::from_save_data(session.to_save_data()).unwrap();
    assert_eq!(
        reloaded
            .world()
            .levels
            .map(LevelId::main(1))
            .unwrap()
            .tile(door_pos_1),
        Ok(TileKind::Door(DoorState::Open))
    );
    assert_eq!(
        reloaded
            .world()
            .levels
            .map(LevelId::main(2))
            .unwrap()
            .tile(hidden_pos_2),
        Ok(TileKind::Door(DoorState::Open))
    );
}

#[test]
fn campaign_traps_pit_dart_bear_paralysis_reveal_and_death_cause() {
    let base = start(42, Role::Knight);

    // 1. Pit trap on Main 1: 3 dmg, paralysis 0
    let pit_pos = find_hidden_trap(&base, LevelId::main(1), TrapKind::Pit).unwrap();
    let mut s_pit = move_player_cleanly(
        &base,
        LevelId::main(1),
        Pos {
            x: pit_pos.x - 1,
            y: pit_pos.y,
        },
        mines_level(2),
    );
    let hp_before = s_pit.observation().player.hp;
    let out = s_pit.submit(C::Move(Direction::East));
    assert!(out.accepted);
    assert!(out.events.iter().any(|event| matches!(
        event,
        GameEvent::TrapTriggered {
            trap: TrapKind::Pit,
            pos,
            damage: 3,
            ..
        } if *pos == pit_pos
    )));
    assert_eq!(s_pit.observation().player.hp, hp_before - 3);
    assert_eq!(s_pit.observation().player.paralysis_turns, 0);
    assert_eq!(
        s_pit.world().current_map().tile(pit_pos),
        Ok(TileKind::Trap(TrapKind::Pit))
    );

    // 2. Dart trap on Main 2: Search로 사전 탐지 후 밟으면 2 dmg
    let dart_pos = find_hidden_trap(&base, LevelId::main(2), TrapKind::Dart).unwrap();
    let mut s_dart = move_player_cleanly(
        &base,
        LevelId::main(2),
        Pos {
            x: dart_pos.x - 1,
            y: dart_pos.y,
        },
        mines_level(2),
    );
    let search_out = s_dart.submit(C::Search);
    assert!(search_out.events.iter().any(|event| matches!(
        event,
        GameEvent::TileRevealed {
            pos,
            tile: TileKind::Trap(TrapKind::Dart),
        } if *pos == dart_pos
    )));
    let hp_before = s_dart.observation().player.hp;
    let step_dart = s_dart.submit(C::Move(Direction::East));
    assert!(step_dart.accepted);
    assert!(step_dart.events.iter().any(|event| matches!(
        event,
        GameEvent::TrapTriggered {
            trap: TrapKind::Dart,
            damage: 2,
            ..
        }
    )));
    assert_eq!(s_dart.observation().player.hp, hp_before - 2);
    assert_eq!(s_dart.observation().player.paralysis_turns, 0);

    // 3. Bear trap on Main 3: 2 dmg + 다음 1턴 마비 (비대기 행동 거부 -> Wait로 해소)
    let bear_pos = find_hidden_trap(&base, LevelId::main(3), TrapKind::Bear).unwrap();
    let mut s_bear = move_player_cleanly(
        &base,
        LevelId::main(3),
        Pos {
            x: bear_pos.x - 1,
            y: bear_pos.y,
        },
        mines_level(2),
    );
    let hp_before = s_bear.observation().player.hp;
    let step_bear = s_bear.submit(C::Move(Direction::East));
    assert!(step_bear.accepted);
    assert!(step_bear.events.iter().any(|event| matches!(
        event,
        GameEvent::TrapTriggered {
            trap: TrapKind::Bear,
            damage: 2,
            ..
        }
    )));
    assert_eq!(s_bear.observation().player.hp, hp_before - 2);
    assert_eq!(s_bear.observation().player.paralysis_turns, 1);
    assert!(!s_bear.submit(C::Move(Direction::East)).accepted);
    assert!(s_bear.submit(C::Wait).accepted);
    assert_eq!(s_bear.observation().player.paralysis_turns, 0);
    assert!(s_bear.submit(C::Move(Direction::East)).accepted);

    // 4. ScrollReveal은 캠페인 층의 숨겨진 함정을 일괄 공개한다.
    let mut s_rev = move_player_cleanly(
        &base,
        LevelId::main(3),
        Pos {
            x: bear_pos.x - 1,
            y: bear_pos.y,
        },
        mines_level(2),
    );
    let scroll_reveal = s_rev
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::ScrollReveal)
        .unwrap()
        .item;
    assert!(
        s_rev
            .submit(C::Read {
                item: scroll_reveal
            })
            .accepted
    );
    assert_eq!(
        s_rev.world().current_map().tile(bear_pos),
        Ok(TileKind::Trap(TrapKind::Bear))
    );

    // 5. 치명적 Dart / Bear 함정 사망 시 DeathCause::Trap { trap }이 정확히 기록된다.
    for (level, pos, kind) in [
        (LevelId::main(2), dart_pos, TrapKind::Dart),
        (LevelId::main(3), bear_pos, TrapKind::Bear),
    ] {
        let mut save = base.to_save_data();
        park_all_monsters_on(&mut save, mines_level(2));
        save.world.current_level = level;
        save.world.entities.set_actor_location(
            save.world.player_id,
            level,
            Pos {
                x: pos.x - 1,
                y: pos.y,
            },
        );
        save.world
            .entities
            .actor_stats_mut(save.world.player_id)
            .unwrap()
            .hp = 1;
        let mut doomed = GameSession::from_save_data(save).unwrap();
        let death_out = doomed.submit(C::Move(Direction::East));
        assert!(death_out.accepted);
        assert!(matches!(
            doomed.run_state(),
            RunState::GameOver {
                cause: DeathCause::Trap { trap },
                ..
            } if trap == kind
        ));
    }
}

#[test]
fn v2_campaign_map_validation_allows_transitions_and_rejects_tampering() {
    let session = start(42, Role::Knight);
    let original = session.to_save_data();

    // 정당한 런타임 상태 전이: 문 열기/공개, 숨겨진 함정 공개, 내부 벽 굴착(Wall->Floor)
    let mut valid_save = original.clone();
    let (door1_pos, _) = find_door_matching(&session, LevelId::main(1), |t| {
        t == TileKind::Door(DoorState::Closed)
    })
    .unwrap();
    let pit1 = find_hidden_trap(&session, LevelId::main(1), TrapKind::Pit).unwrap();
    let (hidden2_pos, _) =
        find_door_matching(&session, LevelId::main(2), |t| t == TileKind::HiddenDoor).unwrap();
    let (hidden4_pos, _) =
        find_door_matching(&session, LevelId::main(4), |t| t == TileKind::HiddenDoor).unwrap();
    let map1 = original.world.levels.map(LevelId::main(1)).unwrap();
    let interior_wall = (1..map1.height - 1)
        .flat_map(|y| (1..map1.width - 1).map(move |x| Pos { x, y }))
        .find(|&pos| map1.tile(pos).ok() == Some(TileKind::Wall))
        .unwrap();
    valid_save
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(door1_pos, TileKind::Door(DoorState::Open))
        .unwrap();
    valid_save
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(pit1, TileKind::Trap(TrapKind::Pit))
        .unwrap();
    valid_save
        .world
        .levels
        .map_mut(LevelId::main(1))
        .unwrap()
        .set_tile(interior_wall, TileKind::Floor)
        .unwrap();
    valid_save
        .world
        .levels
        .map_mut(LevelId::main(2))
        .unwrap()
        .set_tile(hidden2_pos, TileKind::Door(DoorState::Closed))
        .unwrap();
    valid_save
        .world
        .levels
        .map_mut(LevelId::main(4))
        .unwrap()
        .set_tile(hidden4_pos, TileKind::Door(DoorState::Open))
        .unwrap();
    assert!(GameSession::from_save_data(valid_save).is_ok());

    // 부정 변조 거부 목록
    let occupied: std::collections::HashSet<Pos> = original
        .world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| match entity.actor_kind().map(|_| entity.id) {
            Some(id) => original
                .world
                .entities
                .actor_location(id)
                .and_then(|(lvl, pos)| (lvl == LevelId::main(1)).then_some(pos)),
            None => entity.item().and_then(|(_, _, loc, _, _)| match loc {
                EntityLocation::OnMap { level, pos } if level == LevelId::main(1) => Some(pos),
                _ => None,
            }),
        })
        .collect();
    let free_floor = (1..map1.height - 1)
        .flat_map(|y| (1..map1.width - 1).map(move |x| Pos { x, y }))
        .find(|&pos| map1.tile(pos).ok() == Some(TileKind::Floor) && !occupied.contains(&pos))
        .unwrap();
    let stairs_down_1 = session
        .world()
        .levels
        .stairs_down_pos(LevelId::main(1))
        .unwrap();

    for (label, pos, bad_tile) in [
        ("border_breach", Pos { x: 0, y: 5 }, TileKind::Floor),
        ("floor_to_wall", free_floor, TileKind::Wall),
        ("trap_removed", pit1, TileKind::Floor),
        ("trap_kind_swap", pit1, TileKind::Trap(TrapKind::Dart)),
        ("stairs_removed", stairs_down_1, TileKind::Floor),
        ("extra_stairs", free_floor, TileKind::StairsDown),
        ("forged_door", free_floor, TileKind::Door(DoorState::Open)),
    ] {
        let mut tampered = original.clone();
        tampered
            .world
            .levels
            .map_mut(LevelId::main(1))
            .unwrap()
            .set_tile(pos, bad_tile)
            .unwrap();
        let err = GameSession::from_save_data(tampered).unwrap_err();
        assert!(
            err.to_string().contains("campaign generator/map mismatch"),
            "{label}: unexpected error {err}"
        );
    }
}

#[test]
fn cleric_starting_kit_stats_and_xp_growth() {
    let session = start(42, Role::Cleric);
    let stats = session
        .world()
        .entities()
        .actor_stats(session.world().player_id)
        .unwrap();
    assert_eq!(stats.max_hp, 24);
    assert_eq!(stats.hp, 24);
    assert_eq!(stats.hit_bonus, 3);
    assert_eq!(stats.damage_bonus, 1);
    assert_eq!(session.world().carried_weight(), 97);
    assert_eq!(session.world().carrying_capacity(), 120);

    let inv_kinds: Vec<ItemKind> = session
        .observation()
        .inventory
        .iter()
        .map(|entry| entry.kind)
        .collect();
    for expected in [
        ItemKind::Dagger,
        ItemKind::FoodRation,
        ItemKind::WandMagicMissile,
        ItemKind::ScrollReveal,
        ItemKind::Rock,
        ItemKind::Mace,
        ItemKind::PotionExtraHealing,
        ItemKind::ScrollIdentify,
    ] {
        assert!(inv_kinds.contains(&expected), "missing {expected:?}");
    }

    // 실제 캠페인 몬스터 처치를 통해 XP 180(레벨 10)까지 성장 및 저장/복원 검증
    let mut grown_session = session;
    let mace_id = grown_session
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::Mace)
        .unwrap()
        .item;
    assert!(grown_session.submit(C::Wield { item: mace_id }).accepted);
    let targets: Vec<_> = grown_session
        .world()
        .entities()
        .entities()
        .iter()
        .filter(|e| matches!(e.actor_kind(), Some(ActorKind::Monster(_))))
        .map(|e| {
            (
                e.id,
                grown_session
                    .world()
                    .entities()
                    .actor_location(e.id)
                    .unwrap(),
            )
        })
        .collect();
    for (id, (level, monster_pos)) in targets {
        if grown_session.observation().campaign.unwrap().xp >= 180 {
            break;
        }
        let mut save = grown_session.to_save_data();
        let park_lvl = if level == mines_level(2) {
            LevelId::main(1)
        } else {
            mines_level(2)
        };
        park_all_monsters_on(&mut save, park_lvl);
        save.world.current_level = level;
        save.world
            .entities
            .set_actor_location(id, level, monster_pos);
        save.world.entities.actor_stats_mut(id).unwrap().hp = 1;
        let map = save.world.levels.map(level).unwrap();
        let (attack_dir, adj_pos) = [
            (
                Direction::East,
                Pos {
                    x: monster_pos.x - 1,
                    y: monster_pos.y,
                },
            ),
            (
                Direction::West,
                Pos {
                    x: monster_pos.x + 1,
                    y: monster_pos.y,
                },
            ),
            (
                Direction::South,
                Pos {
                    x: monster_pos.x,
                    y: monster_pos.y - 1,
                },
            ),
            (
                Direction::North,
                Pos {
                    x: monster_pos.x,
                    y: monster_pos.y + 1,
                },
            ),
        ]
        .into_iter()
        .find(|(_, p)| map.tile(*p).ok() == Some(TileKind::Floor))
        .unwrap();
        save.world
            .entities
            .set_actor_location(save.world.player_id, level, adj_pos);
        grown_session = GameSession::from_save_data(save).unwrap();
        for _ in 0..20 {
            if !grown_session
                .world()
                .entities
                .get(id)
                .unwrap()
                .is_alive_actor()
            {
                break;
            }
            if grown_session.observation().player.paralysis_turns > 0 {
                assert!(grown_session.submit(C::Wait).accepted);
                continue;
            }
            assert!(grown_session.submit(C::Move(attack_dir)).accepted);
        }
        assert!(!grown_session
            .world()
            .entities
            .get(id)
            .unwrap()
            .is_alive_actor());
    }

    let reloaded = GameSession::from_save_data(grown_session.to_save_data()).unwrap();
    let c = reloaded.observation().campaign.unwrap();
    assert_eq!(c.level, 10);
    assert_eq!(c.xp, 180);
    assert_eq!(reloaded.observation().player.max_hp, 60);
    let g_stats = reloaded
        .world()
        .entities()
        .actor_stats(reloaded.world().player_id)
        .unwrap();
    assert_eq!(g_stats.hit_bonus, 12);
    assert_eq!(g_stats.damage_bonus, 10);
}

#[test]
fn p4_items_monsters_scroll_charging_and_bump_attack_on_trap() {
    let cleric = start(42, Role::Cleric);

    // 1. 5종 신규 아이템 & 3종 신규 몬스터가 실제 생성 캠페인에 존재하는지 확인
    let (short_sword_id, ss_lvl, ss_pos) = find_campaign_item(&cleric, ItemKind::ShortSword);
    let (_, mace_lvl, _) = find_campaign_item(&cleric, ItemKind::Mace);
    let (chain_mail_id, cm_lvl, cm_pos) = find_campaign_item(&cleric, ItemKind::ArmorChainMail);
    let (_, eh_lvl, eh_pos) = find_campaign_item(&cleric, ItemKind::PotionExtraHealing);
    let (charging_id, ch_lvl, ch_pos) = find_campaign_item(&cleric, ItemKind::ScrollCharging);
    assert_ne!(ss_lvl, cm_lvl);
    assert_ne!(mace_lvl, eh_lvl);
    assert_ne!(ch_lvl, cm_lvl);

    let monster_kinds: std::collections::HashSet<MonsterKind> = cleric
        .world()
        .entities()
        .entities()
        .iter()
        .filter_map(|e| match e.actor_kind()? {
            ActorKind::Monster(kind) => Some(kind),
            ActorKind::Player => None,
        })
        .collect();
    assert!(monster_kinds.contains(&MonsterKind::Kobold));
    assert!(monster_kinds.contains(&MonsterKind::Orc));
    assert!(monster_kinds.contains(&MonsterKind::Ogre));

    // 2. Mace 및 ShortSword 장착/근접 공격 확인
    let mut s_combat = move_player_cleanly(&cleric, ss_lvl, ss_pos, mines_level(1));
    assert!(s_combat.submit(C::Pickup).accepted);
    assert!(
        s_combat
            .submit(C::Wield {
                item: short_sword_id
            })
            .accepted
    );
    let starting_mace = s_combat
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::Mace)
        .unwrap()
        .item;
    assert!(
        s_combat
            .submit(C::Wield {
                item: starting_mace
            })
            .accepted
    );

    // 3. 함정 타일 위 적 몬스터에 대한 근접 bump attack + XP 보상
    let pit_pos = find_hidden_trap(&cleric, LevelId::main(1), TrapKind::Pit).unwrap();
    let ogre_id = cleric
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Ogre)))
        .unwrap()
        .id;
    let mut save = cleric.to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    save.world.current_level = LevelId::main(1);
    save.world
        .entities
        .set_actor_location(ogre_id, LevelId::main(1), pit_pos);
    save.world.entities.actor_stats_mut(ogre_id).unwrap().hp = 1;
    save.world.entities.set_actor_location(
        save.world.player_id,
        LevelId::main(1),
        Pos {
            x: pit_pos.x - 1,
            y: pit_pos.y,
        },
    );
    let mut s_bump = GameSession::from_save_data(save).unwrap();
    for _ in 0..10 {
        if !s_bump
            .world()
            .entities
            .get(ogre_id)
            .unwrap()
            .is_alive_actor()
        {
            break;
        }
        assert!(s_bump.submit(C::Move(Direction::East)).accepted);
    }
    assert!(!s_bump
        .world()
        .entities
        .get(ogre_id)
        .unwrap()
        .is_alive_actor());
    assert!(s_bump.world().campaign.unwrap().xp > 0);
    // 플레이어는 공격 후 함정 칸으로 이동하지 않았으므로 Pit 함정이 발동하지 않았다.
    assert_eq!(
        s_bump.world().player_pos(),
        Pos {
            x: pit_pos.x - 1,
            y: pit_pos.y,
        }
    );

    // 4. ArmorChainMail 착용(AC -3) / 저장·복원 / 해제(Drop -> AC 0)
    let mut s_armor = move_player_cleanly(&cleric, cm_lvl, cm_pos, mines_level(1));
    // 무게 여유 확보를 위해 기본 식량/바위를 먼저 내려둔다.
    let rock = s_armor
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::Rock)
        .unwrap()
        .item;
    assert!(s_armor.submit(C::Drop { item: rock }).accepted);
    assert!(s_armor.submit(C::Pickup).accepted);
    assert!(
        s_armor
            .submit(C::Wear {
                item: chain_mail_id
            })
            .accepted
    );
    assert_eq!(
        s_armor
            .world()
            .entities
            .actor_stats(s_armor.world().player_id)
            .unwrap()
            .ac,
        7
    );
    let s_armor_reloaded = GameSession::from_save_data(s_armor.to_save_data()).unwrap();
    assert_eq!(
        s_armor_reloaded
            .world()
            .entities
            .actor_stats(s_armor_reloaded.world().player_id)
            .unwrap()
            .ac,
        7
    );
    let mut s_armor = s_armor_reloaded;
    assert!(
        s_armor
            .submit(C::Drop {
                item: chain_mail_id
            })
            .accepted
    );
    assert_eq!(
        s_armor
            .world()
            .entities
            .actor_stats(s_armor.world().player_id)
            .unwrap()
            .ac,
        10
    );

    // 5. PotionExtraHealing (2d8+8 = 10..=24) 회복 확인
    let mut s_heal = move_player_cleanly(&cleric, eh_lvl, eh_pos, mines_level(1));
    assert!(s_heal.submit(C::Pickup).accepted);
    let extra_heal_id = s_heal
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::PotionExtraHealing)
        .unwrap()
        .item;
    let mut heal_save = s_heal.to_save_data();
    heal_save
        .world
        .entities
        .actor_stats_mut(heal_save.world.player_id)
        .unwrap()
        .hp = 4;
    let mut s_heal = GameSession::from_save_data(heal_save).unwrap();
    assert!(
        s_heal
            .submit(C::Quaff {
                item: extra_heal_id
            })
            .accepted
    );
    assert!(s_heal.observation().player.hp >= 14);
    assert!(s_heal.observation().player.hp <= 24);

    // 6. ScrollCharging으로 소진된 WandMagicMissile 충전 및 상한 유지 + 저장·복원
    let mut s_charge = move_player_cleanly(&cleric, ch_lvl, ch_pos, mines_level(1));
    assert!(s_charge.submit(C::Pickup).accepted);
    let wand_id = s_charge
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::WandMagicMissile)
        .unwrap()
        .item;
    let max_charges = s_charge
        .world()
        .entities
        .get(wand_id)
        .unwrap()
        .item()
        .unwrap()
        .1
        .max_charges
        .unwrap();
    assert!(max_charges > 0);
    for _ in 0..max_charges {
        assert!(
            s_charge
                .submit(C::Zap {
                    item: wand_id,
                    direction: Direction::East,
                })
                .accepted
        );
    }
    assert_eq!(
        s_charge
            .world()
            .entities
            .get(wand_id)
            .unwrap()
            .item()
            .unwrap()
            .4,
        Some(0)
    );
    let read_out = s_charge.submit(C::Read { item: charging_id });
    assert!(read_out.accepted);
    assert!(read_out
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::Message { .. })));
    assert_eq!(
        s_charge
            .world()
            .entities
            .get(wand_id)
            .unwrap()
            .item()
            .unwrap()
            .4,
        Some(max_charges)
    );
    let reloaded_charge = GameSession::from_save_data(s_charge.to_save_data()).unwrap();
    assert_eq!(
        reloaded_charge
            .world()
            .entities
            .get(wand_id)
            .unwrap()
            .item()
            .unwrap()
            .4,
        Some(max_charges)
    );
    // Main 5의 두 번째 ScrollCharging을 읽어도 max_charges를 초과하지 않는다.
    let (second_charging, ch5_lvl, ch5_pos) = reloaded_charge
        .world()
        .entities
        .entities()
        .iter()
        .find_map(|entity| {
            let (kind, _, loc, _, _) = entity.item()?;
            if kind == ItemKind::ScrollCharging {
                match loc {
                    EntityLocation::OnMap { level, pos } => Some((entity.id, level, pos)),
                    _ => None,
                }
            } else {
                None
            }
        })
        .unwrap();
    let mut overcharge = move_player_cleanly(&reloaded_charge, ch5_lvl, ch5_pos, mines_level(1));
    assert!(overcharge.submit(C::Pickup).accepted);
    assert!(
        overcharge
            .submit(C::Read {
                item: second_charging
            })
            .accepted
    );
    assert_eq!(
        overcharge
            .world()
            .entities
            .get(wand_id)
            .unwrap()
            .item()
            .unwrap()
            .4,
        Some(max_charges)
    );
}

#[test]
fn tui_renders_p4_doors_traps_items_monsters_and_cleric_cta() {
    let base = start(42, Role::Cleric);

    // 1. Character Creation 60x24 화면에 [4] Cleric 및 Esc - Back to Title 표시
    let mut creation_app = TuiApp::new(
        GameSession::try_new(42).unwrap(),
        UiRuntimeConfig::default(),
    );
    creation_app
        .handle_candidate_owned(
            aihack_tui::tui::runtime_key_to_candidate(
                crossterm::event::KeyCode::Enter,
                &creation_app.run_state(),
                &creation_app.observation(),
            )
            .unwrap(),
        )
        .unwrap();
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(60, 24)).unwrap();
    terminal
        .draw(|frame| render_frame(frame, &mut creation_app))
        .unwrap();
    let creation_screen: String = (0..24)
        .map(|y| {
            (0..60)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(creation_screen.contains("[4] Cleric - HP24, mace/heal"));
    assert!(creation_screen.contains("Esc - Back to Title"));

    // 2. 문 닫힘('+') vs 문 열림('-') 및 inspect_lines 타일/몬스터 라벨
    let (door_pos, _) = find_door_matching(&base, LevelId::main(1), |t| {
        t == TileKind::Door(DoorState::Closed)
    })
    .unwrap();
    let (player_pos, open_dir) = adjacent_floor_toward(&base, LevelId::main(1), door_pos);
    let neighbors = visible_floor_neighbors(&base, LevelId::main(1), player_pos, &[door_pos], 3);
    let (kobold_pos, orc_pos, ogre_pos) = (neighbors[0], neighbors[1], neighbors[2]);
    let kobold_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Kobold)))
        .unwrap()
        .id;
    let orc_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Orc)))
        .unwrap()
        .id;
    let ogre_id = base
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.actor_kind() == Some(ActorKind::Monster(MonsterKind::Ogre)))
        .unwrap()
        .id;

    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, mines_level(2));
    save.world.current_level = LevelId::main(1);
    save.world
        .entities
        .set_actor_location(save.world.player_id, LevelId::main(1), player_pos);
    save.world
        .entities
        .set_actor_location(kobold_id, LevelId::main(1), kobold_pos);
    save.world
        .entities
        .set_actor_location(orc_id, LevelId::main(1), orc_pos);
    save.world
        .entities
        .set_actor_location(ogre_id, LevelId::main(1), ogre_pos);

    let mut session = GameSession::from_save_data(save).unwrap();
    let closed_inspect = render_panels::inspect_lines(
        &session.observation(),
        Some(door_pos),
        UiPanel::Inspect,
        &[],
    );
    assert!(closed_inspect.iter().any(|l| l == "tile closed door"));
    for (pos, expected) in [
        (kobold_pos, "entity kobold"),
        (orc_pos, "entity orc"),
        (ogre_pos, "entity ogre"),
    ] {
        let lines =
            render_panels::inspect_lines(&session.observation(), Some(pos), UiPanel::Inspect, &[]);
        assert!(
            lines.iter().any(|l| l.starts_with(expected)),
            "missing {expected} in {lines:?}"
        );
    }

    let mut map_term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 28)).unwrap();
    let mut app_closed = TuiApp::new(session.clone(), UiRuntimeConfig::default());
    let layout = aihack_tui::tui::compute_layout(80, 28);
    let vp_closed = app_closed.viewport_for_observation(layout);
    map_term
        .draw(|frame| render_frame(frame, &mut app_closed))
        .unwrap();
    let buf = map_term.backend().buffer();
    let (dx, dy) = vp_closed.world_to_terminal(door_pos, layout.map).unwrap();
    assert_eq!(buf[(dx, dy)].symbol(), "+");
    for (pos, sym) in [(kobold_pos, "k"), (orc_pos, "o"), (ogre_pos, "O")] {
        let (mx, my) = vp_closed.world_to_terminal(pos, layout.map).unwrap();
        assert_eq!(buf[(mx, my)].symbol(), sym);
    }

    assert!(session.submit(C::Open(open_dir)).accepted);
    let open_inspect = render_panels::inspect_lines(
        &session.observation(),
        Some(door_pos),
        UiPanel::Inspect,
        &[],
    );
    assert!(open_inspect.iter().any(|l| l == "tile open door"));
    let mut app_open = TuiApp::new(session, UiRuntimeConfig::default());
    let vp_open = app_open.viewport_for_observation(layout);
    map_term
        .draw(|frame| render_frame(frame, &mut app_open))
        .unwrap();
    let (dx2, dy2) = vp_open.world_to_terminal(door_pos, layout.map).unwrap();
    assert_eq!(map_term.backend().buffer()[(dx2, dy2)].symbol(), "-");

    // 3. Pit / Dart / Bear 함정 라벨
    for (level, kind, label) in [
        (LevelId::main(1), TrapKind::Pit, "tile pit trap"),
        (LevelId::main(2), TrapKind::Dart, "tile dart trap"),
        (LevelId::main(3), TrapKind::Bear, "tile bear trap"),
    ] {
        let trap_pos = find_hidden_trap(&base, level, kind).unwrap();
        let (adj, _) = adjacent_floor_toward(&base, level, trap_pos);
        let mut s_trap = move_player_cleanly(&base, level, adj, mines_level(2));
        assert!(s_trap.submit(C::Search).accepted);
        let lines = render_panels::inspect_lines(
            &s_trap.observation(),
            Some(trap_pos),
            UiPanel::Inspect,
            &[],
        );
        assert!(
            lines.iter().any(|l| l == label),
            "expected {label} in {lines:?}"
        );
    }
}
