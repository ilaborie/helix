# helix-dioxus CLAUDE.md

This file provides guidance to Claude Code when working with the helix-dioxus crate.

## Architecture

helix-dioxus is a Dioxus 0.7 desktop frontend for the Helix text editor, structured as a library crate with a `dhx` binary.

### Key Architectural Pattern

- **Message Passing**: EditorContext (non-Send/Sync) lives on main thread
- **Commands**: UI sends EditorCommand via mpsc channel
- **Snapshots**: EditorSnapshot captures state for rendering (Clone + Send + Sync)
- **Thread-local**: EDITOR_CTX provides synchronous access for immediate updates

### Module Structure

```
helix-dioxus/src/
├── lib.rs                      # Library root: launch(), AppState, public module declarations
├── config.rs                   # DhxConfig: window, font, logging settings (from dhx.toml)
├── app.rs                      # Root App component
├── events.rs                   # Event registration for helix_event (MUST run before hooks)
│
├── bin/dhx/                    # Binary entry point
│   ├── main.rs                 # CLI: loads config, inits tracing/loader, launches app
│   ├── args.rs                 # Command-line argument parsing → StartupAction
│   └── tracing_setup.rs        # Tracing subscriber init from LoggingConfig
│
├── components/                 # UI Components
│   ├── mod.rs                  # Re-exports all components
│   ├── editor_view.rs          # Document rendering with syntax highlighting
│   ├── buffer_bar.rs           # Tab bar with scroll buttons
│   ├── statusline.rs           # Mode, filename, position display
│   ├── keybinding_help.rs      # Context-aware keybinding help bar
│   ├── scrollbar.rs            # Custom scrollbar with diagnostic markers
│   ├── diagnostics.rs          # Diagnostic rendering helpers
│   ├── lsp/                    # LSP-related popups
│   │   ├── mod.rs              # Re-exports
│   │   ├── code_actions.rs     # Code actions menu (uses InlineListDialog)
│   │   ├── completion.rs       # Completion popup (uses InlineListDialog)
│   │   ├── hover.rs            # Hover popup (uses InlineDialogContainer)
│   │   ├── signature_help.rs   # Signature help (uses InlineDialogContainer)
│   │   └── location_picker.rs  # Location picker for LSP references/definitions
│   ├── dialog/                 # Dialogs and prompts
│   │   ├── mod.rs              # Re-exports
│   │   ├── confirmation.rs     # Confirmation dialog (quit with unsaved changes)
│   │   ├── input.rs            # Input dialog (rename symbol, etc.)
│   │   ├── lsp_status.rs       # LSP server status dialog
│   │   ├── notification.rs     # Notification toast container
│   │   └── prompt.rs           # Command/search prompts
│   ├── picker/                 # Picker components (overlay dialogs)
│   │   ├── mod.rs              # Re-exports GenericPicker
│   │   ├── generic.rs          # Main picker container (two-column with preview)
│   │   ├── item.rs             # PickerItemRow component
│   │   ├── highlight.rs        # HighlightedText for fuzzy matches
│   │   └── preview.rs          # PickerPreviewPanel: syntax-highlighted file preview
│   └── inline_dialog/          # Inline dialog primitives (cursor-positioned)
│       ├── mod.rs              # Re-exports InlineDialogContainer, InlineListDialog
│       ├── container.rs        # Base container with positioning logic
│       └── list.rs             # List dialog with selection support
│
├── state/                      # State Management
│   ├── mod.rs                  # EditorContext, command dispatch, config loading
│   ├── types.rs                # Data structures (EditorSnapshot, StartupAction, etc.)
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
│   ├── cli.rs                  # CliOps: execute_command
│   ├── shell.rs                # ShellOps: execute_shell_command (pipe selections)
│   └── word_jump.rs            # WordJumpOps: compute labels, filter, jump
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
    ├── regex.rs                # handle_regex_mode (select/split regex prompt)
    ├── search.rs               # handle_search_mode
    ├── confirmation.rs         # handle_confirmation_mode
    ├── input_dialog.rs         # handle_input_dialog_mode
    └── shell.rs                # handle_shell_mode

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
- `handle_text_input_keys(code, esc, enter, backspace, char_fn)` → shared Esc/Enter/Backspace/Char pattern (used by search/command/regex modes)
- `handle_list_navigation_keys(code, esc, up, down, enter, backspace?, char_fn?)` → shared list navigation (used by location picker, code actions)

Multi-key sequences (f/F/t/T, g, Space, [, ], r, m, ") are handled via `PendingKeySequence` enum in `app.rs`.

The `"` prefix selects a register for the next yank/paste/delete:
- `"<char>` → sets `editor.selected_register` (consumed by next clipboard/editing op)
- Examples: `"ay` (yank to `a`), `"ap` (paste from `a`), `"_d` (delete to black hole)

