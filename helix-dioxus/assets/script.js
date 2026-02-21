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

// Position all .inline-dialog elements relative to the editor cursor.
// Called from use_effect after rendering dialogs with visibility:hidden.
function positionInlineDialogs() {
    var cursor = document.getElementById('editor-cursor');
    if (!cursor) return;
    var cursorRect = cursor.getBoundingClientRect();
    var viewportHeight = window.innerHeight;
    var dialogs = document.querySelectorAll('.inline-dialog');
    dialogs.forEach(function(dialog) {
        var preferAbove = dialog.dataset.position === 'above';
        var goAbove = preferAbove && cursorRect.top > 100;
        if (goAbove) {
            dialog.style.top = cursorRect.top + 'px';
            dialog.style.transform = 'translateY(-100%)';
        } else {
            dialog.style.top = cursorRect.bottom + 'px';
            dialog.style.transform = '';
        }
        dialog.style.left = Math.min(cursorRect.left, viewportHeight > 0 ? window.innerWidth - 500 : 600) + 'px';
        dialog.style.visibility = 'visible';
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

// Scroll the selected inline-dialog item into view within its scrollable container
function scrollSelectedInlineDialogItem() {
    requestAnimationFrame(() => {
        const selected = document.querySelector('.inline-dialog-item-selected');
        if (selected) {
            selected.scrollIntoView({ block: 'nearest' });
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
