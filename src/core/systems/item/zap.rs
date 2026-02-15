// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::core::dungeon::Grid;
use crate::core::entity::monster::DamageType;
use crate::core::entity::player::Player;
use crate::core::entity::{Health, Monster, MonsterTag, PlayerTag, Position};
use crate::core::game_state::Direction;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::world::SubWorld;
use legion::*;

/// 지팡이 사용 또는 주문 시전 액션
#[derive(Debug, Clone)]
pub struct ZapAction {
    pub item_ent: Option<Entity>,
    pub spell_name: Option<String>,
    pub direction: Direction,
}

#[legion::system]
#[read_component(PlayerTag)]
#[write_component(Player)]
#[read_component(Position)]
#[write_component(Health)]
#[read_component(MonsterTag)]
#[read_component(Monster)]
#[write_component(crate::core::entity::Item)]
#[read_component(crate::core::entity::status::StatusBundle)]
pub fn zap(
    world: &mut SubWorld,
    #[resource] zap_action: &mut Option<ZapAction>,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] grid: &mut Grid,
    #[resource] rng: &mut NetHackRng,
) {
    let action = match zap_action.take() {
        Some(a) => a,
        None => return,
    };

    let mut start_pos = (0, 0);
    let mut p_query = <&Position>::query().filter(component::<PlayerTag>());
    if let Some(pos) = p_query.iter(world).next() {
        start_pos = (pos.x, pos.y);
    }

    if let Some(item_ent) = action.item_ent {
        // 지팡이 사용 (Wand Zap)
        let mut wand_info = None;
        if let Ok(mut entry) = world.entry_mut(item_ent) {
            if let Ok(item) = entry.get_component_mut::<crate::core::entity::Item>() {
                if item.spe > 0 {
                    item.spe -= 1;
                    wand_info = Some((item.kind.to_string(), item.spe));
                } else {
                    log.add("The wand is empty.", *turn);
                    return;
                }
            }
        }

        if let Some((template, _charges)) = wand_info {
            log.add_colored(format!("You zap a {}.", template), [255, 255, 0], *turn);
            execute_wand_effect(
                &template,
                start_pos,
                action.direction,
                world,
                log,
                *turn,
                grid,
                rng,
            );
        }
    } else if let Some(spell_name) = action.spell_name {
        // 주문 시전 (Spell Cast)
        log.add(format!("You cast {}.", spell_name), *turn);
        execute_spell_effect_internal(
            &spell_name,
            start_pos,
            action.direction,
            world,
            log,
            *turn,
            grid,
            rng,
        );
    }
}

fn execute_wand_effect(
    name: &str,
    origin: (i32, i32),
    dir: Direction,
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
    grid: &mut Grid,
    rng: &mut NetHackRng,
) {
    // NetHack 3.6.7: zap.c:weffects() 로직 이식
    if name.contains("striking") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Phys,
            10,
            (2, 12),
            "force bolt",
            log,
            turn,
            grid,
            rng,
            false,
        );
    } else if name.contains("magic missile") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Magm,
            10,
            (2, 6),
            "magic missile",
            log,
            turn,
            grid,
            rng,
            false,
        );
    } else if name.contains("fire") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Fire,
            13,
            (6, 6),
            "fire bolt",
            log,
            turn,
            grid,
            rng,
            true,
        );
    } else if name.contains("cold") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Cold,
            13,
            (6, 6),
            "cold bolt",
            log,
            turn,
            grid,
            rng,
            true,
        );
    } else if name.contains("sleep") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Slee,
            15,
            (0, 0),
            "sleep ray",
            log,
            turn,
            grid,
            rng,
            true,
        );
    } else if name.contains("death") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Deth,
            15,
            (500, 1),
            "death ray",
            log,
            turn,
            grid,
            rng,
            true,
        );
    } else if name.contains("teleportation") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Tlpt,
            20,
            (0, 0),
            "teleportation ray",
            log,
            turn,
            grid,
            rng,
            true,
        );
    } else if name.contains("digging") {
        execute_dig(world, origin, dir, grid, log, turn);
    } else if name.contains("opening") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Open,
            12,
            (0, 0),
            "ray of opening",
            log,
            turn,
            grid,
            rng,
            true,
        );
    } else if name.contains("polymorph") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Poly,
            15,
            (0, 0),
            "polymorphic ray",
            log,
            turn,
            grid,
            rng,
            true,
        );
    } else if name.contains("light") {
        execute_light_beam(world, origin, dir, grid, log, turn);
    } else {
        log.add(format!("The {} has no effect yet.", name), turn);
    }
}

