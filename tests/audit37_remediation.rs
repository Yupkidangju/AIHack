use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use aihack_core::{
    action::{CommandIntent as C, WishChoice},
    campaign::Role,
    domain::{
        entity::{ActorKind, EntityLocation},
        item::{BucStatus, ItemClass, ItemKind},
        monster::MonsterKind,
        tile::{TileKind, TrapKind},
    },
    event::GameEvent,
    ids::{BranchId, EntityId, LevelId},
    position::{Direction, Pos},
    run_state::RunState,
    save::{
        SAVE_SCHEMA_VERSION_V1, SAVE_SCHEMA_VERSION_V2, SAVE_SCHEMA_VERSION_V3,
        SAVE_SCHEMA_VERSION_V4,
    },
};
use aihack_runtime::{
    save::{ArtifactStore, SaveDataV1},
    GameSession,
};

fn start_campaign(seed: u64, role: Role) -> GameSession {
    let mut session = GameSession::try_new(seed).unwrap();
    assert!(session.submit(C::Wait).accepted);
    let outcome = session.submit(C::StartCampaign { role });
    assert!(outcome.accepted, "{:?}", outcome.events);
    session
}

fn temp_store_dir(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let root = std::env::temp_dir().join(format!("aihack_{tag}_{}_{nanos}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}

fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0_u32; 64];
        for (i, b) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        let next = [a, b, c, d, e, f, g, hh];
        for (hi, vi) in h.iter_mut().zip(next) {
            *hi = hi.wrapping_add(vi);
        }
    }
    let mut out = String::with_capacity(64);
    for word in h {
        out.push_str(&format!("{word:08x}"));
    }
    out
}

fn find_unpaid_on_level(
    save: &SaveDataV1,
    level: LevelId,
    kind_filter: Option<ItemKind>,
) -> (EntityId, Pos) {
    save.world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            let (kind, _, loc, _, _) = e.item()?;
            if e.item_unpaid() != Some(true) {
                return None;
            }
            if kind_filter.is_some_and(|want| want != kind) {
                return None;
            }
            match loc {
                EntityLocation::OnMap { level: l, pos } if l == level => Some((e.id, pos)),
                _ => None,
            }
        })
        .unwrap_or_else(|| panic!("missing unpaid item on {level:?}"))
}

fn teleport_player(save: &mut SaveDataV1, level: LevelId, pos: Pos) {
    save.world.current_level = level;
    save.world
        .entities
        .set_actor_location(save.world.player_id, level, pos);
}

fn add_inventory_item(
    save: &mut SaveDataV1,
    kind: ItemKind,
    buc: BucStatus,
    enchant: i8,
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
    save.world.entities.set_item_buc_known(id, true);
    save.world.entities.set_item_enchant_known(id, true);
    id
}

