//! All rendering. Pure functions of `&App` → terminal frame.
//!
//! Visual language (inspired by lazygit / k9s / yazi):
//! - A solid accent header bar with right-aligned context.
//! - A context-sensitive footer of key "chips".
//! - The focused panel is drawn with a thick accent border; others are dim.
//! - Yes/no prompts and forms are centered modals over a dimmed background.

use std::time::SystemTime;

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Padding, Paragraph, Row,
    Table, Wrap,
};

use crate::core::delete::DeleteMode;
use crate::core::model::ScanMode;
use crate::tui::app::{
    App, FILTER_LABELS, HomePanel, OPTION_AGE, OPTION_DEPTH, OPTION_HIDDEN, OPTION_SYMLINKS,
    SETTING_BASIS, SETTING_DELETE, SETTING_HIDDEN, SETTING_THEME, SETTING_VIM, Screen,
};

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render(f: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Home => draw_home(f, app),
        Screen::Scanning => draw_scanning(f, app),
        Screen::Review => draw_review(f, app),
        Screen::Summary => draw_summary(f, app),
    }

    // Modals, drawn over the (now dimmed) screen. Only one is active at a time.
    if app.confirm_delete {
        draw_confirm(f, app);
    }
    if app.show_settings {
        draw_settings(f, app);
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

/// Standard page layout: header line, body, status line, footer line.
fn chrome(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(0),    // body
            Constraint::Length(1), // status
            Constraint::Length(1), // footer
        ])
        .split(area)
}

fn header_bar(f: &mut Frame, app: &App, area: Rect, subtitle: &str) {
    let bar = Style::default()
        .bg(app.theme.accent)
        .fg(app.theme.highlight_fg);
    let right = right_context(app);
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(right.len() as u16 + 2),
        ])
        .split(area);

    let left = Paragraph::new(Line::from(vec![
        Span::styled(" 🦀 crab-clean ", bar.add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {subtitle}"), bar),
    ]))
    .style(bar);
    f.render_widget(left, cols[0]);

    let right_p = Paragraph::new(Span::styled(format!("{right} "), bar))
        .alignment(Alignment::Right)
        .style(bar);
    f.render_widget(right_p, cols[1]);
}

fn right_context(app: &App) -> String {
    let mut s = format!("🗑 {}", app.trash_count);
    if app.vim_mode {
        s.push_str("  VIM");
    }
    s
}

fn status_line(app: &App, area: Rect, f: &mut Frame) {
    let p = Paragraph::new(Span::styled(
        format!(" {}", app.status),
        Style::default().fg(app.theme.dim),
    ));
    f.render_widget(p, area);
}

/// Render a footer of `[key] desc` chips.
fn footer_bar(f: &mut Frame, app: &App, area: Rect, chips: &[(&str, &str)]) {
    let chip_key = Style::default()
        .bg(app.theme.accent)
        .fg(app.theme.highlight_fg)
        .add_modifier(Modifier::BOLD);
    let desc = Style::default().fg(app.theme.dim);

    let mut spans = vec![Span::raw(" ")];
    for (key, text) in chips {
        spans.push(Span::styled(format!(" {key} "), chip_key));
        spans.push(Span::styled(format!(" {text}  "), desc));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// A bordered block whose appearance reflects focus.
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
        .padding(Padding::horizontal(1))
        .title(Span::styled(format!("{marker}{title} "), title_style))
}

/// Dim the whole screen, then carve out and frame a centered modal. Returns the
/// inner content area.
fn open_modal(f: &mut Frame, app: &App, w_pct: u16, h_pct: u16, title: &str) -> Rect {
    // Recolor existing content to a dim tone (true transparency isn't possible).
    f.render_widget(
        Block::default().style(
            Style::default()
                .fg(app.theme.border)
                .add_modifier(Modifier::DIM),
        ),
        f.area(),
    );
    let area = centered_rect(w_pct, h_pct, f.area());
    f.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(app.theme.accent))
        .padding(Padding::new(2, 2, 1, 1))
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    f.render_widget(block, area);
    inner
}

