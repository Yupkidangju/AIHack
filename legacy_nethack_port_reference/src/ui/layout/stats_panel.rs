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
#[derive(Debug, Clone, Default)]
pub struct EquipmentSummary {
    pub weapon: String,
    pub shield: String,
    pub armor: String,
    pub helmet: String,
    pub cloak: String,
    pub gloves: String,
    pub boots: String,
    pub ring_left: String,
    pub ring_right: String,
    pub amulet: String,
}

///
#[derive(Debug, Clone, Default)]
pub struct StatsPanelData {
    pub name: String,
    pub title: String, // ?? "the Stripling"
    pub hp: i32,
    pub hp_max: i32,
    pub energy: i32,
    pub energy_max: i32,
    pub str_: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
    pub ac: i32,
    pub level: i32,
    pub gold: u64,
    pub depth: String,
    pub equipment: EquipmentSummary,
}

///
pub fn render_stats_panel(ctx: &egui::Context, data: &StatsPanelData) {
    egui::SidePanel::right("stats_panel")
        .default_width(180.0)
        .min_width(150.0)
        .max_width(250.0)
        .resizable(true)
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(18, 18, 26))
                .inner_margin(egui::Margin::same(8.0))
                .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(45, 45, 60))),
        )
        .show(ctx, |ui| {
            //
            ui.label(
                egui::RichText::new(&data.name)
                    .color(egui::Color32::from_rgb(255, 220, 100))
                    .size(15.0)
                    .strong(),
            );
            ui.label(
                egui::RichText::new(&data.title)
                    .color(egui::Color32::from_rgb(160, 160, 180))
                    .size(11.0),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            //
            ui.label(
                egui::RichText::new("HP")
                    .color(egui::Color32::from_rgb(140, 140, 160))
                    .size(11.0),
            );
            render_large_bar(ui, data.hp, data.hp_max, hp_color(data.hp, data.hp_max));
            ui.label(
                egui::RichText::new(format!("{} / {}", data.hp, data.hp_max))
                    .color(hp_color(data.hp, data.hp_max))
                    .monospace()
                    .size(12.0),
            );

            ui.add_space(4.0);

            //
            ui.label(
                egui::RichText::new("Pw")
                    .color(egui::Color32::from_rgb(140, 140, 160))
                    .size(11.0),
            );
            render_large_bar(
                ui,
                data.energy,
                data.energy_max,
                egui::Color32::from_rgb(80, 140, 255),
            );
            ui.label(
                egui::RichText::new(format!("{} / {}", data.energy, data.energy_max))
                    .color(egui::Color32::from_rgb(120, 170, 255))
                    .monospace()
                    .size(12.0),
            );

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            //
            let stat_label_color = egui::Color32::from_rgb(140, 140, 160);
            let stat_val_color = egui::Color32::from_rgb(210, 210, 230);
            let stats = [
                ("Str", data.str_),
                ("Dex", data.dex),
                ("Con", data.con),
                ("Int", data.int),
                ("Wis", data.wis),
                ("Cha", data.cha),
            ];
            for (name, val) in &stats {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{}:", name))
                            .color(stat_label_color)
                            .monospace()
                            .size(12.0),
                    );
                    ui.label(
                        egui::RichText::new(format!("{:>3}", val))
                            .color(stat_val_color)
                            .monospace()
                            .size(12.0),
                    );
                });
            }

            ui.add_space(4.0);

            // AC, Level
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("AC:{}", data.ac))
                        .color(stat_val_color)
                        .monospace()
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(format!("XL:{}", data.level))
                        .color(stat_val_color)
                        .monospace()
                        .size(12.0),
                );
            });

            //
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("${}", data.gold))
                        .color(egui::Color32::GOLD)
                        .monospace()
                        .size(12.0),
                );
                ui.label(
                    egui::RichText::new(&data.depth)
                        .color(stat_label_color)
                        .monospace()
                        .size(12.0),
                );
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            //
            ui.label(
                egui::RichText::new("Equipment")
                    .color(egui::Color32::from_rgb(200, 200, 220))
                    .size(13.0)
                    .strong(),
            );
            ui.add_space(2.0);

            let eq = &data.equipment;
            let equip_items = [
                ("[W]", "Weapon", &eq.weapon),
                ("[S]", "Shield", &eq.shield),
                ("[A]", "Armor", &eq.armor),
                ("[H]", "Helmet", &eq.helmet),
                ("[C]", "Cloak", &eq.cloak),
                ("[G]", "Gloves", &eq.gloves),
                ("[B]", "Boots", &eq.boots),
                ("[R]", "Ring L", &eq.ring_left),
                ("[R]", "Ring R", &eq.ring_right),
                ("[N]", "Amulet", &eq.amulet),
            ];

            for (icon, slot_name, item_name) in &equip_items {
                let display = if item_name.is_empty() {
                    format!("{} {}: -", icon, slot_name)
                } else {
                    format!("{} {}", icon, item_name)
                };
                let color = if item_name.is_empty() {
                    egui::Color32::from_rgb(70, 70, 90)
                } else {
                    egui::Color32::from_rgb(180, 200, 180)
                };
                ui.label(egui::RichText::new(display).color(color).size(11.0));
            }
        });
}

///
fn hp_color(current: i32, max: i32) -> egui::Color32 {
    if max <= 0 {
        return egui::Color32::RED;
    }
    let ratio = current as f32 / max as f32;
    if ratio <= 0.25 {
        egui::Color32::RED
    } else if ratio <= 0.50 {
        egui::Color32::from_rgb(255, 165, 0)
    } else if ratio <= 0.75 {
        egui::Color32::YELLOW
    } else {
        egui::Color32::from_rgb(80, 220, 80)
    }
}

///
fn render_large_bar(ui: &mut egui::Ui, current: i32, max: i32, fill_color: egui::Color32) {
    let width = ui.available_width();
    let height = 12.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    //
    ui.painter()
        .rect_filled(rect, 3.0, egui::Color32::from_rgb(35, 35, 50));

    //
    if max > 0 {
        let ratio = (current as f32 / max as f32).clamp(0.0, 1.0);
        let fill_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(rect.width() * ratio, height));
        ui.painter().rect_filled(fill_rect, 3.0, fill_color);
    }

    //
    ui.painter().rect_stroke(
        rect,
        3.0,
        egui::Stroke::new(0.5, egui::Color32::from_rgb(70, 70, 90)),
    );
}
