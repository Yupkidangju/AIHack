use aihack_core::action::CommandIntent as C;
use aihack_core::campaign::{CampaignState, Role};
use aihack_core::ids::EntityId;
use aihack_runtime::GameSession;

fn start(seed: u64, role: Role) -> GameSession {
    let mut session = GameSession::try_new(seed).unwrap();
    assert!(session.submit(C::Wait).accepted);
    let outcome = session.submit(C::StartCampaign { role });
    assert!(outcome.accepted, "{:?}", outcome.events);
    assert!(!outcome.turn_advanced);
    session
}

#[test]
fn selected_role_bootstraps_real_campaign_and_v2_save() {
    for role in [Role::Knight, Role::Scout, Role::Mage] {
        let session = start(42, role);
        assert_eq!(session.world().campaign.unwrap().role, role);
        assert_eq!(session.observation().player.max_hp, role.base_hp());
        assert_eq!(session.world().levels.len(), 8);
        let save = session.to_save_data();
        assert_eq!(save.schema_version, 2);
        let restored = GameSession::from_save_data(save).unwrap();
        assert_eq!(
            session.snapshot().stable_hash(),
            restored.snapshot().stable_hash()
        );
    }
}

#[test]
fn roles_and_level_thresholds_are_bounded_and_distinct() {
    assert!(Role::Knight.base_hp() > Role::Mage.base_hp());
    assert!(Role::Scout.base_hit() > Role::Knight.base_hit());
    for (xp, level) in [(0, 1), (19, 1), (20, 2), (179, 9), (180, 10)] {
        let state = CampaignState {
            role: Role::Knight,
            xp,
            amulet: EntityId(1),
        };
        assert_eq!(state.level(), level);
    }
}

#[test]
fn quest_amulet_has_no_consumable_or_equipment_effect() {
    let data = aihack_content::item_data_from_registry(
        aihack_core::domain::item::ItemKind::AmuletAscension,
        aihack_content::registry().unwrap(),
    )
    .unwrap();
    assert_eq!(data.glyph, '"');
    assert!(data.attack_profile.is_none() && data.consumable_effect.is_none());
    assert!(data.wand_effect.is_none() && data.max_charges.is_none());
}

#[test]
fn player_kills_produce_xp_and_growth_is_consumed_by_actor_stats() {
    use aihack_core::{domain::entity::ActorKind, position::Direction};
    let mut session = start(42, Role::Knight);
    let monsters: Vec<_> = session
        .world()
        .entities
        .entities()
        .iter()
        .filter(|e| matches!(e.actor_kind(), Some(ActorKind::Monster(_))))
        .take(2)
        .map(|e| e.id)
        .collect();
    for id in monsters {
        let mut save = session.to_save_data();
        let (level, pos) = save.world.entities.actor_location(id).unwrap();
        save.world.current_level = level;
        save.world.entities.set_actor_location(
            save.world.player_id,
            level,
            aihack_core::position::Pos {
                x: pos.x - 1,
                y: pos.y,
            },
        );
        save.world.entities.actor_stats_mut(id).unwrap().hp = 1;
        session = GameSession::from_save_data(save).unwrap();
        for _ in 0..20 {
            if !session.world().entities.get(id).unwrap().is_alive_actor() {
                break;
            }
            assert!(session.submit(C::Move(Direction::East)).accepted);
        }
        assert!(!session.world().entities.get(id).unwrap().is_alive_actor());
    }
    let campaign = session.world().campaign.unwrap();
    assert_eq!(campaign.level(), 2);
    let stats = session
        .world()
        .entities
        .actor_stats(session.world().player_id)
        .unwrap();
    assert_eq!(stats.max_hp, 32);
    assert_eq!(stats.hit_bonus, 5);
    assert_eq!(stats.damage_bonus, 3);
    GameSession::from_save_data(session.to_save_data()).unwrap();
    let target = session
        .world()
        .entities
        .entities()
        .iter()
        .find(|entity| {
            matches!(entity.actor_kind(), Some(ActorKind::Monster(_))) && entity.is_alive_actor()
        })
        .unwrap()
        .id;
    let mut high = session.to_save_data();
    let (level, pos) = high.world.entities.actor_location(target).unwrap();
    high.world.current_level = level;
    high.world.entities.set_actor_location(
        high.world.player_id,
        level,
        aihack_core::position::Pos {
            x: pos.x - 1,
            y: pos.y,
        },
    );
    let mut low = high.clone();
    low.world.campaign.as_mut().unwrap().xp = 0;
    let stats = low
        .world
        .entities
        .actor_stats_mut(low.world.player_id)
        .unwrap();
    stats.max_hp = 28;
    stats.hp = stats.hp.min(28);
    stats.hit_bonus = 4;
    stats.damage_bonus = 2;
    let mut high = GameSession::from_save_data(high).unwrap();
    let mut low = GameSession::from_save_data(low).unwrap();
    let attack_roll = |outcome: aihack_core::turn::TurnOutcome| {
        outcome
            .events
            .into_iter()
            .find_map(|event| {
                if let aihack_core::event::GameEvent::AttackResolved { attack_roll, .. } = event {
                    Some(attack_roll)
                } else {
                    None
                }
            })
            .unwrap()
    };
    assert_eq!(
        attack_roll(high.submit(C::Move(Direction::East))),
        attack_roll(low.submit(C::Move(Direction::East))) + 1
    );
}

