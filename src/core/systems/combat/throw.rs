// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
use crate::assets::AssetManager;
use crate::core::dungeon::Grid;
use crate::core::entity::{CombatStats, Equipment, Health, Inventory, Item, PlayerTag, Position};
use crate::core::game_state::Direction;
use crate::ui::log::GameLog;
use crate::util::rng::NetHackRng;
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::*;

/// 던지기 액션 리소스
#[derive(Clone, Debug)]
pub struct ThrowAction {
    pub item: Entity,
    pub dir: Direction,
}

/// 원거리 공격 시스템 (Throw, Fire)
#[system]
#[read_component(crate::core::entity::player::Player)]
#[read_component(crate::core::entity::Equipment)]
#[read_component(CombatStats)]
#[write_component(Health)]
pub fn throw(
    world: &mut SubWorld,
    #[resource] action_queue: &mut crate::core::action_queue::ActionQueue,
    #[resource] grid: &Grid,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] _assets: &AssetManager,
    command_buffer: &mut CommandBuffer,
) {
    let mut to_keep = Vec::new();
    let mut action_to_process = None;
    while let Some(game_action) = action_queue.pop() {
        if let crate::core::action_queue::GameAction::Throw(a) = game_action {
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

    let item_ent = action.item;
    let dir = action.dir;

    // 1. 아이템 정보 수집 (Borrow 충돌 방지)
    let item_template_name = if let Ok(entry) = world.entry_ref(item_ent) {
        if let Ok(item) = entry.get_component::<Item>() {
            Some(item.kind.to_string())
        } else {
            None
        }
    } else {
        None
    };

    let item_name = item_template_name.unwrap_or_else(|| "item".to_string());

    let mut returned = false;
    let (player_ent, px, py, player_str_base) = {
        let mut player_query = <(
            Entity,
            &Position,
            &mut Inventory,
            &mut Equipment,
            &crate::core::entity::player::Player,
        )>::query()
        .filter(component::<PlayerTag>());
        let (p_ent, p_pos, p_inv, p_equip, player) = match player_query.iter_mut(world).next() {
            Some(res) => res,
            None => return,
        };

        let p_ent_val = *p_ent;
        let (px, py) = (p_pos.x, p_pos.y);
        log.add(format!("You throw the {}.", item_name), *turn);

        //
        if let Some(pos) = p_inv.items.iter().position(|&e| e == item_ent) {
            p_inv.items.remove(pos);
        } else {
            // Equipment 슬롯 순회
            let mut slot_to_remove = None;
            for (slot, &ent) in &p_equip.slots {
                if ent == item_ent {
                    slot_to_remove = Some(*slot);
                    break;
                }
            }
            if let Some(s) = slot_to_remove {
                p_equip.slots.remove(&s);
            }
        }
        (p_ent_val, px, py, player.str.base)
    };

    let (dx, dy) = dir.to_delta();

    // 투사체 경로 이동 및 충돌 체크
    let mut current_x = px;
    let mut current_y = py;

    // 투척 거리 계산 (NetHack 3.6.7 throw.c: 8 + (Str/2) - (Weight/40))
    let weight = if let Ok(entry) = world.entry_ref(item_ent) {
        entry
            .get_component::<Item>()
            .map(|i| i.weight)
            .unwrap_or(10)
    } else {
        10
    };
    let mut max_range = 8 + (player_str_base / 2) - (weight as i32 / 40);
    if max_range < 2 {
        max_range = 2;
    }
    if max_range > 15 {
        max_range = 15;
    }

    for _ in 0..max_range {
        let nx = current_x + dx;
        let ny = current_y + dy;

        // 범위 및 벽 체크
        if let Some(tile) = grid.get_tile(nx as usize, ny as usize) {
            if crate::core::systems::vision::VisionSystem::does_block(tile) {
                log.add(format!("The {} hits the wall.", item_name), *turn);
                break;
            }
        } else {
            break;
        }

        //
        let mut monster_hit_data: Option<(Entity, i32)> = None;
        let mut monster_query =
            <(Entity, &Position, &mut Health)>::query().filter(!component::<PlayerTag>());

        for (ent, pos, health) in monster_query.iter_mut(world) {
            if pos.x == nx && pos.y == ny {
                let mut rng = NetHackRng::new(*turn);
                let damage = rng.d(1, 6);
                health.current -= damage;
                monster_hit_data = Some((*ent, damage));
                break;
            }
        }

        if let Some((m_ent, dmg)) = monster_hit_data {
            // 49.2 Taming check
            let mut eaten = false;
            if let Ok(m_entry) = world.entry_ref(m_ent) {
                if let Ok(monster) = m_entry.get_component::<crate::core::entity::Monster>() {
                    //
                    if item_name.contains("lichen")
                        || item_name.contains("tripe")
                        || item_name.contains("meat")
                    {
                        if let Some(mt) = _assets.monsters.get_by_kind(monster.kind) {
                            if mt.has_capability(
                                crate::core::entity::capability::MonsterCapability::Animal,
                            ) {
                                log.add(
                                    format!(
                                        "The {} eats the {} and is tamed!",
                                        monster.kind, item_name
                                    ),
                                    *turn,
                                );
                                //
                                command_buffer.add_component(
                                    m_ent,
                                    crate::core::entity::monster::Pet {
                                        owner: player_ent,
                                        name: None,
                                        hunger: 0,
                                        loyalty: 10,
                                    },
                                );
                                command_buffer.add_component(
                                    m_ent,
                                    crate::core::entity::Monster {
                                        kind: monster.kind,
                                        hostile: false,
                                        mon_name: monster.mon_name.clone(),
                                    },
                                );
                                eaten = true;
                            }
                        }
                    }
                }
            }

            if eaten {
                command_buffer.remove(item_ent);
                returned = true; // 더 이상 땅에 떨어뜨리지 않음
            } else {
                log.add(
                    format!("The {} hits the monster for {} damage!", item_name, dmg),
                    *turn,
                );
            }
            current_x = nx;
            current_y = ny;
            break;
        }

        current_x = nx;
        current_y = ny;
    }

    //
    let is_mjollnir = if let Ok(entry) = world.entry_ref(item_ent) {
        if let Ok(item) = entry.get_component::<Item>() {
            item.artifact.as_deref() == Some("Mjollnir")
        } else {
            false
        }
    } else {
        false
    };

    if is_mjollnir {
        // Valkyrie 체크
        let mut p_query =
            <&crate::core::entity::player::Player>::query().filter(component::<PlayerTag>());
        if let Some(player) = p_query.iter(world).next() {
            if player.role == crate::core::entity::player::PlayerClass::Valkyrie {
                log.add("The Mjollnir returns to your hand!", *turn);
                let mut query = <&mut Inventory>::query().filter(component::<PlayerTag>());
                if let Some(inv) = query.iter_mut(world).next() {
                    inv.items.push(item_ent);
                    returned = true;
                }
            }
        }
    }

    if !returned {
        command_buffer.add_component(
            item_ent,
            Position {
                x: current_x,
                y: current_y,
            },
        );
    }

    // 액션 소비 완료
}

// =============================================================================
// [v2.3.1] dothrow.c 확장 이식
// 원본: nethack-3.6.7/src/dothrow.c (2,025줄)
//
// 투척 종류 분류, 투척 데미지, 특수 투척 효과 등
// =============================================================================

/// [v2.3.1] 투척 가능 아이템 분류 (원본: dothrow.c)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrowableType {
    /// 단검/수리검 (시전/바우트)
    Dagger,
    /// 창/자벨린
    Spear,
    /// 도끼
    Axe,
    /// 부메랑
    Boomerang,
    ///
    Ammo,
    /// 물약 (투척 시 깨짐)
    Potion,
    /// 알 (투척 시 깨짐)
    Egg,
    ///
    CreamPie,
    /// 기타
    Other,
}

///
pub fn classify_throwable(name: &str) -> ThrowableType {
    if name.contains("dagger") || name.contains("shuriken") || name.contains("dart") {
        ThrowableType::Dagger
    } else if name.contains("spear") || name.contains("javelin") || name.contains("trident") {
        ThrowableType::Spear
    } else if name.contains("axe") {
        ThrowableType::Axe
    } else if name.contains("boomerang") {
        ThrowableType::Boomerang
    } else if name.contains("arrow") || name.contains("bolt") || name.contains("crossbow bolt") {
        ThrowableType::Ammo
    } else if name.contains("potion") {
        ThrowableType::Potion
    } else if name.contains("egg") {
        ThrowableType::Egg
    } else if name.contains("cream pie") {
        ThrowableType::CreamPie
    } else {
        ThrowableType::Other
    }
}

/// [v2.3.1] 투척 기본 데미지 (원본: dothrow.c thitu)
pub fn throw_base_damage(throwable: ThrowableType, rng: &mut NetHackRng) -> i32 {
    match throwable {
        ThrowableType::Dagger => rng.d(1, 4),
        ThrowableType::Spear => rng.d(1, 6),
        ThrowableType::Axe => rng.d(1, 6) + 1,
        ThrowableType::Boomerang => rng.d(1, 9),
        ThrowableType::Ammo => rng.d(1, 6),
        ThrowableType::Potion => 1,
        ThrowableType::Egg => 1,
        ThrowableType::CreamPie => 0,
        ThrowableType::Other => rng.d(1, 4),
    }
}

/// [v2.3.1] 투척 거리 수정 (원본: dothrow.c)
pub fn throw_range_modifier(throwable: ThrowableType) -> i32 {
    match throwable {
        ThrowableType::Dagger | ThrowableType::Ammo => 2,
        ThrowableType::Spear | ThrowableType::Axe => 0,
        ThrowableType::Boomerang => 4,
        ThrowableType::Potion | ThrowableType::Egg | ThrowableType::CreamPie => -2,
        ThrowableType::Other => -3,
    }
}

///
pub fn throw_breaks_on_impact(throwable: ThrowableType, rng: &mut NetHackRng) -> bool {
    match throwable {
        ThrowableType::Potion | ThrowableType::Egg => true, // 항상 파괴
        ThrowableType::CreamPie => true,
        ThrowableType::Dagger | ThrowableType::Ammo => rng.rn2(5) == 0, // 20% 확률 파괴
        _ => false,
    }
}

/// [v2.3.1] 물약 투척 효과 (원본: potionhit in potion.c, dothrow.c)
pub fn potion_throw_effect(potion_name: &str) -> PotionThrowResult {
    if potion_name.contains("acid") {
        PotionThrowResult {
            message: "The acid burns!",
            damage: 0,
            status_effect: Some("acid_burn"),
            area: false,
        }
    } else if potion_name.contains("sleeping") || potion_name.contains("sleep") {
        PotionThrowResult {
            message: "The fumes put the target to sleep!",
            damage: 0,
            status_effect: Some("sleep"),
            area: true,
        }
    } else if potion_name.contains("blindness") {
        PotionThrowResult {
            message: "The splashing liquid blinds the target!",
            damage: 0,
            status_effect: Some("blind"),
            area: false,
        }
    } else if potion_name.contains("confusion") {
        PotionThrowResult {
            message: "The fumes confuse the target!",
            damage: 0,
            status_effect: Some("confused"),
            area: true,
        }
    } else if potion_name.contains("paralysis") {
        PotionThrowResult {
            message: "The target is frozen in place!",
            damage: 0,
            status_effect: Some("paralyzed"),
            area: false,
        }
    } else if potion_name.contains("hallucination") {
        PotionThrowResult {
            message: "The target starts seeing things!",
            damage: 0,
            status_effect: Some("hallu"),
            area: true,
        }
    } else if potion_name.contains("polymorph") {
        PotionThrowResult {
            message: "The target transforms!",
            damage: 0,
            status_effect: Some("polymorph"),
            area: false,
        }
    } else if potion_name.contains("healing") || potion_name.contains("extra healing") {
        PotionThrowResult {
            message: "The target looks healthier.",
            damage: -10,
            status_effect: None,
            area: false,
        }
    } else if potion_name.contains("oil") {
        PotionThrowResult {
            message: "The oil splashes everywhere!",
            damage: 0,
            status_effect: Some("greasy"),
            area: true,
        }
    } else {
        PotionThrowResult {
            message: "The potion shatters!",
            damage: 0,
            status_effect: None,
            area: false,
        }
    }
}

/// [v2.3.1] 물약 투척 결과
#[derive(Debug, Clone)]
pub struct PotionThrowResult {
    pub message: &'static str,
    pub damage: i32,
    pub status_effect: Option<&'static str>,
    pub area: bool,
}

///
pub fn cream_pie_effect() -> &'static str {
    "Splat! The cream pie blinds the target!"
}

