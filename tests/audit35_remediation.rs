use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use aihack_content::{item_color, monster_resistances, registry, weighted_monster_kinds_for_depth};
use aihack_core::{
    action::{CommandIntent as C, DipTarget, EngravingMethod, EngravingText, InventoryAction},
    campaign::{
        Alignment, LearnedSpell, Race, Role, SpellKind, ARTIFACT_INVOKE_COOLDOWN_TURNS,
        MAX_GOD_ANGER, MAX_LUCK, MAX_PRAYER_TIMEOUT,
    },
    domain::{
        entity::{ActorKind, EntityLocation},
        inventory::MAX_INVENTORY_SLOTS,
        item::{BucStatus, ItemKind},
        level::{Engraving, EngravingKind},
        monster::MonsterKind,
        status::Resistances,
        tile::{DoorState, TileKind, TrapKind},
    },
    event::{GameEvent, MessagePriority},
    ids::{BranchId, EntityId, LevelId},
    position::{Direction, Pos},
    run_state::RunState,
};
use aihack_headless::HeadlessPolicy;
use aihack_llm::service::LlmObservationView;
use aihack_runtime::{
    appearance::{appearance_for_kind, color_for_appearance},
    GameSession,
};
use aihack_tui::tui::{
    compute_layout, input, render_frame, render_panels, runtime_event_to_candidate, TuiApp,
    UiClock, UiCommandCandidate, UiRuntimeConfig,
};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{backend::TestBackend, style::Color, Terminal};

#[derive(Default)]
struct TestClock(AtomicU64);

impl UiClock for TestClock {
    fn now(&self) -> Duration {
        Duration::from_millis(self.0.load(Ordering::Relaxed))
    }
}

fn start_campaign(seed: u64, role: Role) -> GameSession {
    let mut session = GameSession::try_new(seed).unwrap();
    assert!(session.submit(C::Wait).accepted);
    let outcome = session.submit(C::StartCampaign { role });
    assert!(outcome.accepted, "{:?}", outcome.events);
    session
}

fn start_build(seed: u64, role: Role, race: Race, alignment: Alignment) -> GameSession {
    let mut session = GameSession::try_new(seed).unwrap();
    assert!(session.submit(C::Wait).accepted);
    let outcome = session.submit(C::StartCampaignWithBuild {
        role,
        race,
        alignment,
    });
    assert!(outcome.accepted, "{:?}", outcome.events);
    session
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

fn find_floor_adjacent_to(
    save: &aihack_runtime::save::SaveDataV1,
    level_id: LevelId,
    pred: impl Fn(TileKind) -> bool,
) -> (Pos, Pos, Direction) {
    let map = save.world.levels.map(level_id).unwrap();
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let target = Pos { x, y };
            if !map.tile(target).ok().is_some_and(&pred) {
                continue;
            }
            for dir in Direction::ALL {
                let adj = target.offset(dir.delta());
                if map.contains(adj) && map.tile(adj).ok() == Some(TileKind::Floor) {
                    let move_dir = Direction::ALL
                        .into_iter()
                        .find(|d| adj.offset(d.delta()) == target)
                        .unwrap();
                    return (adj, target, move_dir);
                }
            }
        }
    }
    panic!("failed to find floor adjacent to target on {level_id:?}");
}

fn find_floor_with_east_floor(
    save: &aihack_runtime::save::SaveDataV1,
    level_id: LevelId,
) -> (Pos, Pos) {
    let map = save.world.levels.map(level_id).unwrap();
    for y in 2..map.height - 2 {
        for x in 2..map.width - 3 {
            let p = Pos { x, y };
            let e = Pos { x: x + 1, y };
            if map.tile(p).ok() == Some(TileKind::Floor)
                && map.tile(e).ok() == Some(TileKind::Floor)
            {
                return (p, e);
            }
        }
    }
    panic!("failed to find adjacent floor pair on {level_id:?}");
}

fn find_los_blocked_floor_pair(
    save: &aihack_runtime::save::SaveDataV1,
    level_id: LevelId,
) -> (Pos, Pos) {
    let map = save.world.levels.map(level_id).unwrap();
    for y in 2..map.height - 2 {
        for x in 2..map.width - 6 {
            let left = Pos { x, y };
            if map.tile(left).ok() != Some(TileKind::Floor) {
                continue;
            }
            for d in 2..=5_i16 {
                let right = Pos { x: x + d, y };
                if map.tile(right).ok() == Some(TileKind::Floor)
                    && (1..d).any(|k| map.tile(Pos { x: x + k, y }).ok() == Some(TileKind::Wall))
                {
                    return (left, right);
                }
            }
        }
    }
    for x in 2..map.width - 2 {
        for y in 2..map.height - 6 {
            let top = Pos { x, y };
            if map.tile(top).ok() != Some(TileKind::Floor) {
                continue;
            }
            for d in 2..=5_i16 {
                let bottom = Pos { x, y: y + d };
                if map.tile(bottom).ok() == Some(TileKind::Floor)
                    && (1..d).any(|k| map.tile(Pos { x, y: y + k }).ok() == Some(TileKind::Wall))
                {
                    return (top, bottom);
                }
            }
        }
    }
    panic!("failed to find wall-separated floors on {level_id:?}");
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
    save.world.entities.set_item_buc_known(id, true);
    save.world.entities.set_item_enchant_known(id, true);
    id
}

fn spawn_monster_at(
    save: &mut aihack_runtime::save::SaveDataV1,
    kind: MonsterKind,
    level: LevelId,
    pos: Pos,
) -> EntityId {
    let id = save.world.entities.spawn_monster(kind, pos).unwrap();
    save.world.entities.set_actor_location(id, level, pos);
    id
}

fn send_key(
    app: &mut TuiApp,
    clock: &TestClock,
    key: KeyCode,
    w: u16,
    h: u16,
) -> Option<UiCommandCandidate> {
    clock.0.fetch_add(1000, Ordering::Relaxed);
    app.release_transition_gesture_on_idle();
    app.release_transition_gesture_on_idle();
    let cand = runtime_event_to_candidate(
        Event::Key(KeyEvent::new_with_kind(
            key,
            KeyModifiers::NONE,
            KeyEventKind::Press,
        )),
        w,
        h,
        app,
    )?;
    assert!(!app.handle_candidate_owned(cand).unwrap());
    Some(cand)
}