The `m` prefix supports nested sequences:
- `mm` → match bracket
- `mi<char>` → select inside pair
- `ma<char>` → select around pair
- `ms<char>` → surround add
- `md<char>` → surround delete
- `mr<old><new>` → surround replace (3-key sequence)

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

# Run with file (binary is named 'dhx')
cargo run -p helix-dioxus -- <file>

# Run with directory (opens file picker)
cargo run -p helix-dioxus -- <directory>

# Run with glob pattern
cargo run -p helix-dioxus -- "src/*.rs"

# Check compilation (both library and binary)
cargo check -p helix-dioxus --bins --lib

# Lint
cargo clippy -p helix-dioxus --bins --lib

# Tests (use --lib to skip examples)
cargo test -p helix-dioxus --lib

# Documentation
cargo doc -p helix-dioxus --no-deps
```

## Library Usage

helix-dioxus can be used as a library to build custom IDE-like applications:

```rust
use helix_dioxus::{DhxConfig, StartupAction};

fn main() -> anyhow::Result<()> {
    let config = DhxConfig::default()
        .with_window_title("My IDE")
        .with_font_size(16.0);

    helix_loader::initialize_config_file(None);
    helix_loader::initialize_log_file(None);

    let runtime = tokio::runtime::Runtime::new()?;
    let _guard = runtime.enter();

    helix_dioxus::launch(config, StartupAction::None)
}
```

### Configuration

**Two-layer config strategy:**
- **Shared with helix-term**: `~/.config/helix/config.toml` (`[editor]` settings, `theme`) and `languages.toml` (LSP config)
- **GUI-specific**: `~/.config/helix/dhx.toml` for window, font, and logging settings

```toml
# ~/.config/helix/dhx.toml
[window]
title = "My IDE"
width = 1400.0
height = 900.0

[font]
family = "'Fira Code', monospace"
size = 16.0
ligatures = true

[logging]
level = "debug"
log_file = "/tmp/my-ide.log"
```

## Keybinding Comparison

See [KEYBINDINGS.md](KEYBINDINGS.md) for a detailed comparison between helix-dioxus bindings and standard Helix defaults, including matches, deviations, custom extensions, and missing bindings.

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

### Alt+key bindings not working on macOS
- Cause: macOS Option key composes special characters (Alt+o → ø, Alt+i → ˆ, Alt+c → ç). `evt.key()` returns the composed character, so `KeyCode::Char('o')` with ALT modifier never matches
- Solution: In `translate_key_code`, when Alt is pressed, use `evt.code()` (physical key) via `key_code_from_physical()` to get the intended character instead of the composed one
- Affected bindings: Alt+o/i (expand/shrink), Alt+. (repeat), Alt+; (flip selections), Alt+d (delete no yank), Alt+c/C (copy selection), Alt+s (split), Alt+x (shrink to line bounds), Alt+` (case)

## Feature Roadmap