/// 빛 지팡이 효과 (Wand of Light)
fn execute_light_beam(
    _world: &mut SubWorld,
    origin: (i32, i32),
    dir: Direction,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
) {
    let (dx, dy) = dir.to_delta();
    if dx == 0 && dy == 0 {
        //
        grid.light_room_at(origin.0 as usize, origin.1 as usize);
        log.add("A shimmer of light surrounds you.", turn);
        return;
    }

    log.add("A beam of light shoots out!", turn);
    let mut cx = origin.0;
    let mut cy = origin.1;
    let mut range = 10;

    while range > 0 {
        range -= 1;
        cx += dx;
        cy += dy;

        if cx < 0 || cx >= 80 || cy < 0 || cy >= 21 {
            break;
        }

        if let Some(tile) = grid.get_tile_mut(cx as usize, cy as usize) {
            tile.flags |= crate::core::dungeon::tile::TileFlags::LIT;
            if tile.typ.is_wall() {
                break;
            }
        }
    }
}

fn execute_spell_effect_internal(
    name: &str,
    origin: (i32, i32),
    dir: Direction,
    world: &mut SubWorld,
    log: &mut GameLog,
    turn: u64,
    grid: &mut Grid,
    rng: &mut NetHackRng,
) {
    //
    if name.contains("Force Bolt") || name.contains("force bolt") {
        execute_bolt(
            world,
            origin,
            dir,
            DamageType::Phys,
            10,
            (2, 12),
            "force bolt",
            log,
            turn,
            grid,
            rng,
            false,
        );
    } else {
        log.add(format!("The spell {} has no effect yet.", name), turn);
    }
}