#[test]
fn a37_01_unpaid_item_consumption_save_and_partial_stack() {
    let store_dir = temp_store_dir("a37_01");
    let store = ArtifactStore::open(&store_dir).unwrap();
    let mines1 = LevelId::new(BranchId::Mines, 1);

    // 1. Exact Audit 37 reproduction: pick up unpaid Mines 1 ScrollEnchantArmor, read last unit, save & reload.
    let mut save = start_campaign(42, Role::Knight).to_save_data();
    save.world.gold = 500;
    let (scroll_id, scroll_pos) =
        find_unpaid_on_level(&save, mines1, Some(ItemKind::ScrollEnchantArmor));
    teleport_player(&mut save, mines1, scroll_pos);
    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::Pickup).accepted);
    let debt = sess.world().shop_debt;
    assert!(debt > 0);
    assert_eq!(sess.world().shop_debts, vec![(mines1, debt)]);
    store
        .save_session(&sess, Path::new("before_read.json"))
        .unwrap();

    let read_out = sess.submit(C::Read { item: scroll_id });
    assert!(read_out.accepted, "{:?}", read_out.events);
    assert!(read_out
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::ScrollRead { item, .. } if *item == scroll_id)));
    let consumed = sess.world().entities().get(scroll_id).unwrap();
    assert_eq!(consumed.item().unwrap().2, EntityLocation::Consumed);
    assert_eq!(consumed.item_unpaid(), Some(false));
    assert_eq!(consumed.item_unpaid_level(), None);
    assert_eq!(sess.world().shop_debt, debt);
    assert_eq!(sess.world().shop_debts, vec![(mines1, debt)]);

    store
        .save_session(&sess, Path::new("after_read.json"))
        .unwrap();
    let mut reloaded = store.load_session(Path::new("after_read.json")).unwrap();
    assert_eq!(
        sess.snapshot().stable_hash(),
        reloaded.snapshot().stable_hash()
    );
    assert!(reloaded.submit(C::Pay).accepted);
    assert_eq!(reloaded.world().shop_debt, 0);
    assert!(reloaded.world().shop_debts.is_empty());

    // 2. Partial stack consumption (qty = 2 -> consume 1 -> qty = 1 remains unpaid -> drop refunds 1 -> pay remaining 1).
    let mut stack_save = start_campaign(42, Role::Knight).to_save_data();
    stack_save.world.gold = 500;
    let (stack_id, stack_pos) =
        find_unpaid_on_level(&stack_save, mines1, Some(ItemKind::ScrollEnchantArmor));
    stack_save.world.entities.set_item_quantity(stack_id, 2);
    teleport_player(&mut stack_save, mines1, stack_pos);
    let mut stack_sess = GameSession::from_save_data(stack_save).unwrap();
    assert!(stack_sess.submit(C::Pickup).accepted);
    let two_unit_debt = stack_sess.world().shop_debt;
    let unit_price = two_unit_debt / 2;
    assert_eq!(two_unit_debt, unit_price * 2);

    assert!(stack_sess.submit(C::Read { item: stack_id }).accepted);
    let rem = stack_sess.world().entities().get(stack_id).unwrap();
    assert_eq!(rem.item_quantity(), Some(1));
    assert_eq!(rem.item_unpaid(), Some(true));
    assert_eq!(stack_sess.world().shop_debt, two_unit_debt);
    store
        .save_session(&stack_sess, Path::new("partial_stack.json"))
        .unwrap();
    let mut partial_reloaded = store.load_session(Path::new("partial_stack.json")).unwrap();
    assert!(partial_reloaded.submit(C::Drop { item: stack_id }).accepted);
    assert_eq!(partial_reloaded.world().shop_debt, unit_price);
    assert_eq!(
        partial_reloaded.world().shop_debts,
        vec![(mines1, unit_price)]
    );
    assert!(partial_reloaded.submit(C::Pay).accepted);
    assert_eq!(partial_reloaded.world().shop_debt, 0);

    // 3. Negative control: forged Consumed item with unpaid=true must still be rejected.
    let mut forged = partial_reloaded.to_save_data();
    forged.world.entities.set_item_unpaid(stack_id, true);
    forged
        .world
        .entities
        .set_item_location(stack_id, EntityLocation::Consumed);
    assert!(GameSession::from_save_data(forged).is_err());

    let _ = fs::remove_dir_all(&store_dir);
}

