//!
//!
//!
//!

use eframe::egui;

///
pub struct MinimapData<'a> {
    ///
    pub grid: &'a crate::core::dungeon::Grid,
    ///
    pub vision: &'a crate::core::systems::vision::VisionSystem,
    ///
    pub player_x: usize,
    pub player_y: usize,
}

///
pub fn render_minimap(ui: &mut egui::Ui, data: &MinimapData) {
    ui.label(
        egui::RichText::new("Minimap")
            .color(egui::Color32::from_rgb(160, 160, 180))
            .size(12.0)
            .strong(),
    );
    ui.add_space(4.0);

    let cols = crate::core::dungeon::COLNO;
    let rows = crate::core::dungeon::ROWNO;

    //
    let available_width = ui.available_width();
    let cell_size = (available_width / cols as f32).max(1.5).min(3.0);
    let map_height = rows as f32 * cell_size;

    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(cols as f32 * cell_size, map_height),
        egui::Sense::hover(),
    );

    //
    ui.painter()
        .rect_filled(rect, 2.0, egui::Color32::from_rgb(10, 10, 15));

    for y in 0..rows {
        for x in 0..cols {
            let flags = data.vision.viz_array[x][y];
            let is_visible = (flags & crate::core::systems::vision::IN_SIGHT) != 0;
            let is_memorized = (flags & crate::core::systems::vision::MEMORIZED) != 0;

            if !is_visible && !is_memorized {
                continue;
            }

            //
            let tile = &data.grid.locations[x][y];
            let color = minimap_tile_color(tile, is_visible);

            let cell_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(x as f32 * cell_size, y as f32 * cell_size),
                egui::vec2(cell_size, cell_size),
            );

            ui.painter().rect_filled(cell_rect, 0.0, color);
        }
    }

    //
    if data.player_x < cols && data.player_y < rows {
        let player_rect = egui::Rect::from_min_size(
            rect.min
                + egui::vec2(
                    data.player_x as f32 * cell_size,
                    data.player_y as f32 * cell_size,
                ),
            egui::vec2(cell_size, cell_size),
        );
        ui.painter()
            .rect_filled(player_rect, 0.0, egui::Color32::WHITE);
    }

    //
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(0.5, egui::Color32::from_rgb(60, 60, 80)),
    );
}

///
///
fn minimap_tile_color(tile: &crate::core::dungeon::tile::Tile, is_visible: bool) -> egui::Color32 {
    use crate::core::dungeon::tile::TileType;

    let alpha = if is_visible { 255 } else { 120 };

    match tile.typ {
        //
        TileType::VWall
        | TileType::HWall
        | TileType::TlCorner
        | TileType::TrCorner
        | TileType::BlCorner
        | TileType::BrCorner
        | TileType::CrossWall
        | TileType::TuWall
        | TileType::TdWall
        | TileType::TlWall
        | TileType::TrWall
        | TileType::DbWall => egui::Color32::from_rgba_premultiplied(80, 80, 100, alpha),
        //
        TileType::Room => egui::Color32::from_rgba_premultiplied(50, 50, 60, alpha),
        TileType::Corr | TileType::SCorr => {
            egui::Color32::from_rgba_premultiplied(45, 45, 55, alpha)
        }
        //
        TileType::Door | TileType::SDoor => {
            egui::Color32::from_rgba_premultiplied(160, 100, 50, alpha)
        }
        TileType::OpenDoor => egui::Color32::from_rgba_premultiplied(120, 80, 40, alpha),
        //
        TileType::StairsDown => egui::Color32::from_rgba_premultiplied(200, 200, 255, alpha),
        TileType::StairsUp | TileType::Ladder => {
            egui::Color32::from_rgba_premultiplied(255, 255, 200, alpha)
        }
        //
        TileType::Pool | TileType::Moat | TileType::Water => {
            egui::Color32::from_rgba_premultiplied(40, 80, 180, alpha)
        }
        TileType::LavaPool => egui::Color32::from_rgba_premultiplied(220, 80, 20, alpha),
        //
        TileType::Fountain => egui::Color32::from_rgba_premultiplied(100, 150, 255, alpha),
        TileType::Altar => egui::Color32::from_rgba_premultiplied(200, 180, 255, alpha),
        TileType::Throne => egui::Color32::from_rgba_premultiplied(255, 215, 0, alpha),
        TileType::Sink | TileType::Grave => {
            egui::Color32::from_rgba_premultiplied(100, 100, 120, alpha)
        }
        TileType::Tree => egui::Color32::from_rgba_premultiplied(60, 120, 60, alpha),
        TileType::IronBars => egui::Color32::from_rgba_premultiplied(100, 100, 130, alpha),
        TileType::Ice => egui::Color32::from_rgba_premultiplied(180, 220, 255, alpha),
        TileType::DrawbridgeUp | TileType::DrawbridgeDown => {
            egui::Color32::from_rgba_premultiplied(120, 100, 80, alpha)
        }
        TileType::Air => egui::Color32::from_rgba_premultiplied(200, 200, 255, alpha),
        TileType::Cloud => egui::Color32::from_rgba_premultiplied(220, 220, 240, alpha),
        //
        _ => egui::Color32::TRANSPARENT,
    }
}