fn click_ui(app: &mut TuiApp, clock: &TestClock, label: &str, width: u16, height: u16) {
    clock.0.fetch_add(1000, Ordering::Relaxed);
    app.release_transition_gesture_on_idle();
    app.release_transition_gesture_on_idle();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render_frame(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let (x, y) = (0..height)
        .find_map(|y| {
            let line: String = (0..width).map(|x| buffer[(x, y)].symbol()).collect();
            line.find(label)
                .map(|byte| (line[..byte].chars().count() as u16, y))
        })
        .unwrap_or_else(|| panic!("missing CTA label {label:?} at {width}x{height}"));
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
    .unwrap_or_else(|| panic!("click at ({x}, {y}) for {label:?} did not map to candidate"));
    assert!(!app.handle_candidate_owned(candidate).unwrap());
}

#[test]
fn crit01_action_space_priority_and_tui_legal_actions_above_64() {
    let mut save = start_build(42, Role::Mage, Race::Elf, Alignment::Neutral).to_save_data();
    let main2 = LevelId::main(2);
    let shop_pos = find_tile(&save, main2, |t| t == TileKind::Shop);
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world.current_level = main2;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main2, shop_pos);
    save.world.gold = 500;
    for _ in 0..8 {
        add_inventory_item(&mut save, ItemKind::Dagger, BucStatus::Uncursed, 0);
        add_inventory_item(&mut save, ItemKind::PotionHealing, BucStatus::Uncursed, 0);
    }
    let wand = add_inventory_item(&mut save, ItemKind::WandFire, BucStatus::Uncursed, 0);
    save.world.entities.set_item_charges(wand, Some(4));

    let sess = GameSession::from_save_data(save).unwrap();
    let obs = sess.observation();
    assert!(obs.legal_actions.len() > 64);
    assert!(obs.action_space.commands.len() <= 64);

    for dir in Direction::ALL {
        let mv = C::Move(dir);
        if obs.legal_actions.contains(&mv) {
            assert!(
                obs.action_space
                    .commands
                    .contains(&aihack_core::action::ActionIntent::Command(mv)),
                "bounded action_space must keep Move({dir:?}) ahead of combinatorial Tier 3 actions"
            );
        }
    }
    assert!(obs
        .action_space
        .commands
        .contains(&aihack_core::action::ActionIntent::Command(C::Wait)));
    assert!(obs
        .action_space
        .commands
        .contains(&aihack_core::action::ActionIntent::Command(C::Search)));

    assert_eq!(
        input::key_to_candidate('d', &obs),
        Some(UiCommandCandidate::BeginAction('d'))
    );
    assert_eq!(
        input::key_to_candidate('$', &obs),
        Some(UiCommandCandidate::BeginAction('$'))
    );
    assert_eq!(
        input::key_to_candidate('v', &obs),
        Some(UiCommandCandidate::BeginAction('v'))
    );
    assert!(input::command_panel_ctas(&obs)
        .iter()
        .any(|c| c.label == "[i] Inventory" && c.candidate.is_some()));
}

#[test]
fn crit02_rejected_commands_rollback_snapshot_hash_invariantly() {
    // 1. items::read on a non-scroll must reject without consuming/mutating.
    let mut sess = start_campaign(42, Role::Knight);
    let non_scroll = sess
        .observation()
        .inventory
        .iter()
        .find(|i| i.kind == ItemKind::LongSword || i.kind == ItemKind::FoodRation)
        .unwrap()
        .item;
    let before = sess.snapshot().stable_hash();
    let out = sess.submit(C::Read { item: non_scroll });
    assert!(!out.accepted);
    assert_eq!(sess.snapshot().stable_hash(), before);

    // 2. container_take on trapped chest with invalid child OR full inventory rejects cleanly
    // without tripping trap or mutating HP/world.
    let mut cont_save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut cont_save, LevelId::main(6));
    let p_pos = cont_save
        .world
        .entities
        .actor_location(cont_save.world.player_id)
        .unwrap()
        .1;
    let main1 = LevelId::main(1);
    let chest = cont_save
        .world
        .entities
        .spawn_item(
            ItemKind::Chest,
            EntityLocation::OnMap {
                level: main1,
                pos: p_pos,
            },
        )
        .unwrap();
    cont_save
        .world
        .entities
        .set_item_container_trapped(chest, true);
    let child = cont_save
        .world
        .entities
        .spawn_item(
            ItemKind::Dagger,
            EntityLocation::InContainer { container: chest },
        )
        .unwrap();
    cont_save.world.entities.push_contained_item(chest, child);
    while cont_save.world.inventory.entries.len() < MAX_INVENTORY_SLOTS {
        add_inventory_item(&mut cont_save, ItemKind::Dagger, BucStatus::Uncursed, 0);
    }
    let mut cont_sess = GameSession::from_save_data(cont_save).unwrap();
    let cont_before = cont_sess.snapshot().stable_hash();
    assert!(
        !cont_sess
            .submit(C::ContainerTake {
                container: chest,
                item: EntityId(999_999),
            })
            .accepted
    );
    assert_eq!(cont_sess.snapshot().stable_hash(), cont_before);
    assert!(
        !cont_sess
            .submit(C::ContainerTake {
                container: chest,
                item: child,
            })
            .accepted
    );
    assert_eq!(cont_sess.snapshot().stable_hash(), cont_before);
    assert_eq!(
        cont_sess
            .world()
            .entities
            .get(chest)
            .unwrap()
            .item_container_trapped(),
        Some(true)
    );

    // 3. dip splitting a stacked potion when inventory has 52/52 full slots rejects without consuming.
    let mut dip_save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut dip_save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let fountain = find_tile(&dip_save, main1, |t| t == TileKind::Fountain);
    dip_save
        .world
        .entities
        .set_actor_location(dip_save.world.player_id, main1, fountain);
    let stacked_potion = add_inventory_item(
        &mut dip_save,
        ItemKind::PotionHealing,
        BucStatus::Uncursed,
        0,
    );
    dip_save.world.entities.set_item_quantity(stacked_potion, 2);
    while dip_save.world.inventory.entries.len() < MAX_INVENTORY_SLOTS {
        add_inventory_item(&mut dip_save, ItemKind::Dagger, BucStatus::Uncursed, 0);
    }
    let mut dip_sess = GameSession::from_save_data(dip_save).unwrap();
    let dip_before = dip_sess.snapshot().stable_hash();
    assert!(
        !dip_sess
            .submit(C::Dip {
                item: stacked_potion,
                target: DipTarget::Tile,
            })
            .accepted
    );
    assert_eq!(dip_sess.snapshot().stable_hash(), dip_before);

    // 4. engrave with WandFire when MAX_ENGRAVINGS_PER_LEVEL(32) reached must not consume charges.
    let mut eng_save = start_campaign(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut eng_save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let (p_pos, _) = find_floor_with_east_floor(&eng_save, main1);
    eng_save
        .world
        .entities
        .set_actor_location(eng_save.world.player_id, main1, p_pos);
    let wand = add_inventory_item(&mut eng_save, ItemKind::WandFire, BucStatus::Uncursed, 0);
    eng_save.world.entities.set_item_charges(wand, Some(4));
    {
        let lvl = eng_save.world.levels.get_mut(main1).unwrap();
        for idx in 0..32 {
            lvl.engravings.push(Engraving {
                pos: Pos {
                    x: (idx as i16) + 1,
                    y: 19,
                },
                text: "old".to_string(),
                kind: EngravingKind::Dust,
            });
        }
    }
    let mut eng_sess = GameSession::from_save_data(eng_save).unwrap();
    let eng_before = eng_sess.snapshot().stable_hash();
    assert!(
        !eng_sess
            .submit(C::Engrave {
                method: EngravingMethod::Item(wand),
                text: EngravingText::Elbereth,
            })
            .accepted
    );
    assert_eq!(eng_sess.snapshot().stable_hash(), eng_before);
}

