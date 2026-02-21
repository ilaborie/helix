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

// Scroll the selected picker item into view within its scrollable container
function scrollSelectedPickerItem() {
    requestAnimationFrame(function() {
        var selected = document.querySelector('.picker-item-selected');
        if (selected) {
            selected.scrollIntoView({ block: 'nearest' });
        }
    });
}

// Position the git diff hover popup next to the hovered gutter diff marker.
// Called from use_effect when git_diff_hover_visible changes.
function positionGitDiffPopup(line) {
    requestAnimationFrame(function() {
        var popup = document.getElementById('git-diff-popup');
        if (!popup) return;

        // Find the gutter element with the matching data-diff-line attribute
        var marker = document.querySelector('[data-diff-line="' + line + '"]');
        if (!marker) {
            // Fallback: position near cursor
            var cursor = document.getElementById('editor-cursor');
            if (cursor) marker = cursor;
        }
        if (!marker) return;

        var rect = marker.getBoundingClientRect();
        var popupRect = popup.getBoundingClientRect();
        var viewportWidth = window.innerWidth;
        var viewportHeight = window.innerHeight;

        // Position to the right of the gutter marker
        var left = rect.right + 8;
        var top = rect.top;

        // Adjust if popup would go off the right edge
        if (left + popupRect.width > viewportWidth - 16) {
            left = rect.left - popupRect.width - 8;
        }

        // Adjust if popup would go off the bottom edge
        if (top + popupRect.height > viewportHeight - 16) {
            top = viewportHeight - popupRect.height - 16;
        }

        // Don't go above the viewport
        if (top < 8) top = 8;

        popup.style.left = left + 'px';
        popup.style.top = top + 'px';
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
