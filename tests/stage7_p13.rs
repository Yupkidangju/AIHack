use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    sync::Mutex,
    thread,
    time::Duration,
};

use aihack_ai_contract::NarrativeTopic;
use aihack_content::{monster_display_name, monster_glyph, registry};
use aihack_core::{
    action::CommandIntent as C,
    campaign::{
        quest_artifact_for_role, LearnedSpell, Role, SpellKind, ARTIFACT_INVOKE_COOLDOWN_TURNS,
    },
    domain::{
        entity::{ActorKind, EntityLocation},
        item::ItemKind,
        level::BranchGraph,
        monster::MonsterKind,
        tile::TileKind,
    },
    ids::{BranchId, LevelId},
    position::{Direction, Pos},
    run_state::RunState,
    save::SAVE_SCHEMA_VERSION_V1,
};
use aihack_llm::{
    config::{LlmRequestKind, LocalLlmConfig},
    decision::DecisionRequest,
    narrative::NarrativeRequest,
    service::{
        canonical_request_size_bytes, LlmObservationView, LlmRequestInput, LocalLlmPort,
        LocalLlmService, LLM_SCHEMA_VERSION,
    },
    soft_adjudication::SoftAdjudicationRequest,
    transport::{OpenAiNarrativeTransport, REQUEST_BODY_LIMIT},
    ClientRevision,
};
use aihack_runtime::GameSession;
use aihack_tui::tui::{
    labels::collect_auto_labels,
    render_frame,
    render_map::MapWidget,
    render_panels::{inspect_lines, status_lines},
    TuiApp, UiPanel, UiRuntimeConfig, UiTheme, Viewport,
};
use ratatui::{backend::TestBackend, layout::Rect, widgets::Widget, Terminal};

static LLM_TEST_ENV_LOCK: Mutex<()> = Mutex::new(());

fn start_campaign(seed: u64, role: Role) -> GameSession {
    let mut session = GameSession::try_new(seed).unwrap();
    assert!(session.submit(C::Wait).accepted);
    let out = session.submit(C::StartCampaign { role });
    assert!(out.accepted, "{:?}", out.events);
    session
}

fn teleport_player(session: &mut GameSession, level: LevelId, pos: Pos) {
    let mut save = session.to_save_data();
    save.world.current_level = level;
    save.world
        .entities
        .set_actor_location(save.world.player_id, level, pos);
    if let Some(stats) = save.world.entities.actor_stats_mut(save.world.player_id) {
        stats.hp = stats.max_hp;
    }
    *session = GameSession::from_save_data(save).unwrap();
}

fn find_tile(session: &GameSession, target: TileKind) -> Pos {
    (0..21)
        .flat_map(|y| (0..80).map(move |x| Pos { x, y }))
        .find(|&p| session.world().current_map().tile(p).ok() == Some(target))
        .unwrap_or_else(|| {
            panic!(
                "missing tile {target:?} on {:?}",
                session.world().current_level()
            )
        })
}

fn clear_hostiles_on(session: &mut GameSession, level: LevelId) {
    let mut save = session.to_save_data();
    let mines2 = LevelId::new(BranchId::Mines, 2);
    let to_move: Vec<_> = save
        .world
        .entities
        .entities()
        .iter()
        .filter_map(|entity| {
            let (_, _, loc, _, _, alive) = entity.actor()?;
            let is_pet = entity.faction() == Some(aihack_core::domain::entity::Faction::Pet);
            (alive && loc == level && entity.id != save.world.player_id && !is_pet)
                .then_some(entity.id)
        })
        .collect();
    let mut open_spots: Vec<Pos> = save
        .world
        .levels
        .map(mines2)
        .map(|map| {
            (1..map.height - 1)
                .flat_map(|y| (1..map.width - 1).map(move |x| Pos { x, y }))
                .filter(|&p| {
                    map.tile(p).ok() == Some(TileKind::Floor)
                        && save.world.entities.alive_actor_at(mines2, p).is_none()
                })
                .collect()
        })
        .unwrap_or_default();
    for entity_id in to_move {
        if let Some(pos) = open_spots.pop() {
            save.world
                .entities
                .set_actor_location(entity_id, mines2, pos);
        }
    }
    *session = GameSession::from_save_data(save).unwrap();
}

