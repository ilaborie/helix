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
