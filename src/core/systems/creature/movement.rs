// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::assets::AssetManager;
use crate::core::dungeon::tile::TileType;
use crate::core::dungeon::{Grid, COLNO, ROWNO};
use crate::core::entity::monster::MonsterState;
use crate::core::entity::player::Player;
use crate::core::entity::skills::{SkillLevel, WeaponSkill};
use crate::core::entity::status::{StatusBundle, StatusFlags};
use crate::core::entity::{
    CombatStats, Equipment, EquipmentSlot, Health, Inventory, Item, Level, Monster, MonsterTag,
    PlayerTag, Position, Renderable, Species, Talkative,
};
use crate::core::events::{EventQueue, GameEvent}; // [v2.0.0 R5] 이벤트 발행
use crate::core::systems::combat::CombatEngine;
use crate::ui::input::Command;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

/// 엔티티 이동 및 충돌 판정 시스템
#[legion::system]
#[read_component(PlayerTag)]
#[read_component(MonsterTag)]
#[write_component(Position)]
#[read_component(CombatStats)]
#[write_component(Health)]
#[write_component(Player)]
#[read_component(StatusBundle)]
#[read_component(Equipment)]
#[read_component(Monster)]
#[read_component(MonsterState)]
#[read_component(Level)]
#[write_component(Item)]
#[write_component(Inventory)]
#[read_component(crate::core::entity::BoulderTag)]
#[read_component(crate::core::entity::StatueTag)]
#[read_component(crate::core::entity::Trap)]
#[read_component(crate::core::entity::TrapTag)]
#[read_component(crate::core::entity::StructureTag)]
#[write_component(crate::core::entity::Structure)]
pub fn movement(
    world: &mut SubWorld,
    #[resource] grid: &mut Grid,
    #[resource] cmd: &crate::ui::input::Command,
    #[resource] rng: &mut NetHackRng,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] assets: &AssetManager,
    #[resource] event_queue: &mut EventQueue, // [v2.0.0 R5] 전투 이벤트 발행
    command_buffer: &mut CommandBuffer,
) {
    if *cmd == Command::Unknown || *cmd == Command::Wait {
        if *cmd == Command::Wait {
            log.add("Time passes...", *turn);
        }
        return;
    }

    let mut dx = 0;
    let mut dy = 0;

    match cmd {
        Command::MoveN => dy = -1,
        Command::MoveS => dy = 1,
        Command::MoveE => dx = 1,
        Command::MoveW => dx = -1,
        Command::MoveNE => {
            dx = 1;
            dy = -1;
        }
        Command::MoveNW => {
            dx = -1;
            dy = -1;
        }
        Command::MoveSE => {
            dx = 1;
            dy = 1;
        }
        Command::MoveSW => {
            dx = -1;
            dy = 1;
        }
        _ => {}
    }

    if dx == 0 && dy == 0 {
        return;
    }

    //
    let mut player_query = <(
        Entity,
        &Position,
        &CombatStats,
        &Player,
        &StatusBundle,
        Option<&Equipment>,
        &Level,
    )>::query()
    .filter(component::<PlayerTag>());
    let player_info: Vec<(
        Entity,
        Position,
        CombatStats,
        Player,
        StatusBundle,
        Option<Equipment>,
        Level,
    )> = player_query
        .iter(world)
        .map(|(e, p, s, pr, st, eq, l)| (*e, *p, *s, pr.clone(), st.clone(), eq.cloned(), *l))
        .collect();

    // 2. 이동 및 공격 처리
    for (p_ent, p_pos, p_stats, p_player, p_status, p_equip, p_level) in player_info {
        let flags = p_status.flags();

        // 수면 또는 마비 상태 체크
        if flags.intersects(StatusFlags::SLEEPING | StatusFlags::PARALYZED) {
            log.add("You are unable to move!", *turn);
            continue;
        }

        // 함정 구속 체크 (BearTrap, Web)
        let mut restrained = false;
        {
            let mut t_q = <(&Position, &crate::core::entity::Trap, &Level)>::query();
            for (t_pos, t_comp, t_lvl) in t_q.iter(world) {
                if t_pos.x == p_pos.x && t_pos.y == p_pos.y && t_lvl.0 == p_level.0 {
                    use crate::core::entity::TrapType;
                    match t_comp.typ {
                        TrapType::BearTrap => {
                            if rng.rn2(100) < 70 {
                                // 70% 확률로 탈출 불가
                                log.add("You are stuck in a bear trap!", *turn);
                                restrained = true;
                            } else {
                                log.add("You break free from the bear trap!", *turn);
                            }
                        }
                        TrapType::Web => {
                            if rng.rn2(100) < 50 {
                                // 50% 확률로 탈출 불가
                                log.add("You are entangled in a web!", *turn);
                                restrained = true;
                            } else {
                                log.add("You tear through the web!", *turn);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        if restrained {
            continue;
        }

        let (mut move_dx, mut move_dy) = (dx, dy);

        // 혼란 또는 기절 상태 체크 (무작위 이동)
        if flags.intersects(StatusFlags::CONFUSED | StatusFlags::STUNNED) {
            if rng.rn2(5) > 0 {
                // 80% 확률로 무작위 방향
                move_dx = rng.rn2(3) - 1;
                move_dy = rng.rn2(3) - 1;
                if move_dx == 0 && move_dy == 0 {
                    log.add("You stagger around...", *turn);
                    continue;
                }
            }
        }

        let nx = p_pos.x + move_dx;
        let ny = p_pos.y + move_dy;

        if nx < 0 || nx >= COLNO as i32 || ny < 0 || ny >= ROWNO as i32 {
            continue;
        }

        // 50.1 RAID - 구조물 충돌 체크 및 공격
        let mut target_structure = None;
        {
            let mut s_q = <(Entity, &Position, &crate::core::entity::Structure, &Level)>::query()
                .filter(component::<crate::core::entity::StructureTag>());
            for (ent, s_pos, _s, s_lvl) in s_q.iter(world) {
                if s_pos.x == nx && s_pos.y == ny && s_lvl.0 == p_level.0 {
                    target_structure = Some(*ent);
                    break;
                }
            }
        }

        if let Some(s_ent) = target_structure {
            if let Ok(mut entry) = world.entry_mut(s_ent) {
                if let Ok(mut s_comp) = entry.get_component_mut::<crate::core::entity::Structure>()
                {
                    let (dmg, destroyed) = CombatEngine::attack_structure(rng, &mut s_comp);
                    let s_name = match s_comp.typ {
                        crate::core::entity::StructureType::CommBase => "communication antenna",
                        crate::core::entity::StructureType::SupplyDepot => "supply depot",
                    };
                    log.add(format!("You hit the {} for {} damage.", s_name, dmg), *turn);
                    if destroyed {
                        log.add(format!("The {} collapses!", s_name), *turn);
                        // 파괴 처리는 ai.rs의 Structure Destruction Pass에서 처리됨 (integrity <= 0)
                    }
                }
            }
            continue; // 구조물을 타격했으므로 이동 중단
        }

        // 몬스터 충돌 체크 (공격 판정)
        let mut monster_query = <(
            Entity,
            &Position,
            &Monster,
            &CombatStats,
            &MonsterState,
            &Level,
        )>::query()
        .filter(component::<MonsterTag>());
        let target_monster = monster_query
            .iter(world)
            .filter(|(_, pos, _, _, _, lvl)| pos.x == nx && pos.y == ny && lvl.0 == p_level.0)
            .map(|(e, _, m, s, ms, _)| (*e, m.kind.to_string(), *s, ms.clone(), m.clone()))
            .next();

        if let Some((m_ent, m_tmpl_name, m_stats, m_state, m_comp)) = target_monster {
            let m_display_name = if let Some(name) = &m_comp.mon_name {
                name.clone()
            } else {
                m_tmpl_name.clone()
            };
            // 공격 실행
            let m_tmpl = assets.monsters.templates.get(&m_tmpl_name);

            if let Some(m_tmpl_ptr) = m_tmpl {
                // 공격 정보 수집
                let mut attacks = Vec::new();

                // 1. 주무기
                let mut primary_weapon = None;
                if let Some(we_ent) = p_equip
                    .as_ref()
                    .and_then(|eq| eq.slots.get(&EquipmentSlot::Melee))
                {
                    if let Ok(entry) = world.entry_ref(*we_ent) {
                        if let Ok(i) = entry.get_component::<Item>() {
                            if let Some(t) = assets.items.get_by_kind(i.kind) {
                                primary_weapon = Some((i.clone(), t.clone()));
                            }
                        }
                    }
                }
                attacks.push((primary_weapon, false));

                //
                if p_player.two_weapon {
                    if let Some(we_ent) = p_equip
                        .as_ref()
                        .and_then(|eq| eq.slots.get(&EquipmentSlot::Offhand))
                    {
                        if let Ok(entry) = world.entry_ref(*we_ent) {
                            if let Ok(i) = entry.get_component::<Item>() {
                                if let Some(t) = assets.items.get_by_kind(i.kind) {
                                    attacks.push((Some((i.clone(), t.clone())), true));
                                }
                            }
                        }
                    }
                }

                let mut m_dead = false;

                for (weapon_info, is_offhand) in attacks {
                    if m_dead {
                        break;
                    }

                    let (hit, dmg) = if let Some((w_inst, w_tmpl)) = &weapon_info {
                        let h = CombatEngine::calculate_player_hit(
                            rng,
                            &p_player,
                            &p_status,
                            p_stats.level,
                            m_tmpl_ptr,
                            Some(&m_state),
                            Some(&m_stats),
                            Some((w_inst, w_tmpl)),
                            crate::core::entity::monster::AttackType::Weapon,
                            is_offhand,
                        );
                        let d = if h {
                            CombatEngine::calculate_player_damage(
                                rng,
                                &p_player,
                                m_tmpl_ptr,
                                &m_state,
                                Some((w_inst, w_tmpl)),
                                false,
                                is_offhand,
                                &assets.artifacts,
                            )
                        } else {
                            0
                        };
                        (h, d)
                    } else {
                        // 맨손 공격
                        let h = CombatEngine::calculate_player_hit(
                            rng,
                            &p_player,
                            &p_status,
                            p_stats.level,
                            m_tmpl_ptr,
                            Some(&m_state),
                            Some(&m_stats),
                            None,
                            crate::core::entity::monster::AttackType::Weapon,
                            is_offhand,
                        );
                        let d = if h {
                            CombatEngine::calculate_player_damage(
                                rng,
                                &p_player,
                                m_tmpl_ptr,
                                &m_state,
                                None,
                                false,
                                is_offhand,
                                &assets.artifacts,
                            )
                        } else {
                            0
                        };
                        (h, d)
                    };

                    if hit {
                        // 스킬 연습
                        if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                            if let Ok(player_mut) = p_entry.get_component_mut::<Player>() {
                                let skill = CombatEngine::get_weapon_skill(
                                    player_mut,
                                    weapon_info.as_ref().map(|(_, t)| t),
                                );
                                CombatEngine::practice_weapon_skill(
                                    player_mut, skill, 1, log, *turn,
                                );
                                // 쌍수 스킬 연습
                                if is_offhand || p_player.two_weapon {
                                    CombatEngine::practice_weapon_skill(
                                        player_mut,
                                        WeaponSkill::TwoWeapon,
                                        1,
                                        log,
                                        *turn,
                                    );
                                }
                            }
                        }

                        if let Ok(mut m_entry) = world.entry_mut(m_ent) {
                            if let Ok(m_health) = m_entry.get_component_mut::<Health>() {
                                m_health.current -= dmg;
                                if m_health.current <= 0 {
                                    m_dead = true;
                                }
                                log.add(
                                    format!("You hit the {} for {} damage!", m_display_name, dmg),
                                    *turn,
                                );
                            }
                        }

                        // [v2.0.0 R5] DamageDealt 이벤트 발행
                        let weapon_name = weapon_info
                            .as_ref()
                            .map(|(_, t)| t.name.clone())
                            .unwrap_or_else(|| "unarmed".to_string());
                        event_queue.push(GameEvent::DamageDealt {
                            attacker: "player".to_string(),
                            defender: m_display_name.clone(),
                            amount: dmg,
                            source: weapon_name,
                        });

                        // 수동형 반격 처리
                        CombatEngine::passive(
                            rng,
                            world,
                            p_ent,
                            &p_player,
                            m_tmpl_ptr,
                            true,
                            m_dead,
                            log,
                            turn,
                            assets,
                            command_buffer,
                            p_level.0,
                        );
                    } else {
                        log.add(format!("You miss the {}.", m_display_name), *turn);

                        // [v2.0.0 R5] AttackMissed 이벤트 발행
                        event_queue.push(GameEvent::AttackMissed {
                            attacker: "player".to_string(),
                            defender: m_display_name.clone(),
                        });
                    }
                }

                // 분열 처리 (생존 시에만)
                if !m_dead && CombatEngine::try_split(rng, m_tmpl_ptr) {
                    let mut split_pos = None;
                    let mut occupancy = std::collections::HashSet::new();
                    {
                        let mut q = <&Position>::query();
                        for pos in q.iter(world) {
                            occupancy.insert((pos.x, pos.y));
                        }
                    }

                    'outer: for sx in -1..=1 {
                        for sy in -1..=1 {
                            if sx == 0 && sy == 0 {
                                continue;
                            }
                            let tx = nx + sx;
                            let ty = ny + sy;
                            if tx >= 0 && tx < COLNO as i32 && ty >= 0 && ty < ROWNO as i32 {
                                if !occupancy.contains(&(tx, ty)) {
                                    if let Some(tile) = grid.get_tile(tx as usize, ty as usize) {
                                        if !tile.typ.is_wall() && tile.typ != TileType::Stone {
                                            split_pos = Some((tx, ty));
                                            break 'outer;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some((sx, sy)) = split_pos {
                        log.add(format!("The {} splits!", m_display_name), *turn);
                        let hp = (m_tmpl_ptr.level as i32).max(1) * 4;
                        command_buffer.push((
                            MonsterTag,
                            Species {
                                current: m_tmpl_ptr.name.clone(),
                                original: m_tmpl_ptr.name.clone(),
                                timer: None,
                            },
                            Monster {
                                kind: crate::generated::MonsterKind::from_str(&m_tmpl_ptr.name),
                                hostile: true,
                                mon_name: None,
                            },
                            Talkative,
                            Position { x: sx, y: sy },
                            Renderable {
                                glyph: m_tmpl_ptr.symbol,
                                color: m_tmpl_ptr.color,
                            },
                            Health {
                                current: hp,
                                max: hp,
                            },
                            CombatStats {
                                ac: m_tmpl_ptr.ac as i32,
                                level: m_tmpl_ptr.level as i32,
                            },
                        ));
                    }
                }
                continue; // 공격했으므로 이동은 건너뜀
            }
        }

        // 2.5 바위/석상 체크 (Obstacles)
        let mut obstacle_found = None;
        {
            let mut q = <(
                Entity,
                &Position,
                &Item,
                Option<&crate::core::entity::BoulderTag>,
                Option<&crate::core::entity::StatueTag>,
                &Level,
            )>::query();
            for (ent, pos, item, b_tag, s_tag, lvl) in q.iter(world) {
                if pos.x == nx && pos.y == ny && lvl.0 == p_level.0 {
                    if s_tag.is_some() {
                        obstacle_found = Some((*ent, "statue", false, item.kind.to_string()));
                        break;
                    }
                    if b_tag.is_some() {
                        obstacle_found = Some((*ent, "boulder", true, item.kind.to_string()));
                        break;
                    }
                }
            }
        }

        if let Some((o_ent, kind, pushable, o_name)) = obstacle_found {
            if kind == "statue" {
                log.add(format!("There is a statue ({}) in the way.", o_name), *turn);
                continue;
            }
            if kind == "boulder" && pushable {
                // 바위 밀기 시도
                let tnx = nx + dx;
                let tny = ny + dy;

                let mut can_push = false;
                if tnx >= 0 && tnx < COLNO as i32 && tny >= 0 && tny < ROWNO as i32 {
                    if let Some(tile) = grid.get_tile(tnx as usize, tny as usize) {
                        if !tile.typ.is_wall() && tile.typ != TileType::Stone {
                            //
                            let mut m_q =
                                <(&Position, &Level)>::query().filter(component::<MonsterTag>());
                            let m_at_dest = m_q.iter(world).any(|(pos, lvl)| {
                                pos.x == tnx && pos.y == tny && lvl.0 == p_level.0
                            });

                            //
                            let mut b_q = <(&Position, &Level)>::query()
                                .filter(component::<crate::core::entity::BoulderTag>());
                            let b_at_dest = b_q.iter(world).any(|(pos, lvl)| {
                                pos.x == tnx && pos.y == tny && lvl.0 == p_level.0
                            });

                            if !m_at_dest && !b_at_dest {
                                can_push = true;
                            }
                        }
                    }
                }

                if can_push {
                    if p_player.str.base < 6 {
                        log.add("바위를 밀기에는 힘이 부족합니다.", *turn);
                        continue;
                    }

                    log.add("바위를 밀어냅니다.", *turn);

                    // 함정 메우기 체크
                    let mut trap_to_fill = None;
                    {
                        let mut t_q =
                            <(Entity, &Position, &crate::core::entity::Trap, &Level)>::query()
                                .filter(component::<crate::core::entity::TrapTag>());
                        for (t_ent, t_pos, _trap, t_lvl) in t_q.iter(world) {
                            if t_pos.x == tnx && t_pos.y == tny && t_lvl.0 == p_level.0 {
                                trap_to_fill = Some(*t_ent);
                            }
                        }
                    }

                    if let Some(t_ent) = trap_to_fill {
                        log.add("The boulder fills the hole.", *turn);
                        command_buffer.remove(t_ent);
                        command_buffer.remove(o_ent);
                    } else if let Some(target_tile) = grid.get_tile_mut(tnx as usize, tny as usize)
                    {
                        if target_tile.typ == TileType::Pool || target_tile.typ == TileType::Moat {
                            log.add("The boulder fills the water.", *turn);
                            target_tile.typ = TileType::Room;
                            command_buffer.remove(o_ent);
                        } else {
                            command_buffer.add_component(o_ent, Position { x: tnx, y: tny });
                        }
                    } else {
                        command_buffer.add_component(o_ent, Position { x: tnx, y: tny });
                    }

                    //
                    if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                        if let Ok(pos) = p_entry.get_component_mut::<Position>() {
                            pos.x = nx;
                            pos.y = ny;
                        }
                    }
                    continue;
                } else {
                    log.add("The boulder is stuck.", *turn);
                    continue;
                }
            }
        }

        // 3. 지형 체크 및 이동
        let mut door_opened = false;
        let mut tile_typ = TileType::Stone;
        if let Some(tile) = grid.get_tile_mut(nx as usize, ny as usize) {
            if tile.typ == TileType::Door {
                tile.typ = TileType::OpenDoor;
                tile.doormas = 1;
                door_opened = true;
            } else {
                tile_typ = tile.typ;
            }
        }

        if door_opened {
            log.add("You open the door.", *turn);
            continue;
        }

        if !tile_typ.is_wall() && tile_typ != TileType::Stone {
            let mut p_status = StatusFlags::empty();
            let mut item_protection = 0;
            let mut items_to_process = vec![];

            if let Ok(p_entry) = world.entry_ref(p_ent) {
                p_status = p_entry
                    .get_component::<StatusBundle>()
                    .map(|s| s.flags())
                    .unwrap_or(StatusFlags::empty());

                if let Ok(player) = p_entry.get_component::<Player>() {
                    if let Some(record) = player.skills.get(&WeaponSkill::Swimming) {
                        item_protection = match record.level {
                            SkillLevel::Unskilled => 0,
                            SkillLevel::Basic => 30,
                            SkillLevel::Skilled => 60,
                            SkillLevel::Expert | SkillLevel::Master | SkillLevel::GrandMaster => 90,
                            _ => 0,
                        };
                    }
                }
                if let Ok(inventory) = p_entry.get_component::<Inventory>() {
                    items_to_process = inventory.items.clone();
                }
            }

            let mut message = None;
            let mut die = false;
            let can_cross = true; // Wall check passed, assume crossable unless specific logic blocks it (nethack style: almost always crossable if not wall)

            // 3.5 Terrain Effects
            match tile_typ {
                TileType::Pool | TileType::Moat | TileType::Water => {
                    if !p_status.intersects(
                        StatusFlags::FLYING | StatusFlags::LEVITATING | StatusFlags::WATERWALKING,
                    ) {
                        if p_status.contains(StatusFlags::SWIMMING) {
                            message = Some("You are swimming.");
                        } else {
                            message = Some("You drown...");
                            die = true;
                        }
                    }
                }
                TileType::LavaPool => {
                    if !p_status.intersects(StatusFlags::FLYING | StatusFlags::LEVITATING) {
                        message = Some("You burn to a crisp in the lava!");
                        die = true;
                    }
                }
                _ => {}
            }

            // 3.5 Getting Wet (Phase 42, 48)
            if (tile_typ == TileType::Pool
                || tile_typ == TileType::Moat
                || tile_typ == TileType::Water)
                && !p_status.intersects(
                    StatusFlags::FLYING | StatusFlags::LEVITATING | StatusFlags::WATERWALKING,
                )
            {
                crate::core::systems::item_damage::ItemDamageSystem::water_exposure_recursive(
                    world,
                    &items_to_process,
                    item_protection,
                    log,
                    *turn,
                    true,
                    &assets.items,
                    rng,
                );
            }

            if let Some(msg) = message {
                log.add(msg, *turn);
            }

            if die {
                if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                    if let Ok(health) = p_entry.get_component_mut::<Health>() {
                        health.current = 0;
                    }
                    if let Ok(pos) = p_entry.get_component_mut::<Position>() {
                        pos.x = nx;
                        pos.y = ny;
                    }
                }
            } else if can_cross {
                let mut final_nx = nx;
                let mut final_ny = ny;

                //
                if tile_typ == TileType::Ice && rng.rn2(10) == 0 {
                    let snx = nx + dx;
                    let sny = ny + dy;
                    if snx >= 0 && snx < 80 && sny >= 0 && sny < 21 {
                        if let Some(stile) = grid.get_tile(snx as usize, sny as usize) {
                            if !stile.typ.is_wall() && stile.typ != TileType::Stone {
                                log.add("You slip on the ice!", *turn);
                                final_nx = snx;
                                final_ny = sny;
                            }
                        }
                    }
                }

                if let Ok(mut p_entry) = world.entry_mut(p_ent) {
                    if let Ok(pos) = p_entry.get_component_mut::<Position>() {
                        pos.x = final_nx;
                        pos.y = final_ny;

                        // 이동 후 타일 메시지 (Stairs 등)
                        if let Some(tile) = grid.get_tile(final_nx as usize, final_ny as usize) {
                            match tile.typ {
                                TileType::StairsUp => {
                                    log.add("You see some stairs leading up here.", *turn);
                                }
                                TileType::StairsDown => {
                                    log.add("You see some stairs leading down here.", *turn);
                                }
                                TileType::Ladder => {
                                    log.add("You see a ladder leading down.", *turn);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        } else {
            log.add("You bump into a wall.", *turn);
        }
    }

    // 4. 인접 몬스터 오라(Aura) 피해 체크 (Phase 36)
    let p_q_val: Vec<(Position, Entity)> = <(&Position, Entity)>::query()
        .filter(component::<PlayerTag>())
        .iter(world)
        .map(|(p, e)| (*p, *e))
        .collect();

    for (p_pos, p_ent) in p_q_val {
        let mut nearby_monster_templates = Vec::new();

        // 몬스터 위치 및 템플릿 이름 선수집 (world 차용 충돌 방지)
        {
            let mut m_q = <(&Position, &crate::core::entity::Monster)>::query()
                .filter(component::<MonsterTag>());
            for (m_pos, m_comp) in m_q.iter(world) {
                let dx = (p_pos.x - m_pos.x).abs();
                let dy = (p_pos.y - m_pos.y).abs();
                if dx <= 1 && dy <= 1 && (p_pos.x != m_pos.x || p_pos.y != m_pos.y) {
                    if let Some(template) = assets.monsters.get_by_kind(m_comp.kind) {
                        nearby_monster_templates.push(template);
                    }
                }
            }
        }

        //
        if !nearby_monster_templates.is_empty() {
            if let Ok(mut entry) = world.entry_mut(p_ent) {
                //
                for template in nearby_monster_templates {
                    let p_status_flags = entry
                        .get_component::<StatusBundle>()
                        .map(|s| s.flags())
                        .unwrap_or(StatusFlags::empty());
                    if let Ok(p_health) = entry.get_component_mut::<Health>() {
                        CombatEngine::nearby_passive(
                            rng,
                            p_health,
                            &p_status_flags,
                            template,
                            log,
                            *turn,
                        );
                    }
                }
            }
        }
    }
}

// =============================================================================
// [v2.9.2] hack.c 대량 이식  이동 유틸리티/자동탐험/이동제한/지형효과
// 원본: nethack-3.6.7/src/hack.c (2,939줄)
// =============================================================================

/// [v2.9.2] 이동 방향 벡터 (원본: hack.c xdir/ydir)
pub fn direction_vector(direction: u8) -> (i32, i32) {
    match direction {
        0 => (0, -1),  // N
        1 => (1, -1),  // NE
        2 => (1, 0),   // E
        3 => (1, 1),   // SE
        4 => (0, 1),   // S
        5 => (-1, 1),  // SW
        6 => (-1, 0),  // W
        7 => (-1, -1), // NW
        _ => (0, 0),
    }
}

/// [v2.9.2] 반대 방향 (원본: hack.c opp_dir)
pub fn opposite_direction(direction: u8) -> u8 {
    (direction + 4) % 8
}

/// [v2.9.2] 두 좌표 간 거리 (원본: hack.c distmin)
pub fn distance_min(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    dx.max(dy) // Chebyshev distance
}

/// [v2.9.2] 유클리드 거리 제곱 (원본: hack.c dist2)
pub fn distance_squared(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    dx * dx + dy * dy
}

/// [v2.9.2] 직선 상에 있는지 (원본: hack.c online2)
pub fn on_line(x1: i32, y1: i32, x2: i32, y2: i32) -> bool {
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();
    dx == 0 || dy == 0 || dx == dy
}

/// [v2.9.2] 타일 통과 가능 여부 (원본: hack.c test_move)
pub fn can_pass_tile(
    tile_type: TileType,
    is_levitating: bool,
    is_flying: bool,
    has_water_walking: bool,
    is_swimming: bool,
    passes_walls: bool,
) -> bool {
    match tile_type {
        TileType::Stone => passes_walls,
        t if t.is_wall() => passes_walls,
        TileType::Pool | TileType::Moat | TileType::Water => {
            is_levitating || is_flying || has_water_walking || is_swimming
        }
        TileType::LavaPool => is_levitating || is_flying,
        TileType::Door
        | TileType::OpenDoor
        | TileType::Room
        | TileType::Corr
        | TileType::StairsUp
        | TileType::StairsDown
        | TileType::Altar
        | TileType::Fountain
        | TileType::Throne
        | TileType::Grave
        | TileType::Ladder
        | TileType::Ice
        | TileType::Air => true,
        _ => true,
    }
}

/// [v2.9.2] 대각선 이동 제한 (원본: hack.c bad_rock)
/// 벽/돌 사이 대각선 이동 불가 (꺽어서 이동 필요)
pub fn diagonal_blocked(
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
    is_wall_fn: impl Fn(i32, i32) -> bool,
) -> bool {
    let dx = to_x - from_x;
    let dy = to_y - from_y;

    // 대각선이 아니면 항상 허용
    if dx == 0 || dy == 0 {
        return false;
    }

    // 양 옆이 모두 벽이면 대각선 불가
    let wall_h = is_wall_fn(to_x, from_y); // 수평 방향 벽
    let wall_v = is_wall_fn(from_x, to_y); // 수직 방향 벽

    wall_h && wall_v
}

/// [v2.9.2] 자동 이동(travel) 다음 스텝 (원본: hack.c findtravelpath)
/// BFS 기반 경로 탐색  다음 이동 방향 반환
pub fn find_travel_step(
    from_x: i32,
    from_y: i32,
    to_x: i32,
    to_y: i32,
    max_cols: i32,
    max_rows: i32,
    is_passable: impl Fn(i32, i32) -> bool,
) -> Option<(i32, i32)> {
    if from_x == to_x && from_y == to_y {
        return None; // 이미 도착
    }

    // BFS
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    // (x, y, first_step_x, first_step_y)
    for d in 0..8u8 {
        let (ddx, ddy) = direction_vector(d);
        let nx = from_x + ddx;
        let ny = from_y + ddy;
        if nx >= 0 && nx < max_cols && ny >= 0 && ny < max_rows && is_passable(nx, ny) {
            if nx == to_x && ny == to_y {
                return Some((ddx, ddy));
            }
            visited.insert((nx, ny));
            queue.push_back((nx, ny, ddx, ddy));
        }
    }

    while let Some((cx, cy, fx, fy)) = queue.pop_front() {
        for d in 0..8u8 {
            let (ddx, ddy) = direction_vector(d);
            let nx = cx + ddx;
            let ny = cy + ddy;
            if nx >= 0
                && nx < max_cols
                && ny >= 0
                && ny < max_rows
                && !visited.contains(&(nx, ny))
                && is_passable(nx, ny)
            {
                if nx == to_x && ny == to_y {
                    return Some((fx, fy));
                }
                visited.insert((nx, ny));
                // BFS 최대 탐색 범위 제한
                if visited.len() > 2000 {
                    return None;
                }
                queue.push_back((nx, ny, fx, fy));
            }
        }
    }

    None // 경로 없음
}

/// [v2.9.2] 얼음 미끄러짐 (원본: hack.c slip_on_ice)
pub fn ice_slip_check(
    dex: i32,
    has_boots_of_gripping: bool,
    is_fumbling: bool,
    rng: &mut NetHackRng,
) -> bool {
    if has_boots_of_gripping {
        return false;
    }

    let mut slip_chance = 10; // 기본 10%
    if is_fumbling {
        slip_chance += 30;
    }
    if dex < 10 {
        slip_chance += (10 - dex) * 3;
    }
    if dex >= 18 {
        slip_chance = slip_chance.saturating_sub(5);
    }

    rng.rn2(100) < slip_chance
}

/// [v2.9.2] 문 열기 시도 (원본: hack.c doopen)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DoorOpenResult {
    /// 성공
    Opened,
    /// 잠김 (자물쇠)
    Locked,
    /// 걸림 (열쇠 필요 없이 힘으로)
    Stuck,
    /// 문이 아님
    NotADoor,
    /// 이미 열려있음
    AlreadyOpen,
}

/// [v2.9.2] 문 열기 판정
pub fn try_open_door(
    is_door: bool,
    is_open: bool,
    is_locked: bool,
    player_str: i32,
    rng: &mut NetHackRng,
) -> DoorOpenResult {
    if !is_door {
        return DoorOpenResult::NotADoor;
    }
    if is_open {
        return DoorOpenResult::AlreadyOpen;
    }
    if is_locked {
        return DoorOpenResult::Locked;
    }

    // 힘 기반 문 여는 판정
    if rng.rn2(100) < (player_str * 4 + 20).min(95) {
        DoorOpenResult::Opened
    } else {
        DoorOpenResult::Stuck
    }
}

/// [v2.9.2] 문 부수기 (원본: hack.c kick_door)
pub fn kick_door(player_str: i32, is_locked: bool, rng: &mut NetHackRng) -> (bool, &'static str) {
    if !is_locked {
        if rng.rn2(3) < 2 {
            (true, "As you kick the door, it crashes open!")
        } else {
            (false, "The door shudders but holds.")
        }
    } else {
        let break_chance = player_str * 3;
        if rng.rn2(100) < break_chance.min(70) {
            (true, "As you kick the door, it crashes open!")
        } else {
            (false, "THUD! The locked door resists your kick.")
        }
    }
}

/// [v2.9.2] 이동 속도 계산 (원본: hack.c moveloop speed)
pub fn movement_points(
    base_speed: i32,
    is_fast: bool,
    is_very_fast: bool,
    is_slow: bool,
    encumbrance_level: i32, // 0=unenc, 1=burdened, 2=stressed, 3=strained, 4=overtaxed
) -> i32 {
    let mut speed = base_speed;
    if is_very_fast {
        speed += 8;
    } else if is_fast {
        speed += 4;
    }
    if is_slow {
        speed -= 4;
    }

    // 부하 감속
    speed -= encumbrance_level * 2;

    speed.max(1)
}

/// [v2.9.2] 자동 줍기 필터 (원본: hack.c autopickup)
pub fn should_autopickup(
    item_class: &str,
    autopickup_classes: &[&str],
    item_cursed_known: bool,
    item_is_cursed: bool,
) -> bool {
    // 저주된 걸로 알려진 아이템은 무시
    if item_cursed_known && item_is_cursed {
        return false;
    }
    autopickup_classes.contains(&item_class)
}

/// [v2.9.2] 이동 통계
#[derive(Debug, Clone, Default)]
pub struct MovementStatistics {
    pub steps_taken: u64,
    pub doors_opened: u32,
    pub doors_kicked: u32,
    pub ice_slips: u32,
    pub bumps: u32,
    pub boulder_pushes: u32,
    pub travel_uses: u32,
}

impl MovementStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_step(&mut self) {
        self.steps_taken += 1;
    }
    pub fn record_door_open(&mut self) {
        self.doors_opened += 1;
    }
    pub fn record_ice_slip(&mut self) {
        self.ice_slips += 1;
    }
    pub fn record_bump(&mut self) {
        self.bumps += 1;
    }
}

// =============================================================================
// [v2.9.2] hack.c 테스트
// =============================================================================
#[cfg(test)]
mod movement_v292_tests {
    use super::*;

    #[test]
    fn test_direction_vector() {
        assert_eq!(direction_vector(0), (0, -1)); // N
        assert_eq!(direction_vector(4), (0, 1)); // S
        assert_eq!(direction_vector(2), (1, 0)); // E
    }

    #[test]
    fn test_opposite_direction() {
        assert_eq!(opposite_direction(0), 4); // N  S
        assert_eq!(opposite_direction(1), 5); // NE  SW
    }

    #[test]
    fn test_distance_min() {
        assert_eq!(distance_min(0, 0, 3, 4), 4);
        assert_eq!(distance_min(0, 0, 5, 5), 5);
    }

    #[test]
    fn test_distance_squared() {
        assert_eq!(distance_squared(0, 0, 3, 4), 25);
    }

    #[test]
    fn test_on_line() {
        assert!(on_line(0, 0, 0, 5)); // 수직
        assert!(on_line(0, 0, 5, 0)); // 수평
        assert!(on_line(0, 0, 3, 3)); // 대각
        assert!(!on_line(0, 0, 2, 3)); // 비직선
    }

    #[test]
    fn test_can_pass_room() {
        assert!(can_pass_tile(
            TileType::Room,
            false,
            false,
            false,
            false,
            false
        ));
    }

    #[test]
    fn test_can_pass_wall() {
        assert!(!can_pass_tile(
            TileType::Stone,
            false,
            false,
            false,
            false,
            false
        ));
        assert!(can_pass_tile(
            TileType::Stone,
            false,
            false,
            false,
            false,
            true
        ));
    }

    #[test]
    fn test_can_pass_water() {
        assert!(!can_pass_tile(
            TileType::Pool,
            false,
            false,
            false,
            false,
            false
        ));
        assert!(can_pass_tile(
            TileType::Pool,
            true,
            false,
            false,
            false,
            false
        ));
        assert!(can_pass_tile(
            TileType::Pool,
            false,
            false,
            true,
            false,
            false
        ));
    }

    #[test]
    fn test_diagonal_blocked() {
        let walls = |x: i32, y: i32| x == 1 && y == 0 || x == 0 && y == 1;
        assert!(diagonal_blocked(0, 0, 1, 1, walls));
    }

    #[test]
    fn test_diagonal_not_blocked() {
        let walls = |_x: i32, _y: i32| false;
        assert!(!diagonal_blocked(0, 0, 1, 1, walls));
    }

    #[test]
    fn test_travel_adjacent() {
        let step = find_travel_step(0, 0, 1, 0, 80, 21, |_, _| true);
        assert_eq!(step, Some((1, 0)));
    }

    #[test]
    fn test_travel_same_pos() {
        let step = find_travel_step(5, 5, 5, 5, 80, 21, |_, _| true);
        assert!(step.is_none());
    }

    #[test]
    fn test_ice_slip_gripping() {
        let mut rng = NetHackRng::new(42);
        assert!(!ice_slip_check(10, true, false, &mut rng));
    }

    #[test]
    fn test_open_door() {
        let mut rng = NetHackRng::new(42);
        let r = try_open_door(true, false, false, 18, &mut rng);
        assert!(r == DoorOpenResult::Opened || r == DoorOpenResult::Stuck);
    }

    #[test]
    fn test_open_locked_door() {
        let mut rng = NetHackRng::new(42);
        let r = try_open_door(true, false, true, 18, &mut rng);
        assert_eq!(r, DoorOpenResult::Locked);
    }

    #[test]
    fn test_kick_door() {
        let mut rng = NetHackRng::new(42);
        let (broke, msg) = kick_door(18, true, &mut rng);
        assert!(!msg.is_empty());
        let _ = broke; // 확률적
    }

    #[test]
    fn test_movement_points() {
        let sp = movement_points(12, true, false, false, 0);
        assert_eq!(sp, 16);
        let sp2 = movement_points(12, false, false, true, 2);
        assert_eq!(sp2, 4); // 12 - 4 - 4
    }

    #[test]
    fn test_autopickup() {
        assert!(should_autopickup("gold", &["gold", "gem"], false, false));
        assert!(!should_autopickup("scroll", &["gold", "gem"], false, false));
        assert!(!should_autopickup("gold", &["gold"], true, true)); // 저주된 건 무시
    }

    #[test]
    fn test_movement_stats() {
        let mut s = MovementStatistics::new();
        s.record_step();
        s.record_step();
        s.record_door_open();
        assert_eq!(s.steps_taken, 2);
        assert_eq!(s.doors_opened, 1);
    }
}
