use crate::core::dungeon::Grid;
use crate::core::entity::{PlayerTag, Position};
use crate::core::systems::vision::VisionSystem;
use legion::world::SubWorld;
use legion::*;

///
#[legion::system]
#[read_component(Position)]
#[read_component(PlayerTag)]
#[read_component(crate::core::entity::LightSource)]
#[read_component(crate::core::entity::Inventory)]
#[read_component(crate::core::entity::Level)]
#[read_component(crate::core::entity::Item)]
#[read_component(crate::core::entity::status::StatusBundle)]
#[read_component(crate::core::entity::MonsterTag)]
pub fn vision_update(
    world: &mut SubWorld,
    #[resource] grid: &Grid,
    #[resource] vision: &mut VisionSystem,
) {
    use crate::core::entity::status::{StatusBundle, StatusFlags};
    use crate::core::entity::{Inventory, Level, LightSource, PlayerTag, Position};

    //
    let mut p_q = <(&Position, &Level, &StatusBundle, &PlayerTag)>::query();
    let p_info: Option<(Position, Level, StatusFlags)> = p_q
        .iter(world)
        .next()
        .map(|(p, l, s, _)| (*p, *l, s.flags()));

    let (p_pos, p_lvl, p_flags) = match p_info {
        Some(info) => info,
        None => return,
    };

    vision.update_clear_zones(grid);

    //
    if p_flags.contains(StatusFlags::BLIND) {
        vision.reset_sight();
        //
        vision.apply_vision(grid, p_pos.x as usize, p_pos.y as usize, 1);
        return;
    }

    //
    let is_dark = if let Some(tile) = grid.get_tile(p_pos.x as usize, p_pos.y as usize) {
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

    vision.recalc(grid, p_pos.x as usize, p_pos.y as usize, base_radius);

    //
    let mut f_q = <(&Position, &LightSource, &Level)>::query();
    for (pos, light, lvl) in f_q.iter(world) {
        if light.lit && lvl.0 == p_lvl.0 {
            vision.apply_vision(grid, pos.x as usize, pos.y as usize, light.range);
        }
    }

    //
    let mut c_q = <(&Position, &Inventory, &Level)>::query();
    for (pos, inv, lvl) in c_q.iter(world) {
        if lvl.0 == p_lvl.0 {
            for &item_ent in &inv.items {
                if let Ok(entry) = world.entry_ref(item_ent) {
                    if let Ok(light) = entry.get_component::<LightSource>() {
                        if light.lit {
                            vision.apply_vision(grid, pos.x as usize, pos.y as usize, light.range);
                        }
                    }
                }
            }
        }
    }

    //
    if p_flags.contains(StatusFlags::INFRAVISION) {
        let mut m_q = <(&Position, &Level, &crate::core::entity::MonsterTag)>::query();
        for (m_pos, m_lvl, _) in m_q.iter(world) {
            if m_lvl.0 == p_lvl.0 {
                let dx = (m_pos.x - p_pos.x).abs();
                let dy = (m_pos.y - p_pos.y).abs();
                if dx <= 8 && dy <= 8 {
                    //
                    if VisionSystem::has_line_of_sight(
                        grid,
                        p_pos.x as usize,
                        p_pos.y as usize,
                        m_pos.x as usize,
                        m_pos.y as usize,
                    ) {
                        vision.viz_array[m_pos.x as usize][m_pos.y as usize] |=
                            crate::core::systems::vision::IN_SIGHT;
                    }
                }
            }
        }
    }
}

///
#[legion::system]
#[read_component(crate::core::entity::MagicMapRequest)]
#[read_component(PlayerTag)]
pub fn magic_map_effect(
    world: &SubWorld,
    #[resource] vision: &mut VisionSystem,
    command_buffer: &mut legion::systems::CommandBuffer,
) {
    let mut query =
        <(Entity, &crate::core::entity::MagicMapRequest)>::query().filter(component::<PlayerTag>());
    for (ent, _) in query.iter(world) {
        vision.magic_map();
        command_buffer.remove_component::<crate::core::entity::MagicMapRequest>(*ent);
    }
}