/// 공용 볼트/레이 실행 엔진 (zap.c:bhit 이식)
pub fn execute_bolt(
    world: &mut SubWorld,
    origin: (i32, i32),
    dir: Direction,
    dtype: DamageType,
    mut range: i32,
    dice: (i32, i32),
    bolt_name: &str,
    log: &mut GameLog,
    turn: u64,
    grid: &mut Grid,
    rng: &mut NetHackRng,
    is_ray: bool,
) {
    let (mut dx, mut dy) = dir.to_delta();
    if dx == 0 && dy == 0 {
        return;
    }

    let mut cx = origin.0;
    let mut cy = origin.1;
    let mut bounce_count = 0;
    const MAX_BOUNCE: i32 = 8;

    while range > 0 {
        range -= 1;
        let prev_x = cx;
        let prev_y = cy;
        cx += dx;
        cy += dy;

        if cx < 0 || cx >= 80 || cy < 0 || cy >= 21 {
            break;
        }

        // 1. 벽 충돌 (Bouncing)
        let mut hit_wall = false;
        if let Some(tile) = grid.get_tile(cx as usize, cy as usize) {
            if tile.typ.is_wall() {
                hit_wall = true;
            }
        }

        if hit_wall {
            if is_ray && bounce_count < MAX_BOUNCE {
                bounce_count += 1;
                log.add(format!("The {} bounces!", bolt_name), turn);

                // 도탄 로직 (Diagonal bounce)
                // 수평/수직 벽면 판별
                let wall_h = if let Some(t) = grid.get_tile(prev_x as usize, cy as usize) {
                    t.typ.is_wall()
                } else {
                    true
                };
                let wall_v = if let Some(t) = grid.get_tile(cx as usize, prev_y as usize) {
                    t.typ.is_wall()
                } else {
                    true
                };

                if dx != 0 && dy != 0 {
                    // 대각선 입사
                    if wall_h && wall_v {
                        dx = -dx;
                        dy = -dy;
                    } else if wall_h {
                        dx = -dx;
                        // 산란(Scattering) 효과 (Diagonal bounce split)
                        if rng.rn2(4) == 0 {
                            //
                            dy = -dy;
                        }
                    } else if wall_v {
                        dy = -dy;
                    } else {
                        dx = -dx;
                        dy = -dy;
                    }
                } else if dx != 0 {
                    dx = -dx;
                } else {
                    dy = -dy;
                }

                cx = prev_x;
                cy = prev_y;
                continue;
            } else {
                log.add(format!("The {} hits the wall.", bolt_name), turn);
                break;
            }
        }

        //
        if let Some(tile) = grid.get_tile_mut(cx as usize, cy as usize) {
            use crate::core::dungeon::tile::TileType;
            match dtype {
                DamageType::Fire => {
                    if tile.typ == TileType::Ice {
                        log.add("The ice melts!", turn);
                        tile.typ = TileType::Pool;
                    }
                }
                DamageType::Cold => {
                    if tile.typ == TileType::Pool
                        || tile.typ == TileType::Moat
                        || tile.typ == TileType::Water
                    {
                        log.add("The water freezes!", turn);
                        tile.typ = TileType::Ice;
                    }
                }
                _ => {}
            }
        }

        //
        let mut p_reflect = false;
        {
            let mut query = <(&Position, &crate::core::entity::status::StatusBundle)>::query()
                .filter(component::<PlayerTag>());
            if let Some((pos, status)) = query.iter(world).next() {
                if pos.x == cx && pos.y == cy {
                    if status.has(crate::core::entity::status::StatusFlags::REFLECTING) {
                        p_reflect = true;
                    }
                }
            }
        }

        if p_reflect {
            log.add_colored(
                format!("The {} reflects off your shield!", bolt_name),
                [255, 255, 255],
                turn,
            );
            dx = -dx;
            dy = -dy;
            cx = prev_x;
            cy = prev_y;
            bounce_count += 1;
            if bounce_count >= MAX_BOUNCE {
                break;
            }
            continue;
        }

        let mut p_hit = false;
        let mut p_query = <(&Position, &mut Health)>::query().filter(component::<PlayerTag>());
        for (pos, health) in p_query.iter_mut(world) {
            if pos.x == cx && pos.y == cy {
                p_hit = true;
                let dmg = if dice.0 > 0 { rng.d(dice.0, dice.1) } else { 0 };

                match dtype {
                    DamageType::Slee => {
                        log.add_colored("You fall asleep!", [200, 200, 255], turn);
                        // TODO: Status PARALYZED
                    }
                    DamageType::Deth => {
                        log.add_colored("You are struck by a ray of death!", [0, 0, 0], turn);
                        health.current = 0;
                    }
                    DamageType::Tlpt => {
                        log.add("You are teleported!", turn);
                    }
                    _ => {
                        health.current -= dmg;
                        log.add(
                            format!("The {} hits you for {} damage!", bolt_name, dmg),
                            turn,
                        );
                    }
                }
                break;
            }
        }
        if p_hit && !is_ray {
            break;
        }

        // 3. 몬스터 충돌 (Reflection 지원)
        let mut m_reflect = false;
        let mut m_hit_ent = None;

        {
            let mut query = <(
                Entity,
                &Position,
                &crate::core::entity::status::StatusBundle,
            )>::query()
            .filter(component::<MonsterTag>());
            for (ent, pos, status) in query.iter(world) {
                if pos.x == cx && pos.y == cy {
                    if status.has(crate::core::entity::status::StatusFlags::REFLECTING) {
                        m_reflect = true;
                        break;
                    }
                    m_hit_ent = Some(*ent);
                }
            }
        }

        if m_reflect {
            log.add(format!("The {} reflects off the monster!", bolt_name), turn);
            dx = -dx;
            dy = -dy;
            cx = prev_x;
            cy = prev_y;
            bounce_count += 1;
            if bounce_count >= MAX_BOUNCE {
                break;
            }
            continue;
        }

        if let Some(m_ent) = m_hit_ent {
            let dmg = if dice.0 > 0 { rng.d(dice.0, dice.1) } else { 0 };
            if let Ok(mut entry) = world.entry_mut(m_ent) {
                if let Ok(health) = entry.get_component_mut::<Health>() {
                    match dtype {
                        DamageType::Deth => {
                            log.add("The monster dies!", turn);
                            health.current = 0;
                        }
                        DamageType::Slee => {
                            log.add("The monster falls asleep!", turn);
                        }
                        DamageType::Poly => {
                            log.add("The monster changes its form!", turn);
                            // 몬스터 변이 (Polymorph)
                            if let Ok(m) = entry.get_component_mut::<Monster>() {
                                //
                                m.kind = crate::generated::MonsterKind::from_str("newt");
                                // 나중에 랜덤 선별 추가
                            }
                        }
                        DamageType::Open => {
                            log.add("The monster seems unlocked?", turn);
                        }
                        _ => {
                            health.current -= dmg;
                            log.add(
                                format!("The {} hits the monster for {} damage!", bolt_name, dmg),
                                turn,
                            );
                        }
                    }
                }
            }
            if !is_ray {
                break;
            }
        }
    }
}