/// [v2.3.1] 알 투척 효과 (원본: dothrow.c)
pub fn egg_throw_effect(is_cockatrice: bool) -> &'static str {
    if is_cockatrice {
        "The cockatrice egg shatters! The target begins turning to stone!"
    } else {
        "The egg splatters harmlessly."
    }
}

/// [v2.3.1] 부메랑 궤적 계산 (원본: dothrow.c hurtle_step)
///
pub fn boomerang_path(start_x: i32, start_y: i32, dx: i32, dy: i32, range: i32) -> Vec<(i32, i32)> {
    let mut path = Vec::new();
    let mut x = start_x;
    let mut y = start_y;

    // 전반부: 직진
    for _ in 0..(range / 2) {
        x += dx;
        y += dy;
        path.push((x, y));
    }

    // 중간: 측면 이동
    let (side_dx, side_dy) = if dx != 0 { (0, 1) } else { (1, 0) };
    for _ in 0..2 {
        x += side_dx;
        y += side_dy;
        path.push((x, y));
    }

    // 후반부: 반대 직진 (돌아옴)
    for _ in 0..(range / 2) {
        x -= dx;
        y -= dy;
        path.push((x, y));
    }

    // 측면 복귀
    for _ in 0..2 {
        x -= side_dx;
        y -= side_dy;
        path.push((x, y));
    }

    path
}

