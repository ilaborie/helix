# Development Logbook

This file tracks the development progress, decisions, and issues encountered while building helix-dioxus.

---

## 2026-01-31: Project Initialization

### Progress
- Created initial project structure
- Set up `Cargo.toml` with dependencies on helix crates and Dioxus 0.6
- Created `PLAN.md` with implementation roadmap
- Implemented basic application skeleton with:
  - Main entry point with Dioxus desktop app launch
  - Editor state management via `EditorContext`
  - Keyboard input translation from Dioxus to helix format
  - Document rendering with cursor display
  - Status line showing mode, file name, and position
  - Basic vim movement commands (h/j/k/l, w/b, 0/$, G)
  - Insert mode with character insertion and deletion
  - Mode switching (i, a, A, o, O, Esc)

### Architecture Decisions

**Decision: Use Dioxus 0.6 (stable)**
- Rationale: 0.6 is the current stable version on crates.io
- Dioxus 0.7 introduces new features but is not yet stable

**Decision: Message-passing architecture instead of shared Editor**
- Rationale: `helix_view::Editor` contains non-Send/Sync types (Cell, trait objects)
- Cannot use Dioxus context for sharing Editor across threads
- Solution: Commands sent via `mpsc::channel`, Editor lives on main thread
- Snapshots of state are taken for rendering

**Decision: Use custom event handler for command processing**
- Rationale: Need to process commands and update snapshots on each render cycle
- Dioxus `with_custom_event_handler` provides the hook point

**Decision: Use `include_str!` for CSS**
- Rationale: Keep HTML/CSS in separate files for better maintainability
- Assets directory contains `head.html` with base styles

### Technical Challenges Overcome

1. **Thread safety with Editor**
   - Problem: Editor contains `Cell`, `Rc`, trait objects that are not `Send + Sync`
   - Solution: Keep Editor on main thread, use channels for communication

2. **Dioxus keyboard event translation**
   - Problem: Dioxus key events differ from crossterm format
   - Solution: Created translation layer in `input.rs`

3. **Handler initialization**
   - Problem: `helix_view::Handlers` requires various channel senders
   - Solution: Create dummy handlers with unused channel receivers

### Files Created
- `Cargo.toml` - Project configuration
- `PLAN.md` - Implementation plan snapshot
- `DEV_LOGBOOK.md` - This file
- `src/main.rs` - Entry point with Dioxus app setup
- `src/app.rs` - Main App component with keyboard handling
- `src/state.rs` - Editor state management and command processing
- `src/input.rs` - Keyboard event translation
- `src/editor_view.rs` - Document rendering component
- `src/statusline.rs` - Status bar component
- `assets/head.html` - Base HTML/CSS styles

### Research Insights

From GitHub discussions and helix-gpui analysis:
- helix-gpui uses a fork of helix with modifications
- Two main GUI approaches debated: Component Drawer vs Render Surface
- Character width issues highlighted as platform-specific challenge
- Current helix architecture has crossterm dependencies in helix-view

### Next Steps
1. Test the application with real files
2. Add syntax highlighting support
3. Implement file picker (`:o` command)
4. Add save functionality (`:w` command)
5. Improve scrolling and viewport management

---

## 2026-01-31: File Picker and Lucide Icons

### Progress
- Implemented file picker command (`:o` / `:open`)
  - Modal overlay with file listing from working directory
  - j/k navigation with selection highlighting
  - Enter to open file, Esc to cancel
  - Visual scrolling window for large file lists (15 items visible)
- Added command mode prompt component
  - Triggered by `:` key in normal mode
  - Text input with command parsing
- Replaced emoji icons with Lucide SVG icons
  - Added `lucide-dioxus` dependency
  - Folder/File icons for directory/file entries
  - ChevronRight icon for selection indicator
  - Fixed row height consistency

### Files Created/Modified
- `src/picker.rs` - File picker component with Lucide icons
- `src/prompt.rs` - Command mode prompt component
- `src/app.rs` - Integrated picker and prompt into main app
- `src/state.rs` - Added picker state management
- `Cargo.toml` - Added lucide-dioxus dependency

### Technical Notes
- Used opacity toggle for selection indicator to maintain consistent row heights
- Directories detected by trailing `/` in path
- File listing uses `std::fs::read_dir` sorted alphabetically with directories first

### Next Steps
1. Add fuzzy search/filtering to file picker
2. Support relative path navigation (entering directories)
3. Add syntax highlighting to editor view
4. Implement save functionality (`:w` command)
5. Add `:q` quit command
6. Improve scrolling and viewport management

---

## 2026-01-31: File Picker UI Polish

### Progress
- Fixed dialog position to top (no more jumping during filtering)
  - Changed from vertically centered to `align-items: flex-start` with `padding-top: 80px`
- Reduced padding throughout the picker for a tighter UI
  - Header padding: 0
  - Search box padding: `6px 8px` (was `8px 12px`)
  - Help row padding: `2px`
  - Result item padding: `6px 8px` (was `8px 16px`)
- Added `<kbd>` elements for keyboard shortcuts in help text
  - Styled with darker background (`#3e4451`) and rounded corners
- Added "filtered / total" count on help line (right-aligned)
  - Shows e.g. "15 / 42" when filtering
- Added `picker_total` field to `EditorSnapshot` for tracking total items

### Files Modified
- `src/state.rs` - Added `picker_total` field to snapshot
- `src/app.rs` - Pass `total` prop to FilePicker
- `src/picker.rs` - All UI polish changes

### Technical Notes
- The fixed position prevents the dialog from jumping when the filtered result count changes
- kbd styling uses `font-family: inherit` to match the rest of the UI

---

## 2026-01-31: Syntax Highlighting, Save, and Quit

### Progress
- Implemented syntax highlighting using tree-sitter
  - Colors now appear for keywords, strings, comments, types, etc.
  - Follows helix-term's `SyntaxHighlighter` pattern from `document.rs`
- Fixed save command (`:w`) to properly clear modified indicator
  - Calls `append_changes_to_history()` before saving to flush pending changes
  - Updates `last_saved_revision` after save completes
- Quit command (`:q`) already working
  - Checks for unsaved changes, warns unless `:q!` is used
  - Added `should_quit` flag with exit handling in main event loop

### Issues Encountered

**Syntax highlighting producing 0 tokens despite 200+ events**
- Root cause: `set_scopes()` was never called on the syntax loader
- The `Editor::new()` sets the theme but does NOT call `set_scopes()`
- This is only called during `set_theme()` / `set_theme_impl()`
- Solution: Manually call `editor.syn_loader.load().set_scopes(theme.scopes().to_vec())` after creating the Editor

**Save works but modified indicator [+] remains**
- Root cause: Pending changes weren't flushed to history before saving
- Solution: Call `doc.append_changes_to_history(view)` before initiating save

### Technical Notes

The syntax highlighting implementation:
1. `compute_syntax_tokens()` creates a highlighter for visible byte range
2. Processes `HighlightEvent::Refresh` and `HighlightEvent::Push` events
3. Maintains a computed `Style` that accumulates highlight colors
4. Converts `helix_view::graphics::Color` to CSS hex strings via `color_to_css()`
5. `TokenSpan` structs hold start/end offsets and color for each line
6. `render_styled_content()` in `editor_view.rs` renders spans with inline styles

Key insight: The `highlights` iterator from `highlighter.advance()` is only populated when `set_scopes()` has been called with the theme's scope names. Without this, tree-sitter produces events but no highlight information is mapped.

### Files Modified
- `src/state.rs` - Added `set_scopes()` call, `TokenSpan` struct, `compute_syntax_tokens()`, `color_to_css()`
- `src/editor_view.rs` - `render_styled_content()` for token rendering (already existed)
- `src/main.rs` - Changed log level to Info

### Commands Supported
- `:w` / `:write` - Save current file
- `:w!` / `:write!` - Force save
- `:q` / `:quit` - Quit (fails if modified)
- `:q!` / `:quit!` - Force quit
- `:wq` / `:x` - Save and quit
- `:wq!` / `:x!` - Force save and quit

### Next Steps
1. Add undo/redo support (`u`, `Ctrl+r`)
2. Implement visual selection mode
3. Add search functionality (`/`, `?`, `n`, `N`)
4. Support multiple buffers/splits
5. Add LSP integration for diagnostics and completions
6. Improve scrolling with mouse wheel support

---

## 2026-01-31: Fix Viewport Scrolling

### Progress
- Fixed viewport not scrolling when cursor moves off-screen
- Added `ensure_cursor_in_view()` call after processing commands

### Issue
When moving the cursor with j/k beyond the visible viewport, the view didn't scroll to follow the cursor. The cursor would move off-screen and become invisible.

### Solution
Added a call to `self.editor.ensure_cursor_in_view(view_id)` at the end of `process_commands()`. This uses helix-view's built-in viewport management that:
1. Gets the current scrolloff config
2. Calculates if the cursor is outside the visible area
3. Updates the document's view_offset to keep the cursor visible

### Files Modified
- `src/state.rs` - Added ensure_cursor_in_view call in process_commands()

### Technical Notes
The `Editor::ensure_cursor_in_view(view_id)` method internally:
1. Gets the View and Document
2. Calls `view.ensure_cursor_in_view(doc, config.scrolloff)`
3. Which updates `doc.set_view_offset()` if needed

This is the same pattern used throughout helix-term after cursor movements.

---

## 2026-01-31: Undo/Redo Support

### Progress
- Added undo (`u`) and redo (`Ctrl+r`, `U`) commands
- Uses helix's built-in document history system

### Implementation
- Added `Undo` and `Redo` variants to `EditorCommand` enum
- Implemented `undo()` and `redo()` methods calling `doc.undo(view)` / `doc.redo(view)`
- Added keyboard bindings:
  - `u` - Undo
  - `U` (Shift+U) - Redo
  - `Ctrl+r` - Redo

### Files Modified
- `src/state.rs` - Added Undo/Redo commands and handler methods
- `src/app.rs` - Added keyboard bindings for u, U, and Ctrl+r

### Technical Notes
Helix's undo/redo is transaction-based. Each edit operation creates a transaction that gets recorded in the document's history. The `doc.undo(view)` and `doc.redo(view)` methods handle:
- Reverting/replaying the transaction
- Restoring cursor position
- Updating the document state

---

## 2026-01-31: Visual Selection Mode

### Progress
- Implemented visual selection mode (`v` key from normal mode)
- Selection extends with movement keys while preserving anchor point
- Selection is visually highlighted with background color

### Implementation
- Added `v` key binding in normal mode to enter select mode
- Updated `handle_select_mode()` with full key bindings:
  - `Esc` - Exit select mode (returns to normal mode)
  - `h/j/k/l` or arrow keys - Extend selection by character/line
  - `w/b` - Extend selection by word forward/backward
  - `0/$` or Home/End - Extend selection to line start/end
- Added `selection_range` field to `LineSnapshot` to track selection bounds per line
- Updated `render_styled_content()` to render selection highlighting with background color

### Files Modified
- `src/app.rs` - Added `v` binding in normal mode, expanded `handle_select_mode()`
- `src/state.rs` - Added `selection_range` to `LineSnapshot`, computed in `snapshot()`
- `src/editor_view.rs` - Selection highlighting in `render_styled_content()`