#[test]
fn a37_02_shopkeeper_death_and_payment_scoped_per_shop_ledger() {
    let store_dir = temp_store_dir("a37_02");
    let store = ArtifactStore::open(&store_dir).unwrap();
    let main1 = LevelId::main(1);
    let main2 = LevelId::main(2);
    let mines1 = LevelId::new(BranchId::Mines, 1);

    // 1. Exact Audit 37 reproduction: Main 2 + Mines 1 debt -> consume Mines 1 scroll -> kill Main 2 shopkeeper -> Mines 1 debt preserved!
    let mut save = start_campaign(42, Role::Knight).to_save_data();
    save.world.gold = 500;
    let (m2_item, m2_pos) = find_unpaid_on_level(&save, main2, None);
    let (mi_scroll, mi_pos) =
        find_unpaid_on_level(&save, mines1, Some(ItemKind::ScrollEnchantArmor));

    teleport_player(&mut save, main2, m2_pos);
    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(sess.submit(C::Pickup).accepted);
    let main2_debt = sess.world().shop_debt;
    assert_eq!(sess.world().shop_debts, vec![(main2, main2_debt)]);

    let mut s2 = sess.to_save_data();
    teleport_player(&mut s2, mines1, mi_pos);
    sess = GameSession::from_save_data(s2).unwrap();
    assert!(sess.submit(C::Pickup).accepted);
    let total_debt = sess.world().shop_debt;
    let mines1_debt = total_debt - main2_debt;
    assert_eq!(
        sess.world().shop_debts,
        vec![(main2, main2_debt), (mines1, mines1_debt)]
    );

    assert!(sess.submit(C::Read { item: mi_scroll }).accepted);
    assert_eq!(sess.world().shop_debt, total_debt);

    // Kill ONLY Main 2 shopkeeper.
    let mut s3 = sess.to_save_data();
    let wand_death = add_inventory_item(&mut s3, ItemKind::WandDeath, BucStatus::Uncursed, 0);
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
    let sk_p = s3.world.entities.actor_location(sk_m2).unwrap().1;
    let map2 = s3.world.levels.map(main2).unwrap().clone();
    let sk_dir = Direction::ALL
        .into_iter()
        .find(|d| {
            let p = Pos {
                x: sk_p.x - d.delta().dx,
                y: sk_p.y - d.delta().dy,
            };
            map2.contains(p) && map2.tile(p).ok() == Some(TileKind::Floor)
        })
        .unwrap();
    teleport_player(
        &mut s3,
        main2,
        Pos {
            x: sk_p.x - sk_dir.delta().dx,
            y: sk_p.y - sk_dir.delta().dy,
        },
    );
    sess = GameSession::from_save_data(s3).unwrap();
    assert!(
        sess.submit(C::Zap {
            item: wand_death,
            direction: sk_dir
        })
        .accepted
    );
    assert!(!sess.world().entities().get(sk_m2).unwrap().is_alive_actor());
    assert_eq!(sess.world().shop_debt, mines1_debt);
    assert_eq!(sess.world().shop_debts, vec![(mines1, mines1_debt)]);
    assert_eq!(
        sess.world().entities().get(m2_item).unwrap().item_unpaid(),
        Some(false)
    );
    store
        .save_session(&sess, Path::new("after_sk_kill.json"))
        .unwrap();
    let loaded = store.load_session(Path::new("after_sk_kill.json")).unwrap();
    assert_eq!(loaded.world().shop_debt, mines1_debt);

    // 2. Scoped Pay on Main 2 clears ONLY Main 2's debt and does NOT touch Mines 1 debt; outside drop keeps debt without double-bill.
    let mut rev = start_campaign(42, Role::Knight).to_save_data();
    rev.world.gold = 500;
    let (_m2_item2, m2_pos2) = find_unpaid_on_level(&rev, main2, None);
    let (mi_item2, mi_pos2) =
        find_unpaid_on_level(&rev, mines1, Some(ItemKind::ScrollEnchantArmor));
    teleport_player(&mut rev, mines1, mi_pos2);
    let mut rs = GameSession::from_save_data(rev).unwrap();
    assert!(rs.submit(C::Pickup).accepted);
    let mi2_debt = rs.world().shop_debt;

    let mut r2 = rs.to_save_data();
    teleport_player(&mut r2, main2, m2_pos2);
    rs = GameSession::from_save_data(r2).unwrap();
    assert!(rs.submit(C::Pickup).accepted);
    let both_debt = rs.world().shop_debt;
    let m2_debt2 = both_debt - mi2_debt;
    assert!(m2_debt2 > 0);
    assert!(rs.submit(C::Pay).accepted);
    assert_eq!(rs.world().shop_debt, mi2_debt);
    assert_eq!(rs.world().shop_debts, vec![(mines1, mi2_debt)]);
    assert_eq!(
        rs.world().entities().get(mi_item2).unwrap().item_unpaid(),
        Some(true)
    );
    assert!(!rs.observation().legal_actions.contains(&C::Pay));

    // Dropping Mines 1 item outside shop (Main 1) leaves Mines 1 debt intact and avoids re-pickup double billing.
    let mut r3 = rs.to_save_data();
    let m1_start = r3.world.levels.stairs_up_pos(main1).unwrap();
    teleport_player(&mut r3, main1, m1_start);
    rs = GameSession::from_save_data(r3).unwrap();
    assert!(rs.submit(C::Drop { item: mi_item2 }).accepted);
    assert_eq!(rs.world().shop_debt, mi2_debt);
    assert_eq!(
        rs.world().entities().get(mi_item2).unwrap().item_unpaid(),
        Some(false)
    );
    assert!(rs.submit(C::Pickup).accepted);
    assert_eq!(rs.world().shop_debt, mi2_debt);

    // 3. Negative control: mismatched shop_debts sum vs shop_debt or non-ascending/zero entry is rejected.
    let mut bad_sum = rs.to_save_data();
    bad_sum.world.shop_debt += 1;
    assert!(GameSession::from_save_data(bad_sum).is_err());

    let mut bad_zero = rs.to_save_data();
    bad_zero.world.shop_debts.push((main2, 0));
    assert!(GameSession::from_save_data(bad_zero).is_err());

    let _ = fs::remove_dir_all(&store_dir);
}