#[test]
fn forged_campaign_growth_and_goal_identity_are_rejected() {
    let session = start(42, Role::Knight);
    let mut save = session.to_save_data();
    save.world.campaign.as_mut().unwrap().xp = 181;
    assert!(GameSession::from_save_data(save).is_err());
    let mut save = session.to_save_data();
    save.world.campaign.as_mut().unwrap().amulet = save.world.player_id;
    assert!(GameSession::from_save_data(save).is_err());
}

#[test]
fn early_ascension_and_forged_victory_are_rejected() {
    let mut session = start(42, Role::Knight);
    let before = session.snapshot().stable_hash();
    assert!(!session.submit(C::Ascend).accepted);
    assert_eq!(session.snapshot().stable_hash(), before);
    let mut save = session.to_save_data();
    save.run_state = aihack_core::run_state::RunState::Victory { final_score: 10000 };
    assert!(GameSession::from_save_data(save).is_err());
}

fn route(
    session: &GameSession,
    target: aihack_core::position::Pos,
) -> Vec<aihack_core::position::Direction> {
    use aihack_core::{domain::tile::TileKind, position::Direction};
    use std::collections::{HashMap, VecDeque};
    let origin = session.world().player_pos();
    let mut seen = HashMap::new();
    let mut queue = VecDeque::from([origin]);
    seen.insert(origin, None);
    while let Some(pos) = queue.pop_front() {
        if pos == target {
            break;
        }
        for dir in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let next = pos.offset(dir.delta());
            if matches!(
                session.world().current_map().tile(next),
                Ok(TileKind::Floor | TileKind::StairsUp | TileKind::StairsDown)
            ) && !seen.contains_key(&next)
            {
                seen.insert(next, Some((pos, dir)));
                queue.push_back(next);
            }
        }
    }
    assert!(seen.contains_key(&target), "target must be reachable");
    let mut path = Vec::new();
    let mut pos = target;
    while let Some((prev, dir)) = seen[&pos] {
        path.push(dir);
        pos = prev;
    }
    path.reverse();
    path
}

fn walk(session: &mut GameSession, target: aihack_core::position::Pos) {
    use aihack_core::domain::item::ItemKind;
    for _ in 0..1000 {
        if session.world().player_pos() == target {
            return;
        }
        let observation = session.observation();
        assert!(
            session.world().player_alive(),
            "player died at {:?}",
            session.world().current_level()
        );
        let healing = observation
            .inventory
            .iter()
            .find(|i| i.kind == ItemKind::PotionHealing);
        if session.world().carried_weight() > session.world().carrying_capacity() {
            if let Some(item) = healing {
                assert!(session.submit(C::Quaff { item: item.item }).accepted);
                continue;
            }
            let item = observation
                .inventory
                .iter()
                .find(|i| i.kind == ItemKind::Rock || i.kind == ItemKind::FoodRation)
                .unwrap();
            assert!(session.submit(C::Drop { item: item.item }).accepted);
            continue;
        }
        if observation.player.hp * 2 < observation.player.max_hp {
            if let Some(item) = healing {
                assert!(session.submit(C::Quaff { item: item.item }).accepted);
                continue;
            }
        }
        let dir = route(session, target)[0];
        let outcome = session.submit(C::Move(dir));
        assert!(outcome.accepted, "{:?}", outcome.events);
    }
    panic!("route did not progress");
}

