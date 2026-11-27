use aihack_core::{
    action::CommandIntent as C,
    campaign::{Alignment, Race, Role},
    domain::{
        entity::{ActorKind, Entity, EntityLocation, Faction},
        item::{BucStatus, ItemKind},
        monster::MonsterKind,
        tile::{DoorState, TileKind},
    },
    event::GameEvent,
    ids::{BranchId, EntityId, LevelId},
    position::{Direction, Pos},
};
use aihack_headless::{run_to_turn, HeadlessPolicy};
use aihack_llm::service::LlmObservationView;
use aihack_runtime::GameSession;

fn start_campaign(seed: u64, role: Role, race: Race, alignment: Alignment) -> GameSession {
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

fn find_tile(
    save: &aihack_runtime::save::SaveDataV1,
    level: LevelId,
    predicate: impl Fn(TileKind) -> bool,
) -> Pos {
    let map = save.world.levels.map(level).unwrap();
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = Pos { x, y };
            if map.tile(pos).ok().is_some_and(&predicate) {
                return pos;
            }
        }
    }
    panic!("tile not found on {level:?}");
}

fn open_floor_neighbors(
    save: &aihack_runtime::save::SaveDataV1,
    level: LevelId,
    center: Pos,
) -> Vec<(Direction, Pos)> {
    let map = save.world.levels.map(level).unwrap();
    Direction::ALL
        .into_iter()
        .filter_map(|dir| {
            let pos = center.offset(dir.delta());
            (map.contains(pos)
                && map.tile(pos).ok() == Some(TileKind::Floor)
                && save.world.entities.alive_actor_at(level, pos).is_none())
            .then_some((dir, pos))
        })
        .collect()
}

fn park_all_monsters_on(save: &mut aihack_runtime::save::SaveDataV1, park_level: LevelId) {
    let map = save.world.levels.map(park_level).unwrap().clone();
    let mut floor_iter = (1..map.height - 1).flat_map(|y| {
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
        if let Some(pos) = floor_iter.next() {
            save.world.entities.set_actor_location(id, park_level, pos);
        }
    }
}

fn spawn_monster_with_faction(
    save: &mut aihack_runtime::save::SaveDataV1,
    kind: MonsterKind,
    faction: Faction,
    level: LevelId,
    pos: Pos,
) -> EntityId {
    let reg = aihack_content::registry().unwrap();
    let tmpl = aihack_content::monster_template_from_registry(kind, reg).unwrap();
    let id = save
        .world
        .entities
        .spawn_monster_with_faction(kind, tmpl, faction, pos)
        .unwrap();
    save.world.entities.set_actor_location(id, level, pos);
    id
}

#[test]
fn p11_1_starting_pet_friendly_swap_feeding_taming_stairs_and_starvation() {
    // Role/race starting pets
    let knight = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful);
    let knight_pet = knight
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.faction() == Some(Faction::Pet))
        .expect("Knight must start with a pet");
    assert_eq!(
        knight_pet.actor_kind(),
        Some(ActorKind::Monster(MonsterKind::Pony))
    );
    assert_eq!(knight_pet.actor_tameness(), Some(5));
    assert_eq!(knight_pet.actor_pet_hunger(), Some(600));

    let mage = start_campaign(42, Role::Mage, Race::Elf, Alignment::Neutral);
    let mage_pet = mage
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.faction() == Some(Faction::Pet))
        .expect("Mage/Elf must start with a kitten");
    assert_eq!(
        mage_pet.actor_kind(),
        Some(ActorKind::Monster(MonsterKind::Kitten))
    );

    let scout = start_campaign(42, Role::Scout, Race::Human, Alignment::Neutral);
    let scout_pet = scout
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.faction() == Some(Faction::Pet))
        .expect("Scout must start with a little dog");
    assert_eq!(
        scout_pet.actor_kind(),
        Some(ActorKind::Monster(MonsterKind::LittleDog))
    );

    // Friendly displacement swap with starting pet
    let mut session = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful);
    let player_start = session.world().player_pos();
    let (pet_id, pet_pos) = session
        .world()
        .entities()
        .entities()
        .iter()
        .find_map(|e| {
            let (_, Faction::Pet, _, pos, _, true) = e.actor()? else {
                return None;
            };
            Some((e.id, pos))
        })
        .unwrap();
    let dir = Direction::ALL
        .into_iter()
        .find(|d| player_start.offset(d.delta()) == pet_pos)
        .expect("pet spawns adjacent to player");
    assert!(session.observation().legal_actions.contains(&C::Move(dir)));
    let swap_out = session.submit(C::Move(dir));
    assert!(swap_out.accepted, "{:?}", swap_out.events);
    assert_eq!(session.world().player_pos(), pet_pos);
    assert_eq!(
        session.world().entities().actor_location(pet_id).unwrap().1,
        player_start
    );
    assert_eq!(
        session.world().entities().get(pet_id).unwrap().faction(),
        Some(Faction::Pet)
    );

    // Taming wild animal by throwing food without harming it
    let mut save = start_campaign(7, Role::Scout, Race::Human, Alignment::Neutral).to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut save, LevelId::main(6));
    let p_pos = save
        .world
        .entities
        .actor_location(save.world.player_id)
        .unwrap()
        .1;
    let (tame_dir, dog_pos) = open_floor_neighbors(&save, main1, p_pos)[0];
    let wild_dog = spawn_monster_with_faction(
        &mut save,
        MonsterKind::LittleDog,
        Faction::Hostile,
        main1,
        dog_pos,
    );
    let food_id = save
        .world
        .inventory
        .entries
        .iter()
        .find(|e| {
            save.world
                .entities
                .item_data(e.item)
                .is_some_and(|d| d.kind == ItemKind::FoodRation)
        })
        .unwrap()
        .item;
    let mut tame_session = GameSession::from_save_data(save).unwrap();
    let throw_out = tame_session.submit(C::Throw {
        item: food_id,
        direction: tame_dir,
    });
    assert!(throw_out.accepted, "{:?}", throw_out.events);
    let dog_entity = tame_session.world().entities().get(wild_dog).unwrap();
    assert!(dog_entity.is_alive_actor());
    assert_eq!(dog_entity.faction(), Some(Faction::Pet));
    assert!(dog_entity.actor_tameness().unwrap_or(0) >= 3);

    // Pet follows player across stairs
    let mut stairs_save = tame_session.to_save_data();
    let stairs_down = find_tile(&stairs_save, main1, |t| t == TileKind::StairsDown);
    stairs_save
        .world
        .entities
        .set_actor_location(stairs_save.world.player_id, main1, stairs_down);
    let adj = open_floor_neighbors(&stairs_save, main1, stairs_down)[0].1;
    stairs_save
        .world
        .entities
        .set_actor_location(wild_dog, main1, adj);
    let mut stairs_session = GameSession::from_save_data(stairs_save).unwrap();
    let desc = stairs_session.submit(C::Descend);
    assert!(desc.accepted, "{:?}", desc.events);
    assert_eq!(stairs_session.world().current_level(), LevelId::main(2));
    assert_eq!(
        stairs_session
            .world()
            .entities()
            .actor_location(wild_dog)
            .unwrap()
            .0,
        LevelId::main(2)
    );

    // Starving pet goes feral at turn multiple of 20
    let mut starve_save =
        start_campaign(9, Role::Mage, Race::Human, Alignment::Neutral).to_save_data();
    park_all_monsters_on(&mut starve_save, LevelId::main(6));
    let p_pos = starve_save
        .world
        .entities
        .actor_location(starve_save.world.player_id)
        .unwrap()
        .1;
    let pet_pos = open_floor_neighbors(&starve_save, main1, p_pos)[0].1;
    let starving_pet = spawn_monster_with_faction(
        &mut starve_save,
        MonsterKind::Kitten,
        Faction::Pet,
        main1,
        pet_pos,
    );
    starve_save
        .world
        .entities
        .set_actor_tameness(starving_pet, 1);
    starve_save
        .world
        .entities
        .set_actor_pet_hunger(starving_pet, 0);
    starve_save.turn = 19;
    let mut starve_session = GameSession::from_save_data(starve_save).unwrap();
    let out = starve_session.submit(C::Wait);
    assert!(out.accepted);
    assert_eq!(
        starve_session
            .world()
            .entities()
            .get(starving_pet)
            .unwrap()
            .faction(),
        Some(Faction::Hostile)
    );
}