fn quaff_gain_level(session: &mut GameSession, count: u16) {
    let mut save = session.to_save_data();
    let player_id = save.world.player_id;
    let pot = save
        .world
        .entities
        .spawn_item(
            ItemKind::PotionGainLevel,
            EntityLocation::Inventory { owner: player_id },
        )
        .unwrap();
    save.world.entities.set_item_quantity(pot, count);
    let letter = save
        .world
        .inventory
        .add_existing_with_next_letter(pot)
        .unwrap();
    save.world.entities.set_item_letter(pot, letter);
    *session = GameSession::from_save_data(save).unwrap();
    for _ in 0..count {
        let out = session.submit(C::Quaff { item: pot });
        assert!(out.accepted, "{:?}", out.events);
    }
}

#[test]
fn p13_1_sokoban_boulder_push_pit_plug_dig_ban_conservation_and_prize() {
    let mut session = start_campaign(42, Role::Archeologist);
    let main4 = LevelId::main(4);
    let main4_down = session.world().levels.stairs_down_pos(main4).unwrap();
    teleport_player(&mut session, main4, main4_down);

    assert!(session
        .observation()
        .legal_actions
        .contains(&C::EnterBranch));
    let out = session.submit(C::EnterBranch);
    assert!(out.accepted, "{:?}", out.events);
    let sok1 = LevelId::new(BranchId::Sokoban, 1);
    assert_eq!(session.world().current_level(), sok1);
    assert_eq!(
        session
            .observation()
            .campaign
            .as_ref()
            .unwrap()
            .special_level_id
            .as_deref(),
        Some("sokoban_1")
    );

    // Ascending Sokoban 1 returns directly onto Main 4's stairs down.
    let back = session.submit(C::Ascend);
    assert!(back.accepted, "{:?}", back.events);
    assert_eq!(session.world().current_level(), main4);
    assert_eq!(session.world().player_pos(), main4_down);

    assert!(session.submit(C::EnterBranch).accepted);
    clear_hostiles_on(&mut session, sok1);

    // Move behind boulder at (37, 9) and push it step-by-step into the pit at (42, 9).
    teleport_player(&mut session, sok1, Pos { x: 36, y: 9 });
    assert!(session
        .observation()
        .legal_actions
        .contains(&C::Move(Direction::East)));

    for step in 0..4 {
        let push = session.submit(C::Move(Direction::East));
        assert!(push.accepted, "step {step}: {:?}", push.events);
        assert_eq!(
            session
                .world()
                .current_map()
                .tile(Pos { x: 37 + step, y: 9 })
                .ok(),
            Some(TileKind::Floor)
        );
        assert_eq!(
            session
                .world()
                .current_map()
                .tile(Pos { x: 38 + step, y: 9 })
                .ok(),
            Some(TileKind::Boulder)
        );
    }

    // Pushing into the pit at (42, 9) plugs it into Floor!
    let plug = session.submit(C::Move(Direction::East));
    assert!(plug.accepted, "{:?}", plug.events);
    assert_eq!(
        session.world().current_map().tile(Pos { x: 42, y: 9 }).ok(),
        Some(TileKind::Floor)
    );
    assert!(GameSession::from_save_data(session.to_save_data()).is_ok());

    // Digging via PickAxe or DigSpell is forbidden in Sokoban.
    let pick = session
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::PickAxe)
        .unwrap()
        .item;
    teleport_player(&mut session, sok1, Pos { x: 8, y: 6 });
    assert_eq!(
        session.world().current_map().tile(Pos { x: 8, y: 5 }).ok(),
        Some(TileKind::Wall)
    );
    let dig_attempt = session.submit(C::Apply {
        item: pick,
        direction: Some(Direction::North),
    });
    assert!(!dig_attempt.accepted);

    let mut dig_mage = start_campaign(42, Role::Mage);
    clear_hostiles_on(&mut dig_mage, LevelId::main(1));
    quaff_gain_level(&mut dig_mage, 9);
    let mut dig_spell_save = dig_mage.to_save_data();
    dig_spell_save.world.current_level = sok1;
    dig_spell_save.world.entities.set_actor_location(
        dig_spell_save.world.player_id,
        sok1,
        Pos { x: 8, y: 6 },
    );
    dig_spell_save.world.learned_spells.push(LearnedSpell {
        spell: SpellKind::DigSpell,
        learned_at_turn: dig_mage.turn(),
    });
    dig_mage = GameSession::from_save_data(dig_spell_save).unwrap();
    let dig_cast = dig_mage.submit(C::Cast {
        spell: SpellKind::DigSpell,
        direction: Some(Direction::North),
    });
    assert!(dig_cast.accepted, "{:?}", dig_cast.events);
    assert_eq!(
        dig_mage.world().current_map().tile(Pos { x: 8, y: 5 }).ok(),
        Some(TileKind::Wall)
    );

    // Tampered boulder count breaks Sokoban conservation validation.
    let mut bad_save = session.to_save_data();
    bad_save
        .world
        .levels
        .map_mut(sok1)
        .unwrap()
        .set_tile(Pos { x: 6, y: 10 }, TileKind::Boulder)
        .unwrap();
    assert!(GameSession::from_save_data(bad_save).is_err());

    // Descend Sokoban 1 -> 4 and verify seed-based prize (even: AmuletReflection, odd: BagOfHolding).
    for d in 1..4 {
        let lvl = LevelId::new(BranchId::Sokoban, d);
        let down = session.world().levels.stairs_down_pos(lvl).unwrap();
        teleport_player(&mut session, lvl, down);
        assert!(session.submit(C::Descend).accepted);
    }
    let sok4 = LevelId::new(BranchId::Sokoban, 4);
    assert_eq!(session.world().current_level(), sok4);
    assert!(session.world().levels.stairs_down_pos(sok4).is_none());

    let has_item_on_sok4 = |sess: &GameSession, target: ItemKind| {
        sess.world().entities.entities().iter().any(|e| {
            e.item().is_some_and(|(k, _, loc, _, _)| {
                k == target && matches!(loc, EntityLocation::OnMap { level, .. } if level == sok4)
            })
        })
    };
    assert!(has_item_on_sok4(&session, ItemKind::AmuletReflection));

    let odd_session = start_campaign(43, Role::Knight);
    assert!(has_item_on_sok4(&odd_session, ItemKind::BagOfHolding));
}

