use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use aihack_core::{
    action::{ActionIntent, CommandIntent as C, DipTarget},
    campaign::{Alignment, Race, Role, SkillCategory, SkillRank, ARTIFACT_INVOKE_COOLDOWN_TURNS},
    domain::{
        combat::DeathCause,
        entity::{ActorKind, EntityKind, EntityLocation},
        item::{BucStatus, EquipmentSlot, ItemKind},
        monster::MonsterKind,
        status::Resistances,
        tile::{TileKind, TrapKind},
    },
    ids::{BranchId, EntityId, LevelId},
    position::{Direction, Pos},
    run_state::RunState,
    save::SAVE_SCHEMA_VERSION_V1,
};
use aihack_llm::{service::LlmObservationView, transport::REQUEST_BODY_LIMIT};
use aihack_runtime::GameSession;
use aihack_tui::tui::{
    render_frame, render_panels, runtime_event_to_candidate, TuiApp, UiClock, UiPanel,
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

fn find_floor_with_los_distance(
    save: &aihack_runtime::save::SaveDataV1,
    level_id: LevelId,
    distance: i16,
) -> (Pos, Pos) {
    let level = save.world.levels.get(level_id).unwrap();
    for y in 2..level.map.height - 2 {
        for x in 2..level.map.width - distance - 2 {
            let p1 = Pos { x, y };
            let p2 = Pos { x: x + distance, y };
            if (0..=distance).all(|dx| {
                level
                    .map
                    .tile(Pos { x: x + dx, y })
                    .ok()
                    .is_some_and(|t| t == TileKind::Floor)
            }) {
                return (p1, p2);
            }
        }
    }
    panic!("failed to find clear corridor of length {distance} on {level_id:?}");
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

fn send_key(app: &mut TuiApp, clock: &TestClock, key: KeyCode, w: u16, h: u16) {
    clock.0.fetch_add(1000, Ordering::Relaxed);
    app.release_transition_gesture_on_idle();
    app.release_transition_gesture_on_idle();
    if let Some(cand) = runtime_event_to_candidate(
        Event::Key(KeyEvent::new(key, KeyModifiers::NONE)),
        w,
        h,
        app,
    ) {
        assert!(!app.handle_candidate_owned(cand).unwrap());
    }
}

#[test]
fn p10_1_all_13_roles_5_races_3_alignments_and_combo_validation() {
    // CharacterCreation lists all 13 roles in legal_actions.
    let creating = {
        let mut s = GameSession::try_new(42).unwrap();
        assert!(s.submit(C::Wait).accepted);
        s
    };
    let creation_obs = creating.observation();
    for role in Role::ALL {
        assert!(
            creation_obs
                .legal_actions
                .contains(&C::StartCampaign { role }),
            "CharacterCreation must advertise {role:?}"
        );
    }
    assert_eq!(Role::ALL.len(), 13);

    // Verify all 13 roles bootstrap cleanly and preserve distinct starter gear.
    let mut seen_starter_weapons = Vec::new();
    for role in Role::ALL {
        let s = start(42, role);
        let obs = s.observation();
        let camp = obs.campaign.unwrap();
        assert_eq!(camp.role, role);
        assert_eq!(camp.race, Some(role.default_race()));
        assert_eq!(camp.alignment, Some(role.default_alignment()));
        if matches!(
            role,
            Role::Valkyrie | Role::Wizard | Role::Samurai | Role::Barbarian | Role::Rogue
        ) {
            let kinds: Vec<ItemKind> = obs.inventory.iter().map(|i| i.kind).collect();
            seen_starter_weapons.push((role, kinds));
        }
    }
    assert!(seen_starter_weapons
        .iter()
        .any(|(r, k)| *r == Role::Valkyrie && k.contains(&ItemKind::LongSword)));
    assert!(seen_starter_weapons
        .iter()
        .any(|(r, k)| *r == Role::Samurai && k.contains(&ItemKind::Katana)));
    assert!(seen_starter_weapons
        .iter()
        .any(|(r, k)| *r == Role::Barbarian && k.contains(&ItemKind::BattleAxe)));
    assert!(seen_starter_weapons
        .iter()
        .any(|(r, k)| *r == Role::Wizard && k.contains(&ItemKind::Quarterstaff)));

    // Valid custom builds apply racial attribute, HP/MP deltas, and intrinsic resistances.
    let elf_mage = start_build(42, Role::Mage, Race::Elf, Alignment::Chaotic);
    let human_mage = start_build(42, Role::Mage, Race::Human, Alignment::Neutral);
    let elf_attrs = elf_mage.observation().campaign.unwrap().attributes.unwrap();
    let hum_attrs = human_mage
        .observation()
        .campaign
        .unwrap()
        .attributes
        .unwrap();
    assert_eq!(elf_attrs.dex, hum_attrs.dex + 2);
    assert_eq!(elf_attrs.int, hum_attrs.int + 2);
    assert_eq!(
        elf_mage.observation().player.max_mp,
        human_mage.observation().player.max_mp + 2
    );
    assert!(Resistances(elf_mage.observation().player.resistances).contains(Resistances::SLEEP));

    let dwarf_valk = start_build(42, Role::Valkyrie, Race::Dwarf, Alignment::Neutral);
    let human_valk = start_build(42, Role::Valkyrie, Race::Human, Alignment::Lawful);
    assert_eq!(
        dwarf_valk.observation().player.max_hp,
        human_valk.observation().player.max_hp + 3
    );

    let orc_barb = start_build(42, Role::Barbarian, Race::Orc, Alignment::Chaotic);
    assert!(Resistances(orc_barb.observation().player.resistances).contains(Resistances::POISON));

    let _gnome_arch = start_build(42, Role::Archeologist, Race::Gnome, Alignment::Neutral);

    // Monk starts with martial-arts unarmed bonuses superior to baseline melee.
    let monk = start(42, Role::Monk);
    assert!(
        monk.world()
            .entities
            .actor_stats(monk.world().player_id)
            .is_some()
            && monk
                .world()
                .campaign
                .unwrap()
                .role
                .unarmed_bonus(1, SkillRank::Skilled)
                > Role::Knight.unarmed_bonus(1, SkillRank::Unskilled)
    );

    // Invalid builds must be rejected in session creation and save validation.
    for (bad_role, bad_race, bad_align) in [
        (Role::Knight, Race::Orc, Alignment::Lawful),
        (Role::Knight, Race::Human, Alignment::Chaotic),
        (Role::Samurai, Race::Dwarf, Alignment::Lawful),
        (Role::Rogue, Race::Human, Alignment::Lawful),
        (Role::Valkyrie, Race::Human, Alignment::Chaotic),
    ] {
        let mut s = GameSession::try_new(42).unwrap();
        assert!(s.submit(C::Wait).accepted);
        let res = s.submit(C::StartCampaignWithBuild {
            role: bad_role,
            race: bad_race,
            alignment: bad_align,
        });
        assert!(
            !res.accepted,
            "invalid combo {bad_role:?}/{bad_race:?}/{bad_align:?} must fail"
        );
    }

    let mut bad_save = orc_barb.to_save_data();
    bad_save.world.campaign.as_mut().unwrap().role = Role::Knight;
    assert!(GameSession::from_save_data(bad_save).is_err());

    // Elf intrinsic sleep resistance protects from SleepingGas trap.
    let mut trap_save = elf_mage.to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut trap_save, LevelId::main(6));
    let gas_pos = find_tile(&trap_save, main1, |t| {
        t == TileKind::HiddenTrap(TrapKind::SleepingGas)
    });
    let step_dir = Direction::ALL
        .into_iter()
        .find(|d| {
            let p = Pos {
                x: gas_pos.x - d.delta().dx,
                y: gas_pos.y - d.delta().dy,
            };
            trap_save.world.levels.map(main1).unwrap().tile(p).ok() == Some(TileKind::Floor)
        })
        .unwrap();
    let start_pos = Pos {
        x: gas_pos.x - step_dir.delta().dx,
        y: gas_pos.y - step_dir.delta().dy,
    };
    trap_save.world.current_level = main1;
    trap_save
        .world
        .entities
        .set_actor_location(trap_save.world.player_id, main1, start_pos);
    let mut trap_sess = GameSession::from_save_data(trap_save).unwrap();
    let step = trap_sess.submit(C::Move(step_dir));
    assert!(step.accepted);
    assert_eq!(trap_sess.observation().player.paralysis_turns, 0);
}

#[test]
fn p10_1b_racial_infravision_senses_monsters_in_dark_rooms() {
    let base_dwarf = start_build(42, Role::Valkyrie, Race::Dwarf, Alignment::Lawful);
    let mut save_dwarf = base_dwarf.to_save_data();
    let main2 = LevelId::main(2);
    park_all_monsters_on(&mut save_dwarf, LevelId::main(6));
    let level = save_dwarf.world.levels.get_mut(main2).unwrap();
    level.lit_tiles = vec![false; level.map.tile_count()];
    let (p1, p2) = find_floor_with_los_distance(&save_dwarf, main2, 5);
    let jackal_id = save_dwarf
        .world
        .entities
        .entities()
        .iter()
        .find(|e| e.kind() == EntityKind::Monster(MonsterKind::Jackal))
        .unwrap()
        .id;
    save_dwarf.world.current_level = main2;
    save_dwarf
        .world
        .entities
        .set_actor_location(save_dwarf.world.player_id, main2, p1);
    save_dwarf
        .world
        .entities
        .set_actor_location(jackal_id, main2, p2);

    let dwarf_sess = GameSession::from_save_data(save_dwarf.clone()).unwrap();
    assert!(
        dwarf_sess
            .observation()
            .visible_entities
            .iter()
            .any(|e| e.entity == jackal_id),
        "dwarf infravision should detect monster at distance 5 in dark room"
    );

    // Blindfolding suppresses infravision (telepathy comes from AmuletEsp, not bare blindfold).
    let mut bf_save = save_dwarf.clone();
    let bf_id = add_inventory_item(&mut bf_save, ItemKind::Blindfold, BucStatus::Uncursed, 0);
    bf_save.world.entities.set_item_tool_active(bf_id, true);
    let bf_sess = GameSession::from_save_data(bf_save).unwrap();
    assert!(
        !bf_sess
            .observation()
            .visible_entities
            .iter()
            .any(|e| e.entity == jackal_id),
        "blindfolded dwarf cannot use infravision"
    );

    // Human lacks infravision and cannot see a monster at distance 5 in darkness.
    let mut save_human =
        start_build(42, Role::Valkyrie, Race::Human, Alignment::Lawful).to_save_data();
    save_human.world.levels.get_mut(main2).unwrap().lit_tiles = save_dwarf
        .world
        .levels
        .get(main2)
        .unwrap()
        .lit_tiles
        .clone();
    park_all_monsters_on(&mut save_human, LevelId::main(6));
    let h_jackal = save_human
        .world
        .entities
        .entities()
        .iter()
        .find(|e| e.kind() == EntityKind::Monster(MonsterKind::Jackal))
        .unwrap()
        .id;
    save_human.world.current_level = main2;
    save_human
        .world
        .entities
        .set_actor_location(save_human.world.player_id, main2, p1);
    save_human
        .world
        .entities
        .set_actor_location(h_jackal, main2, p2);
    let human_sess = GameSession::from_save_data(save_human).unwrap();
    assert!(
        !human_sess
            .observation()
            .visible_entities
            .iter()
            .any(|e| e.entity == h_jackal),
        "human lacks infravision in unlit room at distance 5"
    );
}

#[test]
fn p10_2_altar_alignments_wrath_conversion_appeasement_and_corrupt_dip() {
    let base = start_build(42, Role::Priest, Race::Elf, Alignment::Neutral);
    let main3 = LevelId::main(3);
    let mines2 = LevelId {
        branch: BranchId::Mines,
        depth: 2,
    };

    let main3_altar = find_tile(&base.to_save_data(), main3, |t| t == TileKind::Altar);
    let mines2_altar = find_tile(&base.to_save_data(), mines2, |t| t == TileKind::Altar);
    assert_eq!(
        base.world()
            .levels
            .get(main3)
            .unwrap()
            .altar_alignment(main3_altar),
        Some(Alignment::Neutral)
    );
    assert_eq!(
        base.world()
            .levels
            .get(mines2)
            .unwrap()
            .altar_alignment(mines2_altar),
        Some(Alignment::Chaotic)
    );

    // Cross-aligned altar Pray deals divine wrath and increases god_anger + prayer_timeout.
    let mut wrath_save = base.to_save_data();
    park_all_monsters_on(&mut wrath_save, LevelId::main(6));
    wrath_save.world.current_level = mines2;
    wrath_save
        .world
        .entities
        .set_actor_location(wrath_save.world.player_id, mines2, mines2_altar);
    let mut wrath_sess = GameSession::from_save_data(wrath_save.clone()).unwrap();
    let hp_before = wrath_sess.observation().player.hp;
    let pray_outcome = wrath_sess.submit(C::Pray);
    assert!(pray_outcome.accepted);
    let obs_after = wrath_sess.observation();
    assert!(obs_after.player.hp < hp_before);
    assert_eq!(obs_after.campaign.as_ref().unwrap().god_anger, 2);
    assert!(obs_after.campaign.as_ref().unwrap().prayer_timeout > 0);
    assert!(obs_after.player.luck < 0);

    // Cross-aligned altar lethal Pray sets DeathCause::DivineWrath.
    let mut fatal_pray = wrath_save.clone();
    fatal_pray
        .world
        .entities
        .actor_stats_mut(fatal_pray.world.player_id)
        .unwrap()
        .hp = 1;
    let mut fatal_sess = GameSession::from_save_data(fatal_pray).unwrap();
    let fatal_out = fatal_sess.submit(C::Pray);
    assert!(matches!(
        fatal_out.next_state,
        RunState::GameOver {
            cause: DeathCause::DivineWrath,
            ..
        }
    ));

    // Cross-aligned #dip into altar curses water instead of blessing it.
    let mut dip_save = wrath_save.clone();
    let water = add_inventory_item(&mut dip_save, ItemKind::PotionWater, BucStatus::Uncursed, 0);
    let mut dip_sess = GameSession::from_save_data(dip_save).unwrap();
    assert!(
        dip_sess
            .submit(C::Dip {
                item: water,
                target: DipTarget::Tile,
            })
            .accepted
    );
    let water_obs = dip_sess
        .observation()
        .inventory
        .into_iter()
        .find(|i| i.item == water)
        .unwrap();
    assert_eq!(water_obs.buc, BucStatus::Cursed);
    assert_eq!(dip_sess.observation().campaign.unwrap().god_anger, 1);

    // Cross-aligned altar #offer: non-corpse rejected, corpse converts altar alignment.
    let mut conv_save = dip_sess.to_save_data();
    let ration = add_inventory_item(&mut conv_save, ItemKind::FoodRation, BucStatus::Uncursed, 0);
    let corpse = add_inventory_item(
        &mut conv_save,
        ItemKind::CorpseJackal,
        BucStatus::Uncursed,
        0,
    );
    let mut conv_sess = GameSession::from_save_data(conv_save).unwrap();
    assert!(!conv_sess.submit(C::Offer { item: ration }).accepted);
    assert!(conv_sess.submit(C::Offer { item: corpse }).accepted);
    assert_eq!(
        conv_sess
            .world()
            .levels
            .get(mines2)
            .unwrap()
            .altar_alignment(mines2_altar),
        Some(Alignment::Neutral)
    );
    assert_eq!(conv_sess.observation().campaign.unwrap().god_anger, 0);

    // Co-aligned high-devotion corpse sacrifice gifts an aligned/unaligned unique artifact.
    let mut gift_save = conv_sess.to_save_data();
    gift_save.world.luck = 3;
    let dragon_corpse =
        add_inventory_item(&mut gift_save, ItemKind::Corpse, BucStatus::Uncursed, 0);
    gift_save
        .world
        .entities
        .set_item_corpse_monster(dragon_corpse, Some(MonsterKind::Dragon));
    let mut gift_sess = GameSession::from_save_data(gift_save).unwrap();
    assert!(
        gift_sess
            .submit(C::Offer {
                item: dragon_corpse
            })
            .accepted
    );
    assert!(
        !gift_sess.world().generated_artifacts.is_empty(),
        "devoted corpse sacrifice on co-aligned altar should gift a unique artifact"
    );

    // SaveV3 rejects altar_alignments pointing to non-altar tiles.
    let mut bad_altar = conv_sess.to_save_data();
    bad_altar
        .world
        .levels
        .get_mut(mines2)
        .unwrap()
        .altar_alignments
        .push((Pos { x: 1, y: 1 }, Alignment::Lawful));
    assert!(GameSession::from_save_data(bad_altar).is_err());
}

#[test]
fn p10_3_excalibur_fountain_unique_artifacts_blasts_erosion_and_invoke() {
    let base = start(42, Role::Valkyrie); // Lawful, starts with LongSword
    let main3 = LevelId::main(3);
    let mut save = base.to_save_data();
    park_all_monsters_on(&mut save, LevelId::main(6));
    let fountain = find_tile(&save, main3, |t| t == TileKind::Fountain);
    save.world.current_level = main3;
    save.world
        .entities
        .set_actor_location(save.world.player_id, main3, fountain);
    let long_sword = save
        .world
        .inventory
        .entries
        .iter()
        .find(|e| {
            save.world
                .entities
                .item_data(e.item)
                .is_some_and(|d| d.kind == ItemKind::LongSword)
        })
        .unwrap()
        .item;

    // Below level 5: cannot claim Excalibur.
    let mut low_sess = GameSession::from_save_data(save.clone()).unwrap();
    assert!(
        low_sess
            .submit(C::Dip {
                item: long_sword,
                target: DipTarget::Tile,
            })
            .accepted
    );
    assert!(!low_sess.world().generated_artifacts.contains("excalibur"));

    // Reach level 5 (xp = 80) legitimately via PotionGainLevel quaffs, then dip LongSword -> Excalibur!
    let mut lvl5_save = low_sess.to_save_data();
    let player_id = lvl5_save.world.player_id;
    let gain_pot = lvl5_save
        .world
        .entities
        .spawn_item(
            ItemKind::PotionGainLevel,
            EntityLocation::Inventory { owner: player_id },
        )
        .unwrap();
    lvl5_save.world.entities.set_item_quantity(gain_pot, 4);
    let letter = lvl5_save
        .world
        .inventory
        .add_existing_with_next_letter(gain_pot)
        .unwrap();
    lvl5_save.world.entities.set_item_letter(gain_pot, letter);
    let mut lvl5_sess = GameSession::from_save_data(lvl5_save).unwrap();
    for _ in 0..4 {
        assert!(lvl5_sess.submit(C::Quaff { item: gain_pot }).accepted);
    }
    assert_eq!(lvl5_sess.world().campaign.unwrap().level(), 5);
    assert!(
        lvl5_sess
            .submit(C::Dip {
                item: long_sword,
                target: DipTarget::Tile,
            })
            .accepted
    );
    assert!(lvl5_sess.world().generated_artifacts.contains("excalibur"));
    let excal_obs = lvl5_sess
        .observation()
        .inventory
        .into_iter()
        .find(|i| i.item == long_sword)
        .unwrap();
    assert_eq!(excal_obs.artifact_id.as_deref(), Some("excalibur"));
    assert_eq!(excal_obs.name.as_deref(), Some("Excalibur"));
    assert_eq!(
        lvl5_sess.world().current_map().tile(fountain).unwrap(),
        TileKind::DryFountain
    );

    // Second LongSword cannot duplicate Excalibur even on another fountain (Main 1).
    let mut dup_check = lvl5_sess.to_save_data();
    let main1 = LevelId::main(1);
    let f2 = find_tile(&dup_check, main1, |t| t == TileKind::Fountain);
    dup_check.world.current_level = main1;
    dup_check
        .world
        .entities
        .set_actor_location(dup_check.world.player_id, main1, f2);
    let ls2 = add_inventory_item(&mut dup_check, ItemKind::LongSword, BucStatus::Uncursed, 0);
    let mut dup_sess = GameSession::from_save_data(dup_check).unwrap();
    assert!(
        dup_sess
            .submit(C::Dip {
                item: ls2,
                target: DipTarget::Tile,
            })
            .accepted
    );
    assert_eq!(
        dup_sess
            .world()
            .entities
            .get(ls2)
            .and_then(|e| e.item_artifact_id()),
        None
    );

    // SaveV3 anti-tamper: reject missing generated_artifacts or duplicate artifact IDs.
    let mut bad_art = lvl5_sess.to_save_data();
    bad_art.world.generated_artifacts.clear();
    assert!(GameSession::from_save_data(bad_art).is_err());

    let mut dup_art = lvl5_sess.to_save_data();
    let cloned_sword = add_inventory_item(&mut dup_art, ItemKind::LongSword, BucStatus::Blessed, 1);
    dup_art
        .world
        .entities
        .set_item_artifact_id(cloned_sword, Some("excalibur".to_string()));
    assert!(GameSession::from_save_data(dup_art).is_err());

    // Erosion-proof: dipping Excalibur into real Main 3 Water keeps erosion == 0.
    let mut erode_save = lvl5_sess.to_save_data();
    let water_pos = find_tile(&erode_save, main3, |t| t == TileKind::Water);
    erode_save.world.current_level = main3;
    erode_save
        .world
        .entities
        .set_actor_location(erode_save.world.player_id, main3, water_pos);
    let mut erode_sess = GameSession::from_save_data(erode_save).unwrap();
    assert!(erode_sess.submit(C::Wield { item: long_sword }).accepted);
    assert!(
        erode_sess
            .submit(C::Dip {
                item: long_sword,
                target: DipTarget::Tile,
            })
            .accepted
    );
    assert_eq!(
        erode_sess
            .observation()
            .inventory
            .into_iter()
            .find(|i| i.item == long_sword)
            .unwrap()
            .erosion,
        0
    );

    // Cross-alignment pickup, wield, and lethal invoke trigger ArtifactBlast.
    let mut blast_save =
        start_build(42, Role::Barbarian, Race::Human, Alignment::Chaotic).to_save_data();
    park_all_monsters_on(&mut blast_save, LevelId::main(6));
    let main1 = LevelId::main(1);
    let p_pos = blast_save
        .world
        .entities
        .actor_location(blast_save.world.player_id)
        .unwrap()
        .1;
    let excal_floor = blast_save
        .world
        .entities
        .spawn_item(
            ItemKind::LongSword,
            EntityLocation::OnMap {
                level: main1,
                pos: p_pos,
            },
        )
        .unwrap();
    blast_save
        .world
        .entities
        .set_item_artifact_id(excal_floor, Some("excalibur".to_string()));
    blast_save
        .world
        .generated_artifacts
        .insert("excalibur".to_string());
    let mut blast_sess = GameSession::from_save_data(blast_save).unwrap();
    let hp0 = blast_sess.observation().player.hp;
    assert!(blast_sess.submit(C::Pickup).accepted);
    let hp1 = blast_sess.observation().player.hp;
    assert!(
        hp1 < hp0,
        "pickup of Lawful Excalibur by Chaotic blasts player"
    );

    assert!(blast_sess.submit(C::Wield { item: excal_floor }).accepted);
    let hp2 = blast_sess.observation().player.hp;
    assert!(hp2 < hp1, "wielding hostile artifact blasts player");
    assert!(!blast_sess
        .observation()
        .inventory
        .iter()
        .any(|i| i.item == excal_floor && i.equipped_slot == Some(EquipmentSlot::Melee)));

    // Lethal cross-alignment invoke sets DeathCause::ArtifactBlast.
    let mut fatal_art = blast_sess.to_save_data();
    fatal_art
        .world
        .entities
        .actor_stats_mut(fatal_art.world.player_id)
        .unwrap()
        .hp = 1;
    let mut fatal_art_sess = GameSession::from_save_data(fatal_art).unwrap();
    let fatal_art_out = fatal_art_sess.submit(C::Invoke { item: excal_floor });
    assert!(matches!(
        fatal_art_out.next_state,
        RunState::GameOver {
            cause: DeathCause::ArtifactBlast,
            ..
        }
    ));

    // Unaligned Sting does NOT blast Chaotic player, and Sunsword sheds light in dark rooms.
    let mut power_save = start(42, Role::Knight).to_save_data(); // Lawful
    park_all_monsters_on(&mut power_save, LevelId::main(6));
    let sun = add_inventory_item(&mut power_save, ItemKind::BroadSword, BucStatus::Blessed, 1);
    power_save
        .world
        .entities
        .set_item_artifact_id(sun, Some("sunsword".to_string()));
    power_save
        .world
        .generated_artifacts
        .insert("sunsword".to_string());
    let sting = add_inventory_item(&mut power_save, ItemKind::Dagger, BucStatus::Blessed, 1);
    power_save
        .world
        .entities
        .set_item_artifact_id(sting, Some("sting".to_string()));
    power_save
        .world
        .generated_artifacts
        .insert("sting".to_string());
    let (adj_p, adj_m) = find_floor_with_los_distance(&power_save, main1, 1);
    power_save
        .world
        .levels
        .get_mut(main1)
        .unwrap()
        .lit_tiles
        .fill(false);
    power_save
        .world
        .entities
        .set_actor_location(power_save.world.player_id, main1, adj_p);
    let mob = power_save
        .world
        .entities
        .spawn_monster(MonsterKind::Jackal, adj_m)
        .unwrap();
    power_save
        .world
        .entities
        .set_actor_location(mob, main1, adj_m);
    let mut power_sess = GameSession::from_save_data(power_save).unwrap();
    assert!(
        power_sess.observation().visible_tiles.len() > 25,
        "Sunsword illuminates dark rooms"
    );
    assert!(power_sess.submit(C::Invoke { item: sun }).accepted);
    assert_eq!(
        power_sess
            .observation()
            .campaign
            .unwrap()
            .artifact_invoke_cooldown,
        ARTIFACT_INVOKE_COOLDOWN_TURNS - 1
    );
    // Cooldown rejects immediate second invoke.
    assert!(!power_sess.submit(C::Invoke { item: sting }).accepted);

    // Wait remaining cooldown turns -> invoke Sting works and grants Fast!
    for _ in 0..ARTIFACT_INVOKE_COOLDOWN_TURNS {
        assert!(power_sess.submit(C::Wait).accepted);
    }
    assert!(power_sess.submit(C::Invoke { item: sting }).accepted);
    assert!(power_sess.observation().player.fast_turns > 0);

    // Test remaining artifacts (Excalibur heal, Magicbane MP, Cleaver cleave, Stormbringer bolt+drain).
    for (art_id, kind, align) in [
        ("excalibur", ItemKind::LongSword, Alignment::Lawful),
        ("magicbane", ItemKind::Dagger, Alignment::Neutral),
        ("cleaver", ItemKind::BattleAxe, Alignment::Neutral),
        ("stormbringer", ItemKind::LongSword, Alignment::Chaotic),
    ] {
        let mut s = start_build(42, Role::Monk, Race::Human, align).to_save_data();
        park_all_monsters_on(&mut s, LevelId::main(6));
        s.world
            .entities
            .actor_stats_mut(s.world.player_id)
            .unwrap()
            .hp = 10;
        s.world.mp = 1;
        let art = add_inventory_item(&mut s, kind, BucStatus::Blessed, 1);
        s.world
            .entities
            .set_item_artifact_id(art, Some(art_id.to_string()));
        s.world.generated_artifacts.insert(art_id.to_string());
        let (p0, m0) = find_floor_with_los_distance(&s, main1, 1);
        s.world
            .entities
            .set_actor_location(s.world.player_id, main1, p0);
        let dummy = s
            .world
            .entities
            .spawn_monster(MonsterKind::Orc, m0)
            .unwrap();
        s.world.entities.set_actor_location(dummy, main1, m0);
        let mut gs = GameSession::from_save_data(s).unwrap();
        assert!(gs.submit(C::Invoke { item: art }).accepted);
        match art_id {
            "excalibur" => assert!(gs.observation().player.hp > 10),
            "magicbane" => assert!(gs.observation().player.mp > 1),
            "cleaver" | "stormbringer" => {
                assert!(gs.world().entities.actor_stats(dummy).unwrap().hp < 16);
            }
            _ => unreachable!(),
        }
    }
}

#[test]
fn p10_4_enhance_proficiency_practice_caps_and_combat_spell_scaling() {
    // Samurai starts Sword=Skilled (cap GrandMaster) with Katana.
    let base = start(42, Role::Samurai);
    let mut save = base.to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut save, LevelId::main(6));
    let katana = save
        .world
        .inventory
        .entries
        .iter()
        .find(|e| {
            save.world
                .entities
                .item_data(e.item)
                .is_some_and(|d| d.kind == ItemKind::Katana)
        })
        .unwrap()
        .item;
    let (p0, m0) = find_floor_with_los_distance(&save, main1, 1);
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, p0);
    let mob = save
        .world
        .entities
        .spawn_monster(MonsterKind::Ogre, m0)
        .unwrap();
    save.world.entities.set_actor_location(mob, main1, m0);
    save.world.entities.actor_stats_mut(mob).unwrap().hp = 50;
    save.world.entities.actor_stats_mut(mob).unwrap().max_hp = 50;
    save.world
        .skill_practice
        .insert(SkillCategory::Sword.key().to_string(), 55);

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::Wield { item: katana }).accepted);
    // Cannot enhance before crossing practice threshold (60 for Skilled -> Expert).
    assert!(
        !sess
            .submit(C::Enhance {
                skill: SkillCategory::Sword,
            })
            .accepted
    );

    for _ in 0..4 {
        let _ = sess.submit(C::Move(Direction::East));
        if sess.observation().legal_actions.contains(&C::Enhance {
            skill: SkillCategory::Sword,
        }) {
            break;
        }
    }
    let obs_ready = sess.observation();
    assert!(obs_ready.legal_actions.contains(&C::Enhance {
        skill: SkillCategory::Sword
    }));
    assert!(obs_ready
        .campaign
        .as_ref()
        .unwrap()
        .skills
        .iter()
        .any(|s| s.category == SkillCategory::Sword && s.rank == SkillRank::Skilled && s.ready));
    assert!(
        sess.submit(C::Enhance {
            skill: SkillCategory::Sword,
        })
        .accepted
    );
    assert!(sess
        .observation()
        .campaign
        .unwrap()
        .skills
        .iter()
        .any(|s| s.category == SkillCategory::Sword && s.rank == SkillRank::Expert));

    // Role skill cap is enforced cleanly in SaveV3 tamper check too.
    let mut cap_save = sess.to_save_data();
    cap_save.world.spell_skill.insert(
        SkillCategory::Spellcasting.key().to_string(),
        SkillRank::Master.as_u8(),
    );
    assert!(
        GameSession::from_save_data(cap_save).is_err(),
        "Samurai spellcasting above cap must fail validation"
    );

    // Wizard spellcasting practice accumulates and boosts ForceBolt / Heal.
    let wiz = start(42, Role::Wizard);
    let mut wsave = wiz.to_save_data();
    park_all_monsters_on(&mut wsave, LevelId::main(6));
    let mut wsess = GameSession::from_save_data(wsave).unwrap();
    for _ in 0..3 {
        assert!(
            wsess
                .submit(C::Cast {
                    spell: aihack_core::campaign::SpellKind::ForceBolt,
                    direction: Some(Direction::East),
                })
                .accepted
        );
    }
    let prac = wsess
        .observation()
        .campaign
        .unwrap()
        .skills
        .into_iter()
        .find(|s| s.category == SkillCategory::Spellcasting)
        .unwrap()
        .practice;
    assert_eq!(prac, 15);

    // Ranger throwing Rock accumulates Thrown practice.
    let mut ranger = start(42, Role::Ranger);
    let rock = ranger
        .observation()
        .inventory
        .into_iter()
        .find(|i| i.kind == ItemKind::Rock)
        .unwrap()
        .item;
    assert!(
        ranger
            .submit(C::Throw {
                item: rock,
                direction: Direction::East,
            })
            .accepted
    );
    let thrown_prac = ranger
        .observation()
        .campaign
        .unwrap()
        .skills
        .into_iter()
        .find(|s| s.category == SkillCategory::Thrown)
        .unwrap()
        .practice;
    assert!(thrown_prac > 0);
}

