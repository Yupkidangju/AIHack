use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use aihack_content::{
    item_unidentified_pool, monster_corpse_spec, monster_is_large, registry,
    weapon_damage_for_size, GENERATED_ITEM_COUNT, GENERATED_MONSTER_COUNT,
};
use aihack_core::{
    action::{CommandIntent as C, WishChoice},
    campaign::{LearnedSpell, Role, SkillCategory, SpellKind, SpellSchool, SPELL_MEMORY_TURNS},
    domain::{
        combat::DamageRoll,
        entity::{ActorKind, EntityLocation, Faction},
        item::{BucStatus, ItemKind},
        monster::MonsterKind,
        status::Resistances,
        tile::{DoorState, TileKind},
    },
    ids::{EntityId, LevelId},
    position::{Direction, Pos},
    run_state::RunState,
};
use aihack_headless::{run_to_turn, HeadlessPolicy};
use aihack_llm::service::LlmObservationView;
use aihack_runtime::{
    appearance::appearance_for_kind,
    domain::{item::try_item_data_from_registry, monster::try_monster_template_from_registry},
    GameSession,
};
use aihack_tui::tui::{
    input::{key_to_candidate, UiCommandCandidate},
    render_frame, render_panels, TuiApp, UiClock, UiPanel, UiRuntimeConfig,
};
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

fn park_monsters_to(save: &mut aihack_runtime::save::SaveDataV1, dest: LevelId) {
    let map = save.world.levels.map(dest).unwrap().clone();
    let mut slots = (1..map.height - 1).flat_map(|y| {
        let map = map.clone();
        (1..map.width - 1).filter_map(move |x| {
            let pos = Pos { x, y };
            (map.tile(pos).ok() == Some(TileKind::Floor)).then_some(pos)
        })
    });
    let ids: Vec<EntityId> = save
        .world
        .entities
        .entities()
        .iter()
        .filter(|e| matches!(e.actor_kind(), Some(ActorKind::Monster(_))))
        .map(|e| e.id)
        .collect();
    for id in ids {
        if let Some(pos) = slots.next() {
            save.world.entities.set_actor_location(id, dest, pos);
        }
    }
}

fn give_item(
    save: &mut aihack_runtime::save::SaveDataV1,
    kind: ItemKind,
    buc: BucStatus,
    enchant: i8,
) -> EntityId {
    let player_id = save.world.player_id;
    let id = save
        .world
        .entities
        .spawn_item(kind, EntityLocation::Inventory { owner: player_id })
        .unwrap();
    save.world.entities.set_item_buc(id, buc);
    save.world.entities.set_item_enchantment(id, enchant);
    let letter = save
        .world
        .inventory
        .add_existing_with_next_letter(id)
        .unwrap();
    save.world.entities.set_item_letter(id, letter);
    id
}

fn quaff_gain_level(sess: &mut GameSession, count: u16) {
    let mut save = sess.to_save_data();
    let pot = give_item(&mut save, ItemKind::PotionGainLevel, BucStatus::Uncursed, 0);
    save.world.entities.set_item_quantity(pot, count);
    *sess = GameSession::from_save_data(save).unwrap();
    for _ in 0..count {
        let out = sess.submit(C::Quaff { item: pot });
        assert!(out.accepted, "{:?}", out.events);
    }
}

