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
