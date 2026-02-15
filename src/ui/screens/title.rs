//!
//!
//!
//!
//!
//! - New Game / Continue / Settings / Quit 硫붾돱
//!
//!
//!
//!
//!
//!

use eframe::egui;

///
pub enum TitleAction {
    ///
    NewGame,
    ///
    Continue,
    ///
    Settings,
    ///
    Quit,
    ///
    None,
}

///
///
pub fn render_title_screen(ctx: &egui::Context) -> TitleAction {
    let mut action = TitleAction::None;

    // [v1.9.0
    ctx.set_visuals(egui::Visuals::dark());

    //
    let save_exists = std::path::Path::new("save/player.sav").exists();

    //
    let dark_frame = egui::Frame::default()
        .fill(egui::Color32::from_rgb(15, 15, 20))
        .inner_margin(egui::Margin::same(20.0));

    egui::CentralPanel::default()
        .frame(dark_frame)
        .show(ctx, |ui| {
            //
            let available = ui.available_size();

            ui.vertical_centered(|ui| {
                //
                ui.add_space(available.y * 0.10);

                //
                ui.label(
                    egui::RichText::new(TITLE_ART)
                        .monospace()
                        .color(egui::Color32::from_rgb(255, 165, 0))
                        .size(14.0),
                );

                ui.add_space(8.0);

                //
                ui.label(
                    egui::RichText::new("Powered by Rust")
                        .color(egui::Color32::from_rgb(200, 160, 100))
                        .size(16.0),
                );

                ui.add_space(4.0);

                //
                ui.label(
                    egui::RichText::new("Based on NetHack 3.6.7")
                        .color(egui::Color32::from_rgb(130, 130, 140))
                        .size(12.0),
                );

                ui.add_space(available.y * 0.08);

                //
                let button_width = 280.0;
                let button_height = 44.0;

                //
                let new_game_btn = ui.add_sized(
                    [button_width, button_height],
                    egui::Button::new(
                        egui::RichText::new("[+] New Game")
                            .size(22.0)
                            .color(egui::Color32::from_rgb(255, 220, 100))
                            .strong(),
                    )
                    .fill(egui::Color32::from_rgb(40, 35, 20))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgb(120, 100, 40),
                    )),
                );
                if new_game_btn.clicked() {
                    action = TitleAction::NewGame;
                }
                //
                if new_game_btn.hovered() {
                    ui.painter().rect_stroke(
                        new_game_btn.rect.expand(2.0),
                        4.0,
                        egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 200, 80)),
                    );
                }

                ui.add_space(10.0);

                //
                let continue_label = if save_exists {
                    "[>] Continue"
                } else {
                    "   Continue (no save)"
                };
                let continue_color = if save_exists {
                    egui::Color32::from_rgb(200, 200, 220)
                } else {
                    egui::Color32::from_rgb(70, 70, 80)
                };
                let continue_bg = if save_exists {
                    egui::Color32::from_rgb(30, 30, 40)
                } else {
                    egui::Color32::from_rgb(25, 25, 30)
                };

                let continue_btn = ui.add_sized(
                    [button_width, button_height],
                    egui::Button::new(
                        egui::RichText::new(continue_label)
                            .size(20.0)
                            .color(continue_color),
                    )
                    .fill(continue_bg)
                    .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(60, 60, 70))),
                );
                if continue_btn.clicked() && save_exists {
                    action = TitleAction::Continue;
                }

                ui.add_space(10.0);

                // ??Settings
                let settings_btn = ui.add_sized(
                    [button_width, button_height],
                    egui::Button::new(
                        egui::RichText::new("[=] Settings")
                            .size(20.0)
                            .color(egui::Color32::from_rgb(180, 180, 200)),
                    )
                    .fill(egui::Color32::from_rgb(30, 30, 40))
                    .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(60, 60, 70))),
                );
                if settings_btn.clicked() {
                    action = TitleAction::Settings;
                }

                ui.add_space(10.0);

                //
                let quit_btn = ui.add_sized(
                    [button_width, button_height],
                    egui::Button::new(
                        egui::RichText::new("[X] Quit")
                            .size(20.0)
                            .color(egui::Color32::from_rgb(200, 100, 100)),
                    )
                    .fill(egui::Color32::from_rgb(35, 20, 20))
                    .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(100, 40, 40))),
                );
                if quit_btn.clicked() {
                    action = TitleAction::Quit;
                }

                //
                ui.add_space(available.y * 0.05);
                ui.label(
                    egui::RichText::new("v2.1.0")
                        .color(egui::Color32::from_rgb(80, 80, 90))
                        .size(11.0),
                );
            });
        });

    // [v1.9.0
    ctx.request_repaint();

    action
}

///
///
const TITLE_ART: &str = r#"
     _    ___ _   _            _    
    / \  |_ _| | | | __ _  ___| | __
   / _ \  | || |_| |/ _` |/ __| |/ /
  / ___ \ | ||  _  | (_| | (__|   < 
 /_/   \_\___|_| |_|\__,_|\___|_|\_\
                                     
   A Modern Roguelike in Rust        
"#;
