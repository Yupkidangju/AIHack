use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use aihack_content::{monster_has_ai_flag, registry};
use aihack_core::{
    action::{CommandIntent as C, DipTarget, EngravingMethod, WishChoice},
    campaign::{Alignment, LearnedSpell, Race, Role, SkillCategory, SpellKind},
    domain::{
        combat::DeathCause,
        entity::{ActorKind, EntityLocation, Faction, DEFAULT_PET_HUNGER},
        item::{BucStatus, EquipmentSlot, ItemKind},
        monster::MonsterKind,
        tile::{DoorState, TileKind},
    },
    event::GameEvent,
    ids::{BranchId, EntityId, LevelId},
    position::{Direction, Pos},
    run_state::RunState,
};
use aihack_runtime::{
    appearance::appearance_for_kind, domain::monster::try_monster_template_from_registry,
    save::SaveDataV1, GameSession,
};
use aihack_tui::tui::{
    render_frame, runtime_event_to_candidate, TuiApp, UiClock, UiCommandCandidate, UiRuntimeConfig,
};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{backend::TestBackend, Terminal};

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

fn park_all_monsters_on(save: &mut SaveDataV1, dest: LevelId) {
    let map = save.world.levels.map(dest).unwrap().clone();
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
            .set_actor_location(id, dest, free[idx % free.len()]);
    }
}

fn add_item(
    save: &mut SaveDataV1,
    kind: ItemKind,
    buc: BucStatus,
    enchant: i8,
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
    save.world.entities.set_item_enchantment(id, enchant);
    save.world.entities.set_item_buc_known(id, known);
    save.world.entities.set_item_enchant_known(id, known);
    if known && !save.world.identified_items.contains(&kind) {
        save.world.identified_items.push(kind);
    }
    id
}

fn clear_inventory(save: &mut SaveDataV1) {
    let ids: Vec<EntityId> = save
        .world
        .inventory
        .entries
        .iter()
        .map(|e| e.item)
        .collect();
    for id in ids {
        save.world.inventory.remove(id);
        save.world
            .entities
            .set_item_location(id, EntityLocation::Consumed);
    }
}

fn quaff_gain_level(sess: &mut GameSession, count: u16) {
    let mut save = sess.to_save_data();
    let owner = save.world.player_id;
    let pot = save
        .world
        .entities
        .spawn_item(
            ItemKind::PotionGainLevel,
            EntityLocation::Inventory { owner },
        )
        .unwrap();
    save.world.entities.set_item_quantity(pot, count);
    let letter = save
        .world
        .inventory
        .add_existing_with_next_letter(pot)
        .unwrap();
    save.world.entities.set_item_letter(pot, letter);
    *sess = GameSession::from_save_data(save).unwrap();
    for _ in 0..count {
        let out = sess.submit(C::Quaff { item: pot });
        assert!(out.accepted, "{:?}", out.events);
    }
}

fn find_tile(save: &SaveDataV1, level: LevelId, pred: impl Fn(TileKind) -> bool) -> Pos {
    let map = save.world.levels.map(level).unwrap();
    for y in 0..map.height {
        for x in 0..map.width {
            let p = Pos { x, y };
            if map.tile(p).ok().is_some_and(&pred) {
                return p;
            }
        }
    }
    panic!("tile not found on {level:?}");
}

fn find_floor_adjacent_to_tile(
    save: &SaveDataV1,
    level: LevelId,
    pred: impl Fn(Pos, TileKind) -> bool,
) -> (Pos, Direction, Pos) {
    let map = save.world.levels.map(level).unwrap();
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let p = Pos { x, y };
            if let Ok(tile) = map.tile(p) {
                if pred(p, tile) {
                    for dir in Direction::ALL {
                        let d = dir.delta();
                        let adj = Pos {
                            x: p.x - d.dx,
                            y: p.y - d.dy,
                        };
                        if map.contains(adj) && map.tile(adj).ok() == Some(TileKind::Floor) {
                            return (adj, dir, p);
                        }
                    }
                }
            }
        }
    }
    panic!("adjacent floor/tile pair not found on {level:?}");
}

fn spawn_monster_at(
    save: &mut SaveDataV1,
    kind: MonsterKind,
    level: LevelId,
    pos: Pos,
    faction: Faction,
) -> EntityId {
    let reg = registry().unwrap();
    let tmpl = try_monster_template_from_registry(kind, reg).unwrap();
    let id = save
        .world
        .entities
        .spawn_monster_with_faction(kind, tmpl, faction, pos)
        .unwrap();
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

fn render_text(app: &mut TuiApp, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| render_frame(frame, app)).unwrap();
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..height {
        for x in 0..width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn a36_dbg_f001_locked_door_save_roundtrip() {
    let mut save = start_campaign(78, Role::Wizard).to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let (adj, door_dir, door_pos) = find_floor_adjacent_to_tile(&save, main1, |_, tile| {
        tile == TileKind::Door(DoorState::Closed)
    });
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, adj);
    save.world.current_level = main1;
    let key_id = add_item(
        &mut save,
        ItemKind::SkeletonKey,
        BucStatus::Uncursed,
        0,
        true,
    );
    let wand_lock = add_item(
        &mut save,
        ItemKind::WandLocking,
        BucStatus::Uncursed,
        0,
        true,
    );
    save.world.learned_spells.push(LearnedSpell {
        spell: SpellKind::WizardLock,
        learned_at_turn: 0,
    });

    let mut sess = GameSession::from_save_data(save).unwrap();

    // 1. SkeletonKey locks Closed -> Locked and survives JSON SaveV3 roundtrip.
    let key_lock = sess.submit(C::Apply {
        item: key_id,
        direction: Some(door_dir),
    });
    assert!(key_lock.accepted, "{:?}", key_lock.events);
    assert_eq!(
        sess.world().map(main1).tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Locked))
    );
    let json1 = serde_json::to_string(&sess.to_save_data()).unwrap();
    let mut sess =
        GameSession::from_save_data(serde_json::from_str::<SaveDataV1>(&json1).unwrap()).unwrap();
    assert_eq!(
        sess.world().map(main1).tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Locked))
    );

    // 2. Unlock, open, then lock Open -> Locked via WizardLock and verify JSON roundtrip.
    assert!(
        sess.submit(C::Apply {
            item: key_id,
            direction: Some(door_dir),
        })
        .accepted
    );
    assert!(sess.submit(C::Open(door_dir)).accepted);
    assert_eq!(
        sess.world().map(main1).tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Open))
    );
    let wiz_lock = sess.submit(C::Cast {
        spell: SpellKind::WizardLock,
        direction: Some(door_dir),
    });
    assert!(wiz_lock.accepted, "{:?}", wiz_lock.events);
    let json2 = serde_json::to_string(&sess.to_save_data()).unwrap();
    let mut sess =
        GameSession::from_save_data(serde_json::from_str::<SaveDataV1>(&json2).unwrap()).unwrap();
    assert_eq!(
        sess.world().map(main1).tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Locked))
    );

    // 3. Unlock and lock via WandLocking, then verify JSON roundtrip.
    assert!(
        sess.submit(C::Apply {
            item: key_id,
            direction: Some(door_dir),
        })
        .accepted
    );
    let zap_lock = sess.submit(C::Zap {
        item: wand_lock,
        direction: door_dir,
    });
    assert!(zap_lock.accepted, "{:?}", zap_lock.events);
    let json3 = serde_json::to_string(&sess.to_save_data()).unwrap();
    let reloaded3 =
        GameSession::from_save_data(serde_json::from_str::<SaveDataV1>(&json3).unwrap()).unwrap();
    assert_eq!(
        reloaded3.world().map(main1).tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Locked))
    );
}