fn tour(seed: u64, role: Role) -> GameSession {
    use aihack_core::{
        domain::{entity::EntityLocation, item::ItemKind},
        ids::{BranchId, LevelId},
        run_state::RunState,
    };
    let mut session = start(seed, role);
    let dagger = session
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::Dagger)
        .unwrap()
        .item;
    assert!(session.submit(C::Wield { item: dagger }).accepted);
    if let Some(armor) = session
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::ArmorLeather)
        .map(|i| i.item)
    {
        assert!(session.submit(C::Wear { item: armor }).accepted);
    }
    for depth in 1..=6 {
        assert_eq!(session.world().current_level(), LevelId::main(depth));
        if depth == 3 {
            assert!(session
                .observation()
                .legal_actions
                .contains(&C::EnterBranch));
            assert!(session.submit(C::EnterBranch).accepted);
            let mines = LevelId {
                branch: BranchId::Mines,
                depth: 1,
            };
            assert_eq!(session.world().current_level(), mines);
            let down = session.world().levels.stairs_down_pos(mines).unwrap();
            walk(&mut session, down);
            assert!(session.submit(C::Descend).accepted);
            assert!(session.submit(C::Ascend).accepted);
            let up = session.world().levels.stairs_up_pos(mines).unwrap();
            walk(&mut session, up);
            assert!(session.submit(C::Ascend).accepted);
        }
        let potions: Vec<_> = session
            .world()
            .entities
            .entities()
            .iter()
            .filter_map(|entity| {
                let (kind, _, location, _, _) = entity.item()?;
                match location {
                    EntityLocation::OnMap { level, pos }
                        if level == session.world().current_level()
                            && kind == ItemKind::PotionHealing =>
                    {
                        Some(pos)
                    }
                    _ => None,
                }
            })
            .collect();
        for pos in potions {
            walk(&mut session, pos);
            assert!(session.submit(C::Pickup).accepted);
        }
        if depth < 6 {
            let pos = session
                .world()
                .levels
                .stairs_down_pos(LevelId::main(depth))
                .unwrap();
            walk(&mut session, pos);
            assert!(session.submit(C::Descend).accepted);
        }
    }
    let amulet = session.world().campaign.unwrap().amulet;
    let EntityLocation::OnMap { pos, .. } = session
        .world()
        .entities
        .get(amulet)
        .unwrap()
        .item()
        .unwrap()
        .2
    else {
        panic!()
    };
    walk(&mut session, pos);
    assert!(session.submit(C::Pickup).accepted);
    assert!(session.observation().campaign.unwrap().has_amulet);
    assert!(session.submit(C::Drop { item: amulet }).accepted);
    assert!(!session.observation().campaign.unwrap().has_amulet);
    assert!(session.submit(C::Pickup).accepted);
    // 실제 목표 획득 지점에서 재실행한 것과 같은 deserialize 경계를 통과한다.
    let save = session.to_save_data();
    let encoded = serde_json::to_string(&save).unwrap();
    let mut restored =
        GameSession::from_save_data(serde_json::from_str(&encoded).unwrap()).unwrap();
    assert_eq!(session.submit(C::Wait), restored.submit(C::Wait));
    session = restored;
    for depth in (1..=6).rev() {
        let up = session
            .world()
            .levels
            .stairs_up_pos(LevelId::main(depth))
            .unwrap();
        walk(&mut session, up);
        let mut checkpoint = session.clone();
        let outcome = session.submit(C::Ascend);
        assert!(outcome.accepted, "{:?}", outcome.events);
        if depth == 1 {
            let line = aihack_core::save::ReplayLineV1 {
                turn_before: checkpoint.turn(),
                command: C::Ascend,
                snapshot_hash_after: outcome.snapshot_hash.clone(),
                outcome: outcome.clone(),
            };
            aihack_headless::run_replay_to_turn(&mut checkpoint, session.turn(), &[line]).unwrap();
            assert_eq!(
                checkpoint.snapshot().stable_hash(),
                session.snapshot().stable_hash()
            );
        }
    }
    assert!(matches!(session.run_state(), RunState::Victory { .. }));
    let before = session.snapshot().stable_hash();
    assert!(!session.submit(C::Ascend).accepted);
    assert_eq!(before, session.snapshot().stable_hash());
    let turn = session.turn();
    let error = aihack_headless::run_to_turn(
        &mut session,
        turn + 1,
        aihack_headless::HeadlessPolicy::wait_v1(),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        aihack_headless::HeadlessRunError::VictoryBeforeTarget {
            submitted_commands: 0,
            ..
        }
    ));
    assert_eq!(before, session.snapshot().stable_hash());
    GameSession::from_save_data(session.to_save_data()).unwrap();
    session
}

