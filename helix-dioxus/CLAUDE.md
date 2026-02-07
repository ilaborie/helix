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
├── events.rs                   # Event registration for helix_event (MUST run before hooks)
├── tracing.rs                  # Logging configuration
│
├── components/                 # UI Components
│   ├── mod.rs                  # Re-exports
│   ├── editor_view.rs          # Document rendering with syntax highlighting
│   ├── buffer_bar.rs           # Tab bar with scroll buttons
│   ├── statusline.rs           # Mode, filename, position display
│   ├── picker/                 # Picker components (overlay dialogs)
│   │   ├── mod.rs              # Re-exports GenericPicker
│   │   ├── generic.rs          # Main picker container
│   │   ├── item.rs             # PickerItemRow component
│   │   └── highlight.rs        # HighlightedText for fuzzy matches
│   ├── inline_dialog/          # Inline dialog components (cursor-positioned)
│   │   ├── mod.rs              # Re-exports InlineDialogContainer, InlineListDialog
│   │   ├── container.rs        # Base container with positioning logic
│   │   └── list.rs             # List dialog with selection support
│   ├── code_actions.rs         # Code actions menu (uses InlineListDialog)
│   ├── completion.rs           # Completion popup (uses InlineListDialog)
│   ├── confirmation_dialog.rs  # Confirmation dialog (quit with unsaved changes)
│   ├── diagnostics.rs          # Diagnostic rendering helpers
│   ├── hover.rs                # Hover popup (uses InlineDialogContainer)
│   ├── input_dialog.rs         # Input dialog (rename symbol, etc.)
│   ├── location_picker.rs      # Location picker for LSP references/definitions
│   ├── lsp_dialog.rs           # LSP server status dialog
│   ├── notification.rs         # Notification toast container
│   ├── scrollbar.rs            # Custom scrollbar with diagnostic markers
│   ├── signature_help.rs       # Signature help (uses InlineDialogContainer)
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
    ├── mod.rs                  # Re-exports + shared helpers (direction_from_key, handle_move_keys, etc.)
    ├── translate.rs            # Dioxus KeyboardEvent → helix KeyEvent translation
    ├── completion.rs           # handle_completion_mode, location_picker, code_actions, lsp_dialog
    ├── normal.rs               # handle_normal_mode
    ├── insert.rs               # handle_insert_mode
    ├── select.rs               # handle_select_mode
    ├── command.rs              # handle_command_mode
    ├── picker.rs               # handle_picker_mode
    ├── search.rs               # handle_search_mode
    ├── confirmation.rs         # handle_confirmation_mode
    └── input_dialog.rs         # handle_input_dialog_mode

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

**CSS Custom Properties** (`styles.css` uses `:root` variables for theming):
- Colors: `--bg-primary`, `--bg-secondary`, `--bg-highlight`, `--bg-selection`, `--bg-deep`, `--text`, `--text-dim`, `--text-dimmer`, `--accent`, `--error`, `--warning`, `--info`, `--hint`, `--success`, `--purple`, `--orange`
- Font: `--font-mono`
- Z-index layers: `--z-dropdown` (100), `--z-overlay` (200), `--z-modal` (300), `--z-notification` (300), `--z-confirmation` (400), `--z-tooltip` (9999)

**CSS Classes** (defined in `styles.css`):
- `.app-container`, `.editor-view`, `.gutter`, `.content`
- `.buffer-bar`, `.buffer-tab`, `.statusline`
- `.picker-*` (overlay, container, header, list, item)
- `.prompt`, `.prompt-cursor`
- `.completion-*`, `.hover-*`, `.code-action-*` (LSP popups)
- `.inline-dialog`, `.inline-dialog-list`, `.inline-dialog-item` (cursor-positioned popups)
- `.notification-*` (toast notifications)
- `.confirmation-*` (modal confirmation dialogs)
- `.editor-scrollbar`, `.scrollbar-*` (custom scrollbar with markers)

**Dynamic Styles**: Styles requiring Rust variables remain inline:
- Mode colors: `style: "background-color: {mode_bg};"`
- Active state: `style: "color: {text_color};"`

### Inline Dialog Pattern

