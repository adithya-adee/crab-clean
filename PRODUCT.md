# Declutter Product Docs

## 1. Elevator Pitch

Declutter CLI is a Rust‑powered command‑line tool that helps developers and everyday users quickly clean up unused, duplicate, and scattered files in any directory—automating smart grouping and safe deletion to keep your workspace lean and organized.

---

## 2. Who Is This App For

- **Developers** working with large codebases, build artifacts, temp files, or multiple project folders
- **Digital power users** who accumulate downloads, archives, and media over time
- **Anyone** who wants a fast, safe way to reclaim disk space and reduce cognitive load from a messy file system

---

## 3. Functional Requirements

1.  **Duplicate Detection & Deletion**

    - Identify exact duplicate files by content hash (e.g., SHA‑256).
    - Prompt user for confirmation before deleting any duplicate.
    - Support for partial content hashing to speed up large file comparison.

2.  **"Unused" File Cleanup**

    - Default threshold: files not modified in the last 30 days.
    - User can override via `--age DAYS` flag or in a config file.
    - Optional tracking of access times in addition to modification times.

3.  **Smart Grouping Modes**

    - **By Extension**: group all `.pdf`, `.jpg`, etc.
    - **By Name Similarity**: detect files with common basename patterns.
    - **By Metadata**: group by file size ranges or timestamps.
    - Mode selectable via `--mode {extension|name|metadata}`; multiple modes combinable.

4.  **Configuration Management**

    - **CLI flags** for quick overrides (`--path`, `--age`, `--mode`, `--dry-run`).
    - **Subcommand structure** for clear separation of concerns:
      ```
      declutter duplicate [path] [options]
      declutter unused [path] --age [days] [options]
      declutter group [path] --mode [modes] [options]
      ```
    - **Optional config file** (`~/.declutter.toml`) for default profiles:
      ```toml
      [defaults]
      age = 30
      modes = ["extension","name"]
      confirm = true
      ```

5.  **Interactive Safety & Feedback**

    - Display a **tree view** of detected groups before any action.
    - Show **progress bar** for long scans.
    - Ask explicit **Y/N** confirmation per group or file.
    - Support a `--dry-run` mode to preview changes without deleting.
    - Detailed error reporting with context-aware messages.

6.  **Platform Support (MVP)**
    - Linux (extensible later to macOS/Windows).
    - File system abstraction layer to ease cross-platform support in future.

---

## 4. User Stories

- **As a developer**, I want to run `declutter unused --path ./project --age 60 --dry-run` to preview unused files older than 60 days so I can review before cleanup.
- **As a digital user**, I want to run `declutter group --mode extension --path ./Pictures` to group all my `.jpg` and `.png` images into visual clusters so I can decide which albums to keep.
- **As a power user**, I want to configure my default modes in `~/.declutter.toml` so I don't have to type flags every time.
- **As a cautious user**, I want a clear tree view and explicit confirmations so I never accidentally delete important files.
- **As a regular user**, I want to run `declutter duplicate` in my current directory to find and safely remove duplicate files.

---

## 5. User Interface

- **CLI-Only**: No GUI; leverages Rust's ecosystem for fast startup and low overhead.
- **Clear, Colorized Output**:
  - Green for "safe to delete," yellow for "review," red for "skipped."
- **Tree Structure Display**: Indented file groups with counts and sizes.
- **Progress Bar**: Shows scanning progress when traversing large directories.
- **Interactive Prompts**:
  ```bash
  [Group: Duplicate Images (5 files, 120 MB)]
    1) img001.png (keep)
    2) img001_copy.png (delete)
    3) ...
  Delete this group? (y/N)
  ```
- **Error Handling**: Friendly messages if permissions are insufficient or paths not found.
- **Consistent Output Format**: Each command follows the same output pattern for user familiarity.

---

## 6. Non-Functional Requirements

### Performance