#[test]
fn actual_command_tour_reaches_amulet_and_ascension_without_fixture_mutation() {
    for (seed, role) in [(42, Role::Knight), (7, Role::Scout), (1234, Role::Mage)] {
        tour(seed, role);
    }
}

#[test]
fn generator_is_repeatable_seed_sensitive_and_every_floor_is_connected() {
    let a = start(42, Role::Knight);
    let b = start(42, Role::Knight);
    let c = start(7, Role::Knight);
    assert_eq!(a.snapshot().stable_hash(), b.snapshot().stable_hash());
    assert_ne!(a.world().levels, c.world().levels);
    for seed in [42, 7, 1234] {
        let session = start(seed, Role::Knight);
        for level in &session.world().levels.levels {
            let up = session.world().levels.stairs_up_pos(level.id).unwrap();
            let mut save = session.to_save_data();
            save.world.current_level = level.id;
            save.world
                .entities
                .set_actor_location(save.world.player_id, level.id, up);
            let probe = GameSession::from_save_data(save).unwrap();
            for y in 0..20 {
                for x in 0..40 {
                    let pos = aihack_core::position::Pos { x, y };
                    if level.map.tile(pos).unwrap() != aihack_core::domain::tile::TileKind::Wall {
                        route(&probe, pos);
                    }
                }
            }
        }
    }
}

fn click_ui(app: &mut aihack_tui::tui::TuiApp, label: &str, width: u16, height: u16) {
    use aihack_tui::tui::{render_frame, runtime_event_to_candidate};
    use crossterm::event::{Event, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render_frame(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let (x, y) = (0..height)
        .find_map(|y| {
            let line: String = (0..width).map(|x| buffer[(x, y)].symbol()).collect();
            line.find(label)
                .map(|byte| (line[..byte].chars().count() as u16, y))
        })
        .unwrap_or_else(|| panic!("missing CTA {label}"));
    let candidate = runtime_event_to_candidate(
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: x,
            row: y,
            modifiers: KeyModifiers::NONE,
        }),
        width,
        height,
        app,
    )
    .unwrap();
    assert!(!app.handle_candidate_owned(candidate).unwrap());
}

#[test]
fn actual_ui_role_buttons_and_victory_restart_work_at_minimum_size() {
    use aihack_tui::tui::{TuiApp, UiRuntimeConfig};
    for (label, role) in [
        ("[1] Knight", Role::Knight),
        ("[2] Scout", Role::Scout),
        ("[3] Mage", Role::Mage),
    ] {
        let mut app = TuiApp::new(
            GameSession::try_new(42).unwrap(),
            UiRuntimeConfig::default(),
        );
        click_ui(&mut app, "Press Enter to Start", 60, 24);
        click_ui(&mut app, label, 60, 24);
        assert_eq!(app.observation().campaign.unwrap().role, role);
        let lines = aihack_tui::tui::render_panels::status_lines(&app.observation());
        assert!(lines.iter().any(|line| line.contains("/120")));
    }
    let victory = tour(42, Role::Knight);
    assert_eq!(
        victory.snapshot().stable_hash(),
        tour(42, Role::Knight).snapshot().stable_hash()
    );
    let mut app = TuiApp::new(victory, UiRuntimeConfig::default());
    click_ui(&mut app, "[N] New Run", 60, 24);
    assert_eq!(app.run_state(), aihack_core::run_state::RunState::Title);
    assert!(app.observation().campaign.is_none());
    assert_eq!(app.observation().seed, 43);
}

