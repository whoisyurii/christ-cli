use crate::api::Resolver;
use crate::data::kjv;
use crate::store::state as session;
use crate::ui::banner::{self, BannerState};
use crate::ui::browser::{self, BrowserState, SearchMode, TRANSLATIONS};
use crate::ui::theme::{self, ThemeName};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::widgets::ListState;
use ratatui::{DefaultTerminal, Frame};
use std::time::{Duration, Instant};

enum AppMode {
    Banner(BannerState),
    Browser(BrowserState),
}

pub struct App {
    mode: AppMode,
    resolver: Resolver,
    should_quit: bool,
    quit_pending: Option<Instant>,
    theme_name: ThemeName,
}

impl App {
    pub fn new(show_banner: bool) -> Self {
        let mode = if show_banner {
            AppMode::Banner(BannerState::new())
        } else {
            AppMode::Browser(BrowserState::new())
        };

        Self {
            mode,
            resolver: Resolver::new(),
            should_quit: false,
            quit_pending: None,
            theme_name: ThemeName::default(),
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> std::io::Result<()> {
        // Load saved session state
        let saved = session::load();
        let has_saved_session = saved.book_index > 0 || saved.chapter > 1;
        self.theme_name = saved.theme;

        // Load the chapter from the saved session (or Genesis 1 for first run)
        let book_name = if has_saved_session {
            crate::data::books::BOOKS
                .get(saved.book_index)
                .map(|b| b.name)
                .unwrap_or("Genesis")
        } else {
            "Genesis"
        };
        let chapter_num = if has_saved_session {
            saved.chapter.max(1)
        } else {
            1
        };

        let translation = &saved.translation;
        let initial_chapter = self
            .resolver
            .get_chapter(book_name, chapter_num, translation)
            .await
            .ok();

        // Fetch localized book names for non-KJV translations
        let localized_books = if !translation.eq_ignore_ascii_case("KJV") {
            self.resolver.get_book_names(translation).await.unwrap_or_default()
        } else {
            Vec::new()
        };

        match &mut self.mode {
            AppMode::Browser(ref mut state) => {
                if has_saved_session {
                    state.restore(&saved);
                }
                state.current_chapter = initial_chapter.clone();
                state.localized_books = localized_books.clone();
            }
            _ => {}
        }

        let mut pending_initial = Some((initial_chapter, saved, localized_books));

        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;

            let tick_rate = match &self.mode {
                AppMode::Banner(_) => Duration::from_millis(16),
                AppMode::Browser(_) => Duration::from_millis(50),
            };

            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code).await;
                    }
                }
            } else {
                if let AppMode::Banner(ref mut state) = self.mode {
                    state.tick();
                    if state.done {
                        let mut browser = BrowserState::new();
                        if let Some((ch, ref saved, ref books)) = pending_initial {
                            let has_saved = saved.book_index > 0 || saved.chapter > 1;
                            if has_saved {
                                browser.restore(saved);
                            }
                            browser.current_chapter = ch;
                            browser.localized_books = books.clone();
                        }
                        pending_initial = None;
                        self.mode = AppMode::Browser(browser);
                    }
                }
            }
        }

        // Save session state on quit
        if let AppMode::Browser(ref state) = self.mode {
            let mut snapshot = state.snapshot();
            snapshot.theme = self.theme_name;
            session::save(&snapshot);
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let theme = theme::get_theme(self.theme_name);

        if let Some(t) = self.quit_pending {
            if t.elapsed() > Duration::from_secs(2) {
                self.quit_pending = None;
            }
        }

        match &mut self.mode {
            AppMode::Banner(state) => {
                banner::render_banner(frame, area, state, &theme);
            }
            AppMode::Browser(state) => {
                browser::render_browser(
                    frame,
                    area,
                    state,
                    self.quit_pending.is_some(),
                    &theme,
                    self.theme_name,
                );
            }
        }
    }

    async fn handle_key(&mut self, key: KeyCode) {
        match &mut self.mode {
            AppMode::Banner(state) => {
                state.done = true;
            }
            AppMode::Browser(state) => {
                // Search mode
                if matches!(state.search, SearchMode::Active { .. }) {
                    match key {
                        KeyCode::Esc => {
                            state.search = SearchMode::Off;
                        }
                        KeyCode::Backspace => {
                            if let SearchMode::Active { query, .. } = &mut state.search {
                                query.pop();
                            }
                            self.live_search();
                        }
                        KeyCode::Char(c) => {
                            if let SearchMode::Active { query, .. } = &mut state.search {
                                query.push(c);
                            }
                            self.live_search();
                        }
                        KeyCode::Up => {
                            if let SearchMode::Active { list_state, .. } = &mut state.search {
                                let i = list_state.selected().unwrap_or(0);
                                if i > 0 {
                                    list_state.select(Some(i - 1));
                                }
                            }
                        }
                        KeyCode::Down => {
                            if let SearchMode::Active {
                                results,
                                list_state,
                                ..
                            } = &mut state.search
                            {
                                let i = list_state.selected().unwrap_or(0);
                                if i < results.len().saturating_sub(1) {
                                    list_state.select(Some(i + 1));
                                }
                            }
                        }
                        KeyCode::Enter => {
                            let target =
                                state.selected_search_result().map(|r| (r.book.clone(), r.chapter));
                            if let Some((book, chapter)) = target {
                                state.jump_to_result(&book, chapter);
                                self.load_chapter().await;
                            }
                        }
                        _ => {}
                    }
                    return;
                }

                // Translation picker mode
                if state.translation_picker {
                    match key {
                        KeyCode::Esc | KeyCode::Char('v') => {
                            state.translation_picker = false;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let i = state.translation_list.selected().unwrap_or(0);
                            if i > 0 {
                                state.translation_list.select(Some(i - 1));
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let i = state.translation_list.selected().unwrap_or(0);
                            if i < TRANSLATIONS.len() - 1 {
                                state.translation_list.select(Some(i + 1));
                            }
                        }
                        KeyCode::Enter => {
                            let changed = state.pick_translation();
                            if changed {
                                self.load_book_names().await;
                                self.load_chapter().await;
                            }
                        }
                        _ => {}
                    }
                    return;
                }

                // Normal browser mode
                if key == KeyCode::Char('q') {
                    if self.quit_pending.is_some() {
                        self.should_quit = true;
                    } else {
                        self.quit_pending = Some(Instant::now());
                    }
                    return;
                }
                self.quit_pending = None;

                match key {
                    KeyCode::Char('/') => {
                        state.search = SearchMode::Active {
                            query: String::new(),
                            results: vec![],
                            list_state: ListState::default(),
                        };
                    }
                    KeyCode::Char('t') => {
                        self.theme_name = self.theme_name.next();
                    }
                    KeyCode::Char('v') => {
                        state.open_translation_picker();
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        state.prev_panel();
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        let should_load = state.next_panel_or_select();
                        if should_load {
                            self.load_chapter().await;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.move_up();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        state.move_down();
                    }
                    KeyCode::Enter => {
                        let should_load = state.select_current();
                        if should_load {
                            self.load_chapter().await;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// Run search instantly using bundled KJV data and update results in-place.
    fn live_search(&mut self) {
        if let AppMode::Browser(ref mut state) = self.mode {
            if let SearchMode::Active {
                query,
                results,
                list_state,
            } = &mut state.search
            {
                if query.len() >= 3 {
                    *results = kjv::search(query);
                    if !results.is_empty() {
                        list_state.select(Some(0));
                    } else {
                        list_state.select(None);
                    }
                } else {
                    results.clear();
                    list_state.select(None);
                }
            }
        }
    }

    async fn load_book_names(&mut self) {
        if let AppMode::Browser(ref mut state) = self.mode {
            let translation = state.translation.clone();
            if translation.eq_ignore_ascii_case("KJV") {
                state.localized_books = Vec::new(); // Use English names
            } else {
                match self.resolver.get_book_names(&translation).await {
                    Ok(names) => state.localized_books = names,
                    Err(_) => state.localized_books = Vec::new(), // Fallback to English
                }
            }
        }
    }

    async fn load_chapter(&mut self) {
        if let AppMode::Browser(ref mut state) = self.mode {
            state.loading = true;
            let book = state.selected_book_name();
            let chapter = state.selected_chapter;
            let translation = state.translation.clone();

            match self.resolver.get_chapter(book, chapter, &translation).await {
                Ok(ch) => {
                    state.current_chapter = Some(ch);
                    state.scripture_scroll = 0;
                }
                Err(e) => {
                    eprintln!("Error loading chapter: {}", e);
                }
            }
            state.loading = false;
        }
    }
}
