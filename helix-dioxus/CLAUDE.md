# helix-dioxus CLAUDE.md

This file provides guidance to Claude Code when working with the helix-dioxus crate.

## Architecture

helix-dioxus is a Dioxus 0.7 desktop frontend for the Helix text editor.

### Key Architectural Pattern

- **Message Passing**: EditorContext (non-Send/Sync) lives on main thread
- **Commands**: UI sends EditorCommand via mpsc channel
- **Snapshots**: EditorSnapshot captures state for rendering (Clone + Send + Sync)
- **Thread-local**: EDITOR_CTX provides synchronous access for immediate updates

### Module Structure

```
helix-dioxus/src/
├── main.rs                     # Entry point (anemic: tracing, helix loader, args, launch)
├── lib.rs                      # Library root: launch(), AppState, module declarations
├── app.rs                      # Root App component
├── args.rs                     # Command-line argument parsing
├── tracing.rs                  # Logging configuration
│
├── components/                 # UI Components
│   ├── mod.rs                  # Re-exports
│   ├── editor_view.rs          # Document rendering with syntax highlighting
│   ├── buffer_bar.rs           # Tab bar with scroll buttons
│   ├── statusline.rs           # Mode, filename, position display
│   ├── picker/                 # Picker components
│   │   ├── mod.rs              # Re-exports GenericPicker
│   │   ├── generic.rs          # Main picker container
│   │   ├── item.rs             # PickerItemRow component
│   │   └── highlight.rs        # HighlightedText for fuzzy matches
│   └── prompt.rs               # Command/search prompts
│
├── state/                      # State Management
│   ├── mod.rs                  # EditorContext, command dispatch
│   ├── types.rs                # Data structures (EditorSnapshot, etc.)
│   └── lsp_events.rs           # LspEventOps: poll_lsp_events, diagnostics handling
│
├── operations/                 # Editor Operations (extension traits)
│   ├── mod.rs                  # Re-exports all traits
│   ├── movement.rs             # MovementOps: move_cursor, goto_*
│   ├── editing.rs              # EditingOps: insert_*, delete_*, undo/redo
│   ├── selection.rs            # SelectionOps: extend_*, select_line
│   ├── clipboard.rs            # ClipboardOps: yank, paste, delete_selection
│   ├── search.rs               # SearchOps: execute_search, search_next
│   ├── picker_ops.rs           # PickerOps: show_*_picker, picker_confirm
│   ├── buffer.rs               # BufferOps: switch_to_buffer, save_document
│   └── cli.rs                  # CliOps: execute_command
│
└── keybindings/                # Key Handling
    ├── mod.rs                  # Re-exports handlers
    ├── translate.rs            # Dioxus KeyboardEvent → helix KeyEvent translation
    ├── normal.rs               # handle_normal_mode
    ├── insert.rs               # handle_insert_mode
    ├── select.rs               # handle_select_mode
    ├── command.rs              # handle_command_mode
    ├── picker.rs               # handle_picker_mode
    └── search.rs               # handle_search_mode

helix-dioxus/assets/
├── styles.css                  # Main stylesheet (loaded via document::Stylesheet)
└── script.js                   # JavaScript functions (loaded via custom head)
```

### Dioxus 0.7 Patterns

- Components receive `version: ReadSignal<usize>` for reactivity
- Use `use_context::<AppState>()` to access shared state
- Use `use_effect` for side effects (scrollIntoView, focus)
- Conditional rendering with `if condition { rsx! { ... } }`
- Read signals in component body to subscribe to changes

### Extension Traits Pattern

Operations are organized as extension traits on EditorContext:

```rust
// operations/movement.rs
pub trait MovementOps {
    fn move_cursor(&mut self, doc_id: DocumentId, view_id: ViewId, direction: Direction);
    // ...
}

impl MovementOps for EditorContext {
    fn move_cursor(&mut self, ...) {
        // implementation
    }
}

// In state/mod.rs, import and use:
use crate::operations::{MovementOps, EditingOps, ...};
// Methods automatically available on EditorContext
```

### Assets Pattern

**External Stylesheet**: CSS is loaded via Dioxus `document::Stylesheet` with `asset!()` macro:

```rust
// In app.rs
rsx! {
    document::Stylesheet { href: asset!("/assets/styles.css") }
    // ...
}
```

**JavaScript Functions**: Custom script is loaded via `include_str!` and wrapped in a script tag:

```rust
// In main.rs
const CUSTOM_SCRIPT: &str = include_str!("../assets/script.js");
// Used with: .with_custom_head(format!("<script>{CUSTOM_SCRIPT}</script>"))
```

Functions defined in `script.js`:
- `focusAppContainer()` - focuses app on mount
- `scrollCursorIntoView()` - scrolls cursor into view

**CSS Classes** (defined in `styles.css`):
- `.app-container`, `.editor-view`, `.gutter`, `.content`
- `.buffer-bar`, `.buffer-tab`, `.statusline`
- `.picker-*` (overlay, container, header, list, item)
- `.prompt`, `.prompt-cursor`
- `.completion-*`, `.hover-*`, `.code-action-*` (LSP popups)

**Dynamic Styles**: Styles requiring Rust variables remain inline:
- Mode colors: `style: "background-color: {mode_bg};"`
- Active state: `style: "color: {text_color};"`

### Coding Conventions

- Keep components under 300 lines
- Use extension traits for operation grouping
- Prefer match over if-let for mode dispatch
- Always call `process_commands_sync()` after sending commands
- Follow Rust derive order: Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default
- Fields that need cross-module access use `pub(crate)`
- Extract static CSS to `styles.css`, keep dynamic styles inline in RSX

## Build Commands

```bash
# Development build
cargo build -p helix-dioxus

# Run with file
cargo run -p helix-dioxus -- <file>

# Run with directory (opens file picker)
cargo run -p helix-dioxus -- <directory>

# Run with glob pattern
cargo run -p helix-dioxus -- "src/*.rs"

# Check compilation
cargo check -p helix-dioxus

# Lint
cargo clippy -p helix-dioxus --bins
```

## Troubleshooting

### Selection appears in Normal mode after movement
- Cause: Helix always has 1-char selection internally
- Solution: Only render selection in Select mode (`has_selection` tied to mode)

### Component not re-rendering after state change
- Cause: Dioxus 0.7 requires reading signal to subscribe
- Solution: Add `let _ = version();` in component body

### SelectionDidChange errors in logs
- Cause: WebView events not handled by Dioxus
- Solution: tracing.rs filters these messages

### macOS dock icon not showing
- Cause: Dock icons require .app bundle on macOS
- Status: Known issue, marked as TODO

## Feature Roadmap

### Planned Enhancements
- [ ] Keybinding help bar above statusline showing common shortcuts (context-aware per mode)
- [ ] Command panel as picker-style UI with fuzzy search
- [ ] File-type specific icons in buffer bar
- [ ] Mouse click support in picker
- [x] ~~LSP integration for diagnostics and completions~~ Diagnostics display with gutter icons, error lens, and status bar counts
- [ ] Multiple splits/views support
- [ ] System clipboard integration
- [ ] Extract theme colors to `theme.rs` or `colors.rs`
- [ ] Add custom hooks (`use_editor_state`, `use_keybinding`)
- [x] ~~Consider splitting picker into `FilePicker`, `BufferPicker` components~~ Split into picker/ folder
- [ ] Add integration tests for key operations