// ----------------------------------------------------------------------- home

fn draw_home(f: &mut Frame, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(1), // breadcrumb
            Constraint::Min(0),    // panels
            Constraint::Length(1), // status
            Constraint::Length(1), // footer
        ])
        .split(f.area());

    header_bar(f, app, rows[0], "interactive file cleaner");

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

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(0)])
        .split(body[0]);

    draw_mode_panel(f, app, left[0]);
    draw_options_panel(f, app, left[1]);
    draw_browser_panel(f, app, body[1]);

    status_line(app, rows[3], f);
    footer_bar(
        f,
        app,
        rows[4],
        &[
            ("Tab", "panel"),
            ("↑↓", "move"),
            ("⏎", "open"),
            ("s", "scan"),
            (",", "settings"),
            ("e", "trash"),
            ("?", "help"),
            ("q", "quit"),
        ],
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
        format!("{} days", app.age_input)
    } else {
        "(not used by this mode)".to_string()
    };
    let depth_val = if app.depth_input.trim().is_empty() {
        "unlimited".to_string()
    } else {
        app.depth_input.clone()
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
            app.current_mode().description(),
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
        let (mark, color) = if on {
            ("◉ on", app.theme.selected)
        } else {
            ("○ off", app.theme.dim)
        };
        spans.push(Span::styled(mark.to_string(), Style::default().fg(color)));
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
        "Browse  (no sub-folders — press s to scan here)".to_string()
    } else {
        "Browse  (⏎ open · ← up)".to_string()
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
    let rows = chrome(f.area());
    header_bar(f, app, rows[0], "scanning…");

    let card = centered_rect(60, 40, rows[1]);
    let spin = SPINNER[(app.frame as usize) % SPINNER.len()];
    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("   {spin}  {}", app.scanning_mode.title()),
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("   {}", app.scan_stage),
            Style::default().fg(app.theme.dim),
        )),
        Line::from(Span::styled(
            format!("   {} files", app.scan_count),
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]);
    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.accent))
            .padding(Padding::new(1, 1, 0, 0)),
    );
    f.render_widget(p, card);

    status_line(app, rows[2], f);
    footer_bar(f, app, rows[3], &[("Esc", "cancel")]);
}

// ---------------------------------------------------------------------- review

fn draw_review(f: &mut Frame, app: &mut App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(1), // stats bar
            Constraint::Min(0),    // table
            Constraint::Length(1), // status
            Constraint::Length(1), // footer
        ])
        .split(f.area());

    header_bar(f, app, rows[0], app.scanning_mode.title());

    // Stats bar.
    let sort_dir = if app.sort_desc { "↓" } else { "↑" };
    let filter = if app.filter.is_active() { "on" } else { "off" };
    let num = Style::default()
        .fg(app.theme.accent)
        .add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(app.theme.dim);
    let stats = Paragraph::new(Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{}", app.entries.len()), num),
        Span::styled(" files · ", dim),
        Span::styled(format!("{}", app.filtered.len()), num),
        Span::styled(" shown · ", dim),
        Span::styled(
            format!("{}", app.selected.len()),
            Style::default()
                .fg(app.theme.selected)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                " marked ({}) · ",
                format_size(app.selected_total_size(), DECIMAL)
            ),
            dim,
        ),
        Span::styled(format!("sort {}{} · ", app.sort_key.label(), sort_dir), dim),
        Span::styled(format!("filter {filter}"), dim),
    ]));
    f.render_widget(stats, rows[1]);

    // Table.
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
                Cell::from(Span::styled("◉", Style::default().fg(app.theme.selected)))
            } else {
                Cell::from(Span::styled("○", Style::default().fg(app.theme.border)))
            };
            let group = app.group_of[i].map(|g| g.to_string()).unwrap_or_default();
            let name_style = if marked {
                Style::default()
                    .fg(app.theme.selected)
                    .add_modifier(Modifier::BOLD)
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

    let table = Table::new(table_rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(app.theme.accent))
                .padding(Padding::horizontal(1)),
        )
        .row_highlight_style(
            Style::default()
                .bg(app.theme.highlight_bg)
                .fg(app.theme.highlight_fg),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(table, rows[2], &mut app.table_state);

    status_line(app, rows[3], f);
    footer_bar(
        f,
        app,
        rows[4],
        &[
            ("Space", "mark"),
            ("a/c/i", "sel"),
            ("f", "filter"),
            ("s/S", "sort"),
            ("d", "delete"),
            (",", "settings"),
            ("q", "back"),
        ],
    );
}

