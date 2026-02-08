# helix-dioxus Keybinding Comparison

This file documents differences between helix-dioxus keybindings and the standard Helix editor (`helix-term`).
Reference: `helix-term/src/keymap/default.rs`

## Legend

- **Matches**: binding behaves the same as Helix
- **Deviation**: binding exists but does something different
- **Custom**: binding added in helix-dioxus that doesn't exist in Helix
- **Missing**: Helix binding not yet implemented

---

## Normal Mode

### Movement — Matches Helix

| Key | Action | Status |
|-----|--------|--------|
| `h/j/k/l`, arrows | Move cursor | Matches |
| `w/b/e` | Word movement | Matches |
| `W/B/E` | WORD movement | Matches |
| `Home` | Line start | Matches |
| `End` | Line end | Matches |
| `f/F/t/T` + char | Find/till character | Matches |
| `G` | Go to last line | Matches |
| `PageUp/PageDown` | Page up/down | Matches |
| `C-b` | Page up | Matches |
| `C-f` | Page down | Matches |
| `C-u` | Half page up | Matches |
| `C-d` | Half page down | Matches |
| `Alt+.` | Repeat last motion | Matches |
| `%` | Select all | Matches |
| `Alt+;` | Flip selections | Matches |
| `X` | Extend to line bounds | Matches |
| `Alt+x` | Shrink to line bounds | Matches |

### Movement — Deviations

| Key | Helix | helix-dioxus | Notes |
|-----|-------|-------------|-------|
| `0` | (not bound) | `goto_line_start` | Vim-style convenience, Helix uses `gh` |
| `$` | `shell_keep_pipe` | `goto_line_end` | Vim-style convenience, Helix uses `gl` |

### Movement — Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| (none) | All basic movement keys are now covered | |

### Editing — Matches Helix

| Key | Action | Status |
|-----|--------|--------|
| `i/I/a/A` | Enter insert mode | Matches |
| `o/O` | Open line below/above | Matches |
| `c` | Change selection | Matches |
| `d` | Delete selection | Matches |
| `u/U` | Undo/redo | Matches |
| `y` | Yank | Matches |
| `p/P` | Paste after/before | Matches |
| `R` | Replace with yanked | Matches |
| `>/<` | Indent/unindent | Matches |
| `J` | Join lines | Matches |
| `~` | Toggle case | Matches |
| `` ` `` | To lowercase | Matches |
| `` Alt+` `` | To uppercase | Matches |
| `r` + char | Replace character | Matches |

### Editing — Deviations

| Key | Helix | helix-dioxus | Notes |
|-----|-------|-------------|-------|
| `K` | `keep_selections` | `TriggerHover` | Helix hover is `Space k`; `K` filters selections by regex |
| `C-c` | `toggle_comments` | `ToggleLineComment` | Same intent but also available via `Space c` |
| `C-r` | (not bound) | `Redo` | Vim-style convenience; Helix uses `U` |

### Editing — Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| `=` | `format_selections` | Format; helix-dioxus has `FormatDocument` in command panel |
| `s` | `select_regex` | Select by regex pattern |
| `S` | `split_selection` | Split selection on regex |
| `C` | `copy_selection_on_next_line` | Multi-cursor |
| `A-d` | `delete_selection_noyank` | Delete without yanking |
| `A-c` | `change_selection_noyank` | Change without yanking |
| `A-o` | `expand_selection` | Tree-sitter expand |
| `A-i` | `shrink_selection` | Tree-sitter shrink |
| `q/Q` | `replay_macro` / `record_macro` | Macro support |
| `&` | `align_selections` | Align selections |
| `_` | `trim_selections` | Trim whitespace |
| `(/)` | `rotate_selections_backward/forward` | Multi-selection rotation |
| `\|` | `shell_pipe` | Pipe selection through shell |
| `!` | `shell_insert_output` | Insert shell output |
| `C-a/C-x` | `increment/decrement` | Increment/decrement numbers |
| `C-i/Tab` | `jump_forward` | Jump list forward |
| `C-o` | `jump_backward` | Jump list backward |
| `C-s` | `save_selection` | Save selection to jump list |

### Custom Bindings (not in Helix)

| Key | helix-dioxus Action | Notes |
|-----|-------------------|-------|
| `C-h` | `PreviousBuffer` | Buffer cycling shortcut |
| `C-l` | `NextBuffer` | Buffer cycling shortcut |
| `C-Space` | `ShowCodeActions` | Quick access to code actions |
| `C-.` | `ShowCodeActions` | Quick access to code actions |

---

## g-prefix (Goto) — Normal Mode

### Matches Helix

