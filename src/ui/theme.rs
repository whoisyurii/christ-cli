use ratatui::style::Color;
use serde::{Deserialize, Serialize};

pub struct Theme {
    pub bg: Color,
    pub surface: Color,
    pub border: Color,
    pub border_active: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub highlight_bg: Color,
    pub search_match: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ThemeName {
    Slate,
    Midnight,
    Parchment,
    Gospel,
}

impl ThemeName {
    pub fn next(self) -> Self {
        match self {
            ThemeName::Slate => ThemeName::Midnight,
            ThemeName::Midnight => ThemeName::Parchment,
            ThemeName::Parchment => ThemeName::Gospel,
            ThemeName::Gospel => ThemeName::Slate,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ThemeName::Slate => "Slate",
            ThemeName::Midnight => "Midnight",
            ThemeName::Parchment => "Parchment",
            ThemeName::Gospel => "Gospel",
        }
    }
}

impl Default for ThemeName {
    fn default() -> Self {
        ThemeName::Slate
    }
}

pub fn get_theme(name: ThemeName) -> Theme {
    match name {
        ThemeName::Slate => SLATE,
        ThemeName::Midnight => MIDNIGHT,
        ThemeName::Parchment => PARCHMENT,
        ThemeName::Gospel => GOSPEL,
    }
}

/// Slate — cool blue-gray dark theme (default)
const SLATE: Theme = Theme {
    bg: Color::Rgb(15, 23, 42),            // slate-900
    surface: Color::Rgb(30, 41, 59),       // slate-800
    border: Color::Rgb(71, 85, 105),       // slate-600
    border_active: Color::Rgb(226, 232, 240), // slate-200
    text: Color::Rgb(241, 245, 249),       // slate-100
    text_dim: Color::Rgb(148, 163, 184),   // slate-400
    text_muted: Color::Rgb(100, 116, 139), // slate-500
    accent: Color::Rgb(255, 255, 255),     // white
    accent_soft: Color::Rgb(203, 213, 225),// slate-300
    highlight_bg: Color::Rgb(55, 70, 95),  // slate-700 (boosted)
    search_match: Color::Rgb(251, 191, 36),// amber-400
};

/// Midnight — shadcn/Vercel style, pure black, neutral grays, sharp contrast
const MIDNIGHT: Theme = Theme {
    bg: Color::Rgb(0, 0, 0),              // pure black
    surface: Color::Rgb(10, 10, 10),      // near-black
    border: Color::Rgb(38, 38, 38),       // neutral-800
    border_active: Color::Rgb(163, 163, 163), // neutral-400
    text: Color::Rgb(250, 250, 250),      // neutral-50
    text_dim: Color::Rgb(115, 115, 115),  // neutral-500
    text_muted: Color::Rgb(82, 82, 82),   // neutral-600
    accent: Color::Rgb(255, 255, 255),    // white
    accent_soft: Color::Rgb(212, 212, 212), // neutral-300
    highlight_bg: Color::Rgb(35, 35, 35), // neutral-800 (boosted)
    search_match: Color::Rgb(234, 179, 8),// yellow-500
};

/// Parchment — warm cream/sepia tones, comfortable long reading
const PARCHMENT: Theme = Theme {
    bg: Color::Rgb(245, 240, 225),        // warm cream
    surface: Color::Rgb(237, 230, 211),   // slightly darker cream
    border: Color::Rgb(196, 181, 153),    // warm tan
    border_active: Color::Rgb(120, 100, 70), // dark warm brown
    text: Color::Rgb(55, 47, 35),         // dark brown
    text_dim: Color::Rgb(140, 125, 100),  // muted brown
    text_muted: Color::Rgb(168, 155, 132),// light brown
    accent: Color::Rgb(40, 32, 20),       // near-black brown
    accent_soft: Color::Rgb(100, 85, 60), // medium brown
    highlight_bg: Color::Rgb(210, 195, 160), // warm tan (high contrast)
    search_match: Color::Rgb(180, 100, 30), // warm orange
};

/// Gospel — clean bright white, crisp and minimal
const GOSPEL: Theme = Theme {
    bg: Color::Rgb(255, 255, 255),        // pure white
    surface: Color::Rgb(249, 250, 251),   // gray-50
    border: Color::Rgb(209, 213, 219),    // gray-300
    border_active: Color::Rgb(55, 65, 81),// gray-700
    text: Color::Rgb(17, 24, 39),         // gray-900
    text_dim: Color::Rgb(107, 114, 128),  // gray-500
    text_muted: Color::Rgb(156, 163, 175),// gray-400
    accent: Color::Rgb(0, 0, 0),          // black
    accent_soft: Color::Rgb(75, 85, 99),  // gray-600
    highlight_bg: Color::Rgb(220, 225, 235), // blue-gray (high contrast)
    search_match: Color::Rgb(217, 119, 6),// amber-600
};