### Technical Notes
- Selection uses helix's `Range` with anchor (start point) and head (cursor/end point)
- `primary_range.from()` and `primary_range.to()` give ordered start/end positions
- Selection highlighting uses `#3e4451` background color (One Dark selection)
- Cursor position is always visible on top of selection highlighting

---

## 2026-01-31: Scroll and Selection UI Fixes

### Progress
- Fixed selection background gaps between lines
- Fixed horizontal and vertical scrolling to follow cursor on window resize

### Issues Encountered

**Selection background had gaps between lines**
- Root cause: Selection background was applied to inline `<span>` elements, but `line-height: 1.5` creates vertical space between lines that spans don't cover
- Solution: Apply selection background at the line `<div>` level instead of individual spans

**Scrolling didn't follow cursor after window resize**
- Root cause: The `ensure_cursor_in_view()` uses a hardcoded viewport of 40 lines, which doesn't account for smaller windows
- Solution: Use JavaScript `scrollIntoView()` on the cursor element after each render
- Implementation details:
  - Added `id="editor-cursor"` to cursor span for DOM targeting
  - Used `use_reactive!` macro to re-run effect when `version` changes
  - Used `requestAnimationFrame` to ensure scroll happens after DOM paint

### Files Modified
- `src/editor_view.rs` - Line-level selection background, cursor ID, scrollIntoView effect
- `build.rs` - Fixed clippy warnings

### Technical Notes
The `scrollIntoView({ block: 'nearest', inline: 'nearest' })` option scrolls the minimum amount needed to make the cursor visible, both vertically and horizontally. Using `requestAnimationFrame` ensures the DOM has been updated before attempting to scroll.

---

## 2026-01-31: Search, Clipboard, and Line Selection

### Progress
- Implemented search functionality (`/`, `?`, `n`, `N`)
- Implemented yank/paste operations (`y`, `p`, `P`)
- Implemented delete in select mode (`d`)
- Implemented line selection (`x`, `X`)

### Implementation Details

**Search (`/`, `?`, `n`, `N`):**
- `/` enters forward search mode, `?` enters backward search mode
- Type pattern and press Enter to search
- `n` finds next occurrence, `N` finds previous
- Search wraps around at document boundaries
- Search prompt shows `/` or `?` prefix with yellow color
- Last search pattern saved for `n`/`N` navigation

**Clipboard (`y`, `p`, `P`):**
- `y` in select mode yanks selection to internal clipboard
- `p` pastes after cursor, `P` pastes before cursor
- Line-wise paste (when clipboard ends with newline) pastes on new line
- Delete (`d`) also saves to clipboard before deleting

**Line Selection (`x`, `X`):**
- `x` selects entire current line (enters select mode)
- `X` extends selection to include next line
- Works in both normal and select modes

### New Commands Added
- `EnterSearchMode { backwards: bool }` - Enter search mode
- `ExitSearchMode` - Cancel search
- `SearchInput(char)` / `SearchBackspace` - Edit search pattern
- `SearchExecute` - Execute search
- `SearchNext` / `SearchPrevious` - Find next/previous match
- `Yank` - Copy selection to clipboard
- `Paste` / `PasteBefore` - Paste from clipboard
- `DeleteSelection` - Delete selection (saves to clipboard first)
- `SelectLine` / `ExtendLine` - Line selection commands

### Files Modified
- `src/state.rs` - Added commands, state fields, and handlers
- `src/app.rs` - Added keyboard bindings and search mode handler
- `src/prompt.rs` - Added SearchPrompt component

### Technical Notes
- Search uses simple substring matching (not regex)
- Clipboard is internal to the editor (not system clipboard)
- Yank in select mode auto-exits to normal mode
- Delete in select mode auto-exits to normal mode

### Next Steps
1. Support multiple buffers/splits
2. Add LSP integration for diagnostics and completions
3. Improve scrolling with mouse wheel support
4. Add regex search support
5. Integrate with system clipboard

---

## 2026-01-31: Selection Highlighting and Logging Fixes

### Progress
- Fixed character-level selection highlighting (was incorrectly applied to entire line)
- Fixed empty line selection visibility (selection background now shows on empty lines)
- Fixed gaps between selected lines (hybrid line-level + span-level approach)
- Fixed 'd' key not working after first delete (added to normal mode)
- Migrated from fern to tracing-subscriber for logging
- Added tracing filter to suppress noisy "SelectionDidChange" webview events

### Issues Encountered

**Selection with 'w' not visible**
- Root cause: Selection background was being applied at the line div level, not to individual characters
- The `render_styled_content` function didn't handle selection ranges
- Fix: Updated `render_styled_content` to take `selection_range` parameter and apply background color only to selected characters within that range

**Gaps between selected lines**
- Root cause: Applying selection at span level causes gaps due to line-height
- Fix: Hybrid approach - apply selection background at LINE level, then mask non-selected parts with normal background at span level

**Empty lines not showing selection**
- Root cause: Selection range calculation used `range_start < range_end` which fails for empty lines where both are 0
- Fix: Changed condition to `range_start <= range_end` and added special handling for `selection_range = Some((0, 0))` to show selection background on empty lines

**Delete ('d') only works once**
- Root cause: 'd' key was only mapped in SELECT mode, but helix's selection-first model creates selections in NORMAL mode (when pressing 'w')
- Fix: Added 'd' and 'y' key bindings to normal mode for delete and yank operations

**SelectionDidChange errors in console**
- Root cause: The error "Dispatched unknown event SelectionDidChange" comes through tracing (used by dioxus-logger), not the log crate
- Our fern filter only captured log crate messages
- Fix: Replaced fern with tracing-subscriber, initialized BEFORE Dioxus launch to prevent dioxus-logger from setting its own subscriber

### Decisions Made
- Use tracing instead of log+fern for unified logging
- Keep log crate for API compatibility (tracing has log compatibility feature)
- Hybrid selection rendering: line-level background + span-level masking for non-selected parts
- Add 'd' and 'y' to normal mode for selection-first model compatibility

### Files Modified
- `Cargo.toml` - Replaced fern with tracing/tracing-subscriber
- `src/main.rs` - New `setup_tracing()` function, removed `setup_logging()`
- `src/state.rs` - Fixed selection range calculation for empty lines
- `src/editor_view.rs` - Hybrid selection rendering approach
- `src/app.rs` - Added 'd' and 'y' key bindings to normal mode

### Next Steps
1. ~~Refine SelectionDidChange filter to actually suppress the messages (current filter is metadata-based)~~
2. Support multiple buffers/splits
3. Add LSP integration

---

## 2026-01-31: Tracing Module Refactor

### Progress
- Moved all tracing configuration to a dedicated `tracing.rs` module
- Implemented content-based message filtering using custom `FormatEvent`
- Now properly suppresses "SelectionDidChange" and "Dispatched unknown event" messages

### Implementation Details

The previous approach used `FilterFn` which only has access to metadata (target, level, module path) and cannot filter based on actual message content. The new approach:

1. Created `FilteringFormatter` that implements `FormatEvent<S, N>` trait
2. Captures the formatted message to a buffer first
3. Checks if the message contains any suppressed patterns
4. Either writes the message or silently suppresses it

**Key code pattern:**
```rust
const SUPPRESSED_PATTERNS: &[&str] = &[
    "SelectionDidChange",
    "Dispatched unknown event",
];

impl<S, N> FormatEvent<S, N> for FilteringFormatter {
    fn format_event(&self, ctx, mut writer, event) -> std::fmt::Result {
        let mut message_buf = String::new();
        self.inner.format_event(ctx, Writer::new(&mut message_buf), event)?;

        let should_suppress = SUPPRESSED_PATTERNS
            .iter()
            .any(|pattern| message_buf.contains(pattern));

        if should_suppress {
            Ok(())  // Don't write - suppresses the message
        } else {
            write!(writer, "{message_buf}")
        }
    }
}
```

### Files Created
- `src/tracing.rs` - New dedicated tracing configuration module

### Files Modified
- `src/main.rs` - Replaced inline `setup_tracing()` with `mod tracing; tracing::init();`

### Technical Notes
- Suppressed patterns are defined in a const array for easy extension
- The module includes unit tests verifying the suppressed patterns
- Must be initialized before Dioxus launch to prevent dioxus-logger from setting its own subscriber

### Next Steps
1. Support multiple buffers/splits
2. Add LSP integration

---

## 2026-01-31: Bug Fixes and Improvements

### Progress
- Fixed buffer tab click handlers not working in Dioxus WebView
  - Changed from `onclick` to `onmousedown` for more reliable event handling
  - Added `pointer-events: none` to icon spans so clicks pass through to parent
  - Added `evt.stop_propagation()` to prevent event bubbling
  - Added `on_change` callback to trigger full app re-render (fixes editor not updating on tab click)
- Added tooltip for truncated buffer names (`title` attribute)
- Added parent directory entry ("..") to file picker for navigation
  - Shows "Parent directory" label right-aligned on same line
- Fixed buffer picker layout - "current" indicator now shows on same line, right-aligned
- Added folder/glob argument support:
  - Multiple file arguments (shell-expanded glob): opens all files as buffers
  - Single argument with `*` or `?` (unexpanded glob): uses glob crate to expand
  - Directory argument: changes cwd and auto-opens recursive file picker
- Auto-scroll buffer bar to show current buffer
  - When opening multiple files, the current (last) buffer is now visible
  - When switching buffers via picker or tab click, bar scrolls to show selection

### Technical Notes
- Buffer bar uses `onmousedown` instead of `onclick` for WebView compatibility
- Auto-scroll logic moved to `buffer_bar_snapshot()` in state.rs to keep scroll offset in sync
- `snapshot()` method now takes `&mut self` to allow scroll offset updates
- BufferBar receives `on_change` callback to trigger app-level re-renders

### Files Modified
- `src/buffer_bar.rs` - Click handlers, tooltip, on_change callback
- `src/state.rs` - Parent directory entry, auto-scroll logic
- `src/picker.rs` - Buffer picker and parent directory layout
- `src/main.rs` - Folder/glob argument handling, mutable editor_ctx
- `src/app.rs` - Pass on_change callback to BufferBar
- `Cargo.toml` - Added `glob` crate

### Next Steps
1. Support multiple buffers/splits
2. Add LSP integration

---

## 2026-01-31: Dioxus 0.7 Migration and Reactivity Fixes

### Progress
- Migrated from Dioxus 0.6 to Dioxus 0.7
- Fixed selection rendering bug caused by props losing reactivity in Dioxus 0.7
- Fixed command prompt not displaying when pressing `:`

### Issues Encountered

**Selection appearing on arrow/hjkl keys in Normal mode**
- Root cause: Multiple issues combined:
  1. Dioxus 0.7 changed props to lose reactivity by default
  2. Race condition between async command processing and UI rendering
  3. Helix's selection-first model always has a 1-char selection internally
- Solutions applied:
  - Changed `version: usize` props to `version: ReadSignal<usize>`
  - Added thread-local `EDITOR_CTX` for synchronous command processing
  - Added `process_commands_sync()` to process commands before re-render
  - Changed `has_selection` logic to only be true in Select mode

