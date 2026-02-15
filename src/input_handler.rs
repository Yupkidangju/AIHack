// [v2.0.0 Phase R1] 입력 처리 핸들러
//
//
//

use eframe::egui;

impl super::NetHackApp {
    /// 입력 상태 폴링 (Bug #3 해결: 이벤트 기반의 누락 방지)
    ///
    pub(crate) fn poll_input(
        &mut self,
        ctx: &egui::Context,
    ) -> (crate::ui::input::Command, Option<char>) {
        use crate::ui::input::Command;
        let mut spell_key = None;

        // 텍스트 입력 처리 (주문 단축키)
        ctx.input(|i| {
            for event in &i.events {
                if let egui::Event::Text(t) = event {
                    if let Some(c) = t.chars().next() {
                        if c.is_ascii_lowercase() {
                            spell_key = Some(c);
                        }
                    }
                }
            }
        });

        let cmd = ctx.input(|i| {
            let mods = i.modifiers;

            // ================================================
            // Ctrl 조합 (최우선)
            // ================================================
            if mods.ctrl {
                if i.key_pressed(egui::Key::P) {
                    return Command::LogHistory;
                }
                if i.key_pressed(egui::Key::O) {
                    return Command::Overview;
                }
            }

            // ================================================
            // Shift 조합 — 달리기 + 기존 Shift 명령
            // ================================================
            if mods.shift {
                // [M4] 달리기 (Shift+이동키)
                if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft) {
                    return Command::RunW;
                }
                if i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown) {
                    return Command::RunS;
                }
                if i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp) {
                    return Command::RunN;
                }
                if i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) {
                    return Command::RunE;
                }
                if i.key_pressed(egui::Key::Y) {
                    return Command::RunNW;
                }
                if i.key_pressed(egui::Key::U) {
                    return Command::RunNE;
                }
                if i.key_pressed(egui::Key::B) {
                    return Command::RunSW;
                }
                if i.key_pressed(egui::Key::N) {
                    return Command::RunSE;
                }

                // 기존 Shift 명령
                if i.key_pressed(egui::Key::Period) {
                    return Command::Descend; // >
                }
                if i.key_pressed(egui::Key::Comma) {
                    return Command::Ascend; // <
                }
                if i.key_pressed(egui::Key::Q) {
                    return Command::Quit;
                }
                if i.key_pressed(egui::Key::S) {
                    return Command::Save;
                }
                if i.key_pressed(egui::Key::Z) {
                    return Command::Cast;
                }
                if i.key_pressed(egui::Key::W) {
                    return Command::Wear;
                }
                if i.key_pressed(egui::Key::T) {
                    return Command::TakeOff;
                }
                if i.key_pressed(egui::Key::C) {
                    return Command::CharacterSheet;
                }
                if i.key_pressed(egui::Key::X) {
                    return Command::TwoWeapon;
                }
                if i.key_pressed(egui::Key::O) {
                    return Command::Offer;
                }
                if i.key_pressed(egui::Key::E) {
                    return Command::Engrave;
                }
                if i.key_pressed(egui::Key::I) {
                    return Command::InventoryClass;
                }
                if i.key_pressed(egui::Key::P) {
                    return Command::Pray;
                }
                // ? (Shift+/)
                if i.key_pressed(egui::Key::Slash) {
                    return Command::Help;
                }
                //
            }

            // ================================================
            // 이동 (일반 — Shift 없이)
            // ================================================
            if !mods.shift {
                if i.key_pressed(egui::Key::H) || i.key_pressed(egui::Key::ArrowLeft) {
                    return Command::MoveW;
                }
                if i.key_pressed(egui::Key::J) || i.key_pressed(egui::Key::ArrowDown) {
                    return Command::MoveS;
                }
                if i.key_pressed(egui::Key::K) || i.key_pressed(egui::Key::ArrowUp) {
                    return Command::MoveN;
                }
                if i.key_pressed(egui::Key::L) || i.key_pressed(egui::Key::ArrowRight) {
                    return Command::MoveE;
                }
                if i.key_pressed(egui::Key::Y) {
                    return Command::MoveNW;
                }
                if i.key_pressed(egui::Key::U) {
                    return Command::MoveNE;
                }
                if i.key_pressed(egui::Key::B) {
                    return Command::MoveSW;
                }
                if i.key_pressed(egui::Key::N) {
                    return Command::MoveSE;
                }
            }

            // ================================================
            //
            // ================================================
            if i.key_pressed(egui::Key::Period) && !mods.shift {
                return Command::Wait;
            }
            if i.key_pressed(egui::Key::Comma) && !mods.shift {
                return Command::Pickup;
            }
            if i.key_pressed(egui::Key::I) && !mods.shift {
                return Command::Inventory;
            }
            if i.key_pressed(egui::Key::O) && !mods.shift && !mods.ctrl {
                return Command::Open;
            }
            if i.key_pressed(egui::Key::C) && !mods.shift {
                return Command::Close;
            }
            if i.key_pressed(egui::Key::S) && !mods.shift {
                return Command::Search;
            }
            if i.key_pressed(egui::Key::Z) && !mods.shift {
                return Command::Zap;
            }
            if i.key_pressed(egui::Key::E) && !mods.shift {
                return Command::Eat;
            }
            if i.key_pressed(egui::Key::Q) && !mods.shift {
                return Command::Quaff;
            }
            if i.key_pressed(egui::Key::R) {
                return Command::Read;
            }
            if i.key_pressed(egui::Key::W) && !mods.shift {
                return Command::Wield;
            }
            if i.key_pressed(egui::Key::D) {
                return Command::Drop;
            }
            if i.key_pressed(egui::Key::T) && !mods.shift {
                return Command::Throw;
            }
            if i.key_pressed(egui::Key::A) {
                return Command::Apply;
            }
            if i.key_pressed(egui::Key::G) {
                return Command::Pickup;
            }
            if i.key_pressed(egui::Key::P) && !mods.shift && !mods.ctrl {
                return Command::Pay;
            }
            if i.key_pressed(egui::Key::X) && !mods.shift {
                return Command::Swap;
            }
            if i.key_pressed(egui::Key::F) {
                return Command::Fire;
            }
            if i.key_pressed(egui::Key::V) {
                return Command::Version;
            }
            if i.key_pressed(egui::Key::Escape) {
                return Command::Cancel;
            }

            Command::Unknown
        });

        // 특수 처리: Shift+C 토글 (캐릭터 시트)
        if ctx.input(|i| i.modifiers.shift && i.key_pressed(egui::Key::C)) {
            self.show_character = !self.show_character;
        }

        (cmd, spell_key)
    }
}