### Planned Enhancements
- [x] ~~Keybinding help bar above statusline showing common shortcuts (context-aware per mode)~~ Context-aware help bar with register indicators
- [x] ~~Command panel as picker-style UI with fuzzy search~~ `Space p` or `:cmd`/`:commands` opens GenericPicker with ~40 commands, fuzzy filter, keybinding hints
- [x] ~~File-type specific icons in buffer bar~~ Added file-type icons
- [x] ~~Mouse click support in picker~~ Added picker mouse clicks
- [x] ~~LSP integration for diagnostics and completions~~ Diagnostics display with gutter icons, error lens, wavy underlines, and status bar counts
- ~~Multiple splits/views support~~ **Not supported** — single-view design decision
- [x] ~~System clipboard integration~~ Uses `Editor.registers` with `'+'` register for system clipboard
- [x] ~~Extract theme colors to `theme.rs` or `colors.rs`~~ Extracted to CSS custom properties in `:root`
- [ ] Add custom hooks (`use_editor_state`, `use_keybinding`)
- [x] ~~Consider splitting picker into `FilePicker`, `BufferPicker` components~~ Split into picker/ folder
- [ ] Add integration tests for key operations
- [x] ~~Named registers (`"a`–`"z`) — register selection before yank/paste (e.g., `"ay`, `"ap`)~~ Full register support with `"` prefix key, named/special registers, black hole `_`, statusline indicator
- [x] ~~Register picker (`:reg` command) — picker-style overlay showing all populated registers~~ GenericPicker with register browsing, confirm sets selected register

### UI Improvements (RustRover-inspired)
- [x] Severity-colored lightbulb indicator - change color based on diagnostic severity (red/yellow/blue/cyan)
- [x] Code actions search box - filter input with count display, typing filters actions
- [x] Diagnostic scrollbar markers - show diagnostic and search positions on right scrollbar edge
- [x] Jump list gutter markers - Bookmark icon on lines with jump list entries, matching picker icon
- [x] Picker file preview panel - side-by-side syntax-highlighted file preview in picker (40%/60% split), with search match highlighting for global search
- [ ] Code actions preview panel - show fix preview before applying (needs LSP resolve)
- [ ] Dialog search mode setting - user setting to toggle between: (1) current behavior where typing filters directly (arrows for navigation), or (2) vim-style where j/k and arrows navigate, '/' toggles search input focus. Applies to pickers and inline dialogs (code actions, completion, etc.)
- [ ] Cursor block visibility — cursor is hard to spot against selection/line-highlight backgrounds, especially after `w`/`b` motions that create multi-char selections. Needs more prominent styling or animation
- [ ] Clipboard register (`+`) visibility in register dialog — register dialog opens but content display needs polish
- [ ] `*` register — currently shows editor selection text; should instead reflect the search register set by the `*` (search word under cursor) command, or be wired to the system primary selection
- [ ] Jump list clear — `:jumplist-clear` command or delete action in jump list picker

### LSP Improvements
- [ ] Investigate rust-analyzer diagnostic line reporting - diagnostics may be reported on the line where parsing fails rather than where the actual error is (e.g., unterminated string reports on the next line). Consider requesting upstream fix or mapping diagnostic positions back to the originating code

### Design Decisions
- **Window/Splits**: Not supported — helix-dioxus uses a single-view design. `C-w` prefix and `Space w` sub-menu will not be implemented.

