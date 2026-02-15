// ============================================================================
// AIHack - A Modern Rust Roguelike
// Copyright (c) 2026 Yupkidangju. Licensed under Apache License 2.0.
//
// This file contains code derived from NetHack 3.6.7.
// Original NetHack source: Copyright (c) Stichting Mathematisch Centrum,
// Amsterdam, 1985. NetHack may be freely redistributed. See LICENSE.NGPL
// for the NetHack General Public License.
// ============================================================================
// [v2.3.0
// 모듈 선언 + eframe::App 구현 + main() 함수
//
// 핵심 로직은 다음 파일에 분산:
// - app.rs:           NetHackApp 구조체 정의 + 초기화 (new, restart_game, initialize_game_with_choices)
//
// - game_loop.rs:     게임 턴 처리 (입력→GameState 분기→시스템 스케줄→사망 체크)
// - game_ui.rs:       UI 렌더링 (egui 패널 + ratatui 맵 + 팝업 윈도우)
// - app_update.rs:    AppState 분기 핸들러 (Title/CharCreation/Playing/GameOver)

pub mod assets;
pub mod core;
pub mod generated; // [v2.0.0 R2] 자동 생성된 MonsterKind/ItemKind enum
pub mod ui;
pub mod util;

// [v2.0.0
mod app; // NetHackApp 구조체 + 초기화
mod app_update; // [v2.3.0 M8] AppState 분기 핸들러
mod game_loop; // process_game_turn() 게임 로직
mod game_ui; // render_game_ui() UI 렌더링
mod input_handler; // poll_input() 입력 처리

use app::NetHackApp;
use eframe::egui;

/// [v2.3.0 M8] 공통 상수
pub const APP_VERSION: &str = "2.3.0";
pub const APP_TITLE: &str = "AIHack";

// ======================================================================
//
// ======================================================================
impl eframe::App for NetHackApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 공통 스타일 적용
        self.apply_global_style(ctx);

        // [v2.3.0
        match self.app_state.clone() {
            crate::core::role::AppState::Title => {
                self.handle_title_screen(ctx);
                return;
            }
            crate::core::role::AppState::CharacterCreation { .. } => {
                self.handle_char_creation_screen(ctx);
                return;
            }
            crate::core::role::AppState::Playing => {
                // GameState::GameOver 감지 → AppState::GameOver 전환
                if self.check_game_over() {
                    return;
                }
                // fall through → 게임 로직 + UI 렌더링
            }
            crate::core::role::AppState::GameOver {
                ref message,
                score,
                turns,
                max_depth,
                ref epitaph,
            } => {
                self.handle_game_over_screen(
                    ctx,
                    message.clone(),
                    score,
                    turns,
                    max_depth,
                    epitaph.clone(),
                );
                return;
            }
        }

        // Playing 상태: 게임 턴 처리 + UI 렌더링
        self.process_game_turn(ctx);
        self.render_game_ui(ctx);
    }
}

fn main() -> eframe::Result<()> {
    println!(
        "[v{}] {} 인프라 및 에셋 로더가 준비되었습니다.",
        APP_VERSION, APP_TITLE
    );

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 800.0])
            .with_title(APP_TITLE),
        ..Default::default()
    };

    eframe::run_native(
        "nethack_rs",
        native_options,
        Box::new(|cc| Box::new(NetHackApp::new(cc))),
    )
}