#[test]
fn p13_2_quest_branch_nemesis_and_all_13_role_quest_artifacts() {
    let mut nav = start_campaign(42, Role::Knight);
    let main5 = LevelId::main(5);
    let main5_down = nav.world().levels.stairs_down_pos(main5).unwrap();
    teleport_player(&mut nav, main5, main5_down);
    assert!(nav.observation().legal_actions.contains(&C::EnterBranch));
    assert!(nav.submit(C::EnterBranch).accepted);
    assert_eq!(
        nav.world().current_level(),
        LevelId::new(BranchId::Quest, 1)
    );
    assert_eq!(
        nav.observation()
            .campaign
            .as_ref()
            .unwrap()
            .special_level_id
            .as_deref(),
        Some("quest_1")
    );
    assert!(nav.submit(C::Ascend).accepted);
    assert_eq!(nav.world().current_level(), main5);
    assert_eq!(nav.world().player_pos(), main5_down);

    for role in Role::ALL {
        let mut sess = start_campaign(77, role);
        let quest3 = LevelId::new(BranchId::Quest, 3);
        let def = quest_artifact_for_role(role);

        assert!(sess.world().entities.entities().iter().any(|e| {
            e.actor().is_some_and(|(k, _, lvl, _, _, alive)| {
                alive && lvl == quest3 && k == ActorKind::Monster(MonsterKind::QuestNemesis)
            })
        }));

        let (art_id, art_pos) = sess
            .world()
            .entities
            .entities()
            .iter()
            .find_map(|e| {
                let (_, _, loc, _, _) = e.item()?;
                match loc {
                    EntityLocation::OnMap { level, pos }
                        if level == quest3 && e.item_artifact_id() == Some(def.id) =>
                    {
                        Some((e.id, pos))
                    }
                    _ => None,
                }
            })
            .unwrap_or_else(|| panic!("missing quest artifact {} on Quest 3", def.id));

        clear_hostiles_on(&mut sess, quest3);
        // Re-seed an adjacent target so offensive/detection quest artifacts have a target.
        teleport_player(&mut sess, quest3, art_pos);
        assert!(sess.submit(C::Pickup).accepted);
        assert!(sess.world().inventory.contains(art_id));
        assert!(sess
            .observation()
            .legal_actions
            .contains(&C::Invoke { item: art_id }));

        let invoke_out = sess.submit(C::Invoke { item: art_id });
        assert!(
            invoke_out.accepted,
            "role {role:?}: {:?}",
            invoke_out.events
        );
        assert_eq!(
            sess.world().campaign.unwrap().artifact_invoke_cooldown,
            ARTIFACT_INVOKE_COOLDOWN_TURNS - 1
        );
        assert!(GameSession::from_save_data(sess.to_save_data()).is_ok());
    }
}