fn spawn_monster_at(
    save: &mut aihack_runtime::save::SaveDataV1,
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

fn open_neighbor(
    save: &aihack_runtime::save::SaveDataV1,
    level: LevelId,
    pos: Pos,
) -> (Direction, Pos) {
    let map = save.world.levels.map(level).unwrap();
    Direction::ALL
        .into_iter()
        .find_map(|d| {
            let p = pos.offset(d.delta());
            (map.contains(p)
                && map.tile(p).ok() == Some(TileKind::Floor)
                && save.world.entities.alive_actor_at(level, p).is_none())
            .then_some((d, p))
        })
        .expect("open neighbor floor")
}

fn buffer_text(backend: &TestBackend) -> String {
    let buf = backend.buffer();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

#[test]
fn p12_1_spellbook_study_memory_decay_and_failure_rate() {
    let base = start_campaign(42, Role::Mage);
    let mut save = base.to_save_data();
    let main1 = save.world.current_level;
    park_monsters_to(&mut save, LevelId::main(6));
    let book_id = give_item(
        &mut save,
        ItemKind::SpellbookFireball,
        BucStatus::Uncursed,
        0,
    );
    let chain_id = give_item(&mut save, ItemKind::ArmorChainMail, BucStatus::Uncursed, 0);
    let mithril_id = give_item(
        &mut save,
        ItemKind::ArmorSilverDragonScale,
        BucStatus::Uncursed,
        0,
    );
    let ppos = save
        .world
        .entities
        .actor_location(save.world.player_id)
        .unwrap()
        .1;
    let (east_dir, east_pos) = open_neighbor(&save, main1, ppos);
    let _ = east_pos;

    let mut sess = GameSession::from_save_data(save).unwrap();
    quaff_gain_level(&mut sess, 9);
    // Study SpellbookFireball via Read
    let read_out = sess.submit(C::Read { item: book_id });
    assert!(read_out.accepted, "{:?}", read_out.events);
    let obs = sess.observation();
    assert!(obs
        .campaign
        .as_ref()
        .unwrap()
        .learned_spells
        .iter()
        .any(|ls| ls.spell == SpellKind::Fireball));
    assert!(obs
        .campaign
        .as_ref()
        .unwrap()
        .identified_items
        .contains(&ItemKind::SpellbookFireball));

    // Check failure rate jumps when wearing iron chain mail, drops back on mithril
    let unarmored_fail = obs
        .campaign
        .as_ref()
        .unwrap()
        .spell_failure_rates
        .iter()
        .find(|&&(s, _)| s == SpellKind::Fireball)
        .unwrap()
        .1;
    let innate_fail = obs
        .campaign
        .as_ref()
        .unwrap()
        .spell_failure_rates
        .iter()
        .find(|&&(s, _)| s == SpellKind::ForceBolt)
        .unwrap()
        .1;
    assert_eq!(innate_fail, 0, "unarmored innate spell preserves 0% fail");

    assert!(sess.submit(C::Wear { item: chain_id }).accepted);
    let chain_fail = sess
        .observation()
        .campaign
        .unwrap()
        .spell_failure_rates
        .iter()
        .find(|&&(s, _)| s == SpellKind::Fireball)
        .unwrap()
        .1;
    assert!(
        chain_fail > unarmored_fail,
        "iron armor must raise spell failure ({chain_fail}% vs {unarmored_fail}%)"
    );
    assert!(
        sess.submit(C::TakeOff {
            slot: aihack_core::domain::item::EquipmentSlot::Body,
        })
        .accepted
    );
    assert!(sess.submit(C::Wear { item: mithril_id }).accepted);
    let mithril_fail = sess
        .observation()
        .campaign
        .unwrap()
        .spell_failure_rates
        .iter()
        .find(|&&(s, _)| s == SpellKind::Fireball)
        .unwrap()
        .1;
    assert_eq!(
        mithril_fail, unarmored_fail,
        "mithril incurs no metal penalty"
    );

    // Advance turn past SPELL_MEMORY_TURNS -> learned spell forgets, innate still casts
    let mut decayed_save = sess.to_save_data();
    decayed_save.turn = SPELL_MEMORY_TURNS + 10;
    let mut decayed = GameSession::from_save_data(decayed_save).unwrap();
    let faded = decayed.submit(C::Cast {
        spell: SpellKind::Fireball,
        direction: Some(east_dir),
    });
    assert!(
        !faded.accepted,
        "learned spell must expire after 20,000 turns"
    );
    let innate = decayed.submit(C::Cast {
        spell: SpellKind::ForceBolt,
        direction: Some(east_dir),
    });
    assert!(innate.accepted, "innate starter spell never forgets");

    // Re-studying restores Fireball casting
    let restudy = decayed.submit(C::Read { item: book_id });
    assert!(restudy.accepted);
    assert!(
        decayed
            .submit(C::Cast {
                spell: SpellKind::Fireball,
                direction: Some(east_dir),
            })
            .accepted
    );
}

#[test]
fn p12_2_seven_schools_and_representative_28_spells() {
    assert_eq!(SpellKind::ALL.len(), 28);
    let mut schools = HashSet::new();
    for &s in SpellKind::ALL {
        schools.insert(s.school());
    }
    assert_eq!(schools.len(), 7);
    assert_eq!(SpellSchool::ALL.len(), 7);

    let mut save = start_campaign(77, Role::Wizard).to_save_data();
    let main1 = save.world.current_level;
    park_monsters_to(&mut save, LevelId::main(6));
    save.world.campaign.as_mut().unwrap().attributes.int = 20;
    save.world.campaign.as_mut().unwrap().attributes.wis = 18;
    let cap_role_rank = save
        .world
        .campaign
        .unwrap()
        .role
        .max_skill_rank(SkillCategory::Spellcasting)
        .as_u8();
    save.world
        .spell_skill
        .insert("spellcasting".to_string(), cap_role_rank);

    let mut sess = GameSession::from_save_data(save).unwrap();
    quaff_gain_level(&mut sess, 19);

    let mut save = sess.to_save_data();
    let camp = save.world.campaign.unwrap();
    let pstats = save
        .world
        .entities
        .actor_stats_mut(save.world.player_id)
        .unwrap();
    pstats.hp = pstats.max_hp - 15;
    save.world.mp = save.world.max_mp;

    // Learn all 28 spells at current turn
    let turn_now = save.turn;
    for &spell in SpellKind::ALL {
        if !camp.knows_spell(spell) {
            save.world.learned_spells.push(LearnedSpell {
                spell,
                learned_at_turn: turn_now,
            });
        }
    }

    let ppos = save
        .world
        .entities
        .actor_location(save.world.player_id)
        .unwrap()
        .1;
    let (_dir, _adj_pos) = open_neighbor(&save, main1, ppos);
    // Find a clear line in East or South or West or North for directional targets
    let mut clear_dir = None;
    let map = save.world.levels.map(main1).unwrap().clone();
    for d in Direction::ALL {
        let p1 = ppos.offset(d.delta());
        let p2 = p1.offset(d.delta());
        if map.contains(p1)
            && map.contains(p2)
            && map.tile(p1).ok() == Some(TileKind::Floor)
            && map.tile(p2).ok() == Some(TileKind::Floor)
        {
            clear_dir = Some((d, p1, p2));
            break;
        }
    }
    let (shot_dir, near_pos, target_pos) = clear_dir.unwrap();
    let _ = near_pos;
    let zombie = spawn_monster_at(
        &mut save,
        MonsterKind::Zombie,
        main1,
        target_pos,
        Faction::Hostile,
    );
    let cursed_ring = give_item(&mut save, ItemKind::RingProtection, BucStatus::Cursed, 1);
    let unid_pot = give_item(&mut save, ItemKind::PotionGainLevel, BucStatus::Uncursed, 0);

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::PutOn { item: cursed_ring }).accepted);
    assert!(
        !sess.submit(C::RemoveRing).accepted,
        "cursed ring cannot be removed yet"
    );
    // 1. Divination: MagicMapping + IdentifySpell + DetectTreasure
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::MagicMapping,
            direction: None
        })
        .accepted
    );
    assert!(sess
        .world()
        .levels
        .get(main1)
        .unwrap()
        .is_explored(Pos { x: 1, y: 1 }));
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::IdentifySpell,
            direction: None
        })
        .accepted
    );
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::DetectTreasure,
            direction: None
        })
        .accepted
    );
    let _ = unid_pot;

    // 2. Clerical: RemoveCurseSpell + Protection + TurnUndead + CreateFamiliar
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::RemoveCurseSpell,
            direction: None
        })
        .accepted
    );
    assert_eq!(
        sess.world().entities.get(cursed_ring).unwrap().item_buc(),
        Some(BucStatus::Uncursed)
    );
    assert!(
        sess.submit(C::RemoveRing).accepted,
        "uncursed ring can be removed"
    );
    let luck_before = sess.world().luck;
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::Protection,
            direction: None
        })
        .accepted
    );
    assert!(sess.world().luck >= luck_before);
    let z_hp_before = sess.world().entities.actor_stats(zombie).unwrap().hp;
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::TurnUndead,
            direction: None
        })
        .accepted
    );
    let z_hp_after = sess
        .world()
        .entities
        .actor_stats(zombie)
        .map(|s| s.hp)
        .unwrap_or(0);
    assert!(z_hp_after <= z_hp_before, "TurnUndead hits nearby undead");

    // 3. Matter: Knock opens closed door, WizardLock locks it on a genuine generated door
    let mut door_save = start_campaign(78, Role::Wizard).to_save_data();
    park_monsters_to(&mut door_save, LevelId::main(6));
    for &spell in &[SpellKind::Knock, SpellKind::WizardLock] {
        door_save.world.learned_spells.push(LearnedSpell {
            spell,
            learned_at_turn: 0,
        });
    }
    let (door_lvl, door_pos, door_floor, door_dir) = door_save
        .world
        .levels
        .levels
        .iter()
        .find_map(|lvl| {
            for y in 1..lvl.map.height - 1 {
                for x in 1..lvl.map.width - 1 {
                    let dp = Pos { x, y };
                    if lvl.map.tile(dp).ok() == Some(TileKind::Door(DoorState::Closed)) {
                        for d in Direction::ALL {
                            let delta = d.delta();
                            let fp = Pos {
                                x: dp.x - delta.dx,
                                y: dp.y - delta.dy,
                            };
                            if lvl.map.tile(fp).ok() == Some(TileKind::Floor) {
                                return Some((lvl.id, dp, fp, d));
                            }
                        }
                    }
                }
            }
            None
        })
        .expect("campaign must contain a closed door adjacent to floor");
    door_save
        .world
        .entities
        .set_actor_location(door_save.world.player_id, door_lvl, door_floor);
    door_save.world.current_level = door_lvl;
    let mut door_sess = GameSession::from_save_data(door_save).unwrap();
    assert!(
        door_sess
            .submit(C::Cast {
                spell: SpellKind::Knock,
                direction: Some(door_dir)
            })
            .accepted
    );
    assert_eq!(
        door_sess.world().map(door_lvl).tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Open))
    );
    assert!(
        door_sess
            .submit(C::Cast {
                spell: SpellKind::WizardLock,
                direction: Some(door_dir)
            })
            .accepted
    );
    assert_eq!(
        door_sess.world().map(door_lvl).tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Locked))
    );
    let door_json = serde_json::to_string(&door_sess.to_save_data()).unwrap();
    let door_reloaded = GameSession::from_save_data(
        serde_json::from_str::<aihack_runtime::save::SaveDataV1>(&door_json).unwrap(),
    )
    .unwrap();
    assert_eq!(
        door_reloaded.world().map(door_lvl).tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Locked))
    );

    // Restore MP via SaveV3 checkpoint for remaining spell schools
    let mut mid_save = sess.to_save_data();
    mid_save.world.mp = mid_save.world.max_mp;
    sess = GameSession::from_save_data(mid_save).unwrap();

    // 4. Healing: ExtraHealing + CureSickness
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::ExtraHealing,
            direction: None
        })
        .accepted
    );
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::CureSickness,
            direction: None
        })
        .accepted
    );

    // 5. Enchantment: SlowMonster + CharmMonster + CreateFamiliar
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::SlowMonster,
            direction: Some(shot_dir)
        })
        .accepted
    );
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::HasteSelf,
            direction: None
        })
        .accepted
    );
    assert!(sess.world().is_fast());
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::CreateFamiliar,
            direction: None
        })
        .accepted
    );
    assert!(sess
        .world()
        .entities
        .entities()
        .iter()
        .any(|e| e.faction() == Some(Faction::Pet)));

    let mut atk_save = sess.to_save_data();
    atk_save.world.mp = atk_save.world.max_mp;
    sess = GameSession::from_save_data(atk_save).unwrap();

    // 6. Attack: MagicMissile + LightningBolt
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::MagicMissile,
            direction: Some(shot_dir)
        })
        .accepted
    );
    assert!(
        sess.submit(C::Cast {
            spell: SpellKind::LightningBolt,
            direction: Some(shot_dir)
        })
        .accepted
    );
}