#[test]
fn crit03_bump_attack_p9_terrain_and_ascension_pursuit_avoids_hazards() {
    let cases = [
        (LevelId::main(1), TileKind::Fountain, false),
        (LevelId::main(1), TileKind::DryFountain, true),
        (LevelId::main(2), TileKind::Sink, false),
        (LevelId::main(3), TileKind::Water, false),
        (LevelId::main(5), TileKind::Lava, false),
    ];
    for (lvl, terrain, convert_to_dry) in cases {
        let mut save = start_campaign(42, Role::Knight).to_save_data();
        park_all_monsters_on(&mut save, LevelId::main(6));
        let query = if convert_to_dry {
            TileKind::Fountain
        } else {
            terrain
        };
        let (p_pos, m_pos, dir) = find_floor_adjacent_to(&save, lvl, |t| t == query);
        if convert_to_dry {
            save.world
                .levels
                .map_mut(lvl)
                .unwrap()
                .set_tile(m_pos, TileKind::DryFountain)
                .unwrap();
        }
        save.world.current_level = lvl;
        save.world
            .entities
            .set_actor_location(save.world.player_id, lvl, p_pos);
        save.world.luck = 10;
        let sword = add_inventory_item(&mut save, ItemKind::LongSword, BucStatus::Blessed, 5);
        save.world.inventory.equip_melee(sword);
        let target = spawn_monster_at(&mut save, MonsterKind::Ogre, lvl, m_pos);

        let mut sess = GameSession::from_save_data(save).unwrap();
        assert!(
            sess.observation().legal_actions.contains(&C::Move(dir)),
            "must allow bump-attack onto monster standing on {terrain:?}"
        );
        let hp_before = sess.world().entities.actor_stats(target).unwrap().hp;
        let mut damaged = false;
        for _ in 0..20 {
            let out = sess.submit(C::Move(dir));
            assert!(out.accepted);
            assert_eq!(sess.world().player_pos(), p_pos);
            let alive = sess
                .world()
                .entities
                .get(target)
                .is_some_and(|e| e.is_alive_actor());
            let hp_after = sess
                .world()
                .entities
                .actor_stats(target)
                .map(|s| s.hp)
                .unwrap_or(0);
            if !alive || hp_after < hp_before {
                damaged = true;
                break;
            }
        }
        assert!(damaged, "expected bump-attack damage on {terrain:?}");
    }

    // Ascension pursuit spawn across Lava level (Main 5) and Water level (Main 3) never picks Water or Lava.
    let mut asc_save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut asc_save, LevelId::main(1));
    let amulet_id = asc_save
        .world
        .entities
        .entities()
        .iter()
        .find(|e| {
            e.item()
                .is_some_and(|(k, ..)| k == ItemKind::AmuletAscension)
        })
        .unwrap()
        .id;
    let owner = asc_save.world.player_id;
    let amulet_letter = asc_save
        .world
        .inventory
        .add_existing_with_next_letter(amulet_id)
        .unwrap();
    asc_save
        .world
        .entities
        .set_item_letter(amulet_id, amulet_letter);
    asc_save
        .world
        .entities
        .set_item_location(amulet_id, EntityLocation::Inventory { owner });

    let main6 = LevelId::main(6);
    let main5 = LevelId::main(5);
    let stairs6_up = find_tile(&asc_save, main6, |t| t == TileKind::StairsUp);
    asc_save.world.current_level = main6;
    asc_save
        .world
        .entities
        .set_actor_location(owner, main6, stairs6_up);

    let mut asc_sess = GameSession::from_save_data(asc_save).unwrap();
    assert!(asc_sess.submit(C::Ascend).accepted);
    assert_eq!(asc_sess.world().current_level(), main5);
    for entity in asc_sess.world().entities.entities() {
        if let Some(ActorKind::Monster(_)) = entity.actor_kind() {
            if let Some((lvl, pos)) = asc_sess.world().entities.actor_location(entity.id) {
                if lvl == main5 {
                    let tile = asc_sess.world().map(main5).tile(pos).unwrap();
                    assert!(
                        !matches!(tile, TileKind::Water | TileKind::Lava),
                        "pursuit monster spawned on hazard {tile:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn maj01_and_maj02_unicorn_horn_race_deltas_and_luck_cap_13() {
    // MAJ-01 negative delta combo (Orc Wizard): UnicornHorn must not buff past racial baseline.
    let mut orc_save = start_build(42, Role::Wizard, Race::Orc, Alignment::Chaotic).to_save_data();
    let orc_base = Role::Wizard.base_attributes().with_race_deltas(Race::Orc);
    assert!(orc_base.int < Role::Wizard.base_attributes().int);
    {
        let c = orc_save.world.campaign.as_mut().unwrap();
        c.attributes.str_ -= 2;
        c.attributes.int -= 2;
    }
    let horn = add_inventory_item(&mut orc_save, ItemKind::UnicornHorn, BucStatus::Uncursed, 0);
    let mut orc_sess = GameSession::from_save_data(orc_save).unwrap();
    assert!(
        orc_sess
            .submit(C::Apply {
                item: horn,
                direction: None,
            })
            .accepted
    );
    let after_orc = orc_sess.world().campaign.unwrap().attributes;
    assert_eq!(after_orc, orc_base);

    // MAJ-01 positive delta combo (Dwarf Valkyrie): UnicornHorn must restore up to racial STR.
    let mut dw_save =
        start_build(42, Role::Valkyrie, Race::Dwarf, Alignment::Lawful).to_save_data();
    let dw_base = Role::Valkyrie
        .base_attributes()
        .with_race_deltas(Race::Dwarf);
    assert!(dw_base.str_ > Role::Valkyrie.base_attributes().str_);
    {
        let c = dw_save.world.campaign.as_mut().unwrap();
        c.attributes.str_ = dw_base.str_ - 2;
    }
    let horn2 = add_inventory_item(&mut dw_save, ItemKind::UnicornHorn, BucStatus::Uncursed, 0);
    let mut dw_sess = GameSession::from_save_data(dw_save).unwrap();
    assert!(
        dw_sess
            .submit(C::Apply {
                item: horn2,
                direction: None,
            })
            .accepted
    );
    let after_dw = dw_sess.world().campaign.unwrap().attributes;
    assert_eq!(after_dw.str_, dw_base.str_);

    // MAJ-02: Fountain luck gain preserves luck >= 4 up to MAX_LUCK(13).
    let mut luck_up = false;
    for seed in 1..40_u64 {
        let mut save = start_campaign(seed, Role::Knight).to_save_data();
        park_all_monsters_on(&mut save, LevelId::main(6));
        let main1 = LevelId::main(1);
        let f_pos = find_tile(&save, main1, |t| t == TileKind::Fountain);
        save.world
            .entities
            .set_actor_location(save.world.player_id, main1, f_pos);
        save.world.luck = 8;
        let mut sess = GameSession::from_save_data(save).unwrap();
        let out = sess.submit(C::Drink);
        if out.accepted && sess.world().luck == 9 {
            luck_up = true;
            const { assert!(MAX_LUCK >= 13) };
            break;
        }
    }
    assert!(luck_up, "expected fountain luck gain 8 -> 9");

    // MAJ-02: Magic trap (generated on Main 6) preserves luck >= 4 instead of clamping to 3.
    let mut magic_up = false;
    for seed in 1..40_u64 {
        let mut save = start_campaign(seed, Role::Knight).to_save_data();
        park_all_monsters_on(&mut save, LevelId::main(1));
        let main6 = LevelId::main(6);
        let (p_pos, _, dir) =
            find_floor_adjacent_to(&save, main6, |t| t == TileKind::HiddenTrap(TrapKind::Magic));
        save.world.current_level = main6;
        save.world
            .entities
            .set_actor_location(save.world.player_id, main6, p_pos);
        save.world.luck = 8;
        let mut sess = GameSession::from_save_data(save).unwrap();
        let out = sess.submit(C::Move(dir));
        if out.accepted && sess.world().luck == 9 {
            magic_up = true;
            break;
        }
    }
    assert!(magic_up, "expected magic trap luck gain 8 -> 9");
}

#[test]
fn maj03_unidentified_color_container_dip_llm_headless_no_leak() {
    let mut save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let (floor_pot_pos, fountain, _) =
        find_floor_adjacent_to(&save, main1, |t| t == TileKind::Fountain);
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, fountain);
    let sack = add_inventory_item(&mut save, ItemKind::Sack, BucStatus::Uncursed, 0);
    let pot_cure = add_inventory_item(&mut save, ItemKind::PotionCure, BucStatus::Uncursed, 0);
    let pot_heal = add_inventory_item(&mut save, ItemKind::PotionHealing, BucStatus::Uncursed, 0);
    let ring = add_inventory_item(&mut save, ItemKind::RingProtection, BucStatus::Uncursed, 0);
    save.world
        .entities
        .spawn_item(
            ItemKind::PotionExtraHealing,
            EntityLocation::OnMap {
                level: main1,
                pos: floor_pot_pos,
            },
        )
        .unwrap();
    save.world.identified_items.retain(|k| {
        !matches!(
            k,
            ItemKind::PotionCure
                | ItemKind::PotionHealing
                | ItemKind::PotionExtraHealing
                | ItemKind::RingProtection
        )
    });

    let mut sess = GameSession::from_save_data(save).unwrap();
    let obs = sess.observation();

    let expected_extra_app = appearance_for_kind(42, ItemKind::PotionExtraHealing).unwrap();
    let floor_obs = obs
        .visible_entities
        .iter()
        .find(|e| e.pos == floor_pot_pos && e.appearance.is_some())
        .unwrap();
    assert_eq!(
        floor_obs.color.as_deref(),
        Some(color_for_appearance(expected_extra_app))
    );
    let expected_cure_app = appearance_for_kind(42, ItemKind::PotionCure).unwrap();
    let inv_cure = obs.inventory.iter().find(|i| i.item == pot_cure).unwrap();
    assert_eq!(
        inv_cure.color.as_deref(),
        Some(color_for_appearance(expected_cure_app))
    );

    let llm_json = serde_json::to_value(LlmObservationView::from(&obs)).unwrap();
    let inv_entry = llm_json["inventory"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["item"].as_u64() == Some(u64::from(pot_cure.0)))
        .unwrap();
    assert!(inv_entry.get("kind").is_none());
    let floor_entry = llm_json["visible_entities"]
        .as_array()
        .unwrap()
        .iter()
        .find(|v| v["entity"].as_u64() == Some(u64::from(floor_obs.entity.0)))
        .unwrap();
    assert!(floor_entry.get("kind").is_none());

    {
        let mut s2 = sess.to_save_data();
        s2.world.poison_turns = 2;
        s2.world
            .entities
            .actor_stats_mut(s2.world.player_id)
            .unwrap()
            .hp = 3;
        let hs = GameSession::from_save_data(s2).unwrap();
        let cands = HeadlessPolicy::survival_v1().candidates(&hs);
        assert!(
            !cands
                .iter()
                .any(|c| matches!(c, C::Quaff { .. } | C::PutOn { .. })),
            "SurvivalV1 must not oracle-use unidentified campaign items: {cands:?}"
        );
    }

    let put = sess.submit(C::ContainerPut {
        container: sack,
        item: pot_cure,
    });
    assert!(put.accepted);
    assert!(put.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => {
            text.contains(expected_cure_app) && !text.contains("cure potion")
        }
        _ => false,
    }));
    let take = sess.submit(C::ContainerTake {
        container: sack,
        item: pot_cure,
    });
    assert!(take.accepted);
    assert!(take.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => {
            text.contains(expected_cure_app) && !text.contains("cure potion")
        }
        _ => false,
    }));

    let alch = sess.submit(C::Dip {
        item: pot_cure,
        target: DipTarget::Item(pot_heal),
    });
    assert!(alch.accepted);
    assert!(alch.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => {
            text.contains(expected_extra_app) && !text.contains("extra healing potion")
        }
        _ => false,
    }));

    let ring_app = appearance_for_kind(42, ItemKind::RingProtection).unwrap();
    let dip_ring = sess.submit(C::Dip {
        item: ring,
        target: DipTarget::Tile,
    });
    assert!(dip_ring.accepted);
    for e in &dip_ring.events {
        if let GameEvent::Message { text, .. } = e {
            assert!(
                !text.contains("ring of protection"),
                "leaked real ring name: {text}"
            );
            if text.contains("rust") {
                assert!(text.contains(ring_app));
            }
        }
    }
}

