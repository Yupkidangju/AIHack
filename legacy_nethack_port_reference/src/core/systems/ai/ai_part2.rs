use crate::assets::AssetManager;
use crate::core::dungeon::Grid;
use crate::core::entity::monster::MonsterFaction;
use crate::core::entity::status::StatusFlags;
use crate::core::entity::{
    monster::MonsterState, status::StatusBundle, status::Swallowed, CombatStats, Health, Level,
    Monster, MonsterTag, PlayerTag, Position,
};
use crate::core::systems::ai::ai_helper::{AiHelper, MoveFlags};
use crate::core::systems::combat::CombatEngine;
use crate::core::systems::world::vision::VisionSystem;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::{EntityStore, SubWorld};
use legion::*;
use std::collections::{HashMap, HashSet};

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
pub fn monster_ai(
    world: &mut SubWorld,
    #[resource] grid: &mut Grid,
    #[resource] rng: &mut NetHackRng,
    #[resource] assets: &AssetManager,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] vision: &mut VisionSystem,
    #[resource] p_status_resists: &HashSet<StatusFlags>,
    command_buffer: &mut CommandBuffer,
) {
    let (p_entity, p_pos, p_status_flags) = <(Entity, &Position, &StatusBundle)>::query()
        .filter(component::<PlayerTag>())
        .iter(world)
        .next()
        .map(|(e, p, s)| (*e, *p, s.flags.clone()))
        .unwrap_or((Entity::new(0, 0), Position { x: 0, y: 0 }, HashSet::new()));

    let p_level = <&Level>::query()
        .filter(component::<PlayerTag>())
        .iter(world)
        .next()
        .map(|l| *l)
        .unwrap_or(crate::core::entity::Level(1));

    vision.compute(grid, &p_pos, 10);

    let mut occupancy = HashSet::new();
    let mut obstacles = HashSet::new();
    let mut trap_positions = HashSet::new();
    for (pos, _) in <(&Position, &MonsterTag)>::query().iter(world) {
        occupancy.insert((pos.x, pos.y));
        obstacles.insert((pos.x, pos.y));
    }

    let mut item_positions: HashMap<(i32, i32), Entity> = HashMap::new();
    for (ent, pos, _) in <(Entity, &Position, &crate::core::entity::ItemTag)>::query().iter(world) {
        item_positions.insert((pos.x, pos.y), *ent);
    }

    let mut faction_map = HashMap::new();
    let mut monster_data_map = HashMap::new();
    for (ent, fac, m, pos) in <(Entity, &MonsterFaction, &Monster, &Position)>::query().iter(world)
    {
        faction_map.insert(*ent, (fac.faction, m.hostile, false, 0));
        monster_data_map.insert(*ent, (*ent, *pos, fac.faction, m.hostile));
    }

    let mut attacks_to_apply = Vec::new();
    let mut pickups_to_apply = Vec::new();
    let mut mhitm_to_apply = Vec::new();
    let mut nearby_passive_to_apply = Vec::new();

    let mut query = <(
        Entity,
        &mut Position,
        &mut MonsterState,
        &Monster,
        &MonsterFaction,
    )>::query();

    // Main Monster Loop
    for (m_entity, m_pos, m_state, monster, m_faction) in query.iter_mut(world) {
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

        // Attack Player Logic
        // Logic: if next to player, attack with all moves
        if dist_sq <= 2 {
            // Adjacent including diagonals
            if let Some(template) = assets.monsters.get_by_kind(monster.kind) {
                for attack in &template.attacks {
                    // Check range attack vs melee? NetHack mixes them but mostly melee here
                    // For now assume all are melee valid if adjacent
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
    let mut current_swallower: Option<Entity> = if let Ok(entry) = world.entry_ref(p_entity) {
        entry.get_component::<Swallowed>().map(|s| s.by).ok()
    } else {
        None
    };

    for (m_ent, m_name, attack, _, target_ac) in attacks_to_apply {
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
        }

        if let Ok(mut p_entry) = world.entry_mut(p_entity) {
            if let Ok(p_status) = p_entry.get_component_mut::<StatusBundle>() {
                if !resisted {
                    match attack.adtype {
                        crate::core::entity::monster::DamageType::Drst => {
                            if rng.rn2(20) > 10 {
                                p_status.add(StatusFlags::POISONED, 30);
                                log.add_colored("You feel poisoned!", [0, 255, 0], *turn);
                            }
                        }
                        // ... (Other cases simplified for length, add critical ones)
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
