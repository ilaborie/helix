//! Diff computation for code action preview.
//!
//! Pure functions to compute line-level diffs from LSP `WorkspaceEdit`s.

use helix_lsp::lsp;
use helix_lsp::OffsetEncoding;

use super::types::{CodeActionPreview, DiffChangeKind, DiffHunk, DiffLine, FileDiff};

/// Apply LSP `TextEdit`s to a string to produce new text.
///
/// Edits are sorted by position (reverse order) and applied back-to-front
/// to avoid offset invalidation.
#[must_use]
pub fn apply_text_edits(original: &str, edits: &[lsp::TextEdit], encoding: OffsetEncoding) -> String {
    if edits.is_empty() {
        return original.to_string();
    }

    let rope = helix_core::Rope::from_str(original);

    // Sort edits by position, reversed — apply from end to start.
    let mut sorted_edits: Vec<_> = edits.to_vec();
    sorted_edits.sort_by(|lhs, rhs| {
        rhs.range
            .start
            .cmp(&lhs.range.start)
            .then_with(|| rhs.range.end.cmp(&lhs.range.end))
    });

    let mut result = original.to_string();

    for edit in &sorted_edits {
        let start = lsp_position_to_byte_offset(&rope, edit.range.start, encoding);
        let end = lsp_position_to_byte_offset(&rope, edit.range.end, encoding);
        result.replace_range(start..end, &edit.new_text);
    }

    result
}

/// Convert an LSP `Position` to a byte offset in the given rope.
fn lsp_position_to_byte_offset(rope: &helix_core::Rope, pos: lsp::Position, encoding: OffsetEncoding) -> usize {
    let line = pos.line as usize;
    if line >= rope.len_lines() {
        return rope.len_bytes();
    }
    let line_start = rope.line_to_byte(line);
    let line_slice = rope.line(line);
    let col = match encoding {
        OffsetEncoding::Utf8 => pos.character as usize,
        OffsetEncoding::Utf32 => {
            let chars = line_slice.chars().take(pos.character as usize);
            chars.map(char::len_utf8).sum()
        }
        OffsetEncoding::Utf16 => {
            let mut utf16_count: u32 = 0;
            let mut byte_offset: usize = 0;
            for ch in line_slice.chars() {
                if utf16_count >= pos.character {
                    break;
                }
                #[allow(clippy::cast_possible_truncation)]
                {
                    utf16_count += ch.len_utf16() as u32;
                }
                byte_offset += ch.len_utf8();
            }
            byte_offset
        }
    };
    let max_col = line_slice.len_bytes();
    line_start + col.min(max_col)
}

/// Compute a line-level diff between two strings, returning hunks with context.
#[must_use]
pub fn compute_file_diff(original: &str, new_text: &str, context_lines: usize) -> Vec<DiffHunk> {
    let old_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    let input = imara_diff::InternedInput::new(original, new_text);
    let mut diff = imara_diff::Diff::compute(imara_diff::Algorithm::Histogram, &input);
    diff.postprocess_lines(&input);

    let raw_changes: Vec<RawChange> = diff
        .hunks()
        .map(|hunk| RawChange {
            old_start: hunk.before.start as usize,
            old_end: hunk.before.end as usize,
            new_start: hunk.after.start as usize,
            new_end: hunk.after.end as usize,
        })
        .collect();

    build_hunks_from_changes(&old_lines, &new_lines, &raw_changes, context_lines)
}

/// Collected raw changes (line ranges that differ).
#[derive(Debug)]
struct RawChange {
    old_start: usize,
    old_end: usize,
    new_start: usize,
    new_end: usize,
}