#[test]
fn a37_06_snapshot_hash_reflects_unpaid_level_and_shop_debts() {
    let main2 = LevelId::main(2);
    let mines1 = LevelId::new(BranchId::Mines, 1);

    let mut base = start_campaign(42, Role::Knight).to_save_data();
    let (_, m2_pos) = find_unpaid_on_level(&base, main2, None);
    let (mi_item, mi_pos) = find_unpaid_on_level(&base, mines1, Some(ItemKind::ScrollEnchantArmor));
    teleport_player(&mut base, main2, m2_pos);
    let mut s = GameSession::from_save_data(base).unwrap();
    assert!(s.submit(C::Pickup).accepted);
    let m2_amt = s.world().shop_debt;
    let mut s_next = s.to_save_data();
    teleport_player(&mut s_next, mines1, mi_pos);
    s = GameSession::from_save_data(s_next).unwrap();
    assert!(s.submit(C::Pickup).accepted);
    let total = s.world().shop_debt;

    let save_a = s.to_save_data();
    let mut save_b = save_a.clone();
    save_b
        .world
        .entities
        .set_item_unpaid_level(mi_item, Some(main2));
    save_b.world.shop_debts = vec![(main2, total)];

    let sess_a = GameSession::from_save_data(save_a).unwrap();
    let sess_b = GameSession::from_save_data(save_b).unwrap();
    assert_ne!(
        sess_a.snapshot().stable_hash(),
        sess_b.snapshot().stable_hash(),
        "states differing in unpaid_level / shop_debts must not hash identically"
    );
    assert!(total > m2_amt);

    // Consumed-debt states with different creditor levels also hash differently.
    let mut save_c = sess_a.to_save_data();
    let inv_items: Vec<EntityId> = save_c
        .world
        .inventory
        .entries
        .iter()
        .map(|e| e.item)
        .collect();
    for id in inv_items {
        if save_c.world.entities.get(id).and_then(|e| e.item_unpaid()) == Some(true) {
            save_c.world.entities.set_item_unpaid(id, false);
            save_c.world.entities.set_item_unpaid_level(id, None);
        }
    }
    save_c.world.shop_debt = 80;
    save_c.world.shop_debts = vec![(mines1, 80)];
    let mut save_d = save_c.clone();
    save_d.world.shop_debts = vec![(main2, 80)];
    assert_ne!(
        GameSession::from_save_data(save_c)
            .unwrap()
            .snapshot()
            .stable_hash(),
        GameSession::from_save_data(save_d)
            .unwrap()
            .snapshot()
            .stable_hash(),
    );

    // Legacy V1 omits shop_debts / unpaid_level from GameSnapshot JSON cleanly.
    let legacy = GameSession::try_new_for_playing(42).unwrap();
    let legacy_json = serde_json::to_string(&legacy.snapshot()).unwrap();
    assert!(!legacy_json.contains("shop_debts"));
    assert!(!legacy_json.contains("unpaid_level"));
}