#[test]
fn creation_keyboard_roles_are_press_only_and_out_of_phase_start_is_rejected() {
    use aihack_tui::tui::{runtime_event_to_candidate, TuiApp, UiRuntimeConfig};
    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    for (key, role) in [('1', Role::Knight), ('2', Role::Scout), ('3', Role::Mage)] {
        let mut session = GameSession::try_new(42).unwrap();
        session.submit(C::Wait);
        let mut app = TuiApp::new(session, UiRuntimeConfig::default());
        let repeat =
            KeyEvent::new_with_kind(KeyCode::Char(key), KeyModifiers::NONE, KeyEventKind::Repeat);
        assert!(runtime_event_to_candidate(Event::Key(repeat), 80, 24, &mut app).is_none());
        let press = KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE);
        let candidate = runtime_event_to_candidate(Event::Key(press), 80, 24, &mut app).unwrap();
        app.handle_candidate_owned(candidate).unwrap();
        assert_eq!(app.observation().campaign.unwrap().role, role);
    }
    let mut session = start(42, Role::Knight);
    let before = session.snapshot().stable_hash();
    assert!(
        !session
            .submit(C::StartCampaign { role: Role::Mage })
            .accepted
    );
    assert_eq!(before, session.snapshot().stable_hash());
}

#[test]
fn campaign_rejects_schema_topology_goal_and_growth_tampering_without_panics() {
    let session = start(42, Role::Knight);
    let original = session.to_save_data();
    let mut bad = original.clone();
    bad.schema_version = 1;
    assert!(GameSession::from_save_data(bad).is_err());
    let mut bad = original.clone();
    bad.world.campaign = None;
    assert!(GameSession::from_save_data(bad).is_err());
    let mut bad = original.clone();
    bad.world.levels.levels.pop();
    assert!(GameSession::from_save_data(bad).is_err());
    let mut bad = original.clone();
    bad.world
        .entities
        .actor_stats_mut(bad.world.player_id)
        .unwrap()
        .hit_bonus += 1;
    assert!(GameSession::from_save_data(bad).is_err());
    let mut bad = original.clone();
    bad.world.entities.set_item_location(
        bad.world.campaign.unwrap().amulet,
        aihack_core::domain::entity::EntityLocation::Consumed,
    );
    assert!(GameSession::from_save_data(bad).is_err());
    let mut bad = original;
    bad.turn = u64::MAX;
    bad.run_state = aihack_core::run_state::RunState::Victory { final_score: 0 };
    assert!(
        std::panic::catch_unwind(|| GameSession::from_save_data(bad))
            .unwrap()
            .is_err()
    );
}

#[test]
fn campaign_teleport_uses_generated_landing_and_remains_loadable() {
    use aihack_core::{
        domain::{entity::EntityLocation, item::ItemKind},
        ids::LevelId,
    };
    let mut save = start(42, Role::Knight).to_save_data();
    let item = save
        .world
        .entities
        .spawn_item(
            ItemKind::ScrollLevelTeleport,
            EntityLocation::Inventory {
                owner: save.world.player_id,
            },
        )
        .unwrap();
    let letter = save
        .world
        .inventory
        .add_existing_with_next_letter(item)
        .unwrap();
    save.world.entities.set_item_letter(item, letter);
    let mut session = GameSession::from_save_data(save).unwrap();
    assert!(session.submit(C::Read { item }).accepted);
    assert_eq!(session.world().current_level(), LevelId::main(2));
    assert_eq!(
        session.world().player_pos(),
        session
            .world()
            .levels
            .stairs_up_pos(LevelId::main(2))
            .unwrap()
    );
    GameSession::from_save_data(session.to_save_data()).unwrap();
}
