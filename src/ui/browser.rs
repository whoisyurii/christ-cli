use crate::api::types::{Chapter, SearchResult};
use crate::data::books::BOOKS;
use crate::store::cache;
use crate::ui::theme::{Theme, ThemeName};
use crate::ui::wrap;
use std::sync::atomic::Ordering;
use unicode_width::UnicodeWidthStr;
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, ListState, Padding, Paragraph,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Books,
    Chapters,
    Scripture,
}

/// How the scripture panel lays out verses.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ViewMode {
    /// One verse per line with a selectable verse cursor.
    #[default]
    VersePerLine,
    /// Verses flow together for a natural reading experience.
    Paragraph,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Off,
    Active {
        query: String,
        results: Vec<SearchResult>,
        list_state: ListState,
    },
}

pub struct TranslationInfo {
    pub code: &'static str,
    pub name: &'static str,
    pub lang: &'static str,
    pub offline: bool,
}

pub const TRANSLATIONS: &[TranslationInfo] = &[
    // English
    TranslationInfo { code: "KJV", name: "King James Version", lang: "English", offline: true },
    TranslationInfo { code: "WEB", name: "World English Bible", lang: "English", offline: false },
    TranslationInfo { code: "NKJV", name: "New King James Version", lang: "English", offline: false },
    TranslationInfo { code: "ESV", name: "English Standard Version", lang: "English", offline: false },
    TranslationInfo { code: "NIV", name: "New International Version", lang: "English", offline: false },
    TranslationInfo { code: "NLT", name: "New Living Translation", lang: "English", offline: false },
    TranslationInfo { code: "NASB", name: "New American Standard Bible", lang: "English", offline: false },
    TranslationInfo { code: "BSB", name: "Berean Standard Bible", lang: "English", offline: false },
    TranslationInfo { code: "NET", name: "New English Translation", lang: "English", offline: false },
    TranslationInfo { code: "MSG", name: "The Message", lang: "English", offline: false },
    TranslationInfo { code: "YLT", name: "Young's Literal Translation", lang: "English", offline: false },
    // Українська
    TranslationInfo { code: "UBIO", name: "Переклад Огієнка", lang: "Українська", offline: false },
    TranslationInfo { code: "UKRK", name: "Переклад Куліша", lang: "Українська", offline: false },
    // Español
    TranslationInfo { code: "RV1960", name: "Reina-Valera 1960", lang: "Español", offline: false },
    TranslationInfo { code: "NVI", name: "Nueva Versión Internacional", lang: "Español", offline: false },
    // Português
    TranslationInfo { code: "NAA", name: "Nova Almeida Atualizada (2017)", lang: "Português", offline: false },
    TranslationInfo { code: "ARA", name: "Almeida Revista e Atualizada (1993)", lang: "Português", offline: false },
    TranslationInfo { code: "ACF11", name: "Almeida Corrigida Fiel (2011)", lang: "Português", offline: false },
    TranslationInfo { code: "NVIPT", name: "Nova Versão Internacional", lang: "Português", offline: false },
    TranslationInfo { code: "NVT", name: "Nova Versão Transformadora (2016)", lang: "Português", offline: false },
    // Français
    TranslationInfo { code: "FRLSG", name: "Louis Segond 1910", lang: "Français", offline: false },
    TranslationInfo { code: "NBS", name: "Nouvelle Bible Segond", lang: "Français", offline: false },
    // Deutsch
    TranslationInfo { code: "LUT", name: "Luther Bibel", lang: "Deutsch", offline: false },
    TranslationInfo { code: "ELB", name: "Elberfelder Bibel", lang: "Deutsch", offline: false },
    // Русский
    TranslationInfo { code: "SYNOD", name: "Синодальный перевод", lang: "Русский", offline: false },
    TranslationInfo { code: "NRT", name: "Новый Русский Перевод", lang: "Русский", offline: false },
    // 中文
    TranslationInfo { code: "CUV", name: "和合本 (Traditional)", lang: "中文", offline: false },
    TranslationInfo { code: "CUNPS", name: "和合本 (Simplified)", lang: "中文", offline: false },
    // 한국어
    TranslationInfo { code: "KRV", name: "개역한글판", lang: "한국어", offline: false },
    // 日本語
    TranslationInfo { code: "JPKJV", name: "口語訳聖書", lang: "日本語", offline: false },
    // Italiano
    TranslationInfo { code: "NR06", name: "Nuova Riveduta 2006", lang: "Italiano", offline: false },
    // Nederlands
    TranslationInfo { code: "HSV17", name: "Herziene Statenvertaling", lang: "Nederlands", offline: false },
];

pub struct BrowserState {
    pub active_panel: Panel,
    pub book_list: ListState,
    pub chapter_list: ListState,
    pub scripture_scroll: u16,
    pub selected_book_idx: usize,
    pub selected_chapter: u32,
    pub current_chapter: Option<Chapter>,
    pub loading: bool,
    pub search: SearchMode,
    pub translation: String,
    pub translation_picker: bool,
    pub translation_list: ListState,
    /// Localized book names for the current translation (indexed by BOOKS order).
    /// Empty vec means use English names (KJV / fallback).
    pub localized_books: Vec<String>,
    /// Background download handle for caching a translation.
    pub download: Option<cache::DownloadHandle>,
    /// Verse to highlight after jumping from search results.
    pub highlight_verse: Option<u32>,
    /// Error message to display in the scripture panel.
    pub error: Option<String>,
    /// Verse-per-line vs paragraph rendering of the scripture panel.
    pub view_mode: ViewMode,
    /// Verse cursor for the scripture panel (verse-per-line mode).
    pub verse_list: ListState,
    /// Anchor verse index while selecting a range to copy (Y).
    pub visual_anchor: Option<usize>,
    /// Transient status-bar message after a copy, with its timestamp.
    pub copy_flash: Option<(String, std::time::Instant)>,
    /// One-shot request to scroll paragraph view to the selected verse.
    pub pending_paragraph_scroll: bool,
    /// One-shot request to derive the verse cursor from the paragraph
    /// scroll position (paragraph -> verse-per-line toggle).
    pub pending_cursor_sync: bool,
    /// Whether the help overlay (?) is open.
    pub help_open: bool,
    /// Scroll offset inside the help overlay (small terminals).
    pub help_scroll: u16,
    /// Set while browsing Books/Chapters: the scripture panel live-previews
    /// the highlighted target after a short debounce (#7).
    pub preview_pending: Option<std::time::Instant>,
}

/// How long Books/Chapters browsing must be still before the scripture
/// panel loads a preview. Keeps held-down j/k from firing a request per
/// keypress (online translations fetch over HTTP).
pub const PREVIEW_DEBOUNCE: std::time::Duration = std::time::Duration::from_millis(150);

impl BrowserState {
    pub fn new() -> Self {
        let mut book_list = ListState::default();
        book_list.select(Some(0));
        let mut chapter_list = ListState::default();
        chapter_list.select(Some(0));
        let mut verse_list = ListState::default();
        verse_list.select(Some(0));

        Self {
            active_panel: Panel::Books,
            book_list,
            chapter_list,
            scripture_scroll: 0,
            selected_book_idx: 0,
            selected_chapter: 1,
            current_chapter: None,
            loading: false,
            search: SearchMode::Off,
            translation: "KJV".to_string(),
            translation_picker: false,
            translation_list: ListState::default(),
            localized_books: Vec::new(),
            download: None,
            highlight_verse: None,
            error: None,
            view_mode: ViewMode::default(),
            verse_list,
            visual_anchor: None,
            copy_flash: None,
            pending_paragraph_scroll: false,
            pending_cursor_sync: false,
            help_open: false,
            help_scroll: 0,
            preview_pending: None,
        }
    }