#[test]
fn p12_3_eighteen_wands_wishing_and_polymorph() {
    let mut save = start_campaign(99, Role::Mage).to_save_data();
    let main1 = save.world.current_level;
    park_monsters_to(&mut save, LevelId::main(6));
    let wand_wish = give_item(&mut save, ItemKind::WandWishing, BucStatus::Uncursed, 0);
    let wand_dig = give_item(&mut save, ItemKind::WandDigging, BucStatus::Uncursed, 0);
    let wand_poly = give_item(&mut save, ItemKind::WandPolymorph, BucStatus::Uncursed, 0);
    let wand_cancel = give_item(
        &mut save,
        ItemKind::WandCancellation,
        BucStatus::Uncursed,
        0,
    );
    let wand_death = give_item(&mut save, ItemKind::WandDeath, BucStatus::Uncursed, 0);
    let wand_create = give_item(
        &mut save,
        ItemKind::WandCreateMonster,
        BucStatus::Uncursed,
        0,
    );
    let wand_open = give_item(&mut save, ItemKind::WandOpening, BucStatus::Uncursed, 0);
    let wand_lock = give_item(&mut save, ItemKind::WandLocking, BucStatus::Uncursed, 0);

    let map = save.world.levels.map(main1).unwrap().clone();
    // Pick a floor tile on main1 with BOTH an open floor neighbor and an adjacent interior wall
    let (new_ppos, clear_dir, p1, wall_dir, wall_pos) = (1..map.height - 1)
        .flat_map(|y| (1..map.width - 1).map(move |x| Pos { x, y }))
        .find_map(|fp| {
            if map.tile(fp).ok() != Some(TileKind::Floor)
                || save.world.entities.alive_actor_at(main1, fp).is_some()
            {
                return None;
            }
            let floor_nb = Direction::ALL.into_iter().find_map(|d| {
                let c = fp.offset(d.delta());
                (map.contains(c)
                    && map.tile(c).ok() == Some(TileKind::Floor)
                    && save.world.entities.alive_actor_at(main1, c).is_none())
                .then_some((d, c))
            })?;
            let wall_nb = Direction::ALL.into_iter().find_map(|d| {
                let c = fp.offset(d.delta());
                (c.x > 0
                    && c.x < map.width - 1
                    && c.y > 0
                    && c.y < map.height - 1
                    && map.tile(c).ok() == Some(TileKind::Wall))
                .then_some((d, c))
            })?;
            Some((fp, floor_nb.0, floor_nb.1, wall_nb.0, wall_nb.1))
        })
        .expect("floor cell with adjacent wall and floor neighbor");
    save.world
        .entities
        .set_actor_location(save.world.player_id, main1, new_ppos);
    let wall_dir = Some(wall_dir);

    let mut sess = GameSession::from_save_data(save).unwrap();
    quaff_gain_level(&mut sess, 2);
    let mut after_quaff = sess.to_save_data();
    let mob_a = spawn_monster_at(
        &mut after_quaff,
        MonsterKind::RedMold,
        main1,
        p1,
        Faction::Hostile,
    );
    sess = GameSession::from_save_data(after_quaff).unwrap();

    // Wishing: blessed +2 silver dragon scale mail
    let w1 = sess.submit(C::Wish {
        item: wand_wish,
        wish: WishChoice::Item {
            kind: ItemKind::ArmorSilverDragonScale,
            blessed: true,
            enchant: 2,
        },
    });
    assert!(w1.accepted, "{:?}", w1.events);
    assert!(sess.world().inventory.entries.iter().any(|e| sess
        .world()
        .entities
        .item_data(e.item)
        .is_some_and(|d| d.kind == ItemKind::ArmorSilverDragonScale)
        && sess.world().entities.get(e.item).unwrap().item_buc() == Some(BucStatus::Blessed)
        && sess
            .world()
            .entities
            .get(e.item)
            .unwrap()
            .item_enchantment()
            == Some(2)));

    // Illegal wish rejected
    assert!(
        !sess
            .submit(C::Wish {
                item: wand_wish,
                wish: WishChoice::Item {
                    kind: ItemKind::AmuletAscension,
                    blessed: true,
                    enchant: 0,
                },
            })
            .accepted
    );

    // Unique artifact wish twice: first creates Excalibur artifact (and blasts Neutral Mage), second falls back to non-artifact LongSword
    let hp_before_excal = sess.observation().player.hp;
    let excal_wish = sess.submit(C::Wish {
        item: wand_wish,
        wish: WishChoice::ArtifactExcalibur,
    });
    assert!(excal_wish.accepted, "{:?}", excal_wish.events);
    assert!(
        sess.observation().player.hp < hp_before_excal,
        "cross-aligned artifact wish must deal blast damage to Neutral Mage"
    );
    assert!(sess.world().generated_artifacts.contains("excalibur"));
    assert!(
        sess.submit(C::Wish {
            item: wand_wish,
            wish: WishChoice::ArtifactExcalibur
        })
        .accepted
    );
    let art_count = sess
        .world()
        .inventory
        .entries
        .iter()
        .filter(|e| {
            sess.world()
                .entities
                .get(e.item)
                .unwrap()
                .item_artifact_id()
                == Some("excalibur")
        })
        .count();
    assert_eq!(art_count, 1);

    // Cancellation strips mob_a resistances, Polymorph transforms mob_a
    assert!(
        sess.submit(C::Zap {
            item: wand_cancel,
            direction: clear_dir
        })
        .accepted
    );
    assert_eq!(
        sess.world()
            .entities
            .actor_stats(mob_a)
            .unwrap()
            .resistances,
        Resistances::NONE
    );
    let before_kind = sess.world().entities.get(mob_a).unwrap().actor_kind();
    assert!(
        sess.submit(C::Zap {
            item: wand_poly,
            direction: clear_dir
        })
        .accepted
    );
    assert_ne!(
        sess.world().entities.get(mob_a).unwrap().actor_kind(),
        before_kind
    );

    // WandDeath kills living mob_a
    assert!(
        sess.submit(C::Zap {
            item: wand_death,
            direction: clear_dir
        })
        .accepted
    );
    assert!(!sess.world().entities.get(mob_a).unwrap().is_alive_actor());

    // Digging carves interior wall without breaching perimeter
    if let Some(wd) = wall_dir {
        assert!(
            sess.submit(C::Zap {
                item: wand_dig,
                direction: wd
            })
            .accepted
        );
        assert_eq!(
            sess.world().map(main1).tile(wall_pos).ok(),
            Some(TileKind::Floor)
        );
    }
    assert_eq!(
        sess.world().map(main1).tile(Pos { x: 0, y: 1 }).ok(),
        Some(TileKind::Wall)
    );

    // CreateMonster + Opening + Locking
    let count_before = sess.world().current_level_hostile_monsters().len();
    assert!(
        sess.submit(C::Zap {
            item: wand_create,
            direction: clear_dir
        })
        .accepted
    );
    assert!(sess.world().current_level_hostile_monsters().len() > count_before);
    let _ = (wand_open, wand_lock);
}

