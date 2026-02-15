//!
//!
//!
//!

use eframe::egui;

///
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    ///
    None,
    // === File ===
    ///
    NewGame,
    ///
    Save,
    ///
    Quit,
    // === View ===
    ///
    ToggleMinimap,
    ///
    ToggleStatsPanel,
    ///
    ToggleMessagePanel,
    ///
    ToggleCommandMode,
    // === Commands ===
    ///
    Inventory,
    ///
    Help,
    ///
    CharacterInfo,
    ///
    Discoveries,
    ///
    MessageHistory,
}

///
///
#[derive(Debug, Clone)]
pub struct LayoutSettings {
    ///
    pub show_minimap: bool,
    ///
    pub show_stats_panel: bool,
    ///
    pub show_message_panel: bool,
    ///
    pub command_advanced_mode: bool,
    ///
    pub show_settings: bool,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            show_minimap: true,
            show_stats_panel: true,
            show_message_panel: true,
            command_advanced_mode: false,
            show_settings: false,
        }
    }
}

///
///
pub fn render_menu_bar(
    ctx: &egui::Context,
    turn: u64,
    settings: &mut LayoutSettings,
) -> MenuAction {
    let mut action = MenuAction::None;

    egui::TopBottomPanel::top("menu_bar")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(25, 25, 35))
                .inner_margin(egui::Margin::symmetric(8.0, 2.0)),
        )
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // === File 硫붾돱 ===
                ui.menu_button(
                    egui::RichText::new("File").color(egui::Color32::from_rgb(200, 200, 220)),
                    |ui| {
                        if ui.button("New Game").clicked() {
                            action = MenuAction::NewGame;
                            ui.close_menu();
                        }
                        if ui.button("Save (S)").clicked() {
                            action = MenuAction::Save;
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            action = MenuAction::Quit;
                            ui.close_menu();
                        }
                    },
                );

                // === View 硫붾돱 ===
                ui.menu_button(
                    egui::RichText::new("View").color(egui::Color32::from_rgb(200, 200, 220)),
                    |ui| {
                        if ui.checkbox(&mut settings.show_minimap, "Minimap").changed() {
                            //
                        }
                        if ui
                            .checkbox(&mut settings.show_stats_panel, "Stats Panel")
                            .changed()
                        {}
                        if ui
                            .checkbox(&mut settings.show_message_panel, "Message Log")
                            .changed()
                        {}
                        ui.separator();
                        if ui
                            .checkbox(&mut settings.command_advanced_mode, "Advanced Commands")
                            .changed()
                        {}
                        ui.separator();
                        if ui
                            .checkbox(&mut settings.show_settings, "??Settings")
                            .changed()
                        {}
                    },
                );

                // === Commands 硫붾돱 ===
                ui.menu_button(
                    egui::RichText::new("Commands").color(egui::Color32::from_rgb(200, 200, 220)),
                    |ui| {
                        if ui.button("Inventory (i)").clicked() {
                            action = MenuAction::Inventory;
                            ui.close_menu();
                        }
                        if ui.button("Character Info (Ctrl+X)").clicked() {
                            action = MenuAction::CharacterInfo;
                            ui.close_menu();
                        }
                        if ui.button("Discoveries (\\)").clicked() {
                            action = MenuAction::Discoveries;
                            ui.close_menu();
                        }
                        if ui.button("Message History (Ctrl+P)").clicked() {
                            action = MenuAction::MessageHistory;
                            ui.close_menu();
                        }
                    },
                );

                // === Help 硫붾돱 ===
                ui.menu_button(
                    egui::RichText::new("Help").color(egui::Color32::from_rgb(200, 200, 220)),
                    |ui| {
                        if ui.button("Keybindings (?)").clicked() {
                            action = MenuAction::Help;
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.label(
                            egui::RichText::new("AIHack v2.1.0")
                                .color(egui::Color32::from_rgb(120, 120, 140))
                                .size(11.0),
                        );
                        ui.label(
                            egui::RichText::new("Based on NetHack 3.6.7")
                                .color(egui::Color32::from_rgb(100, 100, 120))
                                .size(10.0),
                        );
                    },
                );

                //
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("Turn: {}", turn))
                            .color(egui::Color32::from_rgb(160, 160, 180))
                            .monospace()
                            .size(13.0),
                    );
                });
            });
        });

    action
}
