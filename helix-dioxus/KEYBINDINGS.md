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
| `A-d` | Delete without yanking | Matches |
| `A-c` | Change without yanking | Matches |
| `C-a` | Increment number | Matches |
| `C-x` | Decrement number | Matches |
| `_` | Trim selections | Matches |
| `=` | Format selections (LSP) | Matches |
| `&` | Align selections | Matches |
| `s` | Select regex (prompt) | Matches |
| `S` | Split selection on regex | Matches |
| `C` | Copy selection to next line | Matches |
| `A-C` | Copy selection to prev line | Matches |
| `A-s` | Split selection on newlines | Matches |
| `)` | Rotate selections forward | Matches |
| `(` | Rotate selections backward | Matches |
| `A-o` | Expand selection (tree-sitter) | Matches |
| `A-i` | Shrink selection (tree-sitter) | Matches |
| `;` | Collapse selection | Matches |
| `,` | Keep primary selection | Matches |
| `*` | Search word under cursor | Matches |
| `n/N` | Search next/prev | Matches |
| `C-i` | Jump forward | Matches |
| `C-o` | Jump backward | Matches |
| `C-s` | Save selection to jump list | Matches |
| `\|` | Pipe selection through shell | Matches |
| `!` | Insert shell output | Matches |
| `A-\|` | Pipe to shell (discard output) | Matches |
| `A-!` | Append shell output | Matches |
| `q` | Replay macro | Matches |
| `Q` | Record/stop macro | Matches |

### Editing — Deviations

| Key | Helix | helix-dioxus | Notes |
|-----|-------|-------------|-------|
| `K` | `keep_selections` | `TriggerHover` | Helix hover is `Space k`; `K` filters selections by regex |
| `C-c` | `toggle_comments` | `ToggleLineComment` | Same intent but also available via `Space c` |
| `C-r` | (not bound) | `Redo` | Vim-style convenience; Helix uses `U` |

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
| `gD` | Go to declaration |
| `ge` | Go to last line |
| `gf` | Go to file under cursor |
| `gh` | Go to line start |
| `gi` | Go to implementation |
| `gj` | Move down (visual line) |
| `gk` | Move up (visual line) |
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
| `g\|` | Go to column 1 |
| `gw` | Word jump (EasyMotion-style) |

---

## Space Leader — Normal Mode

### Matches Helix

| Key | Action |
|-----|--------|
| `Space /` | Global search |
| `Space ?` | Command palette |
| `Space '` | Resume last picker |
| `Space a` | Code actions |
| `Space b` | Buffer picker |
| `Space c` | Toggle comments |
| `Space C` | Toggle block comments |
| `Space d` | Document diagnostics |
| `Space D` | Workspace diagnostics |
| `Space e` | File explorer |
| `Space E` | File explorer in buffer's directory |
| `Space f` | File picker |
| `Space F` | File picker in buffer's directory |
| `Space g` | Changed file picker (VCS) |
| `Space h` | Select references to symbol |
| `Space j` | Jump list picker |
| `Space k` | Hover |
| `Space p` | Paste from clipboard |
| `Space P` | Paste clipboard before |
| `Space r` | Rename symbol |
| `Space R` | Replace with clipboard |
| `Space s` | Document symbols |
| `Space S` | Workspace symbols |
| `Space y` | Yank to clipboard |
| `Space Y` | Yank main selection to clipboard |

### Custom Bindings (not in Helix)

| Key | helix-dioxus Action | Notes |
|-----|-------------------|-------|
| `Space i` | Toggle inlay hints | Desktop UI extension |

### Missing

| Key | Helix Action | Notes |
|-----|-------------|-------|
| `Space w` | Window sub-menu | Not supported (single-view design) |
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
| `]g / [g` | Next/prev VCS change |

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
| `s/S` | Regex select/split |
| `C` | Copy selection to next line |
| `A-C` | Copy selection to prev line |
| `A-s` | Split selection on newlines |
| `(/)` | Rotate selections |
| `A-;` | Flip selections |
| `A-o/A-i` | Expand/shrink selection |
| `m` prefix | Match/surround (same as normal) |
| `"` prefix | Register selection |
| `C-i/C-o` | Jump forward/backward |
| `C-s` | Save selection to jump list |
| `g` prefix | Goto sub-menu (extend variants) |

### Select Mode g-prefix

| Key | Action |
|-----|--------|
| `gg` | Extend to file start |
| `ge` | Extend to last line |
| `gh` | Extend to line start |
| `gl` | Extend to line end |
| `gs` | Extend to first non-whitespace |
| `g\|` | Extend to column |
| `gd/gD/gy/gi/gr/gf` | LSP goto (jump, no extend) |
| `gn/gp` | Next/prev buffer |
| `gt/gc/gb` | Window top/center/bottom |
| `ga/gm/g.` | Last accessed/modified file |
| `gw` | Extend to word (EasyMotion-style) |

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
| `C-r` | Insert from register |
| `C-s` | Commit undo checkpoint |
| `S-Tab` | Unindent |
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
| `C-x` | `completion` | Helix completion trigger (use `C-Space` instead) |
| `A-Backspace` | `delete_word_backward` | Alt backspace (use `C-w` instead) |