#[test]
fn a37_03_wand_wishing_natural_acquisition_and_full_catalog_supply() {
    // 1. Audit 37 catalog scan parity across seeds [42, 7, 1234] x 13 roles (39 fresh worlds).
    let mut books = HashSet::new();
    let mut wands = HashSet::new();
    let mut wishing_worlds = 0usize;
    let geh1 = LevelId::new(BranchId::Gehennom, 1);

    for seed in [42_u64, 7, 1234] {
        for role in Role::ALL {
            let sess = start_campaign(seed, role);
            let mut found_wish = false;
            for e in sess.world().entities().entities() {
                let Some((k, d, loc, _, _)) = e.item() else {
                    continue;
                };
                if d.class == ItemClass::Spellbook {
                    books.insert(k);
                }
                if d.class == ItemClass::Wand {
                    wands.insert(k);
                }
                if k == ItemKind::WandWishing {
                    found_wish = true;
                    assert!(matches!(loc, EntityLocation::OnMap { level, .. } if level == geh1));
                    assert!(!sess.world().is_item_identified(k));
                }
            }
            wishing_worlds += usize::from(found_wish);
        }
    }
    assert_eq!(wishing_worlds, 39);
    assert_eq!(books.len(), 28, "all 28 spellbooks must supply naturally");
    assert_eq!(wands.len(), 18, "all 18 wands must supply naturally");

    // 2. Fixed seed (42, Knight) natural pickup, identify via zap, wish, charge decrement & save roundtrip.
    let store_dir = temp_store_dir("a37_03");
    let store = ArtifactStore::open(&store_dir).unwrap();
    let mut save = start_campaign(42, Role::Knight).to_save_data();
    let (wand_id, wand_pos) = save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            let (k, _, loc, _, _) = e.item()?;
            match loc {
                EntityLocation::OnMap { level, pos }
                    if level == geh1 && k == ItemKind::WandWishing =>
                {
                    Some((e.id, pos))
                }
                _ => None,
            }
        })
        .expect("Gehennom 1 must spawn WandWishing");
    teleport_player(&mut save, geh1, wand_pos);
    let mut sess = GameSession::from_save_data(save).unwrap();
    assert!(!sess.world().is_item_identified(ItemKind::WandWishing));
    assert!(sess.submit(C::Pickup).accepted);
    assert!(!sess.world().is_item_identified(ItemKind::WandWishing));

    let zap_out = sess.submit(C::Zap {
        item: wand_id,
        direction: Direction::East,
    });
    assert!(zap_out.accepted, "{:?}", zap_out.events);
    assert!(sess.world().is_item_identified(ItemKind::WandWishing));
    assert_eq!(
        sess.world()
            .entities()
            .get(wand_id)
            .unwrap()
            .item()
            .unwrap()
            .4,
        Some(3)
    );

    let wish_cmd = C::Wish {
        item: wand_id,
        wish: WishChoice::Item {
            kind: ItemKind::ArmorSilverDragonScale,
            blessed: true,
            enchant: 2,
        },
    };
    let wish_out = sess.submit(wish_cmd);
    assert!(wish_out.accepted, "{:?}", wish_out.events);
    assert_eq!(
        sess.world()
            .entities()
            .get(wand_id)
            .unwrap()
            .item()
            .unwrap()
            .4,
        Some(2)
    );
    assert!(sess
        .observation()
        .inventory
        .iter()
        .any(|i| i.kind == ItemKind::ArmorSilverDragonScale));

    store
        .save_session(&sess, Path::new("wished_geh1.json"))
        .unwrap();
    let reloaded = store.load_session(Path::new("wished_geh1.json")).unwrap();
    assert_eq!(
        sess.snapshot().stable_hash(),
        reloaded.snapshot().stable_hash()
    );
    let _ = fs::remove_dir_all(&store_dir);
}

fn sokoban_static_shortest_steps(
    map: &aihack_core::domain::map::GameMap,
    start: Pos,
    target: Pos,
) -> Option<usize> {
    let mut dist = HashMap::new();
    let mut queue = VecDeque::new();
    dist.insert(start, 0usize);
    queue.push_back(start);
    while let Some(pos) = queue.pop_front() {
        let d = dist[&pos];
        if pos == target {
            return Some(d);
        }
        for dir in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let next = pos.offset(dir.delta());
            if !map.contains(next) || dist.contains_key(&next) {
                continue;
            }
            let blocked = matches!(
                map.tile(next).ok(),
                Some(
                    TileKind::Wall
                        | TileKind::Boulder
                        | TileKind::Water
                        | TileKind::Lava
                        | TileKind::Trap(TrapKind::Pit)
                        | TileKind::HiddenTrap(TrapKind::Pit)
                ) | None
            );
            if !blocked {
                dist.insert(next, d + 1);
                queue.push_back(next);
            }
        }
    }
    None
}