| Key | Action |
|-----|--------|
| `gg` | Go to file start |
| `gd` | Go to definition |
| `ge` | Go to last line |
| `gh` | Go to line start |
| `gi` | Go to implementation |
| `gl` | Go to line end |
| `gn` | Next buffer |
| `gp` | Previous buffer |
| `gr` | Go to references |
| `gs` | Go to first non-whitespace |
| `gy` | Go to type definition |
| `gt` | Go to window top |
| `gc` | Go to window center |
| `gb` | Go to window bottom |
| `ga` | Go to last accessed file |
| `gm` | Go to last modified file |
| `g.` | Go to last modification |

### Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| `gf` | `goto_file` | Open file under cursor |
| `gD` | `goto_declaration` | Go to declaration |
| `g\|` | `goto_column` | Go to column |
| `gk/gj` | `move_line_up/down` | Move line up/down |
| `gw` | `goto_word` | Word jump |

---

## Space Leader — Normal Mode

### Matches Helix

| Key | Action |
|-----|--------|
| `Space /` | Global search |
| `Space ?` | Command palette |
| `Space a` | Code actions |
| `Space b` | Buffer picker |
| `Space c` | Toggle comments |
| `Space C` | Toggle block comments |
| `Space d` | Document diagnostics |
| `Space D` | Workspace diagnostics |
| `Space f` | File picker |
| `Space k` | Hover |
| `Space p` | Paste from clipboard |
| `Space P` | Paste clipboard before |
| `Space r` | Rename symbol |
| `Space R` | Replace with clipboard |
| `Space s` | Document symbols |
| `Space S` | Workspace symbols |
| `Space y` | Yank to clipboard |

### Custom Bindings (not in Helix)

| Key | helix-dioxus Action | Notes |
|-----|-------------------|-------|
| `Space i` | Toggle inlay hints | Desktop UI extension |

### Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| `Space F` | `file_picker_in_current_directory` | File picker in buffer's directory |
| `Space e` | `file_explorer` | File explorer |
| `Space E` | `file_explorer_in_current_buffer_directory` | File explorer in buffer dir |
| `Space j` | `jumplist_picker` | Jump list picker |
| `Space g` | `changed_file_picker` | Changed files (VCS) |
| `Space h` | `select_references_to_symbol_under_cursor` | Select references |
| `Space Y` | `yank_main_selection_to_clipboard` | Yank main selection to clipboard |
| `Space '` | `last_picker` | Resume last picker |
| `Space w` | Window sub-menu | Not supported (single-view design) |
| `Space A-c` | `toggle_line_comments` | Toggle line comments specifically |
| `Space G` | Debug sub-menu | DAP integration |

---

## Bracket Sequences

### Matches Helix

| Key | Action |
|-----|--------|
| `]d` | Next diagnostic |
| `[d` | Previous diagnostic |
| `] Space` | Add newline below |
| `[ Space` | Add newline above |
| `]D / [D` | Last/first diagnostic |
| `]f / [f` | Next/prev function |
| `]t / [t` | Next/prev class |
| `]a / [a` | Next/prev parameter |
| `]c / [c` | Next/prev comment |
| `]p / [p` | Next/prev paragraph |

### Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| `]g / [g` | Next/prev change | VCS change navigation |

---

## Match (m-prefix) — Matches Helix

| Key | Action | Status |
|-----|--------|--------|
| `mm` | Match brackets | Matches |
| `ms` + char | Surround add | Matches |
| `md` + char | Surround delete | Matches |
| `mr` + old + new | Surround replace | Matches |
| `mi` + char | Select inside pair | Matches |
| `ma` + char | Select around pair | Matches |

---

## Select Mode

### Matches Helix

| Key | Action |
|-----|--------|
| `Esc` | Exit select mode |
| `v` | Exit select mode (toggle) |
| `h/j/k/l`, arrows | Extend selection |
| `w/b/e` | Extend word selection |
| `W/B/E` | Extend WORD selection |
| `Home/End` (`0/$`) | Extend to line start/end |
| `x/X` | Select line / extend to line bounds |
| `y` | Yank (exits select) |
| `d` | Delete selection |
| `c` | Change selection |
| `R`, `p` | Replace with yanked |
| `;` | Collapse selection |
| `,` | Keep primary selection |
| `>/<` | Indent/unindent |
| `f/F/t/T` + char | Extend to find/till character |
| `r` + char | Replace character |
| `n/N` | Extend search next/prev |
| `m` prefix | Match/surround (same as normal) |
| `"` prefix | Register selection |

### Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| `g` | Goto sub-menu (extend variants) | `gg` extends to file start, etc. |

---

## Insert Mode

### Matches Helix

