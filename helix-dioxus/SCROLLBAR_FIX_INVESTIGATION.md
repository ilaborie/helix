# Scrollbar Track Click & Thumb Drag — Investigation

## Problem

The scrollbar in helix-dioxus renders correctly (thumb position, diagnostic/search markers, tooltips) but **track clicking and thumb dragging don't work**. Both require the scrollbar track's pixel height to convert mouse Y coordinates into a document-line ratio.

## Root Cause

The mouse handlers need the track's pixel height to compute `ratio = click_y / track_height`. All attempted approaches to obtain this height in the Dioxus desktop WebView have failed.

## What Has Been Tested

### Attempt 1: `getBoundingClientRect()` via `document::eval` (original code)

```rust
let height = document::eval(
    r#"document.querySelector('.editor-view')?.getBoundingClientRect().height || 0"#,
);
spawn(async move {
    if let Ok(val) = height.await {
        let scrollbar_h: f64 = val.as_f64().unwrap_or(0.0);
        // scrollbar_h is always 0
    }
});
```

**Result**: `getBoundingClientRect().height` returns 0 in the Dioxus desktop WebView context. The querySelector finds the element, but the rect dimensions are all zero.

### Attempt 2: `onmounted` + `get_client_rect()` (original code)

```rust
let onmounted = move |evt: MountedEvent| async move {
    if let Ok(rect) = evt.get_client_rect().await {
        scrollbar_height.set(rect.height()); // Always 0
    }
};
```

**Result**: Returns 0 because the element hasn't been laid out at mount time. Dioxus fires `onmounted` before the browser completes layout.

### Attempt 3: ResizeObserver + cached JS variable (reverted)

Added to `script.js`:
```javascript
var _scrollbarTrackHeight = 0;

function initScrollbarTrackObserver() {
    var track = document.querySelector('.scrollbar-track');
    if (track) {
        var ro = new ResizeObserver(function(entries) {
            _scrollbarTrackHeight = entries[i].contentRect.height;
        });
        ro.observe(track);
        _scrollbarTrackHeight = track.getBoundingClientRect().height;
    } else {
        // MutationObserver fallback to wait for element
        var mo = new MutationObserver(function(_mutations, observer) {
            var el = document.querySelector('.scrollbar-track');
            if (el) {
                observer.disconnect();
                initScrollbarTrackObserver();
            }
        });
        mo.observe(document.body, { childList: true, subtree: true });
    }
}

function getScrollbarTrackHeight() {
    return _scrollbarTrackHeight;
}

initScrollbarTrackObserver();
```

Rust side:
```rust
let height = document::eval("return getScrollbarTrackHeight()");
spawn(async move {
    if let Ok(val) = height.await {
        let track_height: f64 = val.as_f64().unwrap_or(0.0);
        // Use track_height for ratio calculation
    }
});
```

Changes made to `scrollbar.rs`:
- Moved `onmousedown` from `.editor-scrollbar` to `.scrollbar-track` (so `element_coordinates().y` is relative to the track, no 8px offset needed)
- Used `ScrollToLine` instead of `GoToLine` (scrollbar behavior: scroll view without moving cursor)
- Removed dead `scrollbar_height` signal and `onmounted` handler

**Result**: Still not working. Possible reasons:
1. The `ResizeObserver` may not fire in the Dioxus desktop WebView (Wry/WebKitGTK/WebView2), or `contentRect.height` may also be 0
2. The `document::eval("return ...")` async roundtrip may have timing issues — by the time the value returns, the mouse event context is stale
3. The initial seed `getBoundingClientRect().height` in JS may also be 0 (same underlying issue as attempt 1)
4. The `.scrollbar-track` element uses `position: absolute; top: 8px; bottom: 8px;` — its rendered size depends on the parent, which may report 0 in certain WebView contexts

## Key Findings

### `document::eval` requires `return` for values

Dioxus `document::eval()` wraps JavaScript in an async function. You **must** use `return` to get a value back:
- `document::eval("return getMyValue()")` — returns the value when awaited
- `document::eval("getMyValue()")` — calls function but result is discarded

### CSS Layout

```css
.editor-scrollbar {
    width: 14px;
    height: 100%;
    position: relative;
}
.scrollbar-track {
    position: absolute;
    top: 8px;
    bottom: 8px;
    left: 2px;
    right: 2px;
}
```

The track height is derived from the parent's height minus 16px. The parent uses `height: 100%` which depends on flex layout from ancestors.

## Unexplored Approaches

### 1. `use_effect` with delayed eval

Read the height after a `requestAnimationFrame` delay, when layout is guaranteed complete:
```rust
use_effect(move || {
    spawn(async {
        let val = document::eval("
            return new Promise(resolve => {
                requestAnimationFrame(() => {
                    var t = document.querySelector('.scrollbar-track');
                    resolve(t ? t.getBoundingClientRect().height : 0);
                });
            });
        ").await;
        // store in signal
    });
});
```

### 2. Avoid pixel height entirely — use percentage-based approach

Since the thumb position and markers already use percentages (`top: X%`), the track click could also work in percentages by computing `click_y / element_height_from_offsetHeight`:

```javascript
function handleScrollbarClick(clickY) {
    var track = document.querySelector('.scrollbar-track');
    if (track) return clickY / track.offsetHeight;
    return -1;
}
```

Note: `offsetHeight` may work where `getBoundingClientRect().height` doesn't, as it's a layout property rather than a computed geometry query.

### 3. Pass height from Rust via `use_effect` + `eval` with `dioxus.send()`

Use the bidirectional channel:
```rust
let mut eval = document::eval(r#"
    let track = document.querySelector('.scrollbar-track');
    let ro = new ResizeObserver(entries => {
        dioxus.send(entries[0].contentRect.height);
    });
    ro.observe(track);
"#);
// recv in a loop to keep height updated
```

### 4. Compute height from known constants

If `viewport_lines` and font metrics are known, the track height can be computed without querying the DOM at all:
```
window_height = viewport_lines * LINE_HEIGHT + CONTENT_PADDING * 2
track_height = window_height - 16  (8px top + 8px bottom padding)
```

This avoids the DOM query entirely but requires accurate `viewport_lines` (currently hardcoded to 40).

### 5. Use `window.innerHeight` as proxy

`window.innerHeight` might be available and could approximate the scrollbar height:
```javascript
return window.innerHeight - STATUS_BAR_HEIGHT - BUFFER_BAR_HEIGHT - 16;
```

## Recommendations

1. **Try approach #3 first** (bidirectional `dioxus.send()`) — it keeps the ResizeObserver idea but avoids the sync read problem
2. **Try approach #4** if DOM queries are fundamentally broken in Wry — compute from known constants
3. **Add debug logging** to JavaScript (`console.log`) and check Wry's dev tools to verify whether `getBoundingClientRect()` and `offsetHeight` truly return 0, or if the issue is in the Rust-JS bridge