#[test]
fn a37_04_sokoban_mandatory_boulder_pit_choke_all_four_floors() {
    for seed in [42_u64, 7, 1234] {
        let base_save = start_campaign(seed, Role::Knight).to_save_data();
        for depth in 1..=4_i16 {
            let lvl = LevelId::new(BranchId::Sokoban, depth);
            let map = base_save.world.levels.map(lvl).unwrap();
            let start = base_save.world.levels.stairs_up_pos(lvl).unwrap();
            let target = base_save
                .world
                .levels
                .stairs_down_pos(lvl)
                .unwrap_or(Pos { x: 70, y: 10 });
            assert_eq!(
                sokoban_static_shortest_steps(map, start, target),
                None,
                "Sokoban {depth} (seed {seed}) must not allow bypassing boulder/pit puzzle"
            );

            // Check direct step onto unplugged pit is absent from legal_actions and rejected,
            // and pushing the boulder into x=42 pit unlocks reachability + roundtrips cleanly.
            let mut s = base_save.clone();
            // Move any hostiles off Sokoban center lanes onto Mines 2 so East push is clean.
            let mines2 = LevelId::new(BranchId::Mines, 2);
            let m2_floor = s.world.levels.stairs_up_pos(mines2).unwrap();
            let sok_actors: Vec<EntityId> = s
                .world
                .entities
                .entities()
                .iter()
                .filter_map(|e| {
                    let (_, _, l, _, _, alive) = e.actor()?;
                    (alive && l == lvl && e.id != s.world.player_id).then_some(e.id)
                })
                .collect();
            for aid in sok_actors {
                s.world.entities.set_actor_location(aid, mines2, m2_floor);
            }

            let pit_y = (8..=12_i16)
                .find(|&y| {
                    matches!(
                        map.tile(Pos { x: 42, y }).ok(),
                        Some(TileKind::Trap(TrapKind::Pit) | TileKind::HiddenTrap(TrapKind::Pit))
                    )
                })
                .unwrap();
            teleport_player(&mut s, lvl, Pos { x: 41, y: pit_y });
            let mut probe = GameSession::from_save_data(s.clone()).unwrap();
            assert!(!probe
                .observation()
                .legal_actions
                .contains(&C::Move(Direction::East)));
            assert!(!probe.submit(C::Move(Direction::East)).accepted);

            let boulder_x = (34..42_i16)
                .find(|&x| map.tile(Pos { x, y: pit_y }).ok() == Some(TileKind::Boulder))
                .unwrap();
            teleport_player(
                &mut s,
                lvl,
                Pos {
                    x: boulder_x - 1,
                    y: pit_y,
                },
            );
            let mut pusher = GameSession::from_save_data(s).unwrap();
            let steps_needed = 42 - boulder_x;
            for _ in 0..steps_needed {
                let mv = pusher.submit(C::Move(Direction::East));
                assert!(
                    mv.accepted,
                    "push failed on Sokoban {depth}: {:?}",
                    mv.events
                );
            }
            let updated_map = pusher.world().levels.map(lvl).unwrap();
            assert_eq!(
                updated_map.tile(Pos { x: 42, y: pit_y }).ok(),
                Some(TileKind::Floor)
            );
            assert!(
                sokoban_static_shortest_steps(updated_map, start, target).is_some(),
                "Sokoban {depth} must become reachable once pit is plugged"
            );
            let reloaded = GameSession::from_save_data(pusher.to_save_data()).unwrap();
            assert_eq!(
                pusher.snapshot().stable_hash(),
                reloaded.snapshot().stable_hash()
            );
        }
    }
}