**Command prompt not displaying when pressing `:`**
- Root cause: App component created `version` signal but never read it
- In Dioxus 0.7, components only re-render if they read a signal that changed
- Child components (EditorView, BufferBar, StatusLine) re-rendered, but App didn't
- Solution: Added `let _ = version();` to App component to subscribe to changes

**Compilation errors during migration**
- `ReadOnlySignal` deprecated → use `ReadSignal`
- `lucide-dioxus` 0.1 requires Dioxus 0.6 → updated to version 2.563
- `time` crate 0.3.46 requires Rust 1.88 → pinned to 0.3.36
- `Key` import path changed → `dioxus::prelude::Key`

### Decisions Made
- Use `ReadSignal<usize>` for version props (passes signal reference, not value)
- Use thread-local storage for synchronous editor access (avoids async race conditions)
- Filter mouse events from logs (too noisy)
- Log to file `/tmp/helix-dioxus.log` instead of stderr

### Files Modified
- `Cargo.toml` - Updated dioxus to 0.7, lucide-dioxus to 2.563
- `src/main.rs` - Added thread-local EDITOR_CTX, process_commands_sync()
- `src/app.rs` - Pass signals directly, read version to subscribe, call process_commands_sync()
- `src/editor_view.rs` - Changed prop type to ReadSignal<usize>
- `src/buffer_bar.rs` - Changed prop type to ReadSignal<usize>
- `src/statusline.rs` - Changed prop type to ReadSignal<usize>
- `src/input.rs` - Fixed Key import path
- `src/state.rs` - Changed has_selection to only be true in Select mode
- `src/tracing.rs` - Added mouse event filters, log to file

### Technical Notes
The key insight is that Dioxus 0.7's reactivity model requires components to explicitly read signals they depend on. Simply passing a signal as a prop doesn't create a subscription unless the component reads it.

For the selection bug, Helix internally always has a 1-char selection (anchor and head differ by 1), but this should only be visually rendered in Select mode.

---

## 2026-01-31: Window Title and Icon

### Progress
- Implemented dynamic window title that updates based on current buffer
  - Uses Dioxus 0.7's `document::Title` component
  - Title format: `helix-dioxus - {buffer_name}`
  - Reactively updates when switching buffers
- Added `image` crate dependency for PNG loading
- Added `load_icon()` function to load helix icon from `contrib/helix.png`
- Set window icon via `WindowBuilder::with_window_icon()`

### Technical Notes
- The `document::Title` component is placed at the start of the `rsx!` block in `App`
- Icon loading uses `include_bytes!` to embed the PNG at compile time
- `image::load_from_memory()` converts PNG to RGBA8 format required by tao's `Icon`

### Known Issue: macOS Dock Icon
The window icon set via `with_window_icon()` does not display in the macOS dock. On macOS, the dock icon comes from the app bundle, not the window API. When running with `cargo run`, there's no `.app` bundle, so the default icon appears.

**Workaround attempted:**
- Created `Dioxus.toml` with bundle configuration
- Created `assets/icon.png` for the dx CLI
- The `dx` CLI has issues with workspace crates

**TODO for later:** Properly bundle the app using `dx bundle` or create a macOS `.app` bundle manually to display the helix icon in the dock.

### Files Modified
- `Cargo.toml` - Added `image` crate with PNG feature
- `src/main.rs` - Added `load_icon()` function, set window icon and title
- `src/app.rs` - Added `document::Title` component

### Files Created
- `Dioxus.toml` - Bundle configuration for dx CLI (not yet working)
- `assets/icon.png` - Copy of helix icon for bundling

---

## 2026-01-31: Code Reorganization and CSS Extraction

### Progress
- Split picker component into folder structure:
  - `picker/mod.rs` - Re-exports
  - `picker/generic.rs` - Main GenericPicker container
  - `picker/item.rs` - PickerItemRow component
  - `picker/highlight.rs` - HighlightedText for fuzzy matches
- Extracted CSS and JavaScript to `assets/head.html`:
  - All static CSS classes (app-container, editor-view, gutter, buffer-bar, picker, prompt, etc.)
  - JavaScript functions: `focusAppContainer()`, `scrollCursorIntoView()`
- Fixed gutter line highlighting bug:
  - Added version and cursor state to gutter key for proper Dioxus reactivity
  - Gutter lines now correctly update when cursor moves
- Updated CLAUDE.md with new module structure and assets pattern

### Technical Notes

**Picker Split Rationale:**
- Original `picker.rs` was 441 lines with three components
- Split allows for future specialized pickers while sharing common components
- `HighlightedText` and `PickerItemRow` can be reused across picker variants

**CSS Extraction Strategy:**
- Static styles → CSS classes in `head.html`
- Dynamic styles (with Rust variables) → Inline `style` attributes
- JavaScript DOM manipulation → Functions in `head.html`, called via `document::eval()`

**Gutter Reactivity Fix:**
The gutter line div keys only included line number, causing Dioxus to reuse stale elements when cursor moved. Fixed by including `version` and `is_cursor` in the key:
```rust
let gutter_key = format!("{}-{}-{}", line.line_number, version, is_cursor);
```

### Files Created
- `src/components/picker/mod.rs`
- `src/components/picker/generic.rs`
- `src/components/picker/item.rs`
- `src/components/picker/highlight.rs`

### Files Modified
- `assets/head.html` - Expanded with all CSS classes and JS functions
- `src/app.rs` - Use CSS classes and JS function
- `src/components/editor_view.rs` - CSS classes, gutter key fix
- `src/components/statusline.rs` - CSS classes
- `src/components/buffer_bar.rs` - CSS classes
- `src/components/prompt.rs` - CSS classes
- `CLAUDE.md` - Updated module structure and assets pattern

### Files Deleted
- `src/components/picker.rs` - Replaced by picker/ folder

---

## 2026-01-31: LSP Integration Foundation

### Progress
- Created comprehensive LSP integration foundation with UI components and state management
- Implemented thread-safe snapshot types for LSP data (Clone + Send + Sync)
- All components ready for actual LSP client integration

### Components Created

**UI Components:**
- `CompletionPopup` - Auto-complete menu with kind badges, labels, and details
- `HoverPopup` - Documentation popup for symbol hover
- `SignatureHelpPopup` - Function signature with parameter highlighting
- `DiagnosticMarker` - Gutter icons (E/W/I/H) for diagnostic severity
- `ErrorLens` - Inline diagnostic messages at end of lines (VS Code style)
- `CodeActionsMenu` - Quick fix/refactor actions with preferred action indicators
- `LocationPicker` - Multiple location picker for goto operations

**Snapshot Types (thread-safe for UI):**
- `DiagnosticSnapshot` - Diagnostic with line, columns, message, severity
- `CompletionItemSnapshot` - Completion with kind, label, detail, documentation
- `HoverSnapshot` - Hover content with markdown support
- `SignatureHelpSnapshot` - Signature with parameters and active index
- `InlayHintSnapshot` - Inline type hints with position and padding
- `LocationSnapshot` - File location with line, column, preview
- `CodeActionSnapshot` - Code action with title, kind, preferred flag

### State Management
- Added LSP state fields to `EditorContext`:
  - `completion_visible`, `completion_items`, `completion_selected`
  - `hover_visible`, `hover_content`
  - `signature_help_visible`, `signature_help`
  - `code_actions_visible`, `code_actions`, `code_action_selected`
  - `location_picker_visible`, `locations`, `location_selected`
  - `inlay_hints_enabled`
- Added `LspResponse` enum for async response handling
- Added command handlers for all LSP operations

### Commands Added
- Completion: `TriggerCompletion`, `CompletionUp/Down/Confirm/Cancel`
- Hover: `TriggerHover`, `CloseHover`
- Goto: `GotoDefinition`, `GotoReferences`, `GotoTypeDefinition`, `GotoImplementation`
- Locations: `LocationConfirm`, `LocationCancel`, `LocationUp/Down`
- Code Actions: `ShowCodeActions`, `CodeActionConfirm/Cancel/Up/Down`
- Diagnostics: `NextDiagnostic`, `PrevDiagnostic`
- Format: `FormatDocument`, `RenameSymbol`
- Inlay Hints: `ToggleInlayHints`, `RefreshInlayHints`
- Signature Help: `TriggerSignatureHelp`, `CloseSignatureHelp`

### Keybindings Added
- **Normal mode:**
  - `K` - Trigger hover
  - `]d` - Next diagnostic
  - `[d` - Previous diagnostic
- **g prefix (goto):**
  - `gd` - Goto definition
  - `gr` - Goto references
  - `gy` - Goto type definition
  - `gi` - Goto implementation
- **Space leader:**
  - `Space a` - Show code actions
  - `Space f` - Format document
  - `Space i` - Toggle inlay hints
- **Insert mode:**
  - `Ctrl+Space` - Trigger completion
  - `(` - Trigger signature help
- **Completion/Location/Code Actions modes:**
  - `Up/Down`, `j/k` - Navigate
  - `Enter` - Confirm
  - `Esc` - Cancel

### CSS Styles Added
- Diagnostic gutter column with severity-colored markers
- Error lens inline messages (dimmed, right-aligned)
- Completion popup with kind badges and selection highlight
- Hover popup with max dimensions and scrolling
- Signature help with parameter highlighting
- Code actions menu with preferred action styling
- Location picker with file path and preview
- Inlay hint styling (dimmed, italic for types)

### Operations Added
- `LspOps` trait on `EditorContext`:
  - `next_diagnostic()` - Jump to next diagnostic
  - `prev_diagnostic()` - Jump to previous diagnostic
  - `get_diagnostics()` - Get all diagnostics for document

### Technical Notes
- All LSP snapshot types are Clone + Send + Sync for thread-safe UI rendering
- Multi-key sequence handling via `PendingKeySequence` enum (g, ], [, Space prefixes)
- Diagnostic navigation works immediately; other LSP features await client integration
- Comments document where actual LSP client calls would be made

### Files Created
- `src/lsp/mod.rs` - LSP module re-exports
- `src/lsp/types.rs` - Thread-safe snapshot types
- `src/components/diagnostics.rs` - Diagnostic display components
- `src/components/completion.rs` - Completion popup
- `src/components/hover.rs` - Hover popup
- `src/components/signature_help.rs` - Signature help popup
- `src/components/code_actions.rs` - Code actions menu
- `src/components/location_picker.rs` - Location picker
- `src/components/inlay_hints.rs` - Inlay hints utilities
- `src/keybindings/completion.rs` - Completion/location/code actions handlers
- `src/operations/lsp.rs` - LSP operations trait

### Files Modified
- `src/state/mod.rs` - LSP state, command handling, snapshot collection
- `src/state/types.rs` - LSP commands and snapshot fields
- `src/components/mod.rs` - Export new components
- `src/components/editor_view.rs` - Integrate diagnostic gutter
- `src/keybindings/mod.rs` - Export new handlers
- `src/keybindings/normal.rs` - LSP keybindings (g prefix, brackets, space leader)
- `src/keybindings/insert.rs` - Ctrl+Space and `(` triggers
- `src/app.rs` - Multi-key handling, LSP component rendering
- `src/operations/mod.rs` - Export LspOps
- `assets/head.html` - CSS for all LSP components

### Next Steps
1. Integrate actual LSP client (helix-lsp) for server communication
2. Implement async request/response flow via command channel
3. Connect completion, hover, goto operations to language servers
4. Add document symbol and workspace symbol pickers

