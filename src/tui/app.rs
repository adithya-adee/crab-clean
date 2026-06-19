//! Application state and the update logic that reacts to input and scan events.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::{ListState, TableState};

use crate::config::Config;
use crate::core::algorithms::{find_by_age, find_duplicates};
use crate::core::delete::{self, DeleteMode, DeleteReport};
use crate::core::filter::{Filter, parse_date, parse_size};
use crate::core::model::{FileEntry, ScanMode, TimeBasis};
use crate::core::scanner::{ScanOptions, scan};
use crate::tui::theme::Theme;

/// Top-level screen (page) the user is on. Yes/no prompts and forms are modals
/// layered on top of these, not separate screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Scanning,
    Review,
    Summary,
}

/// Rows in the settings modal.
pub const SETTING_COUNT: usize = 5;
pub const SETTING_VIM: usize = 0;
pub const SETTING_THEME: usize = 1;
pub const SETTING_DELETE: usize = 2;
pub const SETTING_BASIS: usize = 3;
pub const SETTING_HIDDEN: usize = 4;

/// Which panel on the home screen currently has focus. The focused panel is
/// drawn with a bold accent border so the user always knows where they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomePanel {
    Mode,
    Browser,
    Options,
}

impl HomePanel {
    const ORDER: [HomePanel; 3] = [HomePanel::Mode, HomePanel::Browser, HomePanel::Options];
    fn next(self) -> Self {
        let i = Self::ORDER.iter().position(|p| *p == self).unwrap_or(0);
        Self::ORDER[(i + 1) % Self::ORDER.len()]
    }
    fn prev(self) -> Self {
        let i = Self::ORDER.iter().position(|p| *p == self).unwrap_or(0);
        Self::ORDER[(i + Self::ORDER.len() - 1) % Self::ORDER.len()]
    }
}

/// Rows in the options panel, navigated with up/down.
pub const OPTION_COUNT: usize = 4;
pub const OPTION_AGE: usize = 0;
pub const OPTION_DEPTH: usize = 1;
pub const OPTION_HIDDEN: usize = 2;
pub const OPTION_SYMLINKS: usize = 3;

/// One entry in the directory browser: a display label and where it leads.
#[derive(Debug, Clone)]
pub struct BrowseItem {
    pub label: String,
    pub path: PathBuf,
    pub is_parent: bool,
}

/// Sort key for the review table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Size,
    Modified,
    Extension,
}

impl SortKey {
    pub fn label(&self) -> &'static str {
        match self {
            SortKey::Name => "name",
            SortKey::Size => "size",
            SortKey::Modified => "modified",
            SortKey::Extension => "extension",
        }
    }
    fn next(self) -> Self {
        match self {
            SortKey::Name => SortKey::Size,
            SortKey::Size => SortKey::Modified,
            SortKey::Modified => SortKey::Extension,
            SortKey::Extension => SortKey::Name,
        }
    }
}

pub const FILTER_LABELS: [&str; 6] = [
    "Extensions (comma sep, e.g. jpg,png)",
    "Name contains",
    "Min size (e.g. 10MB)",
    "Max size (e.g. 2GB)",
    "Modified after (YYYY-MM-DD)",
    "Modified before (YYYY-MM-DD)",
];

/// Draft state for the filter popup (raw strings, parsed on apply).
#[derive(Debug, Clone)]
pub struct FilterDraft {
    pub fields: [String; 6],
    pub focus: usize,
    pub error: Option<String>,
}

/// Message sent from the background scan thread to the UI.
enum ScanMsg {
    Progress { stage: &'static str, count: usize },
    Done(ScanOutcome),
    Error(String),
}

struct ScanOutcome {
    entries: Vec<FileEntry>,
    dup_groups: Option<Vec<Vec<usize>>>,
}

/// The whole application state.
pub struct App {
    pub running: bool,
    pub config: Config,
    pub theme: Theme,
    pub screen: Screen,
    pub status: String,
    pub trash_count: usize,
    /// Animation frame counter (drives the scan spinner).
    pub frame: u64,
    /// Vim-style navigation (hjkl, gg/G).
    pub vim_mode: bool,
    /// Tracks a pending `g` for the vim `gg` (go-to-top) chord.
    pub pending_g: bool,

    // --- Modals (layered over the current screen) ---
    pub show_help: bool,
    pub confirm_empty_trash: bool,
    pub confirm_delete: bool,
    pub show_settings: bool,
    pub settings_index: usize,

