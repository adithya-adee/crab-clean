# crabclean CLI

[![Crates.io](https://img.shields.io/crates/v/crabclean.svg)](https://crates.io/crates/crabclean)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)
[![Build Status](https://github.com/your-username/crabclean/workflows/CI/badge.svg)](https://github.com/your-username/crabclean/actions)

> crabclean CLI is a Rust‑powered command‑line tool that helps developers and everyday users quickly clean up unused, duplicate, and scattered files in any directory—automating smart grouping and safe deletion to keep your workspace lean and organized.

## Features

- **🔍 Duplicate File Detection**: Identifies exact duplicate files using SHA-256 content hashing
- **⏰ Unused File Cleanup**: Finds files that haven't been accessed for a specified number of days
- **🎯 Interactive Deletion**: Safe, user-confirmed deletion with progress tracking
- **⚡ High Performance**: Multi-threaded scanning and hashing using Rayon
- **🛡️ Cross-Platform**: Works on Linux, macOS, and Windows
- **📊 Progress Visualization**: Real-time progress bars and spinners
- **🔄 Dry Run Mode**: Preview operations without making changes

## Installation

### From crates.io (Recommended)

```bash
cargo install crabclean
```

### From Source

```bash
git clone https://github.com/your-username/crabclean.git
cd crabclean
cargo install --path .
```

### Pre-compiled Binaries

Download pre-compiled binaries from the [Releases page](https://github.com/your-username/crabclean/releases).

## Quick Start

### Find Duplicate Files

```bash
# Dry run (preview only)
crabclean duplicate /path/to/directory --dry-run

# Interactive deletion
crabclean duplicate /path/to/directory

# Current directory
crabclean duplicate .
```

### Find Unused Files

```bash
# Find files unused for 30 days (default)
crabclean unused /path/to/directory --dry-run

# Find files unused for 60 days
crabclean unused /path/to/directory --age 60

# Interactive deletion
crabclean unused /path/to/directory --age 30
```

## Usage

```text
crabclean your file system by finding and managing duplicate and unused files

Usage: crabclean <COMMAND> <SOURCE_DIRECTORY> <flag>

Commands:
  duplicate  Find and manage duplicate files
  unused     Find and manage unused files
  help       Print this message or the help of the given subcommand(s)

Flag:
  --dry-run / -n To just know details
  without args   You will be prompted to ask for delete (Press ctrl + c to exit the terminal , only if you don't want to delete)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Duplicate Command

```
Find and manage duplicate files

Usage: crabclean duplicate [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to the directory to scan [default: .]

Options:
  -n, --dry-run  Perform a dry run without deleting files
  -h, --help     Print help
```

### Unused Command

```
Find and manage unused files

Usage: crabclean unused [OPTIONS] [PATH]

Arguments:
  [PATH]  Path to the directory to scan [default: .]

Options:
  -a, --age <AGE>  Age in days for a file to be considered unused [default: 30]
  -n, --dry-run    Perform a dry run without deleting files
  -h, --help       Print help
```

## Examples

```bash
# Find duplicates in Downloads folder (dry run)
crabclean duplicate ~/Downloads --dry-run

# Clean up unused files older than 90 days in project directory
crabclean unused ~/projects --age 90

# Interactive duplicate cleanup in current directory
crabclean duplicate .
```

## Safety Features

- **Dry run by default**: Use `--dry-run` to preview changes
- **Interactive confirmation**: Each file deletion requires user confirmation
- **Progress tracking**: Visual feedback during long operations
- **Error handling**: Graceful error reporting and recovery

## Performance

- **Multi-threaded**: Uses Rayon for parallel file processing
- **Efficient hashing**: SHA-256 with optimized buffer sizes
- **Smart grouping**: Files are first grouped by size before hashing
- **Memory efficient**: Streaming file processing for large files

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for a list of changes in each version.
