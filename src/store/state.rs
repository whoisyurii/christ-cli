use crate::ui::theme::ThemeName;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
