//!
//!
//!
//!
//!

use crate::core::role::*;
use eframe::egui;

///
pub enum CharCreationAction {
    ///
    InProgress,
    ///
    Done(CharCreationChoices),
    ///
    BackToTitle,
}

///
pub fn render_char_creation(
    ctx: &egui::Context,
    step: &mut CharCreationStep,
    choices: &mut CharCreationChoices,
    name_buf: &mut String,
) -> CharCreationAction {
    let mut result = CharCreationAction::InProgress;

    // [v1.9.0
    ctx.set_visuals(egui::Visuals::dark());

    //
    let dark_frame = egui::Frame::default()
        .fill(egui::Color32::from_rgb(15, 15, 20))
        .inner_margin(egui::Margin::same(20.0));

    egui::CentralPanel::default()
        .frame(dark_frame)
        .show(ctx, |ui| {
            let available = ui.available_size();

            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                //
                let step_num = match step {
                    CharCreationStep::SelectRole => 1,
                    CharCreationStep::SelectRace => 2,
                    CharCreationStep::SelectGender => 3,
                    CharCreationStep::SelectAlignment => 4,
                    CharCreationStep::EnterName => 5,
                    CharCreationStep::Confirm => 6,
                };
                ui.heading(
                    egui::RichText::new(format!("Character Creation  [{}/6]", step_num))
                        .size(24.0)
                        .color(egui::Color32::from_rgb(255, 200, 80)),
                );

                ui.separator();
                ui.add_space(10.0);

                //
                render_current_summary(ui, choices);

                ui.add_space(15.0);

                //
                match step {
                    CharCreationStep::SelectRole => {
                        render_role_selection(ui, choices, step, available);
                    }
                    CharCreationStep::SelectRace => {
                        render_race_selection(ui, choices, step);
                    }
                    CharCreationStep::SelectGender => {
                        render_gender_selection(ui, choices, step);
                    }
                    CharCreationStep::SelectAlignment => {
                        render_alignment_selection(ui, choices, step);
                    }
                    CharCreationStep::EnterName => {
                        render_name_entry(ui, choices, step, name_buf);
                    }
                    CharCreationStep::Confirm => {
                        if render_confirmation(ui, choices) {
                            //
                            result = CharCreationAction::Done(choices.clone());
                        }
                    }
                }

                ui.add_space(20.0);

                //
                ui.horizontal(|ui| {
                    //
                    if *step != CharCreationStep::SelectRole {
                        if ui
                            .button(
                                egui::RichText::new("??Back")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(200, 200, 210)),
                            )
                            .clicked()
                        {
                            go_back(step, choices);
                        }
                    } else {
                        //
                        if ui
                            .button(
                                egui::RichText::new("??Title")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(180, 100, 100)),
                            )
                            .clicked()
                        {
                            result = CharCreationAction::BackToTitle;
                        }
                    }

                    ui.add_space(20.0);

                    //
                    if *step != CharCreationStep::Confirm && *step != CharCreationStep::EnterName {
                        if ui
                            .button(
                                egui::RichText::new(" Random")
                                    .size(16.0)
                                    .color(egui::Color32::from_rgb(180, 220, 180)),
                            )
                            .clicked()
                        {
                            random_fill_current_step(step, choices);
                        }
                    }
                });
            });
        });

    result
}

///
fn render_current_summary(ui: &mut egui::Ui, choices: &CharCreationChoices) {
    ui.horizontal(|ui| {
        // 吏곸뾽
        let role_text = if let Some(role) = choices.role {
            format!("{} {}", get_role_data(role).name, role.icon())
        } else {
            "Role: ???".to_string()
        };
        ui.label(
            egui::RichText::new(role_text)
                .size(14.0)
                .color(egui::Color32::from_rgb(180, 180, 200)),
        );

        ui.label(egui::RichText::new(" | ").color(egui::Color32::from_rgb(80, 80, 90)));

        //
        let race_text = if let Some(race) = choices.race {
            format!("{}", get_race_data(race).name)
        } else {
            "Race: ???".to_string()
        };
        ui.label(
            egui::RichText::new(race_text)
                .size(14.0)
                .color(egui::Color32::from_rgb(180, 180, 200)),
        );

        ui.label(egui::RichText::new(" | ").color(egui::Color32::from_rgb(80, 80, 90)));

        //
        let gender_text = if let Some(gender) = choices.gender {
            format!("{}", gender)
        } else {
            "Gender: ???".to_string()
        };
        ui.label(
            egui::RichText::new(gender_text)
                .size(14.0)
                .color(egui::Color32::from_rgb(180, 180, 200)),
        );

        ui.label(egui::RichText::new(" | ").color(egui::Color32::from_rgb(80, 80, 90)));

        //
        let align_text = if let Some(align) = choices.alignment {
            format!("{}", align)
        } else {
            "Align: ???".to_string()
        };
        ui.label(
            egui::RichText::new(align_text)
                .size(14.0)
                .color(egui::Color32::from_rgb(180, 180, 200)),
        );
    });
}

