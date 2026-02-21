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
    #[resource] action_queue: &mut crate::core::action_queue::ActionQueue,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] grid: &mut Grid,
    #[resource] rng: &mut NetHackRng,
) {
    let mut to_keep = Vec::new();
    let mut action_to_process = None;
    while let Some(game_action) = action_queue.pop() {
        if let crate::core::action_queue::GameAction::Zap(a) = game_action {
            action_to_process = Some(a);
        } else {
            to_keep.push(game_action);
        }
    }
    for a in to_keep {
        action_queue.push(a);
    }

    let action = match action_to_process {
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
// [v2.9.4] bhitm() 이식 — 몬스터 대상 즉시 효과 (원본: zap.c L133-481)
// 볼트/레이가 아닌 즉시 효과 지팡이(감속/가속/취소/텔레포트/투명화 등)의
// 몬스터 피격 처리. execute_bolt에서 호출하거나 독립적으로 사용 가능.
// =============================================================================

/// [v2.9.4] 즉시 효과 지팡이 종류 (원본: bhitm switch cases)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmediateEffect {
    /// WAN_STRIKING / SPE_FORCE_BOLT (원본: L157-179)
    Striking,
    /// WAN_SLOW_MONSTER / SPE_SLOW_MONSTER (원본: L180-193)
    SlowMonster,
    /// WAN_SPEED_MONSTER (원본: L194-203)
    SpeedMonster,
    /// WAN_UNDEAD_TURNING / SPE_TURN_UNDEAD (원본: L204-223)
    UndeadTurning,
    /// WAN_POLYMORPH / SPE_POLYMORPH (원본: L224-288)
    Polymorph,
    /// WAN_CANCELLATION / SPE_CANCELLATION (원본: L289-294)
    Cancellation,
    /// WAN_TELEPORTATION / SPE_TELEPORT_AWAY (원본: L295-300)
    Teleportation,
    /// WAN_MAKE_INVISIBLE (원본: L301-316)
    MakeInvisible,
    /// WAN_LOCKING / SPE_WIZARD_LOCK (원본: L317-320)
    Locking,
    /// WAN_PROBING (원본: L321-326)
    Probing,
    /// WAN_OPENING / SPE_KNOCK (원본: L327-362)
    Opening,
    /// SPE_HEALING (원본: L363-371)
    Healing,
    /// SPE_EXTRA_HEALING (원본: L364-397)
    ExtraHealing,
    /// SPE_DRAIN_LIFE (원본: L428-451)
    DrainLife,
    /// SPE_STONE_TO_FLESH (원본: L413-427)
    StoneToFlesh,
    /// WAN_NOTHING (원본: L452-454)
    Nothing,
}

/// [v2.9.4] 즉시 효과 적용 결과 (원본: bhitm 반환값 + 부수 효과)
#[derive(Debug, Clone)]
pub struct BhitResult {
    /// 대상이 깨어나야 하는지 (원본: wake)
    pub should_wake: bool,
    /// 대상이 사망했는지
    pub target_died: bool,
    /// 가한 데미지
    pub damage: i32,
    /// 투명 몬스터 위치 공개 (원본: reveal_invis)
    pub reveal_invisible: bool,
    /// 효과 학습 (원본: learn_it)
    pub learned: bool,
    /// 로그 메시지
    pub message: String,
    /// 텔레포트 발생 여부
    pub teleported: bool,
    /// 속도 변경 (-1=감속, 0=변화없음, 1=가속)
    pub speed_change: i32,
    /// 변신 발생 여부
    pub polymorphed: bool,
    /// 취소 여부
    pub cancelled: bool,
}

impl Default for BhitResult {
    fn default() -> Self {
        Self {
            should_wake: true,
            target_died: false,
            damage: 0,
            reveal_invisible: false,
            learned: false,
            message: String::new(),
            teleported: false,
            speed_change: 0,
            polymorphed: false,
            cancelled: false,
        }
    }
}

/// [v2.9.4] 즉시 효과 대상 정보 (원본: struct monst *mtmp)
#[derive(Debug, Clone)]
pub struct ZapTarget {
    /// 대상 이름
    pub name: String,
    /// 현재 HP
    pub hp: i32,
    /// 최대 HP
    pub hp_max: i32,
    /// 레벨
    pub level: i32,
    /// 마법 저항 여부 (원본: resists_magm)
    pub magic_resistant: bool,
    /// 생명력 흡수 저항 (원본: resists_drli)
    pub drain_resistant: bool,
    /// 언데드인지 (원본: is_undead)
    pub is_undead: bool,
    /// 뱀파이어 변신체인지 (원본: is_vampshifter)
    pub is_vampshifter: bool,
    /// 석상 골렘인지 (원본: PM_STONE_GOLEM)
    pub is_stone_golem: bool,
    /// 길들여진 상태인지 (원본: mtame)
    pub is_tame: bool,
    /// 평화적인지 (원본: mpeaceful)
    pub is_peaceful: bool,
    /// 위장 중인지 (원본: disguised_mimic)
    pub is_disguised: bool,
    /// 투명 상태인지 (원본: minvis)
    pub is_invisible: bool,
    /// 취소되었는지 (원본: mcan)
    pub is_cancelled: bool,
    /// 속도 상태 (-1: 느림, 0: 보통, 1: 빠름)
    pub speed: i32,
}

/// [v2.9.4] 즉시 효과 적용 (원본: bhitm, zap.c L133-481)
///
/// 몬스터에 대한 즉시 효과 지팡이/주문의 효과를 계산한다.
/// 실제 ECS 컴포넌트 수정은 호출부에서 BhitResult를 보고 적용한다.
pub fn bhitm(
    effect: ImmediateEffect,
    target: &ZapTarget,
    rng: &mut NetHackRng,
    is_spell: bool,
    is_blessed_spell: bool,
    caster_level: i32,
    is_knight_quest: bool,
) -> BhitResult {
    let mut result = BhitResult::default();
    let dbldam = is_knight_quest; // 기사 퀘스트 아이템 보유 시 2배 데미지

    match effect {
        // =====================================================================
        // 타격 (원본: WAN_STRIKING / SPE_FORCE_BOLT, L157-179)
        // =====================================================================
        ImmediateEffect::Striking => {
            result.reveal_invisible = true;
            if target.magic_resistant {
                result.message = format!("{}에게 빔이 튕겨나간다! Boing!", target.name);
                result.learned = false;
                result.should_wake = true;
            } else {
                let mut dmg = rng.d(2, 12);
                if dbldam {
                    dmg *= 2;
                }
                if is_spell {
                    dmg += caster_level / 3;
                } // spell_damage_bonus 근사
                result.damage = dmg;
                result.message = format!("{}을(를) 타격한다! ({}dmg)", target.name, dmg);
                result.learned = true;
            }
        }

        // =====================================================================
        // 감속 (원본: WAN_SLOW_MONSTER, L180-193)
        // =====================================================================
        ImmediateEffect::SlowMonster => {
            if !target.magic_resistant {
                result.speed_change = -1;
                result.message = format!("{}이(가) 느려진다!", target.name);
                result.reveal_invisible = true;
            } else {
                result.message = format!("{}은(는) 영향을 받지 않는다.", target.name);
            }
        }

        // =====================================================================
        // 가속 (원본: WAN_SPEED_MONSTER, L194-203)
        // =====================================================================
        ImmediateEffect::SpeedMonster => {
            if !target.magic_resistant {
                result.speed_change = 1;
                result.message = format!("{}이(가) 빨라진다!", target.name);
            } else {
                result.message = format!("{}은(는) 영향을 받지 않는다.", target.name);
            }
            if target.is_tame {
                result.should_wake = false; // 아군 가속은 깨우지 않음
            }
        }

        // =====================================================================
        // 언데드 퇴치 (원본: WAN_UNDEAD_TURNING, L204-223)
        // =====================================================================
        ImmediateEffect::UndeadTurning => {
            result.should_wake = false; // 기본적으로 깨우지 않음
            if target.is_undead || target.is_vampshifter {
                result.reveal_invisible = true;
                result.should_wake = true;
                let mut dmg = rng.rnd(8);
                if dbldam {
                    dmg *= 2;
                }
                if is_spell {
                    dmg += caster_level / 3;
                }
                result.damage = dmg;
                if !target.magic_resistant {
                    // 죽지 않았으면 도주 (호출부에서 처리)
                    result.message = format!(
                        "{}이(가) 퇴치의 힘에 {}를 입는다! ({}dmg)",
                        target.name,
                        if target.hp - dmg <= 0 {
                            "소멸"
                        } else {
                            "피해"
                        },
                        dmg
                    );
                } else {
                    result.damage = 0;
                    result.message = format!("{}은(는) 퇴치에 저항한다.", target.name);
                }
            }
        }

        // =====================================================================
        // 변신 (원본: WAN_POLYMORPH, L224-288)
        // =====================================================================
        ImmediateEffect::Polymorph => {
            if target.magic_resistant {
                result.message = format!("{}은(는) 마법 방어막으로 변신을 막는다!", target.name);
            } else if !target.magic_resistant {
                // 시스템 쇼크: 4% 확률로 즉사 (원본: !rn2(25))
                if rng.rn2(25) == 0 {
                    result.target_died = true;
                    result.message = format!("{}이(가) 시스템 쇼크로 사망한다!", target.name);
                    result.learned = true;
                } else {
                    result.polymorphed = true;
                    result.message = format!("{}이(가) 변신한다!", target.name);
                    result.learned = true;
                }
            }
        }

        // =====================================================================
        // 취소 (원본: WAN_CANCELLATION, L289-294)
        // =====================================================================
        ImmediateEffect::Cancellation => {
            result.cancelled = true;
            result.message = format!("{}이(가) 마법이 해제된다!", target.name);
            result.reveal_invisible = true;
        }

        // =====================================================================
        // 텔레포트 (원본: WAN_TELEPORTATION, L295-300)
        // =====================================================================
        ImmediateEffect::Teleportation => {
            result.teleported = true;
            result.message = format!("{}이(가) 텔레포트된다!", target.name);
            result.reveal_invisible = true;
        }

        // =====================================================================
        // 투명화 (원본: WAN_MAKE_INVISIBLE, L301-316)
        // =====================================================================
        ImmediateEffect::MakeInvisible => {
            if !target.is_invisible {
                result.message = format!("{}이(가) 투명해진다!", target.name);
                result.reveal_invisible = true;
                result.learned = true;
            } else {
                result.message = format!("{}은(는) 이미 투명하다.", target.name);
            }
        }

        // =====================================================================
        // 잠금 (원본: WAN_LOCKING, L317-320)
        // =====================================================================
        ImmediateEffect::Locking => {
            result.should_wake = false;
            result.message = format!("{}에게는 잠금 효과가 없다.", target.name);
            // 함정에 갇힌 몬스터를 잠그는 효과는 호출부에서 처리
        }

        // =====================================================================
        // 탐지 (원본: WAN_PROBING, L321-326)
        // =====================================================================
        ImmediateEffect::Probing => {
            result.should_wake = false;
            result.reveal_invisible = true;
            result.learned = true;
            result.message = format!(
                "{}을(를) 탐지한다! (HP: {}/{}, Lv: {})",
                target.name, target.hp, target.hp_max, target.level
            );
        }

        // =====================================================================
        // 열기 (원본: WAN_OPENING / SPE_KNOCK, L327-362)
        // =====================================================================
        ImmediateEffect::Opening => {
            result.should_wake = false;
            result.message = format!("{}에게는 열기 효과가 없다.", target.name);
            // 삼킨 몬스터 내부, 안장 분리 등은 호출부에서 처리
        }

        // =====================================================================
        // 치유 (원본: SPE_HEALING, L363-388)
        // =====================================================================
        ImmediateEffect::Healing => {
            result.reveal_invisible = true;
            let heal = rng.d(6, 4);
            result.damage = -heal; // 음수 = 회복
            result.should_wake = false;
            if target.is_tame || target.is_peaceful {
                result.message = format!("{}이(가) 치유된다. (HP +{})", target.name, heal);
            } else {
                result.message = format!("{}이(가) 좋아보인다. (HP +{})", target.name, heal);
            }
        }

        // =====================================================================
        // 고급 치유 (원본: SPE_EXTRA_HEALING, L364-397)
        // =====================================================================
        ImmediateEffect::ExtraHealing => {
            result.reveal_invisible = true;
            let heal = rng.d(6, 8);
            result.damage = -heal;
            result.should_wake = false;
            result.message = format!("{}이(가) 훨씬 좋아보인다! (HP +{})", target.name, heal);
        }

        // =====================================================================
        // 생명력 흡수 (원본: SPE_DRAIN_LIFE, L428-451)
        // =====================================================================
        ImmediateEffect::DrainLife => {
            if target.drain_resistant {
                result.message = format!("{}은(는) 생명력 흡수에 저항한다!", target.name);
            } else {
                // 원본: monhp_per_lvl(mtmp) — 레벨당 HP 근사
                let mut dmg = target.hp_max.max(1) / target.level.max(1);
                if dmg < 1 {
                    dmg = 1;
                }
                if dbldam {
                    dmg *= 2;
                }
                if is_spell {
                    dmg += caster_level / 3;
                }

                if target.hp - dmg <= 0 || target.level <= 1 {
                    result.target_died = true;
                    result.message = format!(
                        "{}이(가) 생명력을 빼앗겨 소멸한다! ({}dmg)",
                        target.name, dmg
                    );
                } else {
                    result.damage = dmg;
                    result.message = format!(
                        "{}이(가) 갑자기 약해 보인다! ({}dmg, Lv -1)",
                        target.name, dmg
                    );
                }
            }
        }

        // =====================================================================
        // 석화→육화 (원본: SPE_STONE_TO_FLESH, L413-427)
        // =====================================================================
        ImmediateEffect::StoneToFlesh => {
            if target.is_stone_golem {
                result.polymorphed = true;
                result.message = format!("{}이(가) 살점 골렘으로 변한다!", target.name);
            } else {
                result.should_wake = false;
                result.message = String::new(); // 효과 없음
            }
        }

        // =====================================================================
        // 무효과 (원본: WAN_NOTHING, L452-454)
        // =====================================================================
        ImmediateEffect::Nothing => {
            result.should_wake = false;
            result.message = String::new();
        }
    }

    result
}

/// [v2.9.4] 지팡이 이름으로 즉시 효과 유형 판별
/// 볼트/레이 계열(fire, cold, sleep, death, lightning)은 None 반환
pub fn classify_immediate_effect(wand_name: &str) -> Option<ImmediateEffect> {
    if wand_name.contains("striking") {
        return Some(ImmediateEffect::Striking);
    }
    if wand_name.contains("slow") {
        return Some(ImmediateEffect::SlowMonster);
    }
    if wand_name.contains("speed") {
        return Some(ImmediateEffect::SpeedMonster);
    }
    if wand_name.contains("undead") {
        return Some(ImmediateEffect::UndeadTurning);
    }
    if wand_name.contains("polymorph") {
        return Some(ImmediateEffect::Polymorph);
    }
    if wand_name.contains("cancellation") {
        return Some(ImmediateEffect::Cancellation);
    }
    if wand_name.contains("teleportation") || wand_name.contains("teleport") {
        return Some(ImmediateEffect::Teleportation);
    }
    if wand_name.contains("invisible") {
        return Some(ImmediateEffect::MakeInvisible);
    }
    if wand_name.contains("locking") || wand_name.contains("wizard lock") {
        return Some(ImmediateEffect::Locking);
    }
    if wand_name.contains("probing") {
        return Some(ImmediateEffect::Probing);
    }
    if wand_name.contains("opening") || wand_name.contains("knock") {
        return Some(ImmediateEffect::Opening);
    }
    if wand_name.contains("nothing") {
        return Some(ImmediateEffect::Nothing);
    }
    None // 볼트/레이 계열
}

// =============================================================================
// [v2.9.4] 아이템 파괴 시스템 (원본: destroy_one_item, zap.c L4700+)
// 빔이 인벤토리 아이템을 파괴하는 로직
// =============================================================================

/// [v2.9.4] 빔에 의해 파괴 가능한 아이템 종류 (원본: destroy_one_item)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestroyableItemType {
    /// 불꽃 빔 → 두루마리/물약/마법서 파괴
    Scroll,
    /// 불꽃 빔 → 물약 파괴 (끓임)
    Potion,
    /// 냉기 빔 → 물약 파괴 (동결)
    PotionFreeze,
    /// 번개 빔 → 마법봉 파괴 (과부하)
    Wand,
    /// 번개 빔 → 반지 파괴 (녹음)
    Ring,
}