#[test]
fn a36_dbg_f002_sokoban_second_boulder_over_plugged_pit_roundtrip() {
    let mut save = start_campaign(42, Role::Archeologist).to_save_data();
    let sok1 = LevelId::new(BranchId::Sokoban, 1);
    park_all_monsters_on(&mut save, LevelId::main(6));
    save.world.current_level = sok1;
    save.world
        .entities
        .set_actor_location(save.world.player_id, sok1, Pos { x: 36, y: 9 });
    let mut sess = GameSession::from_save_data(save).unwrap();

    // Push boulder 1 at (37, 9) East into pit at (42, 9) -> pit turns into Floor.
    for _ in 0..5 {
        assert!(sess.submit(C::Move(Direction::East)).accepted);
    }
    assert_eq!(
        sess.world().current_map().tile(Pos { x: 42, y: 9 }).ok(),
        Some(TileKind::Floor)
    );

    // Move behind boulder 2 at (37, 11), push it North twice to (37, 9), then East onto plugged cell (42, 9).
    let mut mid = sess.to_save_data();
    mid.world
        .entities
        .set_actor_location(mid.world.player_id, sok1, Pos { x: 37, y: 12 });
    sess = GameSession::from_save_data(mid).unwrap();
    for _ in 0..2 {
        assert!(sess.submit(C::Move(Direction::North)).accepted);
    }
    let mut lane = sess.to_save_data();
    lane.world
        .entities
        .set_actor_location(lane.world.player_id, sok1, Pos { x: 36, y: 9 });
    sess = GameSession::from_save_data(lane).unwrap();
    for _ in 0..5 {
        assert!(sess.submit(C::Move(Direction::East)).accepted);
    }
    assert_eq!(
        sess.world().current_map().tile(Pos { x: 42, y: 9 }).ok(),
        Some(TileKind::Boulder)
    );

    // SaveV3 JSON roundtrip must accept boulder sitting on plugged-pit coordinate.
    let json = serde_json::to_string(&sess.to_save_data()).unwrap();
    let reloaded =
        GameSession::from_save_data(serde_json::from_str::<SaveDataV1>(&json).unwrap()).unwrap();
    assert_eq!(
        reloaded
            .world()
            .current_map()
            .tile(Pos { x: 42, y: 9 })
            .ok(),
        Some(TileKind::Boulder)
    );

    // Negative check: forging an extra boulder on another floor tile still fails.
    let mut forged = reloaded.to_save_data();
    forged
        .world
        .levels
        .get_mut(sok1)
        .unwrap()
        .map
        .set_tile(Pos { x: 43, y: 9 }, TileKind::Boulder)
        .unwrap();
    assert!(GameSession::from_save_data(forged).is_err());
}

