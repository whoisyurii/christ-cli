use crate::api::types::{Chapter, SearchResult};
use crate::data::books::BOOKS;
use crate::store::cache;
use crate::ui::theme::{Theme, ThemeName};
use std::sync::atomic::Ordering;
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
    TranslationInfo { code: "ARA", name: "Almeida Revista e Atualizada", lang: "Português", offline: false },
    TranslationInfo { code: "NVIPT", name: "NVI Português", lang: "Português", offline: false },
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
}

impl BrowserState {
    pub fn new() -> Self {
        let mut book_list = ListState::default();
        book_list.select(Some(0));
        let mut chapter_list = ListState::default();
        chapter_list.select(Some(0));

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
            Panel::Chapters => {
                let ch = self.chapter_list.selected().unwrap_or(0) as u32 + 1;
                self.selected_chapter = ch;
                self.scripture_scroll = 0;
                self.active_panel = Panel::Scripture;
                true // Signal to load chapter
            }
            Panel::Scripture => false, // Already rightmost
        }
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
                }
            }
            Panel::Chapters => {
                let i = self.chapter_list.selected().unwrap_or(0);
                if i > 0 {
                    self.chapter_list.select(Some(i - 1));
                }
            }
            Panel::Scripture => {
                self.highlight_verse = None;
                if self.scripture_scroll > 0 {
                    self.scripture_scroll -= 1;
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
                }
            }
            Panel::Chapters => {
                let i = self.chapter_list.selected().unwrap_or(0);
                let max = self.selected_book_chapters() as usize;
                if i < max - 1 {
                    self.chapter_list.select(Some(i + 1));
                }
            }
            Panel::Scripture => {
                self.highlight_verse = None;
                self.scripture_scroll += 1;
            }
        }
    }

    pub fn select_current(&mut self) -> bool {
        match self.active_panel {
            Panel::Books => {
                self.chapter_list.select(Some(0));
                self.active_panel = Panel::Chapters;
                false
            }
            Panel::Chapters => {
                let ch = self.chapter_list.selected().unwrap_or(0) as u32 + 1;
                self.selected_chapter = ch;
                self.scripture_scroll = 0;
                self.active_panel = Panel::Scripture;
                true
            }
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

    if has_search_input {
        render_search_results_panel(frame, panels[2], state, theme);
        render_search_input(frame, main_and_status[1], state, theme);
        render_status_bar(frame, main_and_status[2], theme, theme_name, &translation, dl);
    } else {
        render_scripture_panel(frame, panels[2], state, theme);
        render_status_bar(frame, main_and_status[1], theme, theme_name, &translation, dl);
    }

    // Translation picker popup
    if state.translation_picker {
        render_translation_picker(frame, area, state, theme);
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

    let title = if state.current_chapter.is_some() {
        let book_name = state.book_display_name(state.selected_book_idx);
        format!(" {} {} ", book_name, state.selected_chapter)
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

    if let Some(ref chapter) = state.current_chapter {
        let highlight = state.highlight_verse;
        let lines: Vec<Line> = chapter
            .verses
            .iter()
            .flat_map(|v| {
                let is_highlighted = highlight == Some(v.verse);
                let verse_line = Line::from(vec![
                    Span::styled(
                        format!(" {} ", v.verse),
                        if is_highlighted {
                            Style::default().fg(theme.search_match)
                        } else {
                            Style::default().fg(theme.text_muted)
                        },
                    ),
                    Span::styled(
                        &v.text,
                        if is_highlighted {
                            Style::default().fg(theme.search_match)
                        } else {
                            Style::default().fg(theme.text)
                        },
                    ),
                ]);
                vec![verse_line, Line::default()]
            })
            .collect();

        let inner = block.inner(area);
        let visible_height = inner.height;
        let wrap_width = inner.width as usize;

        // Calculate wrapped height per line (for scroll targeting)
        let line_heights: Vec<u16> = lines
            .iter()
            .map(|line| {
                if line.spans.is_empty() {
                    return 1;
                }
                let line_width: usize = line.spans.iter().map(|s| s.content.len()).sum();
                if wrap_width == 0 {
                    1
                } else {
                    ((line_width as f64 / wrap_width as f64).ceil() as u16).max(1)
                }
            })
            .collect();

        let content_height: u16 = line_heights.iter().sum();

        // Auto-scroll to highlighted verse
        if let Some(target_verse) = highlight {
            // Each verse produces 2 lines (verse + blank), target is at index (verse-1)*2
            let target_line_idx = (target_verse.saturating_sub(1) as usize) * 2;
            let scroll_to: u16 = line_heights.iter().take(target_line_idx).sum();
            // Center the verse on screen
            let center_offset = visible_height / 3;
            state.scripture_scroll = scroll_to.saturating_sub(center_offset);
        }

        // Clamp scroll
        if content_height > visible_height {
            let max_scroll = content_height - visible_height;
            if state.scripture_scroll > max_scroll {
                state.scripture_scroll = max_scroll;
            }
        } else {
            state.scripture_scroll = 0;
        }

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((state.scripture_scroll, 0));

        frame.render_widget(paragraph, area);

        // Scrollbar
        if content_height > visible_height {
            let max_scroll = (content_height - visible_height) as usize;
            let mut scrollbar_state = ScrollbarState::new(max_scroll)
                .position(state.scripture_scroll as usize);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme.border));
            frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
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

fn render_search_results_panel(
    frame: &mut Frame,
    area: Rect,
    state: &mut BrowserState,
    theme: &Theme,
) {
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

    let query_lower = query.to_lowercase();
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

            // Highlight matching text
            let ref_str = format!("{} {}:{}", r.book, r.chapter, r.verse);
            let text = truncate_result_text(&r.text, 60);

            let mut spans = vec![
                Span::styled(ref_str, ref_style),
                Span::styled("  ", text_style),
            ];

            // Simple highlight: find match by char index for UTF-8 safety
            let text_chars: Vec<char> = text.chars().collect();
            let query_chars: Vec<char> = query_lower.chars().collect();
            let text_lower_chars: Vec<char> = text.to_lowercase().chars().collect();

            let match_pos = text_lower_chars
                .windows(query_chars.len())
                .position(|w| w == query_chars.as_slice());

            if let Some(pos) = match_pos {
                let before: String = text_chars[..pos].iter().collect();
                let matched: String = text_chars[pos..pos + query_chars.len()].iter().collect();
                let after: String = text_chars[pos + query_chars.len()..].iter().collect();
                spans.push(Span::styled(before, text_style));
                spans.push(Span::styled(
                    matched,
                    Style::default().fg(theme.search_match).add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(after, text_style));
            } else {
                spans.push(Span::styled(text, text_style));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block).highlight_symbol("  ");
    frame.render_stateful_widget(list, area, list_state);
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
) {
    let keybinds = vec![
        ("\u{2190}\u{2192}/hl", "panels"),
        ("\u{2191}\u{2193}/jk", "navigate"),
        ("Enter", "select"),
        ("/", "search"),
        ("t", theme_name.label()),
        ("v", translation),
        ("qq", "quit"),
    ];

    let mut spans: Vec<Span> = keybinds
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
        .collect();

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
    use unicode_width::UnicodeWidthStr;
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
        let suffix = if t.offline { " (offline)" } else { "" };
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