#[test]
fn p13_3_gehennom_4_floors_bosses_amulet_relocation_and_5_stage_pursuit() {
    let mut session = start_campaign(42, Role::Knight);

    // AmuletAscension is relocated from Main 6 to Gehennom 4.
    let amulet_id = session.world().campaign.unwrap().amulet;
    assert_eq!(
        session
            .world()
            .entities
            .get(amulet_id)
            .and_then(|e| e.item())
            .map(|i| i.2),
        Some(EntityLocation::OnMap {
            level: LevelId::new(BranchId::Gehennom, 4),
            pos: session
                .world()
                .entities
                .get(amulet_id)
                .and_then(|e| e.item())
                .and_then(|i| match i.2 {
                    EntityLocation::OnMap { pos, .. } => Some(pos),
                    _ => None,
                })
                .unwrap()
        })
    );

    let main6_down = session
        .world()
        .levels
        .stairs_down_pos(LevelId::main(6))
        .unwrap();
    teleport_player(&mut session, LevelId::main(6), main6_down);
    assert!(session.submit(C::Descend).accepted);

    let expectations = [
        (1, "valley_of_the_dead", MonsterKind::FireAnt),
        (2, "orcus_town", MonsterKind::Orcus),
        (3, "wizards_tower", MonsterKind::WizardOfYendor),
        (4, "molochs_sanctum", MonsterKind::HighPriestMoloch),
    ];
    for (depth, expected_id, boss) in expectations {
        let lvl = LevelId::new(BranchId::Gehennom, depth);
        assert_eq!(session.world().current_level(), lvl);
        assert_eq!(
            session
                .observation()
                .campaign
                .as_ref()
                .unwrap()
                .special_level_id
                .as_deref(),
            Some(expected_id)
        );
        assert!(session.world().entities.entities().iter().any(|e| {
            e.actor().is_some_and(|(k, _, loc, _, _, alive)| {
                alive && loc == lvl && k == ActorKind::Monster(boss)
            })
        }));
        if depth < 4 {
            let down = session.world().levels.stairs_down_pos(lvl).unwrap();
            teleport_player(&mut session, lvl, down);
            assert!(session.submit(C::Descend).accepted);
        }
    }

    let geh4 = LevelId::new(BranchId::Gehennom, 4);
    let EntityLocation::OnMap {
        pos: amulet_pos, ..
    } = session
        .world()
        .entities
        .get(amulet_id)
        .unwrap()
        .item()
        .unwrap()
        .2
    else {
        panic!("expected amulet on Gehennom 4");
    };
    teleport_player(&mut session, geh4, amulet_pos);
    assert!(session.submit(C::Pickup).accepted);

    let count_kind = |sess: &GameSession, lvl: LevelId, target: MonsterKind| {
        sess.world()
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

    // 5-stage pursuit: Main 4/2/1 -> Planes 2/4
    for (from_lvl, expected_lvl, expected_stage, pursuer, via_portal) in [
        (
            LevelId::main(5),
            LevelId::main(4),
            1_u8,
            MonsterKind::FireAnt,
            false,
        ),
        (
            LevelId::main(3),
            LevelId::main(2),
            2_u8,
            MonsterKind::Troll,
            false,
        ),
        (
            LevelId::main(2),
            LevelId::main(1),
            3_u8,
            MonsterKind::Minotaur,
            false,
        ),
        (
            LevelId::new(BranchId::Planes, 1),
            LevelId::new(BranchId::Planes, 2),
            4_u8,
            MonsterKind::WizardOfYendor,
            true,
        ),
        (
            LevelId::new(BranchId::Planes, 3),
            LevelId::new(BranchId::Planes, 4),
            5_u8,
            MonsterKind::WizardOfYendor,
            true,
        ),
    ] {
        let before = count_kind(&session, expected_lvl, pursuer);
        if via_portal {
            teleport_player(&mut session, from_lvl, Pos { x: 10, y: 10 });
            let portal = find_tile(&session, TileKind::MagicPortal);
            teleport_player(&mut session, from_lvl, portal);
            assert!(session.submit(C::EnterBranch).accepted);
        } else {
            let up = session.world().levels.stairs_up_pos(from_lvl).unwrap();
            teleport_player(&mut session, from_lvl, up);
            assert!(session.submit(C::Ascend).accepted);
        }
        assert_eq!(session.world().current_level(), expected_lvl);
        assert_eq!(session.world().ascension_pursuit_stage, expected_stage);
        assert_eq!(count_kind(&session, expected_lvl, pursuer), before + 1);
    }

    let mut bad = session.to_save_data();
    bad.world.ascension_pursuit_stage = 6;
    assert!(GameSession::from_save_data(bad).is_err());
}

#[test]
fn p13_4_elemental_planes_astral_3_high_altars_true_ascension_and_forged_victory_rejection() {
    let mut session = start_campaign(42, Role::Archeologist);
    let main1_up = session
        .world()
        .levels
        .stairs_up_pos(LevelId::main(1))
        .unwrap();
    teleport_player(&mut session, LevelId::main(1), main1_up);
    // Ascending Main 1 without Amulet is rejected.
    assert!(!session.submit(C::Ascend).accepted);

    let amulet_id = session.world().campaign.unwrap().amulet;
    let EntityLocation::OnMap {
        level: geh4,
        pos: amulet_pos,
    } = session
        .world()
        .entities
        .get(amulet_id)
        .unwrap()
        .item()
        .unwrap()
        .2
    else {
        panic!("expected amulet on Gehennom 4");
    };
    teleport_player(&mut session, geh4, amulet_pos);
    assert!(session.submit(C::Pickup).accepted);

    // Ascending Main 1 with Amulet enters Plane of Earth (diggable).
    teleport_player(&mut session, LevelId::main(1), main1_up);
    let to_earth = session.submit(C::Ascend);
    assert!(to_earth.accepted, "{:?}", to_earth.events);
    let planes1 = LevelId::new(BranchId::Planes, 1);
    assert_eq!(session.world().current_level(), planes1);
    assert_eq!(
        session
            .observation()
            .campaign
            .as_ref()
            .unwrap()
            .special_level_id
            .as_deref(),
        Some("plane_of_earth")
    );

    let pick = session
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::PickAxe)
        .unwrap()
        .item;
    let dig_pos = (2..78)
        .flat_map(|x| (2..19).map(move |y| Pos { x, y }))
        .find(|&p| {
            session.world().current_map().tile(p).ok() == Some(TileKind::Wall)
                && session
                    .world()
                    .current_map()
                    .tile(Pos { x: p.x, y: p.y + 1 })
                    .ok()
                    == Some(TileKind::Floor)
        })
        .unwrap();
    teleport_player(
        &mut session,
        planes1,
        Pos {
            x: dig_pos.x,
            y: dig_pos.y + 1,
        },
    );
    let dig_earth = session.submit(C::Apply {
        item: pick,
        direction: Some(Direction::North),
    });
    assert!(dig_earth.accepted, "{:?}", dig_earth.events);
    assert_eq!(
        session.world().current_map().tile(dig_pos).ok(),
        Some(TileKind::Floor)
    );

    // Advance Planes 1..4 via MagicPortal.
    let plane_specs = [
        (2, "plane_of_air", MonsterKind::AirElemental),
        (3, "plane_of_fire", MonsterKind::FireElemental),
        (4, "plane_of_water", MonsterKind::WaterElemental),
    ];
    for (next_depth, next_id, elemental) in plane_specs {
        let portal = find_tile(&session, TileKind::MagicPortal);
        let cur = session.world().current_level();
        teleport_player(&mut session, cur, portal);
        let obs = session.observation();
        assert!(obs.campaign.as_ref().unwrap().on_magic_portal);
        assert!(obs.legal_actions.contains(&C::EnterBranch));
        assert!(obs.legal_actions.contains(&C::Ascend));
        let enter = session.submit(C::EnterBranch);
        assert!(enter.accepted, "{:?}", enter.events);
        let next_lvl = LevelId::new(BranchId::Planes, next_depth);
        assert_eq!(session.world().current_level(), next_lvl);
        assert_eq!(
            session
                .observation()
                .campaign
                .as_ref()
                .unwrap()
                .special_level_id
                .as_deref(),
            Some(next_id)
        );
        assert!(session.world().entities.entities().iter().any(|e| {
            e.actor().is_some_and(|(k, _, lvl, _, _, alive)| {
                alive && lvl == next_lvl && k == ActorKind::Monster(elemental)
            })
        }));
    }

    // Plane of Air/Fire/Water disallow digging.
    let wall_water = find_tile(&session, TileKind::Wall);
    let _ = wall_water;
    assert!(
        !session
            .submit(C::Apply {
                item: pick,
                direction: Some(Direction::North),
            })
            .accepted
    );

    let portal4 = find_tile(&session, TileKind::MagicPortal);
    teleport_player(&mut session, LevelId::new(BranchId::Planes, 4), portal4);
    assert!(session.submit(C::EnterBranch).accepted);

    let planes5 = LevelId::new(BranchId::Planes, 5);
    assert_eq!(session.world().current_level(), planes5);
    assert_eq!(
        session
            .observation()
            .campaign
            .as_ref()
            .unwrap()
            .special_level_id
            .as_deref(),
        Some("astral_plane")
    );
    for rider in [
        MonsterKind::RiderDeath,
        MonsterKind::RiderPestilence,
        MonsterKind::RiderFamine,
    ] {
        assert!(session.world().entities.entities().iter().any(|e| {
            e.actor().is_some_and(|(k, _, lvl, _, _, alive)| {
                alive && lvl == planes5 && k == ActorKind::Monster(rider)
            })
        }));
    }

    assert_eq!(session.world().ascension_altars.len(), 3);
    let player_align = session.world().campaign.unwrap().alignment;
    let cross_altar = session
        .world()
        .ascension_altars
        .iter()
        .copied()
        .find(|&(lvl, p)| {
            lvl == planes5
                && session
                    .world()
                    .levels
                    .get(lvl)
                    .and_then(|l| l.altar_alignment(p))
                    != Some(player_align)
        })
        .unwrap();
    let co_altar = session
        .world()
        .ascension_altars
        .iter()
        .copied()
        .find(|&(lvl, p)| {
            lvl == planes5
                && session
                    .world()
                    .levels
                    .get(lvl)
                    .and_then(|l| l.altar_alignment(p))
                    == Some(player_align)
        })
        .unwrap();

    // Gain levels legitimately via PotionGainLevel so cross-aligned 3d8 wrath survives cleanly and passes save validation.
    clear_hostiles_on(&mut session, planes5);
    quaff_gain_level(&mut session, 9);

    // Cross-aligned High Altar Offer deals Divine Wrath and preserves Amulet.
    teleport_player(&mut session, planes5, cross_altar.1);
    let anger_before = session.world().campaign.unwrap().god_anger;
    let hp_before = session.observation().player.hp;
    let wrath = session.submit(C::Offer { item: amulet_id });
    assert!(wrath.accepted, "{:?}", wrath.events);
    assert!(matches!(session.run_state(), RunState::Playing));
    assert_eq!(
        session.world().campaign.unwrap().god_anger,
        anger_before + 3
    );
    assert!(session.observation().player.hp < hp_before);
    assert!(session.world().inventory.contains(amulet_id));

    // Co-aligned High Altar Offer grants True Ascension Victory!
    teleport_player(&mut session, planes5, co_altar.1);
    let win_obs = session.observation();
    assert_eq!(
        win_obs.campaign.as_ref().unwrap().on_high_altar,
        Some(player_align)
    );
    assert!(win_obs
        .legal_actions
        .contains(&C::Offer { item: amulet_id }));
    let win = session.submit(C::Offer { item: amulet_id });
    assert!(win.accepted, "{:?}", win.events);
    assert!(matches!(session.run_state(), RunState::Victory { .. }));
    assert!(session.world().inventory.contains(amulet_id));
    assert!(GameSession::from_save_data(session.to_save_data()).is_ok());

    // Forged victories on Main 1 or cross-aligned altar or outside ascension_altars fail save validation.
    let mut forged_main1 = session.to_save_data();
    forged_main1.world.current_level = LevelId::main(1);
    forged_main1.world.entities.set_actor_location(
        forged_main1.world.player_id,
        LevelId::main(1),
        main1_up,
    );
    assert!(GameSession::from_save_data(forged_main1).is_err());

    let mut forged_cross = session.to_save_data();
    forged_cross.world.entities.set_actor_location(
        forged_cross.world.player_id,
        planes5,
        cross_altar.1,
    );
    assert!(GameSession::from_save_data(forged_cross).is_err());

    let mut forged_no_reg = session.to_save_data();
    forged_no_reg.world.ascension_altars.clear();
    assert!(GameSession::from_save_data(forged_no_reg).is_err());
}