    // --- Home screen ---
    pub mode_index: usize,
    pub home_panel: HomePanel,
    pub option_index: usize,
    pub age_input: String,
    pub depth_input: String,
    pub include_hidden: bool,
    pub follow_symlinks: bool,

    // --- Directory browser ---
    pub browse_dir: PathBuf,
    pub browse_items: Vec<BrowseItem>,
    pub browse_state: ListState,

    // --- Scanning ---
    scan_rx: Option<Receiver<ScanMsg>>,
    pub scan_stage: String,
    pub scan_count: usize,
    pub scanning_mode: ScanMode,

    // --- Review ---
    pub entries: Vec<FileEntry>,
    pub dup_groups: Option<Vec<Vec<usize>>>,
    pub group_of: Vec<Option<usize>>,
    pub filtered: Vec<usize>,
    pub selected: HashSet<usize>,
    pub table_state: TableState,
    pub filter: Filter,
    pub sort_key: SortKey,
    pub sort_desc: bool,
    pub filter_draft: Option<FilterDraft>,

    // --- Confirm ---
    pub delete_mode: DeleteMode,

    // --- Summary ---
    pub report: Option<DeleteReport>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let theme = Theme::from_name(config.theme);
        let depth_input = config.max_depth.map(|d| d.to_string()).unwrap_or_default();
        let trash_count = delete::trash_item_count().unwrap_or(0);
        let browse_dir = resolve_start_dir(&config.default_path);
        let mut app = App {
            running: true,
            theme,
            screen: Screen::Home,
            status: "Pick a mode (left), browse to a folder (Tab), then press 's' to scan.".into(),
            trash_count,
            frame: 0,
            vim_mode: config.vim_mode,
            pending_g: false,
            show_help: false,
            confirm_empty_trash: false,
            confirm_delete: false,
            show_settings: false,
            settings_index: 0,
            mode_index: 0,
            home_panel: HomePanel::Mode,
            option_index: 0,
            age_input: config.default_age_days.to_string(),
            depth_input,
            include_hidden: config.include_hidden,
            follow_symlinks: config.follow_symlinks,
            browse_dir,
            browse_items: Vec::new(),
            browse_state: ListState::default(),
            scan_rx: None,
            scan_stage: String::new(),
            scan_count: 0,
            scanning_mode: ScanMode::Duplicates,
            entries: Vec::new(),
            dup_groups: None,
            group_of: Vec::new(),
            filtered: Vec::new(),
            selected: HashSet::new(),
            table_state: TableState::default(),
            filter: Filter::default(),
            sort_key: SortKey::Size,
            sort_desc: true,
            filter_draft: None,
            delete_mode: config.delete_mode,
            report: None,
            config,
        };
        app.refresh_browser();
        app
    }

    pub fn current_mode(&self) -> ScanMode {
        ScanMode::ALL[self.mode_index]
    }

    // ------------------------------------------------------------------ input

    pub fn on_key(&mut self, key: KeyEvent) {
        if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
            return;
        }

