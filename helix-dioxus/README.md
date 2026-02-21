# helix-dioxus

A [Dioxus 0.7](https://dioxuslabs.com/) desktop frontend for the [Helix](https://helix-editor.com/) text editor.

helix-dioxus brings Helix's modal editing, tree-sitter syntax highlighting, and LSP integration to a native desktop window rendered via WebView — no terminal required.

## Quick Start

```bash
# Build
cargo build -p helix-dioxus

# Run with a file (binary is named 'dhx')
cargo run -p helix-dioxus -- src/main.rs

# Run with a directory (opens file picker)
cargo run -p helix-dioxus -- .

# Run with a glob pattern
cargo run -p helix-dioxus -- "src/*.rs"
```

## Features

- Full modal editing (Normal, Insert, Select modes)
- 100% Helix keybinding coverage
- Tree-sitter syntax highlighting
- LSP integration (diagnostics, completion, hover, code actions, rename, etc.)
- File/symbol/diagnostics/global search pickers with fuzzy filtering
- Multi-cursor / multi-selection editing
- Macro recording and dot-repeat
- Git diff gutter markers
- Theme system with live preview
- Emoji picker, file-type icons, inlay hints

## Configuration

helix-dioxus shares Helix's editor and language configuration, with an additional GUI-specific config file:

| File | Purpose |
|------|---------|
| `~/.config/helix/config.toml` | Editor settings, theme (shared with `hx`) |
| `~/.config/helix/languages.toml` | LSP and language config (shared with `hx`) |
| `~/.config/helix/dhx.toml` | Window, font, and logging settings (GUI-specific) |

### dhx.toml example

```toml
[window]
title = "My Editor"
width = 1400.0
height = 900.0

[font]
family = "'Fira Code', monospace"
size = 16.0
ligatures = true

[logging]
level = "info"
log_file = "/tmp/dhx.log"
```

## Documentation

- [KEYBINDINGS.md](KEYBINDINGS.md) — detailed comparison with standard Helix keybindings
- [DEV_LOGBOOK.md](DEV_LOGBOOK.md) — development history and decisions
- [CLAUDE.md](CLAUDE.md) — architecture reference for AI-assisted development
- [SCROLLBAR_FIX_INVESTIGATION.md](SCROLLBAR_FIX_INVESTIGATION.md) — investigation notes on a known blocked issue

## Design Decisions

- **No splits/windows**: helix-dioxus uses a single-view design. For multiple views, launch multiple instances.
- **No DAP/Debug**: Debug adapter protocol is not integrated.
- **Shared config**: Editor behavior and LSP settings are shared with the terminal `hx` binary.

## Architecture

helix-dioxus is a crate inside the helix monorepo workspace. It depends on `helix-core`, `helix-view`, `helix-lsp`, `helix-event`, `helix-loader`, `helix-vcs`, and `helix-stdx` via path dependencies.

Key architectural patterns:
- **Message passing**: `EditorContext` (non-Send/Sync) lives on the main thread; UI sends commands via mpsc channel
- **Snapshots**: `EditorSnapshot` captures editor state for rendering (Clone + Send + Sync)
- **Extension traits**: Operations organized as traits on `EditorContext` (MovementOps, EditingOps, etc.)

See [CLAUDE.md](CLAUDE.md) for the full architecture reference.