Cursor-positioned popups use the generic inline dialog components:

```rust
// Content dialog (hover, signature help)
use super::inline_dialog::{DialogConstraints, DialogPosition, InlineDialogContainer};

InlineDialogContainer {
    cursor_line,
    cursor_col,
    position: DialogPosition::Above,  // or Below
    class: "my-popup",
    constraints: DialogConstraints { min_width: None, max_width: Some(500), max_height: Some(300) },
    // content as children
}

// List dialog (completion, code actions)
use super::inline_dialog::{InlineListDialog, InlineListItem};

InlineListDialog {
    cursor_line,
    cursor_col,
    selected,
    empty_message: "No items",
    class: "my-list-popup",
    has_items: !items.is_empty(),

    for (idx, item) in items.iter().enumerate() {
        InlineListItem {
            key: "{idx}",
            is_selected: idx == selected,
            // item content
        }
    }
}
```

CSS classes:
- `.inline-dialog` - Base styles for all inline dialogs
- `.inline-dialog-list` - List variant with padding
- `.inline-dialog-item` - Selectable list item
- `.inline-dialog-item-selected` - Selected state
- `.inline-dialog-empty` - Empty state message

### Event System (helix_event)

**CRITICAL**: helix-view and helix-dioxus use helix_event for event dispatching and hook registration. Events MUST be registered before hooks can be registered for them.

**Initialization Order** (in `lib.rs::launch()`):
1. `events::register()` - registers all helix-view event types with helix_event
2. `EditorContext::new()` - creates handlers and registers hooks via `helix_view::handlers::register_hooks()`

**Why This Matters**:
- `helix_view::handlers::register_hooks()` registers hooks for `DocumentDidChange`, `LanguageServerInitialized`, etc.
- These hooks are ESSENTIAL for LSP synchronization:
  - `DocumentDidChange` → sends `textDocument/didChange` to keep LSP in sync with document edits
  - `LanguageServerInitialized` → sends `textDocument/didOpen` for all documents when LSP starts
- Without proper event registration, the app will panic: "Tried to register handler for unknown event"
- Without the hooks, LSP operations like rename will corrupt text because the server has stale content

**Files involved**:
- `events.rs` - registers events with `helix_event::register_event::<T>()`
- `state/mod.rs::create_handlers()` - calls `helix_view::handlers::register_hooks()`
- `state/lsp_events.rs` - dispatches `LanguageServerInitialized` and `LanguageServerExited` events

**When adding new LSP features**: Check if helix-term dispatches any events in its handling code. If so, helix-dioxus must:
1. Register the event type in `events.rs`
2. Dispatch the event at the appropriate time in `lsp_events.rs`

### Keybinding Helpers Pattern

Shared keybinding logic lives in `keybindings/mod.rs`:
- `direction_from_key(code)` → maps hjkl/arrows to `Direction`
- `handle_move_keys(code)` → direction → `MoveLeft/Right/Up/Down`
- `handle_extend_keys(code)` → direction → `ExtendLeft/Right/Up/Down`
- `handle_text_input_keys(code, esc, enter, backspace, char_fn)` → shared Esc/Enter/Backspace/Char pattern (used by search/command modes)
- `handle_list_navigation_keys(code, esc, up, down, enter, backspace?, char_fn?)` → shared list navigation (used by location picker, code actions)

Multi-key sequences (f/F/t/T, g, Space, [, ]) are handled via `PendingKeySequence` enum in `app.rs`.

### Coding Conventions

- Keep components under 300 lines
- Use extension traits for operation grouping
- Prefer match over if-let for mode dispatch
- Always call `process_commands_sync()` after sending commands
- Follow Rust derive order: Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default
- Fields that need cross-module access use `pub(crate)`
- Extract static CSS to `styles.css` with CSS custom properties, keep dynamic styles inline in RSX
- Use CSS custom properties (`var(--name)`) instead of hardcoded color values in CSS

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

### Test file with intentional error (examples/test_error.rs)
- **DO NOT DELETE** this file - it contains an intentional type error for testing LSP diagnostics
- Used to test: error lens, diagnostic underlines, gutter icons, code actions
- When running `cargo test`, use `--bins` flag to skip examples: `cargo test -p helix-dioxus --bins`
- Or exclude examples: `cargo test -p helix-dioxus --lib`