---

## 2026-02-01: Diagnostic Indicator Improvements

### Progress
- Implemented severity-colored lightbulb indicator
  - Lightbulb color now reflects the highest diagnostic severity on the cursor line
  - Error: `#e06c75` (red), Warning: `#e5c07b` (yellow), Info: `#61afef` (blue), Hint: `#56b6c2` (cyan)
  - Falls back to yellow when code actions exist but no diagnostic
- Consolidated indicator gutter to single position
  - Both lightbulb and diagnostic marker now use bottom-right position
  - Removed separate `indicator-code-action` CSS class
  - Cleaner code with `else if` logic
- Fixed multiple diagnostics per line not showing underlines
  - Added `diagnostics_for_line()` function to get all diagnostics for a line
  - Changed `Line` component to accept `Vec<DiagnosticSnapshot>` instead of `Option`
  - All diagnostics on a line now render their own underline
- Added severity-based rendering order for overlapping diagnostics
  - Diagnostics sorted by severity (ascending) before rendering
  - Higher severity underlines render last (on top)
  - Ensures error underlines are visible over warning underlines when they overlap

### Files Modified
- `src/components/diagnostics.rs` - Added `diagnostics_for_line()` function
- `src/components/mod.rs` - Exported new function
- `src/components/editor_view.rs` - Consolidated indicator, multiple underlines, severity sorting
- `assets/styles.css` - Removed unused `indicator-code-action`, added hover to `indicator-diagnostic`

### Technical Notes
- The `DiagnosticSeverity` enum is ordered Hint < Info < Warning < Error
- Sorting ascending means Error renders last (appears on top in CSS stacking)
- ErrorLens still shows only the highest severity diagnostic message per line

---

## 2026-02-01: Confirmation Dialog

### Progress
- Implemented confirmation dialog for quit/close with unsaved changes
  - Modal overlay centered on screen with backdrop
  - Title, message, and configurable buttons
  - Keyboard shortcuts: `y`/`Y`/`Enter` (confirm), `n`/`N` (deny), `Esc` (cancel)
  - Buttons show keyboard shortcuts as badges

### Use Cases
- `:q` with unsaved changes → "Save & Quit" / "Don't Save" / "Cancel"
- `:bd` with unsaved changes → "Close" / "Cancel"
- `:q!` and `:bd!` → Force quit/close without dialog (unchanged)

### Files Created
- `src/components/confirmation_dialog.rs` - Modal dialog component
- `src/keybindings/confirmation.rs` - Keybinding handler

### Files Modified
- `src/state/types.rs` - Added `ConfirmationAction`, `ConfirmationDialogSnapshot`, `EditorCommand` variants
- `src/state/mod.rs` - Added confirmation dialog state and command handlers
- `src/operations/buffer.rs` - `try_quit()` and `close_current_buffer()` now show confirmation dialog
- `src/keybindings/mod.rs` - Exports `handle_confirmation_mode`
- `src/components/mod.rs` - Exports `ConfirmationDialog`
- `src/app.rs` - Integrated confirmation dialog in key handler and render tree
- `assets/styles.css` - Added `.confirmation-dialog-*` styles
- `CLAUDE.md` - Updated documentation

### Technical Notes
- Confirmation dialog takes highest priority in key handling (before input dialog)
- `ConfirmationAction` enum determines what happens on confirm/deny
- Dialog state stored in `EditorContext` and snapshotted in `EditorSnapshot`

---

## 2026-02-01: Symbol Picker

### Progress
- Implemented document symbols picker (`Space+s`)
  - Shows all symbols (functions, classes, structs, etc.) in the current file
  - Uses LSP `textDocument/documentSymbol` request
  - Handles both flat and nested DocumentSymbol responses
- Implemented workspace symbols picker (`Space+S`)
  - Shows symbols across all files in the workspace
  - Uses LSP `workspace/symbol` request
  - Opens file and navigates to symbol on selection
- Reuses existing `GenericPicker` infrastructure
  - Added `PickerMode::DocumentSymbols` and `PickerMode::WorkspaceSymbols`
  - Fuzzy filtering with match highlighting
  - Keyboard navigation (arrows, Enter, Esc)

### Symbol Icons
- Added Lucide icons for symbol types:
  - Function: `SquareFunction` (blue)
  - Method: `Code` (blue)
  - Class: `Blocks` (yellow)
  - Struct: `Braces` (yellow)
  - Enum: `Layers` (yellow)
  - Interface: `Component` (yellow)
  - Variable: `Variable` (red)
  - Constant: `Hash` (orange)
  - Field: `Code` (red)
  - Module: `Package` (purple)

### Files Created/Modified
- `src/lsp/types.rs` - Added `SymbolKind`, `SymbolSnapshot`, `LspResponse` variants
- `src/lsp/conversions.rs` - Added `convert_document_symbols()`, `convert_workspace_symbols()`
- `src/lsp/mod.rs` - Exported new types and functions
- `src/state/types.rs` - Extended `PickerIcon`, `PickerMode`, `EditorCommand`
- `src/state/mod.rs` - Added `symbols` field, trigger methods, command/response handling
- `src/operations/picker_ops.rs` - Added `goto_line_column()`, symbol picker confirm handling
- `src/keybindings/normal.rs` - Added `Space+s` and `Space+S` bindings
- `src/components/picker/item.rs` - Added symbol icons and colors
- `src/components/picker/generic.rs` - Added picker titles for symbol modes

### Technical Notes
- Symbol navigation uses 1-indexed line/column from LSP, converted to 0-indexed for editor
- Workspace symbols open the target file before navigating
- Symbols are stored in `EditorContext.symbols` and converted to `PickerItem`s for display
- `SymbolKind` maps LSP symbol kinds to `PickerIcon` variants for appropriate icons

---

## 2026-02-01: Save As Dialog and New File Command

### Progress
- Implemented `:new` / `:n` command to create scratch buffers
- Implemented Save As dialog using native OS file picker
  - Opens when running `:w` on a scratch buffer (no path)
  - Uses `rfd` crate's `AsyncFileDialog` for non-blocking native dialog
  - Properly updates buffer name and path after saving

### Implementation Details

**New File Command (`:new` / `:n`):**
- Added `create_new_buffer()` method to `BufferOps` trait
- Calls `editor.new_file(Action::Replace)` to create scratch buffer
- Buffer shows as "[scratch]" in buffer bar

**Save As Dialog:**
- Added `rfd = "0.15"` dependency for native file dialogs
- Used async dialog to avoid blocking UI thread
- Flow: `:w` → detect scratch buffer → spawn async dialog → send `SaveDocumentToPath` command
- After save, `doc.set_path()` updates the document's internal path

### Files Modified
- `Cargo.toml` - Added `rfd` dependency
- `src/state/types.rs` - Added `SaveDocumentToPath(PathBuf)` command
- `src/state/mod.rs` - Added `show_save_as_dialog()` method, handled new command
- `src/operations/buffer.rs` - Added `create_new_buffer()`, call `set_path()` after save
- `src/operations/cli.rs` - Added `:new`/`:n` command, modified `:w` for scratch buffers

### Technical Notes
- Using `AsyncFileDialog` instead of `FileDialog` to avoid blocking the main thread
- The async result is sent back via `command_tx.send(EditorCommand::SaveDocumentToPath(path))`
- `doc.set_path()` is called after save to update the document's path, which:
  - Updates the buffer name in the buffer bar
  - Prevents Save As dialog from showing on subsequent `:w` calls

---

## 2026-02-01: Diagnostics Picker

### Progress
- Implemented document diagnostics picker (`Space+d`)
  - Shows all diagnostics for the current file
  - Sorted by line number
  - Navigate and jump to diagnostic location
- Implemented workspace diagnostics picker (`Space+D`)
  - Shows diagnostics from all open files
  - Sorted by severity (errors first), then file, then line
  - Opens file and jumps to diagnostic on selection
- Display format includes severity badge and diagnostic code:
  - `[error E0308] mismatched types expected 'String', found integer`
  - `[warn] unused variable 'x'`
  - `[hint] expected due to this`

### Bug Fix: Picker Fuzzy Match Highlighting
- Fixed bug where fuzzy match highlighting showed wrong characters
- Root cause: When secondary field (e.g., `test_error.rs:7`) had better match score than display,
  the code incorrectly applied secondary's indices to display text
- Example: Search "to" in secondary `test_error.rs:7` gave indices [0, 8],
  which highlighted `[` and `0` in the display `[error E0308]...`
- Fix: When secondary match wins, still use display's match indices for highlighting
- This was a pre-existing bug that affected all pickers (file, buffer, symbol, diagnostic)

### Files Modified
- `src/state/types.rs` - Added `PickerMode::DocumentDiagnostics/WorkspaceDiagnostics`,
  `PickerIcon::DiagnosticError/Warning/Info/Hint`, `EditorCommand::Show*Diagnostics`
- `src/lsp/types.rs` - Added `DiagnosticPickerEntry` struct
- `src/lsp/mod.rs` - Exported `DiagnosticPickerEntry`
- `src/state/mod.rs` - Added `picker_diagnostics` field, `show_*_diagnostics_picker()` methods,
  `populate_diagnostic_picker_items()`, helper functions
- `src/operations/picker_ops.rs` - Added diagnostic picker confirm handling, fixed highlight bug
- `src/keybindings/normal.rs` - Added `Space+d` and `Space+D` bindings
- `src/components/picker/item.rs` - Added diagnostic icons and colors (icon uses severity color,
  text uses neutral for better highlight visibility)
- `src/components/picker/generic.rs` - Added picker titles for diagnostic modes

### Technical Notes
- Diagnostics are collected from `doc.diagnostics()` and converted to `DiagnosticPickerEntry`
- Severity sorting uses helper function `get_severity_sort_key()` (Error=0, Warning=1, Info=2, Hint=3)
- Diagnostic code conversion handles `NumberOrString` enum from helix-core
- Icon color reflects severity, text uses neutral `#abb2bf` so fuzzy highlighting is visible

---

## 2026-02-01: Global Search Picker

### Progress
- Implemented global search picker (`Space+/`)
  - Searches for text patterns across all files in the workspace
  - Uses `grep-regex`, `grep-searcher`, and `grep-matcher` crates
  - Respects `.gitignore` patterns via the `ignore` crate
  - Smart case detection: lowercase patterns are case-insensitive, uppercase triggers case-sensitive

### Features
- **In-memory search**: Open documents are searched in their in-memory state (shows unsaved changes)
- **Cancellation support**: Cancel running searches with Escape or by starting a new search
- **Batch streaming**: Results stream in batches of 50 for UI responsiveness
- **Result limit**: Maximum 1000 results to avoid memory issues
- **Binary detection**: Automatically skips binary files

### Workflow
1. Press `Space+/` to open the global search picker
2. Type a regex pattern (e.g., "fn main", "TODO")
3. Press Enter to execute the search
4. Results appear with file path, line number, and line content
5. Navigate with arrows, press Enter to open file at that line
6. Press Escape to cancel