/// 굴착(Digging) 지팡이 로직 (zap.c:dig_actual 이식)
pub fn execute_dig(
    _world: &mut SubWorld,
    origin: (i32, i32),
    dir: crate::core::game_state::Direction,
    grid: &mut Grid,
    log: &mut GameLog,
    turn: u64,
) {
    let (dx, dy) = dir.to_delta();
    if dx == 0 && dy == 0 {
        return;
    }

    log.add("A ray of digging emanates from the wand.", turn);

    let mut cx = origin.0;
    let mut cy = origin.1;
    let mut range = 10;

    while range > 0 {
        range -= 1;
        cx += dx;
        cy += dy;

        if cx < 0 || cx >= 80 || cy < 0 || cy >= 21 {
            break;
        }

        if let Some(tile) = grid.get_tile_mut(cx as usize, cy as usize) {
            if tile.typ.is_wall() || tile.typ == crate::core::dungeon::tile::TileType::Stone {
                tile.typ = crate::core::dungeon::tile::TileType::Corr;
                log.add("The wall crumbles!", turn);
                break; // 한 번에 한 벽만 뚫음 (또는 빔처럼 뚫을지 결정 가능)
            }
        }
    }
}

// =============================================================================
// [v2.3.1] zap.c 확장 이식
// 원본: nethack-3.6.7/src/zap.c (5,016줄)
//
// 지팡이 종류 데이터, 충전, 폭발 범위, 자가 대상, 저항 보정 등
// =============================================================================

/// [v2.3.1] 지팡이 종류 데이터 (원본: objects[] + zap.c)
#[derive(Debug, Clone)]
pub struct WandType {
    /// 지팡이 이름
    pub name: &'static str,
    /// 피해 유형
    pub damage_type: DamageType,
    /// 기본 사거리
    pub range: i32,
    /// 기본 데미지 주사위
    pub dice: (i32, i32),
    /// 레이(관통)인지 볼트(단일)인지
    pub is_ray: bool,
    /// 초기 충전 수 (최소, 최대)
    pub charges: (i32, i32),
    /// 방향 필요 여부
    pub directed: bool,
    /// 빔 기호
    pub beam_symbol: char,
}