    /// Restore from a saved session state.
    pub fn restore(&mut self, saved: &crate::store::state::SessionState) {
        let book_idx = saved.book_index.min(BOOKS.len() - 1);
        self.selected_book_idx = book_idx;
        self.book_list.select(Some(book_idx));

        let max_ch = BOOKS[book_idx].chapters;
        self.selected_chapter = saved.chapter.clamp(1, max_ch);
        self.chapter_list.select(Some((self.selected_chapter - 1) as usize));

        self.scripture_scroll = saved.scroll_position;
        self.active_panel = match saved.active_panel {
            0 => Panel::Books,
            1 => Panel::Chapters,
            _ => Panel::Scripture,
        };
        if !saved.translation.is_empty() {
            self.translation = saved.translation.clone();
        }
        self.view_mode = match saved.view_mode {
            1 => ViewMode::Paragraph,
            _ => ViewMode::VersePerLine,
        };
        self.verse_list.select(Some(saved.selected_verse as usize));
    }

    /// Snapshot current state for persistence.
    pub fn snapshot(&self) -> crate::store::state::SessionState {
        crate::store::state::SessionState {
            book_index: self.selected_book_idx,
            chapter: self.selected_chapter,
            scroll_position: self.scripture_scroll,
            active_panel: match self.active_panel {
                Panel::Books => 0,
                Panel::Chapters => 1,
                Panel::Scripture => 2,
            },
            translation: self.translation.clone(),
            view_mode: match self.view_mode {
                ViewMode::VersePerLine => 0,
                ViewMode::Paragraph => 1,
            },
            selected_verse: self.verse_list.selected().unwrap_or(0) as u32,
            ..Default::default()
        }
    }

    /// Returns true if the current translation is available offline (KJV or fully cached).
    /// Returns true if the translation has local data (bundled KJV or any cached chapters).
    /// Used to decide whether search can run locally (instant) vs needing API.
    pub fn is_offline(&self) -> bool {
        cache::has_cached_data(&self.translation)
    }

    /// Check if download is done and clean up the handle.
    pub fn check_download(&mut self) {
        if let Some(ref dl) = self.download {
            if dl.done.load(Ordering::Relaxed) {
                self.download = None;
            }
        }
    }

    /// Get download progress as (completed, total) or None.
    pub fn download_progress(&self) -> Option<(usize, usize)> {
        self.download.as_ref().map(|dl| {
            (dl.completed.load(Ordering::Relaxed), dl.total)
        })
    }

    /// Open translation picker, selecting the current translation.
    pub fn open_translation_picker(&mut self) {
        let current_idx = TRANSLATIONS
            .iter()
            .position(|t| t.code.eq_ignore_ascii_case(&self.translation))
            .unwrap_or(0);
        self.translation_list.select(Some(current_idx));
        self.translation_picker = true;
    }

    /// Select the translation from the picker. Returns true if translation changed.
    pub fn pick_translation(&mut self) -> bool {
        let idx = self.translation_list.selected().unwrap_or(0);
        let new_trans = TRANSLATIONS[idx].code.to_string();
        let changed = !new_trans.eq_ignore_ascii_case(&self.translation);
        self.translation = new_trans;
        self.translation_picker = false;
        changed
    }