#[test]
fn a36_imp_f001_sec_f003_astral_altars_immutable() {
    let mut save = start_build(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    let planes5 = LevelId::new(BranchId::Planes, 5);
    let mines2 = LevelId::new(BranchId::Mines, 2);
    park_all_monsters_on(&mut save, LevelId::main(6));

    let cross_altar = save
        .world
        .ascension_altars
        .iter()
        .copied()
        .find(|&(lvl, p)| {
            lvl == planes5
                && save
                    .world
                    .levels
                    .get(lvl)
                    .and_then(|l| l.altar_alignment(p))
                    != Some(Alignment::Lawful)
        })
        .unwrap()
        .1;
    let cross_align_before = save
        .world
        .levels
        .get(planes5)
        .unwrap()
        .altar_alignment(cross_altar);

    let corpse = add_item(
        &mut save,
        ItemKind::CorpseJackal,
        BucStatus::Uncursed,
        0,
        true,
    );
    save.world.current_level = planes5;
    save.world
        .entities
        .set_actor_location(save.world.player_id, planes5, cross_altar);

    let mut sess = GameSession::from_save_data(save).unwrap();
    let obs = sess.observation();
    assert!(
        !obs.legal_actions.contains(&C::Offer { item: corpse }),
        "Planes 5 cross-aligned high altar must not expose non-amulet Offer"
    );
    let rejected = sess.submit(C::Offer { item: corpse });
    assert!(!rejected.accepted);
    assert!(sess.world().inventory.contains(corpse));
    assert_eq!(
        sess.world()
            .levels
            .get(planes5)
            .unwrap()
            .altar_alignment(cross_altar),
        cross_align_before
    );

    // Non-Astral cross-aligned altar (Mines 2) can still be converted by offering a corpse.
    let mut m_save = sess.to_save_data();
    let mines2_altar = find_tile(&m_save, mines2, |t| t == TileKind::Altar);
    assert_ne!(
        m_save
            .world
            .levels
            .get(mines2)
            .unwrap()
            .altar_alignment(mines2_altar),
        Some(Alignment::Lawful)
    );
    m_save.world.current_level = mines2;
    m_save
        .world
        .entities
        .set_actor_location(m_save.world.player_id, mines2, mines2_altar);
    let mut m_sess = GameSession::from_save_data(m_save).unwrap();
    let converted = m_sess.submit(C::Offer { item: corpse });
    assert!(converted.accepted, "{:?}", converted.events);
    assert_eq!(
        m_sess
            .world()
            .levels
            .get(mines2)
            .unwrap()
            .altar_alignment(mines2_altar),
        Some(Alignment::Lawful)
    );

    // SEC-F003 SaveV3 validation rejects tampered ascension_altars coords or altered/duplicated Planes 5 alignments.
    let mut bad_coord = m_sess.to_save_data();
    bad_coord.world.ascension_altars[0].1.x += 1;
    assert!(GameSession::from_save_data(bad_coord).is_err());

    let mut dup_align = m_sess.to_save_data();
    dup_align
        .world
        .levels
        .get_mut(planes5)
        .unwrap()
        .set_altar_alignment(cross_altar, Alignment::Lawful);
    assert!(GameSession::from_save_data(dup_align).is_err());

    let mut cleared_align = m_sess.to_save_data();
    cleared_align
        .world
        .levels
        .get_mut(planes5)
        .unwrap()
        .altar_alignments
        .clear();
    assert!(GameSession::from_save_data(cleared_align).is_err());
}

#[test]
fn a36_imp_f002_imp_f006_spell_failure_study_and_cast_fail() {
    // 1. Level progression lowers spell failure rate, mithril exempts penalty, iron increases penalty, all within 0..=100.
    let mut save = start_campaign(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let turn0 = save.turn;
    save.world.learned_spells.push(LearnedSpell {
        spell: SpellKind::ConeOfCold,
        learned_at_turn: turn0,
    });
    let chain = add_item(
        &mut save,
        ItemKind::ArmorChainMail,
        BucStatus::Uncursed,
        0,
        true,
    );
    let mithril = add_item(
        &mut save,
        ItemKind::ArmorSilverDragonScale,
        BucStatus::Uncursed,
        0,
        true,
    );
    let mut sess = GameSession::from_save_data(save).unwrap();
    let rate_for = |s: &GameSession, spell: SpellKind| {
        s.observation()
            .campaign
            .unwrap()
            .spell_failure_rates
            .into_iter()
            .find(|&(k, _)| k == spell)
            .unwrap()
            .1
    };
    let lv1_cone = rate_for(&sess, SpellKind::ConeOfCold);
    assert!(sess.submit(C::Wear { item: chain }).accepted);
    let lv1_iron = rate_for(&sess, SpellKind::ConeOfCold);
    assert!(lv1_iron > lv1_cone && lv1_iron <= 100);
    assert!(
        sess.submit(C::TakeOff {
            slot: EquipmentSlot::Body
        })
        .accepted
    );
    assert!(sess.submit(C::Wear { item: mithril }).accepted);
    assert_eq!(rate_for(&sess, SpellKind::ConeOfCold), lv1_cone);
    assert!(
        sess.submit(C::TakeOff {
            slot: EquipmentSlot::Body
        })
        .accepted
    );

    quaff_gain_level(&mut sess, 9);
    let lv10_cone = rate_for(&sess, SpellKind::ConeOfCold);
    assert!(lv10_cone < lv1_cone, "lv10={lv10_cone}% vs lv1={lv1_cone}%");
    for (_, pct) in sess.observation().campaign.unwrap().spell_failure_rates {
        assert!(pct <= 100);
    }

    // 2. Cursed spellbook inflicts blind + confused + 1d6 damage and DeathCause::Explosion when lethal.
    let mut cursed_save = start_campaign(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut cursed_save, LevelId::main(6));
    let cursed_book = add_item(
        &mut cursed_save,
        ItemKind::SpellbookMagicMissile,
        BucStatus::Cursed,
        0,
        true,
    );
    let mut cursed_sess = GameSession::from_save_data(cursed_save).unwrap();
    let hp_before = cursed_sess.observation().player.hp;
    let cursed_read = cursed_sess.submit(C::Read { item: cursed_book });
    assert!(cursed_read.accepted);
    assert!(cursed_sess.world().status_timers.blind >= 2);
    assert!(cursed_sess.world().status_timers.confused >= 3);
    assert!(cursed_sess.observation().player.hp < hp_before);

    let mut lethal_save = start_campaign(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut lethal_save, LevelId::main(6));
    let lethal_book = add_item(
        &mut lethal_save,
        ItemKind::SpellbookMagicMissile,
        BucStatus::Cursed,
        0,
        true,
    );
    lethal_save
        .world
        .entities
        .actor_stats_mut(lethal_save.world.player_id)
        .unwrap()
        .hp = 1;
    let mut lethal_sess = GameSession::from_save_data(lethal_save).unwrap();
    assert!(lethal_sess.submit(C::Read { item: lethal_book }).accepted);
    assert!(matches!(
        lethal_sess.run_state(),
        RunState::GameOver {
            cause: DeathCause::Explosion,
            ..
        }
    ));

    // 3. Uncursed high-level spellbook uses RNG study check (can fail with confusion and succeed across seeds).
    let mut saw_fail = false;
    let mut saw_success = false;
    for seed in 1..=30 {
        let mut s_save = start_campaign(seed, Role::Mage).to_save_data();
        park_all_monsters_on(&mut s_save, LevelId::main(6));
        s_save.world.campaign.as_mut().unwrap().attributes.int = 10;
        let book = add_item(
            &mut s_save,
            ItemKind::SpellbookFireball,
            BucStatus::Uncursed,
            0,
            true,
        );
        let mut s_sess = GameSession::from_save_data(s_save).unwrap();
        let out = s_sess.submit(C::Read { item: book });
        assert!(out.accepted);
        let learned = s_sess
            .world()
            .learned_spells
            .iter()
            .any(|l| l.spell == SpellKind::Fireball);
        if learned {
            saw_success = true;
        } else if s_sess.world().status_timers.confused > 0 {
            saw_fail = true;
        }
        if saw_fail && saw_success {
            break;
        }
    }
    assert!(
        saw_fail && saw_success,
        "fail={saw_fail}, success={saw_success}"
    );

    // 4. Failed cast_at consumes half MP ((cost / 2).max(1)) and awards +1 Spellcasting practice.
    let mut observed_cast_fail = false;
    let fireball_cost = SpellKind::Fireball.mana_cost();
    let expected_fail_spend = (fireball_cost / 2).max(1);
    for seed in 1..=30 {
        let mut c_save = start_campaign(seed, Role::Mage).to_save_data();
        park_all_monsters_on(&mut c_save, LevelId::main(6));
        let armor = add_item(
            &mut c_save,
            ItemKind::ArmorChainMail,
            BucStatus::Uncursed,
            0,
            true,
        );
        let mut c_sess = GameSession::from_save_data(c_save).unwrap();
        quaff_gain_level(&mut c_sess, 2);
        let mut c_save2 = c_sess.to_save_data();
        c_save2.world.learned_spells.push(LearnedSpell {
            spell: SpellKind::Fireball,
            learned_at_turn: c_save2.turn,
        });
        c_sess = GameSession::from_save_data(c_save2).unwrap();
        assert!(c_sess.submit(C::Wear { item: armor }).accepted);
        let mp_before = c_sess.world().mp;
        assert!(mp_before >= fireball_cost);
        let prac_before = c_sess
            .world()
            .skill_practice
            .get(SkillCategory::Spellcasting.key())
            .copied()
            .unwrap_or(0);
        let cast_out = c_sess.submit(C::Cast {
            spell: SpellKind::Fireball,
            direction: Some(Direction::East),
        });
        assert!(cast_out.accepted);
        if c_sess.world().mp == mp_before - expected_fail_spend {
            observed_cast_fail = true;
            assert_eq!(
                c_sess
                    .world()
                    .skill_practice
                    .get(SkillCategory::Spellcasting.key())
                    .copied(),
                Some(prac_before + 1)
            );
            break;
        }
    }
    assert!(observed_cast_fail);
}

#[test]
fn a36_imp_f003_npc_flags_and_pet_hunger() {
    let reg = registry().unwrap();
    for mk in [
        MonsterKind::Shopkeeper,
        MonsterKind::TemplePriest,
        MonsterKind::Oracle,
        MonsterKind::Watchman,
        MonsterKind::WizardOfYendor,
        MonsterKind::HighPriestMoloch,
        MonsterKind::Orcus,
        MonsterKind::QuestNemesis,
        MonsterKind::RiderDeath,
        MonsterKind::RiderPestilence,
        MonsterKind::RiderFamine,
    ] {
        assert!(
            monster_has_ai_flag(mk, "npc", reg),
            "{mk:?} must carry the 'npc' AI flag"
        );
    }

    let mut save = start_campaign(42, Role::Wizard).to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let wand_poly = add_item(
        &mut save,
        ItemKind::WandPolymorph,
        BucStatus::Uncursed,
        0,
        true,
    );

    let mut sess = GameSession::from_save_data(save).unwrap();
    quaff_gain_level(&mut sess, 9);
    let mut prep = sess.to_save_data();
    let ppos = prep
        .world
        .entities
        .actor_location(prep.world.player_id)
        .unwrap()
        .1;
    let map = prep.world.levels.map(main1).unwrap().clone();
    let clear_dirs: Vec<(Direction, Pos)> = Direction::ALL
        .into_iter()
        .filter_map(|d| {
            let p = ppos.offset(d.delta());
            (map.contains(p)
                && map.tile(p).ok() == Some(TileKind::Floor)
                && prep.world.entities.alive_actor_at(main1, p).is_none())
            .then_some((d, p))
        })
        .collect();
    let (npc_dir, npc_pos) = clear_dirs[0];
    let (pet_dir, pet_pos) = clear_dirs[1];

    let wiz = spawn_monster_at(
        &mut prep,
        MonsterKind::WizardOfYendor,
        main1,
        npc_pos,
        Faction::Hostile,
    );
    let kitten_hostile = spawn_monster_at(
        &mut prep,
        MonsterKind::Kitten,
        main1,
        pet_pos,
        Faction::Hostile,
    );
    let turn_now = prep.turn;
    for spell in [SpellKind::CharmMonster, SpellKind::CreateFamiliar] {
        prep.world.learned_spells.push(LearnedSpell {
            spell,
            learned_at_turn: turn_now,
        });
    }
    prep.world.mp = prep.world.max_mp;
    sess = GameSession::from_save_data(prep).unwrap();

    // NPC resists CharmMonster and WandPolymorph.
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::CharmMonster,
            direction: Some(npc_dir),
        })
        .accepted
    );
    assert_ne!(
        sess.world().entities().get(wiz).unwrap().faction(),
        Some(Faction::Pet)
    );
    assert!(
        sess.submit(C::Zap {
            item: wand_poly,
            direction: npc_dir,
        })
        .accepted
    );
    assert_eq!(
        sess.world().entities().get(wiz).unwrap().actor_kind(),
        Some(ActorKind::Monster(MonsterKind::WizardOfYendor))
    );

    // CharmMonster on eligible creature initializes DEFAULT_PET_HUNGER (ticks by 1 at turn end).
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::CharmMonster,
            direction: Some(pet_dir),
        })
        .accepted
    );
    let charmed = sess.world().entities().get(kitten_hostile).unwrap();
    assert_eq!(charmed.faction(), Some(Faction::Pet));
    assert_eq!(charmed.actor_pet_hunger(), Some(DEFAULT_PET_HUNGER - 1));

    // CreateFamiliar initializes DEFAULT_PET_HUNGER on the new familiar (ticks by 1 at turn end).
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::CreateFamiliar,
            direction: None,
        })
        .accepted
    );
    let familiar = sess
        .world()
        .entities()
        .entities()
        .iter()
        .filter(|e| e.faction() == Some(Faction::Pet))
        .max_by_key(|e| e.id.0)
        .expect("familiar spawned");
    assert_eq!(familiar.actor_pet_hunger(), Some(DEFAULT_PET_HUNGER - 1));
}