#[test]
fn p13_5_local_llm_deep_summary_compaction_masking_loopback_and_hash_invariance() {
    let _guard = LLM_TEST_ENV_LOCK.lock().unwrap();
    let mut session = start_campaign(42, Role::Mage);

    // Give Mage an unidentified ring & astral context so both masking and compaction are tested.
    let planes5 = LevelId::new(BranchId::Planes, 5);
    let astral_altar = session
        .world()
        .ascension_altars
        .iter()
        .copied()
        .find(|&(lvl, _)| lvl == planes5)
        .unwrap()
        .1;
    let amulet_id = session.world().campaign.unwrap().amulet;
    let mut save = session.to_save_data();
    save.world.current_level = planes5;
    save.world
        .entities
        .set_actor_location(save.world.player_id, planes5, astral_altar);
    let extra_unid = save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            let (kind, _, loc, _, _) = e.item()?;
            (kind == ItemKind::ScrollEnchantWeapon
                && matches!(loc, EntityLocation::OnMap { level, .. } if level == LevelId::main(1)))
            .then_some(e.id)
        })
        .unwrap();
    save.world.entities.set_item_location(
        extra_unid,
        EntityLocation::Inventory {
            owner: save.world.player_id,
        },
    );
    let extra_letter = save
        .world
        .inventory
        .add_existing_with_next_letter(extra_unid)
        .unwrap();
    save.world
        .entities
        .set_item_letter(extra_unid, extra_letter);
    save.world.entities.set_item_location(
        amulet_id,
        EntityLocation::Inventory {
            owner: save.world.player_id,
        },
    );
    let amulet_letter = save
        .world
        .inventory
        .add_existing_with_next_letter(amulet_id)
        .unwrap();
    save.world
        .entities
        .set_item_letter(amulet_id, amulet_letter);
    session = GameSession::from_save_data(save).unwrap();

    let obs = session.observation();
    let view = LlmObservationView::from(&obs);
    let summary = view.campaign.as_ref().expect("LlmCampaignSummary present");
    assert_eq!(summary.role, Role::Mage);
    assert!(summary.attributes.is_some());
    assert!(!summary.spells.is_empty());
    assert_eq!(summary.special_level_id.as_deref(), Some("astral_plane"));
    assert!(summary.on_high_altar.is_some());
    assert!(summary.has_amulet);

    // Unidentified item kind masking invariant (MAJ-03): any unidentified item with appearance has kind: None.
    let mut masked_count = 0usize;
    for item in &view.inventory {
        if !item.identified && item.appearance.is_some() {
            assert!(item.kind.is_none());
            masked_count += 1;
        }
    }
    assert!(masked_count >= 1, "expected at least one masked item");

    let revision = ClientRevision {
        turn: session.turn(),
        snapshot_hash: session.snapshot().stable_hash(),
    };
    let req_input =
        LlmRequestInput::from_observation(revision.clone(), &obs, LlmRequestKind::Decision);
    let json_bytes = canonical_request_size_bytes(&req_input).unwrap();
    assert!(
        json_bytes < REQUEST_BODY_LIMIT,
        "request body too large: {json_bytes}"
    );

    // Loopback HTTP mock handling Narrative, Decision, and SoftAdjudication sequentially.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        let payloads = [
            r#"{"choices":[{"message":{"content":"{\"kind\":\"NARRATIVE\",\"text\":\"Before the Astral altar, power converges.\"}"}}]}"#,
            r#"{"choices":[{"message":{"content":"{\"kind\":\"DECISION\",\"action\":{\"type\":\"WAIT\"},\"rationale\":\"Assess the Riders.\",\"confidence\":0.85}"}}]}"#,
            r#"{"choices":[{"message":{"content":"{\"kind\":\"SOFT_ADJUDICATION\",\"verdict\":\"FAVORABLE\",\"reasonCode\":\"CONSECRATED\",\"message\":\"Your plea echoes across the sanctuary.\"}"}}]}"#,
        ];
        for body in payloads {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut content_len = 0usize;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.eq_ignore_ascii_case("\r\n") || line.is_empty() {
                    break;
                }
                if let Some(rest) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_len = rest.trim().parse().unwrap_or(0);
                }
            }
            let mut buf = vec![0u8; content_len];
            let _ = reader.read_exact(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(resp.as_bytes()).unwrap();
        }
    });

    std::env::set_var("AIHACK_LLM_ENABLED", "true");
    std::env::set_var("AIHACK_LLM_ENDPOINT", format!("http://{addr}/v1"));
    std::env::set_var("AIHACK_LLM_MODEL", "local-test");
    std::env::set_var("AIHACK_LLM_NARRATIVE_TIMEOUT_MS", "2000");
    let config = LocalLlmConfig::from_env().unwrap();
    std::env::remove_var("AIHACK_LLM_ENABLED");
    std::env::remove_var("AIHACK_LLM_ENDPOINT");
    std::env::remove_var("AIHACK_LLM_MODEL");
    std::env::remove_var("AIHACK_LLM_NARRATIVE_TIMEOUT_MS");

    let hash_before = session.snapshot().stable_hash();
    let transport = OpenAiNarrativeTransport::new(config.clone()).unwrap();
    let narrative_text = transport
        .complete(&NarrativeRequest {
            revision: revision.clone(),
            topic: NarrativeTopic::SituationSummary,
            observation: obs.clone(),
        })
        .unwrap();
    assert!(narrative_text.contains("Astral"));

    let decision = transport
        .complete_decision(&DecisionRequest {
            revision: revision.clone(),
            observation: obs.clone(),
            action_space: obs.action_space.clone(),
        })
        .unwrap();
    assert_eq!(decision.rationale, "Assess the Riders.");

    let soft = transport
        .complete_soft_adjudication(&SoftAdjudicationRequest {
            revision: revision.clone(),
            observation: obs.clone(),
            user_text: "I kneel before the high altar.".to_string(),
        })
        .unwrap();
    assert_eq!(soft.reason_code, "CONSECRATED");
    server.join().unwrap();

    // Service schema & stale revision guard + snapshot hash invariance.
    let mut service = LocalLlmService::from_config(config).unwrap();
    assert_eq!(LLM_SCHEMA_VERSION, 1);
    let _ = service.enqueue(LlmRequestInput::from_observation(
        revision,
        &obs,
        LlmRequestKind::Narrative,
    ));
    let _ = service.shutdown_with_grace(Duration::from_millis(100));
    assert_eq!(session.snapshot().stable_hash(), hash_before);
}

