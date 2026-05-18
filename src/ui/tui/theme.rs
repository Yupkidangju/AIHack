use ratatui::style::Color;

/// [v0.1.0] Phase 10 최소 테마 토큰이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiTheme {
    pub fg: Color,
    pub accent: Color,
    pub danger: Color,
    pub muted: Color,
}

impl UiTheme {
    pub fn standard() -> Self {
        Self {
            fg: Color::White,
            accent: Color::Cyan,
            danger: Color::Red,
            muted: Color::DarkGray,
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            fg: Color::White,
            accent: Color::Yellow,
            danger: Color::LightRed,
            muted: Color::Gray,
        }
    }

    pub fn from_high_contrast(enabled: bool) -> Self {
        if enabled {
            Self::high_contrast()
        } else {
            Self::standard()
        }
    }
}