- **Scan Speed**: The tool should be highly efficient, scanning large directories (e.g., 10,000 files) in seconds, not minutes.
- **Memory Usage**: Minimize memory footprint, especially when dealing with large numbers of files, by processing file paths and metadata iteratively rather than loading all into memory.
- **Hashing Speed**: Implement efficient hashing algorithms (e.g., buffered reads) for duplicate detection to avoid performance bottlenecks.
- **Parallel Processing**: Utilize multi-threading for file scanning and hashing operations where appropriate.

### Reliability & Safety

- **Atomic Operations**: Ensure file deletion and movement operations are as atomic as possible, preventing data corruption or partial deletion in case of interruptions.
- **Permissions Handling**: Gracefully handle file system permission errors without crashing, providing informative messages to the user.
- **Data Integrity**: Never modify original files without explicit user confirmation.
- **Error Propagation**: Structured error handling chain that preserves context through all operations.
- **Undo Mechanism**: (Future consideration, but important for reliability) A log of operations to allow potential rollback.

### Usability

- **Intuitive CLI**: Command syntax should be clear, concise, and follow common CLI patterns.
- **Clear Feedback**: Users should always understand what the tool is doing, why it's suggesting certain actions, and what the outcome of their choices will be.
- **Readability**: Output should be easy to parse and understand, even for users not familiar with the command line.
- **Consistent Command Structure**: All subcommands follow the same pattern for argument parsing and output.

### Maintainability

- **Modular Architecture**: Code divided into clear layers:
  - CLI interface layer (command parsing, user interaction)
  - Core business logic (file scanning, duplicate detection)
  - Configuration management
  - File system utilities
  - Error handling
- **Comprehensive Testing**: Include unit and integration tests for core functionalities (e.g., file scanning, hashing, configuration parsing).
- **Clear Documentation**: Maintain well-commented code and a clear `README.md` for easy onboarding of new contributors.
- **Rust Idioms**: Adhere to idiomatic Rust practices for robustness and readability.

---

## 7. Technical Architecture

### Component Structure

- **CLI Layer**: Handles argument parsing and command routing

  - Uses clap for type-safe command definition
  - Provides help text and usage examples
  - Routes to appropriate command handlers

- **Core Components**:
  - **Scanner**: Discovers files and extracts metadata
  - **Analyzer**: Identifies duplicates and unused files
  - **Organizer**: Handles file operations (deletion, grouping)
- **Configuration Management**:

  - Default settings with override hierarchy
  - Config file support (TOML format)
  - Command-line argument priority

- **Error Handling**:
  - Custom error types with context
  - Friendly user-facing messages
  - Detailed logging for debugging

---

## 8. Future Enhancements (Out of Scope for MVP)

- **macOS and Windows Support**: Extend platform compatibility beyond Linux.
- **Undo Functionality**: Implement a command to revert the last set of changes made by `declutter`.
- **"Watch" Mode**: Continuously monitor a directory for new files and suggest actions in the background.
- **Exclusion Rules**: Allow users to define specific files or directories to ignore during scans.
- **Integration with System Clipboard**: Ability to copy file paths or names from the output directly to the clipboard.
- **Machine Learning (Basic)**: Over time, the tool could learn user preferences to make more accurate and personalized suggestions.
- **GUI (Long-Term)**: A simple graphical interface could be explored for broader appeal, leveraging Rust's GUI frameworks.
- **Cloud Sync Integration**: Basic integration with cloud storage services (e.g., Dropbox, Google Drive) to organize synced folders.
- **Plugin System**: Allow users to extend functionality with custom rules and actions.

---

## 9. Success Metrics

- **Crates.io Downloads**: Number of downloads indicates adoption and interest.
- **GitHub Stars/Forks**: Measures community engagement and project visibility.
- **User Feedback/Issues**: Volume and nature of bug reports and feature requests.
- **Disk Space Reclaimed**: Anecdotal or user-reported figures on how much space the tool helped them save.
- **Performance Benchmarks**: Consistent adherence to scan speed and memory usage targets.
- **Code Quality Metrics**: Maintainability score, test coverage, and static analysis results.

---
