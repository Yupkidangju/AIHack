use crate::core::dungeon::Grid;
use crate::core::entity::{PlayerTag, Position};
use crate::core::systems::vision::VisionSystem;
use legion::*;

/// [v3.0.0] GameContext 기반 전환 완료
pub fn vision_update_system(ctx: &mut crate::core::context::GameContext) {
    use crate::core::entity::status::{StatusBundle, StatusFlags};
    use crate::core::entity::{Inventory, Level, LightSource, PlayerTag, Position};

    //
    let mut p_q = <(&Position, &Level, &StatusBundle, &PlayerTag)>::query();
    let p_info: Option<(Position, Level, StatusFlags)> = p_q
        .iter(ctx.world)
        .next()
        .map(|(p, l, s, _)| (*p, *l, s.flags()));

    let (p_pos, p_lvl, p_flags) = match p_info {
        Some(info) => info,
        None => return,
    };

    ctx.vision.update_clear_zones(ctx.grid);

    //
    if p_flags.contains(StatusFlags::BLIND) {
        ctx.vision.reset_sight();
        //
        ctx.vision
            .apply_vision(ctx.grid, p_pos.x as usize, p_pos.y as usize, 1);
        return;
    }

    //
    let is_dark = if let Some(tile) = ctx.grid.get_tile(p_pos.x as usize, p_pos.y as usize) {
        !tile
            .flags
            .contains(crate::core::dungeon::tile::TileFlags::LIT)
    } else {
        false
    };

    let base_radius = if is_dark {
        if p_flags.contains(StatusFlags::NIGHT_VISION) {
            3
        } else {
            1
        }
    } else {
        5
    };

    ctx.vision
        .recalc(ctx.grid, p_pos.x as usize, p_pos.y as usize, base_radius);

    // 조명원 수집 (Gather)
    let mut light_positions = Vec::new();
    {
        let mut f_q = <(&Position, &LightSource, &Level)>::query();
        for (pos, light, lvl) in f_q.iter(ctx.world) {
            if light.lit && lvl.0 == p_lvl.0 {
                light_positions.push((pos.x as usize, pos.y as usize, light.range));
            }
        }
    }
    // Apply
    for (lx, ly, range) in &light_positions {
        ctx.vision.apply_vision(ctx.grid, *lx, *ly, *range);
    }

    // 인벤토리 내 조명원 수집 (Gather)
    let mut inv_lights = Vec::new();
    {
        let mut c_q = <(&Position, &Inventory, &Level)>::query();
        for (pos, inv, lvl) in c_q.iter(ctx.world) {
            if lvl.0 == p_lvl.0 {
                for &item_ent in &inv.items {
                    inv_lights.push((pos.x as usize, pos.y as usize, item_ent));
                }
            }
        }
    }
    // Apply
    for (px, py, item_ent) in inv_lights {
        if let Ok(entry) = ctx.world.entry_ref(item_ent) {
            if let Ok(light) = entry.get_component::<LightSource>() {
                if light.lit {
                    ctx.vision.apply_vision(ctx.grid, px, py, light.range);
                }
            }
        }
    }

    // 적외선 시야
    if p_flags.contains(StatusFlags::INFRAVISION) {
        let mut infra_targets = Vec::new();
        {
            let mut m_q = <(&Position, &Level, &crate::core::entity::MonsterTag)>::query();
            for (m_pos, m_lvl, _) in m_q.iter(ctx.world) {
                if m_lvl.0 == p_lvl.0 {
                    let dx = (m_pos.x - p_pos.x).abs();
                    let dy = (m_pos.y - p_pos.y).abs();
                    if dx <= 8 && dy <= 8 {
                        infra_targets.push((m_pos.x as usize, m_pos.y as usize));
                    }
                }
            }
        }
        for (mx, my) in infra_targets {
            if VisionSystem::has_line_of_sight(ctx.grid, p_pos.x as usize, p_pos.y as usize, mx, my)
            {
                ctx.vision.viz_array[mx][my] |= crate::core::systems::vision::IN_SIGHT;
            }
        }
    }
}

/// [v3.0.0] GameContext 기반 전환 완료
pub fn magic_map_effect_system(ctx: &mut crate::core::context::GameContext) {
    // Gather: 매직맵 요청이 있는 엔티티 수집
    let mut entities_with_request = Vec::new();
    {
        let mut query = <(Entity, &crate::core::entity::MagicMapRequest)>::query()
            .filter(component::<PlayerTag>());
        for (ent, _) in query.iter(ctx.world) {
            entities_with_request.push(*ent);
        }
    }
    // Apply: 매직맵 실행 + 컴포넌트 제거
    for ent in entities_with_request {
        ctx.vision.magic_map();
        if let Some(mut entry) = ctx.world.entry(ent) {
            entry.remove_component::<crate::core::entity::MagicMapRequest>();
        }
    }
}