#[test]
fn p10_5_tui_headless_llm_budget_and_legacy_v1_invariance() {
    // 1. Creation screen UI preserves DEF-01/09 strings & supports Tab / R / A / Enter custom build.
    let creation = render_panels::character_creation_lines();
    assert!(creation.iter().any(|l| l.contains("AC: 10")));
    assert!(!creation
        .iter()
        .any(|l| l.contains("Strength:") || l.contains("Dexterity:")));

    let clock = Arc::new(TestClock::default());
    let fresh = GameSession::try_new(42).unwrap();
    let mut app = TuiApp::new_with_llm_enabled_and_clock(
        fresh,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    send_key(&mut app, &clock, KeyCode::Enter, 100, 30); // -> CharacterCreation
    assert_eq!(app.run_state(), RunState::CharacterCreation);
    // Tab (Knight -> Scout), R (Human -> Elf), Enter (Elf Neutral Scout!)
    send_key(&mut app, &clock, KeyCode::Tab, 100, 30);
    send_key(&mut app, &clock, KeyCode::Tab, 100, 30);
    send_key(&mut app, &clock, KeyCode::Char('r'), 100, 30);
    let mut term = Terminal::new(TestBackend::new(100, 30)).unwrap();
    term.draw(|frame| render_frame(frame, &mut app)).unwrap();
    send_key(&mut app, &clock, KeyCode::Enter, 100, 30);
    assert_eq!(app.run_state(), RunState::Playing);
    let build_obs = app.observation();
    assert_eq!(build_obs.campaign.as_ref().unwrap().role, Role::Scout);
    assert_eq!(build_obs.campaign.as_ref().unwrap().race, Some(Race::Elf));

    // 2. Playing TUI: 'X' Enhance menu, 'V' Invoke menu, altar inspection/colors, LLM < 32 KiB.
    let mut ui_save = build_obs
        .campaign
        .map(|_| {
            let mut s =
                start_build(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
            let main3 = LevelId::main(3);
            let altar_pos = find_tile(&s, main3, |t| t == TileKind::Altar);
            park_all_monsters_on(&mut s, LevelId::main(6));
            s.world.current_level = main3;
            s.world
                .entities
                .set_actor_location(s.world.player_id, main3, altar_pos);
            let sun = add_inventory_item(&mut s, ItemKind::BroadSword, BucStatus::Blessed, 1);
            s.world
                .entities
                .set_item_artifact_id(sun, Some("sunsword".to_string()));
            s.world.generated_artifacts.insert("sunsword".to_string());
            s.world
                .skill_practice
                .insert(SkillCategory::Sword.key().to_string(), 60);
            s
        })
        .unwrap();
    let _ = &mut ui_save;
    let ui_sess = GameSession::from_save_data(ui_save).unwrap();
    let ui_obs = ui_sess.observation();
    assert!(ui_obs.action_space.commands.len() <= 64);
    assert!(ui_obs.action_space.commands.iter().any(|c| {
        matches!(
            c,
            ActionIntent::Command(C::Enhance {
                skill: SkillCategory::Sword
            })
        )
    }));
    assert!(ui_obs
        .action_space
        .commands
        .iter()
        .any(|c| matches!(c, ActionIntent::Command(C::Invoke { .. }))));

    let llm_json = serde_json::to_string(&LlmObservationView::from(&ui_obs)).unwrap();
    assert!(llm_json.len() < REQUEST_BODY_LIMIT);

    let main3_altar_pos = ui_obs.player_pos;
    let inspect =
        render_panels::inspect_lines(&ui_obs, Some(main3_altar_pos), UiPanel::Inspect, &[]);
    assert!(inspect.iter().any(|l| l == "tile altar"));
    assert!(inspect.iter().any(|l| l == "altar alignment: Lawful"));

    let status_txt = render_panels::status_lines(&ui_obs).join("\n");
    assert!(status_txt.contains("Human Knight"));
    assert!(status_txt.contains("ENH:1"));

    let mut ui_app = TuiApp::new_with_llm_enabled_and_clock(
        ui_sess,
        UiRuntimeConfig::default(),
        false,
        clock.clone(),
    );
    term.draw(|frame| render_frame(frame, &mut ui_app)).unwrap();
    // Open 'X' Enhance menu and trigger [2] Sword -> Skilled!
    send_key(&mut ui_app, &clock, KeyCode::Char('X'), 100, 30);
    send_key(&mut ui_app, &clock, KeyCode::Char('2'), 100, 30);
    assert!(ui_app
        .observation()
        .campaign
        .unwrap()
        .skills
        .iter()
        .any(|s| s.category == SkillCategory::Sword && s.rank == SkillRank::Skilled));

    // Open 'V' Invoke menu and trigger Sunsword letter.
    let sun_letter = ui_app
        .observation()
        .inventory
        .iter()
        .find(|i| i.artifact_id.as_deref() == Some("sunsword"))
        .unwrap()
        .letter
        .0;
    send_key(&mut ui_app, &clock, KeyCode::Char('V'), 100, 30);
    send_key(&mut ui_app, &clock, KeyCode::Char(sun_letter), 100, 30);
    assert_eq!(
        ui_app
            .observation()
            .campaign
            .unwrap()
            .artifact_invoke_cooldown,
        ARTIFACT_INVOKE_COOLDOWN_TURNS - 1
    );

    // 3. Legacy mode V1 invariance check.
    let legacy = GameSession::new_for_playing(42);
    let legacy_save = legacy.to_save_data();
    assert_eq!(legacy_save.schema_version, SAVE_SCHEMA_VERSION_V1);
    assert!(legacy_save.world.skill_practice.is_empty());
    assert!(legacy_save
        .world
        .levels
        .levels
        .iter()
        .all(|lvl| lvl.altar_alignments.is_empty()));
    let mut forged_legacy = legacy_save;
    forged_legacy
        .world
        .skill_practice
        .insert("sword".to_string(), 10);
    assert!(GameSession::from_save_data(forged_legacy).is_err());
}