    pub fn selected_book_name(&self) -> &'static str {
        BOOKS[self.selected_book_idx].name
    }

    /// Get the display name for a book (localized if available).
    pub fn book_display_name(&self, idx: usize) -> &str {
        if let Some(name) = self.localized_books.get(idx) {
            if !name.is_empty() {
                return name.as_str();
            }
        }
        BOOKS[idx].name
    }

    pub fn selected_book_chapters(&self) -> u32 {
        BOOKS[self.selected_book_idx].chapters
    }

    /// Move to the next panel (right arrow). If on Chapters, also selects and loads.
    pub fn next_panel_or_select(&mut self) -> bool {
        match self.active_panel {
            Panel::Books => {
                self.chapter_list.select(Some(0));
                self.active_panel = Panel::Chapters;
                false
            }
            Panel::Chapters => self.commit_chapter_selection(),
            Panel::Scripture => false, // Already rightmost
        }
    }

    /// Commit the chapter highlighted in the Chapters panel and switch to
    /// Scripture. Skips reload + scroll-reset when the chapter is unchanged
    /// so panel-hopping back to Scripture preserves the reader's position.
    fn commit_chapter_selection(&mut self) -> bool {
        let ch = self.chapter_list.selected().unwrap_or(0) as u32 + 1;
        let target_book = self.selected_book_name();
        let needs_reload = match &self.current_chapter {
            Some(c) => c.chapter != ch || c.book != target_book,
            None => true,
        };
        self.selected_chapter = ch;
        self.active_panel = Panel::Scripture;
        if needs_reload {
            self.scripture_scroll = 0;
            self.verse_list.select(Some(0));
            self.highlight_verse = None;
            self.visual_anchor = None;
        }
        needs_reload
    }

    pub fn prev_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Books => Panel::Books, // Already leftmost
            Panel::Chapters => Panel::Books,
            Panel::Scripture => Panel::Chapters,
        };
    }

    pub fn move_up(&mut self) {
        match self.active_panel {
            Panel::Books => {
                let i = self.book_list.selected().unwrap_or(0);
                if i > 0 {
                    self.book_list.select(Some(i - 1));
                    self.selected_book_idx = i - 1;
                    self.request_preview();
                }
            }
            Panel::Chapters => {
                let i = self.chapter_list.selected().unwrap_or(0);
                if i > 0 {
                    self.chapter_list.select(Some(i - 1));
                    self.request_preview();
                }
            }
            Panel::Scripture => {
                self.highlight_verse = None;
                match self.view_mode {
                    ViewMode::VersePerLine => {
                        let i = self.selected_verse_idx();
                        if i > 0 {
                            self.verse_list.select(Some(i - 1));
                        }
                    }
                    ViewMode::Paragraph => {
                        if self.scripture_scroll > 0 {
                            self.scripture_scroll -= 1;
                        }
                    }
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.active_panel {
            Panel::Books => {
                let i = self.book_list.selected().unwrap_or(0);
                if i < BOOKS.len() - 1 {
                    self.book_list.select(Some(i + 1));
                    self.selected_book_idx = i + 1;
                    self.request_preview();
                }
            }
            Panel::Chapters => {
                let i = self.chapter_list.selected().unwrap_or(0);
                let max = self.selected_book_chapters() as usize;
                if i < max - 1 {
                    self.chapter_list.select(Some(i + 1));
                    self.request_preview();
                }
            }
            Panel::Scripture => {
                self.highlight_verse = None;
                match self.view_mode {
                    ViewMode::VersePerLine => {
                        let i = self.selected_verse_idx();
                        if i + 1 < self.verse_count() {
                            self.verse_list.select(Some(i + 1));
                        }
                    }
                    ViewMode::Paragraph => {
                        self.scripture_scroll += 1;
                    }
                }
            }
        }
    }

    /// Arm the live-preview debounce timer (#7).
    fn request_preview(&mut self) {
        self.preview_pending = Some(std::time::Instant::now());
    }

    /// Whether the debounced live preview should load now.
    pub fn preview_due(&self) -> bool {
        self.preview_pending
            .is_some_and(|t| t.elapsed() >= PREVIEW_DEBOUNCE)
    }

    /// The (book, chapter) the live preview should show: chapter 1 of the
    /// highlighted book while browsing Books, the highlighted chapter while
    /// browsing Chapters.
    pub fn preview_target(&self) -> (usize, u32) {
        let chapter = match self.active_panel {
            Panel::Chapters => self.chapter_list.selected().unwrap_or(0) as u32 + 1,
            _ => 1,
        };
        (self.selected_book_idx, chapter)
    }

    /// Number of verses in the loaded chapter.
    pub fn verse_count(&self) -> usize {
        self.current_chapter.as_ref().map_or(0, |c| c.verses.len())
    }

    /// Current verse cursor, clamped to the loaded chapter.
    pub fn selected_verse_idx(&self) -> usize {
        let i = self.verse_list.selected().unwrap_or(0);
        i.min(self.verse_count().saturating_sub(1))
    }

    /// Toggle between verse-per-line and paragraph view, carrying the
    /// reading position across in both directions.
    pub fn toggle_view_mode(&mut self) {
        self.visual_anchor = None;
        self.view_mode = match self.view_mode {
            ViewMode::VersePerLine => {
                self.pending_paragraph_scroll = true;
                ViewMode::Paragraph
            }
            ViewMode::Paragraph => {
                self.pending_cursor_sync = true;
                ViewMode::VersePerLine
            }
        };
    }

    /// Move the verse cursor to the verse with this number, tolerating
    /// translations whose numbering has gaps (e.g. NIV omits Mark 9:44).
    pub fn select_verse_by_number(&mut self, number: u32) {
        let idx = self
            .current_chapter
            .as_ref()
            .and_then(|c| c.verses.iter().position(|v| v.verse == number))
            .unwrap_or(number.saturating_sub(1) as usize);
        self.verse_list.select(Some(idx));
    }

    /// Display name of the book the LOADED chapter belongs to — not the
    /// Books-panel browse cursor, which moves independently of the text.
    fn loaded_book_display_name(&self, chapter: &Chapter) -> String {
        BOOKS
            .iter()
            .position(|b| b.name.eq_ignore_ascii_case(&chapter.book))
            .map(|i| self.book_display_name(i).to_string())
            .unwrap_or_else(|| chapter.book.clone())
    }

    /// Verse indices covered by the copy selection (inclusive).
    fn selection_bounds(&self) -> (usize, usize) {
        let cur = self.selected_verse_idx();
        match self.visual_anchor {
            Some(a) => (a.min(cur), a.max(cur)),
            None => (cur, cur),
        }
    }

    /// Whether a verse index is inside the active visual range.
    fn in_visual_range(&self, idx: usize) -> bool {
        if self.visual_anchor.is_none() {
            return false;
        }
        let (start, end) = self.selection_bounds();
        idx >= start && idx <= end
    }

    /// Build the clipboard text and a short label for what gets copied.
    /// Verse-per-line copies the selected verse (or visual range);
    /// paragraph view copies the whole chapter.
    pub fn copy_payload(&self) -> Option<(String, String)> {
        let chapter = self.current_chapter.as_ref()?;
        if chapter.verses.is_empty() {
            return None;
        }
        let book = self.loaded_book_display_name(chapter);
        let trans = &chapter.translation;

        match self.view_mode {
            ViewMode::Paragraph => {
                let label = format!("{} {}", book, chapter.chapter);
                let mut text = format!("{} ({})\n", label, trans);
                for v in &chapter.verses {
                    text.push_str(&format!("{} {}\n", v.verse, v.text));
                }
                Some((text, label))
            }
            ViewMode::VersePerLine => {
                let (start, end) = self.selection_bounds();
                if start == end {
                    let v = &chapter.verses[start];
                    let label = format!("{} {}:{}", book, chapter.chapter, v.verse);
                    let text = format!("{} - {} ({})", label, v.text, trans);
                    Some((text, label))
                } else {
                    let first = &chapter.verses[start];
                    let last = &chapter.verses[end];
                    let label =
                        format!("{} {}:{}-{}", book, chapter.chapter, first.verse, last.verse);
                    let mut text = format!("{} ({})\n", label, trans);
                    for v in &chapter.verses[start..=end] {
                        text.push_str(&format!("{} {}\n", v.verse, v.text));
                    }
                    Some((text, label))
                }
            }
        }
    }

    /// Show a transient message in the status bar.
    pub fn flash(&mut self, message: impl Into<String>) {
        self.copy_flash = Some((message.into(), std::time::Instant::now()));
    }

    pub fn select_current(&mut self) -> bool {
        match self.active_panel {
            Panel::Books => {
                self.chapter_list.select(Some(0));
                self.active_panel = Panel::Chapters;
                false
            }
            Panel::Chapters => self.commit_chapter_selection(),
            Panel::Scripture => false,
        }
    }

    /// Get the selected search result.
    pub fn selected_search_result(&self) -> Option<&SearchResult> {
        if let SearchMode::Active { results, list_state, .. } = &self.search {
            let idx = list_state.selected()?;
            results.get(idx)
        } else {
            None
        }
    }

    /// Navigate to a book and chapter from a search result.
    pub fn jump_to_result(&mut self, book: &str, chapter: u32, verse: u32) {
        // Find the book index
        if let Some(idx) = BOOKS.iter().position(|b| b.name.eq_ignore_ascii_case(book)) {
            self.selected_book_idx = idx;
            self.book_list.select(Some(idx));
            self.selected_chapter = chapter;
            self.chapter_list.select(Some((chapter - 1) as usize));
            self.scripture_scroll = 0;
            self.active_panel = Panel::Scripture;
            self.search = SearchMode::Off;
            self.highlight_verse = Some(verse);
            // Approximate until the chapter loads; load_chapter re-resolves
            // by verse number (translations can have numbering gaps).
            self.select_verse_by_number(verse);
            self.visual_anchor = None;
            self.pending_paragraph_scroll = true;
        }
    }
}

pub fn render_browser(
    frame: &mut Frame,
    area: Rect,
    state: &mut BrowserState,
    quit_pending: bool,
    theme: &Theme,
    theme_name: ThemeName,
) {
    // Outer border
    let outer_block = Block::default()
        .title(Line::from(vec![
            Span::styled(" christ", Style::default().fg(theme.accent).bold()),
            Span::styled("-cli ", Style::default().fg(theme.text_dim)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Layout: main content + optional search bar + status bar
    let has_search_input = matches!(state.search, SearchMode::Active { .. });
    let main_and_status = if has_search_input {
        Layout::vertical([
            Constraint::Min(1),    // Main content
            Constraint::Length(3), // Search input
            Constraint::Length(1), // Status bar
        ])
        .split(inner)
    } else {
        Layout::vertical([
            Constraint::Min(1),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(inner)
    };

    // Three panels
    let panels = Layout::horizontal([
        Constraint::Percentage(22), // Books
        Constraint::Percentage(13), // Chapters
        Constraint::Percentage(65), // Scripture
    ])
    .split(main_and_status[0]);

    render_books_panel(frame, panels[0], state, theme);
    render_chapters_panel(frame, panels[1], state, theme);

    let translation = state.translation.clone();
    let dl = state.download_progress();
    let flash = state
        .copy_flash
        .as_ref()
        .map(|(msg, _)| msg.clone());

    if has_search_input {
        render_search_results_panel(frame, panels[2], state, theme);
        render_search_input(frame, main_and_status[1], state, theme);
        render_status_bar(frame, main_and_status[2], theme, theme_name, &translation, dl, flash.as_deref());
    } else {
        render_scripture_panel(frame, panels[2], state, theme);
        render_status_bar(frame, main_and_status[1], theme, theme_name, &translation, dl, flash.as_deref());
    }

    // Translation picker popup
    if state.translation_picker {
        render_translation_picker(frame, area, state, theme);
    }

    // Help overlay
    if state.help_open {
        render_help_popup(frame, area, state, theme);
    }

    // Quit confirmation popup
    if quit_pending {
        render_quit_popup(frame, area, theme);
    }
}

fn panel_border_style(active: bool, theme: &Theme) -> Style {
    if active {
        Style::default().fg(theme.border_active)
    } else {
        Style::default().fg(theme.border)
    }
}

fn render_books_panel(frame: &mut Frame, area: Rect, state: &mut BrowserState, theme: &Theme) {
    let is_active = state.active_panel == Panel::Books && matches!(state.search, SearchMode::Off);
    let block = Block::default()
        .title(Span::styled(
            " Books ",
            Style::default()
                .fg(if is_active { theme.accent } else { theme.text_dim })
                .bold(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(panel_border_style(is_active, theme))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(theme.surface));

    // Available width for book names: area - borders(2) - padding(2) - highlight_symbol(3)
    let max_name_width = (area.width as usize).saturating_sub(7);

    let items: Vec<ListItem> = BOOKS
        .iter()
        .enumerate()
        .map(|(i, _book)| {
            let style = if Some(i) == state.book_list.selected() {
                Style::default()
                    .fg(theme.accent)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let name = truncate_display_name(&state.book_display_name(i), max_name_width);
            ListItem::new(Span::styled(name, style))
        })
        .collect();

    let list = List::new(items).block(block).highlight_symbol(" > ");

    frame.render_stateful_widget(list, area, &mut state.book_list);
}

fn render_chapters_panel(frame: &mut Frame, area: Rect, state: &mut BrowserState, theme: &Theme) {
    let is_active = state.active_panel == Panel::Chapters && matches!(state.search, SearchMode::Off);
    let block = Block::default()
        .title(Span::styled(
            " Ch ",
            Style::default()
                .fg(if is_active { theme.accent } else { theme.text_dim })
                .bold(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(panel_border_style(is_active, theme))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(theme.surface));

    let chapter_count = state.selected_book_chapters();
    let items: Vec<ListItem> = (1..=chapter_count)
        .map(|ch| {
            let is_selected = Some(ch as usize - 1) == state.chapter_list.selected();
            let style = if is_selected {
                Style::default()
                    .fg(theme.accent)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(Span::styled(format!("{}", ch), style))
        })
        .collect();

    let list = List::new(items).block(block).highlight_symbol(" > ");

    frame.render_stateful_widget(list, area, &mut state.chapter_list);
}

fn render_scripture_panel(frame: &mut Frame, area: Rect, state: &mut BrowserState, theme: &Theme) {
    let is_active = state.active_panel == Panel::Scripture && matches!(state.search, SearchMode::Off);

    let title = if let Some(ref ch) = state.current_chapter {
        format!(" {} {} ", state.loaded_book_display_name(ch), ch.chapter)
    } else {
        " Scripture ".to_string()
    };

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default()
                .fg(if is_active { theme.accent } else { theme.text_dim })
                .bold(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(panel_border_style(is_active, theme))
        .padding(Padding::new(2, 2, 1, 1))
        .style(Style::default().bg(theme.surface));

    if state.loading {
        let loading = Paragraph::new(Line::from(Span::styled(
            "Loading...",
            Style::default().fg(theme.text_dim),
        )))
        .block(block)
        .alignment(Alignment::Center);
        frame.render_widget(loading, area);
        return;
    }

    if let Some(ref err) = state.error {
        let error_msg = Paragraph::new(vec![
            Line::default(),
            Line::from(Span::styled(
                format!("Error: {}", err),
                Style::default().fg(theme.search_match),
            )),
            Line::default(),
            Line::from(Span::styled(
                "Press Enter to retry",
                Style::default().fg(theme.text_dim),
            )),
        ])
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });
        frame.render_widget(error_msg, area);
        return;
    }

    if state.current_chapter.is_some() {
        match state.view_mode {
            ViewMode::VersePerLine => render_verse_list(frame, area, block, state, theme),
            ViewMode::Paragraph => render_paragraph_view(frame, area, block, state, theme),
        }
    } else {
        let hint = Paragraph::new(vec![
            Line::default(),
            Line::default(),
            Line::from(Span::styled(
                "Select a book and chapter to begin reading",
                Style::default().fg(theme.text_dim),
            )),
            Line::default(),
            Line::from(Span::styled(
                "Use arrow keys to navigate, Enter to select",
                Style::default().fg(theme.text_muted),
            )),
        ])
        .block(block)
        .alignment(Alignment::Center);
        frame.render_widget(hint, area);
    }
}

/// Verse-per-line view: a selectable list with one (wrapped) verse per item,
/// a cursor arrow like the Books/Chapters panels, and visual-range styling.
fn render_verse_list(frame: &mut Frame, area: Rect, block: Block, state: &mut BrowserState, theme: &Theme) {
    let inner = block.inner(area);

    // One-shot: coming back from paragraph view, land the cursor on the
    // verse at the previous reading position (scroll + a third of the
    // viewport, mirroring the offset pending_paragraph_scroll applies).
    if state.pending_cursor_sync {
        state.pending_cursor_sync = false;
        let target = state.scripture_scroll as usize + (inner.height as usize) / 3;
        let synced = state.current_chapter.as_ref().map(|chapter| {
            let mut tracker = wrap::RowTracker::new(inner.width as usize);
            let mut idx = 0;
            for (i, v) in chapter.verses.iter().enumerate() {
                if tracker.row() <= target {
                    idx = i;
                } else {
                    break;
                }
                tracker.push_text(&format!("{} {}", v.verse, v.text));
            }
            idx
        });
        if let Some(idx) = synced {
            state.verse_list.select(Some(idx));
        }
    }

    // Clamp the cursor: restored sessions can point past a shorter chapter.
    let selected = state.selected_verse_idx();
    state.verse_list.select(Some(selected));

    let highlight = state.highlight_verse;
    let arrow_w = 2usize; // "▸ "
    let text_w = (inner.width as usize).saturating_sub(arrow_w);

    let chapter = state.current_chapter.as_ref().expect("chapter checked by caller");
    let count = chapter.verses.len();
    let mut items: Vec<ListItem> = Vec::with_capacity(count);
    let mut item_heights: Vec<usize> = Vec::with_capacity(count);

    for (i, v) in chapter.verses.iter().enumerate() {
        let is_selected = i == selected;
        let in_range = state.in_visual_range(i);
        let is_highlighted = highlight == Some(v.verse);

        let num = format!("{} ", v.verse);
        let num_w = num.width();
        let body_w = text_w.saturating_sub(num_w).max(10);
        let wrapped = wrap::wrap_text(&v.text, body_w);

        let (num_style, text_style) = if is_highlighted {
            (
                Style::default().fg(theme.search_match).bold(),
                Style::default().fg(theme.search_match),
            )
        } else if is_selected {
            (
                Style::default().fg(theme.accent).bg(theme.highlight_bg).bold(),
                Style::default().fg(theme.text).bg(theme.highlight_bg),
            )
        } else if in_range {
            (
                Style::default().fg(theme.accent_soft).bg(theme.highlight_bg).bold(),
                Style::default().fg(theme.text).bg(theme.highlight_bg),
            )
        } else {
            (
                Style::default().fg(theme.text_muted),
                Style::default().fg(theme.text),
            )
        };

        let arrow = if is_selected {
            Span::styled("\u{25b8} ", Style::default().fg(theme.accent).bold())
        } else {
            Span::raw("  ")
        };

        let mut lines: Vec<Line> = Vec::with_capacity(wrapped.len() + 1);
        for (li, seg) in wrapped.iter().enumerate() {
            let lead = if li == 0 { arrow.clone() } else { Span::raw("  ") };
            let num_span = if li == 0 {
                Span::styled(num.clone(), num_style)
            } else {
                Span::styled(" ".repeat(num_w), num_style)
            };
            lines.push(Line::from(vec![lead, num_span, Span::styled(seg.clone(), text_style)]));
        }
        lines.push(Line::default());

        item_heights.push(lines.len());
        items.push(ListItem::new(ratatui::text::Text::from(lines)));
    }

    let list = List::new(items).block(block);
    frame.render_stateful_widget(list, area, &mut state.verse_list);

    // Scrollbar based on wrapped row counts.
    let total_rows: usize = item_heights.iter().sum();
    let visible = inner.height as usize;
    if total_rows > visible && visible > 0 {
        let offset = state.verse_list.offset().min(count);
        let rows_before: usize = item_heights[..offset].iter().sum();
        let max_scroll = total_rows - visible;
        let mut scrollbar_state =
            ScrollbarState::new(max_scroll).position(rows_before.min(max_scroll));
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(theme.border));
        frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }
}

/// Paragraph view: verses flow together as continuous text with inline
/// verse numbers, scrolled by line.
fn render_paragraph_view(frame: &mut Frame, area: Rect, block: Block, state: &mut BrowserState, theme: &Theme) {
    let inner = block.inner(area);
    let visible_height = inner.height;
    let wrap_width = (inner.width as usize).max(1);

    let chapter = state.current_chapter.as_ref().expect("chapter checked by caller");
    let highlight = state.highlight_verse;

    let mut spans: Vec<Span> = Vec::with_capacity(chapter.verses.len() * 3);
    for v in &chapter.verses {
        let is_highlighted = highlight == Some(v.verse);
        spans.push(Span::styled(
            format!("{} ", v.verse),
            if is_highlighted {
                Style::default().fg(theme.search_match).bold()
            } else {
                Style::default().fg(theme.text_muted)
            },
        ));
        spans.push(Span::styled(
            v.text.clone(),
            if is_highlighted {
                Style::default().fg(theme.search_match)
            } else {
                Style::default().fg(theme.text)
            },
        ));
        spans.push(Span::raw(" "));
    }

    // Row where each verse starts in the wrapped flow, plus total height,
    // using the same greedy wrap the renderer approximates.
    let (verse_rows, content_height) = {
        let mut tracker = wrap::RowTracker::new(wrap_width);
        let mut rows = Vec::with_capacity(chapter.verses.len());
        for v in &chapter.verses {
            rows.push(tracker.row() as u16);
            tracker.push_text(&format!("{} {}", v.verse, v.text));
        }
        (rows, tracker.total_rows() as u16)
    };

    // One-shot: carry the reading position (selected verse) into this view.
    if state.pending_paragraph_scroll {
        let idx = state.selected_verse_idx();
        let rows_before = verse_rows.get(idx).copied().unwrap_or(0);
        state.scripture_scroll = rows_before.saturating_sub(visible_height / 3);
    }
    state.pending_paragraph_scroll = false;

    // Clamp scroll
    if content_height > visible_height {
        let max_scroll = content_height - visible_height;
        if state.scripture_scroll > max_scroll {
            state.scripture_scroll = max_scroll;
        }
    } else {
        state.scripture_scroll = 0;
    }

    let paragraph = Paragraph::new(vec![Line::from(spans)])
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((state.scripture_scroll, 0));

    frame.render_widget(paragraph, area);

    if content_height > visible_height {
        let max_scroll = (content_height - visible_height) as usize;
        let mut scrollbar_state =
            ScrollbarState::new(max_scroll).position(state.scripture_scroll as usize);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(theme.border));
        frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }
}

/// Split `text` into styled spans, highlighting the first occurrence of
/// `query_lower` (case-insensitive, UTF-8 safe).
fn highlight_query_spans(
    text: &str,
    query_lower: &str,
    base: Style,
    matched: Style,
) -> Vec<Span<'static>> {
    let text_chars: Vec<char> = text.chars().collect();
    let query_chars: Vec<char> = query_lower.chars().collect();
    let text_lower_chars: Vec<char> = text.to_lowercase().chars().collect();

    if query_chars.is_empty() {
        return vec![Span::styled(text.to_string(), base)];
    }

    let match_pos = text_lower_chars
        .windows(query_chars.len())
        .position(|w| w == query_chars.as_slice())
        // Lowercasing may change char counts (e.g. İ); only use the index
        // when it maps back into the original text.
        .filter(|pos| pos + query_chars.len() <= text_chars.len());

    match match_pos {
        Some(pos) => {
            let before: String = text_chars[..pos].iter().collect();
            let hit: String = text_chars[pos..pos + query_chars.len()].iter().collect();
            let after: String = text_chars[pos + query_chars.len()..].iter().collect();
            vec![
                Span::styled(before, base),
                Span::styled(hit, matched),
                Span::styled(after, base),
            ]
        }
        None => vec![Span::styled(text.to_string(), base)],
    }
}

fn render_search_results_panel(
    frame: &mut Frame,
    area: Rect,
    state: &mut BrowserState,
    theme: &Theme,
) {
    let selected_result = state.selected_search_result().cloned();
    let (query, results, list_state) = match &mut state.search {
        SearchMode::Active { query, results, list_state } => (query.clone(), results, list_state),
        _ => return,
    };

    let title = format!(" Search: \"{}\" ({} results) ", query, results.len());

    let block = Block::default()
        .title(Span::styled(
            title,
            Style::default().fg(theme.accent).bold(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(theme.surface));

    if results.is_empty() {
        let msg = if query.len() < 3 {
            "Type at least 3 characters to search"
        } else {
            "No results found"
        };
        let empty = Paragraph::new(vec![
            Line::default(),
            Line::default(),
            Line::from(Span::styled(
                msg,
                Style::default().fg(theme.text_dim),
            )),
            Line::default(),
            Line::from(Span::styled(
                "Press Esc to go back",
                Style::default().fg(theme.text_muted),
            )),
        ])
        .block(block)
        .alignment(Alignment::Center);
        frame.render_widget(empty, area);
        return;
    }

    // Reserve the bottom of the panel for a full-text preview of the
    // selected result, sized to its wrapped height.
    let preview_height = selected_result.as_ref().map_or(0, |r| {
        let text_w = (area.width as usize).saturating_sub(4).max(10);
        (wrap::wrapped_height(&r.text, text_w) as u16 + 2).min(area.height / 2)
    });
    let chunks = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(preview_height),
    ])
    .split(area);

    let query_lower = query.to_lowercase();
    let match_style = Style::default()
        .fg(theme.search_match)
        .add_modifier(Modifier::BOLD);

    let items: Vec<ListItem> = results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let is_selected = Some(i) == list_state.selected();
            let ref_style = if is_selected {
                Style::default()
                    .fg(theme.accent)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.accent_soft).bold()
            };
            let text_style = if is_selected {
                Style::default().fg(theme.text).bg(theme.highlight_bg)
            } else {
                Style::default().fg(theme.text_dim)
            };

            let ref_str = format!("{} {}:{}", r.book, r.chapter, r.verse);
            // chrome = borders(2) + padding(2) + highlight_symbol(2)
            // + ref display width + 2-space gap. Display width (not codepoints)
            // matters for CJK book names that occupy two columns each.
            let chrome = 6 + UnicodeWidthStr::width(ref_str.as_str());
            let max_chars = (chunks[0].width as usize).saturating_sub(chrome).max(20);
            let text = truncate_result_text(&r.text, max_chars);

            let mut spans = vec![
                Span::styled(ref_str, ref_style),
                Span::styled("  ", text_style),
            ];
            spans.extend(highlight_query_spans(&text, &query_lower, text_style, match_style));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block).highlight_symbol("  ");
    frame.render_stateful_widget(list, chunks[0], list_state);

    // Full-text preview of the selected result (word-wrapped, no truncation).
    if let Some(r) = selected_result {
        if chunks[1].height >= 3 {
            let preview_title = format!(" {} {}:{} \u{00b7} {} ", r.book, r.chapter, r.verse, r.translation);
            let preview_block = Block::default()
                .title(Span::styled(
                    preview_title,
                    Style::default().fg(theme.accent).bold(),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .padding(Padding::horizontal(1))
                .style(Style::default().bg(theme.surface));

            let spans = highlight_query_spans(
                &r.text,
                &query_lower,
                Style::default().fg(theme.text),
                match_style,
            );
            let preview = Paragraph::new(Line::from(spans))
                .block(preview_block)
                .wrap(Wrap { trim: false });
            frame.render_widget(preview, chunks[1]);
        }
    }
}

fn render_search_input(frame: &mut Frame, area: Rect, state: &BrowserState, theme: &Theme) {
    let query = match &state.search {
        SearchMode::Active { query, .. } => query.as_str(),
        _ => return,
    };

    let block = Block::default()
        .title(Span::styled(" / Search ", Style::default().fg(theme.accent).bold()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(theme.surface));

    let cursor = "\u{2588}";
    let mut spans = vec![
        Span::styled(query, Style::default().fg(theme.text)),
        Span::styled(cursor, Style::default().fg(theme.accent_soft)),
    ];

    // Show hint for online translations
    if !state.is_offline() && query.is_empty() {
        spans.push(Span::styled(
            " Enter to search",
            Style::default().fg(theme.text_dim),
        ));
    }

    let input = Paragraph::new(Line::from(spans)).block(block);
    frame.render_widget(input, area);
}

fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    theme_name: ThemeName,
    translation: &str,
    download_progress: Option<(usize, usize)>,
    flash: Option<&str>,
) {
    // While a flash message is up it replaces the key hints entirely —
    // appended after ~115 columns of hints it would be clipped off-screen
    // on common terminal widths.
    let mut spans: Vec<Span> = if let Some(msg) = flash {
        vec![Span::styled(
            format!(" {} ", msg),
            Style::default().fg(theme.search_match).bold(),
        )]
    } else {
        let keybinds = vec![
            ("\u{2190}\u{2192}/hl", "panels"),
            ("\u{2191}\u{2193}/jk", "navigate"),
            ("/", "search"),
            ("y/Y", "copy"),
            ("p", "view"),
            ("t", theme_name.label()),
            ("v", translation),
            ("?", "help"),
            ("qq", "quit"),
        ];
        keybinds
            .iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(
                        format!(" {} ", key),
                        Style::default().fg(theme.accent_soft).bold(),
                    ),
                    Span::styled(
                        format!("{} ", desc),
                        Style::default().fg(theme.text_muted),
                    ),
                    Span::styled("  ", Style::default()),
                ]
            })
            .collect()
    };

    if let Some((completed, total)) = download_progress {
        let pct = if total > 0 {
            (completed * 100) / total
        } else {
            0
        };
        spans.push(Span::styled(
            format!(" Caching {}%", pct),
            Style::default().fg(theme.accent).bold(),
        ));
    }

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.bg));
    frame.render_widget(bar, area);
}

fn truncate_display_name(name: &str, max_width: usize) -> String {
    let w = name.width();
    if w <= max_width {
        return name.to_string();
    }
    // Truncate to fit within max_width, leaving room for ellipsis
    let target = max_width.saturating_sub(1); // 1 for ellipsis character
    let mut truncated = String::new();
    let mut current_w = 0;
    for ch in name.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_w + cw > target {
            break;
        }
        truncated.push(ch);
        current_w += cw;
    }
    truncated.push('\u{2026}');
    truncated
}

fn render_translation_picker(
    frame: &mut Frame,
    area: Rect,
    state: &mut BrowserState,
    theme: &Theme,
) {
    // Build display lines with language headers
    let mut lines: Vec<Line> = Vec::new();
    let mut last_lang = "";
    let mut selected_display_row: u16 = 0;

    for (i, t) in TRANSLATIONS.iter().enumerate() {
        if t.lang != last_lang {
            if !last_lang.is_empty() {
                lines.push(Line::default()); // blank separator between groups
            }
            lines.push(Line::from(Span::styled(
                format!("  {}", t.lang),
                Style::default().fg(theme.text_muted).add_modifier(Modifier::BOLD),
            )));
            last_lang = t.lang;
        }

        if Some(i) == state.translation_list.selected() {
            selected_display_row = lines.len() as u16;
        }

        let is_selected = Some(i) == state.translation_list.selected();
        let is_current = t.code.eq_ignore_ascii_case(&state.translation);
        let style = if is_selected {
            Style::default()
                .fg(theme.accent)
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD)
        } else if is_current {
            Style::default().fg(theme.accent_soft).bold()
        } else {
            Style::default().fg(theme.text)
        };

        let prefix = if is_selected { " \u{25b8} " } else { "   " };
        let suffix = if t.offline || cache::is_fully_cached(t.code) {
            " (offline)"
        } else if cache::has_cached_data(t.code) {
            " (cached)"
        } else {
            ""
        };
        let marker = if is_current { " \u{2713}" } else { "" };
        lines.push(Line::from(vec![
            Span::styled(prefix.to_string(), style),
            Span::styled(format!("{:<8}", t.code), style),
            Span::styled(t.name.to_string(), style),
            Span::styled(suffix.to_string(), Style::default().fg(theme.text_muted)),
            Span::styled(marker.to_string(), Style::default().fg(theme.search_match).bold()),
        ]));
    }

    let popup_width = 54u16;
    let popup_height = (lines.len() as u16 + 4).min(area.height.saturating_sub(4));

    let horizontal = Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .split(area);
    let vertical = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(horizontal[0]);
    let popup_area = vertical[0];

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(
            " Select Translation ",
            Style::default().fg(theme.accent).bold(),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(theme.surface));

    let inner_height = block.inner(popup_area).height;
    let scroll = if selected_display_row >= inner_height {
        selected_display_row.saturating_sub(inner_height / 2)
    } else {
        0
    };

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll, 0));

    frame.render_widget(paragraph, popup_area);
}

/// Full keybinding reference, opened with '?'.
fn render_help_popup(frame: &mut Frame, area: Rect, state: &mut BrowserState, theme: &Theme) {
    let key_style = Style::default().fg(theme.accent_soft).bold();
    let desc_style = Style::default().fg(theme.text);
    let section_style = Style::default().fg(theme.accent).bold();
    let dim_style = Style::default().fg(theme.text_muted);

    let key_line = |key: &str, desc: &str| {
        Line::from(vec![
            Span::styled(format!("  {:<13}", key), key_style),
            Span::styled(desc.to_string(), desc_style),
        ])
    };
    let section = |name: &str| Line::from(Span::styled(format!(" {}", name), section_style));
    let note = |text: &str| Line::from(Span::styled(format!("  {}", text), dim_style));

    let lines: Vec<Line> = vec![
        section("Navigation"),
        key_line("\u{2190}/\u{2192}  h/l", "switch panels (Books \u{00b7} Chapters \u{00b7} Scripture)"),
        key_line("\u{2191}/\u{2193}  j/k", "move in lists; verse cursor in Scripture"),
        key_line("Enter", "open book / load chapter"),
        Line::default(),
        section("Reading"),
        key_line("p", "toggle verse-per-line \u{21c4} paragraph view"),
        note("your position carries over between the two views"),
        Line::default(),
        section("Copy to clipboard"),
        key_line("y  or  c", "copy the selected verse"),
        key_line("Y  or  C", "start a verse range; extend with j/k, Y copies"),
        key_line("Esc", "cancel the range selection"),
        note("in paragraph view, y copies the whole chapter"),
        note("works over SSH too (OSC 52)"),
        Line::default(),
        section("Search"),
        key_line("/", "live search (type 3+ characters)"),
        key_line("\u{2191}/\u{2193}", "move through results; full text previews below"),
        key_line("Enter", "jump to the selected verse"),
        key_line("Esc", "close search"),
        Line::default(),
        section("Settings"),
        key_line("t", "cycle themes"),
        key_line("v", "choose translation (Enter applies)"),
        Line::default(),
        section("Other"),
        key_line("?", "toggle this help"),
        key_line("q q", "quit (press q twice)"),
        Line::default(),
        section("CLI"),
        note("christ read \"John 3:16\"  \u{00b7}  \"Jo\u{e3}o 3.16\"  \u{00b7}  \"1. Mose 3,16\""),
        note("christ search \"...\"  \u{00b7}  christ random  \u{00b7}  christ --help"),
    ];

    let popup_width = 68u16;
    let content_height = lines.len() as u16;
    let popup_height = (content_height + 4).min(area.height.saturating_sub(2));

    let horizontal = Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .split(area);
    let vertical = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(horizontal[0]);
    let popup_area = vertical[0];

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(" Help ", Style::default().fg(theme.accent).bold()))
        .title_bottom(Span::styled(
            " Esc/? to close ",
            Style::default().fg(theme.text_muted),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .padding(Padding::new(1, 1, 1, 1))
        .style(Style::default().bg(theme.surface));

    let inner_height = block.inner(popup_area).height;
    let max_scroll = content_height.saturating_sub(inner_height);
    if state.help_scroll > max_scroll {
        state.help_scroll = max_scroll;
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((state.help_scroll, 0));

    frame.render_widget(paragraph, popup_area);
}

fn render_quit_popup(frame: &mut Frame, area: Rect, theme: &Theme) {
    let popup_width = 32u16;
    let popup_height = 3u16;

    let horizontal = Layout::horizontal([Constraint::Length(popup_width)])
        .flex(Flex::Center)
        .split(area);
    let vertical = Layout::vertical([Constraint::Length(popup_height)])
        .flex(Flex::Center)
        .split(horizontal[0]);
    let popup_area = vertical[0];

    frame.render_widget(Clear, popup_area);

    let popup = Paragraph::new(Line::from(vec![
        Span::styled("  Press ", Style::default().fg(theme.text_dim)),
        Span::styled("q", Style::default().fg(theme.accent).bold()),
        Span::styled(" again to quit  ", Style::default().fg(theme.text_dim)),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.border_active))
            .style(Style::default().bg(theme.surface)),
    );

    frame.render_widget(popup, popup_area);
}

fn truncate_result_text(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::types::{Chapter, Verse};

    fn state_at(book_idx: usize, chapter: u32, scroll: u16) -> BrowserState {
        let mut s = BrowserState::new();
        s.selected_book_idx = book_idx;
        s.book_list.select(Some(book_idx));
        s.selected_chapter = chapter;
        s.chapter_list.select(Some((chapter - 1) as usize));
        s.scripture_scroll = scroll;
        s.current_chapter = Some(Chapter {
            book: BOOKS[book_idx].name.to_string(),
            chapter,
            verses: (1..=3)
                .map(|n| Verse {
                    book: BOOKS[book_idx].name.to_string(),
                    chapter,
                    verse: n,
                    text: format!("stub {}", n),
                    translation: "KJV".to_string(),
                })
                .collect(),
            translation: "KJV".to_string(),
        });
        s.active_panel = Panel::Scripture;
        s
    }

    #[test]
    fn panel_hop_back_to_scripture_preserves_scroll() {
        let mut s = state_at(0, 1, 42);
        s.prev_panel(); // Scripture -> Chapters
        let needs_reload = s.next_panel_or_select(); // Chapters -> Scripture, same chapter
        assert!(!needs_reload, "no reload when chapter unchanged");
        assert_eq!(s.scripture_scroll, 42, "scroll preserved");
    }

    #[test]
    fn select_same_chapter_via_enter_preserves_scroll() {
        let mut s = state_at(0, 1, 42);
        s.prev_panel();
        let needs_reload = s.select_current(); // Enter on same chapter
        assert!(!needs_reload);
        assert_eq!(s.scripture_scroll, 42);
    }

    #[test]
    fn switching_to_different_chapter_resets_scroll() {
        let mut s = state_at(0, 1, 42);
        s.prev_panel();
        s.chapter_list.select(Some(2)); // pick chapter 3
        let needs_reload = s.next_panel_or_select();
        assert!(needs_reload, "must reload for new chapter");
        assert_eq!(s.scripture_scroll, 0, "scroll resets on chapter change");
        assert_eq!(s.selected_chapter, 3);
    }

    #[test]
    fn switching_to_different_book_resets_scroll() {
        let mut s = state_at(0, 1, 42);
        // Go back to Books, change book, descend through Chapters to Scripture.
        s.prev_panel();
        s.prev_panel();
        s.selected_book_idx = 1;
        s.book_list.select(Some(1));
        s.next_panel_or_select(); // Books -> Chapters (selects ch 1)
        let needs_reload = s.next_panel_or_select(); // Chapters -> Scripture
        assert!(needs_reload, "must reload when book changed");
        assert_eq!(s.scripture_scroll, 0);
        assert_eq!(s.selected_book_name(), BOOKS[1].name);
    }

    #[test]
    fn portuguese_translations_include_naa() {
        let codes: Vec<&str> = TRANSLATIONS
            .iter()
            .filter(|t| t.lang == "Português")
            .map(|t| t.code)
            .collect();
        assert!(codes.contains(&"NAA"), "NAA must be in the picker");
        assert!(codes.contains(&"ARA"), "ARA preserved");
        assert!(codes.contains(&"ACF11"));
    }

    #[test]
    fn ara_display_name_includes_year() {
        let ara = TRANSLATIONS
            .iter()
            .find(|t| t.code == "ARA")
            .expect("ARA exists");
        assert!(
            ara.name.contains("1993"),
            "ARA must show its year so it's not confused with other Almeidas"
        );
    }

    #[test]
    fn verse_cursor_moves_in_verse_per_line_mode() {
        let mut s = state_at(0, 1, 0);
        assert_eq!(s.selected_verse_idx(), 0);
        s.move_down();
        assert_eq!(s.selected_verse_idx(), 1);
        s.move_down();
        s.move_down(); // clamped at the last verse
        assert_eq!(s.selected_verse_idx(), 2);
        s.move_up();
        assert_eq!(s.selected_verse_idx(), 1);
        assert_eq!(s.scripture_scroll, 0, "cursor moves must not scroll");
    }

    #[test]
    fn paragraph_mode_scrolls_lines_not_cursor() {
        let mut s = state_at(0, 1, 0);
        s.view_mode = ViewMode::Paragraph;
        s.move_down();
        assert_eq!(s.scripture_scroll, 1);
        assert_eq!(s.selected_verse_idx(), 0, "cursor untouched in paragraph mode");
    }

    #[test]
    fn toggle_view_mode_round_trips() {
        let mut s = state_at(0, 1, 0);
        assert_eq!(s.view_mode, ViewMode::VersePerLine);
        s.toggle_view_mode();
        assert_eq!(s.view_mode, ViewMode::Paragraph);
        assert!(s.pending_paragraph_scroll, "carries reading position over");
        s.toggle_view_mode();
        assert_eq!(s.view_mode, ViewMode::VersePerLine);
    }

    #[test]
    fn copy_payload_single_verse() {
        let s = state_at(0, 1, 0);
        let (text, label) = s.copy_payload().unwrap();
        assert_eq!(label, "Genesis 1:1");
        assert_eq!(text, "Genesis 1:1 - stub 1 (KJV)");
    }

    #[test]
    fn copy_payload_visual_range() {
        let mut s = state_at(0, 1, 0);
        s.visual_anchor = Some(0);
        s.verse_list.select(Some(2));
        let (text, label) = s.copy_payload().unwrap();
        assert_eq!(label, "Genesis 1:1-3");
        assert!(text.starts_with("Genesis 1:1-3 (KJV)\n"));
        assert!(text.contains("\n2 stub 2\n"));
        assert!(text.contains("\n3 stub 3\n"));
    }

    #[test]
    fn copy_payload_range_works_backwards() {
        let mut s = state_at(0, 1, 0);
        s.visual_anchor = Some(2);
        s.verse_list.select(Some(0));
        let (_, label) = s.copy_payload().unwrap();
        assert_eq!(label, "Genesis 1:1-3", "anchor below cursor still copies forward");
    }

    #[test]
    fn copy_payload_paragraph_copies_whole_chapter() {
        let mut s = state_at(0, 1, 0);
        s.view_mode = ViewMode::Paragraph;
        let (text, label) = s.copy_payload().unwrap();
        assert_eq!(label, "Genesis 1");
        assert!(text.starts_with("Genesis 1 (KJV)\n"));
        assert!(text.contains("3 stub 3"));
    }

    #[test]
    fn jump_to_result_selects_verse() {
        let mut s = state_at(0, 1, 0);
        s.jump_to_result("Exodus", 2, 3);
        assert_eq!(s.selected_book_name(), "Exodus");
        assert_eq!(s.selected_chapter, 2);
        assert_eq!(s.highlight_verse, Some(3));
        assert_eq!(s.verse_list.selected(), Some(2), "cursor lands on the verse");
    }

    #[test]
    fn new_chapter_resets_verse_cursor_and_visual_range() {
        let mut s = state_at(0, 1, 0);
        s.verse_list.select(Some(2));
        s.visual_anchor = Some(0);
        s.prev_panel();
        s.chapter_list.select(Some(4)); // chapter 5
        assert!(s.next_panel_or_select());
        assert_eq!(s.verse_list.selected(), Some(0));
        assert!(s.visual_anchor.is_none());
        assert!(s.highlight_verse.is_none());
    }

    #[test]
    fn copy_label_uses_loaded_chapter_not_books_cursor() {
        // Browsing the Books panel moves selected_book_idx without loading
        // anything; the copied citation must name the LOADED book.
        let mut s = state_at(0, 1, 0); // Genesis 1 loaded
        s.prev_panel();
        s.prev_panel(); // to Books panel
        s.move_down(); // cursor on Exodus, nothing loaded
        assert_eq!(s.selected_book_idx, 1);
        let (text, label) = s.copy_payload().unwrap();
        assert_eq!(label, "Genesis 1:1");
        assert!(text.starts_with("Genesis 1:1"));
    }

    #[test]
    fn select_verse_by_number_handles_numbering_gaps() {
        // Some translations omit verses (NIV drops Mark 9:44), so
        // chapter.verses is not densely numbered.
        let mut s = state_at(0, 1, 0);
        if let Some(ch) = s.current_chapter.as_mut() {
            ch.verses[0].verse = 1;
            ch.verses[1].verse = 2;
            ch.verses[2].verse = 5; // gap: 3 and 4 omitted
        }
        s.select_verse_by_number(5);
        assert_eq!(s.verse_list.selected(), Some(2), "found by number, not index");
        s.select_verse_by_number(2);
        assert_eq!(s.verse_list.selected(), Some(1));
    }

    #[test]
    fn toggle_from_paragraph_requests_cursor_sync() {
        let mut s = state_at(0, 1, 0);
        s.view_mode = ViewMode::Paragraph;
        s.toggle_view_mode();
        assert_eq!(s.view_mode, ViewMode::VersePerLine);
        assert!(s.pending_cursor_sync, "cursor must be derived from paragraph scroll");
    }

    #[test]
    fn browsing_books_or_chapters_arms_live_preview() {
        let mut s = state_at(0, 1, 0);

        // Moving the verse cursor in Scripture must NOT trigger a preview.
        s.move_down();
        assert!(s.preview_pending.is_none());

        // Browsing chapters previews the highlighted chapter.
        s.prev_panel();
        s.move_down();
        assert!(s.preview_pending.is_some());
        assert_eq!(s.preview_target(), (0, 2));

        // Browsing books previews chapter 1 of the highlighted book.
        s.preview_pending = None;
        s.prev_panel();
        s.move_down();
        assert!(s.preview_pending.is_some());
        assert_eq!(s.preview_target(), (1, 1));
    }

    #[test]
    fn moving_against_a_list_edge_does_not_arm_preview() {
        let mut s = state_at(0, 1, 0);
        s.prev_panel();
        s.prev_panel(); // Books, cursor on Genesis (top)
        s.move_up(); // no-op at the edge
        assert!(s.preview_pending.is_none());
    }
}