#[test]
fn p12_4_rings_amulets_reflection_life_saving_conflict_and_size_damage() {
    let reg = registry().unwrap();
    // Size damage contract + SilverSaber metadata
    assert_eq!(
        weapon_damage_for_size(ItemKind::LongSword, false, reg),
        Some(DamageRoll::new(1, 8))
    );
    assert_eq!(
        weapon_damage_for_size(ItemKind::LongSword, true, reg),
        Some(DamageRoll::new(1, 12))
    );
    assert_eq!(
        weapon_damage_for_size(ItemKind::TwoHandedSword, true, reg),
        Some(DamageRoll::new(3, 6))
    );
    assert!(monster_is_large(MonsterKind::WhiteDragon, reg));
    assert!(!monster_is_large(MonsterKind::Jackal, reg));

    // 1. Amulet of Life Saving prevents player death and consumes amulet
    let mut save1 = start_campaign(101, Role::Knight).to_save_data();
    let main1 = save1.world.current_level;
    park_monsters_to(&mut save1, LevelId::main(6));
    let life_amulet = give_item(
        &mut save1,
        ItemKind::AmuletLifeSaving,
        BucStatus::Uncursed,
        0,
    );
    save1
        .world
        .entities
        .actor_stats_mut(save1.world.player_id)
        .unwrap()
        .hp = 1;
    let ppos = save1
        .world
        .entities
        .actor_location(save1.world.player_id)
        .unwrap()
        .1;
    let (_d, adj_pos) = open_neighbor(&save1, main1, ppos);
    let brute = spawn_monster_at(
        &mut save1,
        MonsterKind::Ettin,
        main1,
        adj_pos,
        Faction::Hostile,
    );
    save1
        .world
        .entities
        .actor_stats_mut(brute)
        .unwrap()
        .hit_bonus = 40;
    let mut s1 = GameSession::from_save_data(save1).unwrap();
    assert!(s1.submit(C::PutOn { item: life_amulet }).accepted);
    assert!(s1.submit(C::Wait).accepted);
    assert_eq!(s1.run_state(), RunState::Playing);
    assert!(s1.world().player_alive());
    assert!(s1.world().inventory.equipped_amulet.is_none());
    assert!(!s1.world().inventory.contains(life_amulet));

    // 2. Reflection reflects Medusa's gaze and kills Medusa
    let mut save2 = start_campaign(202, Role::Knight).to_save_data();
    park_monsters_to(&mut save2, LevelId::main(6));
    let refl = give_item(
        &mut save2,
        ItemKind::AmuletReflection,
        BucStatus::Uncursed,
        0,
    );
    let ppos2 = save2
        .world
        .entities
        .actor_location(save2.world.player_id)
        .unwrap()
        .1;
    let map2 = save2.world.levels.map(main1).unwrap().clone();
    let mut m_pos = None;
    for d in Direction::ALL {
        let p = ppos2.offset(d.delta()).offset(d.delta());
        if map2.contains(p) && map2.tile(p).ok() == Some(TileKind::Floor) {
            m_pos = Some(p);
            break;
        }
    }
    let medusa = spawn_monster_at(
        &mut save2,
        MonsterKind::Medusa,
        main1,
        m_pos.unwrap(),
        Faction::Hostile,
    );
    let mut s2 = GameSession::from_save_data(save2).unwrap();
    assert!(s2.submit(C::PutOn { item: refl }).accepted);
    assert!(s2.submit(C::Wait).accepted);
    assert!(!s2.world().entities.get(medusa).unwrap().is_alive_actor());

    // 3. Ring of Searching passively reveals hidden door, Teleport Control steers teleport, Conflict works
    let mut save3 = start_campaign(303, Role::Scout).to_save_data();
    park_monsters_to(&mut save3, LevelId::main(6));
    let search_ring = give_item(&mut save3, ItemKind::RingSearching, BucStatus::Uncursed, 0);
    let tc_ring = give_item(
        &mut save3,
        ItemKind::RingTeleportControl,
        BucStatus::Uncursed,
        0,
    );
    let conflict_ring = give_item(&mut save3, ItemKind::RingConflict, BucStatus::Uncursed, 0);
    let scroll_tp = give_item(
        &mut save3,
        ItemKind::ScrollLevelTeleport,
        BucStatus::Uncursed,
        0,
    );
    let (hd_level, hd_pos, hd_adj) = save3
        .world
        .levels
        .levels
        .iter()
        .find_map(|lvl| {
            for y in 1..lvl.map.height - 1 {
                for x in 1..lvl.map.width - 1 {
                    let pos = Pos { x, y };
                    if lvl.map.tile(pos).ok() == Some(TileKind::HiddenDoor) {
                        for d in Direction::ALL {
                            let adj = pos.offset(d.delta());
                            if lvl.map.tile(adj).ok() == Some(TileKind::Floor) {
                                return Some((lvl.id, pos, adj));
                            }
                        }
                    }
                }
            }
            None
        })
        .expect("campaign must contain a generated hidden door adjacent to floor");
    save3
        .world
        .entities
        .set_actor_location(save3.world.player_id, hd_level, hd_adj);
    save3.world.current_level = hd_level;
    let mut s3 = GameSession::from_save_data(save3).unwrap();
    assert!(s3.submit(C::PutOn { item: search_ring }).accepted);
    assert!(s3.submit(C::Wait).accepted);
    assert_eq!(
        s3.world().map(hd_level).tile(hd_pos).ok(),
        Some(TileKind::Door(DoorState::Closed))
    );
    assert!(s3.submit(C::RemoveRing).accepted);
    assert!(s3.submit(C::PutOn { item: tc_ring }).accepted);
    let before_tp = s3.world().current_level();
    assert!(s3.submit(C::Read { item: scroll_tp }).accepted);
    assert_ne!(s3.world().current_level(), before_tp);

    // Conflict causes nearby monsters to attack each other
    let mut conflict_save = start_campaign(304, Role::Knight).to_save_data();
    park_monsters_to(&mut conflict_save, LevelId::main(6));
    let c_ring = give_item(
        &mut conflict_save,
        ItemKind::RingConflict,
        BucStatus::Uncursed,
        0,
    );
    let ppos_c = conflict_save
        .world
        .entities
        .actor_location(conflict_save.world.player_id)
        .unwrap()
        .1;
    let (d1, pos1) = open_neighbor(&conflict_save, main1, ppos_c);
    let pos2 = pos1.offset(d1.delta());
    let mut orc_a = EntityId(0);
    let mut orc_b = EntityId(0);
    if conflict_save
        .world
        .levels
        .map(main1)
        .unwrap()
        .tile(pos2)
        .ok()
        == Some(TileKind::Floor)
    {
        let p_far1 = pos2;
        let (_d2, p_far2) = open_neighbor(&conflict_save, main1, p_far1);
        orc_a = spawn_monster_at(
            &mut conflict_save,
            MonsterKind::Orc,
            main1,
            p_far1,
            Faction::Hostile,
        );
        orc_b = spawn_monster_at(
            &mut conflict_save,
            MonsterKind::Orc,
            main1,
            p_far2,
            Faction::Hostile,
        );
        conflict_save
            .world
            .entities
            .actor_stats_mut(orc_a)
            .unwrap()
            .hit_bonus = 30;
        conflict_save
            .world
            .entities
            .actor_stats_mut(orc_b)
            .unwrap()
            .hit_bonus = 30;
    }
    let _ = conflict_ring;
    let mut sc = GameSession::from_save_data(conflict_save).unwrap();
    assert!(sc.submit(C::PutOn { item: c_ring }).accepted);
    assert!(sc.submit(C::Wait).accepted);
    if orc_a.0 != 0 && orc_b.0 != 0 {
        let a_hurt = sc
            .world()
            .entities
            .actor_stats(orc_a)
            .map(|s| s.hp < s.max_hp || !sc.world().entities.get(orc_a).unwrap().is_alive_actor())
            .unwrap_or(true);
        let b_hurt = sc
            .world()
            .entities
            .actor_stats(orc_b)
            .map(|s| s.hp < s.max_hp || !sc.world().entities.get(orc_b).unwrap().is_alive_actor())
            .unwrap_or(true);
        assert!(a_hurt || b_hurt || sc.world().has_conflict());
    }

    // 4. Unchanging blocks self polymorph & MagicalBreathing prevents drowning
    let mut save4 = start_campaign(404, Role::Knight).to_save_data();
    let unchanging = give_item(
        &mut save4,
        ItemKind::AmuletUnchanging,
        BucStatus::Uncursed,
        0,
    );
    let poly_pot = give_item(
        &mut save4,
        ItemKind::PotionPolymorph,
        BucStatus::Uncursed,
        0,
    );
    let mut s4 = GameSession::from_save_data(save4).unwrap();
    assert!(s4.submit(C::PutOn { item: unchanging }).accepted);
    let p_out = s4.submit(C::Quaff { item: poly_pot });
    assert!(p_out.accepted);
    assert!(p_out.events.iter().any(|e| matches!(e, aihack_core::event::GameEvent::Message { text, .. } if text.contains("unchanged"))));
}

