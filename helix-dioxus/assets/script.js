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
