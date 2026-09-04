use iced::{Color, theme::Palette};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn palette(&self) -> Palette {
        match self {
            Theme::Dark => Palette {
                background: Color::from_rgb(0.15, 0.15, 0.15),
                text: Color::from_rgb(0.9, 0.9, 0.9),
                primary: Color::from_rgb(0.3, 0.5, 0.8),
                success: Color::from_rgb(0.3, 0.7, 0.3),
                danger: Color::from_rgb(0.8, 0.3, 0.3),
            },
            Theme::Light => Palette {
                background: Color::from_rgb(0.95, 0.95, 0.95),
                text: Color::from_rgb(0.1, 0.1, 0.1),
                primary: Color::from_rgb(0.2, 0.4, 0.7),
                success: Color::from_rgb(0.2, 0.6, 0.2),
                danger: Color::from_rgb(0.7, 0.2, 0.2),
            },
        }
    }

    pub fn editor_bg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.12, 0.12, 0.12),
            Theme::Light => Color::from_rgb(1.0, 1.0, 1.0),
        }
    }

    pub fn editor_fg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.85, 0.85, 0.85),
            Theme::Light => Color::from_rgb(0.1, 0.1, 0.1),
        }
    }

    pub fn line_number_bg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.18, 0.18, 0.18),
            Theme::Light => Color::from_rgb(0.92, 0.92, 0.92),
        }
    }

    pub fn line_number_fg(&self) -> Option<Color> {
        match self {
            Theme::Dark => Some(Color::from_rgb(0.5, 0.5, 0.5)),
            Theme::Light => Some(Color::from_rgb(0.6, 0.6, 0.6)),
        }
    }

    pub fn selection_bg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.25, 0.4, 0.6),
            Theme::Light => Color::from_rgb(0.7, 0.85, 1.0),
        }
    }

    pub fn cursor_color(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.8, 0.8, 0.8),
            Theme::Light => Color::from_rgb(0.2, 0.2, 0.2),
        }
    }

    pub fn status_bar_bg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.2, 0.2, 0.2),
            Theme::Light => Color::from_rgb(0.85, 0.85, 0.85),
        }
    }

    pub fn find_bar_bg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.22, 0.22, 0.22),
            Theme::Light => Color::from_rgb(0.88, 0.88, 0.88),
        }
    }

    pub fn toggle(&self) -> Theme {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}
