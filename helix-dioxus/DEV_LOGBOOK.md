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