// --------------------------------------------------------------------- summary

fn draw_summary(f: &mut Frame, app: &App) {
    let rows = chrome(f.area());
    header_bar(f, app, rows[0], "summary");

    let card = centered_rect(70, 60, rows[1]);
    f.render_widget(Clear, card);

    let mut lines: Vec<Line> = vec![Line::from("")];
    if let Some(r) = &app.report {
        lines.push(Line::from(vec![
            Span::raw("  ✔ Deleted "),
            Span::styled(
                format!("{} file(s)", r.deleted),
                Style::default()
                    .fg(app.theme.selected)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::raw("  💾 Freed   "),
            Span::styled(
                format_size(r.bytes_freed, DECIMAL),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ]));
        if !r.failed.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  ✖ {} failure(s):", r.failed.len()),
                Style::default().fg(app.theme.warning),
            )));
            for (path, reason) in r.failed.iter().take(8) {
                lines.push(Line::from(Span::styled(
                    format!("     {} — {}", path.display(), reason),
                    Style::default().fg(app.theme.dim),
                )));
            }
        }
    }

    let p = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(app.theme.accent))
            .padding(Padding::new(1, 1, 0, 0))
            .title(Span::styled(
                " Done ",
                Style::default()
                    .fg(app.theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    f.render_widget(p, card);

    status_line(app, rows[2], f);
    footer_bar(f, app, rows[3], &[("⏎", "back to home")]);
}

// ---------------------------------------------------------------------- modals

fn draw_confirm(f: &mut Frame, app: &App) {
    let inner = open_modal(f, app, 72, 70, "Confirm deletion");

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(4)])
        .split(inner);

    let selected = app.selected_entries();
    let capacity = parts[0].height.saturating_sub(1).max(1) as usize;
    let mut items: Vec<ListItem> = selected
        .iter()
        .take(capacity)
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
            format!("…and {} more", selected.len() - items.len()),
            Style::default().fg(app.theme.dim),
        )));
    }
    f.render_widget(
        List::new(items).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(app.theme.border))
                .title(Span::styled(
                    format!("{} file(s) selected", selected.len()),
                    Style::default().fg(app.theme.dim),
                )),
        ),
        parts[0],
    );

    let mode_style = match app.delete_mode {
        DeleteMode::Trash => Style::default().fg(app.theme.selected),
        DeleteMode::Permanent => Style::default()
            .fg(app.theme.warning)
            .add_modifier(Modifier::BOLD),
    };
    let info = Paragraph::new(Text::from(vec![
        Line::from(vec![
            Span::raw("Free up "),
            Span::styled(
                format_size(app.selected_total_size(), DECIMAL),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("   ·   Mode: "),
            Span::styled(app.delete_mode.label(), mode_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "y/⏎ confirm    t toggle trash/permanent    n/Esc cancel",
            Style::default().fg(app.theme.accent),
        )),
    ]));
    f.render_widget(info, parts[1]);
}

fn draw_settings(f: &mut Frame, app: &App) {
    let inner = open_modal(f, app, 60, 55, "Settings");

    let rows = [
        ("Vim navigation", on_off(app.config.vim_mode)),
        ("Color theme", app.config.theme.label().to_string()),
        (
            "Default delete mode",
            app.config.delete_mode.short_label().to_string(),
        ),
        (
            "Date basis (filters)",
            app.config.time_basis.label().to_string(),
        ),
        ("Show hidden files", on_off(app.config.include_hidden)),
    ];
    let active = [
        SETTING_VIM,
        SETTING_THEME,
        SETTING_DELETE,
        SETTING_BASIS,
        SETTING_HIDDEN,
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (i, (label, value)) in rows.iter().enumerate() {
        let focused = app.settings_index == active[i];
        let marker = if focused { "▶ " } else { "  " };
        let label_style = if focused {
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let value_style = if focused {
            Style::default()
                .fg(app.theme.selected)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.dim)
        };
        lines.push(Line::from(vec![
            Span::styled(marker.to_string(), label_style),
            Span::styled(format!("{label:<22}"), label_style),
            Span::styled(format!("< {value} >"), value_style),
        ]));
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(
        "↑/↓ choose   ←/→/Space change   Esc close & save",
        Style::default().fg(app.theme.dim),
    )));

    f.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn on_off(v: bool) -> String {
    if v {
        "on".to_string()
    } else {
        "off".to_string()
    }
}

fn draw_filter(f: &mut Frame, app: &App) {
    let Some(draft) = &app.filter_draft else {
        return;
    };
    let inner = open_modal(f, app, 70, 70, "Filters");

    let mut lines: Vec<Line> = Vec::new();
    for (i, label) in FILTER_LABELS.iter().enumerate() {
        let focused = draft.focus == i;
        let marker = if focused { "▶ " } else { "  " };
        let label_style = if focused {
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.dim)
        };
        lines.push(Line::from(Span::styled(
            format!("{marker}{label}"),
            label_style,
        )));
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
            format!("  ⚠ {err}"),
            Style::default().fg(app.theme.warning),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Tab/↑↓ move    type to edit    ⏎ apply    Esc cancel",
        Style::default().fg(app.theme.dim),
    )));

    f.render_widget(Paragraph::new(Text::from(lines)), inner);
}