#[test]
fn maj04_character_creation_60x24_all_13_roles_keys_mouse_dynamic_stats() {
    for lines in [
        render_panels::character_creation_lines(),
        render_panels::character_creation_lines_with_selection(
            Some(Role::Archeologist),
            Race::Gnome,
            Alignment::Neutral,
        ),
    ] {
        assert!(lines.len() <= 22, "too tall for 60x24: {}", lines.len());
        for line in &lines {
            assert!(
                line.chars().count() <= 58,
                "line exceeds 58 cols ({}): {line}",
                line.chars().count()
            );
            assert!(!line.contains("Strength:") && !line.contains("Dexterity:"));
        }
        assert!(lines.iter().any(|l| l.contains("AC: 10")));
    }

    let clock = Arc::new(TestClock::default());
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        GameSession::try_new(42).unwrap(),
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    send_key(&mut app, &clock, KeyCode::Enter, 60, 24);
    assert_eq!(app.run_state(), RunState::CharacterCreation);

    send_key(&mut app, &clock, KeyCode::Char('a'), 60, 24);
    assert_eq!(app.run_state(), RunState::CharacterCreation);
    let mut term = Terminal::new(TestBackend::new(60, 24)).unwrap();
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    let screen: String = (0..24)
        .map(|y| {
            (0..60)
                .map(|x| term.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.contains("[A] Align: Neutral"));
    assert!(screen.contains("HP: 28/28"));

    let role_keys = [
        (KeyCode::Char('1'), "[1] Knight", Role::Knight),
        (KeyCode::Char('2'), "[2] Scout", Role::Scout),
        (KeyCode::Char('3'), "[3] Mage", Role::Mage),
        (KeyCode::Char('4'), "[4] Cleric", Role::Cleric),
        (KeyCode::Char('5'), "[5] Valkyrie", Role::Valkyrie),
        (KeyCode::Char('6'), "[6] Wizard", Role::Wizard),
        (KeyCode::Char('7'), "[7] Rogue", Role::Rogue),
        (KeyCode::Char('8'), "[8] Priest", Role::Priest),
        (KeyCode::Char('9'), "[9] Samurai", Role::Samurai),
        (KeyCode::Char('b'), "[b] Barbarian", Role::Barbarian),
        (KeyCode::Char('g'), "[g] Ranger", Role::Ranger),
        (KeyCode::Char('m'), "[m] Monk", Role::Monk),
        (KeyCode::Char('k'), "[k] Archeologist", Role::Archeologist),
    ];
    for &(key, label, expected_role) in &role_keys {
        let mut kb = TuiApp::new_with_llm_enabled_and_clock(
            GameSession::try_new(42).unwrap(),
            UiRuntimeConfig::default(),
            false,
            clock.clone(),
        );
        send_key(&mut kb, &clock, KeyCode::Enter, 60, 24);
        send_key(&mut kb, &clock, KeyCode::Tab, 60, 24);
        send_key(&mut kb, &clock, key, 60, 24);
        assert_eq!(
            kb.run_state(),
            RunState::Playing,
            "key failed for {expected_role:?}"
        );
        assert_eq!(kb.observation().campaign.unwrap().role, expected_role);

        let mut ms = TuiApp::new_with_llm_enabled_and_clock(
            GameSession::try_new(42).unwrap(),
            UiRuntimeConfig::default(),
            false,
            clock.clone(),
        );
        click_ui(&mut ms, &clock, "Press Enter to Start", 60, 24);
        click_ui(&mut ms, &clock, label, 60, 24);
        assert_eq!(
            ms.run_state(),
            RunState::Playing,
            "mouse failed for {label}"
        );
        assert_eq!(ms.observation().campaign.unwrap().role, expected_role);
    }

    let mut cycle = TuiApp::new_with_llm_enabled_and_clock(
        GameSession::try_new(42).unwrap(),
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    click_ui(&mut cycle, &clock, "Press Enter to Start", 60, 24);
    click_ui(&mut cycle, &clock, "[Tab] Next", 60, 24);
    click_ui(&mut cycle, &clock, "[Tab] Next", 60, 24);
    click_ui(&mut cycle, &clock, "[R] Race:", 60, 24);
    click_ui(&mut cycle, &clock, "Press Enter to confirm", 60, 24);
    assert_eq!(cycle.run_state(), RunState::Playing);
    let camp = cycle.observation().campaign.unwrap();
    assert_eq!(camp.role, Role::Scout);
    assert_eq!(camp.race, Some(Race::Elf));
}

#[test]
fn maj05_monster_resistances_con_regen_and_depth_weighted_spawn() {
    let reg = registry().unwrap();
    assert!(monster_resistances(MonsterKind::Dragon, reg).contains(Resistances::FIRE));
    assert!(monster_resistances(MonsterKind::FireAnt, reg).contains(Resistances::FIRE));
    assert!(monster_resistances(MonsterKind::GiantSpider, reg).contains(Resistances::POISON));

    let mut save = start_campaign(42, Role::Mage).to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut save, LevelId::main(6));
    let (p_pos, e_pos) = find_floor_with_east_floor(&save, main1);
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, p_pos);
    let fire_wand = add_inventory_item(&mut save, ItemKind::WandFire, BucStatus::Uncursed, 0);
    save.world.entities.set_item_charges(fire_wand, Some(4));
    let dragon = spawn_monster_at(&mut save, MonsterKind::Dragon, main1, e_pos);
    assert!(save
        .world
        .entities
        .actor_stats(dragon)
        .unwrap()
        .resistances
        .contains(Resistances::FIRE));
    let hp_before = save.world.entities.actor_stats(dragon).unwrap().hp;
    let mut sess = GameSession::from_save_data(save).unwrap();
    let out = sess.submit(C::Zap {
        item: fire_wand,
        direction: Direction::East,
    });
    assert!(out.accepted);
    assert_eq!(
        sess.world().entities.actor_stats(dragon).unwrap().hp,
        hp_before
    );
    assert!(out.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => text.contains("no effect"),
        _ => false,
    }));

    let mut high_con = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut high_con, LevelId::main(6));
    high_con.world.campaign.as_mut().unwrap().attributes.con = 18;
    high_con
        .world
        .entities
        .actor_stats_mut(high_con.world.player_id)
        .unwrap()
        .hp = 10;

    let mut low_con = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut low_con, LevelId::main(6));
    low_con.world.campaign.as_mut().unwrap().attributes.con = 12;
    low_con
        .world
        .entities
        .actor_stats_mut(low_con.world.player_id)
        .unwrap()
        .hp = 10;

    let mut hs = GameSession::from_save_data(high_con).unwrap();
    let mut ls = GameSession::from_save_data(low_con).unwrap();
    for _ in 0..13 {
        assert!(hs.submit(C::Wait).accepted);
        assert!(ls.submit(C::Wait).accepted);
    }
    assert_eq!(hs.turn(), 13);
    assert_eq!(hs.observation().player.hp, 11);
    assert_eq!(ls.observation().player.hp, 10);

    assert!(!weighted_monster_kinds_for_depth(3, reg).is_empty());
    let base_camp = start_campaign(42, Role::Knight);
    let main2_monsters = base_camp
        .world()
        .entities
        .entities()
        .iter()
        .filter(|e| {
            matches!(e.actor_kind(), Some(ActorKind::Monster(_)))
                && base_camp
                    .world()
                    .entities
                    .actor_location(e.id)
                    .is_some_and(|(lvl, _)| lvl == LevelId::main(2))
        })
        .count();
    assert!(
        main2_monsters >= 2,
        "Main 2 should have fixed Jackal + Room 5 depth-weighted spawn, got {main2_monsters}"
    );
}