#[test]
fn p11_2_pet_combat_evolution_cursed_avoidance_and_shop_stealing() {
    // Pet fights hostile monster, earns XP, and evolves Kitten -> Housecat
    let mut save = start_campaign(42, Role::Mage, Race::Human, Alignment::Neutral).to_save_data();
    let main1 = LevelId::main(1);
    park_all_monsters_on(&mut save, LevelId::main(6));
    let p_pos = save
        .world
        .entities
        .actor_location(save.world.player_id)
        .unwrap()
        .1;
    let neighbors = open_floor_neighbors(&save, main1, p_pos);
    let pet_pos = neighbors[0].1;
    let pet_id =
        spawn_monster_with_faction(&mut save, MonsterKind::Kitten, Faction::Pet, main1, pet_pos);
    save.world.entities.set_actor_tameness(pet_id, 6);
    save.world.entities.set_actor_pet_hunger(pet_id, 600);
    save.world.entities.set_actor_pet_xp(pet_id, 10);
    let foe_pos = open_floor_neighbors(&save, main1, pet_pos)
        .into_iter()
        .find(|&(_, p)| p != p_pos)
        .unwrap()
        .1;
    let foe_id = spawn_monster_with_faction(
        &mut save,
        MonsterKind::Goblin,
        Faction::Hostile,
        main1,
        foe_pos,
    );
    save.world.entities.actor_stats_mut(foe_id).unwrap().hp = 1;
    save.world.entities.actor_stats_mut(foe_id).unwrap().ac = 10;
    save.world
        .entities
        .actor_stats_mut(pet_id)
        .unwrap()
        .hit_bonus = 20;
    let mut session = GameSession::from_save_data(save).unwrap();
    for _ in 0..5 {
        if !session
            .world()
            .entities()
            .get(foe_id)
            .unwrap()
            .is_alive_actor()
        {
            break;
        }
        assert!(session.submit(C::Wait).accepted);
    }
    assert!(!session
        .world()
        .entities()
        .get(foe_id)
        .unwrap()
        .is_alive_actor());
    let evolved = session.world().entities().get(pet_id).unwrap();
    assert_eq!(
        evolved.actor_kind(),
        Some(ActorKind::Monster(MonsterKind::Housecat))
    );
    assert!(evolved.actor_stats_hp() >= 12);

    // Pet avoids cursed floor item and reveals buc_known = true
    let mut csave = start_campaign(12, Role::Mage, Race::Human, Alignment::Neutral).to_save_data();
    park_all_monsters_on(&mut csave, LevelId::main(6));
    let p_pos = csave
        .world
        .entities
        .actor_location(csave.world.player_id)
        .unwrap()
        .1;
    // Place pet 2 tiles away from player with a cursed dagger on the direct tile between them
    let map = csave.world.levels.map(main1).unwrap().clone();
    let (mid_pos, far_pos) = Direction::ALL
        .into_iter()
        .find_map(|d| {
            let mid = p_pos.offset(d.delta());
            let far = mid.offset(d.delta());
            (map.contains(mid)
                && map.contains(far)
                && map.tile(mid).ok() == Some(TileKind::Floor)
                && map.tile(far).ok() == Some(TileKind::Floor))
            .then_some((mid, far))
        })
        .expect("straight 2-tile floor corridor near start");
    let cpet = spawn_monster_with_faction(
        &mut csave,
        MonsterKind::Kitten,
        Faction::Pet,
        main1,
        far_pos,
    );
    csave.world.entities.set_actor_tameness(cpet, 5);
    csave.world.entities.set_actor_pet_hunger(cpet, 600);
    let cursed_dagger = csave
        .world
        .entities
        .spawn_item(
            ItemKind::Dagger,
            EntityLocation::OnMap {
                level: main1,
                pos: mid_pos,
            },
        )
        .unwrap();
    csave
        .world
        .entities
        .set_item_buc(cursed_dagger, BucStatus::Cursed);
    let mut csession = GameSession::from_save_data(csave).unwrap();
    let out = csession.submit(C::Wait);
    assert!(out.accepted);
    assert_ne!(
        csession.world().entities().actor_location(cpet).unwrap().1,
        mid_pos,
        "pet must refuse to step onto cursed floor item"
    );
    assert_eq!(
        csession
            .world()
            .entities()
            .get(cursed_dagger)
            .unwrap()
            .item_buc_known(),
        Some(true)
    );

    // Pet steals unpaid item from Main 2 shop and drops it outside shop with unpaid = false
    let mut ssave = start_campaign(42, Role::Mage, Race::Human, Alignment::Neutral).to_save_data();
    let main2 = LevelId::main(2);
    let shop_pos = find_tile(&ssave, main2, |t| t == TileKind::Shop);
    let (unpaid_id, unpaid_pos) = ssave
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            if e.item_unpaid() != Some(true) {
                return None;
            }
            match e.item()?.2 {
                EntityLocation::OnMap { level, pos } if level == main2 => Some((e.id, pos)),
                _ => None,
            }
        })
        .expect("Main 2 shop must have unpaid floor wares");
    // Place player outside shop zone (distance > 4) on Main 2 floor
    let map2 = ssave.world.levels.map(main2).unwrap().clone();
    let outside_pos = (1..map2.height - 1)
        .flat_map(|y| (1..map2.width - 1).map(move |x| Pos { x, y }))
        .find(|&p| {
            map2.tile(p).ok() == Some(TileKind::Floor)
                && p.chebyshev_distance(shop_pos) > 5
                && ssave.world.entities.alive_actor_at(main2, p).is_none()
        })
        .unwrap();
    ssave.world.current_level = main2;
    ssave
        .world
        .entities
        .set_actor_location(ssave.world.player_id, main2, outside_pos);
    let thief_pet = spawn_monster_with_faction(
        &mut ssave,
        MonsterKind::Kitten,
        Faction::Pet,
        main2,
        unpaid_pos,
    );
    ssave.world.entities.set_actor_tameness(thief_pet, 8);
    ssave.world.entities.set_actor_pet_hunger(thief_pet, 600);
    let mut ssession = GameSession::from_save_data(ssave).unwrap();
    // Turn 1: pet stands on unpaid item inside shop -> picks it up into monster_inventory!
    assert!(ssession.submit(C::Wait).accepted);
    assert_eq!(ssession.world().shop_debt, 0);
    assert!(ssession
        .world()
        .entities()
        .get(thief_pet)
        .unwrap()
        .actor_monster_inventory()
        .unwrap()
        .contains(&unpaid_id));
    // Move pet next to player outside shop -> pet drops item with unpaid = false!
    let mut drop_save = ssession.to_save_data();
    let drop_spot = open_floor_neighbors(&drop_save, main2, outside_pos)[0].1;
    drop_save
        .world
        .entities
        .set_actor_location(thief_pet, main2, drop_spot);
    let mut dsession = GameSession::from_save_data(drop_save).unwrap();
    assert!(dsession.submit(C::Wait).accepted);
    let item_after = dsession.world().entities().get(unpaid_id).unwrap();
    assert_eq!(item_after.item_unpaid(), Some(false));
    assert!(matches!(
        item_after.item().unwrap().2,
        EntityLocation::OnMap { level, .. } if level == main2
    ));
}