/// [v2.3.1] 투척 무기 역할별 보너스 (원본: dothrow.c skill_bonus)
pub fn throw_skill_bonus(role: &str, throwable: ThrowableType) -> i32 {
    match role {
        "Rogue" => match throwable {
            ThrowableType::Dagger => 3,
            _ => 0,
        },
        "Samurai" => match throwable {
            ThrowableType::Dagger => 2, // 수리검
            _ => 0,
        },
        "Ranger" => match throwable {
            ThrowableType::Ammo => 3,
            ThrowableType::Spear => 1,
            _ => 0,
        },
        "Valkyrie" => match throwable {
            ThrowableType::Spear => 2,
            ThrowableType::Axe => 1,
            _ => 0,
        },
        _ => 0,
    }
}

/// [v2.3.1] 발사체 착탄 판정 (원본: thitu)
pub fn throw_hit_chance(
    player_dex: i32,
    distance: i32,
    target_ac: i32,
    skill_bonus: i32,
    rng: &mut NetHackRng,
) -> bool {
    let to_hit = player_dex + skill_bonus - distance - target_ac;
    rng.rn2(20) < to_hit.max(1)
}

// =============================================================================
// [v2.9.0] dothrow.c 대량 이식  hurtle/multishot/gem/walking_missile
// 원본: nethack-3.6.7/src/dothrow.c (2,025줄)
// =============================================================================

