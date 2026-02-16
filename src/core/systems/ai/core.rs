// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::assets::AssetManager;
use crate::core::dungeon::Grid;
use crate::core::entity::status::StatusFlags;
use crate::core::entity::{
    monster::MonsterState, status::StatusBundle, status::Swallowed, CombatStats, Health, Level,
    Monster, MonsterTag, PlayerTag, Position,
};
use crate::core::events::{EventQueue, GameEvent}; // [v2.0.0 R5] 이벤트 발행
use crate::core::systems::ai_helper::{AiHelper, MoveFlags};
use crate::core::systems::combat::CombatEngine;
use crate::core::systems::vision::VisionSystem;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::{EntityStore, SubWorld};
use legion::*;
use std::collections::{HashMap, HashSet};

///
#[system]
#[read_component(MonsterTag)]
#[write_component(crate::core::entity::monster::Pet)]
pub fn pet_hunger(
    world: &mut SubWorld,
    #[resource] rng: &mut NetHackRng,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
) {
    let mut query = <(&mut crate::core::entity::monster::Pet, &MonsterTag)>::query();
    for (pet, _) in query.iter_mut(world) {
        if *turn % 10 == 0 {
            pet.hunger += 1;
            if pet.hunger > 1000 && rng.rn2(20) == 0 {
                log.add("Your pet whines.", *turn);
            }
        }
    }
}
use crate::core::entity::monster::MonsterFaction;