#[test]
fn a36_imp_f004_make_invisible_wish_enchant_and_artifact_blast() {
    let mut save = start_campaign(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let (ppos, dir, target_pos) = find_floor_adjacent_to_tile(&save, main1, |_, _| true);
    let _ = ppos;
    // Pick a floor tile with an adjacent floor tile for target_pos cleanly.
    let (adj, mdir, mpos) = find_floor_adjacent_to_tile(&save, main1, |pos, tile| {
        tile == TileKind::Floor && save.world.entities.alive_actor_at(main1, pos).is_none()
    });
    let _ = (dir, target_pos);
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, adj);
    let mob = spawn_monster_at(
        &mut save,
        MonsterKind::Jackal,
        main1,
        mpos,
        Faction::Hostile,
    );
    let wand_invis = add_item(
        &mut save,
        ItemKind::WandMakeInvisible,
        BucStatus::Uncursed,
        0,
        true,
    );
    let wand_cancel = add_item(
        &mut save,
        ItemKind::WandCancellation,
        BucStatus::Uncursed,
        0,
        true,
    );
    let wand_wish = add_item(
        &mut save,
        ItemKind::WandWishing,
        BucStatus::Uncursed,
        0,
        true,
    );
    let see_invis_ring = add_item(
        &mut save,
        ItemKind::RingSeeInvisible,
        BucStatus::Uncursed,
        0,
        true,
    );

    let mut sess = GameSession::from_save_data(save).unwrap();
    quaff_gain_level(&mut sess, 3);

    // 1. WandMakeInvisible sets actor_invisible, hides from visible_entities until RingSeeInvisible, Cancellation clears.
    assert!(
        sess.submit(C::Zap {
            item: wand_invis,
            direction: mdir,
        })
        .accepted
    );
    assert_eq!(
        sess.world().entities().get(mob).unwrap().actor_invisible(),
        Some(true)
    );
    assert!(
        !sess
            .observation()
            .visible_entities
            .iter()
            .any(|v| v.entity == mob),
        "invisible monster must stay hidden without SeeInvisible/Telepathy"
    );
    assert!(
        sess.submit(C::PutOn {
            item: see_invis_ring
        })
        .accepted
    );
    assert!(
        sess.observation()
            .visible_entities
            .iter()
            .any(|v| v.entity == mob),
        "RingSeeInvisible reveals invisible monster"
    );
    assert!(
        sess.submit(C::Zap {
            item: wand_cancel,
            direction: mdir,
        })
        .accepted
    );
    assert_eq!(
        sess.world().entities().get(mob).unwrap().actor_invisible(),
        Some(false)
    );

    // 2. Wish enchantment out of -3..=3 is rejected without losing charges; -3 and 3 work.
    let charges_before = sess.world().entities().item_charges(wand_wish).unwrap();
    assert!(
        !sess
            .submit(C::Wish {
                item: wand_wish,
                wish: WishChoice::Item {
                    kind: ItemKind::LongSword,
                    blessed: false,
                    enchant: 4,
                },
            })
            .accepted
    );
    assert!(
        !sess
            .submit(C::Wish {
                item: wand_wish,
                wish: WishChoice::Item {
                    kind: ItemKind::LongSword,
                    blessed: false,
                    enchant: -4,
                },
            })
            .accepted
    );
    assert_eq!(
        sess.world().entities().item_charges(wand_wish).unwrap(),
        charges_before
    );
    assert!(
        sess.submit(C::Wish {
            item: wand_wish,
            wish: WishChoice::Item {
                kind: ItemKind::LongSword,
                blessed: false,
                enchant: -3,
            },
        })
        .accepted
    );
    assert!(sess.world().inventory.entries.iter().any(|e| sess
        .world()
        .entities()
        .get(e.item)
        .is_some_and(|ent| ent.item_enchantment() == Some(-3))));

    // 3. Cross-aligned artifact wish deals 2d6 ArtifactBlast to Neutral Mage (lethal if HP <= 2).
    let hp_before = sess.observation().player.hp;
    let blast_wish = sess.submit(C::Wish {
        item: wand_wish,
        wish: WishChoice::ArtifactExcalibur,
    });
    assert!(blast_wish.accepted);
    assert!(sess.observation().player.hp < hp_before);
    assert!(blast_wish.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => text.contains("blasted by Excalibur"),
        _ => false,
    }));

    let mut lethal_wish = sess.to_save_data();
    let lethal_wand = add_item(
        &mut lethal_wish,
        ItemKind::WandWishing,
        BucStatus::Uncursed,
        0,
        true,
    );
    lethal_wish
        .world
        .entities
        .actor_stats_mut(lethal_wish.world.player_id)
        .unwrap()
        .hp = 1;
    let mut lethal_sess = GameSession::from_save_data(lethal_wish).unwrap();
    assert!(
        lethal_sess
            .submit(C::Wish {
                item: lethal_wand,
                wish: WishChoice::ArtifactStormbringer,
            })
            .accepted
    );
    assert!(matches!(
        lethal_sess.run_state(),
        RunState::GameOver {
            cause: DeathCause::ArtifactBlast,
            ..
        }
    ));
}

