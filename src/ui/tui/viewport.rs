use crate::core::Pos;
use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    pub origin: Pos,
    pub player_pos: Pos,
    pub width: u16,
    pub height: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

impl Viewport {
    pub fn from_rect(origin: Pos, player_pos: Pos, rect: Rect) -> Self {
        Self {
            origin,
            player_pos,
            width: rect.width,
            height: rect.height,
            cell_width: 1,
            cell_height: 1,
        }
    }

    pub fn terminal_to_world(self, column: u16, row: u16, rect: Rect) -> Option<Pos> {
        if column < rect.x
            || row < rect.y
            || column >= rect.x + rect.width
            || row >= rect.y + rect.height
        {
            return None;
        }
        Some(Pos {
            x: self.origin.x + (column - rect.x) as i16,
            y: self.origin.y + (row - rect.y) as i16,
        })
    }

    pub fn world_to_terminal(self, pos: Pos, rect: Rect) -> Option<(u16, u16)> {
        let dx = pos.x - self.origin.x;
        let dy = pos.y - self.origin.y;
        if dx < 0 || dy < 0 || dx >= rect.width as i16 || dy >= rect.height as i16 {
            return None;
        }
        Some((rect.x + dx as u16, rect.y + dy as u16))
    }
}
