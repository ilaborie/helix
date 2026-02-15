// helix-dioxus JavaScript functions

// Focus the app container on load
function focusAppContainer() {
    requestAnimationFrame(() => {
        const container = document.querySelector('.app-container');
        if (container) {
            container.focus();
        }
    });
}

// Scroll the cursor element into view
function scrollCursorIntoView() {
    requestAnimationFrame(() => {
        const cursor = document.getElementById('editor-cursor');
        if (cursor) {
            cursor.scrollIntoView({ block: 'nearest', inline: 'nearest' });
        }
    });
}

// Ensure app container stays focused — re-focus on any document-level keydown
// This handles cases where WebView loses focus after re-renders
document.addEventListener('keydown', function(e) {
    const container = document.querySelector('.app-container');
    if (container && document.activeElement !== container) {
        container.focus();
    }
});

// --- Scrollbar track height (ResizeObserver) ---

// Cached height, updated by ResizeObserver whenever layout changes
var _scrollbarTrackHeight = 0;

// Start observing .scrollbar-track for size changes.
// Uses a MutationObserver fallback if the element doesn't exist yet.
function initScrollbarTrackObserver() {
    var track = document.querySelector('.scrollbar-track');
    if (track) {
        var ro = new ResizeObserver(function(entries) {
            for (var i = 0; i < entries.length; i++) {
                _scrollbarTrackHeight = entries[i].contentRect.height;
            }
        });
        ro.observe(track);
        // Seed initial value
        _scrollbarTrackHeight = track.getBoundingClientRect().height;
    } else {
        // Element not in DOM yet — watch for it with MutationObserver
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

// Return the cached track height (synchronous)
function getScrollbarTrackHeight() {
    return _scrollbarTrackHeight;
}

// Auto-init on load
initScrollbarTrackObserver();