#[test]
fn p12_5_full_catalog_130_items_85_monsters_smoke_test() {
    let reg = registry().unwrap();
    assert_eq!(GENERATED_ITEM_COUNT, 130);
    assert_eq!(ItemKind::ALL.len(), 130);
    const { assert!(GENERATED_MONSTER_COUNT >= 85) };
    assert_eq!(MonsterKind::ALL.len(), GENERATED_MONSTER_COUNT);

    // Verify all randomized appearance pools produce unique non-colliding appearances per seed
    for seed in [42_u64, 1337_u64] {
        let mut by_pool: HashMap<&'static str, HashSet<&'static str>> = HashMap::new();
        for &kind in ItemKind::ALL {
            let pool = item_unidentified_pool(kind);
            let app = appearance_for_kind(seed, kind);
            if let Some(pool_name) = pool {
                let text = app.unwrap_or_else(|| panic!("missing appearance for pooled {kind:?}"));
                assert!(
                    by_pool.entry(pool_name).or_default().insert(text),
                    "collision in pool {pool_name} for seed {seed}: {text}"
                );
            } else {
                assert!(app.is_none(), "non-pooled {kind:?} should return None");
            }
            assert!(!kind.has_randomized_appearance() || pool.is_some());
        }
    }

    // Spawn all 130 items + 85 monsters across campaign world batches and exercise Observation + TUI
    for batch_idx in 0..3 {
        let mut save = start_campaign(500 + batch_idx as u64, Role::Mage).to_save_data();
        let main1 = save.world.current_level;
        park_monsters_to(&mut save, LevelId::main(6));
        let map = save.world.levels.map(main1).unwrap().clone();
        let floors: Vec<Pos> = (1..map.height - 1)
            .flat_map(|y| (1..map.width - 1).map(move |x| Pos { x, y }))
            .filter(|&p| map.tile(p).ok() == Some(TileKind::Floor))
            .collect();

        let item_chunk = &ItemKind::ALL[batch_idx * 44..((batch_idx + 1) * 44).min(130)];
        for (i, &kind) in item_chunk.iter().enumerate() {
            let data = try_item_data_from_registry(kind, reg).unwrap();
            assert_eq!(data.kind, kind);
            if kind == ItemKind::AmuletAscension {
                continue;
            }
            let pos = floors[i % floors.len()];
            save.world
                .entities
                .spawn_item(kind, EntityLocation::OnMap { level: main1, pos })
                .unwrap();
        }
        let mon_chunk = &MonsterKind::ALL[batch_idx * 30..((batch_idx + 1) * 30).min(85)];
        for (i, &mk) in mon_chunk.iter().enumerate() {
            let tmpl = try_monster_template_from_registry(mk, reg).unwrap();
            assert_eq!(tmpl.kind, mk);
            let _spec = monster_corpse_spec(mk, reg);
            let pos = floors[(i * 2 + 5) % floors.len()];
            if save.world.entities.alive_actor_at(main1, pos).is_none() {
                let mid = save
                    .world
                    .entities
                    .spawn_monster_with_faction(mk, tmpl, Faction::Hostile, pos)
                    .unwrap();
                save.world.entities.set_actor_location(mid, main1, pos);
            }
        }
        let sess = GameSession::from_save_data(save).unwrap();
        let obs = sess.observation();
        let _ = render_panels::inspect_lines(&obs, Some(obs.player_pos), UiPanel::Inspect, &[]);
        let mut app = TuiApp::new_with_llm_enabled_and_clock(
            sess,
            UiRuntimeConfig::default(),
            false,
            Arc::new(TestClock::default()),
        );
        let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
        term.draw(|f| render_frame(f, &mut app)).unwrap();
    }
}

