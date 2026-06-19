//! Headless smoke test for the TUI: drives the app with simulated key events
//! against ratatui's `TestBackend` and asserts every screen renders without
//! panicking. The delete step targets a throwaway tempdir file only.

use std::fs;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use crab_clean::config::Config;
use crab_clean::core::model::FileEntry;
use crab_clean::tui::app::{App, HomePanel, Screen};
use crab_clean::tui::ui;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tempfile::tempdir;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn tui_drives_full_flow_on_mock_file() {
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let mut app = App::new(Config::default());

    let draw = |t: &mut Terminal<TestBackend>, a: &mut App| {
        t.draw(|f| ui::render(f, a)).unwrap();
    };

    // Home renders, mode navigation + help overlay work.
    draw(&mut terminal, &mut app);
    app.on_key(key(KeyCode::Down));
    app.on_key(key(KeyCode::Up));
    app.on_key(key(KeyCode::F(1)));
    assert!(app.show_help);
    draw(&mut terminal, &mut app);
    app.on_key(key(KeyCode::Esc)); // any key closes help
    assert!(!app.show_help);

    // Build a review state backed by a real mock file in a tempdir.
    let dir = tempdir().unwrap();
    let path = dir.path().join("mock.txt");
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(b"throwaway").unwrap();

    app.entries = vec![FileEntry {
        path: path.clone(),
        size: 9,
        modified: None,
        accessed: None,
        created: None,
        extension: Some("txt".to_string()),
    }];
    app.group_of = vec![None];
    app.filtered = vec![0];
    app.table_state.select(Some(0));
    app.screen = Screen::Review;
    draw(&mut terminal, &mut app);

    // Filter popup opens, renders, and closes.
    app.on_key(key(KeyCode::Char('f')));
    assert!(app.filter_draft.is_some());
    draw(&mut terminal, &mut app);
    app.on_key(key(KeyCode::Esc));
    assert!(app.filter_draft.is_none());

    // Mark the file, open the confirm modal, switch to permanent, render.
    app.on_key(key(KeyCode::Char(' ')));
    assert_eq!(app.selected.len(), 1);
    app.on_key(key(KeyCode::Char('d')));
    assert!(app.confirm_delete, "confirm modal should open");
    assert_eq!(
        app.screen,
        Screen::Review,
        "still on Review behind the modal"
    );
    app.on_key(key(KeyCode::Char('t'))); // -> permanent (deletes the mock file)
    draw(&mut terminal, &mut app);

    // Confirm deletion of the mock file, then render the summary.
    app.on_key(key(KeyCode::Char('y')));
    assert_eq!(app.screen, Screen::Summary);
    draw(&mut terminal, &mut app);

    let report = app.report.as_ref().expect("report present");
    assert_eq!(report.deleted, 1);
    assert!(!path.exists(), "mock file should be gone");

    // Return home.
    app.on_key(key(KeyCode::Enter));
    assert_eq!(app.screen, Screen::Home);
    draw(&mut terminal, &mut app);
}

#[test]
fn home_browser_navigates_and_scans() {
    // A mock tree: a subdir to browse into, plus two identical files for the
    // duplicate scan to find.
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("sub")).unwrap();
    for name in ["dup_a.txt", "dup_b.txt"] {
        let mut f = fs::File::create(root.path().join(name)).unwrap();
        f.write_all(b"identical bytes for dedupe").unwrap();
    }

    let config = Config {
        default_path: root.path().to_string_lossy().into_owned(),
        ..Config::default()
    };
    let mut terminal = Terminal::new(TestBackend::new(120, 40)).unwrap();
    let mut app = App::new(config);
    let draw = |t: &mut Terminal<TestBackend>, a: &mut App| {
        t.draw(|f| ui::render(f, a)).unwrap();
    };

    draw(&mut terminal, &mut app);

    // Focus the browser, descend into "sub", then go back up.
    app.on_key(key(KeyCode::Tab));
    assert_eq!(app.home_panel, HomePanel::Browser);
    // Move to the "sub/" entry (item 0 is "../") and open it.
    app.on_key(key(KeyCode::Down));
    app.on_key(key(KeyCode::Enter));
    assert!(app.browse_dir.ends_with("sub"));
    draw(&mut terminal, &mut app);
    app.on_key(key(KeyCode::Left)); // back up to root
    assert!(!app.browse_dir.ends_with("sub"));

    // Mode defaults to Duplicates; kick off a scan with 's'.
    app.on_key(key(KeyCode::Char('s')));
    assert_eq!(app.screen, Screen::Scanning);

    // Pump the event loop until the background scan completes.
    for _ in 0..300 {
        app.tick();
        draw(&mut terminal, &mut app);
        if app.screen == Screen::Review {
            break;
        }
        sleep(Duration::from_millis(10));
    }

    assert_eq!(
        app.screen,
        Screen::Review,
        "scan should reach the review screen"
    );
    assert_eq!(
        app.entries.len(),
        2,
        "two identical files form one duplicate group"
    );
    // All-but-newest pre-selected for duplicates.
    assert_eq!(app.selected.len(), 1);
}

#[test]
fn settings_modal_toggles_vim_and_theme() {
    // NOTE: we never close the modal here — closing persists config to the real
    // user config dir, which a test must not touch. We only assert in-memory
    // state changes.
    let mut terminal = Terminal::new(TestBackend::new(100, 36)).unwrap();
    let mut app = App::new(Config::default());
    let draw = |t: &mut Terminal<TestBackend>, a: &mut App| {
        t.draw(|f| ui::render(f, a)).unwrap();
    };

    assert!(!app.vim_mode);

    // Open settings from Home and render the modal.
    app.on_key(key(KeyCode::Char(',')));
    assert!(app.show_settings);
    draw(&mut terminal, &mut app);

    // Row 0 is "Vim navigation" — toggle it on.
    app.on_key(key(KeyCode::Enter));
    assert!(app.vim_mode);
    assert!(app.config.vim_mode);

    // Move to the theme row and cycle it.
    let before = app.config.theme;
    app.on_key(key(KeyCode::Down));
    app.on_key(key(KeyCode::Right));
    assert_ne!(app.config.theme, before, "theme should cycle");
    draw(&mut terminal, &mut app);
}

#[test]
fn renders_on_tiny_terminal_without_panic() {
    // Layout math must not panic on cramped terminals.
    let mut terminal = Terminal::new(TestBackend::new(20, 6)).unwrap();
    let mut app = App::new(Config::default());

    terminal.draw(|f| ui::render(f, &mut app)).unwrap(); // Home

    app.show_help = true;
    terminal.draw(|f| ui::render(f, &mut app)).unwrap();
    app.show_help = false;

    app.show_settings = true;
    terminal.draw(|f| ui::render(f, &mut app)).unwrap();
    app.show_settings = false;

    app.confirm_empty_trash = true;
    terminal.draw(|f| ui::render(f, &mut app)).unwrap();
    app.confirm_empty_trash = false;

    app.screen = Screen::Scanning;
    terminal.draw(|f| ui::render(f, &mut app)).unwrap();

    app.screen = Screen::Summary;
    terminal.draw(|f| ui::render(f, &mut app)).unwrap();
}
