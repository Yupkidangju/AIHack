use aihack_runtime::GameSession;
use aihack_tui::tui::{compute_layout, TuiApp, UiRuntimeConfig};

#[test]
fn tui_package_owns_the_runtime_app_and_layout() {
    let mut app = TuiApp::new(GameSession::new_for_playing(42), UiRuntimeConfig::default());

    assert_eq!(app.observation().turn, 0);
    assert!(compute_layout(80, 28).validate().is_ok());
    assert!(app.run_single_frame(80, 28).is_ok());
}
