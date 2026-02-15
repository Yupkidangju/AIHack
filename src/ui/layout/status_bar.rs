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
pub enum StatusBarMode {
    ///
    Classic,
    ///
    Graphical,
}

impl Default for StatusBarMode {
    fn default() -> Self {
        StatusBarMode::Graphical
    }
}

///
///
#[derive(Debug, Clone, Default)]
pub struct StatusInfo {
    //
    pub str_: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
    //
    pub hp: i32,
    pub hp_max: i32,
    pub energy: i32,
    pub energy_max: i32,
    pub ac: i32,
    pub level: i32,
    pub experience: u64,
    pub gold: u64,
    //
    pub xp_for_next_level: u64,
    //
    pub depth_str: String,
    pub turn: u64,
    //
    pub alignment: String,
    //
    pub player_name: String,
    pub player_title: String,
    //
    pub status_effects: Vec<StatusEffect>,
}

///
#[derive(Debug, Clone)]
pub struct StatusEffect {
    pub name: &'static str,
    pub icon: &'static str,
    pub color: egui::Color32,
}

///
///
pub fn xp_for_level(level: i32) -> u64 {
    match level {
        1 => 0,
        2 => 20,
        3 => 40,
        4 => 80,
        5 => 160,
        6 => 320,
        7 => 640,
        8 => 1280,
        9 => 2560,
        10 => 5120,
        11 => 10000,
        12 => 20000,
        13 => 40000,
        14 => 80000,
        15 => 160000,
        16 => 320000,
        17 => 640000,
        18 => 1280000,
        19 => 2560000,
        20 => 5120000,
        _ => {
            //
            if level > 20 {
                5120000 * (1u64 << (level as u64 - 20).min(40))
            } else {
                0
            }
        }
    }
}

///
pub fn render_status_bar(ctx: &egui::Context, info: &StatusInfo, mode: StatusBarMode) {
    match mode {
        StatusBarMode::Classic => render_classic_status_bar(ctx, info),
        StatusBarMode::Graphical => render_graphical_status_bar(ctx, info),
    }
}

// ========================================================================
//
// ========================================================================
fn render_classic_status_bar(ctx: &egui::Context, info: &StatusInfo) {
    egui::TopBottomPanel::bottom("status_bar")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(12, 12, 18))
                .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(40, 40, 55))),
        )
        .show(ctx, |ui| {
            let mono = egui::Color32::from_rgb(180, 180, 200);

            //
            ui.horizontal(|ui| {
                //
                if !info.player_name.is_empty() {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} the {}",
                            info.player_name, info.player_title
                        ))
                        .color(egui::Color32::from_rgb(220, 220, 255))
                        .monospace()
                        .size(12.0),
                    );
                    ui.add_space(8.0);
                }

                //
                for (label, val) in [
                    ("St", info.str_),
                    ("Dx", info.dex),
                    ("Co", info.con),
                    ("In", info.int),
                    ("Wi", info.wis),
                    ("Ch", info.cha),
                ] {
                    ui.label(
                        egui::RichText::new(format!("{}:{}", label, val))
                            .color(if val >= 18 {
                                egui::Color32::from_rgb(100, 255, 100)
                            } else if val <= 6 {
                                egui::Color32::from_rgb(255, 100, 100)
                            } else {
                                mono
                            })
                            .monospace()
                            .size(12.0),
                    );
                }

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(&info.alignment)
                        .color(egui::Color32::LIGHT_BLUE)
                        .size(12.0),
                );
            });

            //
            ui.horizontal(|ui| {
                //
                ui.label(
                    egui::RichText::new(&info.depth_str)
                        .color(mono)
                        .monospace()
                        .size(12.0),
                );
                //
                ui.label(
                    egui::RichText::new(format!("$:{}", info.gold))
                        .color(egui::Color32::GOLD)
                        .monospace()
                        .size(12.0),
                );
                // HP
                ui.label(
                    egui::RichText::new(format!("HP:{}({})", info.hp, info.hp_max))
                        .color(hp_color(info.hp, info.hp_max))
                        .monospace()
                        .size(12.0),
                );
                // MP
                ui.label(
                    egui::RichText::new(format!("Pw:{}({})", info.energy, info.energy_max))
                        .color(egui::Color32::from_rgb(120, 170, 255))
                        .monospace()
                        .size(12.0),
                );
                //
                ui.label(
                    egui::RichText::new(format!("AC:{}", info.ac))
                        .color(ac_color(info.ac))
                        .monospace()
                        .size(12.0),
                );
                // XL
                ui.label(
                    egui::RichText::new(format!("XL:{}", info.level))
                        .color(mono)
                        .monospace()
                        .size(12.0),
                );
                // ??
                ui.label(
                    egui::RichText::new(format!("T:{}", info.turn))
                        .color(egui::Color32::from_rgb(140, 140, 160))
                        .monospace()
                        .size(12.0),
                );

                //
                for effect in &info.status_effects {
                    ui.label(
                        egui::RichText::new(format!("{}{}", effect.icon, effect.name))
                            .color(effect.color)
                            .size(11.0),
                    );
                }
            });
        });
}

