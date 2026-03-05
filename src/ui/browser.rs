use crate::api::types::{Chapter, SearchResult};
use crate::data::books::BOOKS;
use crate::ui::theme::{Theme, ThemeName};
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
    /// Not searching
    Off,
    /// Typing in the search input
    Input(String),
    /// Viewing search results
    Results {
        query: String,
        results: Vec<SearchResult>,
        list_state: ListState,
    },
}

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
            ..Default::default()
        }
    }

    pub fn selected_book_name(&self) -> &'static str {
        BOOKS[self.selected_book_idx].name
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

    /// Get the selected search result (if in results mode).
    pub fn selected_search_result(&self) -> Option<&SearchResult> {
        if let SearchMode::Results { results, list_state, .. } = &self.search {
            let idx = list_state.selected()?;
            results.get(idx)
        } else {
            None
        }
    }

    /// Navigate to a book and chapter from a search result.
    pub fn jump_to_result(&mut self, book: &str, chapter: u32) {
        // Find the book index
        if let Some(idx) = BOOKS.iter().position(|b| b.name.eq_ignore_ascii_case(book)) {
            self.selected_book_idx = idx;
            self.book_list.select(Some(idx));
            self.selected_chapter = chapter;
            self.chapter_list.select(Some((chapter - 1) as usize));
            self.scripture_scroll = 0;
            self.active_panel = Panel::Scripture;
            self.search = SearchMode::Off;
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
    let has_search_input = matches!(state.search, SearchMode::Input(_));
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

    // Scripture panel shows search results when in results mode
    match &mut state.search {
        SearchMode::Results { .. } => {
            render_search_results_panel(frame, panels[2], state, theme);
        }
        _ => {
            render_scripture_panel(frame, panels[2], state, theme);
        }
    }

    if has_search_input {
        render_search_input(frame, main_and_status[1], state, theme);
        render_status_bar(frame, main_and_status[2], theme, theme_name);
    } else {
        render_status_bar(frame, main_and_status[1], theme, theme_name);
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

    let items: Vec<ListItem> = BOOKS
        .iter()
        .enumerate()
        .map(|(i, book)| {
            let style = if Some(i) == state.book_list.selected() {
                Style::default()
                    .fg(theme.accent)
                    .bg(theme.highlight_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(Span::styled(book.name, style))
        })
        .collect();

    let list = List::new(items).block(block).highlight_symbol("  ");

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

    let list = List::new(items).block(block).highlight_symbol("  ");

    frame.render_stateful_widget(list, area, &mut state.chapter_list);
}

fn render_scripture_panel(frame: &mut Frame, area: Rect, state: &mut BrowserState, theme: &Theme) {
    let is_active = state.active_panel == Panel::Scripture && matches!(state.search, SearchMode::Off);

    let title = if let Some(ref ch) = state.current_chapter {
        format!(" {} {} ", ch.book, ch.chapter)
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

    if let Some(ref chapter) = state.current_chapter {
        let lines: Vec<Line> = chapter
            .verses
            .iter()
            .flat_map(|v| {
                let verse_line = Line::from(vec![
                    Span::styled(
                        format!(" {} ", v.verse),
                        Style::default().fg(theme.text_muted),
                    ),
                    Span::styled(&v.text, Style::default().fg(theme.text)),
                ]);
                vec![verse_line, Line::default()]
            })
            .collect();

        let inner = block.inner(area);
        let visible_height = inner.height;
        let wrap_width = inner.width as usize;

        // Calculate actual wrapped content height
        let content_height: u16 = lines
            .iter()
            .map(|line| {
                if line.spans.is_empty() {
                    return 1; // empty line
                }
                let line_width: usize = line.spans.iter().map(|s| s.content.len()).sum();
                if wrap_width == 0 {
                    1
                } else {
                    ((line_width as f64 / wrap_width as f64).ceil() as u16).max(1)
                }
            })
            .sum();

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
        SearchMode::Results { query, results, list_state } => (query.clone(), results, list_state),
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
        let empty = Paragraph::new(vec![
            Line::default(),
            Line::default(),
            Line::from(Span::styled(
                "No results found",
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

            // Simple highlight: split on query match
            let text_lower = text.to_lowercase();
            if let Some(pos) = text_lower.find(&query_lower) {
                let before = &text[..pos];
                let matched = &text[pos..pos + query_lower.len()];
                let after = &text[pos + query_lower.len()..];
                spans.push(Span::styled(before.to_string(), text_style));
                spans.push(Span::styled(
                    matched.to_string(),
                    Style::default().fg(theme.search_match).add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(after.to_string(), text_style));
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
    let input_text = match &state.search {
        SearchMode::Input(text) => text.as_str(),
        _ => "",
    };

    let block = Block::default()
        .title(Span::styled(" Search ", Style::default().fg(theme.accent).bold()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border_active))
        .padding(Padding::horizontal(1))
        .style(Style::default().bg(theme.surface));

    let cursor = "\u{2588}"; // block cursor
    let input = Paragraph::new(Line::from(vec![
        Span::styled(input_text, Style::default().fg(theme.text)),
        Span::styled(cursor, Style::default().fg(theme.accent_soft)),
    ]))
    .block(block);

    frame.render_widget(input, area);
}

fn render_status_bar(frame: &mut Frame, area: Rect, theme: &Theme, theme_name: ThemeName) {
    let keybinds = vec![
        ("\u{2190}\u{2192}", "panels"),
        ("\u{2191}\u{2193}", "navigate"),
        ("Enter", "select"),
        ("/", "search"),
        ("t", theme_name.label()),
        ("qq", "quit"),
    ];

    let spans: Vec<Span> = keybinds
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

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.bg));
    frame.render_widget(bar, area);
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

fn truncate_result_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}