/// [v2.3.1] 전체 지팡이 종류 테이블
pub fn wand_type_table() -> Vec<WandType> {
    vec![
        WandType {
            name: "wand of magic missile",
            damage_type: DamageType::Magm,
            range: 10,
            dice: (2, 6),
            is_ray: false,
            charges: (5, 8),
            directed: true,
            beam_symbol: '-',
        },
        WandType {
            name: "wand of fire",
            damage_type: DamageType::Fire,
            range: 13,
            dice: (6, 6),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of cold",
            damage_type: DamageType::Cold,
            range: 13,
            dice: (6, 6),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of sleep",
            damage_type: DamageType::Slee,
            range: 15,
            dice: (0, 0),
            is_ray: true,
            charges: (5, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of death",
            damage_type: DamageType::Deth,
            range: 15,
            dice: (500, 1),
            is_ray: true,
            charges: (3, 5),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of striking",
            damage_type: DamageType::Phys,
            range: 10,
            dice: (2, 12),
            is_ray: false,
            charges: (5, 8),
            directed: true,
            beam_symbol: '-',
        },
        WandType {
            name: "wand of teleportation",
            damage_type: DamageType::Tlpt,
            range: 20,
            dice: (0, 0),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of digging",
            damage_type: DamageType::Phys,
            range: 10,
            dice: (0, 0),
            is_ray: false,
            charges: (5, 8),
            directed: true,
            beam_symbol: '-',
        },
        WandType {
            name: "wand of opening",
            damage_type: DamageType::Open,
            range: 12,
            dice: (0, 0),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of polymorph",
            damage_type: DamageType::Poly,
            range: 15,
            dice: (0, 0),
            is_ray: true,
            charges: (3, 6),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of light",
            damage_type: DamageType::Phys,
            range: 10,
            dice: (0, 0),
            is_ray: false,
            charges: (10, 15),
            directed: false,
            beam_symbol: '*',
        },
        WandType {
            name: "wand of nothing",
            damage_type: DamageType::Phys,
            range: 0,
            dice: (0, 0),
            is_ray: false,
            charges: (5, 8),
            directed: true,
            beam_symbol: '-',
        },
        WandType {
            name: "wand of locking",
            damage_type: DamageType::Phys,
            range: 12,
            dice: (0, 0),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of probing",
            damage_type: DamageType::Phys,
            range: 15,
            dice: (0, 0),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of undead turning",
            damage_type: DamageType::Phys,
            range: 10,
            dice: (0, 0),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of speed monster",
            damage_type: DamageType::Phys,
            range: 0,
            dice: (0, 0),
            is_ray: false,
            charges: (5, 8),
            directed: true,
            beam_symbol: '-',
        },
        WandType {
            name: "wand of slow monster",
            damage_type: DamageType::Phys,
            range: 15,
            dice: (0, 0),
            is_ray: true,
            charges: (5, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of cancellation",
            damage_type: DamageType::Phys,
            range: 10,
            dice: (0, 0),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of make invisible",
            damage_type: DamageType::Phys,
            range: 10,
            dice: (0, 0),
            is_ray: true,
            charges: (4, 8),
            directed: true,
            beam_symbol: '/',
        },
        WandType {
            name: "wand of create monster",
            damage_type: DamageType::Phys,
            range: 0,
            dice: (0, 0),
            is_ray: false,
            charges: (5, 8),
            directed: false,
            beam_symbol: '*',
        },
    ]
}

///
pub fn get_wand_type(name: &str) -> Option<WandType> {
    let lower = name.to_lowercase();
    wand_type_table()
        .into_iter()
        .find(|w| lower.contains(w.name))
}

/// [v2.3.1] 지팡이 충전 (원본: recharge)
pub fn recharge_wand(current_charges: i32, blessed: bool, rng: &mut NetHackRng) -> RechargeResult {
    let max_recharge = if blessed { 15 } else { 10 };

    if current_charges >= max_recharge {
        //
        return RechargeResult::Exploded;
    }

    let added = if blessed {
        rng.rn1(5, 3) // 3~7
    } else {
        rng.rn1(3, 1) // 1~3
    };

    let new_charges = (current_charges + added).min(max_recharge);
    RechargeResult::Success(new_charges)
}

/// [v2.3.1] 충전 결과
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RechargeResult {
    Success(i32),
    Exploded,
}

/// [v2.3.1] 자가 대상 효과 (원본: zap_self)
pub fn self_zap_effect(wand_name: &str) -> &'static str {
    if wand_name.contains("death") {
        "You irradiate yourself with pure death!"
    } else if wand_name.contains("sleep") {
        "You fall asleep from your own wand!"
    } else if wand_name.contains("fire") {
        "You burn yourself!"
    } else if wand_name.contains("cold") {
        "You freeze yourself!"
    } else if wand_name.contains("polymorph") {
        "You feel a change coming over you!"
    } else if wand_name.contains("teleportation") {
        "You teleport yourself!"
    } else if wand_name.contains("speed") {
        "You feel yourself speed up!"
    } else if wand_name.contains("make invisible") {
        "You become invisible!"
    } else if wand_name.contains("light") {
        "You are blinded by the flash!"
    } else if wand_name.contains("probing") {
        "You probe yourself. You are still alive."
    } else {
        "Nothing seems to happen."
    }
}

/// [v2.3.1] 저항별 데미지 보정 계수 (원본: resist_reduce_dmg)
pub fn resistance_damage_factor(
    dtype: DamageType,
    has_fire_res: bool,
    has_cold_res: bool,
    has_shock_res: bool,
    has_magic_res: bool,
    has_death_res: bool,
) -> f32 {
    match dtype {
        DamageType::Fire => {
            if has_fire_res {
                0.0
            } else {
                1.0
            }
        }
        DamageType::Cold => {
            if has_cold_res {
                0.0
            } else {
                1.0
            }
        }
        DamageType::Elec => {
            if has_shock_res {
                0.0
            } else {
                1.0
            }
        }
        DamageType::Magm => {
            if has_magic_res {
                0.5
            } else {
                1.0
            }
        }
        DamageType::Deth => {
            if has_death_res {
                0.0
            } else {
                1.0
            }
        }
        _ => 1.0,
    }
}

/// [v2.3.1] 빔 시각화 기호 (원본: zap.c visualization)
pub fn beam_visual(dtype: DamageType, horizontal: bool) -> char {
    match dtype {
        DamageType::Fire => {
            if horizontal {
                '-'
            } else {
                '|'
            }
        }
        DamageType::Cold => {
            if horizontal {
                '-'
            } else {
                '|'
            }
        }
        DamageType::Magm => '*',
        DamageType::Deth => {
            if horizontal {
                '-'
            } else {
                '|'
            }
        }
        DamageType::Slee => ')',
        DamageType::Elec => '#',
        _ => {
            if horizontal {
                '-'
            } else {
                '|'
            }
        }
    }
}

/// [v2.3.1] 폭발 범위 타일 계산 (원본: explode())
/// 중심 (cx, cy) 주변 radius 내의 (x, y) 리스트 반환
pub fn explosion_area(cx: i32, cy: i32, radius: i32) -> Vec<(i32, i32)> {
    let mut tiles = Vec::new();
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            let x = cx + dx;
            let y = cy + dy;
            // 원형 범위
            if dx * dx + dy * dy <= radius * radius {
                tiles.push((x, y));
            }
        }
    }
    tiles
}