        // Global: Ctrl-C / Ctrl-Q always quit.
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('q'))
        {
            self.running = false;
            return;
        }

        // Any key other than `g` cancels a pending vim `gg` chord.
        if key.code != KeyCode::Char('g') {
            self.pending_g = false;
        }

        // Help is dismissed by any key.
        if self.show_help {
            self.show_help = false;
            return;
        }
        // F1 opens help from anywhere (even while editing a text field).
        if key.code == KeyCode::F(1) {
            self.show_help = true;
            return;
        }

        // Modal dialogs capture all input while open (highest priority first).
        if self.confirm_empty_trash {
            self.on_key_empty_trash(key);
            return;
        }
        if self.show_settings {
            self.on_key_settings(key);
            return;
        }
        if self.confirm_delete {
            self.on_key_confirm(key);
            return;
        }
        if self.filter_draft.is_some() {
            self.on_key_filter(key);
            return;
        }

        match self.screen {
            Screen::Home => self.on_key_home(key),
            Screen::Scanning => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    // Detach the scan and return home.
                    self.scan_rx = None;
                    self.screen = Screen::Home;
                }
            }
            Screen::Review => self.on_key_review(key),
            Screen::Summary => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q')) {
                    self.back_to_home();
                }
            }
        }
    }

    fn on_key_home(&mut self, key: KeyEvent) {
        // Global home commands first (no free-text fields exist on this screen,
        // so single-letter commands never conflict with input).
        match key.code {
            KeyCode::Esc => {
                self.running = false;
                return;
            }
            KeyCode::Tab => {
                self.home_panel = self.home_panel.next();
                return;
            }
            KeyCode::BackTab => {
                self.home_panel = self.home_panel.prev();
                return;
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.start_scan();
                return;
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.open_empty_trash_confirm();
                return;
            }
            KeyCode::Char(',') => {
                self.open_settings();
                return;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Char('~') => {
                if let Some(home) = dirs::home_dir() {
                    self.browse_dir = home;
                    self.refresh_browser();
                    self.browse_state.select(Some(0));
                    self.home_panel = HomePanel::Browser;
                }
                return;
            }
            KeyCode::Enter => {
                // In the browser, Enter opens the highlighted directory;
                // elsewhere it starts the scan.
                if self.home_panel == HomePanel::Browser {
                    self.browser_open();
                } else {
                    self.start_scan();
                }
                return;
            }
            _ => {}
        }

        let vim = self.vim_mode;
        match self.home_panel {
            HomePanel::Mode => match key.code {
                KeyCode::Up => self.mode_prev(),
                KeyCode::Down => self.mode_next(),
                KeyCode::Char('k') if vim => self.mode_prev(),
                KeyCode::Char('j') if vim => self.mode_next(),
                _ => {}
            },
            HomePanel::Browser => match key.code {
                KeyCode::Up => self.browse_move(-1),
                KeyCode::Down => self.browse_move(1),
                KeyCode::Char('k') if vim => self.browse_move(-1),
                KeyCode::Char('j') if vim => self.browse_move(1),
                KeyCode::PageUp => self.browse_move(-10),
                KeyCode::PageDown => self.browse_move(10),
                KeyCode::Right => self.browser_open(),
                KeyCode::Char('l') if vim => self.browser_open(),
                KeyCode::Left | KeyCode::Backspace => self.browser_up(),
                KeyCode::Char('h') if vim => self.browser_up(),
                KeyCode::Char('G') if vim && !self.browse_items.is_empty() => {
                    self.browse_state.select(Some(self.browse_items.len() - 1));
                }
                KeyCode::Char('g') if vim => {
                    if self.pending_g {
                        self.browse_state.select(Some(0));
                        self.pending_g = false;
                    } else {
                        self.pending_g = true;
                    }
                }
                _ => {}
            },
            HomePanel::Options => self.on_key_options(key),
        }
    }

    fn mode_prev(&mut self) {
        self.mode_index = (self.mode_index + ScanMode::ALL.len() - 1) % ScanMode::ALL.len();
    }

    fn mode_next(&mut self) {
        self.mode_index = (self.mode_index + 1) % ScanMode::ALL.len();
    }

    fn on_key_options(&mut self, key: KeyEvent) {
        let vim = self.vim_mode;
        match key.code {
            KeyCode::Up => {
                self.option_index = (self.option_index + OPTION_COUNT - 1) % OPTION_COUNT;
            }
            KeyCode::Down => {
                self.option_index = (self.option_index + 1) % OPTION_COUNT;
            }
            KeyCode::Char('k') if vim => {
                self.option_index = (self.option_index + OPTION_COUNT - 1) % OPTION_COUNT;
            }
            KeyCode::Char('j') if vim => {
                self.option_index = (self.option_index + 1) % OPTION_COUNT;
            }
            KeyCode::Left => self.adjust_option(-1),
            KeyCode::Right => self.adjust_option(1),
            KeyCode::Char('h') if vim => self.adjust_option(-1),
            KeyCode::Char('l') if vim => self.adjust_option(1),
            KeyCode::Char(' ') => match self.option_index {
                OPTION_HIDDEN => self.toggle_hidden(),
                OPTION_SYMLINKS => self.follow_symlinks = !self.follow_symlinks,
                _ => {}
            },
            KeyCode::Char(c) if c.is_ascii_digit() => match self.option_index {
                OPTION_AGE if self.age_input.len() < 5 => self.age_input.push(c),
                OPTION_DEPTH if self.depth_input.len() < 3 => self.depth_input.push(c),
                _ => {}
            },
            KeyCode::Backspace => match self.option_index {
                OPTION_AGE => {
                    self.age_input.pop();
                }
                OPTION_DEPTH => {
                    self.depth_input.pop();
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn adjust_option(&mut self, delta: i64) {
        match self.option_index {
            OPTION_AGE => {
                let cur = self.age_input.trim().parse::<i64>().unwrap_or(30);
                let next = (cur + delta).clamp(1, 99_999);
                self.age_input = next.to_string();
            }
            OPTION_DEPTH => {
                // Empty string = unlimited; stepping down from unlimited has no effect.
                let cur = self.depth_input.trim().parse::<i64>().ok();
                let next = match cur {
                    Some(v) => (v + delta).max(0),
                    None if delta < 0 => return,
                    None => 1,
                };
                self.depth_input = next.to_string();
            }
            OPTION_HIDDEN => self.toggle_hidden(),
            OPTION_SYMLINKS => self.follow_symlinks = !self.follow_symlinks,
            _ => {}
        }
    }

    fn toggle_hidden(&mut self) {
        self.include_hidden = !self.include_hidden;
        // Hidden directories should appear/disappear from the browser too.
        self.refresh_browser();
    }

    // ------------------------------------------------------------- dir browser

    /// Rebuild the browser listing for the current `browse_dir`.
    pub fn refresh_browser(&mut self) {
        let mut items = Vec::new();
        if let Some(parent) = self.browse_dir.parent() {
            items.push(BrowseItem {
                label: "../".to_string(),
                path: parent.to_path_buf(),
                is_parent: true,
            });
        }
        for dir in read_subdirs(&self.browse_dir, self.include_hidden) {
            let name = dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            items.push(BrowseItem {
                label: format!("{name}/"),
                path: dir,
                is_parent: false,
            });
        }
        let len = items.len();
        self.browse_items = items;
        let sel = self.browse_state.selected().unwrap_or(0);
        self.browse_state.select(if len == 0 {
            None
        } else {
            Some(sel.min(len - 1))
        });
    }

    fn browse_move(&mut self, delta: i64) {
        if self.browse_items.is_empty() {
            return;
        }
        let cur = self.browse_state.selected().unwrap_or(0) as i64;
        let last = (self.browse_items.len() - 1) as i64;
        self.browse_state
            .select(Some((cur + delta).clamp(0, last) as usize));
    }

    /// Descend into (or go up to) the highlighted directory.
    fn browser_open(&mut self) {
        let Some(sel) = self.browse_state.selected() else {
            return;
        };
        let Some(item) = self.browse_items.get(sel).cloned() else {
            return;
        };
        let previous = self.browse_dir.clone();
        self.browse_dir = item.path;
        self.refresh_browser();
        // When going up, re-highlight the directory we came from.
        if let Some(i) = self.browse_items.iter().position(|it| it.path == previous) {
            self.browse_state.select(Some(i));
        } else {
            self.browse_state.select(Some(0));
        }
    }

    fn browser_up(&mut self) {
        if let Some(parent) = self.browse_dir.parent().map(Path::to_path_buf) {
            let previous = self.browse_dir.clone();
            self.browse_dir = parent;
            self.refresh_browser();
            if let Some(i) = self.browse_items.iter().position(|it| it.path == previous) {
                self.browse_state.select(Some(i));
            } else {
                self.browse_state.select(Some(0));
            }
        }
    }

    fn on_key_review(&mut self, key: KeyEvent) {
        let vim = self.vim_mode;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.back_to_home(),
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(1),
            KeyCode::PageUp => self.move_cursor(-10),
            KeyCode::PageDown => self.move_cursor(10),
            KeyCode::Home => self.set_cursor(0),
            KeyCode::End if !self.filtered.is_empty() => {
                self.set_cursor(self.filtered.len() - 1);
            }
            // `g`: immediate top in normal mode; `gg` chord in vim mode.
            KeyCode::Char('g') => {
                if vim {
                    if self.pending_g {
                        self.set_cursor(0);
                        self.pending_g = false;
                    } else {
                        self.pending_g = true;
                    }
                } else {
                    self.set_cursor(0);
                }
            }
            KeyCode::Char('G') if !self.filtered.is_empty() => {
                self.set_cursor(self.filtered.len() - 1);
            }
            KeyCode::Char(' ') | KeyCode::Char('x') => self.toggle_current(),
            KeyCode::Char('a') => self.select_all_filtered(),
            KeyCode::Char('c') => {
                self.selected.clear();
                self.status = "Selection cleared".into();
            }
            KeyCode::Char('i') => self.invert_filtered(),
            KeyCode::Char('f') | KeyCode::Char('/') => self.open_filter(),
            KeyCode::Char('F') => {
                self.filter = Filter {
                    time_basis: self.config.time_basis,
                    ..Filter::default()
                };
                self.rebuild_view();
                self.status = "Filters cleared".into();
            }
            KeyCode::Char('s') => {
                self.sort_key = self.sort_key.next();
                self.sort_filtered();
                self.status = format!("Sorted by {}", self.sort_key.label());
            }
            KeyCode::Char('S') => {
                self.sort_desc = !self.sort_desc;
                self.sort_filtered();
            }
            KeyCode::Char(',') => self.open_settings(),
            KeyCode::Char('?') => self.show_help = true,
            KeyCode::Enter | KeyCode::Char('d') => self.go_to_confirm(),
            _ => {}
        }
    }

    fn on_key_filter(&mut self, key: KeyEvent) {
        let Some(draft) = self.filter_draft.as_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.filter_draft = None,
            KeyCode::Enter => self.apply_filter_draft(),
            KeyCode::Tab | KeyCode::Down => draft.focus = (draft.focus + 1) % 6,
            KeyCode::BackTab | KeyCode::Up => draft.focus = (draft.focus + 5) % 6,
            KeyCode::Backspace => {
                draft.fields[draft.focus].pop();
            }
            KeyCode::Char(c) => draft.fields[draft.focus].push(c),
            _ => {}
        }
    }

    fn on_key_confirm(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('t') | KeyCode::Tab => {
                self.delete_mode = self.delete_mode.toggled();
            }
            KeyCode::Char('y') | KeyCode::Enter => self.do_delete(),
            KeyCode::Esc | KeyCode::Char('n') => self.confirm_delete = false,
            _ => {}
        }
    }

    // ----------------------------------------------------------------- settings

    fn open_settings(&mut self) {
        self.settings_index = 0;
        self.show_settings = true;
    }

    fn on_key_settings(&mut self, key: KeyEvent) {
        let vim = self.vim_mode;
        match key.code {
            KeyCode::Esc | KeyCode::Char(',') => {
                self.show_settings = false;
                // Persist on close (best effort).
                match self.config.save() {
                    Ok(()) => self.status = "Settings saved".into(),
                    Err(e) => self.status = format!("Could not save settings: {e}"),
                }
            }
            KeyCode::Up => self.settings_prev(),
            KeyCode::Down => self.settings_next(),
            KeyCode::Char('k') if vim => self.settings_prev(),
            KeyCode::Char('j') if vim => self.settings_next(),
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Enter
            | KeyCode::Char(' ')
            | KeyCode::Char('h')
            | KeyCode::Char('l') => self.settings_change(),
            _ => {}
        }
    }

    fn settings_prev(&mut self) {
        self.settings_index = (self.settings_index + SETTING_COUNT - 1) % SETTING_COUNT;
    }

    fn settings_next(&mut self) {
        self.settings_index = (self.settings_index + 1) % SETTING_COUNT;
    }

    /// Toggle / cycle the highlighted setting and apply it immediately.
    fn settings_change(&mut self) {
        match self.settings_index {
            SETTING_VIM => {
                self.config.vim_mode = !self.config.vim_mode;
                self.vim_mode = self.config.vim_mode;
            }
            SETTING_THEME => {
                self.config.theme = self.config.theme.next();
                self.theme = Theme::from_name(self.config.theme);
            }
            SETTING_DELETE => {
                self.config.delete_mode = self.config.delete_mode.toggled();
                self.delete_mode = self.config.delete_mode;
            }
            SETTING_BASIS => {
                self.config.time_basis = self.config.time_basis.next();
                self.filter.time_basis = self.config.time_basis;
            }
            SETTING_HIDDEN => {
                self.config.include_hidden = !self.config.include_hidden;
                self.include_hidden = self.config.include_hidden;
                self.refresh_browser();
            }
            _ => {}
        }
    }

    fn on_key_empty_trash(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                match delete::empty_trash() {
                    Ok(n) => self.status = format!("Emptied trash ({n} item(s) purged)"),
                    Err(e) => self.status = format!("Failed to empty trash: {e}"),
                }
                self.trash_count = 0;
                self.confirm_empty_trash = false;
            }
            KeyCode::Esc | KeyCode::Char('n') => self.confirm_empty_trash = false,
            _ => {}
        }
    }

    // --------------------------------------------------------------- scanning

    fn start_scan(&mut self) {
        let mode = self.current_mode();
        let root = self.browse_dir.clone();
        if !root.is_dir() {
            self.status = format!("Not a directory: {}", root.display());
            return;
        }

        let age = self
            .age_input
            .trim()
            .parse::<u32>()
            .unwrap_or(self.config.default_age_days)
            .max(1);
        let max_depth = {
            let d = self.depth_input.trim();
            if d.is_empty() {
                None
            } else {
                d.parse::<usize>().ok()
            }
        };
        let opts = ScanOptions {
            max_depth,
            include_hidden: self.include_hidden,
            follow_symlinks: self.follow_symlinks,
        };

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || run_scan(&tx, root, opts, mode, age));

        self.scan_rx = Some(rx);
        self.scanning_mode = mode;
        self.scan_stage = "Scanning files".into();
        self.scan_count = 0;
        self.screen = Screen::Scanning;
        self.selected.clear();
        self.filter = Filter {
            time_basis: self.config.time_basis,
            ..Filter::default()
        };
    }

    /// Drain any pending messages from the scan thread. Called every loop tick.
    pub fn tick(&mut self) {
        // Advance the animation clock (used by the scan spinner).
        self.frame = self.frame.wrapping_add(1);

        let Some(rx) = &self.scan_rx else {
            return;
        };
        loop {
            match rx.try_recv() {
                Ok(ScanMsg::Progress { stage, count }) => {
                    self.scan_stage = stage.to_string();
                    self.scan_count = count;
                }
                Ok(ScanMsg::Done(outcome)) => {
                    self.scan_rx = None;
                    self.on_scan_done(outcome);
                    break;
                }
                Ok(ScanMsg::Error(e)) => {
                    self.scan_rx = None;
                    self.status = format!("Scan failed: {e}");
                    self.screen = Screen::Home;
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.scan_rx = None;
                    break;
                }
            }
        }
    }

    fn on_scan_done(&mut self, outcome: ScanOutcome) {
        self.entries = outcome.entries;
        self.dup_groups = outcome.dup_groups;
        self.group_of = vec![None; self.entries.len()];
        self.selected.clear();

        if let Some(groups) = &self.dup_groups {
            for (gi, idxs) in groups.iter().enumerate() {
                for &i in idxs {
                    self.group_of[i] = Some(gi);
                }
                // Pre-select all but the newest in each duplicate group.
                let keep = idxs
                    .iter()
                    .copied()
                    .max_by_key(|&i| self.entries[i].modified.unwrap_or(UNIX_EPOCH));
                if let Some(keep) = keep {
                    for &i in idxs {
                        if i != keep {
                            self.selected.insert(i);
                        }
                    }
                }
            }
        }

        self.rebuild_view();
        self.table_state.select(if self.filtered.is_empty() {
            None
        } else {
            Some(0)
        });
        self.screen = Screen::Review;
        self.status = format!(
            "{} candidate file(s) found ({} pre-selected)",
            self.entries.len(),
            self.selected.len()
        );
    }

    // ----------------------------------------------------------------- review

    fn rebuild_view(&mut self) {
        self.filtered = (0..self.entries.len())
            .filter(|&i| self.filter.matches(&self.entries[i]))
            .collect();
        self.sort_filtered();
        let len = self.filtered.len();
        match self.table_state.selected() {
            Some(s) if s >= len => {
                self.table_state
                    .select(if len == 0 { None } else { Some(len - 1) });
            }
            None if len > 0 => self.table_state.select(Some(0)),
            _ => {}
        }
    }

    fn sort_filtered(&mut self) {
        let key = self.sort_key;
        let desc = self.sort_desc;
        let entries = &self.entries;
        self.filtered.sort_by(|&a, &b| {
            let ord = match key {
                SortKey::Name => entries[a]
                    .file_name()
                    .to_lowercase()
                    .cmp(&entries[b].file_name().to_lowercase()),
                SortKey::Size => entries[a].size.cmp(&entries[b].size),
                SortKey::Modified => entries[a].modified.cmp(&entries[b].modified),
                SortKey::Extension => entries[a].ext_str().cmp(entries[b].ext_str()),
            };
            if desc { ord.reverse() } else { ord }
        });
    }

    fn move_cursor(&mut self, delta: i64) {
        if self.filtered.is_empty() {
            return;
        }
        let cur = self.table_state.selected().unwrap_or(0) as i64;
        let last = (self.filtered.len() - 1) as i64;
        let next = (cur + delta).clamp(0, last);
        self.table_state.select(Some(next as usize));
    }

    fn set_cursor(&mut self, pos: usize) {
        if self.filtered.is_empty() {
            return;
        }
        self.table_state
            .select(Some(pos.min(self.filtered.len() - 1)));
    }

    fn toggle_current(&mut self) {
        if let Some(s) = self.table_state.selected()
            && let Some(&idx) = self.filtered.get(s)
            && !self.selected.insert(idx)
        {
            self.selected.remove(&idx);
        }
    }

    fn select_all_filtered(&mut self) {
        for &i in &self.filtered {
            self.selected.insert(i);
        }
        self.status = format!("{} file(s) selected", self.selected.len());
    }

    fn invert_filtered(&mut self) {
        for &i in &self.filtered {
            if !self.selected.insert(i) {
                self.selected.remove(&i);
            }
        }
    }

    fn open_filter(&mut self) {
        let f = &self.filter;
        let fields = [
            f.extensions.join(","),
            f.name_contains.clone().unwrap_or_default(),
            f.min_size.map(|v| v.to_string()).unwrap_or_default(),
            f.max_size.map(|v| v.to_string()).unwrap_or_default(),
            f.after.map(fmt_date).unwrap_or_default(),
            f.before.map(fmt_date).unwrap_or_default(),
        ];
        self.filter_draft = Some(FilterDraft {
            fields,
            focus: 0,
            error: None,
        });
    }

    fn apply_filter_draft(&mut self) {
        let Some(d) = self.filter_draft.clone() else {
            return;
        };
        let mut f = Filter {
            time_basis: self.config.time_basis,
            ..Filter::default()
        };

        f.extensions = d.fields[0]
            .split(',')
            .map(|s| s.trim().trim_start_matches('.').to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let name = d.fields[1].trim();
        if !name.is_empty() {
            f.name_contains = Some(name.to_lowercase());
        }

        if !d.fields[2].trim().is_empty() {
            match parse_size(&d.fields[2]) {
                Some(v) => f.min_size = Some(v),
                None => return self.set_filter_error("Invalid min size"),
            }
        }
        if !d.fields[3].trim().is_empty() {
            match parse_size(&d.fields[3]) {
                Some(v) => f.max_size = Some(v),
                None => return self.set_filter_error("Invalid max size"),
            }
        }
        if !d.fields[4].trim().is_empty() {
            match parse_date(&d.fields[4]) {
                Some(v) => f.after = Some(v),
                None => return self.set_filter_error("Invalid 'after' date (use YYYY-MM-DD)"),
            }
        }
        if !d.fields[5].trim().is_empty() {
            match parse_date(&d.fields[5]) {
                Some(v) => f.before = Some(v),
                None => return self.set_filter_error("Invalid 'before' date (use YYYY-MM-DD)"),
            }
        }

        self.filter = f;
        self.filter_draft = None;
        self.rebuild_view();
        self.status = format!("{} file(s) match filters", self.filtered.len());
    }

    fn set_filter_error(&mut self, msg: &str) {
        if let Some(d) = self.filter_draft.as_mut() {
            d.error = Some(msg.to_string());
        }
    }

    // ---------------------------------------------------------------- delete

    fn go_to_confirm(&mut self) {
        if self.selected.is_empty() {
            self.status = "Nothing selected — press Space to mark files".into();
            return;
        }
        self.confirm_delete = true;
    }

    /// Entries currently marked for deletion (sorted by path for a stable view).
    pub fn selected_entries(&self) -> Vec<&FileEntry> {
        let mut v: Vec<&FileEntry> = self
            .selected
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .collect();
        v.sort_by(|a, b| a.path.cmp(&b.path));
        v
    }

    pub fn selected_total_size(&self) -> u64 {
        self.selected
            .iter()
            .filter_map(|&i| self.entries.get(i))
            .map(|e| e.size)
            .sum()
    }

    fn do_delete(&mut self) {
        let entries: Vec<FileEntry> = self
            .selected
            .iter()
            .filter_map(|&i| self.entries.get(i).cloned())
            .collect();
        let report = delete::delete_files(&entries, self.delete_mode);
        self.status = format!(
            "Deleted {} file(s), {} failed",
            report.deleted,
            report.failed.len()
        );
        self.report = Some(report);
        self.trash_count = delete::trash_item_count().unwrap_or(self.trash_count);
        self.confirm_delete = false;
        self.screen = Screen::Summary;
    }

    // ------------------------------------------------------------------ misc

    fn open_empty_trash_confirm(&mut self) {
        self.trash_count = delete::trash_item_count().unwrap_or(0);
        if self.trash_count == 0 {
            self.status = "Trash is already empty".into();
        } else {
            self.confirm_empty_trash = true;
        }
    }

    fn back_to_home(&mut self) {
        self.screen = Screen::Home;
        self.entries.clear();
        self.filtered.clear();
        self.selected.clear();
        self.group_of.clear();
        self.dup_groups = None;
        self.report = None;
        self.table_state = TableState::default();
    }
}