fn make_synthetic_p12_v3_save(seed: u64, role: Role) -> SaveDataV1 {
    let mut save = start_campaign(seed, role).to_save_data();
    let p12_ids = [
        LevelId::main(1),
        LevelId::main(2),
        LevelId::main(3),
        LevelId::main(4),
        LevelId::main(5),
        LevelId::main(6),
        LevelId::new(BranchId::Mines, 1),
        LevelId::new(BranchId::Mines, 2),
    ];
    let main6 = LevelId::main(6);
    let main6_down = save.world.levels.stairs_down_pos(main6).unwrap();
    save.world
        .levels
        .map_mut(main6)
        .unwrap()
        .set_tile(main6_down, TileKind::Floor)
        .unwrap();
    save.world.levels.levels.retain(|l| p12_ids.contains(&l.id));
    save.world.ascension_altars.clear();
    let amulet_id = save.world.campaign.unwrap().amulet;
    save.world.entities.set_item_location(
        amulet_id,
        EntityLocation::OnMap {
            level: main6,
            pos: main6_down,
        },
    );

    let keep_ids: HashSet<EntityId> = save
        .world
        .entities
        .entities()
        .iter()
        .filter(|e| {
            if let Some((_, _, lvl, _, _, _)) = e.actor() {
                p12_ids.contains(&lvl)
            } else if let Some((_, _, EntityLocation::OnMap { level, .. }, _, _)) = e.item() {
                p12_ids.contains(&level)
            } else {
                true
            }
        })
        .map(|e| e.id)
        .collect();

    let mut json_val = serde_json::to_value(&save).unwrap();
    let entities = json_val["world"]["entities"]["entities"]
        .as_array_mut()
        .unwrap();
    entities.retain(|ent| {
        let id = EntityId(ent["id"].as_u64().unwrap() as u32);
        keep_ids.contains(&id)
    });
    let max_id = entities
        .iter()
        .map(|e| e["id"].as_u64().unwrap())
        .max()
        .unwrap();
    json_val["world"]["entities"]["next_id"] = serde_json::json!(max_id + 1);
    json_val["schema_version"] = serde_json::json!(SAVE_SCHEMA_VERSION_V3);
    serde_json::from_value(json_val).unwrap()
}

