use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    // === App Shell Colors ===
    pub fn canvas_bg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.102, 0.106, 0.180), // #1a1b2e
            Theme::Light => Color::from_rgb(0.95, 0.95, 0.95),
        }
    }

    pub fn surface(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.145, 0.149, 0.251), // #252640
            Theme::Light => Color::from_rgb(0.95, 0.95, 0.95),
        }
    }

    pub fn surface_recessed(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.122, 0.125, 0.208), // #1f2035
            Theme::Light => Color::from_rgb(0.90, 0.90, 0.90),
        }
    }

    pub fn surface_hover(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.176, 0.180, 0.302), // #2d2e4d
            Theme::Light => Color::from_rgb(0.88, 0.88, 0.88),
        }
    }

    pub fn border(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.176, 0.180, 0.290), // #2d2e4a
            Theme::Light => Color::from_rgb(0.80, 0.80, 0.80),
        }
    }

    // === Accent ===
    pub fn primary(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.643, 0.788, 1.0), // #a4c9ff
            Theme::Light => Color::from_rgb(0.290, 0.620, 1.0),
        }
    }

    pub fn primary_container(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.290, 0.620, 1.0), // #4a9eff
            Theme::Light => Color::from_rgb(0.200, 0.500, 0.900),
        }
    }

    pub fn primary_tint(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgba(0.290, 0.620, 1.0, 0.15), // rgba(74, 158, 255, 0.15)
            Theme::Light => Color::from_rgba(0.290, 0.620, 1.0, 0.10),
        }
    }

    // === Text Colors ===
    pub fn text_primary(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.878, 0.878, 0.925), // #e1e0fb
            Theme::Light => Color::from_rgb(0.1, 0.1, 0.1),
        }
    }

    pub fn text_secondary(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.620, 0.620, 0.722), // #9e9eb8
            Theme::Light => Color::from_rgb(0.4, 0.4, 0.4),
        }
    }

    pub fn text_subtle(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.420, 0.420, 0.541), // #6b6b8a
            Theme::Light => Color::from_rgb(0.6, 0.6, 0.6),
        }
    }

    pub fn text_on_primary(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.0, 0.192, 0.365), // #00315d
            Theme::Light => Color::from_rgb(1.0, 1.0, 1.0),
        }
    }

    // === Document Colors ===
    pub fn page_bg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(1.0, 1.0, 1.0), // #ffffff
            Theme::Light => Color::from_rgb(1.0, 1.0, 1.0),
        }
    }

    pub fn page_text(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.102, 0.102, 0.102), // #1a1a1a
            Theme::Light => Color::from_rgb(0.1, 0.1, 0.1),
        }
    }

    pub fn page_selection(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgba(0.290, 0.620, 1.0, 0.28),
            Theme::Light => Color::from_rgba(0.290, 0.620, 1.0, 0.25),
        }
    }

    pub fn line_number_bg(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgba(0.0, 0.0, 0.0, 0.0),
            Theme::Light => Color::from_rgba(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn line_number_fg(&self) -> Option<Color> {
        match self {
            Theme::Dark => Some(Color::from_rgb(0.420, 0.420, 0.541)), // #6b6b8a
            Theme::Light => Some(Color::from_rgb(0.6, 0.6, 0.6)),
        }
    }

    pub fn cursor_color(&self) -> Color {
        match self {
            Theme::Dark => Color::from_rgb(0.290, 0.620, 1.0), // #4a9eff
            Theme::Light => Color::from_rgb(0.290, 0.620, 1.0),
        }
    }

    pub fn toggle(&self) -> Theme {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}
