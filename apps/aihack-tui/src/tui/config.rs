use serde::{Deserialize, Serialize};

use super::{MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH};

/// [v0.1.0] Phase 10 TUI runtime 설정이다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiRuntimeConfig {
    pub enable_mouse: bool,
    pub enable_animations: bool,
    pub reduced_motion: bool,
    pub high_contrast: bool,
    pub min_terminal_width: u16,
    pub min_terminal_height: u16,
}

impl Default for UiRuntimeConfig {
    fn default() -> Self {
        Self {
            enable_mouse: true,
            enable_animations: true,
            reduced_motion: false,
            high_contrast: false,
            min_terminal_width: MIN_TERMINAL_WIDTH,
            min_terminal_height: MIN_TERMINAL_HEIGHT,
        }
    }
}