#[test]
fn a36_dbg_f003_shopkeeper_death_scoping_and_oracle() {
    let mut save = start_campaign(42, Role::Knight).to_save_data();
    let main2 = LevelId::main(2);
    let mines1 = LevelId::new(BranchId::Mines, 1);
    let main4 = LevelId::main(4);

    // Clear non-shopkeeper monsters on Main 2 and Mines 1 so nothing interferes.
    let to_park: Vec<EntityId> = save
        .world
        .entities
        .entities()
        .iter()
        .filter_map(|e| {
            let (ActorKind::Monster(mk), _, l, _, _, true) = e.actor()? else {
                return None;
            };
            if (l == main2 || l == mines1) && mk != MonsterKind::Shopkeeper {
                Some(e.id)
            } else {
                None
            }
        })
        .collect();
    let main6_floor = find_tile(&save, LevelId::main(6), |t| t == TileKind::Floor);
    for id in to_park {
        save.world
            .entities
            .set_actor_location(id, LevelId::main(6), main6_floor);
    }

    let m2_unpaid = save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            let (_, _, loc, _, _) = e.item()?;
            match loc {
                EntityLocation::OnMap { level, pos }
                    if level == main2 && e.item_unpaid() == Some(true) =>
                {
                    Some((e.id, pos))
                }
                _ => None,
            }
        })
        .expect("Main 2 must bootstrap unpaid merchandise");
    let mi1_unpaid = save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            let (_, _, loc, _, _) = e.item()?;
            match loc {
                EntityLocation::OnMap { level, pos }
                    if level == mines1 && e.item_unpaid() == Some(true) =>
                {
                    Some((e.id, pos))
                }
                _ => None,
            }
        })
        .expect("Mines 1 must bootstrap unpaid merchandise");

    save.world.current_level = main2;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main2, m2_unpaid.1);
    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::Pickup).accepted);
    let main2_debt = sess.world().shop_debt;
    assert!(main2_debt > 0);

    let mut s2 = sess.to_save_data();
    s2.world.current_level = mines1;
    s2.world
        .entities
        .set_actor_location(s2.world.player_id, mines1, mi1_unpaid.1);
    sess = GameSession::from_save_data(s2).unwrap();
    assert!(sess.submit(C::Pickup).accepted);
    let total_debt = sess.world().shop_debt;
    assert!(total_debt > main2_debt);
    let mines1_debt = total_debt - main2_debt;

    // Kill ONLY Main 2 shopkeeper using WandDeath.
    let mut s3 = sess.to_save_data();
    let wand_death = add_item(&mut s3, ItemKind::WandDeath, BucStatus::Uncursed, 0, true);
    let sk_m2 = s3
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            let (ActorKind::Monster(MonsterKind::Shopkeeper), _, l, _, _, true) = e.actor()? else {
                return None;
            };
            (l == main2).then_some(e.id)
        })
        .unwrap();
    let (_, sk_pos, sk_dir) = {
        let sk_p = s3.world.entities.actor_location(sk_m2).unwrap().1;
        let map = s3.world.levels.map(main2).unwrap();
        let d = Direction::ALL
            .into_iter()
            .find(|dir| {
                let p = Pos {
                    x: sk_p.x - dir.delta().dx,
                    y: sk_p.y - dir.delta().dy,
                };
                map.contains(p) && map.tile(p).ok() == Some(TileKind::Floor)
            })
            .unwrap();
        (
            sk_m2,
            Pos {
                x: sk_p.x - d.delta().dx,
                y: sk_p.y - d.delta().dy,
            },
            d,
        )
    };
    s3.world.current_level = main2;
    s3.world
        .entities
        .set_actor_location(s3.world.player_id, main2, sk_pos);
    sess = GameSession::from_save_data(s3).unwrap();
    let zap = sess.submit(C::Zap {
        item: wand_death,
        direction: sk_dir,
    });
    assert!(zap.accepted, "{:?}", zap.events);
    assert!(!sess.world().entities().get(sk_m2).unwrap().is_alive_actor());
    assert_eq!(sess.world().shop_debt, mines1_debt);
    assert_eq!(
        sess.world()
            .entities()
            .get(m2_unpaid.0)
            .unwrap()
            .item_unpaid(),
        Some(false)
    );
    assert_eq!(
        sess.world()
            .entities()
            .get(mi1_unpaid.0)
            .unwrap()
            .item_unpaid(),
        Some(true)
    );

    // Oracle consultation identifies enchant_known and emits GameEvent::TileRevealed.
    let mut o_save = start_campaign(42, Role::Knight).to_save_data();
    o_save.world.gold = 200;
    let oracle_pos = o_save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            let (ActorKind::Monster(MonsterKind::Oracle), _, l, p, _, true) = e.actor()? else {
                return None;
            };
            (l == main4).then_some(p)
        })
        .expect("Main 4 Oracle present");
    let adj_oracle = Direction::ALL
        .into_iter()
        .find_map(|d| {
            let p = oracle_pos.offset(d.delta());
            (o_save.world.levels.map(main4).unwrap().tile(p).ok() == Some(TileKind::Floor))
                .then_some(p)
        })
        .unwrap();
    // Clear inventory cleanly except one unidentified +2 sword; Main 4 naturally has hidden features.
    clear_inventory(&mut o_save);
    let mystery = add_item(
        &mut o_save,
        ItemKind::LongSword,
        BucStatus::Blessed,
        2,
        false,
    );
    o_save.world.current_level = main4;
    o_save
        .world
        .entities
        .set_actor_location(o_save.world.player_id, main4, adj_oracle);
    let mut o_sess = GameSession::from_save_data(o_save).unwrap();
    let pay = o_sess.submit(C::Pay);
    assert!(pay.accepted, "{:?}", pay.events);
    assert!(pay
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::TileRevealed { .. })));
    assert_eq!(
        o_sess
            .world()
            .entities()
            .get(mystery)
            .unwrap()
            .item_enchant_known(),
        Some(true)
    );
}