#[system]
#[read_component(PlayerTag)]
#[read_component(Position)]
#[read_component(CombatStats)]
#[write_component(Health)]
#[write_component(StatusBundle)]
#[write_component(crate::core::entity::Inventory)]
#[read_component(crate::core::entity::Item)]
#[read_component(crate::core::entity::ItemTag)]
#[read_component(Swallowed)]
#[read_component(MonsterTag)]
#[read_component(Monster)]
#[write_component(MonsterState)]
#[read_component(MonsterFaction)]
#[read_component(crate::core::entity::monster::Pet)]
#[read_component(crate::core::entity::Structure)]
#[read_component(crate::core::entity::StructureTag)]
#[read_component(Level)]
pub fn monster_ai(
    world: &mut SubWorld,
    #[resource] grid: &mut Grid,
    #[resource] rng: &mut NetHackRng,
    #[resource] assets: &AssetManager,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] vision: &mut VisionSystem,
    #[resource] p_status_resists: &StatusFlags,
    #[resource] event_queue: &mut EventQueue, // [v2.0.0 R5] 몬스터 공격 이벤트
    command_buffer: &mut CommandBuffer,
) {
    //
    let player_data = <(Entity, &Position, &StatusBundle)>::query()
        .filter(component::<PlayerTag>())
        .iter(world)
        .next()
        .map(|(e, p, s)| (*e, *p, s.flags().clone()));

    let (p_entity, p_pos, p_status_flags) = match player_data {
        Some(data) => data,
        None => return, // 플레이어가 없으면 몬스터 AI 실행 불필요
    };

    // [v2.0.0
    let p_level = <&Level>::query()
        .filter(component::<PlayerTag>())
        .iter(world)
        .next()
        .map(|l| *l)
        .unwrap_or(crate::core::entity::Level(crate::core::dungeon::LevelID {
            branch: crate::core::dungeon::DungeonBranch::Main,
            depth: 1,
        }));

    if p_pos.x >= 0 && p_pos.y >= 0 {
        vision.recalc(grid, p_pos.x as usize, p_pos.y as usize, 10);
    }

    // [v2.0.0
    let occupancy: HashSet<(i32, i32)> = <(&Position, &MonsterTag, &Level)>::query()
        .iter(world)
        .filter(|(_, _, lvl)| lvl.0 == p_level.0)
        .map(|(pos, _, _)| (pos.x, pos.y))
        .collect();
    let obstacles = occupancy.clone();
    let trap_positions: HashSet<(i32, i32)> = HashSet::new(); // TODO: 트랩 위치 수집

    let _item_positions: HashMap<(i32, i32), Entity> =
        <(Entity, &Position, &crate::core::entity::ItemTag, &Level)>::query()
            .iter(world)
            .filter(|(_, _, _, lvl)| lvl.0 == p_level.0)
            .map(|(ent, pos, _, _)| ((pos.x, pos.y), *ent))
            .collect();

    let mut faction_map = HashMap::new();
    let mut monster_data_map = HashMap::new();
    for (ent, fac, m, pos) in <(Entity, &MonsterFaction, &Monster, &Position)>::query().iter(world)
    {
        faction_map.insert(*ent, (fac.faction, m.hostile, false, 0));
        monster_data_map.insert(*ent, (*ent, *pos, fac.faction, m.hostile));
    }

    let mut attacks_to_apply = Vec::new();
    let mut pickups_to_apply = Vec::new();
    let _mhitm_to_apply: Vec<(Entity, Entity)> = Vec::new(); // 미구현 (향후 몬스터 간 공격용)
    let mut nearby_passive_to_apply = Vec::new();

    let mut query = <(
        Entity,
        &mut Position,
        &mut MonsterState,
        &Monster,
        &MonsterFaction,
        &Level,
    )>::query();

    // Main Monster Loop
    for (m_entity, m_pos, m_state, monster, m_faction, m_level) in query.iter_mut(world) {
        // [v2.0.0
        if m_level.0 != p_level.0 {
            continue;
        }

        // [v2.0.0] 속도 기반 행동 판정 (원본 monmove.c:mcalcmove)
        let m_speed = assets
            .monsters
            .get_by_kind(monster.kind)
            .map(|t| t.movement as i32)
            .unwrap_or(12);
        if !crate::core::systems::monmove::can_act_this_turn(m_speed, *turn) {
            continue;
        }

        let dist_sq = (m_pos.x - p_pos.x).pow(2) + (m_pos.y - p_pos.y).pow(2);
        let m_is_pet = false; // Simplified for now, check component if needed

        // Wake up logic
        let wake_chance = if m_state.msleeping { 10 } else { 0 };
        if m_state.msleeping {
            if dist_sq < 36 && rng.rn2(wake_chance + 1) == 0 {
                m_state.msleeping = false;
                if rng.rn2(3) == 0 {
                    log.add(format!("The {} wakes up!", monster.kind), *turn);
                }
            } else {
                continue;
            }
        }

        // Flee logic (Phase 49.2)
        if m_state.mflee {
            if m_state.mfleetim > 0 {
                m_state.mfleetim -= 1;
                move_away(
                    grid,
                    m_pos,
                    &p_pos,
                    monster,
                    assets,
                    rng,
                    &occupancy,
                    &obstacles,
                    &trap_positions,
                    m_is_pet,
                );
                continue;
            } else {
                m_state.mflee = false;
            }
        }

        let mut attached_attack = false;

        // [v2.0.0
        let is_hostile = monster.hostile;

        // Attack Player Logic
        // Logic: if next to player AND hostile, attack with all moves
        if dist_sq <= 2 && is_hostile {
            //
            if let Some(template) = assets.monsters.get_by_kind(monster.kind) {
                for attack in &template.attacks {
                    attacks_to_apply.push((
                        *m_entity,
                        monster.kind,
                        attack.clone(),
                        m_faction.faction,
                        10,
                    )); // AC 10 placeholder
                    attached_attack = true;
                }
                // Passive touch check
                nearby_passive_to_apply.push(*m_entity);
            }
        }

        if !attached_attack {
            // Movement Logic
            if dist_sq < 100
                && VisionSystem::has_line_of_sight(
                    grid,
                    m_pos.x as usize,
                    m_pos.y as usize,
                    p_pos.x as usize,
                    p_pos.y as usize,
                )
            {
                // Move towards player
                move_towards(
                    grid,
                    m_pos,
                    &p_pos,
                    monster,
                    assets,
                    rng,
                    &occupancy,
                    &obstacles,
                    &trap_positions,
                    m_is_pet,
                );
            } else {
                move_random(
                    grid,
                    m_pos,
                    &p_pos,
                    monster,
                    assets,
                    rng,
                    &occupancy,
                    &obstacles,
                    &trap_positions,
                    m_is_pet,
                );
            }
        }

        // Social / Chat (Partial)
        // ... (Skipping for brevity in reconstruction, focus on core)
    }

    // Attacks Application Pass (The Fix)
    let current_swallower: Option<Entity> = if let Ok(entry) = world.entry_ref(p_entity) {
        entry.get_component::<Swallowed>().map(|s| s.by).ok()
    } else {
        None
    };

    for (m_ent, m_name, attack, _, target_ac) in attacks_to_apply {
        // [v2.0.0] 공격 메시지 출력 (원본 mhitu.c:mattacku)
        log.add(format!("The {} attacks you!", m_name), *turn);
        if let Some(swallower) = current_swallower {
            if swallower != m_ent {
                continue;
            }
        }

        let dmg = CombatEngine::calculate_monster_damage(rng, &attack, target_ac, &p_status_flags);

        // --- NEW MHITU LOGIC START ---
        // 데미지 및 저항력 체크 (NetHack mhitu.c)
        let mut final_dmg = dmg;
        let mut resisted = false;

        match attack.adtype {
            crate::core::entity::monster::DamageType::Fire => {
                if p_status_resists.contains(StatusFlags::FIRE_RES) {
                    resisted = true;
                    log.add("You shake off the fire.", *turn);
                }
            }
            crate::core::entity::monster::DamageType::Cold => {
                if p_status_resists.contains(StatusFlags::COLD_RES) {
                    resisted = true;
                    log.add("You feel cold but unhurt.", *turn);
                }
            }
            crate::core::entity::monster::DamageType::Elec => {
                if p_status_resists.contains(StatusFlags::SHOCK_RES) {
                    resisted = true;
                    log.add("You absorb the shock.", *turn);
                }
            }
            crate::core::entity::monster::DamageType::Acid => {
                if p_status_resists.contains(StatusFlags::ACID_RES) {
                    resisted = true;
                }
            }
            crate::core::entity::monster::DamageType::Slee => {
                if p_status_resists.contains(StatusFlags::SLEEP_RES) {
                    resisted = true;
                }
            }
            crate::core::entity::monster::DamageType::Drst => {
                // Poison
                if p_status_resists.contains(StatusFlags::POISON_RES) {
                    resisted = true;
                }
            }
            _ => {}
        }

        if resisted {
            final_dmg = rng.d(1, 4);
        }

        if final_dmg > 0 {
            if let Ok(mut p_entry) = world.entry_mut(p_entity) {
                if let Ok(p_health) = p_entry.get_component_mut::<Health>() {
                    p_health.current -= final_dmg;
                }
            }

            // [v2.0.0
            event_queue.push(GameEvent::DamageDealt {
                attacker: m_name.to_string(),
                defender: "player".to_string(),
                amount: final_dmg,
                source: format!("{:?}", attack.adtype),
            });
        }

        if let Ok(mut p_entry) = world.entry_mut(p_entity) {
            if let Ok(p_status) = p_entry.get_component_mut::<StatusBundle>() {
                if !resisted {
                    match attack.adtype {
                        crate::core::entity::monster::DamageType::Drst => {
                            if rng.rn2(20) > 10 {
                                p_status.add(StatusFlags::POISONED, 30);
                                log.add_colored("You feel poisoned!", [0, 255, 0], *turn);

                                // [v2.0.0 R5] 독 상태 적용 이벤트
                                event_queue.push(GameEvent::StatusApplied {
                                    target: "player".to_string(),
                                    status: StatusFlags::POISONED,
                                    turns: 30,
                                });
                            }
                        }
                        // ... (Other cases simplified for length, add critical ones)
                        // [v2.9.3] 생명력 흡수: mhitu.rs 유틸 활용
                        crate::core::entity::monster::DamageType::Drli => {
                            if rng.rn2(10) == 0 {
                                log.add_colored(
                                    "You feel your life force draining away...",
                                    [100, 0, 100],
                                    *turn,
                                );
                                if let Ok(h) = p_entry.get_component_mut::<Health>() {
                                    h.max = (h.max - 1).max(1);
                                    h.current = h.current.min(h.max);
                                }
                            }
                        }
                        // [v2.9.3] 절도 공격: mhitu.rs steal_check ECS 연결
                        crate::core::entity::monster::DamageType::Sgld
                        | crate::core::entity::monster::DamageType::Sitm => {
                            // 인벤토리 크기 확인 (골드는 향후 별도 컴포넌트로 관리)
                            let (inv_size, has_gold) = {
                                if let Ok(inv) =
                                    p_entry.get_component::<crate::core::entity::Inventory>()
                                {
                                    (inv.items.len(), false) // TODO: 골드 컴포넌트 연결
                                } else {
                                    (0, false)
                                }
                            };
                            let m_type = match attack.adtype {
                                crate::core::entity::monster::DamageType::Sgld => "leprechaun",
                                _ => "nymph",
                            };
                            let theft = crate::core::systems::combat::mhitu::steal_check(
                                &m_name.to_string(),
                                m_type,
                                10,
                                inv_size,
                                has_gold,
                                rng,
                            );
                            if theft.stolen {
                                log.add_colored(&theft.message, [255, 200, 0], *turn);
                            } else {
                                log.add(&theft.message, *turn);
                            }
                        }
                        // [v2.9.3] 녹 공격: mhitu.rs rust_attack_effect ECS 연결
                        crate::core::entity::monster::DamageType::Rust => {
                            let rust_result =
                                crate::core::systems::combat::mhitu::rust_attack_effect(
                                    "armor", 0, false,
                                );
                            match rust_result {
                                crate::core::systems::combat::mhitu::RustResult::Eroded {
                                    slot_name,
                                    new_level,
                                } => {
                                    log.add_colored(
                                        &format!(
                                            "Your {} is damaged! (erosion {})",
                                            slot_name, new_level
                                        ),
                                        [200, 100, 0],
                                        *turn,
                                    );
                                }
                                crate::core::systems::combat::mhitu::RustResult::Destroyed {
                                    slot_name,
                                } => {
                                    log.add_colored(
                                        &format!("Your {} crumbles to dust!", slot_name),
                                        [255, 0, 0],
                                        *turn,
                                    );
                                }
                                crate::core::systems::combat::mhitu::RustResult::NoEffect => {}
                            }
                        }
                        // [v2.9.3] 라이칸스로피 공격: mhitu.rs lycanthropy_attack ECS 연결
                        crate::core::entity::monster::DamageType::Were => {
                            let effect = crate::core::systems::combat::mhitu::lycanthropy_attack(
                                &m_name.to_string(),
                                "werewolf",
                                false,
                                rng,
                                log,
                                *turn,
                            );
                            if effect.is_some() {
                                if let Ok(p_status) = p_entry.get_component_mut::<StatusBundle>() {
                                    p_status.add(StatusFlags::LYCANTHROPY, 200);
                                }
                            }
                        }
                        // [v2.9.3] 질병 공격: mhitu.rs disease_attack ECS 연결
                        crate::core::entity::monster::DamageType::Dise => {
                            let dummy_atk = crate::core::entity::monster::Attack {
                                atype: attack.atype,
                                adtype: attack.adtype,
                                dice: attack.dice,
                                sides: attack.sides,
                            };
                            let result = crate::core::systems::combat::mhitu::disease_attack(
                                &dummy_atk,
                                &m_name.to_string(),
                                false,
                                12,
                                rng,
                                log,
                                *turn,
                            );
                            if let Some(
                                crate::core::systems::combat::mhitu::StatusEffect::Disease,
                            ) = result.status_effect
                            {
                                if let Ok(p_status) = p_entry.get_component_mut::<StatusBundle>() {
                                    p_status.add(StatusFlags::SICK, 100);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // --- NEW MHITU LOGIC END ---
    }

    // 6. Pickup Pass
    for (m_ent, item_ent) in pickups_to_apply {
        if let Ok(mut entry) = world.entry_mut(m_ent) {
            if let Ok(inv) = entry.get_component_mut::<crate::core::entity::Inventory>() {
                inv.items.push(item_ent);
                command_buffer.remove_component::<Position>(item_ent);
            }
        }
    }
}

#[allow(dead_code)] // 향후 AI 고도화 시 사용 예정
fn can_see_pos(vision: &VisionSystem, pos: &Position) -> bool {
    let ux = pos.x as usize;
    let uy = pos.y as usize;
    if ux < crate::core::dungeon::COLNO && uy < crate::core::dungeon::ROWNO {
        vision.viz_array[ux][uy] & crate::core::systems::vision::IN_SIGHT != 0
    } else {
        false
    }
}

fn move_towards(
    grid: &mut Grid,
    m_pos: &mut Position,
    p_pos: &Position,
    monster: &Monster,
    assets: &AssetManager,
    rng: &mut NetHackRng,
    occupancy: &HashSet<(i32, i32)>,
    obstacles: &HashSet<(i32, i32)>,
    trap_positions: &HashSet<(i32, i32)>,
    is_pet: bool,
) {
    if let Some(template) = assets.monsters.get_by_kind(monster.kind) {
        let mut flags = MoveFlags::empty();
        if !template.has_capability(crate::core::entity::capability::MonsterCapability::Animal) {
            flags |= MoveFlags::OPENDOOR;
        }

        let candidates = AiHelper::mfndpos(
            grid,
            m_pos.x as usize,
            m_pos.y as usize,
            template,
            flags,
            occupancy,
            obstacles,
        );

        if candidates.is_empty() {
            return;
        }

        let mut best_pos = None;
        if let Some(path) = AiHelper::get_path(
            grid,
            (m_pos.x, m_pos.y),
            (p_pos.x, p_pos.y),
            template,
            flags,
            occupancy,
            obstacles,
        ) {
            if path.len() > 1 {
                let next = path[1];
                if !is_pet || !trap_positions.contains(&(next.0, next.1)) {
                    best_pos = Some(crate::core::entity::Position {
                        x: next.0,
                        y: next.1,
                    });
                }
            }
        }

        let best_pos = if let Some(bp) = best_pos {
            bp
        } else {
            let mut greedy_pos = *m_pos;
            let mut min_dist = (m_pos.x - p_pos.x).pow(2) + (m_pos.y - p_pos.y).pow(2);
            for pos in candidates {
                if pos.x == p_pos.x && pos.y == p_pos.y {
                    continue;
                }
                if is_pet && trap_positions.contains(&(pos.x, pos.y)) {
                    continue;
                }
                let dist = (pos.x - p_pos.x).pow(2) + (pos.y - p_pos.y).pow(2);
                if dist < min_dist {
                    min_dist = dist;
                    greedy_pos = pos;
                } else if dist == min_dist && rng.rn2(2) == 0 {
                    greedy_pos = pos;
                }
            }
            greedy_pos
        };

        if let Some(tile) = grid.get_tile_mut(best_pos.x as usize, best_pos.y as usize) {
            if tile.typ == crate::core::dungeon::tile::TileType::Door {
                tile.typ = crate::core::dungeon::tile::TileType::OpenDoor;
            }
        }
        m_pos.x = best_pos.x;
        m_pos.y = best_pos.y;
    }
}

fn move_random(
    grid: &mut Grid,
    m_pos: &mut Position,
    _p_pos: &Position,
    monster: &Monster,
    assets: &AssetManager,
    rng: &mut NetHackRng,
    occupancy: &HashSet<(i32, i32)>,
    obstacles: &HashSet<(i32, i32)>,
    trap_positions: &HashSet<(i32, i32)>,
    is_pet: bool,
) {
    if let Some(template) = assets.monsters.get_by_kind(monster.kind) {
        // [v2.0.0
        let mut flags = MoveFlags::empty();
        if !template.has_capability(crate::core::entity::capability::MonsterCapability::Animal) {
            flags |= MoveFlags::OPENDOOR;
        }
        let candidates = AiHelper::mfndpos(
            grid,
            m_pos.x as usize,
            m_pos.y as usize,
            template,
            flags,
            occupancy,
            obstacles,
        );
        if candidates.is_empty() {
            return;
        }

        let filtered: Vec<_> = candidates
            .into_iter()
            .filter(|pos| !is_pet || !trap_positions.contains(&(pos.x, pos.y)))
            .collect();

        if filtered.is_empty() {
            return;
        }
        let idx = rng.rn2(filtered.len() as i32) as usize;
        let next_pos = filtered[idx];

        if let Some(tile) = grid.get_tile_mut(next_pos.x as usize, next_pos.y as usize) {
            if tile.typ == crate::core::dungeon::tile::TileType::Door {
                tile.typ = crate::core::dungeon::tile::TileType::OpenDoor;
            }
        }
        m_pos.x = next_pos.x;
        m_pos.y = next_pos.y;
    }
}

fn move_away(
    grid: &mut Grid,
    m_pos: &mut Position,
    p_pos: &Position,
    monster: &Monster,
    assets: &AssetManager,
    rng: &mut NetHackRng,
    occupancy: &HashSet<(i32, i32)>,
    obstacles: &HashSet<(i32, i32)>,
    trap_positions: &HashSet<(i32, i32)>,
    is_pet: bool,
) {
    // Similar to move_towards but maximizing distance
    // (Simplified implementation for brevity)
    move_random(
        grid,
        m_pos,
        p_pos,
        monster,
        assets,
        rng,
        occupancy,
        obstacles,
        trap_positions,
        is_pet,
    );
}
