# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- **Directory browser** on Home — navigate the filesystem to pick a scan target
  instead of typing a path (breadcrumb + sub-folder list; `Enter`/`→` open,
  `←`/`Backspace` up, `~` home).
- **Vim mode** — optional `hjkl` / `gg` / `G` navigation, `/` to filter, `x` to
  mark; toggleable in Settings and persisted to config.
- **Settings dialog** (`,`) — change vim mode, color theme, default delete mode,
  date basis, and show-hidden live; saved on close.
- **Modal UI redesign** — confirm/settings/filter/help/empty-trash are now
  centered dialogs over a dimmed background; an accent header bar, context
  footer of key "chips", and a per-panel focus highlight (thick accent border).
- Animated spinner on the scanning screen.

### Changed

- Delete confirmation is now a modal over the review screen rather than a
  separate page.

## [0.2.0] - 2026-06-19

### Changed (breaking)

- **Rewritten as an interactive TUI** (built on ratatui + crossterm). The
  argument/subcommand-based CLI (`duplicate` / `unused` / `group`, `--dry-run`,
  `--age`) is replaced by a keyboard-driven app launched with `crabclean`.

### Added

- **Old files** scan mode (by modification time) and **All files** browse mode,
  alongside the existing duplicate and unused scans.
- **Multi-select** review table: mark files, select all / clear / invert, sort by
  name / size / date / extension.
- **Filters**: by extension, name substring, size range, and date range.
- **Trash-based deletion** by default (recoverable) with an opt-in permanent
  mode, plus an "empty system trash" action.
- **TOML configuration** (`~/.config/crab-clean/config.toml`) for default path,
  scan depth, hidden/symlink handling, default age, time basis, delete mode, and
  color theme (Default / Ocean / Sunset / Mono).
- Background scanning/hashing so the UI stays responsive; live progress.

### Fixed

- Scanner no longer panics on permission errors / broken links (previous
  `metadata().unwrap()` calls); unreadable entries are skipped.
- Scan depth is configurable instead of hard-coded; duplicate counts and the
  unused/old distinction are reported correctly.
- Errors migrated to `thiserror`; version is sourced from `Cargo.toml`.

## [0.1.1] - 2025-06-20

### Fixed

- Updated documentation to ensure accuracy and completeness

## [0.1.0] - 2025-06-19

### Added

- Initial release
- Duplicate file detection using SHA-256 hashing
- Unused file detection based on access time
- Interactive and non-interactive deletion modes
- Cross-platform support
- Progress bars and spinners
- Dry run mode for safe preview
- Add MIT + APACHE license
- Architecture of application in ARCHITECTURE.excalidraw