#[test]
fn maj06_save_v3_tamper_hole_death_wait_energy_and_hazard_turns() {
    let base = start_campaign(42, Role::Knight).to_save_data();

    let mut t1 = base.clone();
    t1.world.campaign.as_mut().unwrap().god_anger = MAX_GOD_ANGER + 1;
    assert!(GameSession::from_save_data(t1).is_err());

    let mut t1b = base.clone();
    t1b.world.campaign.as_mut().unwrap().god_anger = -1;
    assert!(GameSession::from_save_data(t1b).is_err());

    let mut t2 = base.clone();
    t2.world.campaign.as_mut().unwrap().prayer_timeout = MAX_PRAYER_TIMEOUT + 1;
    assert!(GameSession::from_save_data(t2).is_err());

    let mut t3 = base.clone();
    t3.world.campaign.as_mut().unwrap().artifact_invoke_cooldown =
        ARTIFACT_INVOKE_COOLDOWN_TURNS + 1;
    assert!(GameSession::from_save_data(t3).is_err());

    let mut t4 = base.clone();
    t4.world.learned_spells = vec![
        LearnedSpell {
            spell: SpellKind::Heal,
            learned_at_turn: 0,
        },
        LearnedSpell {
            spell: SpellKind::Heal,
            learned_at_turn: 1,
        },
    ];
    assert!(GameSession::from_save_data(t4).is_err());

    let mut t5 = base.clone();
    t5.world.ascension_altars = vec![
        (LevelId::main(1), Pos { x: 1, y: 1 }),
        (LevelId::main(1), Pos { x: 1, y: 1 }),
    ];
    assert!(GameSession::from_save_data(t5).is_err());

    let t6 = base.clone();
    let first_inv = t6.world.inventory.entries[0].item;
    let first_monster = t6
        .world
        .entities
        .entities()
        .iter()
        .find(|e| matches!(e.actor_kind(), Some(ActorKind::Monster(_))))
        .unwrap()
        .id;
    let mut t6v = serde_json::to_value(&t6).unwrap();
    for ent in t6v["world"]["entities"]["entities"].as_array_mut().unwrap() {
        if ent["id"].as_u64() == Some(u64::from(first_monster.0)) {
            ent["payload"]["Actor"]["monster_inventory"] = serde_json::json!([first_inv.0]);
        }
    }
    let bad_t6: aihack_runtime::save::SaveDataV1 = serde_json::from_value(t6v).unwrap();
    assert!(GameSession::from_save_data(bad_t6).is_err());

    let mut t7 = base.clone();
    let dagger = add_inventory_item(&mut t7, ItemKind::Dagger, BucStatus::Uncursed, 0);
    assert!(t7
        .world
        .entities
        .set_item_corpse_monster(dagger, Some(MonsterKind::Jackal)));
    assert!(GameSession::from_save_data(t7).is_err());

    let mut t8 = base;
    let art = add_inventory_item(&mut t8, ItemKind::LongSword, BucStatus::Blessed, 1);
    t8.world
        .entities
        .set_item_artifact_id(art, Some("excalibur".to_string()));
    t8.world.generated_artifacts.insert("excalibur".to_string());
    t8.world.entities.set_item_erosion(art, 1);
    assert!(GameSession::from_save_data(t8).is_err());

    // 2. Fatal TrapKind::Hole (Main 4) stays on Main 4 without LevelChanged event.
    let mut hole_save = start_campaign(42, Role::Knight).to_save_data();
    let main4 = LevelId::main(4);
    park_all_monsters_on(&mut hole_save, LevelId::main(6));
    let (p_pos, _, hole_dir) = find_floor_adjacent_to(&hole_save, main4, |t| {
        t == TileKind::HiddenTrap(TrapKind::Hole)
    });
    hole_save.world.current_level = main4;
    hole_save
        .world
        .entities
        .set_actor_location(hole_save.world.player_id, main4, p_pos);
    hole_save
        .world
        .entities
        .actor_stats_mut(hole_save.world.player_id)
        .unwrap()
        .hp = 1;
    let mut hole_sess = GameSession::from_save_data(hole_save).unwrap();
    let hole_out = hole_sess.submit(C::Move(hole_dir));
    assert!(hole_out.accepted);
    assert_eq!(hole_sess.world().current_level(), main4);
    assert!(!hole_out
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::LevelChanged { .. })));
    assert!(matches!(hole_sess.run_state(), RunState::GameOver { .. }));

    // 3. Fast monster (Bat, speed 24) waiting on another floor/out of chase range clamps energy.
    let mut bat_save = start_campaign(42, Role::Knight).to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut bat_save, LevelId::main(6));
    let far_bat_pos = find_tile(&bat_save, main1, |t| t == TileKind::Floor);
    let bat = spawn_monster_at(&mut bat_save, MonsterKind::Bat, main1, far_bat_pos);
    let mut bat_sess = GameSession::from_save_data(bat_save).unwrap();
    for _ in 0..10 {
        assert!(bat_sess.submit(C::Wait).accepted);
    }
    let bat_energy = bat_sess.world().entities.actor_stats(bat).unwrap().energy;
    assert!(
        bat_energy <= 12,
        "waiting Bat energy must be bounded, got {bat_energy}"
    );

    // 4. Waiting on Lava (Main 5) & Levitation expiry over Lava apply hazard damage.
    let mut lava_save = start_campaign(42, Role::Knight).to_save_data();
    let main5 = LevelId::main(5);
    park_all_monsters_on(&mut lava_save, LevelId::main(6));
    let lava_pos = find_tile(&lava_save, main5, |t| t == TileKind::Lava);
    lava_save.world.current_level = main5;
    lava_save
        .world
        .entities
        .set_actor_location(lava_save.world.player_id, main5, lava_pos);
    lava_save.world.status_timers.levitation = 1;
    let mut lava_sess = GameSession::from_save_data(lava_save).unwrap();
    let hp0 = lava_sess.observation().player.hp;
    let wait_out = lava_sess.submit(C::Wait);
    assert!(wait_out.accepted);
    assert_eq!(lava_sess.world().status_timers.levitation, 0);
    assert!(
        lava_sess.observation().player.hp < hp0
            || matches!(lava_sess.run_state(), RunState::GameOver { .. }),
        "Levitation expiry onto Lava must deal lava damage immediately"
    );
    assert!(wait_out.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => text.contains("molten lava"),
        _ => false,
    }));
}