### Selection visibility
- Helix always has a 1-char selection internally (even in Normal mode)
- Solution: Show selection highlighting only when `sel_end > sel_start + 1` (multi-char selections in any mode)

### Component not re-rendering after state change
- Cause: Dioxus 0.7 requires reading signal to subscribe
- Solution: Add `let _ = version();` in component body

### SelectionDidChange errors in logs
- Cause: WebView events not handled by Dioxus
- Solution: tracing.rs filters these messages

### macOS dock icon not showing
- Cause: Dock icons require .app bundle on macOS
- Status: Known issue, marked as TODO

### LSP rename corrupts text (missing characters after rename)
- Cause: LSP server not receiving `textDocument/didChange` notifications
- Solution: Ensure `events::register()` is called before `EditorContext::new()`, and that `helix_view::handlers::register_hooks()` is called when creating handlers
- See "Event System" section above

### Panic: "Tried to register handler for unknown event"
- Cause: `helix_view::handlers::register_hooks()` called before events are registered
- Solution: Call `events::register()` at the start of `launch()` before creating `EditorContext`

## Feature Roadmap

### Planned Enhancements
- [ ] Keybinding help bar above statusline showing common shortcuts (context-aware per mode)
- [ ] Command panel as picker-style UI with fuzzy search
- [x] ~~File-type specific icons in buffer bar~~ Added file-type icons
- [x] ~~Mouse click support in picker~~ Added picker mouse clicks
- [x] ~~LSP integration for diagnostics and completions~~ Diagnostics display with gutter icons, error lens, wavy underlines, and status bar counts
- [ ] Multiple splits/views support
- [ ] System clipboard integration
- [x] ~~Extract theme colors to `theme.rs` or `colors.rs`~~ Extracted to CSS custom properties in `:root`
- [ ] Add custom hooks (`use_editor_state`, `use_keybinding`)
- [x] ~~Consider splitting picker into `FilePicker`, `BufferPicker` components~~ Split into picker/ folder
- [ ] Add integration tests for key operations

### UI Improvements (RustRover-inspired)
- [x] Severity-colored lightbulb indicator - change color based on diagnostic severity (red/yellow/blue/cyan)
- [x] Code actions search box - filter input with count display, typing filters actions
- [x] Diagnostic scrollbar markers - show diagnostic and search positions on right scrollbar edge
- [ ] Code actions preview panel - show fix preview before applying (needs LSP resolve)
- [ ] Dialog search mode setting - user setting to toggle between: (1) current behavior where typing filters directly (arrows for navigation), or (2) vim-style where j/k and arrows navigate, '/' toggles search input focus. Applies to pickers and inline dialogs (code actions, completion, etc.)

### LSP Improvements
- [ ] Investigate rust-analyzer diagnostic line reporting - diagnostics may be reported on the line where parsing fails rather than where the actual error is (e.g., unterminated string reports on the next line). Consider requesting upstream fix or mapping diagnostic positions back to the originating code

### Recently Completed
- [x] Find/till character motions (f/F/t/T) with repeat (;) and reverse (,)
- [x] Indent/unindent (>/<), search word under cursor (*)
- [x] Insert mode: Ctrl+w (delete word backward), Ctrl+u (delete to line start)
- [x] Picker: Home/End/PageUp/PageDown navigation
- [x] CSS custom properties - extracted all hardcoded colors/z-indices to `:root` variables
- [x] Keybinding refactoring - shared helpers for move/extend/text-input/list-navigation patterns
- [x] Buffer save refactoring - `save_doc_inner` helper, `build_confirmation_dialog` helper
- [x] Selection visibility fix - show multi-char selections in all modes (not just Select)
- [x] Removed unused `inlay_hints.rs` module and dead code comments
- [x] Confirmation dialog for quit/close with unsaved changes
- [x] LSP document synchronization - register helix_event hooks
- [x] Scrollbar with diagnostic and search result markers, tooltips
- [x] Generic inline dialog components
- [x] LSP progress tracking, server restart, code actions search