///
fn render_role_selection(
    ui: &mut egui::Ui,
    choices: &mut CharCreationChoices,
    step: &mut CharCreationStep,
    available: egui::Vec2,
) {
    ui.label(
        egui::RichText::new("Choose your Role:")
            .size(18.0)
            .color(egui::Color32::from_rgb(220, 220, 230)),
    );

    ui.add_space(10.0);

    //
    let col_count = 3;
    let roles = all_roles();
    let button_width = (available.x * 0.8 / col_count as f32).min(220.0);

    egui::Grid::new("role_grid")
        .num_columns(col_count)
        .spacing([10.0, 8.0])
        .show(ui, |ui| {
            for (i, &role) in roles.iter().enumerate() {
                let data = get_role_data(role);
                let stats = &data.base_stats;

                //
                let label = format!(
                    "{} {}\nHP:{} Str:{} Int:{}\n{}",
                    role.icon(),
                    data.name,
                    data.base_hp,
                    stats.str_,
                    stats.int,
                    role.description(),
                );

                let is_selected = choices.role == Some(role);
                let btn = ui.add_sized(
                    [button_width, 70.0],
                    egui::Button::new(egui::RichText::new(label).size(12.0).color(
                        if is_selected {
                            egui::Color32::from_rgb(255, 220, 100)
                        } else {
                            egui::Color32::from_rgb(200, 200, 210)
                        },
                    ))
                    .selected(is_selected),
                );

                if btn.clicked() {
                    choices.role = Some(role);
                    //
                    choices.race = None;
                    choices.gender = None;
                    choices.alignment = None;
                    *step = CharCreationStep::SelectRace;
                }

                //
                if (i + 1) % col_count == 0 {
                    ui.end_row();
                }
            }
        });
}

///
fn render_race_selection(
    ui: &mut egui::Ui,
    choices: &mut CharCreationChoices,
    step: &mut CharCreationStep,
) {
    let Some(role) = choices.role else { return };
    let valid_races = valid_races_for_role(role);

    ui.label(
        egui::RichText::new("Choose your Race:")
            .size(18.0)
            .color(egui::Color32::from_rgb(220, 220, 230)),
    );

    ui.add_space(10.0);

    for &race in all_races() {
        let is_valid = valid_races.contains(&race);
        let race_data = get_race_data(race);

        let label = format!(
            "{} ??{}{}",
            race_data.name,
            race.description(),
            if !is_valid { " [unavailable]" } else { "" },
        );

        let is_selected = choices.race == Some(race);

        let btn = ui.add_sized(
            [400.0, 36.0],
            egui::Button::new(egui::RichText::new(label).size(15.0).color(if !is_valid {
                egui::Color32::from_rgb(80, 80, 90)
            } else if is_selected {
                egui::Color32::from_rgb(255, 220, 100)
            } else {
                egui::Color32::from_rgb(200, 200, 210)
            }))
            .selected(is_selected),
        );

        if btn.clicked() && is_valid {
            choices.race = Some(race);
            choices.gender = None;
            choices.alignment = None;
            *step = CharCreationStep::SelectGender;
        }
    }
}

///
fn render_gender_selection(
    ui: &mut egui::Ui,
    choices: &mut CharCreationChoices,
    step: &mut CharCreationStep,
) {
    let Some(role) = choices.role else { return };
    let valid_genders = valid_genders_for_role(role);

    ui.label(
        egui::RichText::new("Choose your Gender:")
            .size(18.0)
            .color(egui::Color32::from_rgb(220, 220, 230)),
    );

    ui.add_space(10.0);

    for &gender in &[Gender::Male, Gender::Female] {
        let is_valid = valid_genders.contains(&gender);

        //
        let role_data = get_role_data(role);
        let display_role_name = if gender == Gender::Female {
            role_data.name_female.unwrap_or(role_data.name)
        } else {
            role_data.name
        };

        let label = format!(
            "{} ??(You will be a {}){}",
            gender,
            display_role_name,
            if !is_valid { " [unavailable]" } else { "" },
        );

        let is_selected = choices.gender == Some(gender);

        let btn = ui.add_sized(
            [400.0, 40.0],
            egui::Button::new(egui::RichText::new(label).size(16.0).color(if !is_valid {
                egui::Color32::from_rgb(80, 80, 90)
            } else if is_selected {
                egui::Color32::from_rgb(255, 220, 100)
            } else {
                egui::Color32::from_rgb(200, 200, 210)
            }))
            .selected(is_selected),
        );

        if btn.clicked() && is_valid {
            choices.gender = Some(gender);
            choices.alignment = None;
            *step = CharCreationStep::SelectAlignment;
        }
    }
}