/// [v2.9.4] 아이템 파괴 확률 (원본: destroy_percentage)
pub fn item_destroy_chance(dtype: DamageType, item_class: &str) -> i32 {
    match dtype {
        DamageType::Fire => {
            match item_class {
                "scroll" => 50,    // 두루마리: 50% (원본: L4730)
                "potion" => 50,    // 물약: 50% (원본: L4735)
                "spellbook" => 25, // 마법서: 25% (원본: L4740 근사)
                _ => 0,
            }
        }
        DamageType::Cold => {
            match item_class {
                "potion" => 25, // 물약 동결: 25% (원본: L4750)
                _ => 0,
            }
        }
        DamageType::Elec => {
            match item_class {
                "wand" => 25, // 마법봉 과부하: 25% (원본: L4760)
                "ring" => 25, // 반지: 25%
                _ => 0,
            }
        }
        _ => 0,
    }
}

/// [v2.9.4] 아이템 파괴 판정 (원본: destroy_one_item)
/// 반환: (파괴 여부, 메시지)
pub fn try_destroy_item(
    dtype: DamageType,
    item_name: &str,
    item_class: &str,
    rng: &mut NetHackRng,
) -> (bool, String) {
    let chance = item_destroy_chance(dtype, item_class);
    if chance <= 0 {
        return (false, String::new());
    }
    if rng.rn2(100) < chance {
        let verb = match dtype {
            DamageType::Fire => "타버린다",
            DamageType::Cold => "동결되어 깨진다",
            DamageType::Elec => "과부하로 파괴된다",
            _ => "파괴된다",
        };
        (true, format!("당신의 {}이(가) {}!", item_name, verb))
    } else {
        (false, String::new())
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

// =============================================================================
// [v2.9.4] bhitm / 아이템 파괴 테스트
// =============================================================================
#[cfg(test)]
mod bhitm_tests {
    use super::*;
    use crate::util::rng::NetHackRng;

    /// 테스트용 기본 대상 생성
    fn make_target() -> ZapTarget {
        ZapTarget {
            name: "고블린".to_string(),
            hp: 20,
            hp_max: 20,
            level: 3,
            magic_resistant: false,
            drain_resistant: false,
            is_undead: false,
            is_vampshifter: false,
            is_stone_golem: false,
            is_tame: false,
            is_peaceful: false,
            is_disguised: false,
            is_invisible: false,
            is_cancelled: false,
            speed: 0,
        }
    }

    #[test]
    fn test_striking_normal() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::Striking,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(result.damage > 0); // 2d12 최소 2
        assert!(result.reveal_invisible);
        assert!(result.learned);
    }

    #[test]
    fn test_striking_magic_resistant() {
        let mut target = make_target();
        target.magic_resistant = true;
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::Striking,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert_eq!(result.damage, 0);
        assert!(result.message.contains("Boing"));
    }

    #[test]
    fn test_slow_monster() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::SlowMonster,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert_eq!(result.speed_change, -1);
    }

    #[test]
    fn test_speed_monster() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::SpeedMonster,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert_eq!(result.speed_change, 1);
    }

    #[test]
    fn test_speed_tame_no_wake() {
        let mut target = make_target();
        target.is_tame = true;
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::SpeedMonster,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(!result.should_wake);
    }

    #[test]
    fn test_undead_turning_non_undead() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::UndeadTurning,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert_eq!(result.damage, 0); // 언데드가 아니면 무효
        assert!(!result.should_wake);
    }

    #[test]
    fn test_undead_turning_on_undead() {
        let mut target = make_target();
        target.is_undead = true;
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::UndeadTurning,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(result.damage > 0);
        assert!(result.should_wake);
    }

    #[test]
    fn test_cancellation() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::Cancellation,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(result.cancelled);
    }

    #[test]
    fn test_teleportation() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::Teleportation,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(result.teleported);
    }

    #[test]
    fn test_make_invisible() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::MakeInvisible,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(result.learned);
        assert!(result.message.contains("투명"));
    }

    #[test]
    fn test_probing() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::Probing,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(!result.should_wake);
        assert!(result.learned);
        assert!(result.message.contains("HP: 20/20"));
    }

    #[test]
    fn test_healing() {
        let target = make_target();
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::Healing,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(result.damage < 0); // 음수 = 회복
        assert!(!result.should_wake);
    }

    #[test]
    fn test_drain_life_resistant() {
        let mut target = make_target();
        target.drain_resistant = true;
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::DrainLife,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert_eq!(result.damage, 0);
        assert!(result.message.contains("저항"));
    }

    #[test]
    fn test_stone_to_flesh_golem() {
        let mut target = make_target();
        target.is_stone_golem = true;
        let mut rng = NetHackRng::new(42);
        let result = bhitm(
            ImmediateEffect::StoneToFlesh,
            &target,
            &mut rng,
            false,
            false,
            1,
            false,
        );
        assert!(result.polymorphed);
        assert!(result.message.contains("살점 골렘"));
    }

    #[test]
    fn test_classify_immediate_effect() {
        assert_eq!(
            classify_immediate_effect("wand of striking"),
            Some(ImmediateEffect::Striking)
        );
        assert_eq!(
            classify_immediate_effect("wand of slow monster"),
            Some(ImmediateEffect::SlowMonster)
        );
        assert_eq!(
            classify_immediate_effect("wand of speed monster"),
            Some(ImmediateEffect::SpeedMonster)
        );
        assert_eq!(
            classify_immediate_effect("wand of teleportation"),
            Some(ImmediateEffect::Teleportation)
        );
        assert_eq!(
            classify_immediate_effect("wand of cancellation"),
            Some(ImmediateEffect::Cancellation)
        );
        assert_eq!(
            classify_immediate_effect("wand of probing"),
            Some(ImmediateEffect::Probing)
        );
        assert_eq!(
            classify_immediate_effect("wand of nothing"),
            Some(ImmediateEffect::Nothing)
        );
        // 볼트/레이 계열은 None
        assert_eq!(classify_immediate_effect("wand of fire"), None);
        assert_eq!(classify_immediate_effect("wand of death"), None);
    }

    #[test]
    fn test_item_destroy_fire_scroll() {
        let mut rng = NetHackRng::new(42);
        // 여러 번 시도하여 최소 한 번은 파괴 발생 확인
        let mut destroyed = false;
        for _ in 0..20 {
            let (d, _msg) =
                try_destroy_item(DamageType::Fire, "scroll of identify", "scroll", &mut rng);
            if d {
                destroyed = true;
                break;
            }
        }
        assert!(destroyed, "20번 시도에서 두루마리 파괴가 한 번도 안 일어남");
    }
}