trait ActorHpExt {
    fn actor_stats_hp(&self) -> i16;
}

impl ActorHpExt for Entity {
    fn actor_stats_hp(&self) -> i16 {
        self.actor().map(|(_, _, _, _, s, _)| s.hp).unwrap_or(0)
    }
}

#[test]
fn p11_3_living_npcs_shop_debt_pay_theft_priest_oracle_and_watchman() {
    let base = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    let main2 = LevelId::main(2);
    let main3 = LevelId::main(3);
    let main4 = LevelId::main(4);
    let mines1 = LevelId {
        branch: BranchId::Mines,
        depth: 1,
    };

    // Verify all 4 NPC archetypes exist in generated campaign
    assert!(base.world.entities.entities().iter().any(|e| matches!(
        e.actor(),
        Some((ActorKind::Monster(MonsterKind::Shopkeeper), Faction::Shopkeeper, l, _, _, true)) if l == main2
    )));
    assert!(base.world.entities.entities().iter().any(|e| matches!(
        e.actor(),
        Some((ActorKind::Monster(MonsterKind::TemplePriest), Faction::Peaceful, l, _, _, true)) if l == main3
    )));
    assert!(base.world.entities.entities().iter().any(|e| matches!(
        e.actor(),
        Some((ActorKind::Monster(MonsterKind::Oracle), Faction::Peaceful, l, _, _, true)) if l == main4
    )));
    assert!(base.world.entities.entities().iter().any(|e| matches!(
        e.actor(),
        Some((ActorKind::Monster(MonsterKind::Watchman), Faction::Peaceful, l, _, _, true)) if l == mines1
    )));

    // 1. Shop unpaid pickup -> debt -> Pay -> debt cleared & unpaid=false
    let mut shop_save = base.clone();
    shop_save.world.gold = 1000;
    let (ware_id, ware_pos) = shop_save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            if e.item_unpaid() != Some(true) {
                return None;
            }
            match e.item()?.2 {
                EntityLocation::OnMap { level, pos } if level == main2 => Some((e.id, pos)),
                _ => None,
            }
        })
        .unwrap();
    shop_save.world.current_level = main2;
    shop_save
        .world
        .entities
        .set_actor_location(shop_save.world.player_id, main2, ware_pos);
    let mut shop_session = GameSession::from_save_data(shop_save).unwrap();
    let p_out = shop_session.submit(C::Pickup);
    assert!(p_out.accepted, "{:?}", p_out.events);
    let debt = shop_session.world().shop_debt;
    assert!(debt > 0);
    assert_eq!(shop_session.observation().player.shop_debt, debt);
    assert!(shop_session
        .observation()
        .inventory
        .iter()
        .any(|i| i.item == ware_id && i.unpaid));
    assert!(shop_session.observation().legal_actions.contains(&C::Pay));

    let gold_before = shop_session.world().gold();
    let pay_out = shop_session.submit(C::Pay);
    assert!(pay_out.accepted, "{:?}", pay_out.events);
    assert_eq!(shop_session.world().shop_debt, 0);
    assert_eq!(shop_session.world().gold(), gold_before - debt);
    assert_eq!(
        shop_session
            .world()
            .entities()
            .get(ware_id)
            .unwrap()
            .item_unpaid(),
        Some(false)
    );

    // 2. Theft on Mines 1 (leaving shop with unpaid debt) angers Shopkeeper AND Watchman!
    let mut theft_save = base.clone();
    let shop_m1 = find_tile(&theft_save, mines1, |t| t == TileKind::Shop);
    let (m1_ware, m1_ware_pos) = theft_save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| {
            if e.item_unpaid() != Some(true) {
                return None;
            }
            match e.item()?.2 {
                EntityLocation::OnMap { level, pos } if level == mines1 => Some((e.id, pos)),
                _ => None,
            }
        })
        .unwrap();
    theft_save.world.current_level = mines1;
    theft_save
        .world
        .entities
        .set_actor_location(theft_save.world.player_id, mines1, m1_ware_pos);
    let mut theft_session = GameSession::from_save_data(theft_save).unwrap();
    assert!(theft_session.submit(C::Pickup).accepted);
    assert!(theft_session.world().shop_debt > 0);
    // Teleport player outside shop zone then take 1 step -> triggers theft anger!
    let mut flee_save = theft_session.to_save_data();
    let map_m1 = flee_save.world.levels.map(mines1).unwrap().clone();
    let (edge_pos, step_dir) = (1..map_m1.height - 1)
        .flat_map(|y| (1..map_m1.width - 1).map(move |x| Pos { x, y }))
        .find_map(|p| {
            if map_m1.tile(p).ok() != Some(TileKind::Floor)
                || p.chebyshev_distance(shop_m1) <= 5
                || flee_save.world.entities.alive_actor_at(mines1, p).is_some()
            {
                return None;
            }
            let dir = open_floor_neighbors(&flee_save, mines1, p)
                .into_iter()
                .find(|&(_, np)| np.chebyshev_distance(shop_m1) > 5)
                .map(|(d, _)| d)?;
            Some((p, dir))
        })
        .unwrap();
    flee_save
        .world
        .entities
        .set_actor_location(flee_save.world.player_id, mines1, edge_pos);
    let mut flee_session = GameSession::from_save_data(flee_save).unwrap();
    let flee_out = flee_session.submit(C::Move(step_dir));
    assert!(flee_out.accepted);
    assert!(flee_session.world().entities().entities().iter().any(|e| {
        matches!(
            e.actor(),
            Some((ActorKind::Monster(MonsterKind::Shopkeeper), Faction::Hostile, l, _, _, true))
                if l == mines1
        )
    }));
    assert!(flee_session.world().entities().entities().iter().any(|e| {
        matches!(
            e.actor(),
            Some((ActorKind::Monster(MonsterKind::Watchman), Faction::Hostile, l, _, _, true))
                if l == mines1
        )
    }));
    // Hostile shopkeeper refuses Buy/Sell on shop tile
    let mut angry_shop = flee_session.to_save_data();
    angry_shop
        .world
        .entities
        .set_actor_location(angry_shop.world.player_id, mines1, shop_m1);
    let mut angry_session = GameSession::from_save_data(angry_shop).unwrap();
    assert!(
        !angry_session
            .submit(C::Buy {
                kind: ItemKind::FoodRation
            })
            .accepted
    );
    let _ = m1_ware;

    // 3. TemplePriest Chat & Pay donation reduces god_anger, cures poison, increases luck
    let mut priest_save = base.clone();
    priest_save.world.current_level = main3;
    priest_save.world.gold = 500;
    priest_save.world.poison_turns = 4;
    priest_save.world.campaign.as_mut().unwrap().god_anger = 3;
    let priest_pos = priest_save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| match e.actor() {
            Some((
                ActorKind::Monster(MonsterKind::TemplePriest),
                Faction::Peaceful,
                l,
                p,
                _,
                true,
            )) if l == main3 => Some(p),
            _ => None,
        })
        .unwrap();
    let (p_dir_inv, player_spot) = open_floor_neighbors(&priest_save, main3, priest_pos)[0];
    let chat_dir = Direction::ALL
        .into_iter()
        .find(|d| player_spot.offset(d.delta()) == priest_pos)
        .unwrap();
    let _ = p_dir_inv;
    priest_save
        .world
        .entities
        .set_actor_location(priest_save.world.player_id, main3, player_spot);
    let mut priest_session = GameSession::from_save_data(priest_save).unwrap();
    let chat_out = priest_session.submit(C::Chat(chat_dir));
    assert!(chat_out.accepted);
    assert!(chat_out
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::Message { text, .. } if text.contains("Temple priest"))));
    let pay_out = priest_session.submit(C::Pay);
    assert!(pay_out.accepted, "{:?}", pay_out.events);
    assert_eq!(priest_session.world().campaign.unwrap().god_anger, 0);
    assert_eq!(priest_session.world().poison_turns, 0);
    assert!(priest_session.world().luck >= 1);

    // 4. Oracle Chat & Pay (50g) reveals hidden features
    let mut oracle_save = base.clone();
    oracle_save.world.current_level = main4;
    oracle_save.world.gold = 200;
    let oracle_pos = oracle_save
        .world
        .entities
        .entities()
        .iter()
        .find_map(|e| match e.actor() {
            Some((ActorKind::Monster(MonsterKind::Oracle), Faction::Peaceful, l, p, _, true))
                if l == main4 =>
            {
                Some(p)
            }
            _ => None,
        })
        .unwrap();
    let player_spot = open_floor_neighbors(&oracle_save, main4, oracle_pos)[0].1;
    let oracle_dir = Direction::ALL
        .into_iter()
        .find(|d| player_spot.offset(d.delta()) == oracle_pos)
        .unwrap();
    oracle_save
        .world
        .entities
        .set_actor_location(oracle_save.world.player_id, main4, player_spot);
    let mut oracle_session = GameSession::from_save_data(oracle_save).unwrap();
    let o_chat = oracle_session.submit(C::Chat(oracle_dir));
    assert!(o_chat.accepted);
    assert!(o_chat
        .events
        .iter()
        .any(|e| matches!(e, GameEvent::Message { text, .. } if text.contains("Oracle whispers"))));
    let o_pay = oracle_session.submit(C::Pay);
    assert!(o_pay.accepted, "{:?}", o_pay.events);
    assert_eq!(oracle_session.world().gold(), 150);
}