#[test]
fn min01_through_min07_edge_cases_and_content_consistency() {
    let reg = registry().unwrap();

    let amulet_def = reg.item("item.quest.ascension").unwrap();
    assert!(amulet_def.artifact_id.is_none());

    assert_eq!(MonsterKind::FloatingEye.difficulty(), 5);
    assert_eq!(reg.monster("monster.floating_eye").unwrap().difficulty, 5);

    let camp = start_campaign(42, Role::Knight);
    for (lvl, kind) in [
        (
            LevelId {
                branch: BranchId::Mines,
                depth: 1,
            },
            ItemKind::Dagger,
        ),
        (
            LevelId {
                branch: BranchId::Mines,
                depth: 2,
            },
            ItemKind::RingProtection,
        ),
    ] {
        let map = camp.world().map(lvl);
        let found_pos = camp
            .world()
            .entities
            .entities()
            .iter()
            .find_map(|e| match e.item() {
                Some((k, _, EntityLocation::OnMap { level, pos }, _, _))
                    if k == kind && level == lvl =>
                {
                    Some(pos)
                }
                _ => None,
            })
            .expect("expected mines item on floor");
        assert_eq!(map.tile(found_pos).unwrap(), TileKind::Floor);
    }

    let mut put_save = start_campaign(42, Role::Knight).to_save_data();
    let amulet = add_inventory_item(&mut put_save, ItemKind::AmuletEsp, BucStatus::Uncursed, 0);
    put_save.run_state = RunState::AwaitingInventorySelection {
        action: InventoryAction::PutOn,
    };
    let mut put_sess = GameSession::from_save_data(put_save).unwrap();
    assert!(put_sess.submit(C::PutOn { item: amulet }).accepted);

    assert_eq!(
        item_color(ItemKind::AmuletAscension, reg),
        Some("bright_yellow")
    );
    assert_eq!(item_color(ItemKind::Katana, reg), Some("bright_cyan"));
    assert_eq!(
        item_color(ItemKind::PotionHallucination, reg),
        Some("bright_magenta")
    );
    assert_eq!(item_color(ItemKind::PotionConfusion, reg), Some("orange"));
    assert_eq!(item_color(ItemKind::PotionBlindness, reg), Some("black"));
    let mut col_save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut col_save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let (p_pos, kat_pos) = find_floor_with_east_floor(&col_save, main1);
    col_save
        .world
        .entities
        .set_actor_location(col_save.world.player_id, main1, p_pos);
    col_save
        .world
        .entities
        .spawn_item(
            ItemKind::Katana,
            EntityLocation::OnMap {
                level: main1,
                pos: kat_pos,
            },
        )
        .unwrap();
    let col_sess = GameSession::from_save_data(col_save).unwrap();
    let mut col_app = TuiApp::new(col_sess, UiRuntimeConfig::default());
    let layout = compute_layout(80, 24);
    let vp = col_app.viewport_for_observation(layout);
    let mut term = Terminal::new(TestBackend::new(80, 24)).unwrap();
    term.draw(|frame| render_frame(frame, &mut col_app))
        .unwrap();
    let (kx, ky) = vp.world_to_terminal(kat_pos, layout.map).unwrap();
    assert_eq!(term.backend().buffer()[(kx, ky)].fg, Color::LightCyan);

    // MIN-03: Apply ('.' chest/lock alongside adjacent generated Locked door on Mines 2) and item_menu 'V' Invoke.
    let mut menu_save = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut menu_save, LevelId::main(6));
    let locked_lvl = LevelId {
        branch: BranchId::Mines,
        depth: 2,
    };
    let (m_pos, _, _) = find_floor_adjacent_to(&menu_save, locked_lvl, |t| {
        t == TileKind::Door(DoorState::Locked)
    });
    menu_save.world.current_level = locked_lvl;
    menu_save
        .world
        .entities
        .set_actor_location(menu_save.world.player_id, locked_lvl, m_pos);
    menu_save
        .world
        .entities
        .spawn_item(
            ItemKind::Chest,
            EntityLocation::OnMap {
                level: locked_lvl,
                pos: m_pos,
            },
        )
        .unwrap();
    let key_id = add_inventory_item(
        &mut menu_save,
        ItemKind::SkeletonKey,
        BucStatus::Uncursed,
        0,
    );
    let sun_id = add_inventory_item(&mut menu_save, ItemKind::BroadSword, BucStatus::Blessed, 1);
    menu_save
        .world
        .entities
        .set_item_artifact_id(sun_id, Some("sunsword".to_string()));
    menu_save
        .world
        .generated_artifacts
        .insert("sunsword".to_string());
    let key_letter = menu_save.world.inventory.letter_for(key_id).unwrap().0;
    let sun_letter = menu_save.world.inventory.letter_for(sun_id).unwrap().0;
    let clock = Arc::new(TestClock::default());
    let mut menu_app = TuiApp::new_with_llm_enabled_and_clock(
        GameSession::from_save_data(menu_save).unwrap(),
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    send_key(&mut menu_app, &clock, KeyCode::Char('a'), 100, 30);
    send_key(&mut menu_app, &clock, KeyCode::Char(key_letter), 100, 30);
    let mut menu_term = Terminal::new(TestBackend::new(100, 30)).unwrap();
    menu_term
        .draw(|frame| render_frame(frame, &mut menu_app))
        .unwrap();
    let apply_screen: String = (0..30)
        .map(|y| {
            (0..100)
                .map(|x| menu_term.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(apply_screen.contains("Here (chest / lock)"));
    send_key(&mut menu_app, &clock, KeyCode::Esc, 100, 30);
    send_key(&mut menu_app, &clock, KeyCode::Char('i'), 100, 30);
    send_key(&mut menu_app, &clock, KeyCode::Char(sun_letter), 100, 30);
    menu_term
        .draw(|frame| render_frame(frame, &mut menu_app))
        .unwrap();
    let inv_screen: String = (0..30)
        .map(|y| {
            (0..100)
                .map(|x| menu_term.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(inv_screen.contains("[V] Invoke"));

    // MIN-04: Stormbringer LOS blocking on existing wall-separated floors.
    let mut sb_save = start_build(42, Role::Rogue, Race::Human, Alignment::Chaotic).to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut sb_save, LevelId::main(6));
    let (p_pos, hidden_pos) = find_los_blocked_floor_pair(&sb_save, main1);
    sb_save
        .world
        .entities
        .set_actor_location(sb_save.world.player_id, main1, p_pos);
    let blocked_mob = spawn_monster_at(&mut sb_save, MonsterKind::Orc, main1, hidden_pos);
    let sb = add_inventory_item(&mut sb_save, ItemKind::LongSword, BucStatus::Uncursed, 1);
    sb_save
        .world
        .entities
        .set_item_artifact_id(sb, Some("stormbringer".to_string()));
    sb_save
        .world
        .generated_artifacts
        .insert("stormbringer".to_string());
    let mut sb_sess = GameSession::from_save_data(sb_save).unwrap();
    let mob_hp0 = sb_sess
        .world()
        .entities
        .actor_stats(blocked_mob)
        .unwrap()
        .hp;
    assert!(sb_sess.submit(C::Invoke { item: sb }).accepted);
    assert_eq!(
        sb_sess
            .world()
            .entities
            .actor_stats(blocked_mob)
            .unwrap()
            .hp,
        mob_hp0,
        "Stormbringer #invoke must respect line of sight through walls"
    );

    let mut blast_save = start_campaign(42, Role::Knight).to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut blast_save, LevelId::main(6));
    let b_pos = blast_save
        .world
        .entities
        .actor_location(blast_save.world.player_id)
        .unwrap()
        .1;
    let b_chest = blast_save
        .world
        .entities
        .spawn_item(
            ItemKind::Chest,
            EntityLocation::OnMap {
                level: main1,
                pos: b_pos,
            },
        )
        .unwrap();
    let hostile_sb = blast_save
        .world
        .entities
        .spawn_item(
            ItemKind::LongSword,
            EntityLocation::InContainer { container: b_chest },
        )
        .unwrap();
    blast_save
        .world
        .entities
        .set_item_artifact_id(hostile_sb, Some("stormbringer".to_string()));
    blast_save
        .world
        .generated_artifacts
        .insert("stormbringer".to_string());
    blast_save
        .world
        .entities
        .push_contained_item(b_chest, hostile_sb);
    let mut blast_sess = GameSession::from_save_data(blast_save).unwrap();
    let b_hp0 = blast_sess.observation().player.hp;
    let take_art = blast_sess.submit(C::ContainerTake {
        container: b_chest,
        item: hostile_sb,
    });
    assert!(take_art.accepted);
    assert!(blast_sess.observation().player.hp < b_hp0);

    // MIN-05: Throw weapon enchantment / blessing reflects into damage.
    let mut th_save = start_campaign(42, Role::Ranger).to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut th_save, LevelId::main(6));
    let (t_pos, m_pos) = find_floor_with_east_floor(&th_save, main1);
    th_save
        .world
        .entities
        .set_actor_location(th_save.world.player_id, main1, t_pos);
    th_save.world.luck = 10;
    let good_dagger = add_inventory_item(&mut th_save, ItemKind::Dagger, BucStatus::Blessed, 5);
    let ogre = spawn_monster_at(&mut th_save, MonsterKind::Ogre, main1, m_pos);
    let ogre_hp0 = th_save.world.entities.actor_stats(ogre).unwrap().hp;
    let mut th_sess = GameSession::from_save_data(th_save).unwrap();
    let out = th_sess.submit(C::Throw {
        item: good_dagger,
        direction: Direction::East,
    });
    assert!(out.accepted);
    assert!(th_sess.world().entities.actor_stats(ogre).unwrap().hp <= ogre_hp0 - 5);

    // MIN-06: Shop buy log (on generated Main 2 shop) & render_panels non-Debug events.
    let mut shop_save = start_campaign(42, Role::Knight).to_save_data();
    let main2 = LevelId::main(2);
    park_all_monsters_on(&mut shop_save, LevelId::main(6));
    let shop_tile = find_tile(&shop_save, main2, |t| t == TileKind::Shop);
    shop_save.world.current_level = main2;
    shop_save
        .world
        .entities
        .set_actor_location(shop_save.world.player_id, main2, shop_tile);
    shop_save.world.gold = 300;
    shop_save
        .world
        .identified_items
        .retain(|k| *k != ItemKind::PotionHealing);
    let mut shop_sess = GameSession::from_save_data(shop_save).unwrap();
    let expected_pot_app = appearance_for_kind(42, ItemKind::PotionHealing).unwrap();
    let buy = shop_sess.submit(C::Buy {
        kind: ItemKind::PotionHealing,
    });
    assert!(buy.accepted);
    assert!(buy.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => {
            text.contains(expected_pot_app) && !text.contains("PotionHealing")
        }
        _ => false,
    }));
    let mut msg_obs = shop_sess.observation();
    msg_obs.last_events.push(GameEvent::Message {
        priority: MessagePriority::Warning,
        text: "Smoke rises from the brazier.".to_string(),
    });
    let panel_logs = render_panels::log_lines(&msg_obs, &[]);
    assert!(panel_logs
        .iter()
        .any(|l| l.contains("Smoke rises from the brazier.") && !l.contains("Message {")));
}
