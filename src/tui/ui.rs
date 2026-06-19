//! All rendering. Pure functions of `&App` → terminal frame.

use std::time::SystemTime;

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, Wrap,
};

use crate::core::delete::DeleteMode;
use crate::core::model::ScanMode;
use crate::tui::app::{
    App, FILTER_LABELS, HomePanel, OPTION_AGE, OPTION_DEPTH, OPTION_HIDDEN, OPTION_SYMLINKS, Screen,
};

pub fn render(f: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Home => draw_home(f, app),
        Screen::Scanning => draw_scanning(f, app),
        Screen::Review => draw_review(f, app),
        Screen::Confirm => draw_confirm(f, app),
        Screen::Summary => draw_summary(f, app),
    }

    if app.filter_draft.is_some() {
        draw_filter(f, app);
    }
    if app.confirm_empty_trash {
        draw_empty_trash(f, app);
    }
    if app.show_help {
        draw_help(f, app);
    }
}

// --------------------------------------------------------------------- chrome

fn outer(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Min(0),    // body
            Constraint::Length(1), // status
            Constraint::Length(1), // hints
        ])
        .split(area)
}

fn title_bar(app: &App, extra: &str) -> Paragraph<'static> {
    let trash = format!("trash: {} item(s)", app.trash_count);
    let text = format!("🦀 crab-clean   {extra}");
    Paragraph::new(Line::from(vec![
        Span::styled(
            text,
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(trash, Style::default().fg(app.theme.dim)),
    ]))
}

fn status_line(app: &App) -> Paragraph<'static> {
    Paragraph::new(Span::styled(
        app.status.clone(),
        Style::default().fg(app.theme.dim),
    ))
}

fn hints_line(app: &App, hints: &str) -> Paragraph<'static> {
    Paragraph::new(Span::styled(
        hints.to_string(),
        Style::default().fg(app.theme.border),
    ))
}

/// A bordered block whose appearance reflects focus: the active panel gets a
/// thick, accent-colored, bold border + title; inactive panels are dim and
/// rounded. This is the primary "where am I?" cue across the app.
fn panel_block(app: &App, title: &str, focused: bool) -> Block<'static> {
    let (border_style, border_type, title_style) = if focused {
        (
            Style::default().fg(app.theme.accent),
            BorderType::Thick,
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(app.theme.border),
            BorderType::Rounded,
            Style::default().fg(app.theme.dim),
        )
    };
    let marker = if focused { "● " } else { "  " };
    Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .title(Span::styled(format!("{marker}{title} "), title_style))
}

// ----------------------------------------------------------------------- home