/// [v2.9.0] 밀려남 결과 (원본: dothrow.c:1200-1280 hurtle)
#[derive(Debug, Clone)]
pub struct HurtleResult {
    /// 밀려난 거리
    pub distance: i32,
    /// 최종 위치
    pub final_pos: (i32, i32),
    /// 벽/장애물에 충돌했는지
    pub hit_obstacle: bool,
    /// 충돌 데미지
    pub collision_damage: i32,
    /// 메시지
    pub message: String,
}

/// [v2.9.0] 밀려남 계산 (원본: dothrow.c hurtle)
/// 플레이어나 몬스터가 힘에 의해 밀려나는 효과
pub fn hurtle_calc(
    start_x: i32,
    start_y: i32,
    dx: i32,
    dy: i32,
    range: i32,
    is_player: bool,
    check_walkable: &dyn Fn(i32, i32) -> bool,
) -> HurtleResult {
    let mut x = start_x;
    let mut y = start_y;
    let mut dist = 0;
    let mut hit_obs = false;

    for _ in 0..range {
        let nx = x + dx;
        let ny = y + dy;
        if !check_walkable(nx, ny) {
            hit_obs = true;
            break;
        }
        x = nx;
        y = ny;
        dist += 1;
    }

    // 충돌 데미지: 벽에 부딪힌 경우
    let collision_dmg = if hit_obs { (range - dist).max(1) } else { 0 };
    let msg = if hit_obs {
        if is_player {
            "You slam into an obstacle!".to_string()
        } else {
            "The monster slams into an obstacle!".to_string()
        }
    } else if is_player {
        format!("You are pushed {} squares!", dist)
    } else {
        format!("The monster is pushed {} squares!", dist)
    };

    HurtleResult {
        distance: dist,
        final_pos: (x, y),
        hit_obstacle: hit_obs,
        collision_damage: collision_dmg,
        message: msg,
    }
}

