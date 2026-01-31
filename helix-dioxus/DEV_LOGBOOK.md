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

## Planned Enhancements

### Buffer Bar
- [ ] File-type specific icons (use lucide icons based on extension)
- [ ] Option to hide buffer bar (add setting)
- [ ] Context menu on right-click (close, close others, close all)

### Picker
- [ ] Mouse click to select items
- [ ] Scrollbar for long lists

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
