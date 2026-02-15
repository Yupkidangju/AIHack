//!
//!
//!
//!
//!

use eframe::egui;

///
#[derive(Debug, Clone, PartialEq)]
pub enum CommandBarAction {
    ///
    None,
    //
    ///
    Pickup,
    ///
    Inventory,
    ///
    Eat,
    ///
    Quaff,
    ///
    Read,
    ///
    Apply,
    /// 吏쒓린 (z)
    Zap,
    ///
    Cast,
    ///
    Pray,
    ///
    Search,
    ///
    Help,
    //
    ///
    Throw,
    ///
    Kick,
    ///
    Open,
    ///
    Close,
    ///
    Wear,
    ///
    TakeOff,
    ///
    Wield,
    ///
    Engrave,
    ///
    Name,
    ///
    Save,
}

///
///
pub fn render_command_bar(ctx: &egui::Context, advanced_mode: bool) -> CommandBarAction {
    let mut action = CommandBarAction::None;

    egui::TopBottomPanel::bottom("command_bar")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(22, 22, 30))
                .inner_margin(egui::Margin::symmetric(6.0, 4.0))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 65))),
        )
        .show(ctx, |ui| {
            //
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;

                //
                let simple_buttons = [
                    ("[P]", "Pick", CommandBarAction::Pickup),
                    ("[I]", "Inv", CommandBarAction::Inventory),
                    ("[E]", "Eat", CommandBarAction::Eat),
                    ("[Q]", "Quaff", CommandBarAction::Quaff),
                    ("[R]", "Read", CommandBarAction::Read),
                    ("[A]", "Apply", CommandBarAction::Apply),
                    ("[Z]", "Zap", CommandBarAction::Zap),
                    ("[C]", "Cast", CommandBarAction::Cast),
                    ("[P]", "Pray", CommandBarAction::Pray),
                    ("[S]", "Search", CommandBarAction::Search),
                    ("[?]", "Help", CommandBarAction::Help),
                ];

                for (icon, label, btn_action) in &simple_buttons {
                    let btn_text = format!("{} {}", icon, label);
                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(btn_text)
                                .size(12.0)
                                .color(egui::Color32::from_rgb(190, 190, 210)),
                        )
                        .fill(egui::Color32::from_rgb(35, 35, 48))
                        .min_size(egui::vec2(52.0, 24.0)),
                    );
                    if btn.clicked() {
                        action = btn_action.clone();
                    }
                }
            });

            //
            if advanced_mode {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 3.0;

                    let advanced_buttons = [
                        ("[T]", "Throw", CommandBarAction::Throw),
                        ("[K]", "Kick", CommandBarAction::Kick),
                        ("[O]", "Open", CommandBarAction::Open),
                        ("[C]", "Close", CommandBarAction::Close),
                        ("[W]", "Wear", CommandBarAction::Wear),
                        ("[T]", "TakeOff", CommandBarAction::TakeOff),
                        ("[W]", "Wield", CommandBarAction::Wield),
                        ("[E]", "Engrave", CommandBarAction::Engrave),
                        ("[N]", "Name", CommandBarAction::Name),
                        ("[S]", "Save", CommandBarAction::Save),
                    ];

                    for (icon, label, btn_action) in &advanced_buttons {
                        let btn_text = format!("{} {}", icon, label);
                        let btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new(btn_text)
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(170, 170, 190)),
                            )
                            .fill(egui::Color32::from_rgb(30, 30, 42))
                            .min_size(egui::vec2(52.0, 22.0)),
                        );
                        if btn.clicked() {
                            action = btn_action.clone();
                        }
                    }
                });
            }
        });

    action
}