/// [v2.9.0] 밀려남 거리 계산 (원본: dothrow.c:1283 hurtle_step)
/// 큰 몬스터는 밀려남이 줄어듦
pub fn hurtle_range(weight: i32, force: i32) -> i32 {
    let base = force * 3 / weight.max(1);
    base.clamp(1, 8)
}

/// [v2.9.0] 다중 발사(Multishot) 판정 (원본: dothrow.c:390-475 multishot)
/// 역할/숙련도에 따라 한 턴에 여러 발 발사
pub fn multishot_count(
    role: &str,
    throwable: ThrowableType,
    skill_level: i32, // 0=Unskilled, 1=Basic, 2=Skilled, 3=Expert
    player_level: i32,
    _rng: &mut NetHackRng,
) -> i32 {
    let mut count = 1;

    // 숙련도 보너스: Skilled +1, Expert +2
    if skill_level >= 2 {
        count += 1;
    }
    if skill_level >= 3 {
        count += 1;
    }

    // 레인저 특수: 화살/볼트 Skilled 이상이면 추가
    if role == "Ranger" && matches!(throwable, ThrowableType::Ammo) && skill_level >= 2 {
        count += 1;
    }

    // 사무라이 특수: 수리검(단검류) Expert이면 추가
    if role == "Samurai" && matches!(throwable, ThrowableType::Dagger) && skill_level >= 3 {
        count += 1;
    }

    // 레벨 보너스: 20레벨 이상이면 1발 추가
    if player_level >= 20 {
        count += 1;
    }

    // 최대 제한
    count.min(5)
}

/// [v2.9.0] 보석 수락 (원본: dothrow.c:505-585 gem_accept)
/// 유니콘에게 보석을 던지면 행운이 변함
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemAcceptResult {
    /// 유니콘이 보석을 수락 (행운 증가)
    Accepted,
    /// 유니콘이 유리를 거부 (행운 감소)
    Rejected,
    /// 유니콘이 무시
    Ignored,
}

pub fn gem_accept(
    _gem_name: &str,
    is_valuable: bool,
    unicorn_same_alignment: bool,
) -> (GemAcceptResult, i32, &'static str) {
    if !is_valuable {
        // 유리/무가치 돌  행운 감소
        return (
            GemAcceptResult::Rejected,
            -1,
            "The unicorn is not impressed.",
        );
    }

    if unicorn_same_alignment {
        // 같은 성향 유니콘  큰 행운 증가
        (
            GemAcceptResult::Accepted,
            5,
            "The unicorn graciously accepts the gem!",
        )
    } else {
        // 다른 성향 유니콘  작은 행운 증가
        (
            GemAcceptResult::Accepted,
            2,
            "The unicorn takes the gem and leaves.",
        )
    }
}

/// [v2.9.0] 걷는 미사일 (원본: dothrow.c walking_missile 개념)
/// 밀려남에 의해 이동 중인 엔티티가 각 타일에서 효과를 발생
pub fn walking_missile_check(tile_type: &str, is_player: bool) -> Option<&'static str> {
    match tile_type {
        "lava" => Some(if is_player {
            "You are pushed into lava!"
        } else {
            "The monster falls into lava!"
        }),
        "water" | "pool" => Some(if is_player {
            "You fall into the water!"
        } else {
            "The monster falls into water!"
        }),
        "trap_pit" | "pit" => Some(if is_player {
            "You fall into a pit!"
        } else {
            "The monster falls into a pit!"
        }),
        "trap_hole" => Some(if is_player {
            "You fall through a hole!"
        } else {
            "The monster falls through a hole!"
        }),
        _ => None,
    }
}