#[test]
fn a36_sec_f001_turn0_progression_anti_forging() {
    let mut bad_xp = start_campaign(42, Role::Mage).to_save_data();
    bad_xp.turn = 0;
    bad_xp.world.campaign.as_mut().unwrap().xp = 20;
    assert!(GameSession::from_save_data(bad_xp).is_err());

    let mut bad_hp = start_campaign(42, Role::Mage).to_save_data();
    bad_hp.turn = 0;
    bad_hp
        .world
        .entities
        .actor_stats_mut(bad_hp.world.player_id)
        .unwrap()
        .max_hp += 10;
    assert!(GameSession::from_save_data(bad_hp).is_err());

    let mut bad_mp = start_campaign(42, Role::Mage).to_save_data();
    bad_mp.turn = 0;
    bad_mp.world.max_mp += 10;
    assert!(GameSession::from_save_data(bad_mp).is_err());

    // Turn > 0 with unquaffed/unidentified PotionGainLevel on floor cannot justify forged XP.
    let mut forged_xp = start_campaign(42, Role::Mage).to_save_data();
    forged_xp.turn = 50;
    let ppos = forged_xp
        .world
        .entities
        .actor_location(forged_xp.world.player_id)
        .unwrap()
        .1;
    forged_xp
        .world
        .entities
        .spawn_item(
            ItemKind::PotionGainLevel,
            EntityLocation::OnMap {
                level: LevelId::main(1),
                pos: ppos,
            },
        )
        .unwrap();
    forged_xp
        .world
        .identified_items
        .retain(|k| *k != ItemKind::PotionGainLevel);
    let camp = forged_xp.world.campaign.as_mut().unwrap();
    camp.xp = 180;
    let max_hp = camp.max_hp();
    let max_mp = camp.max_mp();
    let hit = camp.hit_bonus();
    let dmg = camp.damage_bonus();
    forged_xp.world.max_mp = max_mp;
    forged_xp.world.mp = max_mp;
    let stats = forged_xp
        .world
        .entities
        .actor_stats_mut(forged_xp.world.player_id)
        .unwrap();
    stats.max_hp = max_hp;
    stats.hp = max_hp;
    stats.hit_bonus = hit;
    stats.damage_bonus = dmg;
    assert!(GameSession::from_save_data(forged_xp).is_err());
}

