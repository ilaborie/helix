# Helix Dioxus Integration Plan

## Approach: Study helix-gpui, Adapt for Dioxus

Based on research, [helix-gpui](https://github.com/polachok/helix-gpui) provides the best reference implementation for a GUI frontend. We'll study its patterns and adapt them for Dioxus.

---

## helix-gpui Architecture

### Source Structure
```
src/
├── main.rs          (11KB)  - Entry point, GPUI app setup
├── application.rs   (36KB)  - Core app logic, event handling
├── document.rs      (34KB)  - Document rendering, syntax highlighting
├── workspace.rs     (15KB)  - Workspace/editor state management
├── statusline.rs    (17KB)  - Status bar component
├── picker.rs        (2KB)   - File/symbol picker
├── prompt.rs        (2KB)   - Command prompt
├── overlay.rs       (2KB)   - Overlay component
├── info_box.rs      (3KB)   - Info box/tooltip
├── notification.rs  (9KB)   - Notification system
└── utils.rs         (7KB)   - Utility functions
```

### Helix Crates Used
- `helix-core` - Text editing primitives
- `helix-view` - Editor/Document/View abstractions
- `helix-term` - Commands (reused, not just for terminal)
- `helix-lsp` - Language server support
- `helix-loader` - Grammar/config loading
- `helix-event` - Event system

**Key insight**: helix-gpui uses a **fork of helix** with modifications. We may need similar changes.

---

## Implementation Plan

### Phase 1: Project Setup
1. Create `helix-dioxus` crate in this repo or separate repo
2. Add dependencies:
   ```toml
   [dependencies]
   dioxus = { version = "0.6", features = ["desktop"] }
   helix-core = { path = "../helix-core" }
   helix-view = { path = "../helix-view" }
   helix-loader = { path = "../helix-loader" }
   helix-lsp = { path = "../helix-lsp" }
   helix-event = { path = "../helix-event" }
   tokio = { version = "1", features = ["full"] }
   ```
3. Basic Dioxus desktop app skeleton

### Phase 2: Editor State Integration
Study `helix-gpui/src/workspace.rs` and `application.rs`:
1. Initialize `helix_loader` for runtime resources
2. Create `Editor` from `helix-view`
3. Manage document lifecycle
4. Handle editor configuration

**Key types to wrap:**
- `Editor` - Main editor state
- `Document` - Open file
- `View` - View into document
- `Theme` - Syntax highlighting theme

### Phase 3: Document Rendering
Study `helix-gpui/src/document.rs`:
1. Create Dioxus component for document view
2. Render text lines with syntax highlighting
3. Render cursor(s) and selections
4. Handle scrolling and viewport

**Rendering approach:**
```rust
#[component]
fn DocumentView(editor: Signal<Editor>, view_id: ViewId) -> Element {
    // Get document and view
    // Render visible lines
    // Apply syntax highlighting spans
    // Draw cursors and selections
}
```

### Phase 4: Input Handling
Study how helix-gpui translates GPUI events to helix:
1. Map Dioxus keyboard events → `helix_view::input::KeyEvent`
2. Map Dioxus mouse events → helix mouse handling
3. Execute helix commands via `helix_term::commands`
4. Handle mode changes (Normal/Insert/Select)

### Phase 5: UI Components
Implement Dioxus equivalents of:
1. **Status line** - Mode, file info, position
2. **Picker** - File picker, symbol picker, buffer picker
3. **Prompt** - Command input (`:` commands)
4. **Completion** - Autocomplete popup
5. **Diagnostics** - Error/warning inline display

### Phase 6: LSP Integration
1. Start language servers via `helix-lsp`
2. Handle LSP events in Dioxus async context
3. Display completions, hover info, diagnostics

---

## Technical Challenges & Solutions

### 1. Async Runtime Bridge
**Challenge**: Helix uses tokio, Dioxus has its own async runtime
**Solution**: Use `tokio::runtime::Runtime` for helix operations, bridge with Dioxus signals

### 2. Keyboard Event Translation
**Challenge**: Dioxus keyboard events differ from crossterm
**Solution**: Create translation layer:
```rust
fn translate_key(event: &KeyboardEvent) -> Option<helix_view::input::KeyEvent> {
    // Map key codes and modifiers
}
```

### 3. Syntax Highlighting
**Challenge**: helix uses tree-sitter highlights, need to convert to Dioxus styles
**Solution**: Convert `helix_core::syntax::Highlight` spans to CSS classes or inline styles

### 4. Multiple Cursors
**Challenge**: Helix supports multiple selections
**Solution**: Iterate over all ranges in `Selection`, render each cursor

### 5. Performance with Large Files
**Challenge**: Rendering large documents
**Solution**: Virtual scrolling - only render visible lines

---

## File Structure for helix-dioxus

```
helix-dioxus/
├── Cargo.toml
├── PLAN.md               # Initial implementation plan (snapshot)
├── DEV_LOGBOOK.md        # Development journal: progress, changes, issues
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Main App component
│   ├── editor_view.rs    # Document rendering
│   ├── statusline.rs     # Status bar
│   ├── picker.rs         # File/buffer picker
│   ├── prompt.rs         # Command prompt
│   ├── completion.rs     # Autocomplete
│   ├── input.rs          # Keyboard/mouse translation
│   └── state.rs          # Dioxus state management
└── assets/
    └── styles.css        # Base styles
```

### Documentation Files

**PLAN.md** - Initial implementation plan (static snapshot):
- Project goals and scope
- Architecture overview
- Implementation phases
- Success criteria

**DEV_LOGBOOK.md** - Development journal (updated continuously):
- Date-stamped entries
- Progress updates
- Plan changes with rationale
- Issues encountered and solutions
- Decisions made and alternatives considered
- Links to relevant commits/PRs

---

## First Steps (Minimal Viable Editor)

1. **Display a file** - Load file into helix Document, render text
2. **Show cursor** - Display primary cursor position
3. **Basic navigation** - h/j/k/l movement in Normal mode
4. **Mode indicator** - Show current mode in status line
5. **Insert mode** - Basic text insertion

---

## References

- [helix-gpui source](https://github.com/polachok/helix-gpui)
- [Helix GUI Issue #39](https://github.com/helix-editor/helix/issues/39)
- [Dioxus 0.6 documentation](https://dioxuslabs.com/)
- [helix-view Editor type](helix-view/src/editor.rs)
- [helix-term commands](helix-term/src/commands.rs)

---

## Success Criteria

1. **Phase 1 Complete**: Application launches, shows empty window
2. **Phase 2 Complete**: Can open a file and see its contents
3. **Phase 3 Complete**: Syntax highlighting works, cursor visible
4. **Phase 4 Complete**: Basic vim motions (h/j/k/l, i, Esc) work
5. **Phase 5 Complete**: Status line shows mode and file info
6. **MVP Complete**: Can edit and save a file with basic vim commands