/// [v2.9.0] 투척 시 파괴 판정 확장 (원본: dothrow.c:breakobj)
/// 유리, 물약, 알, 크리스탈볼 등 파괴되는 아이템
pub fn breakobj_check(
    item_name: &str,
    hit_wall: bool,
    rng: &mut NetHackRng,
) -> (bool, &'static str) {
    let l = item_name.to_lowercase();

    // 물약  항상 파괴
    if l.contains("potion") {
        return (true, "The potion shatters!");
    }
    // 알  항상 파괴
    if l.contains("egg") {
        return (true, "The egg splatters!");
    }
    // 유리  높은 확률
    if l.contains("mirror") || l.contains("crystal ball") {
        if hit_wall || rng.rn2(3) == 0 {
            return (true, "The glass shatters into pieces!");
        }
    }
    // 빈 병  벽에 부딪히면 파괴
    if l.contains("empty bottle") && hit_wall {
        return (true, "The bottle shatters!");
    }
    // 일반 경우
    (false, "")
}

/// [v2.9.0] 투척 메시지 생성 (원본: dothrow.c:245-280)
pub fn throw_message(item_name: &str, target_name: &str, damage: i32, is_kill: bool) -> String {
    if is_kill {
        format!("The {} strikes {} fatally!", item_name, target_name)
    } else if damage > 0 {
        format!(
            "The {} hits {} for {} damage!",
            item_name, target_name, damage
        )
    } else {
        format!("The {} misses {}.", item_name, target_name)
    }
}

/// [v2.9.0] 러너 체크  몬스터가 투척물을 맞고 도주
pub fn monster_flees_from_throw(
    monster_hp: i32,
    monster_max_hp: i32,
    damage: i32,
    is_peaceful: bool,
    rng: &mut NetHackRng,
) -> bool {
    // HP가 1/3 이하이고 평화적이지 않으면 도주 확률 있음
    let remaining = monster_hp - damage;
    if remaining <= monster_max_hp / 3 && !is_peaceful {
        rng.rn2(3) == 0
    } else {
        false
    }
}

/// [v2.9.0] 투척된 아이템이 스택에 합류 가능한지 판정
pub fn can_merge_thrown(
    thrown_name: &str,
    ground_name: &str,
    thrown_blessed: bool,
    ground_blessed: bool,
    thrown_cursed: bool,
    ground_cursed: bool,
) -> bool {
    thrown_name == ground_name && thrown_blessed == ground_blessed && thrown_cursed == ground_cursed
}

/// [v2.9.0] 투척 데미지에 대한 아이템 보정 (spe, BUC)
pub fn throw_damage_adjustment(
    base_damage: i32,
    spe: i32,
    is_blessed: bool,
    is_cursed: bool,
) -> i32 {
    let mut dmg = base_damage + spe;
    if is_blessed {
        dmg += 1;
    }
    if is_cursed {
        dmg -= 1;
    }
    dmg.max(0)
}

/// [v2.9.0] 투척 방향 유효성 검증
pub fn validate_throw_direction(dx: i32, dy: i32) -> bool {
    // 0,0은 자기 자신에게  유효하지 않음
    dx != 0 || dy != 0
}

/// [v2.9.0] 골렘에게 투척 시 특수 효과 (원본: dothrow.c thitmonst)
pub fn throw_at_golem(golem_type: &str, item_name: &str) -> Option<(i32, &'static str)> {
    let l_item = item_name.to_lowercase();
    let l_gol = golem_type.to_lowercase();

    // 아이언 골렘 + 철제 투척물  회복
    if l_gol.contains("iron") && (l_item.contains("iron") || l_item.contains("steel")) {
        return Some((0, "The iron golem absorbs the metal!"));
    }
    // 플레시 골렘 + 음식  회복
    if l_gol.contains("flesh") && (l_item.contains("corpse") || l_item.contains("meat")) {
        return Some((0, "The flesh golem absorbs the organic matter!"));
    }
    None
}