#[test]
fn p11_4_advanced_monster_ai_doors_inventory_consumables_and_death_drop() {
    let main1 = LevelId::main(1);

    // 1. Orc opens closed door when chasing player
    let mut dsave = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut dsave, LevelId::main(6));
    let door_pos = find_tile(&dsave, main1, |t| t == TileKind::Door(DoorState::Closed));
    let neighbors = open_floor_neighbors(&dsave, main1, door_pos);
    // Find two opposite orthogonal sides of the closed door
    let (side_a, side_b) = [
        (Direction::North, Direction::South),
        (Direction::West, Direction::East),
    ]
    .into_iter()
    .find_map(|(da, db)| {
        let a = door_pos.offset(da.delta());
        let b = door_pos.offset(db.delta());
        let map = dsave.world.levels.map(main1).unwrap();
        (map.tile(a).ok() == Some(TileKind::Floor) && map.tile(b).ok() == Some(TileKind::Floor))
            .then_some((a, b))
    })
    .unwrap_or_else(|| (neighbors[0].1, neighbors[1].1));
    dsave
        .world
        .entities
        .set_actor_location(dsave.world.player_id, main1, side_a);
    let _orc = spawn_monster_with_faction(
        &mut dsave,
        MonsterKind::Orc,
        Faction::Hostile,
        main1,
        side_b,
    );
    let mut dsession = GameSession::from_save_data(dsave).unwrap();
    let out = dsession.submit(C::Wait);
    assert!(out.accepted);
    assert_eq!(
        dsession.world().current_map().tile(door_pos).ok(),
        Some(TileKind::Door(DoorState::Open)),
        "Orc with opens_doors flag must open closed door"
    );

    // 2. Orc picks up floor weapon & potion, drinks potion when wounded, drops weapon on death
    let mut isave = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut isave, LevelId::main(6));
    let p_pos = isave
        .world
        .entities
        .actor_location(isave.world.player_id)
        .unwrap()
        .1;
    let orc_pos = open_floor_neighbors(&isave, main1, p_pos)[0].1;
    let orc_id = spawn_monster_with_faction(
        &mut isave,
        MonsterKind::Orc,
        Faction::Hostile,
        main1,
        orc_pos,
    );
    let floor_sword = isave
        .world
        .entities
        .spawn_item(
            ItemKind::LongSword,
            EntityLocation::OnMap {
                level: main1,
                pos: orc_pos,
            },
        )
        .unwrap();
    let mut isession = GameSession::from_save_data(isave).unwrap();
    assert!(isession.submit(C::Wait).accepted);
    assert!(
        isession
            .world()
            .entities()
            .get(orc_id)
            .unwrap()
            .actor_monster_inventory()
            .unwrap()
            .contains(&floor_sword),
        "Orc must pick up floor weapon into monster_inventory"
    );

    // Give Orc a healing potion and wound it -> Orc drinks potion!
    let mut hsave = isession.to_save_data();
    let pot_id = hsave
        .world
        .entities
        .spawn_item(
            ItemKind::PotionHealing,
            EntityLocation::Inventory { owner: orc_id },
        )
        .unwrap();
    hsave.world.entities.push_monster_inventory(orc_id, pot_id);
    hsave.world.entities.actor_stats_mut(orc_id).unwrap().hp = 2;
    let mut hsession = GameSession::from_save_data(hsave).unwrap();
    let h_out = hsession.submit(C::Wait);
    assert!(h_out.accepted);
    assert!(
        hsession.world().entities().actor_stats(orc_id).unwrap().hp > 2,
        "wounded Orc must drink carried healing potion"
    );
    assert_eq!(
        hsession.world().entities().item_location(pot_id),
        Some(EntityLocation::Consumed)
    );

    // Kill Orc -> floor_sword drops back to OnMap!
    let mut ksave = hsession.to_save_data();
    let orc_loc = ksave.world.entities.actor_location(orc_id).unwrap().1;
    let p_loc = ksave
        .world
        .entities
        .actor_location(ksave.world.player_id)
        .unwrap()
        .1;
    let dir = Direction::ALL
        .into_iter()
        .find(|d| p_loc.offset(d.delta()) == orc_loc)
        .unwrap();
    ksave.world.entities.actor_stats_mut(orc_id).unwrap().hp = 1;
    ksave.world.entities.actor_stats_mut(orc_id).unwrap().ac = 10;
    let mut ksession = GameSession::from_save_data(ksave).unwrap();
    for _ in 0..15 {
        if !ksession
            .world()
            .entities()
            .get(orc_id)
            .unwrap()
            .is_alive_actor()
        {
            break;
        }
        assert!(ksession.submit(C::Move(dir)).accepted);
    }
    assert!(!ksession
        .world()
        .entities()
        .get(orc_id)
        .unwrap()
        .is_alive_actor());
    assert!(matches!(
        ksession.world().entities().item_location(floor_sword),
        Some(EntityLocation::OnMap { level, pos }) if level == main1 && pos == orc_loc
    ));
}

