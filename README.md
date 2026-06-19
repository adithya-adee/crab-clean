![Crab Clean Logo](crab_clean_logo.png)

# Crab Clean

[![Crates.io](https://img.shields.io/badge/crates_io-blue.svg)](https://crates.io/crates/crab-clean)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

> Crab Clean is a Rust-powered **interactive terminal UI** (built with [ratatui](https://ratatui.rs)) that helps you find and safely remove **duplicate**, **old**, and **unused** files in any directory — with rich filtering, multi-select, and recoverable trash-based deletion.

## Features

- 🖥️ **Full TUI** — no command-line flags to memorize; everything is driven from the keyboard inside the app.
- 🔍 **Duplicate detection** — exact duplicates via size bucketing + SHA-256, hashed in parallel with Rayon. Groups are shown together and all-but-the-newest is pre-selected.
- ⏳ **Old & unused files** — find files not *modified* (old) or not *accessed* (unused) for N days.
- 🗂️ **Browse-all mode** — list every file and narrow it down with filters.
- 🎛️ **Powerful filters** — by extension, name substring, size range, and date range (`YYYY-MM-DD`).
- ✅ **Multi-select** — mark individual files, select all / clear / invert, sort by name/size/date/extension.
- 🗑️ **Safe by default** — deletions go to the OS trash (recoverable). Toggle to permanent per-action. Empty the system trash from inside the app.
- ⚙️ **Customizable** — a TOML config for default path, scan depth, hidden/symlink handling, default age, time basis, delete mode, and color theme.
- ⚡ **Responsive** — scanning and hashing run on a background thread so the UI never freezes.

## Installation

### From crates.io

```bash
cargo install crab-clean
```

### From source

```bash
git clone https://github.com/adithya-adee/crab-clean.git
cd crab-clean
cargo install --path .
```

This installs the `crabclean` binary onto your `PATH` (`~/.cargo/bin`).

## Usage

Just launch it — there are no subcommands or flags:

```bash
crabclean
```

You land on the **Home** screen. Pick a mode, set the directory and options, then press **Enter** to scan.

### Key bindings

**Home**

| Key | Action |
| --- | --- |
| `↑` / `↓` | Choose scan mode (Duplicates / Unused / Old / All files) |
| `Tab` | Move between Path / Age / Max-depth / toggles |
| `Space` | Toggle a checkbox (include hidden, follow symlinks) |
| `Enter` | Start the scan |
| `e` | Empty the system trash |
| `F1` | Help (works anywhere) |
| `Esc` | Quit |

**Review**

| Key | Action |
| --- | --- |
| `↑`/`↓` or `j`/`k` | Move cursor |
| `Space` | Mark / unmark a file for deletion |
| `a` / `c` / `i` | Select all / clear / invert |
| `f` / `F` | Open filters / reset filters |
| `s` / `S` | Cycle sort key / flip direction |
| `g` / `G` | Jump to top / bottom |
| `d` or `Enter` | Review & delete the marked files |
| `q` / `Esc` | Back to Home |

**Confirm**

| Key | Action |
| --- | --- |
| `t` | Toggle Trash ↔ Permanent |
| `y` / `Enter` | Delete |
| `n` / `Esc` | Cancel |

## Configuration

Crab Clean reads an optional TOML file (defaults are used if it's absent):

- Linux: `~/.config/crab-clean/config.toml`
- macOS: `~/Library/Application Support/crab-clean/config.toml`
- Windows: `%APPDATA%\crab-clean\config.toml`

```toml
default_path = "."
max_depth = 8            # omit / null for unlimited
include_hidden = false
follow_symlinks = false
default_age_days = 30
time_basis = "Modified"  # Modified | Accessed | Created  (used by date filters)
delete_mode = "Trash"    # Trash | Permanent
theme = "Default"        # Default | Ocean | Sunset | Mono
```

## Safety

- **Trash by default** — files go to your OS recycle bin and can be restored. Permanent deletion is opt-in per action.
- **Explicit confirmation** — nothing is deleted until you review the exact list and confirm.
- **Robust scanning** — unreadable files/dirs are skipped instead of crashing the scan.

## Library use

The detection logic is also usable as a library:

```rust
use crab_clean::core::scanner::{scan, ScanOptions};
use crab_clean::core::algorithms::find_duplicates;
use std::path::Path;

let files = scan(Path::new("."), &ScanOptions::default(), |_| {}).unwrap();
let groups = find_duplicates(&files);
println!("found {} duplicate group(s)", groups.len());
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Changelog

See [CHANGELOG.md](CHANGELOG.md).