fn draw_home(f: &mut Frame, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // title
            Constraint::Length(1), // current path
            Constraint::Min(0),    // body (panels)
            Constraint::Length(1), // status
            Constraint::Length(1), // hints
        ])
        .split(f.area());

    f.render_widget(title_bar(app, "interactive file cleaner"), rows[0]);

    // Current-directory breadcrumb.
    let path = Paragraph::new(Line::from(vec![
        Span::styled("  📂 ", Style::default().fg(app.theme.accent)),
        Span::styled(
            app.browse_dir.display().to_string(),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    f.render_widget(path, rows[1]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(rows[2]);

    // Left column: mode picker (top) + options (bottom).
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(body[0]);

    draw_mode_panel(f, app, left[0]);
    draw_options_panel(f, app, left[1]);
    draw_browser_panel(f, app, body[1]);

    f.render_widget(status_line(app), rows[3]);
    f.render_widget(
        hints_line(
            app,
            "Tab panel · ↑/↓ select · Enter open/scan · ←/⌫ up dir · s scan · e empty-trash · F1 help",
        ),
        rows[4],
    );
}

fn draw_mode_panel(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.home_panel == HomePanel::Mode;
    let items: Vec<ListItem> = ScanMode::ALL
        .iter()
        .map(|m| ListItem::new(m.title()))
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.mode_index));
    let list = List::new(items)
        .block(panel_block(app, "Mode", focused))
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .fg(app.theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_options_panel(f: &mut Frame, app: &App, area: Rect) {
    let focused = app.home_panel == HomePanel::Options;
    let uses_age = app.current_mode().uses_age();

    let age_val = if uses_age {
        format!("{} days  (←/→ or type)", app.age_input)
    } else {
        "(not used by this mode)".to_string()
    };
    let depth_val = if app.depth_input.trim().is_empty() {
        "unlimited  (←/→ or type)".to_string()
    } else {
        format!("{}  (←/→ or type)", app.depth_input)
    };

    let lines = vec![
        option_line(
            app,
            "Age",
            &age_val,
            focused && app.option_index == OPTION_AGE,
            false,
            false,
        ),
        option_line(
            app,
            "Max depth",
            &depth_val,
            focused && app.option_index == OPTION_DEPTH,
            false,
            false,
        ),
        option_line(
            app,
            "Include hidden",
            "",
            focused && app.option_index == OPTION_HIDDEN,
            true,
            app.include_hidden,
        ),
        option_line(
            app,
            "Follow symlinks",
            "",
            focused && app.option_index == OPTION_SYMLINKS,
            true,
            app.follow_symlinks,
        ),
        Line::from(""),
        Line::from(Span::styled(
            "  About: ",
            Style::default().fg(app.theme.dim),
        )),
        Line::from(Span::styled(
            format!("  {}", app.current_mode().description()),
            Style::default().fg(app.theme.dim),
        )),
    ];

    let form = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: true })
        .block(panel_block(app, "Options", focused));
    f.render_widget(form, area);
}

fn option_line(
    app: &App,
    label: &str,
    value: &str,
    active: bool,
    is_toggle: bool,
    on: bool,
) -> Line<'static> {
    let marker = if active { "▶ " } else { "  " };
    let label_style = if active {
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mut spans = vec![
        Span::styled(marker.to_string(), label_style),
        Span::styled(format!("{label:<16}"), label_style),
    ];
    if is_toggle {
        let check = if on { "[x]" } else { "[ ]" };
        spans.push(Span::styled(
            check.to_string(),
            Style::default().fg(app.theme.selected),
        ));
    } else {
        let value_style = if active {
            Style::default().add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(app.theme.dim)
        };
        spans.push(Span::styled(value.to_string(), value_style));
    }
    Line::from(spans)
}

fn draw_browser_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.home_panel == HomePanel::Browser;
    let items: Vec<ListItem> = app
        .browse_items
        .iter()
        .map(|item| {
            let (icon, style) = if item.is_parent {
                ("⬆ ", Style::default().fg(app.theme.dim))
            } else {
                ("📁 ", Style::default().fg(app.theme.accent))
            };
            ListItem::new(Line::from(vec![
                Span::styled(icon, style),
                Span::raw(item.label.clone()),
            ]))
        })
        .collect();

    let title = if items.is_empty() {
        "Browse  (no sub-folders — press 's' to scan here)".to_string()
    } else {
        "Browse  (Enter to open, ← to go up)".to_string()
    };

    let list = List::new(items)
        .block(panel_block(app, &title, focused))
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .fg(app.theme.highlight_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut app.browse_state);
}

// -------------------------------------------------------------------- scanning

fn draw_scanning(f: &mut Frame, app: &App) {
    let rows = outer(f.area());
    f.render_widget(title_bar(app, "scanning…"), rows[0]);

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {} {}", app.scanning_mode.title(), "scan in progress"),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("  Stage : {}", app.scan_stage)),
        Line::from(format!("  Files : {}", app.scan_count)),
    ]);
    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border)),
    );
    f.render_widget(p, rows[1]);

    f.render_widget(status_line(app), rows[2]);
    f.render_widget(hints_line(app, "Esc to cancel"), rows[3]);
}

// ---------------------------------------------------------------------- review

