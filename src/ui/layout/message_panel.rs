//!
//!
//!
//!
//!
//!
//!
//!
//!
//!

use eframe::egui;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageCategory {
    ///
    Normal,
    ///
    Danger,
    ///
    Loot,
    ///
    System,
    ///
    Achievement,
    ///
    Movement,
}

///
///
fn classify_message(text: &str) -> MessageCategory {
    let lower = text.to_lowercase();

    //
    if lower.contains("hit")
        || lower.contains("attack")
        || lower.contains("bite")
        || lower.contains("hurt")
        || lower.contains("dies")
        || lower.contains("killed")
        || lower.contains("destroy")
        || lower.contains("miss")
        || lower.contains("is dead")
        || lower.contains("poison")
        || lower.contains("sick")
        || lower.contains("choking")
        || lower.contains("paralyze")
        || lower.contains("blind")
        || lower.contains("confuse")
        || lower.contains("stun")
        || lower.contains("can't see")
        || lower.contains("rotten")
        || lower.contains("explod")
    {
        return MessageCategory::Danger;
    }

    //
    if lower.contains("pick up")
        || lower.contains("gold piece")
        || lower.contains("you see")
        || lower.contains("you find")
        || lower.contains("you feel")
        || lower.contains("discover")
        || lower.contains("identify")
        || lower.contains("learn the spell")
    {
        return MessageCategory::Loot;
    }

    //
    if lower.contains("experience level")
        || lower.contains("welcome to")
        || lower.contains("well done")
        || lower.contains("protection")
        || lower.contains("speed up")
    {
        return MessageCategory::Achievement;
    }

    //
    if lower.contains("save")
        || lower.contains("restore")
        || lower.contains("--more--")
        || lower.contains("nothing happen")
        || lower.contains("you escape")
        || lower.contains("wait")
        || lower.contains("never mind")
        || lower.contains("not wearing")
        || lower.contains("nothing to")
        || lower.contains("have no")
    {
        return MessageCategory::System;
    }

    //
    if lower.contains("door")
        || lower.contains("staircase")
        || lower.contains("corridor")
        || lower.contains("fountain")
        || lower.contains("sink")
        || lower.contains("altar")
        || lower.contains("trap")
        || lower.contains("you are on")
    {
        return MessageCategory::Movement;
    }

    MessageCategory::Normal
}

///
fn category_color(cat: MessageCategory) -> egui::Color32 {
    match cat {
        MessageCategory::Normal => egui::Color32::from_rgb(220, 220, 230),
        MessageCategory::Danger => egui::Color32::from_rgb(255, 90, 90),
        MessageCategory::Loot => egui::Color32::from_rgb(80, 220, 80),
        MessageCategory::System => egui::Color32::from_rgb(220, 220, 100),
        MessageCategory::Achievement => egui::Color32::from_rgb(100, 180, 255),
        MessageCategory::Movement => egui::Color32::from_rgb(160, 160, 180),
    }
}

///
fn category_icon(cat: MessageCategory) -> &'static str {
    match cat {
        MessageCategory::Normal => "",
        MessageCategory::Danger => "??",
        MessageCategory::Loot => "??",
        MessageCategory::System => "??",
        MessageCategory::Achievement => "??",
        MessageCategory::Movement => "",
    }
}

///
///
pub fn render_message_panel(ui: &mut egui::Ui, log: &crate::ui::log::GameLog) {
    ui.add_space(4.0);

    //
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(" Messages")
                .color(egui::Color32::from_rgb(180, 180, 210))
                .size(12.0)
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{} msgs", log.messages.len()))
                    .color(egui::Color32::from_rgb(100, 100, 120))
                    .monospace()
                    .size(9.0),
            );
        });
    });
    ui.add_space(2.0);
    ui.separator();

    //
    if log.needs_more {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("--More--")
                    .color(egui::Color32::YELLOW)
                    .size(13.0)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(" (press Space)")
                    .color(egui::Color32::from_rgb(160, 160, 100))
                    .italics()
                    .size(10.0),
            );
        });
        ui.separator();
    }

    //
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .max_height(ui.available_height())
        .show(ui, |ui| {
            //
            let start = if log.messages.len() > 80 {
                log.messages.len() - 80
            } else {
                0
            };

            let mut prev_turn: u64 = 0;

            for msg in &log.messages[start..] {
                //
                if prev_turn != 0 && msg.turn != prev_turn {
                    ui.add_space(2.0);
                    ui.separator();
                }
                prev_turn = msg.turn;

                ui.horizontal(|ui| {
                    //
                    ui.label(
                        egui::RichText::new(format!("{:>4}", msg.turn))
                            .color(egui::Color32::from_rgb(65, 65, 85))
                            .monospace()
                            .size(9.0),
                    );

                    //
                    let display_text = if msg.count > 1 {
                        format!("{} (x{})", msg.text, msg.count)
                    } else {
                        msg.text.clone()
                    };

                    //
                    let is_custom_color = msg.color != [255, 255, 255] && msg.color != [0, 0, 0];

                    let (icon, text_color) = if is_custom_color {
                        //
                        let cat = classify_message(&msg.text);
                        (
                            category_icon(cat),
                            egui::Color32::from_rgb(msg.color[0], msg.color[1], msg.color[2]),
                        )
                    } else {
                        //
                        let cat = classify_message(&msg.text);
                        (category_icon(cat), category_color(cat))
                    };

                    ui.label(
                        egui::RichText::new(format!("{}{}", icon, display_text))
                            .color(text_color)
                            .size(11.0),
                    );
                });
            }
        });
}