#[test]
fn p11_5_special_monsters_steal_rust_split_gaze_summon_and_faction_war() {
    let main1 = LevelId::main(1);

    // 1. Leprechaun steals gold and teleports away
    let mut lsave = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut lsave, LevelId::main(6));
    lsave.world.gold = 200;
    let p_pos = lsave
        .world
        .entities
        .actor_location(lsave.world.player_id)
        .unwrap()
        .1;
    let lep_pos = open_floor_neighbors(&lsave, main1, p_pos)[0].1;
    let lep = spawn_monster_with_faction(
        &mut lsave,
        MonsterKind::Leprechaun,
        Faction::Hostile,
        main1,
        lep_pos,
    );
    lsave.world.entities.actor_stats_mut(lep).unwrap().hit_bonus = 25;
    let mut lsession = GameSession::from_save_data(lsave).unwrap();
    assert!(lsession.submit(C::Wait).accepted);
    assert!(lsession.world().gold() < 200);
    assert_ne!(
        lsession.world().entities().actor_location(lep).unwrap().1,
        lep_pos,
        "Leprechaun must teleport away after stealing gold"
    );

    // 2. Nymph steals non-Amulet item into monster_inventory and teleports away
    let mut nsave = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut nsave, LevelId::main(6));
    let p_pos = nsave
        .world
        .entities
        .actor_location(nsave.world.player_id)
        .unwrap()
        .1;
    let nym_pos = open_floor_neighbors(&nsave, main1, p_pos)[0].1;
    let nym = spawn_monster_with_faction(
        &mut nsave,
        MonsterKind::Nymph,
        Faction::Hostile,
        main1,
        nym_pos,
    );
    nsave.world.entities.actor_stats_mut(nym).unwrap().hit_bonus = 25;
    let inv_before = nsave.world.inventory.entries.len();
    let mut nsession = GameSession::from_save_data(nsave).unwrap();
    assert!(nsession.submit(C::Wait).accepted);
    assert_eq!(nsession.world().inventory().entries.len(), inv_before - 1);
    assert_eq!(
        nsession
            .world()
            .entities()
            .get(nym)
            .unwrap()
            .actor_monster_inventory()
            .unwrap()
            .len(),
        1
    );
    assert_ne!(
        nsession.world().entities().actor_location(nym).unwrap().1,
        nym_pos
    );

    // 3. RustMonster corrodes equipped iron armor
    let mut rsave = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut rsave, LevelId::main(6));
    let p_pos = rsave
        .world
        .entities
        .actor_location(rsave.world.player_id)
        .unwrap()
        .1;
    let rust_pos = open_floor_neighbors(&rsave, main1, p_pos)[0].1;
    let rust = spawn_monster_with_faction(
        &mut rsave,
        MonsterKind::RustMonster,
        Faction::Hostile,
        main1,
        rust_pos,
    );
    rsave
        .world
        .entities
        .actor_stats_mut(rust)
        .unwrap()
        .hit_bonus = 25;
    let mail = rsave
        .world
        .entities
        .spawn_item(
            ItemKind::ArmorChainMail,
            EntityLocation::Inventory {
                owner: rsave.world.player_id,
            },
        )
        .unwrap();
    let letter = rsave
        .world
        .inventory
        .add_existing_with_next_letter(mail)
        .unwrap();
    rsave.world.entities.set_item_letter(mail, letter);
    let mut rsession = GameSession::from_save_data(rsave).unwrap();
    assert!(rsession.submit(C::Wear { item: mail }).accepted);
    assert!(
        rsession
            .world()
            .entities()
            .get(mail)
            .unwrap()
            .item_erosion()
            .unwrap_or(0)
            >= 1,
        "RustMonster must corrode equipped iron chain mail"
    );

    // 4. BlackPudding splits on melee hit
    let mut bsave = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut bsave, LevelId::main(6));
    let p_pos = bsave
        .world
        .entities
        .actor_location(bsave.world.player_id)
        .unwrap()
        .1;
    let (b_dir, b_pos) = open_floor_neighbors(&bsave, main1, p_pos)[0];
    let pudding = spawn_monster_with_faction(
        &mut bsave,
        MonsterKind::BlackPudding,
        Faction::Hostile,
        main1,
        b_pos,
    );
    bsave.world.entities.actor_stats_mut(pudding).unwrap().hp = 20;
    bsave.world.entities.actor_stats_mut(pudding).unwrap().ac = 10;
    let mut bsession = GameSession::from_save_data(bsave).unwrap();
    for _ in 0..15 {
        let count = bsession
            .world()
            .entities()
            .entities()
            .iter()
            .filter(|e| {
                matches!(
                    e.actor(),
                    Some((ActorKind::Monster(MonsterKind::BlackPudding), _, l, _, _, true)) if l == main1
                )
            })
            .count();
        if count >= 2 {
            break;
        }
        assert!(bsession.submit(C::Move(b_dir)).accepted);
    }
    let pudding_count = bsession
        .world()
        .entities()
        .entities()
        .iter()
        .filter(|e| {
            matches!(
                e.actor(),
                Some((ActorKind::Monster(MonsterKind::BlackPudding), _, l, _, _, true)) if l == main1
            )
        })
        .count();
    assert!(pudding_count >= 2, "BlackPudding must split into 2 on hit");

    // 5. Medusa gaze blocked by Blindfold, hits when unblinded
    let mut msave = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut msave, LevelId::main(6));
    let p_pos = msave
        .world
        .entities
        .actor_location(msave.world.player_id)
        .unwrap()
        .1;
    let map1 = msave.world.levels.map(main1).unwrap().clone();
    let medusa_pos = Direction::ALL
        .into_iter()
        .find_map(|d| {
            let p1 = p_pos.offset(d.delta());
            let p2 = p1.offset(d.delta());
            (map1.tile(p1).ok() == Some(TileKind::Floor)
                && map1.tile(p2).ok() == Some(TileKind::Floor))
            .then_some(p2)
        })
        .unwrap();
    let _medusa = spawn_monster_with_faction(
        &mut msave,
        MonsterKind::Medusa,
        Faction::Hostile,
        main1,
        medusa_pos,
    );
    let blindfold = msave
        .world
        .entities
        .spawn_item(
            ItemKind::Blindfold,
            EntityLocation::Inventory {
                owner: msave.world.player_id,
            },
        )
        .unwrap();
    let bletter = msave
        .world
        .inventory
        .add_existing_with_next_letter(blindfold)
        .unwrap();
    msave.world.entities.set_item_letter(blindfold, bletter);
    msave.world.entities.set_item_tool_active(blindfold, true);
    let mut blind_session = GameSession::from_save_data(msave.clone()).unwrap();
    let hp_before = blind_session.observation().player.hp;
    assert!(blind_session.submit(C::Wait).accepted);
    assert_eq!(blind_session.observation().player.paralysis_turns, 0);
    assert_eq!(blind_session.observation().player.hp, hp_before);

    // Without blindfold, Medusa's gaze paralyzes and deals damage
    msave.world.entities.set_item_tool_active(blindfold, false);
    let mut unblind_session = GameSession::from_save_data(msave).unwrap();
    let hp_before = unblind_session.observation().player.hp;
    assert!(unblind_session.submit(C::Wait).accepted);
    assert!(unblind_session.observation().player.paralysis_turns > 0);
    assert!(unblind_session.observation().player.hp < hp_before);

    // 6. Orc vs Dwarf inter-monster faction warfare
    let mut fsave = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut fsave, LevelId::main(6));
    let p_pos = fsave
        .world
        .entities
        .actor_location(fsave.world.player_id)
        .unwrap()
        .1;
    let map1 = fsave.world.levels.map(main1).unwrap().clone();
    let (orc_p, dwarf_p) = (1..map1.height - 1)
        .flat_map(|y| (1..map1.width - 1).map(move |x| Pos { x, y }))
        .find_map(|p| {
            let p2 = Pos { x: p.x + 1, y: p.y };
            (p.chebyshev_distance(p_pos) >= 8
                && map1.tile(p).ok() == Some(TileKind::Floor)
                && map1.tile(p2).ok() == Some(TileKind::Floor))
            .then_some((p, p2))
        })
        .unwrap();
    let orc =
        spawn_monster_with_faction(&mut fsave, MonsterKind::Orc, Faction::Hostile, main1, orc_p);
    let dwarf = spawn_monster_with_faction(
        &mut fsave,
        MonsterKind::Dwarf,
        Faction::Hostile,
        main1,
        dwarf_p,
    );
    fsave.world.entities.actor_stats_mut(orc).unwrap().hit_bonus = 20;
    fsave.world.entities.actor_stats_mut(dwarf).unwrap().hp = 2;
    fsave.world.entities.actor_stats_mut(dwarf).unwrap().ac = 10;
    let mut fsession = GameSession::from_save_data(fsave).unwrap();
    assert!(fsession.submit(C::Wait).accepted);
    assert!(
        !fsession
            .world()
            .entities()
            .get(dwarf)
            .unwrap()
            .is_alive_actor(),
        "Orc must attack and kill adjacent rival Dwarf in faction warfare"
    );

    // 7. OrcCaptain summons Orc allies
    let mut sum_save =
        start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful).to_save_data();
    park_all_monsters_on(&mut sum_save, LevelId::main(6));
    let p_pos = sum_save
        .world
        .entities
        .actor_location(sum_save.world.player_id)
        .unwrap()
        .1;
    let cap_pos = open_floor_neighbors(&sum_save, main1, p_pos)[0].1;
    let _cap = spawn_monster_with_faction(
        &mut sum_save,
        MonsterKind::OrcCaptain,
        Faction::Hostile,
        main1,
        cap_pos,
    );
    let mut sum_session = GameSession::from_save_data(sum_save).unwrap();
    let mut summoned = false;
    for _ in 0..12 {
        assert!(sum_session.submit(C::Wait).accepted);
        if sum_session.world().entities().entities().iter().any(|e| {
            matches!(
                e.actor(),
                Some((ActorKind::Monster(MonsterKind::Orc), Faction::Hostile, l, _, _, true))
                    if l == main1
            )
        }) {
            summoned = true;
            break;
        }
    }
    assert!(summoned, "OrcCaptain must summon Orc reinforcement");
}