/// [v2.3.1] 지팡이 꺾기 (원본: break_wand)
pub fn break_wand_effect(wand_name: &str, charges_left: i32) -> BreakWandResult {
    if charges_left <= 0 {
        return BreakWandResult {
            damage: 0,
            message: "The wand sputters and dies.".to_string(),
            explosion_radius: 0,
        };
    }

    // 잔여 충전에 비례한 폭발
    let damage = charges_left * 4;
    let radius = if charges_left > 5 { 2 } else { 1 };

    let msg = if wand_name.contains("fire") {
        "Kaboom! The wand explodes in a burst of flame!".to_string()
    } else if wand_name.contains("cold") {
        "Wham! The wand explodes in a blast of frost!".to_string()
    } else if wand_name.contains("death") {
        "The wand of death shatters, releasing terrible energy!".to_string()
    } else {
        format!("As you {}, the wand explodes!", "break the wand")
    };

    BreakWandResult {
        damage,
        message: msg,
        explosion_radius: radius,
    }
}

/// [v2.3.1] 지팡이 꺾기 결과
#[derive(Debug, Clone)]
pub struct BreakWandResult {
    pub damage: i32,
    pub message: String,
    pub explosion_radius: i32,
}

// =============================================================================
// [v2.3.4] 지팡이 확장 (원본 zap.c: advanced wand mechanics)
// =============================================================================

