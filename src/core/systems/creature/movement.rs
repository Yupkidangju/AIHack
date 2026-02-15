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