///
fn render_alignment_selection(
    ui: &mut egui::Ui,
    choices: &mut CharCreationChoices,
    step: &mut CharCreationStep,
) {
    let Some(role) = choices.role else { return };
    let Some(race) = choices.race else { return };
    let valid_aligns = valid_alignments_for(role, race);

    ui.label(
        egui::RichText::new("Choose your Alignment:")
            .size(18.0)
            .color(egui::Color32::from_rgb(220, 220, 230)),
    );

    ui.add_space(10.0);

    for &align in &[Alignment::Lawful, Alignment::Neutral, Alignment::Chaotic] {
        let is_valid = valid_aligns.contains(&align);

        let desc = match align {
            Alignment::Lawful => "Follows rules, protects the weak",
            Alignment::Neutral => "Maintains balance in all things",
            Alignment::Chaotic => "Values freedom above all else",
        };

        let label = format!(
            "{} ??{}{}",
            align,
            desc,
            if !is_valid { " [unavailable]" } else { "" },
        );

        let is_selected = choices.alignment == Some(align);

        let btn = ui.add_sized(
            [400.0, 40.0],
            egui::Button::new(egui::RichText::new(label).size(16.0).color(if !is_valid {
                egui::Color32::from_rgb(80, 80, 90)
            } else if is_selected {
                egui::Color32::from_rgb(255, 220, 100)
            } else {
                egui::Color32::from_rgb(200, 200, 210)
            }))
            .selected(is_selected),
        );

        if btn.clicked() && is_valid {
            choices.alignment = Some(align);
            *step = CharCreationStep::EnterName;
        }
    }
}

///
fn render_name_entry(
    ui: &mut egui::Ui,
    choices: &mut CharCreationChoices,
    step: &mut CharCreationStep,
    name_buf: &mut String,
) {
    ui.label(
        egui::RichText::new("Enter your name:")
            .size(18.0)
            .color(egui::Color32::from_rgb(220, 220, 230)),
    );

    ui.add_space(10.0);

    //
    let edit = ui.add_sized(
        [300.0, 30.0],
        egui::TextEdit::singleline(name_buf)
            .font(egui::TextStyle::Heading)
            .hint_text("Your character's name..."),
    );

    //
    if edit.gained_focus() || name_buf.is_empty() {
        edit.request_focus();
    }

    ui.add_space(10.0);

    //
    let display_name = if name_buf.is_empty() {
        choices.role_display_name()
    } else {
        name_buf.clone()
    };

    ui.label(
        egui::RichText::new(format!(
            "Preview: {} the {}",
            display_name,
            choices.role_display_name()
        ))
        .size(14.0)
        .color(egui::Color32::from_rgb(150, 150, 170)),
    );

    ui.add_space(10.0);

    //
    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
    let continue_clicked = ui
        .button(
            egui::RichText::new("Continue ->")
                .size(16.0)
                .color(egui::Color32::from_rgb(100, 220, 100)),
        )
        .clicked();

    if enter_pressed || continue_clicked {
        //
        if name_buf.is_empty() {
            choices.name = choices.role_display_name();
        } else {
            choices.name = name_buf.clone();
        }
        *step = CharCreationStep::Confirm;
    }
}

