//!
//!
//!
//!

use eframe::egui;

///
#[derive(Debug, Clone, PartialEq)]
pub enum MouseAction {
    ///
    None,
    ///
    AdjacentMove {
        ///
        target_x: i32,
        target_y: i32,
        ///
        dir: crate::core::game_state::Direction,
    },
    ///
    Travel { target_x: i32, target_y: i32 },
    ///
    SelfClick,
    ///
    Inspect { grid_x: i32, grid_y: i32 },
}

///
///
///
pub fn screen_to_grid(
    screen_pos: egui::Pos2,
    rect: egui::Rect,
    char_width: f32,
    char_height: f32,
    x_offset: f32,
    y_offset: f32,
) -> Option<(i32, i32)> {
    let gx = ((screen_pos.x - rect.min.x - x_offset) / char_width) as i32;
    let gy = ((screen_pos.y - rect.min.y - y_offset) / char_height) as i32;

    if gx >= 0
        && gx < crate::core::dungeon::COLNO as i32
        && gy >= 0
        && gy < crate::core::dungeon::ROWNO as i32
    {
        Some((gx, gy))
    } else {
        None
    }
}

///
pub fn handle_left_click(grid_x: i32, grid_y: i32, player_x: i32, player_y: i32) -> MouseAction {
    let dx = grid_x - player_x;
    let dy = grid_y - player_y;

    //
    if dx == 0 && dy == 0 {
        return MouseAction::SelfClick;
    }

    //
    if dx.abs() <= 1 && dy.abs() <= 1 {
        //
        if let Some(dir) = delta_to_direction(dx, dy) {
            return MouseAction::AdjacentMove {
                target_x: grid_x,
                target_y: grid_y,
                dir,
            };
        }
    }

    //
    MouseAction::Travel {
        target_x: grid_x,
        target_y: grid_y,
    }
}

///
pub fn handle_right_click(grid_x: i32, grid_y: i32) -> MouseAction {
    MouseAction::Inspect { grid_x, grid_y }
}

///
fn delta_to_direction(dx: i32, dy: i32) -> Option<crate::core::game_state::Direction> {
    use crate::core::game_state::Direction;
    match (dx, dy) {
        (0, -1) => Some(Direction::North),
        (0, 1) => Some(Direction::South),
        (-1, 0) => Some(Direction::West),
        (1, 0) => Some(Direction::East),
        (-1, -1) => Some(Direction::NorthWest),
        (1, -1) => Some(Direction::NorthEast),
        (-1, 1) => Some(Direction::SouthWest),
        (1, 1) => Some(Direction::SouthEast),
        _ => None,
    }
}

///
pub fn delta_to_direction_pub(dx: i32, dy: i32) -> Option<crate::core::game_state::Direction> {
    delta_to_direction(dx, dy)
}

///
pub fn direction_to_command(dir: crate::core::game_state::Direction) -> crate::ui::input::Command {
    use crate::core::game_state::Direction;
    use crate::ui::input::Command;
    match dir {
        Direction::North => Command::MoveN,
        Direction::South => Command::MoveS,
        Direction::East => Command::MoveE,
        Direction::West => Command::MoveW,
        Direction::NorthEast => Command::MoveNE,
        Direction::NorthWest => Command::MoveNW,
        Direction::SouthEast => Command::MoveSE,
        Direction::SouthWest => Command::MoveSW,
        Direction::Here => Command::Wait,
    }
}
