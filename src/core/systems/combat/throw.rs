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
    #[resource] throw_action: &mut Option<ThrowAction>,
    #[resource] grid: &Grid,
    #[resource] log: &mut GameLog,
    #[resource] turn: &u64,
    #[resource] _assets: &AssetManager,
    command_buffer: &mut CommandBuffer,
) {
    let action = match throw_action {
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

    // 액션 소비
    *throw_action = None;
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