/// Build diff hunks from raw changes with context lines.
fn build_hunks_from_changes(
    old_lines: &[&str],
    new_lines: &[&str],
    changes: &[RawChange],
    context_lines: usize,
) -> Vec<DiffHunk> {
    if changes.is_empty() {
        return Vec::new();
    }

    let mut hunks = Vec::new();
    let mut lines: Vec<DiffLine> = Vec::new();

    // Track current position in old and new files.
    let mut old_pos = 0usize;
    let mut new_pos = 0usize;

    for (i, change) in changes.iter().enumerate() {
        // Context before this change.
        let ctx_start = change.old_start.saturating_sub(context_lines);
        // But don't overlap with previous hunk.
        let actual_ctx_start = ctx_start.max(old_pos);

        // Check if we need to start a new hunk (gap between changes).
        if i > 0 && actual_ctx_start > old_pos + context_lines {
            // Emit trailing context for previous hunk.
            let trail_end = (old_pos + context_lines).min(old_lines.len());
            for j in old_pos..trail_end {
                let new_line_num = new_pos + (j - old_pos);
                lines.push(DiffLine {
                    kind: DiffChangeKind::Context,
                    content: old_lines.get(j).unwrap_or(&"").to_string(),
                    old_line_number: Some(j + 1),
                    new_line_number: Some(new_line_num + 1),
                });
            }
            new_pos += trail_end - old_pos;
            old_pos = trail_end;

            // Flush current hunk.
            if !lines.is_empty() {
                hunks.push(DiffHunk {
                    lines: std::mem::take(&mut lines),
                });
            }
        }

        // Leading context for this change.
        let lead_start = actual_ctx_start.max(old_pos);
        for j in lead_start..change.old_start {
            let new_line_num = new_pos + (j - old_pos);
            lines.push(DiffLine {
                kind: DiffChangeKind::Context,
                content: old_lines.get(j).unwrap_or(&"").to_string(),
                old_line_number: Some(j + 1),
                new_line_number: Some(new_line_num + 1),
            });
        }

        // Removed lines.
        for j in change.old_start..change.old_end {
            lines.push(DiffLine {
                kind: DiffChangeKind::Removed,
                content: old_lines.get(j).unwrap_or(&"").to_string(),
                old_line_number: Some(j + 1),
                new_line_number: None,
            });
        }
        old_pos = change.old_end;

        // Added lines.
        for j in change.new_start..change.new_end {
            lines.push(DiffLine {
                kind: DiffChangeKind::Added,
                content: new_lines.get(j).unwrap_or(&"").to_string(),
                old_line_number: None,
                new_line_number: Some(j + 1),
            });
        }
        new_pos = change.new_end;
    }

    // Trailing context for the last change.
    let trail_end = (old_pos + context_lines).min(old_lines.len());
    for j in old_pos..trail_end {
        let new_line_num = new_pos + (j - old_pos);
        lines.push(DiffLine {
            kind: DiffChangeKind::Context,
            content: old_lines.get(j).unwrap_or(&"").to_string(),
            old_line_number: Some(j + 1),
            new_line_number: Some(new_line_num + 1),
        });
    }

    if !lines.is_empty() {
        hunks.push(DiffHunk { lines });
    }

    hunks
}

/// Compute a preview from a `WorkspaceEdit`.
///
/// The `file_reader` closure provides file contents for URIs.
pub fn compute_preview(
    workspace_edit: &lsp::WorkspaceEdit,
    encoding: OffsetEncoding,
    file_reader: impl Fn(&lsp::Url) -> Option<String>,
) -> CodeActionPreview {
    let mut file_diffs = Vec::new();
    let mut lines_added = 0usize;
    let mut lines_removed = 0usize;

    // Handle `changes` field (simple text edits per file).
    if let Some(changes) = &workspace_edit.changes {
        for (uri, edits) in changes {
            if let Some(diff) = compute_single_file_diff(uri, edits, encoding, &file_reader) {
                for hunk in &diff.hunks {
                    for line in &hunk.lines {
                        match line.kind {
                            DiffChangeKind::Added => lines_added += 1,
                            DiffChangeKind::Removed => lines_removed += 1,
                            DiffChangeKind::Context => {}
                        }
                    }
                }
                file_diffs.push(diff);
            }
        }
    }

    // Handle `document_changes` field (more detailed changes).
    if let Some(doc_changes) = &workspace_edit.document_changes {
        match doc_changes {
            lsp::DocumentChanges::Edits(edits) => {
                for edit in edits {
                    let text_edits: Vec<lsp::TextEdit> = edit
                        .edits
                        .iter()
                        .map(|edit| match edit {
                            lsp::OneOf::Left(te) => te.clone(),
                            lsp::OneOf::Right(annotated) => lsp::TextEdit {
                                range: annotated.text_edit.range,
                                new_text: annotated.text_edit.new_text.clone(),
                            },
                        })
                        .collect();

                    if let Some(diff) =
                        compute_single_file_diff(&edit.text_document.uri, &text_edits, encoding, &file_reader)
                    {
                        for hunk in &diff.hunks {
                            for line in &hunk.lines {
                                match line.kind {
                                    DiffChangeKind::Added => lines_added += 1,
                                    DiffChangeKind::Removed => lines_removed += 1,
                                    DiffChangeKind::Context => {}
                                }
                            }
                        }
                        file_diffs.push(diff);
                    }
                }
            }
            lsp::DocumentChanges::Operations(ops) => {
                for op in ops {
                    if let lsp::DocumentChangeOperation::Edit(edit) = op {
                        let text_edits: Vec<lsp::TextEdit> = edit
                            .edits
                            .iter()
                            .map(|edit| match edit {
                                lsp::OneOf::Left(te) => te.clone(),
                                lsp::OneOf::Right(annotated) => lsp::TextEdit {
                                    range: annotated.text_edit.range,
                                    new_text: annotated.text_edit.new_text.clone(),
                                },
                            })
                            .collect();

                        if let Some(diff) =
                            compute_single_file_diff(&edit.text_document.uri, &text_edits, encoding, &file_reader)
                        {
                            for hunk in &diff.hunks {
                                for line in &hunk.lines {
                                    match line.kind {
                                        DiffChangeKind::Added => lines_added += 1,
                                        DiffChangeKind::Removed => lines_removed += 1,
                                        DiffChangeKind::Context => {}
                                    }
                                }
                            }
                            file_diffs.push(diff);
                        }
                    }
                }
            }
        }
    }

    CodeActionPreview {
        file_diffs,
        lines_added,
        lines_removed,
    }
}