// ========================================================================
//
// ========================================================================
fn render_graphical_status_bar(ctx: &egui::Context, info: &StatusInfo) {
    egui::TopBottomPanel::bottom("status_bar")
        .frame(
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(18, 18, 24))
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .stroke(egui::Stroke::new(0.5, egui::Color32::from_rgb(45, 45, 60))),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                //
                //
                //
                ui.vertical(|ui| {
                    let mono = egui::Color32::from_rgb(180, 180, 200);
                    let highlight = egui::Color32::from_rgb(220, 220, 240);

                    //
                    ui.label(
                        egui::RichText::new("@")
                            .color(egui::Color32::from_rgb(255, 255, 255))
                            .monospace()
                            .size(20.0),
                    );

                    for (label, val) in [
                        ("St", info.str_),
                        ("Dx", info.dex),
                        ("Co", info.con),
                        ("In", info.int),
                        ("Wi", info.wis),
                        ("Ch", info.cha),
                    ] {
                        ui.label(
                            egui::RichText::new(format!("{}:{:>2}", label, val))
                                .color(if val >= 18 {
                                    egui::Color32::from_rgb(100, 255, 100)
                                } else if val <= 6 {
                                    egui::Color32::from_rgb(255, 100, 100)
                                } else if val >= 14 {
                                    highlight
                                } else {
                                    mono
                                })
                                .monospace()
                                .size(10.0),
                        );
                    }
                });

                ui.separator();

                //
                //
                //
                ui.vertical(|ui| {
                    //
                    ui.horizontal(|ui| {
                        if !info.player_name.is_empty() {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} the {}",
                                    info.player_name, info.player_title
                                ))
                                .color(egui::Color32::from_rgb(220, 220, 255))
                                .size(12.0)
                                .strong(),
                            );
                            ui.add_space(15.0);
                        }
                        ui.label(
                            egui::RichText::new(&info.depth_str)
                                .color(egui::Color32::from_rgb(180, 180, 200))
                                .monospace()
                                .size(11.0),
                        );
                        ui.add_space(5.0);
                        ui.label(
                            egui::RichText::new(format!("T:{}", info.turn))
                                .color(egui::Color32::from_rgb(140, 140, 160))
                                .monospace()
                                .size(11.0),
                        );
                    });

                    //
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("HP:")
                                .color(egui::Color32::from_rgb(180, 180, 200))
                                .monospace()
                                .size(11.0),
                        );
                        render_stat_bar(
                            ui,
                            info.hp,
                            info.hp_max,
                            hp_color(info.hp, info.hp_max),
                            100.0,
                        );
                        ui.label(
                            egui::RichText::new(format!("{}/{}", info.hp, info.hp_max))
                                .color(hp_color(info.hp, info.hp_max))
                                .monospace()
                                .size(11.0),
                        );
                    });

                    //
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("MP:")
                                .color(egui::Color32::from_rgb(180, 180, 200))
                                .monospace()
                                .size(11.0),
                        );
                        render_stat_bar(
                            ui,
                            info.energy,
                            info.energy_max,
                            egui::Color32::from_rgb(80, 140, 255),
                            100.0,
                        );
                        ui.label(
                            egui::RichText::new(format!("{}/{}", info.energy, info.energy_max))
                                .color(egui::Color32::from_rgb(120, 170, 255))
                                .monospace()
                                .size(11.0),
                        );
                    });

                    //
                    let xp_current = info.experience;
                    let xp_next = info.xp_for_next_level;
                    let xp_prev = xp_for_level(info.level);
                    let xp_range = if xp_next > xp_prev {
                        xp_next - xp_prev
                    } else {
                        1
                    };
                    let xp_progress = if xp_current >= xp_prev {
                        xp_current - xp_prev
                    } else {
                        0
                    };

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("XP:")
                                .color(egui::Color32::from_rgb(180, 180, 200))
                                .monospace()
                                .size(11.0),
                        );
                        render_stat_bar(
                            ui,
                            xp_progress as i32,
                            xp_range as i32,
                            egui::Color32::from_rgb(200, 180, 80),
                            100.0,
                        );
                        ui.label(
                            egui::RichText::new(format!("{}/{}", xp_current, xp_next))
                                .color(egui::Color32::from_rgb(200, 200, 120))
                                .monospace()
                                .size(11.0),
                        );
                    });

                    //
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("AC:{}", info.ac))
                                .color(ac_color(info.ac))
                                .monospace()
                                .size(11.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("XL:{}", info.level))
                                .color(egui::Color32::from_rgb(180, 180, 200))
                                .monospace()
                                .size(11.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("Gold:{}", info.gold))
                                .color(egui::Color32::GOLD)
                                .monospace()
                                .size(11.0),
                        );

                        ui.add_space(8.0);

                        //
                        for effect in &info.status_effects {
                            ui.label(
                                egui::RichText::new(format!("{}{}", effect.icon, effect.name))
                                    .color(effect.color)
                                    .size(10.0),
                            );
                        }
                    });
                });
            });
        });
}

// ========================================================================
//
// ========================================================================

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
fn ac_color(ac: i32) -> egui::Color32 {
    if ac <= 0 {
        //
        egui::Color32::from_rgb(80, 255, 80)
    } else if ac <= 3 {
        //
        egui::Color32::from_rgb(120, 200, 120)
    } else if ac <= 6 {
        //
        egui::Color32::from_rgb(220, 220, 100)
    } else if ac <= 9 {
        //
        egui::Color32::from_rgb(255, 165, 0)
    } else {
        //
        egui::Color32::from_rgb(255, 80, 80)
    }
}

///
///
fn render_stat_bar(
    ui: &mut egui::Ui,
    current: i32,
    max: i32,
    fill_color: egui::Color32,
    width: f32,
) {
    let height = 10.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    //
    ui.painter()
        .rect_filled(rect, 2.0, egui::Color32::from_rgb(40, 40, 55));

    //
    if max > 0 {
        let ratio = (current as f32 / max as f32).clamp(0.0, 1.0);
        let fill_width = rect.width() * ratio;
        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, height));
        ui.painter().rect_filled(fill_rect, 2.0, fill_color);
    }

    //
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(0.5, egui::Color32::from_rgb(80, 80, 100)),
    );
}
