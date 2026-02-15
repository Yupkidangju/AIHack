// Copyright 2026 Yupkidangju. Licensed under Apache-2.0.
// Based on NetHack 3.6.7 (NGPL). See LICENSE and LICENSE.NGPL.
//! 용기 및 보관함 시스템 (Loot)
//! 원본 NetHack의 invent.c, pickup.c 기반

use crate::assets::AssetManager;
use crate::core::entity::{ContainerTag, Inventory, Item, PlayerTag, Position};
use crate::ui::log::GameLog;
use legion::systems::CommandBuffer;
use legion::{component, Entity, EntityStore, IntoQuery};

/// 루팅 시도 (L 명령)
pub fn try_loot(
    direction: crate::core::game_state::Direction,
    world: &mut legion::World,
    _assets: &AssetManager,
    log: &mut GameLog,
    turn: u64,
    _command_buffer: &mut CommandBuffer,
    state: &mut crate::core::game_state::GameState,
) {
    let mut player_pos = None;
    let mut query = <&Position>::query().filter(component::<PlayerTag>());
    for pos in query.iter(world) {
        player_pos = Some((pos.x, pos.y));
    }

    if let Some((px, py)) = player_pos {
        let (dx, dy) = direction.to_delta();
        let tx = px + dx;
        let ty = py + dy;

        // 1. 해당 위치에 용기가 있는지 확인
        let mut container_ent = None;
        let mut item_name = String::new();
        let mut is_locked = false;

        // 용기 태그가 붙은 아이템 검색
        let mut container_query =
            <(Entity, &Position, &Item)>::query().filter(component::<ContainerTag>());
        for (ent, pos, item) in container_query.iter(world) {
            if pos.x == tx && pos.y == ty {
                container_ent = Some(*ent);
                item_name = item.kind.to_string();
                is_locked = item.olocked;
                break;
            }
        }

        if let Some(ent) = container_ent {
            if is_locked {
                log.add(format!("The {} is locked.", item_name), turn);
            } else {
                log.add(format!("You open the {}.", item_name), turn);
                loot_container(ent, world, log, turn, state);
            }
        } else {
            log.add("There is nothing to loot there.", turn);
        }
    }
}

/// 용기 내용물 처리
fn loot_container(
    container_ent: Entity,
    world: &mut legion::World,
    log: &mut GameLog,
    turn: u64,
    state: &mut crate::core::game_state::GameState,
) {
    //
    if let Ok(entry) = world.entry_ref(container_ent) {
        if let Ok(inv) = entry.get_component::<Inventory>() {
            if inv.items.is_empty() {
                log.add("It is empty.", turn);
            } else {
                log.add(format!("It contains {} items.", inv.items.len()), turn);
                // [v1.3.0] 루팅 UI 상태로 전환
                *state = crate::core::game_state::GameState::Looting {
                    container: container_ent,
                };
            }
        }
    }
}
