use crate::core::Observation;
use ratatui::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use super::{labels::AutoLabel, viewport::Viewport, UiTheme};

/// 라벨 표시를 지원하는 맵 위젯이다.
pub struct MapWidget<'a> {
    pub observation: &'a Observation,
    pub viewport: Viewport,
    /// 맵 위에 표시할 자동 라벨 목록이다.
    pub labels: &'a [AutoLabel],
    pub theme: UiTheme,
}

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 타일 렌더링
        for tile in &self.observation.visible_tiles {
            if let Some((cx, cy)) = self.viewport.world_to_terminal(tile.pos, area) {
                let glyph = match tile.tile {
                    crate::domain::tile::TileKind::Wall => '#',
                    crate::domain::tile::TileKind::Floor => '.',
                    crate::domain::tile::TileKind::Door(_) => '+',
                    crate::domain::tile::TileKind::StairsDown => '>',
                    crate::domain::tile::TileKind::StairsUp => '<',
                    crate::domain::tile::TileKind::Trap(_) => '^',
                    crate::domain::tile::TileKind::HiddenDoor => '#',
                    crate::domain::tile::TileKind::HiddenTrap(_) => '.',
                };
                buf[(cx, cy)]
                    .set_char(glyph)
                    .set_style(Style::default().fg(self.theme.fg).bg(self.theme.bg));
            }
        }
        if let Some((px, py)) = self
            .viewport
            .world_to_terminal(self.observation.player_pos, area)
        {
            buf[(px, py)]
                .set_char('@')
                .set_style(Style::default().fg(self.theme.accent).bg(self.theme.bg));
        }

        // 라벨은 core tile을 덮지 않도록 오른쪽 한 셀에 표시한다.
        for label in self.labels {
            if let Some((cx, cy)) = self.viewport.world_to_terminal(label.pos, area) {
                // 라벨 텍스트를 셀 우측에 표시 (한 칸 오른쪽)
                let label_x = cx + 1;
                if label_x < area.x + area.width {
                    // 텍스트의 첫 글자만 해당 셀에 표시
                    if let Some(first_char) = label.text.chars().next() {
                        buf[(label_x, cy)]
                            .set_char(first_char)
                            .set_style(Style::default().fg(self.theme.danger).bg(self.theme.bg));
                    }
                }
            }
        }
    }
}