### Files Created/Modified
- `Cargo.toml` - Added `grep-regex`, `grep-searcher`, `grep-matcher` dependencies
- `src/state/types.rs` - Added `PickerMode::GlobalSearch`, `PickerIcon::SearchResult`,
  `GlobalSearchResult` struct, `EditorCommand::ShowGlobalSearch/GlobalSearchExecute/GlobalSearchResults/GlobalSearchComplete`
- `src/state/mod.rs` - Added global search state fields, command handlers, picker cancel cleanup
- `src/operations/picker_ops.rs` - Added `show_global_search_picker()`, `execute_global_search()`,
  `cancel_global_search()`, `update_global_search_picker_items()`, `execute_global_search_blocking()`,
  GlobalSearch handling in `picker_confirm()`
- `src/keybindings/normal.rs` - Added `Space+/` keybinding
- `src/components/picker/item.rs` - Added `TextSearch` icon (green) for search results
- `src/components/picker/generic.rs` - Added "Global Search" title, "search/open" help text,
  contextual empty state messages

### Technical Notes
- Search runs on `tokio::task::spawn_blocking` since it's CPU-bound (file walking/grep)
- Uses `tokio::sync::watch` channel for cancellation signaling
- Open documents are collected before spawning the task and searched in-memory
- Results are sent back via the command channel in batches for progressive UI updates
- Smart case: `pattern.chars().any(|c| c.is_uppercase())` determines case sensitivity

---

## 2026-02-01: LSP References & Definitions Picker

### Progress
- Converted LSP References (`gr`) and Definitions (`gd`) from using the standalone `LocationPicker`
  component to using the `GenericPicker` infrastructure
- Both now have fuzzy filtering, match highlighting, count display, and windowing (15 visible items)

### Features
- **References picker** (`gr`): Shows all references to symbol under cursor
  - Blue Link2 icon for reference locations
  - Title: "References"
- **Definitions picker** (`gd`): Shows definitions when multiple exist
  - Purple FileCode icon for definition locations
  - Title: "Definitions"
- **Single result optimization**: If only one location is found, jumps directly without showing picker
- **Consistent UI**: Same look and feel as other pickers (symbols, diagnostics, global search)

### Files Modified
- `src/state/types.rs` - Added `PickerMode::References`, `PickerMode::Definitions`,
  `PickerIcon::Reference`, `PickerIcon::Definition`
- `src/state/mod.rs` - Modified `LspResponse::References` and `LspResponse::GotoDefinition`
  handling to use GenericPicker instead of LocationPicker
- `src/operations/picker_ops.rs` - Added `update_references_picker_items()`, `show_references_picker()`,
  `update_definitions_picker_items()`, `show_definitions_picker()`, combined handling in `picker_confirm()`
- `src/components/picker/item.rs` - Added `Link2` and `FileCode` icon imports and rendering
- `src/components/picker/generic.rs` - Added "References" and "Definitions" titles

### Technical Notes
- References and Definitions share the same confirm handler logic since both use the `locations` field
- The `LocationPicker` component is now unused for these features (could be deprecated)
- Display format: `relative/path/file.rs:line:column` with preview text as secondary

---

## 2026-02-01: Scrollbar Diagnostic Marker Improvements

### Progress
- Fixed issue where error markers were hidden behind hint markers in the scrollbar
- Added conditional thumb rendering for small files that fit in the viewport
- Implemented line-aligned marker positioning for small files

### Issues Fixed
- **Error marker not visible**: Multiple diagnostics at similar positions caused lower-severity
  markers (hint) to render on top of higher-severity markers (error)
- **Unnecessary scrollbar thumb**: Small files showed a scrollbar thumb even when content fit in viewport

### Implementation Details
- **Severity ordering**: Diagnostics are now sorted by severity (ascending) before rendering,
  so errors render last and appear on top in the DOM
- **CSS z-index by severity**: Each severity level has its own z-index (hint=2, info=3, warning=4, error=5)
  providing a double-layered approach to ensure errors are always visible
- **Conditional thumb**: Thumb only renders when `total_lines > viewport_lines`
- **Line-aligned positioning**: For small files, markers use pixel-based positioning (`8px + line * 21px`)
  to align with actual editor line positions; large files continue to use percentage-based positioning

### Files Modified
- `src/components/scrollbar.rs` - Added severity sorting, conditional thumb, line-aligned positioning
- `assets/styles.css` - Added severity-specific z-index values for scrollbar markers

### Technical Notes
- Line height constant: `21.0px` (1.5em at 14px font-size)
- Content padding constant: `8.0px` (matches `.content` padding)
- `DiagnosticSeverity` already implements `Ord` with Hint < Info < Warning < Error

---

## 2026-02-01: Scrollbar Search Result Markers

### Progress
- Implemented search result markers in the scrollbar when using `/` search
- Yellow markers appear at positions where search matches are found
- Markers provide visual overview of all match locations across the document

### Implementation Details
- **Search match collection**: Added `collect_search_match_lines()` function that finds all lines
  containing matches for the current search pattern (case-insensitive)
- **Deduplicated markers**: Only one marker per line, even with multiple matches on the same line
- **Z-index hierarchy**: Search markers (z-index: 1) render below diagnostic markers
  (hint=2, info=3, warning=4, error=5) so important diagnostics remain visible
- **Semi-transparent thumb**: Scrollbar thumb is now 50% transparent so markers are always visible
- **Click-ready**: Markers have `pointer-events` enabled and `cursor: pointer` for future
  click-to-navigate functionality

### Files Modified
- `src/operations/search.rs` - Added `collect_search_match_lines()` public function
- `src/operations/mod.rs` - Exported the new function
- `src/state/types.rs` - Added `search_match_lines: Vec<usize>` to `EditorSnapshot`
- `src/state/mod.rs` - Collect and pass search match lines to snapshot
- `src/components/scrollbar.rs` - Added `search_match_lines` prop and search marker rendering
- `src/components/editor_view.rs` - Pass `search_match_lines` to Scrollbar component
- `assets/styles.css` - Added `.scrollbar-marker-search` styling, made thumb semi-transparent

### Design Decisions
- **Color**: Yellow/gold (`#e5c07b`) matches existing search theme colors
- **Timing**: Markers appear after search is executed (Enter), not during typing
- **Thumb transparency**: 50% opacity allows markers to show through while maintaining
  visual indication of viewport position

---

## 2026-02-01: Critical Bug Fix - Byte vs Char Position in Search

### The Bug
Search (`/pattern`) and next/previous match (`n`/`N`) were jumping to wrong positions in files
containing multi-byte UTF-8 characters (like "→" arrows or emoji). The cursor would land
at incorrect positions, making search unreliable.

