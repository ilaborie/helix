# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Development build (fast iteration)
cargo run

# Release build
cargo build --release

# Check compilation
cargo check
```

## Testing

```bash
# Unit tests and doc tests
cargo test --workspace

# Integration tests (helix-term)
cargo integration-test

# Integration tests with debug logging
HELIX_LOG_LEVEL=debug cargo integration-test

# Run a single test
cargo test --package helix-core test_name
```

## Code Quality

```bash
# Format check
cargo fmt --all --check

# Format code
cargo fmt --all

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Documentation (must compile without warnings)
cargo doc --no-deps --workspace --document-private-items
```

## Other Tasks

```bash
# Generate auto-documentation (commands, languages)
cargo xtask docgen

# Validate tree-sitter queries
cargo xtask query-check

# Validate themes
cargo xtask theme-check

# Preview user documentation
mdbook serve book
```

## Architecture

Helix is a Kakoune/Neovim-inspired modal text editor written in Rust. The codebase follows a layered architecture with functional primitives inspired by CodeMirror 6.

### Crate Structure

| Crate | Purpose |
|-------|---------|
| **helix-core** | Core editing primitives: Rope (via ropey), Selection, Transaction, Syntax (tree-sitter) |
| **helix-view** | UI abstractions: Document, View, Editor, Surface, Component system |
| **helix-term** | Terminal frontend: Application loop, commands, keymaps (builds to `hx` binary) |
| **helix-tui** | TUI library: Fork of tui-rs with double-buffer rendering |
| **helix-lsp** | Language Server Protocol client |
| **helix-dap** | Debug Adapter Protocol client |
| **helix-event** | Event system with hooks and AsyncHook for debouncing |
| **helix-loader** | Build bootstrapping: grammar fetching/building, resource loading |
| **helix-vcs** | Version control integration (git via gix) |
| **helix-stdx** | Standard library extensions |
| **helix-parsec** | Minimal parser combinator library |

### Key Concepts

- **Rope**: Main text buffer data structure (cheap to clone, enables snapshots)
- **Selection**: Multiple selections with Range (head + anchor) - core editing primitive
- **Transaction**: OT-like changes to documents; can be inverted for undo
- **Document**: Ties together Rope, Selection(s), Syntax, History, language server
- **View**: An open split in the UI, holds document ID and view-specific state
- **Component**: Widgets that render to a Surface; managed by Compositor in layers

### Runtime Resources

- `runtime/grammars/` - Tree-sitter grammar sources
- `runtime/queries/` - Syntax highlighting, indentation, textobject queries
- `runtime/themes/` - Color themes

## Debug Logging

Use `log::info!`, `log::warn!`, or `log::error!` for debug output:
```bash
# Run with logging
cargo run -- -v file.txt          # info level
cargo run -- -vv file.txt         # debug level
cargo run -- --log foo.log file.txt  # log to file
```

## MSRV

Minimum Supported Rust Version: 1.87 (follows Firefox MSRV policy)