/// 빔 반사 횟수 계산 (원본: zap.c reflection chain)
pub fn beam_reflection_max(beam_type: DamageType, reflectable: bool) -> i32 {
    if !reflectable {
        return 0;
    }
    match beam_type {
        DamageType::Magm => 3, // 마법 미사일: 3회 반사
        DamageType::Fire => 0, // 화염: 반사 불가
        DamageType::Cold => 2, // 냉기: 2회 반사
        DamageType::Slee => 1, // 수면: 1회 반사
        DamageType::Drst => 0, // 죽음: 반사 불가 (마법 저항으로만)
        DamageType::Elec => 2, // 전격: 2회 반사
        DamageType::Acid => 0, // 산: 반사 불가
        _ => 1,
    }
}

/// 빔 감쇠 계산 (원본: zap.c beam attenuation)
pub fn beam_attenuation(base_damage: i32, distance: i32, beam_type: DamageType) -> i32 {
    let decay_rate = match beam_type {
        DamageType::Magm => 5,  // 마법 미사일: 느린 감쇠
        DamageType::Fire => 10, // 화염: 빠른 감쇠
        DamageType::Cold => 8,  // 냉기: 중간 감쇠
        DamageType::Elec => 3,  // 전격: 매우 느린 감쇠
        _ => 7,
    };
    let reduction = (distance * decay_rate) / 10;
    (base_damage - reduction).max(1)
}

///
pub fn wand_overcharge_check(current_charges: i32, max_charges: i32, rng: &mut NetHackRng) -> bool {
    if current_charges <= max_charges {
        return false;
    }
    //
    let excess = current_charges - max_charges;
    rng.rn2(100) < excess * 15
}

///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeamTerrainEffect {
    None,
    MeltIce,        // 얼음 녹임
    FreezeWater,    // 물 얼림
    EvaporateWater, // 물 증발
    BurnTree,       // 나무 태움
    DestroyDoor,    // 문 파괴
    MakeLava,       // 용암 생성
    ElectrifyWater, // 물에 전기 전도
}

///
pub fn beam_terrain_interaction(beam_type: DamageType, terrain: &str) -> BeamTerrainEffect {
    let t = terrain.to_lowercase();
    match beam_type {
        DamageType::Fire => {
            if t.contains("ice") {
                BeamTerrainEffect::MeltIce
            } else if t.contains("tree") || t.contains("door") {
                BeamTerrainEffect::BurnTree
            } else if t.contains("water") {
                BeamTerrainEffect::EvaporateWater
            } else {
                BeamTerrainEffect::None
            }
        }
        DamageType::Cold => {
            if t.contains("water") || t.contains("pool") {
                BeamTerrainEffect::FreezeWater
            } else {
                BeamTerrainEffect::None
            }
        }
        DamageType::Elec => {
            if t.contains("water") || t.contains("pool") {
                BeamTerrainEffect::ElectrifyWater
            } else if t.contains("door") {
                BeamTerrainEffect::DestroyDoor
            } else {
                BeamTerrainEffect::None
            }
        }
        _ => BeamTerrainEffect::None,
    }
}