// ------------------------------------------------------------------ helpers

/// Expand a leading `~` to the user's home directory.
fn expand_path(input: &str) -> PathBuf {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix('~')
        && let Some(home) = dirs::home_dir()
    {
        let rest = rest.trim_start_matches('/');
        return home.join(rest);
    }
    PathBuf::from(trimmed)
}

/// Resolve the initial browse directory to an absolute path, falling back to the
/// current working directory and finally the filesystem root.
fn resolve_start_dir(configured: &str) -> PathBuf {
    let candidate = expand_path(configured);
    std::fs::canonicalize(&candidate)
        .or_else(|_| std::env::current_dir())
        .unwrap_or_else(|_| PathBuf::from("/"))
}

/// List the sub-directories of `dir`, sorted by name, honoring the hidden flag.
fn read_subdirs(dir: &Path, include_hidden: bool) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(read) => read
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .filter(|p| {
                include_hidden
                    || !p
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with('.'))
                        .unwrap_or(false)
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    dirs.sort_by(|a, b| {
        a.file_name()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .cmp(&b.file_name().unwrap_or_default().to_ascii_lowercase())
    });
    dirs
}

/// Format a `SystemTime` as `YYYY-MM-DD`.
fn fmt_date(t: SystemTime) -> String {
    use chrono::{DateTime, Local};
    let dt: DateTime<Local> = t.into();
    dt.format("%Y-%m-%d").to_string()
}