fn draw_review(f: &mut Frame, app: &mut App) {
    let rows = outer(f.area());

    let title = format!(
        "{}  ·  {}/{} shown  ·  {} marked ({})",
        app.scanning_mode.title(),
        app.filtered.len(),
        app.entries.len(),
        app.selected.len(),
        format_size(app.selected_total_size(), DECIMAL),
    );
    f.render_widget(title_bar(app, &title), rows[0]);

    // Build table rows.
    let header = Row::new(vec![
        Cell::from(" "),
        Cell::from("Name"),
        Cell::from("Size"),
        Cell::from("Modified"),
        Cell::from("Ext"),
        Cell::from("Grp"),
    ])
    .style(
        Style::default()
            .fg(app.theme.accent)
            .add_modifier(Modifier::BOLD),
    );

    let table_rows: Vec<Row> = app
        .filtered
        .iter()
        .map(|&i| {
            let e = &app.entries[i];
            let marked = app.selected.contains(&i);
            let marker = if marked {
                Cell::from(Span::styled("✔", Style::default().fg(app.theme.selected)))
            } else {
                Cell::from(" ")
            };
            let group = app.group_of[i].map(|g| g.to_string()).unwrap_or_default();
            let name_style = if marked {
                Style::default().fg(app.theme.selected)
            } else {
                Style::default()
            };
            Row::new(vec![
                marker,
                Cell::from(Span::styled(e.file_name(), name_style)),
                Cell::from(format_size(e.size, DECIMAL)),
                Cell::from(fmt_time(e.modified)),
                Cell::from(e.ext_str().to_string()),
                Cell::from(group),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Min(20),
        Constraint::Length(11),
        Constraint::Length(17),
        Constraint::Length(8),
        Constraint::Length(5),
    ];

    let sort_dir = if app.sort_desc { "↓" } else { "↑" };
    let filter_tag = if app.filter.is_active() {
        " · filtered"
    } else {
        ""
    };
    let table = Table::new(table_rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    " sort: {} {}{} ",
                    app.sort_key.label(),
                    sort_dir,
                    filter_tag
                ))
                .border_style(Style::default().fg(app.theme.border)),
        )
        .row_highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .fg(app.theme.highlight_fg),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(table, rows[1], &mut app.table_state);

    f.render_widget(status_line(app), rows[2]);
    f.render_widget(
        hints_line(
            app,
            "Space mark · a all · c clear · i invert · f filter · F reset · s sort · d delete · q back",
        ),
        rows[3],
    );
}

// --------------------------------------------------------------------- confirm

fn draw_confirm(f: &mut Frame, app: &App) {
    let rows = outer(f.area());
    f.render_widget(title_bar(app, "confirm deletion"), rows[0]);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(6)])
        .split(rows[1]);

    let selected = app.selected_entries();
    let area_height = body[0].height.saturating_sub(2) as usize;
    let mut items: Vec<ListItem> = selected
        .iter()
        .take(area_height.saturating_sub(1).max(1))
        .map(|e| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>10}  ", format_size(e.size, DECIMAL)),
                    Style::default().fg(app.theme.dim),
                ),
                Span::raw(e.path.display().to_string()),
            ]))
        })
        .collect();
    if selected.len() > items.len() {
        items.push(ListItem::new(Span::styled(
            format!("  …and {} more", selected.len() - items.len()),
            Style::default().fg(app.theme.dim),
        )));
    }
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} file(s) to delete ", selected.len()))
            .border_style(Style::default().fg(app.theme.border)),
    );
    f.render_widget(list, body[0]);

    let (mode_label, mode_style) = match app.delete_mode {
        DeleteMode::Trash => (
            app.delete_mode.label(),
            Style::default().fg(app.theme.selected),
        ),
        DeleteMode::Permanent => (
            app.delete_mode.label(),
            Style::default()
                .fg(app.theme.warning)
                .add_modifier(Modifier::BOLD),
        ),
    };
    let info = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::raw("Total to free: "),
            Span::styled(
                format_size(app.selected_total_size(), DECIMAL),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("Mode: "),
            Span::styled(mode_label, mode_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "y/Enter confirm   ·   t toggle trash/permanent   ·   n/Esc cancel",
            Style::default().fg(app.theme.accent),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border)),
    );
    f.render_widget(info, body[1]);

    f.render_widget(status_line(app), rows[2]);
    f.render_widget(hints_line(app, ""), rows[3]);
}

// --------------------------------------------------------------------- summary

