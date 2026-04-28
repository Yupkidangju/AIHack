// [v2.3.0
//
//
//
//
//
//
//
//

use eframe::egui;

impl super::NetHackApp {
    ///
    pub(crate) fn apply_global_style(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(230, 230, 230));
        style.visuals.panel_fill = egui::Color32::from_rgb(20, 20, 25);
        style.visuals.window_fill = egui::Color32::from_rgb(25, 25, 30);
        ctx.set_style(style);
    }

    ///
    pub(crate) fn handle_title_screen(&mut self, ctx: &egui::Context) {
        use crate::ui::screens::title::{render_title_screen, TitleAction};
        match render_title_screen(ctx) {
            TitleAction::NewGame => {
                //
                self.ctx.app_state = crate::core::role::AppState::CharacterCreation {
                    step: crate::core::role::CharCreationStep::SelectRole,
                    choices: crate::core::role::CharCreationChoices::new(),
                };
                self.ctx.char_creation_step = crate::core::role::CharCreationStep::SelectRole;
                self.ctx.char_creation_choices = crate::core::role::CharCreationChoices::new();
                self.ctx.char_name_buf.clear();
            }
            TitleAction::Continue => {
                //
                if self.ctx.game_initialized {
                    self.ctx.app_state = crate::core::role::AppState::Playing;
                }
            }
            TitleAction::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            TitleAction::Settings | TitleAction::None => {}
        }
    }

    ///
    pub(crate) fn handle_char_creation_screen(&mut self, ctx: &egui::Context) {
        use crate::ui::screens::char_creation::{render_char_creation, CharCreationAction};
        match render_char_creation(
            ctx,
            &mut self.ctx.char_creation_step,
            &mut self.ctx.char_creation_choices,
            &mut self.ctx.char_name_buf,
        ) {
            CharCreationAction::Done(choices) => {
                //
                self.initialize_game_with_choices(&choices);
                self.ctx.app_state = crate::core::role::AppState::Playing;
            }
            CharCreationAction::BackToTitle => {
                self.ctx.app_state = crate::core::role::AppState::Title;
            }
            CharCreationAction::InProgress => {}
        }
    }

    ///
    ///
    pub(crate) fn check_game_over(&mut self) -> bool {
        if let crate::core::game_state::GameState::GameOver { ref message } = self.input.game_state {
            //
            let msg = message.clone();
            let (score, turns, max_depth, epitaph) = {
                use legion::*;
                let mut xp: u64 = 0;
                let mut gold: u64 = 0;
                let mut turn_count: u64 = 0;
                let mut dlvl: i32 = 1;
                let mut char_info = String::new();

                //
                if let Some(turn) = self.game.resources.get::<u64>() {
                    turn_count = *turn;
                }

                //
                let mut q = <&crate::core::entity::player::Player>::query()
                    .filter(component::<crate::core::entity::PlayerTag>());
                if let Some(p) = q.iter(&self.game.world).next() {
                    xp = p.experience;
                    gold = p.gold;
                    dlvl = p.level;
                    char_info = format!("Level {} {:?} ({:?})", p.exp_level, p.role, p.race);
                }

                let final_score = xp + gold;
                (final_score, turn_count, dlvl, char_info)
            };

            self.ctx.app_state = crate::core::role::AppState::GameOver {
                message: msg,
                score,
                turns,
                max_depth,
                epitaph,
            };
            return true;
        }
        false
    }

    ///
    pub(crate) fn handle_game_over_screen(
        &mut self,
        ctx: &egui::Context,
        message: String,
        score: u64,
        turns: u64,
        max_depth: i32,
        epitaph: String,
    ) {
        use crate::ui::screens::game_over::{render_game_over_screen, GameOverAction};
        match render_game_over_screen(ctx, &message, score, turns, max_depth, &epitaph) {
            GameOverAction::BackToTitle => {
                self.restart_game();
                self.ctx.app_state = crate::core::role::AppState::Title;
            }
            GameOverAction::RestartNow => {
                self.restart_game();
                self.ctx.app_state = crate::core::role::AppState::CharacterCreation {
                    step: crate::core::role::CharCreationStep::SelectRole,
                    choices: crate::core::role::CharCreationChoices::new(),
                };
                self.ctx.char_creation_step = crate::core::role::CharCreationStep::SelectRole;
                self.ctx.char_creation_choices = crate::core::role::CharCreationChoices::new();
                self.ctx.char_name_buf.clear();
            }
            GameOverAction::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            GameOverAction::None => {}
        }
    }
}
