//!
//!
//!
//!
//!

use eframe::egui;

///
pub enum GameOverAction {
    ///
    BackToTitle,
    ///
    RestartNow,
    ///
    Quit,
    ///
    None,
}

///
///
pub fn render_game_over_screen(
    ctx: &egui::Context,
    message: &str,
    score: u64,
    turns: u64,
    max_depth: i32,
    epitaph: &str,
) -> GameOverAction {
    let mut action = GameOverAction::None;

    //
    ctx.set_visuals(egui::Visuals::dark());

    //
    let dark_frame = egui::Frame::default()
        .fill(egui::Color32::from_rgb(12, 8, 10))
        .inner_margin(egui::Margin::same(20.0));

    egui::CentralPanel::default()
        .frame(dark_frame)
        .show(ctx, |ui| {
            let available = ui.available_size();

            ui.vertical_centered(|ui| {
                //
                ui.add_space(available.y * 0.06);

                //
                ui.label(
                    egui::RichText::new(TOMBSTONE_ART)
                        .monospace()
                        .color(egui::Color32::from_rgb(140, 130, 120))
                        .size(12.0),
                );

                ui.add_space(12.0);

                //
                ui.label(
                    egui::RichText::new("REST  IN  PEACE")
                        .color(egui::Color32::from_rgb(180, 50, 50))
                        .size(28.0)
                        .strong(),
                );

                ui.add_space(8.0);

                //
                if !epitaph.is_empty() {
                    ui.label(
                        egui::RichText::new(epitaph)
                            .color(egui::Color32::from_rgb(200, 180, 140))
                            .size(16.0)
                            .italics(),
                    );
                    ui.add_space(6.0);
                }

                //
                ui.label(
                    egui::RichText::new(message)
                        .color(egui::Color32::from_rgb(220, 200, 180))
                        .size(18.0),
                );

                ui.add_space(16.0);

                //
                ui.separator();
                ui.add_space(8.0);

                //
                //
                egui::Grid::new("game_over_stats")
                    .num_columns(2)
                    .spacing([40.0, 6.0])
                    .show(ui, |ui| {
                        //
                        ui.label(
                            egui::RichText::new("Final Score")
                                .color(egui::Color32::from_rgb(150, 150, 170))
                                .size(14.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("{}", score))
                                .color(egui::Color32::from_rgb(255, 220, 100))
                                .size(14.0)
                                .strong(),
                        );
                        ui.end_row();

                        // ????
                        ui.label(
                            egui::RichText::new("Turns Survived")
                                .color(egui::Color32::from_rgb(150, 150, 170))
                                .size(14.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("{}", turns))
                                .color(egui::Color32::from_rgb(200, 200, 220))
                                .size(14.0),
                        );
                        ui.end_row();

                        //
                        ui.label(
                            egui::RichText::new("Deepest Level")
                                .color(egui::Color32::from_rgb(150, 150, 170))
                                .size(14.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("Dlvl:{}", max_depth))
                                .color(egui::Color32::from_rgb(200, 200, 220))
                                .size(14.0),
                        );
                        ui.end_row();
                    });

                ui.add_space(20.0);

                //
                let btn_width = 260.0;
                let btn_height = 42.0;

                //
                let title_btn = ui.add_sized(
                    [btn_width, btn_height],
                    egui::Button::new(
                        egui::RichText::new("?? New Game")
                            .size(20.0)
                            .color(egui::Color32::from_rgb(255, 220, 100))
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(40, 35, 20))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(120, 100, 40),
                    )),
                );
                if title_btn.clicked() {
                    action = GameOverAction::BackToTitle;
                }
                //
                if title_btn.hovered() {
                    ui.painter().rect_stroke(
                        title_btn.rect.expand(2.0),
                        4.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 80)),
                    );
                }

                ui.add_space(8.0);

                //
                let quit_btn = ui.add_sized(
                    [btn_width, btn_height],
                    egui::Button::new(
                        egui::RichText::new("?? Quit to Desktop")
                            .size(18.0)
                            .color(egui::Color32::from_rgb(200, 100, 100)),
                    )
                    .fill(egui::Color32::from_rgb(35, 20, 20))
                    .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(100, 40, 40))),
                );
                if quit_btn.clicked() {
                    action = GameOverAction::Quit;
                }

                //
                ui.add_space(available.y * 0.04);
                ui.label(
                    egui::RichText::new("Press any key or click to continue...")
                        .color(egui::Color32::from_rgb(70, 70, 80))
                        .size(11.0)
                        .italics(),
                );
            });
        });

    //
    ctx.request_repaint();

    action
}

///
const TOMBSTONE_ART: &str = r#"
          ----------
         /          \
        /    REST    \
       /      IN      \
      /     PEACE      \
     /                  \
     |                  |
     |                  |
     |                  |
     |                  |
    *|     *  *  *      |*
   _|___________________|__
"#;