---

## View Mode (z/Z prefix)

### Matches Helix

| Key | Action |
|-----|--------|
| `zz` / `zc` / `zm` | Align view center |
| `zt` | Align view top |
| `zb` | Align view bottom |
| `zk/zj` | Scroll up/down |
| `z C-b / z PageUp` | Page up |
| `z C-f / z PageDown` | Page down |
| `z C-u` | Half page up |
| `z C-d` | Half page down |
| `z Space` | Half page down |
| `z Backspace` | Half page up |
| `z/`, `z?` | Search forward/backward |
| `zn/zN` | Search next/prev |
| `Z` prefix | Sticky view mode (same keys, stays until Esc) |

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
| `:write` | `:w` | Save (shows Save As dialog for scratch buffers) |
| `:write!` | `:w!` | Force save |
| `:wq` | `:x` | Save and quit |
| `:wq!` | `:x!` | Force save and quit |
| `:new` | `:n` | New scratch buffer |
| `:buffer` | `:b` | Buffer picker |
| `:bnext` | `:bn` | Next buffer |
| `:bprev` | `:bp` | Previous buffer |
| `:bdelete` | `:bd` | Close buffer |
| `:bdelete!` | `:bd!` | Force close buffer |
| `:reload` | `:rl` | Reload from disk |
| `:write-all` | `:wa` | Save all |
| `:quit-all` | `:qa` | Quit all |
| `:quit-all!` | `:qa!` | Force quit all |
| `:buffer-close-all` | `:bca` | Close all buffers |
| `:buffer-close-all!` | `:bca!` | Force close all |
| `:buffer-close-others` | `:bco` | Close other buffers |
| `:cd` | `:change-current-directory` | Change directory |
| `:pwd` | | Print working directory |
| `:registers` | `:reg` | Register picker |
| `:commands` | `:cmd` | Command panel |
| `:earlier` | | Undo history (`:earlier N`) |
| `:later` | | Redo history (`:later N`) |
| `:pipe` | `:sh` | Pipe selection through shell (replace) |
| `:insert-output` | | Insert shell command output |
| `:append-output` | | Append shell command output |
| `:pipe-to` | | Pipe to shell (discard output) |
| `:run-shell-command` | `:run` | Run shell command |
| `:theme` | | Change theme (no args = picker with live preview) |
| `:sort` | | Sort multi-cursor selections |
| `:reflow` | | Reflow text (`:reflow [width]`) |
| `:config-open` | | Open config file |
| `:log-open` | | Open log file |
| `:encoding` | | Show/set file encoding |
| `:set-line-ending` | `:line-ending` | Show/set line ending (lf/crlf) |
| `:tree-sitter-scopes` | | Show tree-sitter scopes at cursor |
| `:jumplist-clear` | | Clear jump list |

| `:config-reload` | | Reload config (editor, languages, theme) |
| `:set` | | Set config option (`:set <key> <value>`) |
| `:toggle` | | Toggle config option (`:toggle <key> [val1 val2 ...]`) |
| `:format` | `:fmt` | Format document via LSP |
| `:lsp-restart` | | Restart LSP server(s) for current document |

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

## Macro Support

| Feature | Status |
|---------|--------|
| `Q` | Record/stop macro (default register `@`) |
| `q` | Replay macro from register |
| `"aQ` | Record to named register `a` |
| `"aq` | Replay from named register `a` |
| Statusline `REC [@]` indicator | Matches |

---

## Summary

### Coverage Statistics

| Category | Implemented | Missing | Coverage |
|----------|-------------|---------|----------|
| Normal Mode | 65+ bindings | 0 | 100% |
| Goto (g-prefix) | 23 bindings | 0 | 100% |
| Space Leader | 26 bindings | 1 (`G`) | 96% |
| Bracket Sequences | 18 bindings | 0 | 100% |
| View Mode (z/Z) | 13 bindings | 0 | 100% |
| Match (m-prefix) | 6 bindings | 0 | 100% |
| Select Mode | 35+ bindings | 0 | 100% |
| Insert Mode | 18 bindings | 0 | 100% |
| Macros | 4 bindings | 0 | 100% |
| Commands | 45 commands | 0 | 100% |
| **Overall** | **~98%** | | |

### Design Decisions

- **Window/Splits**: Not supported — helix-dioxus uses a single-view design. `C-w` prefix and `Space w` sub-menu will not be implemented.

### Remaining Feature Categories

1. **DAP/Debug** — `Space G` sub-menu