/// Compute a diff for a single file from LSP text edits.
fn compute_single_file_diff(
    uri: &lsp::Url,
    edits: &[lsp::TextEdit],
    encoding: OffsetEncoding,
    file_reader: &impl Fn(&lsp::Url) -> Option<String>,
) -> Option<FileDiff> {
    let original = file_reader(uri)?;
    let new_text = apply_text_edits(&original, edits, encoding);

    if original == new_text {
        return None;
    }

    let hunks = compute_file_diff(&original, &new_text, 3);
    if hunks.is_empty() {
        return None;
    }

    let file_path = uri
        .to_file_path()
        .ok()
        .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
        .unwrap_or_else(|| uri.to_string());

    Some(FileDiff { file_path, hunks })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_position(line: u32, character: u32) -> lsp::Position {
        lsp::Position { line, character }
    }

    fn make_range(sl: u32, sc: u32, el: u32, ec: u32) -> lsp::Range {
        lsp::Range {
            start: make_position(sl, sc),
            end: make_position(el, ec),
        }
    }

    fn make_edit(sl: u32, sc: u32, el: u32, ec: u32, new_text: &str) -> lsp::TextEdit {
        lsp::TextEdit {
            range: make_range(sl, sc, el, ec),
            new_text: new_text.to_string(),
        }
    }

    // --- apply_text_edits ---

    #[test]
    fn apply_single_line_insertion() {
        let original = "fn main() {\n}\n";
        let edits = vec![make_edit(1, 0, 1, 0, "    println!(\"hello\");\n")];
        let result = apply_text_edits(original, &edits, OffsetEncoding::Utf16);
        assert_eq!(result, "fn main() {\n    println!(\"hello\");\n}\n");
    }

    #[test]
    fn apply_single_line_deletion() {
        let original = "line1\nline2\nline3\n";
        let edits = vec![make_edit(1, 0, 2, 0, "")];
        let result = apply_text_edits(original, &edits, OffsetEncoding::Utf16);
        assert_eq!(result, "line1\nline3\n");
    }

    #[test]
    fn apply_line_replacement() {
        let original = "let x = 1;\nlet y = 2;\n";
        let edits = vec![make_edit(0, 4, 0, 9, "foo = 42")];
        let result = apply_text_edits(original, &edits, OffsetEncoding::Utf16);
        assert_eq!(result, "let foo = 42;\nlet y = 2;\n");
    }

    #[test]
    fn apply_multiple_edits_in_one_file() {
        let original = "aaa\nbbb\nccc\n";
        let edits = vec![make_edit(0, 0, 0, 3, "AAA"), make_edit(2, 0, 2, 3, "CCC")];
        let result = apply_text_edits(original, &edits, OffsetEncoding::Utf16);
        assert_eq!(result, "AAA\nbbb\nCCC\n");
    }

    #[test]
    fn apply_empty_edit() {
        let original = "hello\nworld\n";
        let edits = vec![];
        let result = apply_text_edits(original, &edits, OffsetEncoding::Utf16);
        assert_eq!(result, "hello\nworld\n");
    }

    #[test]
    fn apply_edit_at_file_start() {
        let original = "hello\n";
        let edits = vec![make_edit(0, 0, 0, 0, "// comment\n")];
        let result = apply_text_edits(original, &edits, OffsetEncoding::Utf16);
        assert_eq!(result, "// comment\nhello\n");
    }

    #[test]
    fn apply_edit_at_file_end() {
        let original = "hello\n";
        let edits = vec![make_edit(1, 0, 1, 0, "world\n")];
        let result = apply_text_edits(original, &edits, OffsetEncoding::Utf16);
        assert_eq!(result, "hello\nworld\n");
    }

    // --- compute_file_diff ---

    #[test]
    fn diff_single_insertion() {
        let original = "line1\nline2\nline3\n";
        let new_text = "line1\nline2\nnew_line\nline3\n";
        let hunks = compute_file_diff(original, new_text, 2);
        assert_eq!(hunks.len(), 1);

        let added: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.kind == DiffChangeKind::Added)
            .collect();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].content, "new_line");
    }

    #[test]
    fn diff_single_deletion() {
        let original = "line1\nline2\nline3\n";
        let new_text = "line1\nline3\n";
        let hunks = compute_file_diff(original, new_text, 2);
        assert_eq!(hunks.len(), 1);

        let removed: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.kind == DiffChangeKind::Removed)
            .collect();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].content, "line2");
    }

    #[test]
    fn diff_line_replacement() {
        let original = "line1\nold\nline3\n";
        let new_text = "line1\nnew\nline3\n";
        let hunks = compute_file_diff(original, new_text, 1);
        assert_eq!(hunks.len(), 1);

        let removed: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.kind == DiffChangeKind::Removed)
            .collect();
        let added: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.kind == DiffChangeKind::Added)
            .collect();
        assert_eq!(removed.len(), 1);
        assert_eq!(added.len(), 1);
        assert_eq!(removed[0].content, "old");
        assert_eq!(added[0].content, "new");
    }

    #[test]
    fn diff_context_lines() {
        let original = "a\nb\nc\nd\ne\nf\ng\n";
        let new_text = "a\nb\nc\nX\ne\nf\ng\n";
        let hunks = compute_file_diff(original, new_text, 2);
        assert_eq!(hunks.len(), 1);

        let context: Vec<_> = hunks[0]
            .lines
            .iter()
            .filter(|l| l.kind == DiffChangeKind::Context)
            .collect();
        // 2 context before + 2 context after.
        assert!(context.len() >= 2, "should have at least 2 context lines");
    }

    #[test]
    fn diff_no_changes() {
        let text = "line1\nline2\n";
        let hunks = compute_file_diff(text, text, 3);
        assert!(hunks.is_empty());
    }

    #[test]
    fn diff_multiple_separated_changes() {
        // Two changes far enough apart to produce separate hunks (with context=1).
        let original = "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n";
        let new_text = "a\nB\nc\nd\ne\nf\ng\nh\nI\nj\n";
        let hunks = compute_file_diff(original, new_text, 1);
        // Should have 2 separate hunks since b→B and i→I are far apart.
        assert_eq!(hunks.len(), 2, "expected 2 separate hunks");
    }

    // --- compute_preview ---

    #[test]
    fn preview_from_workspace_edit_changes() {
        let uri = lsp::Url::parse("file:///test.rs").expect("valid url");
        let edits = vec![make_edit(1, 0, 2, 0, "")];

        let mut changes = std::collections::HashMap::new();
        changes.insert(uri.clone(), edits);

        let workspace_edit = lsp::WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
            change_annotations: None,
        };

        let original = "line1\nline2\nline3\n";
        let preview = compute_preview(&workspace_edit, OffsetEncoding::Utf16, |u| {
            if *u == uri {
                Some(original.to_string())
            } else {
                None
            }
        });

        assert_eq!(preview.lines_removed, 1);
        assert_eq!(preview.lines_added, 0);
        assert_eq!(preview.file_diffs.len(), 1);
        assert_eq!(preview.file_diffs[0].file_path, "test.rs");
    }

    #[test]
    fn preview_empty_edit() {
        let workspace_edit = lsp::WorkspaceEdit {
            changes: Some(std::collections::HashMap::new()),
            document_changes: None,
            change_annotations: None,
        };

        let preview = compute_preview(&workspace_edit, OffsetEncoding::Utf16, |_| None);

        assert_eq!(preview.lines_added, 0);
        assert_eq!(preview.lines_removed, 0);
        assert!(preview.file_diffs.is_empty());
    }

    #[test]
    fn preview_line_numbers_on_diff_lines() {
        let original = "a\nb\nc\n";
        let new_text = "a\nB\nc\n";
        let hunks = compute_file_diff(original, new_text, 1);
        assert_eq!(hunks.len(), 1);

        // Find the removed line.
        let removed = hunks[0]
            .lines
            .iter()
            .find(|l| l.kind == DiffChangeKind::Removed)
            .expect("should have a removed line");
        assert_eq!(removed.old_line_number, Some(2));
        assert_eq!(removed.new_line_number, None);

        // Find the added line.
        let added = hunks[0]
            .lines
            .iter()
            .find(|l| l.kind == DiffChangeKind::Added)
            .expect("should have an added line");
        assert_eq!(added.old_line_number, None);
        assert_eq!(added.new_line_number, Some(2));
    }
}