/// Runs on the scan thread: walk, then apply the mode-specific algorithm.
fn run_scan(
    tx: &mpsc::Sender<ScanMsg>,
    root: PathBuf,
    opts: ScanOptions,
    mode: ScanMode,
    age: u32,
) {
    let _ = tx.send(ScanMsg::Progress {
        stage: "Scanning files",
        count: 0,
    });

    let all = match scan(&root, &opts, |count| {
        let _ = tx.send(ScanMsg::Progress {
            stage: "Scanning files",
            count,
        });
    }) {
        Ok(v) => v,
        Err(e) => {
            let _ = tx.send(ScanMsg::Error(e.to_string()));
            return;
        }
    };

    let outcome = match mode {
        ScanMode::Duplicates => {
            let _ = tx.send(ScanMsg::Progress {
                stage: "Hashing potential duplicates",
                count: all.len(),
            });
            let groups = find_duplicates(&all);
            let mut entries = Vec::new();
            let mut idx_groups = Vec::with_capacity(groups.len());
            for group in groups {
                let mut idxs = Vec::with_capacity(group.len());
                for entry in group {
                    idxs.push(entries.len());
                    entries.push(entry);
                }
                idx_groups.push(idxs);
            }
            ScanOutcome {
                entries,
                dup_groups: Some(idx_groups),
            }
        }
        ScanMode::Unused => ScanOutcome {
            entries: find_by_age(&all, age, TimeBasis::Accessed),
            dup_groups: None,
        },
        ScanMode::Old => ScanOutcome {
            entries: find_by_age(&all, age, TimeBasis::Modified),
            dup_groups: None,
        },
        ScanMode::AllFiles => ScanOutcome {
            entries: all,
            dup_groups: None,
        },
    };

    let _ = tx.send(ScanMsg::Done(outcome));
}
