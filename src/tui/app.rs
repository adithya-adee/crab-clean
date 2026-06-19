//! Application state and the update logic that reacts to input and scan events.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::TableState;

use crate::config::Config;
use crate::core::algorithms::{find_by_age, find_duplicates};
use crate::core::delete::{self, DeleteMode, DeleteReport};
use crate::core::filter::{Filter, parse_date, parse_size};
use crate::core::model::{FileEntry, ScanMode, TimeBasis};
use crate::core::scanner::{ScanOptions, scan};
use crate::tui::theme::Theme;

/// Top-level screen the user is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Home,
    Scanning,
    Review,
    Confirm,
    Summary,
}

/// Which control on the home screen receives text/toggle input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomeFocus {
    Path,
    Age,
    Depth,
    Hidden,
    Symlinks,
}

impl HomeFocus {
    const ORDER: [HomeFocus; 5] = [
        HomeFocus::Path,
        HomeFocus::Age,
        HomeFocus::Depth,
        HomeFocus::Hidden,
        HomeFocus::Symlinks,
    ];
    fn next(self) -> Self {
        let i = Self::ORDER.iter().position(|f| *f == self).unwrap_or(0);
        Self::ORDER[(i + 1) % Self::ORDER.len()]
    }
    fn prev(self) -> Self {
        let i = Self::ORDER.iter().position(|f| *f == self).unwrap_or(0);
        Self::ORDER[(i + Self::ORDER.len() - 1) % Self::ORDER.len()]
    }
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
    pub show_help: bool,
    pub confirm_empty_trash: bool,
    pub trash_count: usize,

    // --- Home screen ---
    pub mode_index: usize,
    pub home_focus: HomeFocus,
    pub path_input: String,
    pub age_input: String,
    pub depth_input: String,
    pub include_hidden: bool,
    pub follow_symlinks: bool,

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
        App {
            running: true,
            theme,
            screen: Screen::Home,
            status: "Welcome to crab-clean. Pick a mode, set a path, press Enter to scan.".into(),
            show_help: false,
            confirm_empty_trash: false,
            trash_count,
            mode_index: 0,
            home_focus: HomeFocus::Path,
            path_input: config.default_path.clone(),
            age_input: config.default_age_days.to_string(),
            depth_input,
            include_hidden: config.include_hidden,
            follow_symlinks: config.follow_symlinks,
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
        }
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

        // Modal overlays take precedence.
        if self.show_help {
            self.show_help = false;
            return;
        }
        if self.confirm_empty_trash {
            self.on_key_empty_trash(key);
            return;
        }

        // F1 opens help from anywhere (even while editing a text field, where
        // '?' would be typed instead).
        if key.code == KeyCode::F(1) {
            self.show_help = true;
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
            Screen::Review => {
                if self.filter_draft.is_some() {
                    self.on_key_filter(key);
                } else {
                    self.on_key_review(key);
                }
            }
            Screen::Confirm => self.on_key_confirm(key),
            Screen::Summary => {
                if matches!(key.code, KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q')) {
                    self.back_to_home();
                }
            }
        }
    }

    fn on_key_home(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.running = false,
            KeyCode::Enter => self.start_scan(),
            KeyCode::Tab => self.home_focus = self.home_focus.next(),
            KeyCode::BackTab => self.home_focus = self.home_focus.prev(),
            KeyCode::Up => {
                self.mode_index = (self.mode_index + ScanMode::ALL.len() - 1) % ScanMode::ALL.len()
            }
            KeyCode::Down => self.mode_index = (self.mode_index + 1) % ScanMode::ALL.len(),
            KeyCode::Backspace => match self.home_focus {
                HomeFocus::Path => {
                    self.path_input.pop();
                }
                HomeFocus::Age => {
                    self.age_input.pop();
                }
                HomeFocus::Depth => {
                    self.depth_input.pop();
                }
                _ => {}
            },
            KeyCode::Char(c) => self.home_char(c),
            _ => {}
        }
    }

    fn home_char(&mut self, c: char) {
        match self.home_focus {
            HomeFocus::Path => self.path_input.push(c),
            HomeFocus::Age => {
                if c.is_ascii_digit() {
                    self.age_input.push(c);
                }
            }
            HomeFocus::Depth => {
                if c.is_ascii_digit() {
                    self.depth_input.push(c);
                }
            }
            HomeFocus::Hidden => {
                if c == ' ' {
                    self.include_hidden = !self.include_hidden;
                }
            }
            HomeFocus::Symlinks => {
                if c == ' ' {
                    self.follow_symlinks = !self.follow_symlinks;
                }
            }
        }

        // Commands available regardless of focus that don't conflict with input.
        if c == '?' && !matches!(self.home_focus, HomeFocus::Path) {
            self.show_help = true;
        }
        if c == 'e' && !matches!(self.home_focus, HomeFocus::Path) {
            self.open_empty_trash_confirm();
        }
    }

    fn on_key_review(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.back_to_home(),
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(1),
            KeyCode::PageUp => self.move_cursor(-10),
            KeyCode::PageDown => self.move_cursor(10),
            KeyCode::Home | KeyCode::Char('g') => self.set_cursor(0),
            KeyCode::End | KeyCode::Char('G') if !self.filtered.is_empty() => {
                self.set_cursor(self.filtered.len() - 1);
            }
            KeyCode::Char(' ') => self.toggle_current(),
            KeyCode::Char('a') => self.select_all_filtered(),
            KeyCode::Char('c') => {
                self.selected.clear();
                self.status = "Selection cleared".into();
            }
            KeyCode::Char('i') => self.invert_filtered(),
            KeyCode::Char('f') => self.open_filter(),
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
            KeyCode::Char('t') => {
                self.delete_mode = match self.delete_mode {
                    DeleteMode::Trash => DeleteMode::Permanent,
                    DeleteMode::Permanent => DeleteMode::Trash,
                };
            }
            KeyCode::Char('y') | KeyCode::Enter => self.do_delete(),
            KeyCode::Esc | KeyCode::Char('n') => self.screen = Screen::Review,
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
        let root = expand_path(&self.path_input);
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
        self.screen = Screen::Confirm;
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
