use ratatui::style::Color;

/// 실제 renderer가 소비하는 semantic theme token이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiTheme {
    pub fg: Color,
    pub bg: Color,
    pub accent: Color,
    pub danger: Color,
    pub muted: Color,
}

impl UiTheme {
    pub fn standard() -> Self {
        Self {
            fg: Color::White,
            bg: Color::Black,
            accent: Color::Cyan,
            danger: Color::Red,
            muted: Color::DarkGray,
        }
    }

    pub fn high_contrast() -> Self {
        Self {
            fg: Color::White,
            bg: Color::Black,
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
