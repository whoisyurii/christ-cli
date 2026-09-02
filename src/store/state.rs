use crate::ui::theme::ThemeName;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub book_index: usize,
    pub chapter: u32,
    pub scroll_position: u16,
    pub active_panel: u8, // 0=Books, 1=Chapters, 2=Scripture
    #[serde(default)]
    pub theme: ThemeName,
    #[serde(default = "default_translation")]
    pub translation: String,
    #[serde(default)]
    pub view_mode: u8, // 0 = verse-per-line, 1 = paragraph
    #[serde(default)]
    pub selected_verse: u32, // 0-based verse index in the current chapter
    /// The intro banner plays on the first launch only (#10); this flips to
    /// true after any completed session. `christ intro` replays it.
    #[serde(default)]
    pub banner_shown: bool,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            book_index: 0,
            chapter: 1,
            scroll_position: 0,
            active_panel: 0,
            theme: ThemeName::default(),
            translation: default_translation(),
            view_mode: 0,
            selected_verse: 0,
            banner_shown: false,
        }
    }
}

fn default_translation() -> String {
    "KJV".to_string()
}

fn state_path() -> Option<PathBuf> {
    let dirs = ProjectDirs::from("", "", "christ-cli")?;
    let data_dir = dirs.data_dir();
    Some(data_dir.join("state.json"))
}

pub fn load() -> SessionState {
    let Some(path) = state_path() else {
        return SessionState::default();
    };

    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => SessionState::default(),
    }
}

pub fn save(state: &SessionState) {
    let Some(path) = state_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = fs::write(&path, json);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_state_defaults_to_showing_banner_once() {
        // Pre-0.6.1 state files have no banner_shown field: it must
        // default to false so the banner plays one more time, and to
        // true-after-save from then on.
        let legacy = r#"{"book_index":42,"chapter":3,"scroll_position":0,"active_panel":2}"#;
        let state: SessionState = serde_json::from_str(legacy).unwrap();
        assert!(!state.banner_shown);
        assert_eq!(state.book_index, 42);
        assert_eq!(state.translation, "KJV");
    }

    #[test]
    fn default_session_is_kjv_genesis_1() {
        // Default::default() must match serde defaults — derive(Default)
        // left translation as "" and chapter as 0, so a fresh install
        // skipped the bundled KJV path and showed an empty scripture panel.
        let state = SessionState::default();
        assert_eq!(state.translation, "KJV");
        assert_eq!(state.chapter, 1);
        assert_eq!(state.book_index, 0);
    }
}