#[test]
fn a36_sec_f002_no_unidentified_or_buc_leakage() {
    let mut save = start_campaign(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let main3 = LevelId::main(3);
    let altar_pos = find_tile(&save, main3, |t| t == TileKind::Altar);
    save.world.current_level = main3;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main3, altar_pos);
    clear_inventory(&mut save);
    save.world.identified_items.clear();

    let unid_wish = add_item(
        &mut save,
        ItemKind::WandWishing,
        BucStatus::Uncursed,
        0,
        false,
    );
    let wish_letter = save.world.entities.item_letter(unid_wish).unwrap().0;
    let unid_fire = add_item(&mut save, ItemKind::WandFire, BucStatus::Uncursed, 0, false);
    let unid_water = add_item(
        &mut save,
        ItemKind::PotionWater,
        BucStatus::Blessed,
        0,
        false,
    );
    let unid_heal = add_item(
        &mut save,
        ItemKind::PotionHealing,
        BucStatus::Uncursed,
        0,
        false,
    );

    let sess = GameSession::from_save_data(save).unwrap();
    let obs = sess.observation();
    assert!(!obs
        .legal_actions
        .iter()
        .any(|c| matches!(c, C::Wish { item, .. } if *item == unid_wish)));
    assert!(!obs.legal_actions.iter().any(|c| matches!(
        c,
        C::Engrave {
            method: EngravingMethod::Item(id),
            ..
        } if *id == unid_fire
    )));
    assert!(obs.legal_actions.contains(&C::Dip {
        item: unid_water,
        target: DipTarget::Tile,
    }));
    assert!(obs.legal_actions.contains(&C::Dip {
        item: unid_heal,
        target: DipTarget::Tile,
    }));

    // TUI 'z' menu must not show Make a Wish before WandWishing is identified.
    let saved_for_dip = sess.to_save_data();
    let clock = Arc::new(TestClock::default());
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        sess,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    send_key(&mut app, &clock, KeyCode::Char('z'), 80, 24);
    let list_text = render_text(&mut app, 80, 24);
    assert!(!list_text.contains("wishing"));
    send_key(&mut app, &clock, KeyCode::Char(wish_letter), 80, 24);
    let sub_text = render_text(&mut app, 80, 24);
    assert!(
        !sub_text.contains("Make a Wish"),
        "TUI leaked Make a Wish menu for unidentified wand:\n{sub_text}"
    );
    send_key(&mut app, &clock, KeyCode::Esc, 80, 24);

    // Dipping unidentified non-water potion on co-aligned altar responds "Nothing happens." instead of oracle Err.
    let mut sess = GameSession::from_save_data(saved_for_dip).unwrap();
    let dip_nonwater = sess.submit(C::Dip {
        item: unid_heal,
        target: DipTarget::Tile,
    });
    assert!(dip_nonwater.accepted, "{:?}", dip_nonwater.events);
    assert!(dip_nonwater.events.iter().any(|e| match e {
        GameEvent::Message { text, .. } => text == "Nothing happens.",
        _ => false,
    }));

    // After zapping unidentified WandWishing, it becomes identified and TUI 'z' shows Make a Wish.
    assert!(
        sess.submit(C::Zap {
            item: unid_wish,
            direction: Direction::East,
        })
        .accepted
    );
    assert!(sess
        .observation()
        .campaign
        .unwrap()
        .identified_items
        .contains(&ItemKind::WandWishing));
    let post_id_save = sess.to_save_data();
    let mut app2 = TuiApp::new_with_llm_enabled_and_clock(
        sess,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    send_key(&mut app2, &clock, KeyCode::Char('z'), 80, 24);
    send_key(&mut app2, &clock, KeyCode::Char(wish_letter), 80, 24);
    let id_sub_text = render_text(&mut app2, 80, 24);
    assert!(
        id_sub_text.contains("Make a Wish"),
        "identified WandWishing should open Make a Wish menu:\n{id_sub_text}"
    );
    let sess = GameSession::from_save_data(post_id_save).unwrap();

    // Monster pickup logs use masked appearance name rather than real item name.
    let mut m_save = sess.to_save_data();
    let (adj, _d, orc_pos) = find_floor_adjacent_to_tile(&m_save, main1, |pos, tile| {
        tile == TileKind::Floor && m_save.world.entities.alive_actor_at(main1, pos).is_none()
    });
    m_save.world.current_level = main1;
    m_save
        .world
        .entities
        .set_actor_location(m_save.world.player_id, main1, adj);
    spawn_monster_at(
        &mut m_save,
        MonsterKind::Orc,
        main1,
        orc_pos,
        Faction::Hostile,
    );
    m_save
        .world
        .entities
        .spawn_item(
            ItemKind::PotionHealing,
            EntityLocation::OnMap {
                level: main1,
                pos: orc_pos,
            },
        )
        .unwrap();
    let expected_app = appearance_for_kind(42, ItemKind::PotionHealing).unwrap();
    let mut m_sess = GameSession::from_save_data(m_save).unwrap();
    let turn = m_sess.submit(C::Wait);
    assert!(turn.accepted);
    for ev in &turn.events {
        if let GameEvent::Message { text, .. } = ev {
            assert!(!text.contains("healing potion"), "leaked real name: {text}");
            if text.contains("picks up") {
                assert!(
                    text.contains(expected_app),
                    "expected '{expected_app}' in {text}"
                );
            }
        }
    }
}

#[test]
fn a36_dbg_f004_non_diggable_dig_actions_filtered() {
    let mut save = start_campaign(42, Role::Archeologist).to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let pick = add_item(&mut save, ItemKind::PickAxe, BucStatus::Uncursed, 0, true);

    for (lvl, expect_dig) in [
        (LevelId::new(BranchId::Sokoban, 1), false),
        (LevelId::new(BranchId::Planes, 2), false),
        (LevelId::new(BranchId::Planes, 3), false),
        (LevelId::new(BranchId::Planes, 4), false),
        (LevelId::new(BranchId::Planes, 5), false),
        (LevelId::new(BranchId::Planes, 1), true),
    ] {
        let mut s = save.clone();
        let map = s.world.levels.map(lvl).unwrap().clone();
        let adj = (1..map.height - 1)
            .flat_map(|y| (1..map.width - 1).map(move |x| Pos { x, y }))
            .find(|&pos| {
                let Ok(t) = map.tile(pos) else {
                    return false;
                };
                if t == TileKind::Wall {
                    return false;
                }
                Direction::ALL.into_iter().any(|d| {
                    let nb = pos.offset(d.delta());
                    nb.x > 0
                        && nb.x < map.width - 1
                        && nb.y > 0
                        && nb.y < map.height - 1
                        && map.tile(nb).ok() == Some(TileKind::Wall)
                })
            })
            .expect("cell adjacent to interior wall");
        s.world.current_level = lvl;
        s.world
            .entities
            .set_actor_location(s.world.player_id, lvl, adj);
        let sess = GameSession::from_save_data(s).unwrap();
        let has_dig = sess
            .observation()
            .legal_actions
            .iter()
            .any(|c| matches!(c, C::Apply { item, direction: Some(_) } if *item == pick));
        assert_eq!(
            has_dig, expect_dig,
            "unexpected dig legal action on {lvl:?}"
        );
    }
}

#[test]
fn a36_dbg_f005_edge_cases() {
    // 1. AmuletLifeSaving resets status_timers & hallucinating and recomputes AC.
    let mut save = start_campaign(42, Role::Mage).to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut save, LevelId::main(6));
    let amulet = add_item(
        &mut save,
        ItemKind::AmuletLifeSaving,
        BucStatus::Uncursed,
        0,
        true,
    );
    save.world.status_timers.blind = 5;
    save.world.status_timers.confused = 5;
    save.world.hallucinating = true;
    save.world
        .entities
        .actor_stats_mut(save.world.player_id)
        .unwrap()
        .hp = 1;
    let cursed_book = add_item(
        &mut save,
        ItemKind::SpellbookMagicMissile,
        BucStatus::Cursed,
        0,
        true,
    );
    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::PutOn { item: amulet }).accepted);
    let ac_expected = sess.observation().player.ac;
    assert!(sess.submit(C::Read { item: cursed_book }).accepted);
    assert!(matches!(sess.run_state(), RunState::Playing));
    assert_eq!(sess.world().status_timers.blind, 0);
    assert_eq!(sess.world().status_timers.confused, 0);
    assert!(!sess.world().hallucinating);
    assert_eq!(sess.observation().player.ac, ac_expected);

    // 2. Non-player kill (Dwarf vs Orc faction war) does not credit player kill_count or gold.
    let mut fw = start_campaign(42, Role::Knight).to_save_data();
    park_all_monsters_on(&mut fw, LevelId::main(6));
    fw.world.gold = 0;
    let (adj, _, pos_a) = find_floor_adjacent_to_tile(&fw, main1, |p, t| {
        t == TileKind::Floor && fw.world.entities.alive_actor_at(main1, p).is_none()
    });
    let map1 = fw.world.levels.map(main1).unwrap().clone();
    let pos_b = Direction::ALL
        .into_iter()
        .find_map(|d| {
            let p = pos_a.offset(d.delta());
            (p != adj && map1.contains(p) && map1.tile(p).ok() == Some(TileKind::Floor))
                .then_some(p)
        })
        .unwrap();
    let far = find_tile(&fw, main1, |t| t == TileKind::StairsDown);
    fw.world
        .entities
        .set_actor_location(fw.world.player_id, main1, far);
    spawn_monster_at(&mut fw, MonsterKind::Dwarf, main1, pos_a, Faction::Hostile);
    let orc = spawn_monster_at(&mut fw, MonsterKind::Orc, main1, pos_b, Faction::Hostile);
    fw.world.entities.actor_stats_mut(orc).unwrap().hp = 1;
    let mut fw_sess = GameSession::from_save_data(fw).unwrap();
    for _ in 0..3 {
        let _ = fw_sess.submit(C::Wait);
        if !fw_sess
            .world()
            .entities()
            .get(orc)
            .unwrap()
            .is_alive_actor()
        {
            break;
        }
    }
    assert_eq!(fw_sess.world().kill_count, 0);
    assert_eq!(fw_sess.world().gold, 0);

    // 3. Monster picking up two same-slot armors does not stack AC twice.
    let mut ac_save = start_campaign(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut ac_save, LevelId::main(6));
    let (p_far, _, orc_p) = find_floor_adjacent_to_tile(&ac_save, main1, |p, t| {
        t == TileKind::Floor && ac_save.world.entities.alive_actor_at(main1, p).is_none()
    });
    ac_save
        .world
        .entities
        .set_actor_location(ac_save.world.player_id, main1, p_far);
    let m_orc = spawn_monster_at(
        &mut ac_save,
        MonsterKind::Orc,
        main1,
        orc_p,
        Faction::Hostile,
    );
    let orc_ac_base = ac_save.world.entities.actor_stats(m_orc).unwrap().ac;
    ac_save
        .world
        .entities
        .spawn_item(
            ItemKind::ArmorChainMail,
            EntityLocation::OnMap {
                level: main1,
                pos: orc_p,
            },
        )
        .unwrap();
    ac_save
        .world
        .entities
        .spawn_item(
            ItemKind::ArmorChainMail,
            EntityLocation::OnMap {
                level: main1,
                pos: orc_p,
            },
        )
        .unwrap();
    let mut ac_sess = GameSession::from_save_data(ac_save).unwrap();
    assert!(ac_sess.submit(C::Wait).accepted);
    assert!(ac_sess.submit(C::Wait).accepted);
    let orc_ac_after = ac_sess.world().entities().actor_stats(m_orc).unwrap().ac;
    assert_eq!(orc_ac_after, (orc_ac_base - 3).max(-10));

    // 4. Reflected WandDeath against an undead Vampire (without magic_resistance) does not kill it.
    let mut ref_save = start_campaign(42, Role::Mage).to_save_data();
    park_all_monsters_on(&mut ref_save, LevelId::main(6));
    let refl = add_item(
        &mut ref_save,
        ItemKind::AmuletReflection,
        BucStatus::Uncursed,
        0,
        true,
    );
    let (p_ref, _, v_pos) = find_floor_adjacent_to_tile(&ref_save, main1, |p, t| {
        t == TileKind::Floor && ref_save.world.entities.alive_actor_at(main1, p).is_none()
    });
    ref_save
        .world
        .entities
        .set_actor_location(ref_save.world.player_id, main1, p_ref);
    let vamp = spawn_monster_at(
        &mut ref_save,
        MonsterKind::Vampire,
        main1,
        v_pos,
        Faction::Hostile,
    );
    ref_save
        .world
        .entities
        .spawn_item(
            ItemKind::WandDeath,
            EntityLocation::OnMap {
                level: main1,
                pos: v_pos,
            },
        )
        .unwrap();
    let mut ref_sess = GameSession::from_save_data(ref_save).unwrap();
    assert!(ref_sess.submit(C::PutOn { item: refl }).accepted);
    assert!(ref_sess.submit(C::Wait).accepted);
    assert!(ref_sess.submit(C::Wait).accepted);
    assert!(ref_sess
        .world()
        .entities()
        .get(vamp)
        .unwrap()
        .is_alive_actor());

    // 5. MasterKeyOfThievery invoke emits GameEvent::DoorChanged (Locked -> Closed).
    let mut mk_save = start_build(42, Role::Rogue, Race::Human, Alignment::Chaotic).to_save_data();
    park_all_monsters_on(&mut mk_save, LevelId::main(6));
    let (d_adj, _, d_pos) = find_floor_adjacent_to_tile(&mk_save, main1, |_, tile| {
        tile == TileKind::Door(DoorState::Closed)
    });
    mk_save
        .world
        .levels
        .get_mut(main1)
        .unwrap()
        .map
        .set_tile(d_pos, TileKind::Door(DoorState::Locked))
        .unwrap();
    mk_save
        .world
        .entities
        .set_actor_location(mk_save.world.player_id, main1, d_adj);
    let mkey = mk_save
        .world
        .entities
        .entities()
        .iter()
        .find(|e| e.item_artifact_id() == Some("master_key_of_thievery"))
        .expect("Rogue quest artifact already bootstrapped")
        .id;
    let mk_owner = mk_save.world.player_id;
    mk_save
        .world
        .entities
        .set_item_location(mkey, EntityLocation::Inventory { owner: mk_owner });
    let mk_letter = mk_save
        .world
        .inventory
        .add_existing_with_next_letter(mkey)
        .unwrap();
    mk_save.world.entities.set_item_letter(mkey, mk_letter);
    let mut mk_sess = GameSession::from_save_data(mk_save).unwrap();
    let inv = mk_sess.submit(C::Invoke { item: mkey });
    assert!(inv.accepted, "{:?}", inv.events);
    assert!(inv.events.iter().any(|e| matches!(
        e,
        GameEvent::DoorChanged {
            pos,
            from: DoorState::Locked,
            to: DoorState::Closed,
        } if *pos == d_pos
    )));
}
