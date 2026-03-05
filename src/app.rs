use crate::api::Resolver;
use crate::store::state as session;
use crate::ui::banner::{self, BannerState};
use crate::ui::browser::{self, BrowserState, SearchMode};
use crate::ui::theme::{self, ThemeName};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::ListState;
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
        let chapter_num = if has_saved_session { saved.chapter.max(1) } else { 1 };

        let initial_chapter = self
            .resolver
            .get_chapter(book_name, chapter_num, "KJV")
            .await
            .ok();

        match &mut self.mode {
            AppMode::Browser(ref mut state) => {
                if has_saved_session {
                    state.restore(&saved);
                }
                state.current_chapter = initial_chapter.clone();
            }
            _ => {}
        }

        let mut pending_initial = Some((initial_chapter, saved));

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
                        if let Some((ch, ref saved)) = pending_initial {
                            let has_saved = saved.book_index > 0 || saved.chapter > 1;
                            if has_saved {
                                browser.restore(saved);
                            }
                            browser.current_chapter = ch;
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
                // Handle search input mode first
                match &state.search {
                    SearchMode::Input(_) => {
                        self.handle_search_input(key).await;
                        return;
                    }
                    SearchMode::Results { .. } => {
                        self.handle_search_results(key).await;
                        return;
                    }
                    SearchMode::Off => {}
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
                        state.search = SearchMode::Input(String::new());
                    }
                    KeyCode::Char('t') => {
                        self.theme_name = self.theme_name.next();
                    }
                    KeyCode::Left => {
                        state.prev_panel();
                    }
                    KeyCode::Right => {
                        let should_load = state.next_panel_or_select();
                        if should_load {
                            self.load_chapter().await;
                        }
                    }
                    KeyCode::Up => {
                        state.move_up();
                    }
                    KeyCode::Down => {
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

    async fn handle_search_input(&mut self, key: KeyCode) {
        let state = match &mut self.mode {
            AppMode::Browser(s) => s,
            _ => return,
        };

        match key {
            KeyCode::Esc => {
                state.search = SearchMode::Off;
            }
            KeyCode::Enter => {
                let query = match &state.search {
                    SearchMode::Input(q) => q.clone(),
                    _ => return,
                };
                if query.trim().is_empty() {
                    state.search = SearchMode::Off;
                    return;
                }
                // Perform search
                match self.resolver.search(&query, "KJV").await {
                    Ok(results) => {
                        let mut list_state = ListState::default();
                        if !results.is_empty() {
                            list_state.select(Some(0));
                        }
                        state.search = SearchMode::Results {
                            query,
                            results,
                            list_state,
                        };
                    }
                    Err(_) => {
                        state.search = SearchMode::Results {
                            query,
                            results: vec![],
                            list_state: ListState::default(),
                        };
                    }
                }
            }
            KeyCode::Backspace => {
                if let SearchMode::Input(ref mut text) = state.search {
                    text.pop();
                }
            }
            KeyCode::Char(c) => {
                if let SearchMode::Input(ref mut text) = state.search {
                    text.push(c);
                }
            }
            _ => {}
        }
    }

    async fn handle_search_results(&mut self, key: KeyCode) {
        let state = match &mut self.mode {
            AppMode::Browser(s) => s,
            _ => return,
        };

        match key {
            KeyCode::Esc => {
                state.search = SearchMode::Off;
            }
            KeyCode::Up => {
                if let SearchMode::Results { list_state, .. } = &mut state.search {
                    let i = list_state.selected().unwrap_or(0);
                    if i > 0 {
                        list_state.select(Some(i - 1));
                    }
                }
            }
            KeyCode::Down => {
                if let SearchMode::Results { results, list_state, .. } = &mut state.search {
                    let i = list_state.selected().unwrap_or(0);
                    if i < results.len().saturating_sub(1) {
                        list_state.select(Some(i + 1));
                    }
                }
            }
            KeyCode::Enter => {
                // Jump to selected result
                let target = state.selected_search_result().map(|r| {
                    (r.book.clone(), r.chapter)
                });
                if let Some((book, chapter)) = target {
                    state.jump_to_result(&book, chapter);
                    self.load_chapter().await;
                }
            }
            KeyCode::Char('/') => {
                // Start a new search
                state.search = SearchMode::Input(String::new());
            }
            _ => {}
        }
    }

    async fn load_chapter(&mut self) {
        if let AppMode::Browser(ref mut state) = self.mode {
            state.loading = true;
            let book = state.selected_book_name();
            let chapter = state.selected_chapter;

            match self.resolver.get_chapter(book, chapter, "KJV").await {
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
