# Changelog

All notable changes to this project will be documented in this file.

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
