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
                use ratatui::style::Color;
                let glyph = match tile.tile {
                    crate::domain::tile::TileKind::Wall => '#',
                    crate::domain::tile::TileKind::Floor => '.',
                    crate::domain::tile::TileKind::Door(
                        aihack_ai_contract::DoorState::Closed
                        | aihack_ai_contract::DoorState::Locked,
                    ) => '+',
                    crate::domain::tile::TileKind::Door(aihack_ai_contract::DoorState::Open) => '-',
                    crate::domain::tile::TileKind::StairsDown => '>',
                    crate::domain::tile::TileKind::StairsUp => '<',
                    crate::domain::tile::TileKind::Trap(_) => '^',
                    crate::domain::tile::TileKind::HiddenDoor => '#',
                    crate::domain::tile::TileKind::HiddenTrap(_) => '.',
                    crate::domain::tile::TileKind::Shop => '$',
                    crate::domain::tile::TileKind::Altar => '_',
                    crate::domain::tile::TileKind::Fountain
                    | crate::domain::tile::TileKind::DryFountain => '{',
                    crate::domain::tile::TileKind::Sink => '}',
                    crate::domain::tile::TileKind::Water | crate::domain::tile::TileKind::Lava => {
                        '~'
                    }
                };
                let style = if tile.visible {
                    let fg = match tile.tile {
                        crate::domain::tile::TileKind::Altar => match tile.altar_alignment {
                            Some(aihack_ai_contract::Alignment::Lawful) => Color::White,
                            Some(aihack_ai_contract::Alignment::Neutral) => Color::Yellow,
                            Some(aihack_ai_contract::Alignment::Chaotic) => Color::LightRed,
                            None => self.theme.fg,
                        },
                        crate::domain::tile::TileKind::Fountain => Color::Cyan,
                        crate::domain::tile::TileKind::DryFountain => self.theme.muted,
                        crate::domain::tile::TileKind::Sink => Color::Gray,
                        crate::domain::tile::TileKind::Water => Color::Blue,
                        crate::domain::tile::TileKind::Lava => Color::LightRed,
                        crate::domain::tile::TileKind::Floor if tile.engraving.is_some() => {
                            self.theme.accent
                        }
                        _ => self.theme.fg,
                    };
                    Style::default().fg(fg).bg(self.theme.bg)
                } else {
                    Style::default().fg(self.theme.muted).bg(self.theme.bg)
                };
                buf[(cx, cy)].set_char(glyph).set_style(style);
            }
        }
        // 바닥 물건 위에 actor를 그리고 마지막에 player를 그려 겹친 셀의 우선순위를 보장한다.
        for actors in [false, true] {
            for entity in &self.observation.visible_entities {
                use aihack_ai_contract::EntityKind;
                if !entity.alive || matches!(entity.kind, EntityKind::Item(_)) == actors {
                    continue;
                }
                let glyph = entity.glyph.unwrap_or_else(|| match entity.kind {
                    EntityKind::Player => '@',
                    EntityKind::Monster(kind) => aihack_runtime::monster_glyph(kind),
                    EntityKind::Item(kind) => aihack_runtime::item_glyph(kind),
                });
                let fg = match entity.faction {
                    Some(aihack_ai_contract::Faction::Pet) => ratatui::style::Color::LightGreen,
                    Some(aihack_ai_contract::Faction::Shopkeeper) => ratatui::style::Color::White,
                    Some(aihack_ai_contract::Faction::Peaceful) => {
                        entity_color(entity.color.as_deref()).unwrap_or(ratatui::style::Color::Cyan)
                    }
                    _ => entity_color(entity.color.as_deref()).unwrap_or(if actors {
                        self.theme.danger
                    } else {
                        self.theme.accent
                    }),
                };
                if let Some((x, y)) = self.viewport.world_to_terminal(entity.pos, area) {
                    buf[(x, y)]
                        .set_char(glyph)
                        .set_style(Style::default().fg(fg).bg(self.theme.bg));
                }
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
                let occupied = self.observation.visible_entities.iter().any(|entity| {
                    self.viewport.world_to_terminal(entity.pos, area) == Some((label_x, cy))
                }) || self
                    .viewport
                    .world_to_terminal(self.observation.player_pos, area)
                    == Some((label_x, cy));
                if label_x < area.x + area.width && !occupied {
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

fn entity_color(color: Option<&str>) -> Option<ratatui::style::Color> {
    use ratatui::style::Color;
    match color? {
        "brown" | "yellow" | "orange" => Some(Color::Yellow),
        "bright_yellow" => Some(Color::LightYellow),
        "gray" | "dark_gray" => Some(Color::Gray),
        "black" => Some(Color::DarkGray),
        "bright_blue" => Some(Color::LightBlue),
        "blue" => Some(Color::Blue),
        "green" => Some(Color::Green),
        "bright_green" => Some(Color::LightGreen),
        "magenta" => Some(Color::Magenta),
        "bright_magenta" => Some(Color::LightMagenta),
        "red" => Some(Color::Red),
        "bright_red" => Some(Color::LightRed),
        "cyan" => Some(Color::Cyan),
        "bright_cyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => None,
    }
}
