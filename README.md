# HardBore

Fast file manager built with Rust and Svelte. Sub-150ms cold start, native OS integration, full keyboard navigation.

![Tauri v2](https://img.shields.io/badge/Tauri-v2-FFC131?style=flat-square)
![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?style=flat-square)

## Features

- **Native browser integration** - Becomes your system file picker (Linux)
- **Keyboard-first** - Full navigation without touching the mouse
- **Fast search** - SQLite FTS5 indexing with fuzzy matching
- **Multi-threaded** - Parallel directory crawling with jwalk
- **Low memory** - <25MB RAM idle
- **CLI picker mode** - Use as file picker in scripts
- **Default app integration** - Open files with system default applications

## Quick Start

```bash
npm install
npm run tauri dev
```

### Install as System File Picker (Linux Only)

```bash
cd portal
./install.sh
```

After installation, all apps (Firefox, Chrome, VSCode) **should** use HardBore for file selection.

## Stack

- **Rust** (Tauri v2) - Backend, file operations
- **SvelteKit** - Reactive UI
- **SQLite FTS5** - Full-text search indexing

## Performance

Real-world benchmarks on Arch Linux (tested with system directories):

| Operation | Throughput | Details |
|-----------|-----------|---------|
| **Single dir read** | 709,000 items/sec | 2,820 files in 3ms |
| **Recursive crawl** | 424,000 items/sec | 38,214 files across /usr (depth 3) |
| **Indexing** | 19,663 files/sec | 47,225 files with SQLite FTS5 |
| **FTS5 search** | 1.7ms avg | Sub-2ms queries across 47K indexed files |
| **Cold start** | 2ms | DB open + first search |
| **Large directory** | 320,000 items/sec | 9,906 files from /usr/lib |

Multi-threaded crawling with jwalk + rayon. Optimized SQLite inserts with prepared statements and batch commits.

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `Ctrl+P` | Command palette / search |
| `j` `k` | Navigate up/down |
| `h` `l` | Parent / enter directory |
| `Enter` | Open file with default app / enter directory |
| `p` | Toggle preview |
| `b` | Toggle sidebar |
| `Ctrl+H` | Toggle hidden files |
| `Alt+←` `→` | History navigation |

**Mouse:** Double-click files to open with default application. Right-click for context menu.

## CLI Picker Mode

Use HardBore in shell scripts:

```bash
hardbore --picker                   # Select file
hardbore --picker --multiple        # Multiple files
hardbore --picker-dirs              # Select directory
hardbore --picker --types png,jpg   # Filter by extension
```

Output: `HARDBORE_SELECTED:/path/to/file`

## Build

**Requirements:** Rust, Node.js 18+, webkit2gtk

**Arch:**
```bash
sudo pacman -S webkit2gtk base-devel
```

**Ubuntu/Debian:**
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential
```

**Build:**
```bash
npm install
npm run tauri dev    # Development
npm run tauri build  # Production
```

## Structure

```
src/          SvelteKit frontend
src-tauri/    Rust backend + file operations
portal/       XDG Desktop Portal integration
docs/         Documentation
```

## Architecture

- **fs_engine.rs** - Multi-threaded directory crawling
- **indexer.rs**   - Background SQLite FTS5 indexing
- **portal.rs**    - D-Bus file chooser interface

## License

AGPL-3.0