fn draw_summary(f: &mut Frame, app: &App) {
    let rows = outer(f.area());
    f.render_widget(title_bar(app, "summary"), rows[0]);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(r) = &app.report {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("  Deleted: "),
            Span::styled(
                format!("{} file(s)", r.deleted),
                Style::default()
                    .fg(app.theme.selected)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(format!(
            "  Space freed: {}",
            format_size(r.bytes_freed, DECIMAL)
        )));
        if !r.failed.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {} failure(s):", r.failed.len()),
                Style::default().fg(app.theme.warning),
            )));
            for (path, reason) in r.failed.iter().take(10) {
                lines.push(Line::from(Span::styled(
                    format!("    {} — {}", path.display(), reason),
                    Style::default().fg(app.theme.dim),
                )));
            }
        }
    }
    let p = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Done ")
            .border_style(Style::default().fg(app.theme.border)),
    );
    f.render_widget(p, rows[1]);

    f.render_widget(status_line(app), rows[2]);
    f.render_widget(hints_line(app, "Enter / Esc to return home"), rows[3]);
}

// --------------------------------------------------------------------- popups

fn draw_filter(f: &mut Frame, app: &App) {
    let Some(draft) = &app.filter_draft else {
        return;
    };
    let area = centered_rect(70, 60, f.area());
    f.render_widget(Clear, area);

    let mut lines: Vec<Line> = Vec::new();
    for (i, label) in FILTER_LABELS.iter().enumerate() {
        let focused = draft.focus == i;
        let marker = if focused { "▶ " } else { "  " };
        let style = if focused {
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.dim)
        };
        lines.push(Line::from(Span::styled(format!("{marker}{label}"), style)));
        let val_style = if focused {
            Style::default().add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(
            format!("    {}", draft.fields[i]),
            val_style,
        )));
    }
    if let Some(err) = &draft.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("  {err}"),
            Style::default().fg(app.theme.warning),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  Tab/↑↓ move · type to edit · Enter apply · Esc cancel",
        Style::default().fg(app.theme.border),
    )));

    let p = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Filters ")
            .border_style(Style::default().fg(app.theme.accent)),
    );
    f.render_widget(p, area);
}

fn draw_empty_trash(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);
    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Empty the system trash? ({} item(s))", app.trash_count),
            Style::default()
                .fg(app.theme.warning)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  This permanently removes everything in the trash."),
        Line::from(""),
        Line::from(Span::styled(
            "  y/Enter confirm   ·   n/Esc cancel",
            Style::default().fg(app.theme.accent),
        )),
    ]);
    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Empty Trash ")
            .border_style(Style::default().fg(app.theme.warning)),
    );
    f.render_widget(p, area);
}

fn draw_help(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);
    let lines = vec![
        Line::from(Span::styled(
            "crab-clean — keys",
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Home  (Tab switches panel — the focused panel is highlighted)"),
        Line::from("  Mode panel     ↑/↓ choose what to clean"),
        Line::from("  Browse panel   ↑/↓ move · Enter/→ open folder · ←/⌫ go up · ~ home"),
        Line::from("  Options panel  ↑/↓ pick row · ←/→ adjust · Space toggle · digits type"),
        Line::from("  s              scan the current folder (Enter also scans off-browser)"),
        Line::from("  e              empty system trash"),
        Line::from(""),
        Line::from("Review"),
        Line::from("  ↑/↓ j/k    move cursor      Space  mark/unmark file"),
        Line::from("  a / c / i  all / clear / invert selection"),
        Line::from("  f          open filters     F      reset filters"),
        Line::from("  s / S      cycle sort / flip direction"),
        Line::from("  g / G      jump to top / bottom"),
        Line::from("  d / Enter  review & delete marked files"),
        Line::from("  q / Esc    back to home"),
        Line::from(""),
        Line::from("Confirm"),
        Line::from("  t          toggle trash / permanent"),
        Line::from("  y/Enter    delete    n/Esc  cancel"),
        Line::from(""),
        Line::from("Global"),
        Line::from("  F1         this help from anywhere (any key closes)"),
        Line::from("  ?          help (when not editing a text field)"),
        Line::from("  Ctrl-C     quit immediately"),
    ];
    let p = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .border_style(Style::default().fg(app.theme.accent)),
    );
    f.render_widget(p, area);
}

// --------------------------------------------------------------------- helpers

fn fmt_time(t: Option<SystemTime>) -> String {
    match t {
        Some(t) => {
            let dt: DateTime<Local> = t.into();
            dt.format("%Y-%m-%d %H:%M").to_string()
        }
        None => "—".to_string(),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