/// 빔 색상 테이블 (원본: zap.c beam colors)
pub fn beam_color(beam_type: DamageType) -> &'static str {
    match beam_type {
        DamageType::Magm => "bright purple",
        DamageType::Fire => "orange-red",
        DamageType::Cold => "icy blue",
        DamageType::Elec => "crackling white",
        DamageType::Acid => "sickly green",
        DamageType::Slee => "misty gray",
        DamageType::Drst => "deathly black",
        DamageType::Ston => "dull gray",
        _ => "shimmering",
    }
}

/// 지팡이 식별 힌트 (원본: zap.c wand identification)
pub fn wand_identify_hint(wand_name: &str, charges: i32) -> &'static str {
    if charges == 0 {
        "It seems depleted."
    } else if charges >= 10 {
        "It seems fully charged."
    } else if charges >= 5 {
        "It seems moderately charged."
    } else {
        "It seems weakly charged."
    }
}

/// 지팡이 충전 비용 (원본: zap.c recharge cost)
pub fn wand_recharge_cost(wand_name: &str) -> i32 {
    let l = wand_name.to_lowercase();
    if l.contains("wishing") {
        return 500;
    }
    if l.contains("death") || l.contains("polymorph") {
        return 300;
    }
    if l.contains("teleportation") || l.contains("fire") || l.contains("cold") {
        return 200;
    }
    if l.contains("lightning") || l.contains("sleep") {
        return 150;
    }
    if l.contains("digging") || l.contains("speed") {
        return 100;
    }
    75
}

/// 빔 통계
#[derive(Debug, Clone, Default)]
pub struct BeamStatistics {
    pub beams_fired: u32,
    pub reflections: u32,
    pub terrain_effects: u32,
    pub overcharges: u32,
    pub total_beam_damage: i32,
    pub wands_recharged: u32,
}

impl BeamStatistics {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record_fire(&mut self, damage: i32) {
        self.beams_fired += 1;
        self.total_beam_damage += damage;
    }
    pub fn record_reflect(&mut self) {
        self.reflections += 1;
    }
}

#[cfg(test)]
mod zap_extended_tests {
    use super::*;

    #[test]
    fn test_reflection_max() {
        assert_eq!(beam_reflection_max(DamageType::Magm, true), 3);
        assert_eq!(beam_reflection_max(DamageType::Fire, true), 0);
    }

    #[test]
    fn test_attenuation() {
        let d1 = beam_attenuation(20, 0, DamageType::Magm);
        let d2 = beam_attenuation(20, 5, DamageType::Magm);
        assert!(d1 > d2);
    }

    #[test]
    fn test_overcharge() {
        let mut rng = NetHackRng::new(42);
        // 초과 없으면 안전
        assert!(!wand_overcharge_check(5, 10, &mut rng));
    }

    #[test]
    fn test_terrain_interaction() {
        assert_eq!(
            beam_terrain_interaction(DamageType::Fire, "ice"),
            BeamTerrainEffect::MeltIce
        );
        assert_eq!(
            beam_terrain_interaction(DamageType::Cold, "water"),
            BeamTerrainEffect::FreezeWater
        );
    }

    #[test]
    fn test_beam_color() {
        assert_eq!(beam_color(DamageType::Fire), "orange-red");
    }

    #[test]
    fn test_recharge_cost() {
        assert!(wand_recharge_cost("wand of wishing") > wand_recharge_cost("wand of digging"));
    }

    #[test]
    fn test_beam_stats() {
        let mut s = BeamStatistics::new();
        s.record_fire(25);
        s.record_reflect();
        assert_eq!(s.beams_fired, 1);
        assert_eq!(s.total_beam_damage, 25);
    }
}
