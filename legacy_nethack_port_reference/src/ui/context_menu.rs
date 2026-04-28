//!
//!
//!
//!

use eframe::egui;
use legion::*;

///
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    /// 메뉴 표시 여부
    pub visible: bool,
    /// 검사 대상 그리드 좌표
    pub grid_x: i32,
    pub grid_y: i32,
}

impl Default for ContextMenuState {
    fn default() -> Self {
        Self {
            visible: false,
            grid_x: 0,
            grid_y: 0,
        }
    }
}

///
///
pub fn render_context_menu(
    ctx: &egui::Context,
    state: &mut ContextMenuState,
    world: &World,
    grid: &crate::core::dungeon::Grid,
    resources: &Resources,
) {
    if !state.visible {
        return;
    }

    let gx = state.grid_x;
    let gy = state.grid_y;

    // 시야 확인
    let mut is_visible = false;
    let mut is_memorized = false;
    if let Some(vision) = resources.get::<crate::core::systems::vision::VisionSystem>() {
        if gx >= 0
            && gx < crate::core::dungeon::COLNO as i32
            && gy >= 0
            && gy < crate::core::dungeon::ROWNO as i32
        {
            let flags = vision.viz_array[gx as usize][gy as usize];
            is_visible = (flags & crate::core::systems::vision::IN_SIGHT) != 0;
            is_memorized = (flags & crate::core::systems::vision::MEMORIZED) != 0;
        }
    }

    if !is_visible && !is_memorized {
        state.visible = false;
        return;
    }

    let window_title = format!("Inspect ({}, {})", gx, gy);
    let mut still_open = true;

    egui::Window::new(window_title)
        .open(&mut still_open)
        .collapsible(false)
        .resizable(false)
        .default_width(220.0)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(20, 20, 28))
                .inner_margin(egui::Margin::same(10.0))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 80))),
        )
        .show(ctx, |ui| {
            // 타일 정보
            if let Some(tile) = grid.get_tile(gx as usize, gy as usize) {
                ui.label(
                    egui::RichText::new(format!("Terrain: {:?}", tile.typ))
                        .color(egui::Color32::from_rgb(180, 180, 200))
                        .size(13.0),
                );

                // 새기기 정보
                if let Some(eng) = &tile.engraving {
                    ui.label(
                        egui::RichText::new(format!("Engraving: \"{}\"", eng.text))
                            .color(egui::Color32::from_rgb(200, 200, 160))
                            .size(11.0),
                    );
                }
            }

            ui.separator();

            //
            if is_visible {
                let mut found_monster = false;
                let mut m_query = <(
                    &crate::core::entity::Position,
                    &crate::core::entity::Health,
                    &crate::core::entity::CombatStats,
                )>::query()
                .filter(!component::<crate::core::entity::PlayerTag>());

                for (pos, health, stats) in m_query.iter(world) {
                    if pos.x == gx && pos.y == gy {
                        found_monster = true;
                        ui.label(
                            egui::RichText::new("Monster")
                                .color(egui::Color32::from_rgb(255, 120, 120))
                                .size(13.0)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "HP: {}/{}, AC: {}, Level: {}",
                                health.current, health.max, stats.ac, stats.level
                            ))
                            .color(egui::Color32::from_rgb(200, 180, 180))
                            .size(12.0),
                        );
                    }
                }

                // 아이템 정보
                let mut found_item = false;
                let mut i_query =
                    <(&crate::core::entity::Position, &crate::core::entity::Item)>::query();
                for (pos, item) in i_query.iter(world) {
                    if pos.x == gx && pos.y == gy {
                        if !found_item {
                            if found_monster {
                                ui.separator();
                            }
                            ui.label(
                                egui::RichText::new("Items")
                                    .color(egui::Color32::YELLOW)
                                    .size(13.0)
                                    .strong(),
                            );
                            found_item = true;
                        }
                        ui.label(
                            egui::RichText::new(format!("  {}", item.kind))
                                .color(egui::Color32::from_rgb(220, 220, 160))
                                .size(12.0),
                        );
                    }
                }

                if !found_monster && !found_item {
                    ui.label(
                        egui::RichText::new("(nothing here)")
                            .color(egui::Color32::from_rgb(100, 100, 120))
                            .size(11.0),
                    );
                }
            } else {
                ui.label(
                    egui::RichText::new("(out of sight — memory only)")
                        .color(egui::Color32::from_rgb(100, 100, 120))
                        .size(11.0),
                );
            }
        });

    if !still_open {
        state.visible = false;
    }
}