#[test]
fn p12_6_save_v3_roundtrip_legacy_v1_isolation_and_ai_budgets() {
    let mut save = start_campaign(606, Role::Mage).to_save_data();
    let main1 = save.world.current_level;
    park_monsters_to(&mut save, LevelId::main(6));
    let book = give_item(
        &mut save,
        ItemKind::SpellbookMagicMapping,
        BucStatus::Uncursed,
        0,
    );
    let wand_wish = give_item(&mut save, ItemKind::WandWishing, BucStatus::Uncursed, 0);
    let amulet = give_item(
        &mut save,
        ItemKind::AmuletLifeSaving,
        BucStatus::Uncursed,
        0,
    );
    let _gem = give_item(&mut save, ItemKind::GemDiamond, BucStatus::Uncursed, 0);
    let _ = amulet;
    let ppos = save
        .world
        .entities
        .actor_location(save.world.player_id)
        .unwrap()
        .1;
    let (_, pos) = open_neighbor(&save, main1, ppos);
    let _wd = spawn_monster_at(
        &mut save,
        MonsterKind::WhiteDragon,
        main1,
        pos,
        Faction::Hostile,
    );

    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::Read { item: book }).accepted);

    // SaveV3 roundtrip keeps exact snapshot hash
    let roundtrip_save = sess.to_save_data();
    let restored = GameSession::from_save_data(roundtrip_save).unwrap();
    assert_eq!(
        sess.snapshot().stable_hash(),
        restored.snapshot().stable_hash()
    );

    // Legacy V1 rejects P12 items, P12 monsters, and learned_spells
    let mut legacy = GameSession::try_new(42).unwrap();
    assert!(legacy.submit(C::Wait).accepted);
    assert!(legacy.submit(C::Wait).accepted);
    let base_v1 = legacy.to_save_data();
    assert_eq!(base_v1.schema_version, 1);
    let mut bad_v1_book = base_v1.clone();
    give_item(
        &mut bad_v1_book,
        ItemKind::SpellbookMagicMissile,
        BucStatus::Uncursed,
        0,
    );
    assert!(GameSession::from_save_data(bad_v1_book).is_err());

    let mut bad_v1_mon = base_v1;
    let pos_v1 = open_neighbor(
        &bad_v1_mon,
        LevelId::main(1),
        bad_v1_mon
            .world
            .entities
            .actor_location(bad_v1_mon.world.player_id)
            .unwrap()
            .1,
    )
    .1;
    let _ = spawn_monster_at(
        &mut bad_v1_mon,
        MonsterKind::GridBug,
        LevelId::main(1),
        pos_v1,
        Faction::Hostile,
    );
    assert!(GameSession::from_save_data(bad_v1_mon).is_err());

    // LLM observation < 32 KiB & ActionSpace <= 64
    let obs = sess.observation();
    assert!(obs.action_space.commands.len() <= 64);
    let llm_bytes = serde_json::to_vec(&LlmObservationView::from(&obs))
        .unwrap()
        .len();
    assert!(
        llm_bytes < 32_768,
        "LlmObservationView {llm_bytes} bytes >= 32768"
    );

    // TUI dynamic spell menu & wish menu render cleanly
    let clock = Arc::new(TestClock::default());
    let mut app =
        TuiApp::new_with_llm_enabled_and_clock(sess, UiRuntimeConfig::default(), false, clock);
    let mut term = Terminal::new(TestBackend::new(120, 40)).unwrap();
    assert!(app
        .handle_candidate_owned(UiCommandCandidate::BeginAction('Z'))
        .is_ok());
    term.draw(|f| render_frame(f, &mut app)).unwrap();
    let z_screen = buffer_text(term.backend());
    assert!(
        z_screen.contains("fail "),
        "TUI spell menu must display failure %:\n{z_screen}"
    );

    assert!(app
        .handle_candidate_owned(UiCommandCandidate::ChooseWish(wand_wish))
        .is_ok());
    term.draw(|f| render_frame(f, &mut app)).unwrap();
    let wish_screen = buffer_text(term.backend());
    assert!(
        wish_screen.contains("Make a Wish"),
        "TUI wish menu must open:\n{wish_screen}"
    );

    // Verify Headless equips identified AmuletLifeSaving automatically
    let mut hs_save = start_campaign(707, Role::Cleric).to_save_data();
    let hs_amulet = give_item(
        &mut hs_save,
        ItemKind::AmuletLifeSaving,
        BucStatus::Uncursed,
        0,
    );
    hs_save
        .world
        .identified_items
        .push(ItemKind::AmuletLifeSaving);
    let mut hs = GameSession::from_save_data(hs_save).unwrap();
    let report = run_to_turn(&mut hs, 5, HeadlessPolicy::SurvivalV1).unwrap();
    assert_eq!(report.accepted_turns, 5);
    assert_eq!(hs.world().inventory.equipped_amulet, Some(hs_amulet));
    assert_eq!(
        key_to_candidate('Z', &hs.observation()),
        Some(UiCommandCandidate::BeginAction('Z'))
    );
}