fn draw_empty_trash(f: &mut Frame, app: &App) {
    let inner = open_modal(f, app, 52, 34, "Empty Trash");
    let text = Text::from(vec![
        Line::from(Span::styled(
            format!("Permanently delete {} trashed item(s)?", app.trash_count),
            Style::default()
                .fg(app.theme.warning)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "This cannot be undone.",
            Style::default().fg(app.theme.dim),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "y/⏎ confirm    n/Esc cancel",
            Style::default().fg(app.theme.accent),
        )),
    ]);
    f.render_widget(Paragraph::new(text), inner);
}

fn draw_help(f: &mut Frame, app: &App) {
    let inner = open_modal(f, app, 74, 86, "Help & Keys");
    let nav = if app.vim_mode {
        "h/j/k/l move · gg/G top/bottom · arrows also work"
    } else {
        "↑/↓ move · g/G top/bottom (enable Vim in settings for hjkl)"
    };
    let lines = vec![
        Line::from(Span::styled(
            format!("Navigation: {nav}"),
            Style::default().fg(app.theme.accent),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Home",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Tab          switch panel (Mode · Browse · Options)"),
        Line::from("  Browse       ⏎/→ open folder · ←/⌫ up · ~ home"),
        Line::from("  Options      ←/→ adjust · Space toggle · digits type"),
        Line::from("  s            scan the current folder"),
        Line::from("  e            empty the system trash"),
        Line::from(""),
        Line::from(Span::styled(
            "Review",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Space / x    mark / unmark file"),
        Line::from("  a / c / i    select all / clear / invert"),
        Line::from("  f or /       filters       F   reset filters"),
        Line::from("  s / S        sort key / direction"),
        Line::from("  d or ⏎       delete marked files"),
        Line::from(""),
        Line::from(Span::styled(
            "Global",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  ,            settings (Vim mode, theme, …)"),
        Line::from("  F1 / ?       this help        Ctrl-C  quit"),
        Line::from(""),
        Line::from(Span::styled(
            "press any key to close",
            Style::default().fg(app.theme.dim),
        )),
    ];
    f.render_widget(Paragraph::new(Text::from(lines)), inner);
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
