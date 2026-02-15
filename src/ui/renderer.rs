use crate::assets::AssetManager;
use crate::core::dungeon::{Grid, COLNO, ROWNO};
use crate::core::entity::{Position, Renderable};
use legion::*;
use ratatui::{
    backend::TestBackend,
    style::{Color, Style},
    Terminal,
};

///
pub struct HybridRenderer {
    pub terminal: Terminal<TestBackend>,
}

use crate::core::systems::vision::VisionSystem;

impl HybridRenderer {
    pub fn new() -> Self {
        //
        let backend = TestBackend::new(COLNO as u16, ROWNO as u16);
        let terminal = Terminal::new(backend).unwrap();

        Self { terminal }
    }

    ///
    pub fn render_frame(
        &mut self,
        grid: &Grid,
        world: &World,
        assets: &AssetManager,
        vision: &VisionSystem,
    ) {
        self.terminal
            .draw(|f| {
                let buffer = f.buffer_mut();

                //
                for x in 0..COLNO {
                    for y in 0..ROWNO {
                        let flags = vision.viz_array[x][y];
                        let visible = (flags & crate::core::systems::vision::IN_SIGHT) != 0;
                        let memorized = (flags & crate::core::systems::vision::MEMORIZED) != 0;

                        if visible {
                            if let Some(tile) = grid.get_tile(x, y) {
                                let symbol = assets.symbols.get_tile_symbol(tile.typ);
                                let color = assets.symbols.get_tile_color(tile.typ);

                                buffer
                                    .get_mut(x as u16, y as u16)
                                    .set_symbol(&symbol.to_string())
                                    .set_style(Style::default().fg(u8_to_ratatui_color(color)));
                            }
                        } else if memorized {
                            //
                            if let Some(tile) = grid.get_tile(x, y) {
                                let symbol = assets.symbols.get_tile_symbol(tile.typ);
                                //
                                //
                                //
                                buffer
                                    .get_mut(x as u16, y as u16)
                                    .set_symbol(&symbol.to_string())
                                    .set_style(Style::default().fg(Color::DarkGray));
                            }
                        } else {
                            //
                            buffer
                                .get_mut(x as u16, y as u16)
                                .set_symbol(" ")
                                .set_style(Style::default());
                        }
                    }
                }

                //
                use crate::core::entity::status::{StatusBundle, StatusFlags};
                use crate::core::entity::PlayerTag;

                //
                let mut p_flags = StatusFlags::empty();
                if let Some((_, status)) = <(&PlayerTag, &StatusBundle)>::query().iter(world).next()
                {
                    p_flags = status.flags();
                }

                let mut query = <(Entity, &Position, &Renderable, Option<&PlayerTag>)>::query();
                for (_ent, pos, render, is_player) in query.iter(world) {
                    if pos.x >= 0 && pos.x < COLNO as i32 && pos.y >= 0 && pos.y < ROWNO as i32 {
                        let x = pos.x as usize;
                        let y = pos.y as usize;
                        let visible =
                            (vision.viz_array[x][y] & crate::core::systems::vision::IN_SIGHT) != 0;

                        //
                        if is_player.is_some() || visible {
                            let mut glyph = render.glyph;
                            let mut color = render.color;

                            //
                            if p_flags.contains(StatusFlags::HALLUCINATING) && is_player.is_none() {
                                //
                                //
                                let seed = (x as u64)
                                    .wrapping_mul(y as u64)
                                    .wrapping_add(vision.viz_array[x][y] as u64);
                                glyph = match seed % 10 {
                                    0 => 'g',
                                    1 => 'o',
                                    2 => 'd',
                                    3 => 'a',
                                    4 => 'x',
                                    5 => 'Y',
                                    6 => 'D',
                                    7 => 'R',
                                    8 => '?',
                                    9 => '%',
                                    _ => '?',
                                } as char;
                                color = (seed % 14 + 1) as u8;
                            }

                            buffer
                                .get_mut(pos.x as u16, pos.y as u16)
                                .set_symbol(&glyph.to_string())
                                .set_style(Style::default().fg(u8_to_ratatui_color(color)));
                        }
                    }
                }
            })
            .unwrap();
    }
}

///
pub fn u8_to_ratatui_color(color: u8) -> Color {
    match color {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White, // LightWhite
        _ => Color::White,
    }
}