### Recently Completed
- [x] Shell integration (`|`, `!`, `A-|`, `A-!`) — pipe selections through shell commands with interactive prompt, per-selection processing, CLI commands (`:pipe`, `:sh`, `:insert-output`, `:append-output`, `:pipe-to`, `:run`), help bar hints, command panel entries
- [x] Word jump (`gw`) — EasyMotion-style two-char label navigation, `gw` in normal mode jumps to word, `gw` in select mode extends selection, labels rendered as overlay spans, dimming on first char filter, Esc to cancel
- [x] Picker file preview panel — side-by-side two-column layout (40% list / 60% preview) with syntax-highlighted file content, focus line indicator, search match highlighting for GlobalSearch; supports all file-based picker modes (DirectoryBrowser, FilesRecursive, Buffers, Symbols, Diagnostics, GlobalSearch, References, Definitions, JumpList); single-column preserved for Registers/Commands; `compute_tokens_for_rope` extracted as reusable helper for both editor view and preview
- [x] Jump list gutter markers — orange Bookmark icon in indicator gutter for lines with jump list entries, `jump_lines` in `EditorSnapshot`, cache key updated for re-renders
- [x] Fix Alt+key bindings on macOS — Option key composed special characters (ø, ˆ, ç) instead of intended keys; now uses physical key code (`evt.code()`) for Alt normalization in `translate.rs`
- [x] Tree-sitter expand/shrink selection — `A-o` expands to parent syntax node (pushes history), `A-i` shrinks back (pops history or uses tree-sitter), both in normal and select modes, command panel entries
- [x] Multi-selection, regex select/split, copy/rotate — multi-selection rendering (all ranges, not just primary), `s`/`S` regex select/split with prompt, `A-s` split on newline, `C`/`A-C` copy selection on next/prev line, `(`/`)` rotate selections
- [x] Format document + align selections — `:format` / command panel now uses LSP `textDocument/formatting`, `&` aligns multi-cursor selections, `=` formats via LSP range formatting
- [x] Quick wins batch (6 bindings) — `A-d`/`A-c` delete/change without yanking, `C-a`/`C-x` increment/decrement numbers and dates, `_` trim selections, `=` format selections via LSP range formatting
- [x] Goto + bracket navigation batch (20 bindings) — `gt`/`gc`/`gb` window position, `ga`/`gm`/`g.` file/edit goto, `]f`/`[f` function, `]t`/`[t` class, `]a`/`[a` parameter, `]c`/`[c` comment, `]p`/`[p` paragraph, `]D`/`[D` first/last diagnostic, `X` extend to line bounds, `A-x` shrink to line bounds
- [x] Keybinding gap reduction (3 batches, ~30 new bindings) — Batch 1: wire existing commands (`C-b`/`C-f` page, `ge`/`gh`/`gl`/`gn`/`gp` goto, `Space b` buffer picker, insert `C-h`/`C-d`/`C-j`). Batch 2: new operations (`C-u`/`C-d` half-page, `%` select all, `A-;` flip selections, `gs` first non-whitespace, `] Space`/`[ Space` add newline, `C-k` kill to line end, `A-d` delete word forward). Batch 3: select mode extend variants (`f`/`F`/`t`/`T`/`r` in select mode, `n`/`N` extend search next/prev, mode-aware dispatch in `app.rs`)
- [x] Command panel — `Space p` or `:cmd`/`:commands` opens GenericPicker with ~40 editor commands, fuzzy filter, keybinding hints as secondary text, `Terminal` icon (cyan), deferred dispatch via `command_tx`
- [x] Register picker — `:reg`/`:registers` command opens GenericPicker showing all registers, populated first, confirm sets `editor.selected_register` for next yank/paste
- [x] Named registers — `"` prefix key for register selection (`"ay`, `"ap`, `"_d`), `take_register()` helper, black hole register `_`, statusline `reg=` indicator, help bar hints, select mode `p` fix (`ReplaceWithYanked`)
- [x] Core tutor commands batch — 21 commands: `;` (collapse selection), `,` (keep primary), `Alt-.` (repeat motion), `c` (change), `e`/`W`/`E`/`B` (word motions), `I` (insert line start), `r` (replace char), `R` (replace with yank), `J` (join), `~`/`` ` ``/`Alt+`` ` (case ops), `mm` (match bracket), `mi`/`ma` (select inside/around), `ms`/`md`/`mr` (surround)
- [x] Register indicators in help bar — `+` (clipboard), `*` (selection), `/` (search) with active/inactive highlighting, click-to-open dialog with content view and Clear button
- [x] Keybinding help bar — context-aware shortcut hints above statusline per mode and pending key sequence
- [x] Find/till character motions (f/F/t/T) with repeat (Alt-.)
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