### Root Cause
The `do_search()` function was mixing **byte positions** (from `String::find()`) with
**char positions** (used by helix's `Selection`). In Rust:
- `String::find()` returns byte offsets
- `String::len()` returns byte length
- But helix's Selection expects char positions

For ASCII text, byte == char, so it worked. But UTF-8 multi-byte characters (like "→" = 3 bytes)
caused the positions to diverge.

### The Fix
Use Rope's efficient conversion methods:
- `rope.char_to_byte(char_pos)` - convert char index to byte index for string slicing
- `rope.byte_to_char(byte_pos)` - convert search result back to char index for Selection
- `pattern.chars().count()` instead of `pattern.len()` for char length

These methods are O(log n) using Rope's internal structure, vs O(n) for manual iteration.

### Key Learning: Byte vs Char in helix-dioxus
When working with text in helix-dioxus, always be aware of the index type:

| Source | Returns | Use For |
|--------|---------|---------|
| `String::find()` | byte position | String slicing only |
| `String::len()` | byte length | String operations only |
| `str.chars().count()` | char count | When you need char length |
| `Selection::cursor()` | char position | Rope operations |
| `Rope::char_to_byte()` | byte position | O(log n) conversion |
| `Rope::byte_to_char()` | char position | O(log n) conversion |
| `Rope::char_to_line()` | line number | Line-based operations |

**Rule**: Never pass a byte position to Selection, and never slice a String with a char position.

### Files Modified
- `src/operations/search.rs` - Fixed `do_search()` to properly convert between byte and char positions

### Performance Note
`collect_search_match_lines()` still uses O(n) char counting per match. For very large documents
with many matches, consider pre-computing a byte-to-char lookup table. Added TODO comment.

---

## 2026-02-01: Scrollbar Marker Click & Tooltip

### Progress
- Added click-to-navigate on scrollbar markers (both search and diagnostic markers)
- Added hover tooltip showing marker details (severity, line number, message)
- Changed click behavior to use `GoToLine` command (moves cursor) vs `ScrollToLine` (only scrolls)

### Implementation Details
- **New command**: `EditorCommand::GoToLine(usize)` - moves cursor to line and scrolls view
- **Marker tooltip**: Shows on hover with severity (Error/Warning/Search match), line number, and message
- **Tooltip positioning**: Positioned at the marker's vertical position, to the left of scrollbar
- **Message truncation**: Long diagnostic messages truncated to 80 chars with "..."

### Files Modified
- `src/state/types.rs` - Added `GoToLine` command, added `message` field to `ScrollbarDiagnostic`
- `src/state/mod.rs` - Handle `GoToLine` command, populate diagnostic message
- `src/components/scrollbar.rs` - Added `MarkerTooltip` struct, hover handlers, tooltip rendering
- `assets/styles.css` - Added scrollbar tooltip styles with severity-colored headers

---

## 2026-02-01: Picker Mouse Click Support

### Progress
- Added mouse click support to picker items for direct selection

### Implementation Details
- Added `on_click` prop to `PickerItemRow` component
- `GenericPicker` passes click handlers that call `PickerConfirmItem(idx)`
- New `EditorCommand::PickerConfirmItem(usize)` selects and confirms the item

### Files Modified
- `src/components/picker/item.rs` - Added `on_click` prop
- `src/components/picker/generic.rs` - Pass click handlers to items
- `src/state/types.rs` - Added `PickerConfirmItem` command
- `src/state/mod.rs` - Handle `PickerConfirmItem` command

---

## 2026-02-01: File-Type Icons & Missing Commands

### Progress
- Implemented file-type specific icons in buffer bar
  - Icons are determined by file extension
  - Supports 30+ file types with appropriate Lucide icons
- Implemented 10 commonly-used missing commands

### File-Type Icons (buffer_bar.rs)
- Added `file_icon()` helper function that maps extensions to Lucide icons:
  - **Code files**: `.rs`, `.js`, `.ts`, `.py`, `.go`, `.java`, etc. → `FileCode`
  - **Config files**: `.toml`, `.yaml`, `.json` → `Braces`
  - **Documentation**: `.md`, `.txt`, `.rst` → `FileText`
  - **Web markup**: `.html`, `.xml`, `.svg` → `Code`
  - **Stylesheets**: `.css`, `.scss`, `.sass` → `Palette`
  - **Shell scripts**: `.sh`, `.bash`, `.zsh` → `Terminal`
  - **Images**: `.png`, `.jpg`, `.gif` → `Image`
  - **Git files**: `.gitignore`, `.gitattributes` → `GitBranch`
  - **Lock files**: `.lock` → `Lock`
  - **Default**: `FileText`

### New Commands Implemented
| Command | Aliases | Description |
|---------|---------|-------------|
| `:reload` | `:rl` | Reload file from disk |
| `:write-all` | `:wa` | Save all modified buffers |
| `:quit-all` | `:qa` | Close all buffers and quit |
| `:quit-all!` | `:qa!` | Force close all and quit |
| `:buffer-close-all` | `:bca` | Close all buffers |
| `:buffer-close-all!` | `:bca!` | Force close all buffers |
| `:buffer-close-others` | `:bco` | Close all except current |
| `:cd` | `:change-current-directory` | Change working directory |
| `:pwd` | - | Print working directory |
| `:earlier` | - | Undo to earlier state (N steps) |
| `:later` | - | Redo to later state (N steps) |

### Files Modified
- `src/components/buffer_bar.rs` - Added `file_icon()` function and new icon imports
- `src/state/types.rs` - Added `EditorCommand` variants for new commands
- `src/operations/cli.rs` - Added command parsing for all new commands
- `src/operations/buffer.rs` - Added `BufferOps` implementations for buffer/directory commands
- `src/operations/editing.rs` - Added `earlier()` and `later()` implementations using `UndoKind`
- `src/state/mod.rs` - Added command handlers in `handle_command()`

### Technical Notes
- `:cd` with no arguments navigates to home directory using `helix_stdx::path::home_dir()`
- `:earlier` and `:later` use `helix_core::history::UndoKind::Steps(n)` for multi-step undo/redo
- `:wa` iterates through all modified documents and saves each one
- `:qa` checks for unsaved changes and shows confirmation dialog if any exist

---

## 2026-02-07: Core Tutor Commands — 21 New Editing/Motion/Surround Commands

### Progress
Implemented the most impactful missing commands taught in the Helix tutor, bringing the Dioxus frontend much closer to feature parity with the terminal UI.

### Commands Added

**Fixed keybindings (correctness fix):**
- `;` → `CollapseSelection` (was incorrectly mapped to RepeatLastFind)
- `,` → `KeepPrimarySelection` (was incorrectly mapped to ReverseLastFind)
- `Alt-.` → `RepeatLastFind` (correct Helix binding for repeat last find/till)
- Removed `ReverseLastFind` command (no default Helix binding)

**Word motions:**
- `e` — move to word end (`MoveWordEnd`)
- `W`/`E`/`B` — WORD motions (whitespace-delimited words)
- All word motions work in both normal (move) and select (extend) modes

**Editing commands:**
- `c` — change selection (delete + enter insert mode)
- `I` — insert at first non-whitespace of line
- `r<char>` — replace characters in selection (two-step via `ReplacePrefix`)
- `R` — replace selection with yanked text (without updating clipboard)
- `J` — join lines (replace newlines + leading whitespace with space)

**Case commands:**
- `~` — toggle case
- `` ` `` — convert to lowercase
- `` Alt+` `` — convert to uppercase

**Bracket matching:**
- `mm` — jump to matching bracket (uses tree-sitter when available, plaintext fallback)

**Select inside/around:**
- `mi<char>` — select inside bracket/quote pair (e.g., `mi(`, `mi"`)
- `ma<char>` — select around bracket/quote pair

**Surround operations:**
- `ms<char>` — add surround pair around selection
- `md<char>` — delete surround pair
- `mr<old><new>` — replace surround pair (3-key sequence)

### Architecture: Nested Pending Key Sequences
The `m` prefix introduces multi-level pending key sequences:
- `PendingKeySequence::MatchPrefix` → waits for `m`/`i`/`a`/`s`/`d`/`r`
- `MatchInside`/`MatchAround`/`MatchSurround`/`MatchDeleteSurround` → waits for char
- `MatchReplaceSurroundFrom` → waits for old char → `MatchReplaceSurroundTo(old)` → waits for new char

This is the first 3-key sequence (`mr<old><new>`) in the codebase.

### Implementation Details
- **Word motions** use `helix_core::movement::move_next_word_end`, `move_next_long_word_start/end`, `move_prev_long_word_start`
- **Replace char** uses `Transaction::change_by_selection`, preserving newlines
- **Join lines** builds changes for each line break in selection range
- **Case ops** use `Transaction::change_by_selection` with char-level transformations
- **Match bracket** uses `find_matching_bracket_fuzzy()` (with syntax) or `find_matching_bracket_plaintext()` (without)
- **Select inside/around** uses `helix_core::surround::find_nth_pairs_pos()`
- **Surround delete/replace** uses `helix_core::surround::get_surround_pos()`
- **Surround add** wraps selection content with open/close chars via `Transaction::change_by_selection`

### Files Modified (11 files, +769/-16 lines)
- `state/types.rs` — +15 EditorCommand variants, +8 PendingKeySequence variants
- `state/mod.rs` — +15 match arms in `handle_command()`, `EnterInsertModeLineStart` impl
- `operations/movement.rs` — `move_word_end`, 3 WORD motions, `match_bracket`
- `operations/editing.rs` — `change_selection`, `replace_char`, `join_lines`, `toggle_case`, `to_lower/uppercase`, 3 surround ops
- `operations/selection.rs` — 4 extend word/WORD ops, `collapse_selection`, `keep_primary_selection`, `select_inside/around_pair`
- `operations/clipboard.rs` — `replace_with_yanked`
- `keybindings/normal.rs` — +15 key mappings, Alt modifier handling
- `keybindings/select.rs` — +8 key mappings
- `app.rs` — +10 pending key sequence handlers
- `components/keybinding_help.rs` — context-aware hints for all new pending states
- `CLAUDE.md` — updated roadmap and documentation

### Verification
- `cargo check` — passes
- `cargo fmt --all --check` — clean
- `cargo test -p helix-dioxus --lib` — all 29 tests pass
- No new clippy warnings in helix-dioxus

---

## 2026-02-07 → 2026-02-14: Feature Parity Sprint

### Overview
Massive sprint bringing helix-dioxus to 100% keybinding coverage and full LSP integration. Tests grew from 29 to 319. Changes are grouped thematically below.

### Multi-Selection, Regex, and Tree-Sitter Selection
- Multi-selection rendering with regex select/split modes
- Tree-sitter expand/shrink selection (`Alt-o`/`Alt-i`)
- Select-mode g-prefix and 10 additional keybindings
- Test coverage doubled from 87 to 175 tests

### Jump List, Alt-Key Fix, and LSP Animations
- Jump list with gutter markers, picker (`Space j`), and `C-o`/`C-i`/`C-s` navigation
- Fixed Alt+key bindings on macOS (Option key composing special characters)
- LSP icon animations for transitioning states (blinking/spinning)

### Picker Preview and Shell Integration
- Picker file preview panel with syntax highlighting
- Shell integration (pipe selections to shell commands)
- Word jump (`gw`) with two-char EasyMotion-style labels
- Insert mode keybindings: `C-r`, `C-s`, `S-Tab`

### Theme System and Cursor Visibility
- Live theme preview in picker with dynamic CSS variable system
- Cursor glow pulse and improved selection opacity
- Macro recording/replay (`Q`/`q`)

### CLI Commands and Configuration
- 7 missing CLI commands (`:sort`, `:reflow`, `:config-open`, `:encoding`, etc.)
- 7 planned enhancements batch (various improvements)
- File explorer picker (`Space e` / `Space E`)
- VCS integration (`]g`/`[g` navigation, `Space g` changed files, gutter diff markers)
- `:config-reload`, `:set`, `:toggle` commands for runtime configuration
- `:format` and `:lsp-restart` wired as `:` commands

### UI Components
- Command mode autocompletion popup with fuzzy filtering
- Code action preview panel with two-column diff display
- Reusable KbdKey component for consistent keyboard key styling
- Word/paragraph textobjects support in `mi`/`ma` selections
- Component reorganization into `lsp/` and `dialog/` subfolders

### Files Changed
- 50+ files modified across components, operations, keybindings, and state
- Tests: 29 → 319 passing

---

## 2026-02-15: Multi-Cursor Display

### Progress
- Display all cursors for multi-selection operations (`C`/`Alt-C`)
- Secondary cursors rendered with mode-specific CSS classes (`.cursor-secondary-normal`, `.cursor-secondary-select`, `.cursor-secondary-insert`)
- All selections now visible simultaneously in the editor view

---

## 2026-02-15: Markdown Rendering in LSP Popups

### Progress
- Created `src/components/lsp/markdown.rs` module for parsing and rendering markdown
- Hover and signature help popups now render markdown content with proper formatting
- Added syntax highlighting for fenced code blocks in hover popups
- Popups auto-dismiss on keypress for smoother editing flow

---

## 2026-02-15: dioxus-iconify Migration

### Progress
- Replaced `lucide-dioxus` crate with CLI-generated icon data via `dioxus-iconify`
- Created `src/icons/` module with `mod.rs` (Icon component + IconData struct), `lucide.rs` (37 Lucide icons), `logos.rs` (1 logo), `vscode_icons.rs` (37 VSCode file type icons)
- Icons are now stroke-based SVG constants generated at build time, no runtime dependency on external icon crate

---

## 2026-02-15: Image Preview in File Picker

### Progress
- Added real image preview in the file picker for image files (PNG, JPG, GIF, SVG, etc.)
- Preview panel shows the image with metadata (dimensions, file size)
- Reuses the existing `PickerPreviewPanel` infrastructure

---

## 2026-02-15: Command Picker Kbd Styling

### Progress
- Shortcut keys in the command picker are now rendered as `<kbd>` elements
- Added `.kbd-key-compact` CSS class for 20px height help bar integration
- Consistent keyboard key styling across the UI using the reusable KbdKey component

---

## 2026-02-15: Visible Whitespace and Rulers

### Progress
- Implemented visible whitespace rendering (spaces, tabs, newlines) controlled by editor config
- Added column ruler rendering (vertical guide lines at configured columns)
- Removed cursor glow pulse effect for cleaner appearance

---

## 2026-02-15: Bug Fixes

### open_line_below fix
- Fixed `open_line_below` (`o`) on unterminated last lines (lines without trailing newline)

### Inlay hints fix
- Re-request inlay hints from LSP after progress notifications complete
- Fixes stale hints after LSP server finishes background work

### Soft-wrap clipping fix
- Fixed last character being clipped on soft-wrapped lines

---

## 2026-02-16: Binary Entry Point Refactor

### Progress
- Moved binary entry point from `src/bin/dhx.rs` to standard `src/main.rs`
- Simplified build configuration — `cargo run -p helix-dioxus` works without extra flags

---

## 2026-02-16: Signal<EditorSnapshot> Refactor

### Progress
- Replaced manual version counter (`ReadSignal<usize>`) with `Signal<EditorSnapshot>` provided as Dioxus context
- Components now use `use_snapshot()` hook instead of receiving `version` prop
- After sending commands, callers invoke `app_state.process_and_notify(&mut signal)` to update the signal and trigger re-renders
- Eliminated prop drilling — components subscribe to the snapshot signal directly via context

### Decisions Made
- `use_snapshot()` returns a cloned `EditorSnapshot`, keeping the signal borrow short-lived
- `use_snapshot_signal()` returns the raw `Signal` for components that need to write back

---

## 2026-02-16: CSS include_str! Fix

### Progress
- Switched CSS loading from `document::Stylesheet { href: asset!() }` to `document::Style { {include_str!()} }`
- The `asset!()` macro requires the `dx` CLI to resolve paths; `include_str!()` works with standard `cargo build`
- Ensures the app builds and runs correctly without the Dioxus CLI toolchain

---

## Planned Enhancements

### Helix Commands & Modes
- [x] Buffer management commands (`:reload`, `:wa`, `:qa`, `:bca`, `:bco`)
- [x] Directory commands (`:cd`, `:pwd`)
- [x] History navigation (`:earlier`, `:later`)
- [x] Match mode (`m` prefix - matching brackets, surround)
- [x] Core motions (`e`, `W`/`E`/`B`, `I`, `c`, `r`, `R`, `J`, case ops)
- [x] Selection operations (`;` collapse, `,` keep primary, `mi`/`ma`)
- [x] Keybinding coverage (100% — all standard Helix bindings implemented)
- [x] Right/Left bracket modes (`]`/`[` prefix - next/prev item navigation)
- [x] Macro recording/replay (`Q`/`q`)
- [x] Shell integration (pipe selections to shell)
- [x] Word jump (`gw` — EasyMotion-style two-char labels)
- [x] Dot-repeat (`.` — replay last insert session)

### Configuration
- [x] Use helix configuration (`~/.config/helix/config.toml`)
- [x] Use language configuration (`languages.toml`)
- [x] User preferences (`:set`, `:toggle`, `:config-reload`)

### Standard UI Components
- [x] Toast notifications
- [x] Confirm dialogs
- [x] Rename prompt (for LSP rename)
- [x] Documentation popup (hover info with markdown rendering)
- [x] Keybinding help bar (context-aware hints)
- [x] Completion popup (LSP autocompletion)
- [x] Command mode autocompletion popup
- [x] KbdKey reusable component (physical key styling)
- [x] Error lens (inline diagnostics)
- [x] Emoji picker (`Ctrl+Cmd+Space`)
- [x] Buffer bar right-click context menu (Close, Close Others, Close All, Copy Path, Save)
- [x] Shared ModalOverlay and PopupMenu reusable components

### LSP Integration
- [x] LSP snapshot types (thread-safe for UI rendering)
- [x] LSP client integration (actual server communication)
- [x] Diagnostics display with Error Lens
- [x] Completion popup component
- [x] Hover popup with markdown rendering and syntax-highlighted code blocks
- [x] Signature help popup component
- [x] Code actions menu with two-column preview and diff display
- [x] Location picker component
- [x] Inlay hints utilities
- [x] Diagnostic navigation (`]d`, `[d`)
- [x] LSP keybindings (K, gd, gr, gy, gi, Space+a/f/i)
- [x] LSP icon animations (blinking/spinning for transitioning states)
- [x] `:format` and `:lsp-restart` commands

### Gutter Improvements
- [x] Git diff indicators (added/modified/removed lines)
- [x] Diagnostic indicators (error/warning icons)
- [x] Jumplist markers

### Multi-Cursor / Multi-Selection
- [x] Multi-selection rendering (all cursors visible for C/Alt-C)
- [x] Regex select/split
- [x] Tree-sitter expand/shrink selection (Alt-o/Alt-i)

### Picker Infrastructure
- [x] Preview pane (syntax-highlighted file content preview)
- [x] Symbol picker (document symbols via LSP)
- [x] Workspace symbol picker (project-wide symbols)
- [x] Global search picker (grep-based, `Space+/`)
- [x] Diagnostics picker (jump to errors/warnings)
- [x] References picker (LSP references, `gr`)
- [x] Definitions picker (LSP definitions, `gd`)
- [x] Command picker (all available commands)
- [x] Theme picker (with live preview via CSS variables)
- [x] Jumplist picker (navigation history)
- [x] Changed files picker (modified buffers)
- [x] File explorer picker (`Space e` / `Space E`)
- [x] Emoji picker (`Ctrl+Cmd+Space` / `:emoji`)
- [x] Scrollbar for long lists (picker track+thumb indicator)

### Theming
- [x] Dynamic CSS variable system
- [x] Live theme preview in picker
- [x] Material Icon Theme file icons (~40 file types)
- [x] Cursor glow pulse and selection opacity

### Remaining TODOs
- [ ] Scrollbar interactions (track click, thumb drag) — **BLOCKED** by Dioxus element height issue (see [SCROLLBAR_FIX_INVESTIGATION.md](SCROLLBAR_FIX_INVESTIGATION.md))
- [x] Buffer bar: hide option (`bufferline` config: `never`/`always`/`multiple`)
- [ ] macOS dock icon (needs .app bundle)

### Architecture Note
Split views are **not planned** for helix-dioxus. For multiple views, users should launch multiple editor instances. This keeps the architecture simpler and aligns with a single-document-focus paradigm.
DAP/Debug is **not supported** — `Space G` sub-menu will not be implemented.

---

## 2026-02-15: Emoji Picker

### Progress
- Added emoji picker using `emojis` crate v0.8 (Unicode 17.0)
- `Ctrl+Cmd+Space` in insert mode or `:emoji` command opens the picker
- Reuses existing `GenericPicker` with new `PickerMode::Emojis` and `PickerIcon::Emoji`
- Added `EditorCommand::InsertText(String)` for multi-codepoint text insertion (flags, ZWJ sequences)
- Added `KeyModifiers::SUPER` translation from `mods.meta()` for macOS Cmd key support
- Lucide `Smile` icon with `var(--warning)` color for picker items
- 5 new tests: 3 for `insert_text` (emoji, multi-codepoint, empty no-op), 2 for keybinding dispatch

### Files Changed (10)
- `Cargo.toml` — `emojis = "0.8"` dependency
- `keybindings/translate.rs` — meta → SUPER modifier
- `keybindings/insert.rs` — Ctrl+Cmd+Space binding + tests
- `state/types.rs` — `PickerIcon::Emoji`, `PickerMode::Emojis`, `InsertText`, `ShowEmojiPicker`
- `operations/editing.rs` — `insert_text()` method + tests
- `operations/picker_ops.rs` — `show_emoji_picker()`, confirm handler, command panel entry
- `operations/cli.rs` — `:emoji` command
- `components/picker/item.rs` — Emoji icon rendering
- `components/keybinding_help.rs` — insert mode hint
- `state/mod.rs` — command dispatch wiring

---

## 2026-02-15: Material Icon Theme File Icons

### Progress
- Replaced monochrome Lucide icons with colorful SVGs from Material Icon Theme (MIT licensed)
- Created `src/components/file_icons/` module with ~40 embedded SVG constants
- `icon_svg_for_filename()` matches special filenames first (Dockerfile, Makefile, .gitignore), then extension, then default
- `FileTypeIcon` and `FolderTypeIcon` Dioxus components render inline SVGs via `dangerous_inner_html`
- Integrated into buffer bar (14px) and file picker (16px)
- Symbol, diagnostic, VCS, and other picker icon types still use Lucide icons

### Icon Coverage (~40 icons)
- **Languages**: Rust, Python, JavaScript, TypeScript, Go, Java, C, C++, C#, Ruby, PHP, Swift, Kotlin, Scala, Lua, Elixir, Haskell, OCaml, Zig, Nim
- **Web**: HTML, CSS, SCSS, SVG
- **Data/Config**: JSON, YAML, TOML, XML, Markdown
- **Build/DevOps**: Docker, Shell, Makefile, Nix
- **Other**: Git, Lock, Image, Default file, Folder, Folder-open

### Issues Encountered
- Raw string literals `r#"..."#` failed because SVG content contains `"#` sequences (e.g., `fill="#ff7043"`); fixed by using `r##"..."##`
- `std::ptr::eq` for `const` string identity doesn't work reliably (const items are inlined); switched tests to `assert_eq!`

### Files Changed
- `src/components/file_icons/svgs.rs` — created: ~40 const SVG strings
- `src/components/file_icons/mod.rs` — created: components, mapping, 18 tests
- `src/components/mod.rs` — added module + re-exports
- `src/components/buffer_bar.rs` — removed `file_icon()`, use `FileTypeIcon`
- `src/components/picker/item.rs` — File/Folder/Buffer use Material icons, rest keep Lucide
- `assets/styles.css` — added `.file-type-icon` rules

---

## 2026-02-15: Dot-Repeat (`.` command)

### Progress
- Implemented the `.` (dot) command to repeat the last insert mode session
- Records the entry command (e.g., `i`, `a`, `o`, `c`) and all editing commands typed during insert mode
- On `.` press in normal mode, replays the entry command + recorded edits + ExitInsertMode
- Added `RepeatLastInsert` variant to `EditorCommand`
- Added `is_insert_recordable()` method to classify which commands get recorded (insert/delete/indent/comment)
- Recording uses mode-transition detection in `handle_command()` to start/stop automatically
- Replay guard (`replaying_insert`) prevents re-recording during playback

### Files Changed
- `src/state/types.rs` — `RepeatLastInsert` variant, `is_insert_recordable()`, 9 unit tests
- `src/state/mod.rs` — 4 recording fields on `EditorContext`, mode-transition detection in `handle_command()`, `replay_last_insert()`
- `src/keybindings/normal.rs` — `.` key mapping, 1 unit test
- `src/integration_tests.rs` — 5 integration tests (insert chars, preserves recording, open line below, backspace, no re-record during replay)
- `KEYBINDINGS.md` — added `.` to Normal Mode Matches

---

## 2026-02-21: Picker Scrollbar

### Progress
- Added a lightweight scrollbar to the picker item list to indicate scroll position within large filtered lists
- The picker renders a 15-item sliding window from potentially large lists — previously there was no visual indicator of position
- Created `PickerScrollbar` component using CSS-only track+thumb positioning
- Extracted `PICKER_WINDOW_SIZE` constant and `scrollbar_thumb_geometry()` pure function with unit tests for edge cases
- Replaced native `overflow-y: auto` with `overflow: hidden` (list is already server-side windowed)

### Files Created
- `src/components/picker/scrollbar.rs` — PickerScrollbar component

### Files Modified
- `src/components/picker/generic.rs` — Integrate scrollbar, use constant for window size
- `src/components/picker/mod.rs` — Export new component
- `src/state/types.rs` — `scrollbar_thumb_geometry()` function + unit tests
- `src/state/mod.rs` — Pass scroll position data to snapshot
- `assets/styles.css` — `.picker-scrollbar-track`, `.picker-scrollbar-thumb` styles

---

## 2026-02-21: Buffer Bar Right-Click Context Menu

### Progress
- Added right-click context menu to buffer bar tabs
- Menu actions: Close, Close Others, Close All, Copy Path, Save
- Context-aware disabled states:
  - Close Others disabled when only one buffer open
  - Save disabled when buffer is unmodified
  - Copy Path hidden for scratch buffers

### Files Modified
- `src/components/buffer_bar.rs` — Context menu rendering and event handling (+189 lines)
- `src/operations/buffer.rs` — Added `CopyPath` command support
- `src/state/types.rs` — Added `CopyPath` command variant
- `assets/styles.css` — `.context-menu-*` styles (`.context-menu`, `.context-menu-backdrop`, `.context-menu-item`, `.context-menu-item-disabled`, `.context-menu-separator`)

---

## 2026-02-21: JS-Based Inline Dialog Positioning and Shared Modal/Popup Components

### Progress
- Moved inline dialog positioning from Rust (async `document::eval` round-trip) to JavaScript
  - Dialogs now render with `visibility: hidden`, then `use_effect` calls `positionInlineDialogs()` which reads `getBoundingClientRect()` and sets `top`/`left`/`visibility` in one synchronous JS pass
  - Removes `CursorPixelPosition` struct and all `cursor_pos` props throughout the component tree
- Extracted shared reusable components:
  - `ModalOverlay` — shared overlay+backdrop+container for all modal dialogs
  - `PopupMenu` — positioned context menu with backdrop (used by buffer bar context menu)
- Consolidated duplicate CSS overlay styles across picker, confirmation, LSP dialog, location picker

### Files Created
- `src/components/modal_overlay.rs` — Reusable ModalOverlay component
- `src/components/popup_menu.rs` — Reusable PopupMenu component

### Files Modified (18 files, -648/+586 lines — net reduction)
- `assets/script.js` — Added `positionInlineDialogs()` function
- `assets/styles.css` — Consolidated overlay CSS, added `.modal-overlay`, `.modal-container` shared classes
- `src/components/inline_dialog/container.rs` — Simplified: removed Rust-side pixel positioning
- `src/components/buffer_bar.rs` — Use PopupMenu for context menu
- `src/components/dialog/confirmation.rs` — Use ModalOverlay
- `src/components/dialog/lsp_status.rs` — Use ModalOverlay
- `src/components/lsp/location_picker.rs` — Use ModalOverlay
- `src/components/picker/generic.rs` — Use ModalOverlay

### Technical Notes
- The JS-based approach eliminates the async eval round-trip that caused flickering on dialog open
- `positionInlineDialogs()` is defined in `script.js` and queries all `.inline-dialog` elements

---

## 2026-02-21: Completion Item Sorting, Filtering, and Scroll-into-View

### Progress
- Completion popup now sorts items by preselect flag → sort_text → original server index (previously showed raw LSP order, burying local variables)
- Typed prefix filters the completion list in real-time on each keystroke (`InsertChar`/`DeleteCharBackward`)
- Selected item scrolls into view during arrow-key navigation via `scrollInlineDialogItemIntoView()` JS function

### Files Modified
- `src/state/mod.rs` — Sorting logic, prefix filtering on keystroke, filter refresh
- `src/lsp/types.rs` — Added `sort_text` field to `CompletionItemSnapshot`
- `src/lsp/conversions.rs` — Populate `sort_text` from LSP response
- `src/components/inline_dialog/list.rs` — Added `id` attribute for scroll-into-view targeting
- `assets/script.js` — Added `scrollInlineDialogItemIntoView()` function

---

## 2026-02-21: Strip LSP Snippet Placeholders from Completion Text

### Progress
- Completion items from rust-analyzer often include snippet syntax (`$0`, `$1`, `${1:default}`) which was being inserted literally into the document
- Added `strip_snippet_placeholders()` function that removes tabstop markers and preserves default text from placeholders
- Examples: `clone()$0` → `clone()`, `fn ${1:name}($2)$0` → `fn name()`

### Files Modified
- `src/state/mod.rs` — Added `strip_snippet_placeholders()` with comprehensive unit tests (+128 lines)

### Technical Notes
- Function handles nested placeholders, escaped `\$`, and mixed plain text + snippet syntax
- Applied during `CompletionConfirm` command processing before text insertion

---

## 2026-02-21: Configurable Keymap System

### Progress
- Migrated Normal, Insert, and Select mode dispatch from hardcoded `handle_normal_mode`/`handle_insert_mode`/`handle_select_mode` handlers to a trie-based `DhxKeymaps` system in the `keymap/` module
- User keymaps loaded from `config.toml` `[keys]` section and merged with defaults via `merge_keys()`
- ~150 command registry entries mapping command names to `EditorCommand` variants
- Supports await-char commands (f/F/t/T/r), sticky nodes (g, Space, m, brackets), and multi-key sequences
- Deleted old hardcoded handler files: `keybindings/normal.rs`, `keybindings/insert.rs`, `keybindings/select.rs`

### Architecture
- `keymap/trie.rs` — `DhxKeyTrie`/`DhxKeyTrieNode` data structure with `TrieSearchResult` (Found, Prefix, NotFound)
- `keymap/command.rs` — `CommandSlot` enum (Single, AwaitChar), `AwaitCharKind`, command name → `EditorCommand` mapping
- `keymap/default.rs` — Default keymaps for all modes (normal, insert, select, g-prefix, Space-prefix, etc.)
- `keymap/mod.rs` — `DhxKeymapResult`, dispatch logic, key sequence state management

### Files Deleted
- `src/keybindings/normal.rs` — replaced by keymap trie dispatch
- `src/keybindings/insert.rs` — replaced by keymap trie dispatch
- `src/keybindings/select.rs` — replaced by keymap trie dispatch

### Technical Notes
- Dialog/prompt modes (command, search, picker, regex, completion, confirmation, input, shell) still use dedicated handlers in `keybindings/`
- The trie approach enables user customization without code changes — users can remap any key in any mode via `config.toml`

---

## 2026-02-21: Signature Help Parameter Highlighting

### Progress
- Active parameter in signature help popup is now visually highlighted
- Uses bold + accent color to distinguish the current parameter from the rest of the signature

---

## 2026-02-21: Buffer Bar Hide Option (bufferline config)

### Progress
- Wired helix-view's `bufferline` config (`never`/`always`/`multiple`) to control buffer bar visibility
- helix-dioxus overrides the default from `Never` to `Always` (tabs are a natural GUI element)
- Added `show_buffer_bar` to `EditorSnapshot`, computed from `editor.config().bufferline`
- Conditional rendering in `app.rs` — buffer bar is only rendered when `show_buffer_bar` is true
- Dynamic chrome height: viewport line computation accounts for buffer bar presence/absence (+1 line when hidden)
- Runtime support: `:set bufferline never|always|multiple` toggles the buffer bar live
- 4 integration tests covering all three modes

### Files Changed
- `src/state/mod.rs` — `load_editor_config()` override, `snapshot()` computation, dynamic `compute_viewport_lines()`
- `src/state/types.rs` — `show_buffer_bar: bool` field on `EditorSnapshot`
- `src/app.rs` — conditional `BufferBar {}` rendering
- `src/integration_tests.rs` — 4 new tests

---

## 2026-02-22: Performance Optimizations (use_memo, row keys, peek)

### Progress
- Added `use_memo` to `BufferBar` — extracts only `open_buffers` + `buffer_scroll_offset` (2 of ~50 fields), eliminating ~95% of unnecessary re-renders (cursor moves, mode changes, content edits no longer trigger)
- Added `use_memo` to `StatusLine` with `StatusLineData` struct — extracts 12 fields (mode, cursor, diagnostics, LSP), skips re-renders from content-only changes
- Removed `{version}` from `EditorView` row keys — version was incremented on every snapshot, making every key unique and defeating VDOM diffing entirely. For a 500-line file, this changes from destroying/recreating ~500 DOM subtrees to diffing only the ~5 that actually changed
- Changed `snapshot_signal.read()` to `.peek()` in `onkeydown` handler — makes non-subscribing intent explicit
- Updated `hooks.rs` doc comments to recommend `use_memo` pattern
- Updated CLAUDE.md with new "Performance: Memoization and Signal Usage" section

### Rationale
- Zero `use_memo` usage existed in the codebase despite `EditorSnapshot` having ~50 fields
- Components like `BufferBar` (2 fields) and `StatusLine` (12 fields) were re-rendering on every keystroke
- The `version` in row keys was the single biggest performance issue — every render destroyed and recreated the entire document DOM

### Files Changed
- `src/components/buffer_bar.rs` — `use_memo` replacing `use_snapshot()`
- `src/components/statusline.rs` — `StatusLineData` struct + `use_memo`
- `src/components/editor_view.rs` — removed `{version}` from `row_key`
- `src/app.rs` — `.read()` → `.peek()` in event handler
- `src/hooks.rs` — updated doc comments

---

## 2026-02-22: Fix Picker Scroll Not Following Keyboard Navigation

### Progress
- Fixed `use_effect` in `GenericPicker` and `InlineListDialog` that only ran once on mount
- Root cause: `use_effect` in Dioxus 0.7 only re-runs when *signal* dependencies change — capturing a plain `usize` prop creates no reactive dependency
- Solution: Bridge prop → signal pattern (`use_signal` + `.peek()`/`.set()` + `.read()` in effect)
- `scrollSelectedPickerItem()` and `scrollSelectedInlineDialogItem()` JS functions now fire on every selection change

### Files Changed
- `src/components/picker/generic.rs` — prop-to-signal bridge for scroll effect
- `src/components/inline_dialog/list.rs` — same pattern for inline list dialogs

---

## 2026-02-22: Implement 42 Missing CLI Commands (88 total)

### Progress
- Implemented 42 new CLI commands across 4 batches, bringing the total from 46 to 88
- **Batch 1 — Buffer/File Aliases & Force Variants** (15 commands): `buffer-close`/`bc`/`bclose`, `buffer-close!`, `buffer-close-others!`/`bco!`, `exit`/`xit`, `exit!`/`xit!`, `write-all!`/`wa!`, `write-buffer-close`/`wbc`, `write-buffer-close!`/`wbc!`, `write-quit-all`/`wqa`/`xa`, `write-quit-all!`/`wqa!`/`xa!`, `update`/`u`, `reload-all`/`rla`, `read`/`r`, `move`/`mv`, `move!`/`mv!`
- **Batch 2 — Config & Language** (5 commands): `get-option`/`get`, `set-language`/`lang`, `indent-style`, `config-open-workspace`, `tutor`
- **Batch 3 — Register & Clipboard** (16 commands): `clipboard-yank`, `clipboard-paste-after/before/replace`, `primary-clipboard-yank`, `primary-clipboard-paste-after/before/replace`, `show-clipboard-provider`, `yank-join`, `yank-diagnostic`, `clear-register`, `set-register`, `character-info`, `echo`, `goto`
- **Batch 4 — LSP & Misc** (6 commands): `lsp-stop`, `reset-diff-change`/`diffget`/`diffg`, `tree-sitter-highlight-name`, `tree-sitter-subtree`/`ts-subtree`, `cquit`/`cq`, `cquit!`/`cq!`
- Refactored `buffer_close_others` and `write_all` to accept `force: bool` parameter
- Added 7 new methods to `BufferOps` trait: `write_all_impl`, `write_buffer_close`, `reload_all`, `read_file`, `move_file`, `update_document`
- Added helper methods on `EditorContext`: `get_option`, `set_language`, `indent_style_command`, `yank_join`, `yank_diagnostic_impl`, `show_character_info`, `lsp_stop`

### Files Changed
- `src/operations/cli.rs` — 42 `CommandCompletion` entries + match arms + helper methods (+706 lines)
- `src/operations/buffer.rs` — 7 new trait methods + implementations (+157 lines)
- `src/operations/vcs.rs` — `reset-diff-change` reuses `revert_hunk_at_line()`
- `src/state/mod.rs` — updated `buffer_close_others` → `buffer_close_others_impl(false)`

---

## Template for Future Entries

```markdown
## YYYY-MM-DD: Title

### Progress
- What was accomplished

### Issues Encountered
- Problems faced and solutions

### Decisions Made
- Key decisions and rationale

### Next Steps
- What to do next
```
