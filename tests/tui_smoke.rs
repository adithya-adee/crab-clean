//! Headless smoke test for the TUI: drives the app with simulated key events
//! against ratatui's `TestBackend` and asserts every screen renders without
//! panicking. The delete step targets a throwaway tempdir file only.

use std::fs;
use std::io::Write;

use crab_clean::config::Config;
use crab_clean::core::model::FileEntry;
use crab_clean::tui::app::{App, Screen};
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

    // Mark the file, go to confirm, switch to permanent, render.
    app.on_key(key(KeyCode::Char(' ')));
    assert_eq!(app.selected.len(), 1);
    app.on_key(key(KeyCode::Char('d')));
    assert_eq!(app.screen, Screen::Confirm);
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