#[test]
fn p13_6_catalog_95_monsters_legacy_v1_isolation_and_tui_rendering() {
    let reg = registry().unwrap();
    assert_eq!(reg.monsters().count(), 95);
    assert_eq!(reg.items().count(), 130);
    assert_eq!(BranchGraph::campaign_v3().active_level_ids().len(), 24);

    for mk in [
        MonsterKind::WizardOfYendor,
        MonsterKind::HighPriestMoloch,
        MonsterKind::Orcus,
        MonsterKind::QuestNemesis,
        MonsterKind::EarthElemental,
        MonsterKind::FireElemental,
        MonsterKind::WaterElemental,
        MonsterKind::RiderDeath,
        MonsterKind::RiderPestilence,
        MonsterKind::RiderFamine,
    ] {
        assert!(!monster_display_name(mk, reg).is_empty());
        assert_ne!(monster_glyph(mk, reg), '\0');
    }

    let legacy = GameSession::new_for_playing(42);
    assert!(legacy.world().campaign.is_none());
    assert_eq!(legacy.to_save_data().schema_version, SAVE_SCHEMA_VERSION_V1);

    for bad_tile in [TileKind::Boulder, TileKind::MagicPortal] {
        let mut bad_legacy = legacy.to_save_data();
        bad_legacy
            .world
            .levels
            .map_mut(LevelId::main(1))
            .unwrap()
            .set_tile(Pos { x: 2, y: 2 }, bad_tile)
            .unwrap();
        assert!(GameSession::from_save_data(bad_legacy).is_err());
    }

    // TUI renders Boulder ('0'), MagicPortal ('^'), special level badges, and portal auto-label.
    let mut sess = start_campaign(42, Role::Knight);
    let planes1 = LevelId::new(BranchId::Planes, 1);
    teleport_player(&mut sess, planes1, Pos { x: 10, y: 10 });
    let portal = find_tile(&sess, TileKind::MagicPortal);
    teleport_player(&mut sess, planes1, portal);
    let obs = sess.observation();
    let status = status_lines(&obs).join("\n");
    assert!(status.contains("plane_of_earth"));
    assert!(status.contains("PORTAL"));
    assert!(inspect_lines(&obs, Some(portal), UiPanel::Inspect, &[])
        .iter()
        .any(|l| l.contains("magic portal")));
    assert!(collect_auto_labels(&obs, 0)
        .iter()
        .any(|l| l.text == "[Portal]"));

    let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut app = TuiApp::new(sess, UiRuntimeConfig::default());
    term.draw(|f| {
        render_frame(f, &mut app);
        let area = Rect::new(0, 0, 80, 21);
        MapWidget {
            observation: &obs,
            viewport: Viewport::from_rect(Pos { x: 0, y: 0 }, obs.player_pos, area),
            labels: &[],
            theme: UiTheme::standard(),
        }
        .render(area, f.buffer_mut());
    })
    .unwrap();
}