///
///
fn render_confirmation(ui: &mut egui::Ui, choices: &CharCreationChoices) -> bool {
    ui.label(
        egui::RichText::new("Confirm your character:")
            .size(18.0)
            .color(egui::Color32::from_rgb(220, 220, 230)),
    );

    ui.add_space(15.0);

    //
    let Some(role) = choices.role else {
        return false;
    };
    let Some(race) = choices.race else {
        return false;
    };
    let role_data = get_role_data(role);
    let race_data = get_race_data(race);
    let stats = &role_data.base_stats;

    egui::Frame::default()
        .inner_margin(egui::Margin::same(15.0))
        .fill(egui::Color32::from_rgb(30, 30, 40))
        .rounding(8.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(80, 80, 100)))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(format!(
                    "{} the {} {} {}",
                    choices.name,
                    race_data.adjective,
                    choices.role_display_name(),
                    role.icon(),
                ))
                .size(20.0)
                .color(egui::Color32::from_rgb(255, 220, 100)),
            );

            ui.add_space(8.0);

            ui.label(
                egui::RichText::new(format!(
                    "Gender: {}  |  Alignment: {}",
                    choices
                        .gender
                        .map(|g| format!("{}", g))
                        .unwrap_or_else(|| "???".to_string()),
                    choices
                        .alignment
                        .map(|a| format!("{}", a))
                        .unwrap_or_else(|| "???".to_string()),
                ))
                .size(14.0)
                .color(egui::Color32::from_rgb(180, 180, 200)),
            );

            ui.add_space(8.0);

            //
            ui.label(
                egui::RichText::new(format!(
                    "Str: {}  Int: {}  Wis: {}  Dex: {}  Con: {}  Cha: {}",
                    stats.str_, stats.int, stats.wis, stats.dex, stats.con, stats.cha,
                ))
                .size(14.0)
                .monospace()
                .color(egui::Color32::from_rgb(160, 220, 160)),
            );

            ui.label(
                egui::RichText::new(format!(
                    "Base HP: {}  |  Starting Rank: {}",
                    role_data.base_hp + race_data.hp_bonus,
                    role_data.ranks[0]
                ))
                .size(14.0)
                .color(egui::Color32::from_rgb(180, 180, 200)),
            );

            if !role_data.gods.0.is_empty() {
                ui.label(
                    egui::RichText::new(format!(
                        "Gods: {} / {} / {}",
                        role_data.gods.0, role_data.gods.1, role_data.gods.2,
                    ))
                    .size(12.0)
                    .color(egui::Color32::from_rgb(140, 140, 160)),
                );
            }
        });

    ui.add_space(20.0);

    //
    let start_btn = ui.add_sized(
        [260.0, 45.0],
        egui::Button::new(
            egui::RichText::new("Begin Adventure!")
                .size(22.0)
                .color(egui::Color32::from_rgb(100, 255, 100)),
        ),
    );

    start_btn.clicked()
}

///
fn go_back(step: &mut CharCreationStep, choices: &mut CharCreationChoices) {
    match step {
        CharCreationStep::SelectRace => {
            choices.race = None;
            *step = CharCreationStep::SelectRole;
        }
        CharCreationStep::SelectGender => {
            choices.gender = None;
            *step = CharCreationStep::SelectRace;
        }
        CharCreationStep::SelectAlignment => {
            choices.alignment = None;
            *step = CharCreationStep::SelectGender;
        }
        CharCreationStep::EnterName => {
            *step = CharCreationStep::SelectAlignment;
        }
        CharCreationStep::Confirm => {
            *step = CharCreationStep::EnterName;
        }
        _ => {}
    }
}

///
fn random_fill_current_step(step: &mut CharCreationStep, choices: &mut CharCreationChoices) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    match step {
        CharCreationStep::SelectRole => {
            let roles = all_roles();
            let idx = rng.gen_range(0..roles.len());
            choices.role = Some(roles[idx]);
            choices.race = None;
            choices.gender = None;
            choices.alignment = None;
            *step = CharCreationStep::SelectRace;
        }
        CharCreationStep::SelectRace => {
            if let Some(role) = choices.role {
                let valid = valid_races_for_role(role);
                if !valid.is_empty() {
                    let idx = rng.gen_range(0..valid.len());
                    choices.race = Some(valid[idx]);
                    choices.gender = None;
                    choices.alignment = None;
                    *step = CharCreationStep::SelectGender;
                }
            } // if let Some(role)
        }
        CharCreationStep::SelectGender => {
            if let Some(role) = choices.role {
                let valid = valid_genders_for_role(role);
                if !valid.is_empty() {
                    let idx = rng.gen_range(0..valid.len());
                    choices.gender = Some(valid[idx]);
                    choices.alignment = None;
                    *step = CharCreationStep::SelectAlignment;
                }
            } // if let Some(role)
        }
        CharCreationStep::SelectAlignment => {
            if let (Some(role), Some(race)) = (choices.role, choices.race) {
                let valid = valid_alignments_for(role, race);
                if !valid.is_empty() {
                    let idx = rng.gen_range(0..valid.len());
                    choices.alignment = Some(valid[idx]);
                    *step = CharCreationStep::EnterName;
                }
            } // if let (Some(role), Some(race))
        }
        _ => {}
    }
}
