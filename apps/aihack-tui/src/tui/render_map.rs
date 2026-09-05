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
        // 바닥 물건 위에 actor를 그리고 마지막에 player를 그려 겹친 셀의 우선순위를 보장한다.
        for actors in [false, true] {
            for entity in &self.observation.visible_entities {
                use aihack_ai_contract::{EntityKind, ItemKind, MonsterKind};
                if !entity.alive || matches!(entity.kind, EntityKind::Item(_)) == actors {
                    continue;
                }
                let glyph = match entity.kind {
                    EntityKind::Player => '@',
                    EntityKind::Monster(MonsterKind::Jackal) => 'd',
                    EntityKind::Monster(MonsterKind::Goblin) => 'g',
                    EntityKind::Monster(MonsterKind::FloatingEye) => 'e',
                    EntityKind::Item(kind) => match kind {
                        ItemKind::Dagger => ')',
                        ItemKind::ArmorLeather => '[',
                        ItemKind::PotionHealing => '!',
                        ItemKind::WandMagicMissile => '/',
                        ItemKind::Rock => '*',
                        ItemKind::FoodRation | ItemKind::CorpseJackal => '%',
                        ItemKind::AmuletAscension => '"',
                        _ => '?',
                    },
                };
                if let Some((x, y)) = self.viewport.world_to_terminal(entity.pos, area) {
                    buf[(x, y)].set_char(glyph).set_style(
                        Style::default()
                            .fg(if actors {
                                self.theme.danger
                            } else {
                                self.theme.accent
                            })
                            .bg(self.theme.bg),
                    );
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