// =============================================================================
// 테스트
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify() {
        assert_eq!(classify_throwable("silver dagger"), ThrowableType::Dagger);
        assert_eq!(classify_throwable("javelin"), ThrowableType::Spear);
        assert_eq!(classify_throwable("boomerang"), ThrowableType::Boomerang);
        assert_eq!(classify_throwable("arrow"), ThrowableType::Ammo);
    }

    #[test]
    fn test_throw_range() {
        assert_eq!(throw_range_modifier(ThrowableType::Boomerang), 4);
        assert_eq!(throw_range_modifier(ThrowableType::Potion), -2);
    }

    #[test]
    fn test_hurtle_no_obstacle() {
        let result = hurtle_calc(5, 5, 1, 0, 3, true, &|_, _| true);
        assert_eq!(result.distance, 3);
        assert_eq!(result.final_pos, (8, 5));
        assert!(!result.hit_obstacle);
    }

    #[test]
    fn test_hurtle_with_wall() {
        let result = hurtle_calc(5, 5, 1, 0, 5, true, &|x, _| x < 8);
        assert!(result.hit_obstacle);
        assert_eq!(result.final_pos, (7, 5));
    }

    #[test]
    fn test_hurtle_range() {
        assert_eq!(hurtle_range(100, 20), 1);
        assert_eq!(hurtle_range(10, 30), 8); // clamped to 8
    }

    #[test]
    fn test_multishot_basic() {
        let mut rng = NetHackRng::new(42);
        let count = multishot_count("Ranger", ThrowableType::Ammo, 3, 20, &mut rng);
        assert!(count >= 3);
        assert!(count <= 5);
    }

    #[test]
    fn test_multishot_samurai() {
        let mut rng = NetHackRng::new(42);
        let count = multishot_count("Samurai", ThrowableType::Dagger, 3, 10, &mut rng);
        assert!(count >= 4); // Expert + Samurai bonus
    }

    #[test]
    fn test_gem_accept_valuable() {
        let (result, luck, _) = gem_accept("diamond", true, true);
        assert_eq!(result, GemAcceptResult::Accepted);
        assert_eq!(luck, 5);
    }

    #[test]
    fn test_gem_accept_glass() {
        let (result, luck, _) = gem_accept("worthless glass", false, true);
        assert_eq!(result, GemAcceptResult::Rejected);
        assert_eq!(luck, -1);
    }

    #[test]
    fn test_walking_missile() {
        assert!(walking_missile_check("lava", true).is_some());
        assert!(walking_missile_check("floor", true).is_none());
    }

    #[test]
    fn test_breakobj_potion() {
        let mut rng = NetHackRng::new(1);
        let (broken, _) = breakobj_check("potion of healing", false, &mut rng);
        assert!(broken);
    }

    #[test]
    fn test_breakobj_mirror() {
        let mut rng = NetHackRng::new(1);
        let (broken, _) = breakobj_check("mirror", true, &mut rng);
        assert!(broken);
    }

    #[test]
    fn test_throw_message() {
        let msg = throw_message("dagger", "orc", 5, false);
        assert!(msg.contains("hits"));
        let msg2 = throw_message("dagger", "orc", 5, true);
        assert!(msg2.contains("fatally"));
    }

    #[test]
    fn test_damage_adjustment() {
        assert_eq!(throw_damage_adjustment(5, 2, true, false), 8); // 5+2+1
        assert_eq!(throw_damage_adjustment(5, 0, false, true), 4); // 5-1
    }

    #[test]
    fn test_validate_direction() {
        assert!(validate_throw_direction(1, 0));
        assert!(!validate_throw_direction(0, 0));
    }

    #[test]
    fn test_throw_at_golem() {
        let r = throw_at_golem("iron golem", "iron chain");
        assert!(r.is_some());
        assert!(throw_at_golem("stone golem", "iron chain").is_none());
    }

    #[test]
    fn test_can_merge() {
        assert!(can_merge_thrown(
            "arrow", "arrow", false, false, false, false
        ));
        assert!(!can_merge_thrown(
            "arrow", "bolt", false, false, false, false
        ));
    }
}
