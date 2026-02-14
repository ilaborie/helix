//! Text manipulation operations: sort selections, reflow text.

use helix_core::{Tendril, Transaction};

use crate::state::{EditorContext, NotificationSeverity};

/// Extension trait for text manipulation operations.
pub trait TextManipulationOps {
    fn sort_selections(&mut self);
    fn reflow_selections(&mut self, text_width: Option<usize>);
}

impl TextManipulationOps for EditorContext {
    /// Sort multi-cursor selections alphabetically.
    ///
    /// Requires at least 2 selections; with a single selection shows a warning.
    fn sort_selections(&mut self) {
        let (view, doc) = helix_view::current!(self.editor);
        let view_id = view.id;
        let text = doc.text().slice(..);
        let selection = doc.selection(view_id);

        if selection.len() == 1 {
            self.show_notification(
                "Sorting requires multiple selections".to_string(),
                NotificationSeverity::Warning,
            );
            return;
        }

        let mut fragments: Vec<Tendril> = selection
            .slices(text)
            .map(|fragment| fragment.chunks().collect())
            .collect();

        fragments.sort();

        let transaction = Transaction::change(
            doc.text(),
            selection
                .into_iter()
                .zip(fragments)
                .map(|(range, fragment)| (range.from(), range.to(), Some(fragment))),
        );

        let (view, doc) = helix_view::current!(self.editor);
        doc.apply(&transaction, view.id);
        doc.append_changes_to_history(view);
    }

    /// Reflow (rewrap) selected text to the given width, or the document's text-width.
    fn reflow_selections(&mut self, text_width: Option<usize>) {
        let (view, doc) = helix_view::current!(self.editor);
        let view_id = view.id;
        let width = text_width.unwrap_or_else(|| doc.text_width());
        let rope = doc.text();
        let selection = doc.selection(view_id);

        let transaction = Transaction::change_by_selection(rope, selection, |range| {
            let fragment = range.fragment(rope.slice(..));
            let reflowed = helix_core::wrap::reflow_hard_wrap(&fragment, width);
            (range.from(), range.to(), Some(reflowed))
        });

        let (view, doc) = helix_view::current!(self.editor);
        doc.apply(&transaction, view.id);
        doc.append_changes_to_history(view);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::{assert_text, doc_view, init, test_context};

    use super::*;

    #[test]
    fn sort_selections_alphabetical() {
        // Three selections: "cherry", "apple", "banana" → sorted to "apple", "banana", "cherry"
        let mut ctx = test_context("#[cherry|]# #(apple|)# #(banana|)#\n");
        let (_, _) = doc_view(&ctx);
        ctx.sort_selections();
        assert_text(&ctx, "apple banana cherry\n");
    }

    #[test]
    fn sort_single_selection_is_noop() {
        // Hold the runtime guard since show_notification calls tokio::spawn
        let _guard = init();
        let mut ctx = test_context("#[hello|]# world\n");
        let (_, _) = doc_view(&ctx);
        ctx.sort_selections();
        // Text unchanged — only a notification shown
        assert_text(&ctx, "hello world\n");
    }

    #[test]
    fn reflow_wraps_long_line() {
        // Select a long line and reflow to width 20
        let mut ctx = test_context("#[the quick brown fox jumps over the lazy dog|]#\n");
        let (_, _) = doc_view(&ctx);
        ctx.reflow_selections(Some(20));
        let (_, doc) = helix_view::current_ref!(ctx.editor);
        let text: String = doc.text().slice(..).into();
        // Each line should be <= 20 chars
        for line in text.lines() {
            assert!(
                line.len() <= 20,
                "line exceeds width 20: {:?} (len={})",
                line,
                line.len()
            );
        }
        // Content should be preserved (same words)
        assert!(text.contains("the"));
        assert!(text.contains("quick"));
        assert!(text.contains("lazy"));
        assert!(text.contains("dog"));
    }
}