| Key | Action |
|-----|--------|
| `Esc` | Exit insert mode |
| `Backspace` | Delete char backward |
| `Delete` | Delete char forward |
| `Enter` | Insert newline |
| `Tab` | Insert tab |
| `C-w` | Delete word backward |
| `C-u` | Kill to line start |
| `C-k` | Kill to line end |
| `C-h` | Delete char backward |
| `C-d` | Delete char forward |
| `C-j` | Insert newline |
| `A-d` | Delete word forward |
| arrows, Home, End | Cursor movement |
| PageUp/PageDown | Page movement |

### Custom Bindings (not in Helix)

| Key | helix-dioxus Action | Notes |
|-----|-------------------|-------|
| `C-c` | `ToggleLineComment` | Comment toggle in insert mode |
| `C-Space` | `TriggerCompletion` | Explicit completion trigger |
| `C-.` | `ShowCodeActions` | Quick fix in insert mode |

### Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| `C-x` | `completion` | Helix completion trigger |
| `C-r` | `insert_register` | Insert from register |
| `C-s` | `commit_undo_checkpoint` | Manual undo checkpoint |
| `S-Tab` | `insert_tab` | Literal tab insert |
| `A-Backspace` | `delete_word_backward` | Alt backspace |

---

## View Mode (z/Z prefix)

### Matches Helix

| Key | Action |
|-----|--------|
| `zz` / `zc` | Align view center |
| `zt` | Align view top |
| `zb` | Align view bottom |
| `zk/zj` | Scroll up/down |
| `z C-b / z PageUp` | Page up |
| `z C-f / z PageDown` | Page down |
| `z C-u` | Half page up |
| `z C-d` | Half page down |
| `z/`, `z?` | Search forward/backward |
| `zn/zN` | Search next/prev |
| `Z` prefix | Sticky view mode (same keys, stays until Esc) |

### Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| `zm` | Align view middle | Cursor at middle column |

---

## Window Mode (C-w / Space w) — Not Supported

Window/split management is not supported. helix-dioxus uses a single-view design.

---

## Command Mode (`:` commands)

### Implemented

| Command | Aliases | Notes |
|---------|---------|-------|
| `:open` | `:o` | Open file / file picker |
| `:quit` | `:q` | Quit |
| `:quit!` | `:q!` | Force quit |
| `:write` | `:w` | Save |
| `:write!` | `:w!` | Force save |
| `:wq` | `:x` | Save and quit |
| `:new` | `:n` | New scratch buffer |
| `:buffer` | `:b` | Buffer picker |
| `:bnext` | `:bn` | Next buffer |
| `:bprev` | `:bp` | Previous buffer |
| `:bdelete` | `:bd` | Close buffer |
| `:reload` | `:rl` | Reload from disk |
| `:write-all` | `:wa` | Save all |
| `:quit-all` | `:qa` | Quit all |
| `:buffer-close-all` | `:bca` | Close all buffers |
| `:buffer-close-others` | `:bco` | Close other buffers |
| `:cd` | | Change directory |
| `:pwd` | | Print working directory |
| `:registers` | `:reg` | Register picker |
| `:commands` | `:cmd` | Command panel |
| `:earlier` | | Undo history |
| `:later` | | Redo history |

### Missing Notable Commands

| Command | Helix Action |
|---------|-------------|
| `:theme` | Change color theme |
| `:config-reload` | Reload configuration |
| `:lsp-restart` | Restart LSP (available via LSP dialog) |
| `:set` | Change editor settings |
| `:sort` | Sort selection |
| `:reflow` | Reflow text |
| `:pipe` | Pipe through shell |
| `:run-shell-command` | Run shell command |
| `:encoding` | Set file encoding |
| `:line-ending` | Set line ending |

---

## Register Prefix (`"`)

| Feature | Status |
|---------|--------|
| `"` + register + operation | Matches Helix |
| Named registers (`a`-`z`) | Matches |
| Clipboard (`+`) | Matches |
| Black hole (`_`) | Matches |
| Default (`"`) | Matches |
| Search (`/`) | Matches |

---

## Summary

### Design Decisions

- **Window/Splits**: Not supported — helix-dioxus uses a single-view design. `C-w` prefix and `Space w` sub-menu will not be implemented.

### Feature Categories Not Yet Implemented

1. **Multi-cursor** — `C`, `A-C`, selections rotation `(/)`, `s`/`S` regex selection
2. **Jump list** — `C-i`/`C-o`, `Space j`, `Space '`
3. **Tree-sitter expand/shrink** — `A-o`/`A-i`, `gw` word jump
4. **Macros** — `q/Q` record/replay
5. **Shell integration** — `|`, `!`, `$` pipe/insert/keep
6. **VCS integration** — `]g/[g` change navigation, `Space g` changed files
7. **DAP/Debug** — `Space G` sub-menu