#[test]
fn p11_6_save_v3_tamper_legacy_v1_isolation_tui_headless_and_llm_budget() {
    // 1. SaveV3 roundtrip with pet/NPC/debt preserves exact snapshot hash
    let mut session = start_campaign(42, Role::Knight, Race::Human, Alignment::Lawful);
    let save = session.to_save_data();
    let restored = GameSession::from_save_data(save.clone()).unwrap();
    assert_eq!(
        session.snapshot().stable_hash(),
        restored.snapshot().stable_hash()
    );

    // 2. SaveV3 tamper rejection
    let mut bad_tame = save.clone();
    let pet_id = bad_tame
        .world
        .entities
        .entities()
        .iter()
        .find(|e| e.faction() == Some(Faction::Pet))
        .unwrap()
        .id;
    bad_tame.world.entities.set_actor_tameness(pet_id, 20);
    // Tamper via JSON to exceed MAX_PET_TAMENESS (20)
    let raw = serde_json::to_string(&bad_tame)
        .unwrap()
        .replace("\"tameness\":20", "\"tameness\":25");
    let parsed: aihack_runtime::save::SaveDataV1 = serde_json::from_str(&raw).unwrap();
    assert!(GameSession::from_save_data(parsed).is_err());

    let mut bad_debt = save.clone();
    bad_debt.world.shop_debt = 2_000_000;
    assert!(GameSession::from_save_data(bad_debt).is_err());

    let mut bad_unpaid_amulet = save.clone();
    let amulet_id = bad_unpaid_amulet.world.campaign.unwrap().amulet;
    bad_unpaid_amulet
        .world
        .entities
        .set_item_unpaid(amulet_id, true);
    assert!(GameSession::from_save_data(bad_unpaid_amulet).is_err());

    // Legacy V1 rejects P11 fields
    let mut legacy = GameSession::try_new(42).unwrap();
    assert!(legacy.submit(C::Wait).accepted);
    assert!(legacy.submit(C::Wait).accepted);
    let mut legacy_save = legacy.to_save_data();
    assert_eq!(legacy_save.schema_version, 1);
    legacy_save.world.shop_debt = 10;
    assert!(GameSession::from_save_data(legacy_save).is_err());

    // 3. LLM observation budget (< 32,768 bytes) & ActionSpace <= 64
    let obs = session.observation();
    assert!(obs.action_space.commands.len() <= 64);
    assert!(obs
        .visible_entities
        .iter()
        .any(|e| e.faction == Some(Faction::Pet)));
    let llm_view = LlmObservationView::from(&obs);
    let bytes = serde_json::to_vec(&llm_view).unwrap().len();
    assert!(
        bytes < 32_768,
        "LlmObservationView exceeded 32 KiB: {bytes}"
    );

    // 4. TUI key bindings & HUD debt/faction inspect
    assert_eq!(
        aihack_tui::tui::input::key_to_candidate('m', &obs),
        Some(aihack_tui::tui::input::UiCommandCandidate::BeginAction('m'))
    );
    let pet_pos = obs
        .visible_entities
        .iter()
        .find(|e| e.faction == Some(Faction::Pet))
        .unwrap()
        .pos;
    let inspect = aihack_tui::tui::render_panels::inspect_lines(
        &obs,
        Some(pet_pos),
        aihack_tui::tui::UiPanel::Map,
        &[],
    );
    assert!(inspect.iter().any(|line| line.contains("[pet]")));

    // 5. Headless survival-v1 does NOT attack adjacent Pet and survives 20 campaign turns
    let pet_entity_id = session
        .world()
        .entities()
        .entities()
        .iter()
        .find(|e| e.faction() == Some(Faction::Pet))
        .unwrap()
        .id;
    let report = run_to_turn(&mut session, 20, HeadlessPolicy::SurvivalV1).unwrap();
    assert_eq!(report.accepted_turns, 20);
    assert_eq!(
        session
            .world()
            .entities()
            .get(pet_entity_id)
            .unwrap()
            .faction(),
        Some(Faction::Pet),
        "Headless survival-v1 must never turn starting pet hostile"
    );
}