#[test]
fn a37_05_p12_v3_non_destructive_upgrade_and_schema_guards() {
    let store_dir = temp_store_dir("a37_05");
    let store = ArtifactStore::open(&store_dir).unwrap();

    // 1. Self-contained 8-level P12 V3 save upgrades cleanly in memory without modifying disk file bytes.
    let p12_v3 = make_synthetic_p12_v3_save(42, Role::Knight);
    assert_eq!(p12_v3.schema_version, SAVE_SCHEMA_VERSION_V3);
    assert_eq!(p12_v3.world.levels.len(), 8);
    let p12_bytes = serde_json::to_vec_pretty(&p12_v3).unwrap();
    fs::write(store_dir.join("p12_v3.json"), &p12_bytes).unwrap();

    let mut upgraded = store.load_session(Path::new("p12_v3.json")).unwrap();
    let after_bytes = fs::read(store_dir.join("p12_v3.json")).unwrap();
    assert_eq!(
        p12_bytes, after_bytes,
        "non-destructive V3->V4 load must not modify source file on disk"
    );
    assert_eq!(upgraded.world().levels.len(), 24);
    assert_eq!(
        upgraded.to_save_data().schema_version,
        SAVE_SCHEMA_VERSION_V4
    );
    assert_eq!(upgraded.world().ascension_altars.len(), 3);
    let amulet_id = upgraded.world().campaign.unwrap().amulet;
    assert!(matches!(
        upgraded.world().entities().get(amulet_id).unwrap().item().unwrap().2,
        EntityLocation::OnMap { level, .. } if level == LevelId::new(BranchId::Gehennom, 4)
    ));
    let report =
        aihack_headless::run_to_turn(&mut upgraded, 5, aihack_headless::HeadlessPolicy::wait_v1())
            .unwrap();
    assert_eq!(report.accepted_turns, 5);
    assert_eq!(upgraded.turn(), 5);

    // 2. Real Audit 37 P12 artifact check (read-only) if present on disk: preserves exact SHA-256.
    let audit_runtime = Path::new("target/audit37-20261128-1859/runtime");
    let audit_p12_file = audit_runtime.join("p12-run37.json");
    if audit_p12_file.exists() {
        let orig_bytes = fs::read(&audit_p12_file).unwrap();
        assert_eq!(
            sha256_hex(&orig_bytes),
            "822490777ad16a95e1f923007bf04dd12a4621bc0a4f490d23d6118371e45199"
        );
        let audit_store = ArtifactStore::open(audit_runtime).unwrap();
        let mut real_p12 = audit_store
            .load_session(Path::new("p12-run37.json"))
            .unwrap();
        assert_eq!(
            sha256_hex(&fs::read(&audit_p12_file).unwrap()),
            "822490777ad16a95e1f923007bf04dd12a4621bc0a4f490d23d6118371e45199"
        );
        let start_turn = real_p12.turn();
        let real_report = aihack_headless::run_to_turn(
            &mut real_p12,
            start_turn + 4,
            aihack_headless::HeadlessPolicy::wait_v1(),
        )
        .unwrap();
        assert_eq!(real_report.accepted_turns, 4);
        assert_eq!(real_p12.world().levels.len(), 24);
        assert_eq!(
            real_p12.to_save_data().schema_version,
            SAVE_SCHEMA_VERSION_V4
        );
    }

    // 3. Carried amulet + legacy unpaid P12 V3 upgrade preserves inventory amulet and backfills shop_debts.
    let mut p12_carried = make_synthetic_p12_v3_save(42, Role::Knight);
    let carried_amulet = p12_carried.world.campaign.unwrap().amulet;
    let letter = p12_carried
        .world
        .inventory
        .add_existing_with_next_letter(carried_amulet)
        .unwrap();
    p12_carried.world.entities.set_item_location(
        carried_amulet,
        EntityLocation::Inventory {
            owner: p12_carried.world.player_id,
        },
    );
    p12_carried
        .world
        .entities
        .set_item_letter(carried_amulet, letter);
    p12_carried.world.shop_debt = 80;
    let carried_sess = GameSession::from_save_data(p12_carried).unwrap();
    assert_eq!(
        carried_sess
            .world()
            .entities()
            .get(carried_amulet)
            .unwrap()
            .item()
            .unwrap()
            .2,
        EntityLocation::Inventory {
            owner: carried_sess.world().player_id
        }
    );
    assert_eq!(
        carried_sess.world().shop_debts,
        vec![(LevelId::main(2), 80)]
    );

    // 4. Negative controls: truncated 8-level V4, 24-level V3, V3 Victory, missing-level V3, V2, Legacy != V1.
    let mut trunc_v4 = p12_v3.clone();
    trunc_v4.schema_version = SAVE_SCHEMA_VERSION_V4;
    assert!(GameSession::from_save_data(trunc_v4).is_err());

    let mut p13_as_v3 = start_campaign(42, Role::Knight).to_save_data();
    p13_as_v3.schema_version = SAVE_SCHEMA_VERSION_V3;
    assert!(GameSession::from_save_data(p13_as_v3).is_err());

    let mut p12_victory = p12_v3.clone();
    p12_victory.run_state = RunState::Victory { final_score: 10000 };
    assert!(GameSession::from_save_data(p12_victory).is_err());

    let mut p12_seven_levels = p12_v3.clone();
    p12_seven_levels.world.levels.levels.pop();
    assert!(GameSession::from_save_data(p12_seven_levels).is_err());

    let mut v2_save = start_campaign(42, Role::Knight).to_save_data();
    v2_save.schema_version = SAVE_SCHEMA_VERSION_V2;
    assert!(GameSession::from_save_data(v2_save).is_err());

    let mut legacy_v4 = GameSession::try_new_for_playing(42).unwrap().to_save_data();
    assert_eq!(legacy_v4.schema_version, SAVE_SCHEMA_VERSION_V1);
    legacy_v4.schema_version = SAVE_SCHEMA_VERSION_V4;
    assert!(GameSession::from_save_data(legacy_v4).is_err());

    let _ = fs::remove_dir_all(&store_dir);
}
