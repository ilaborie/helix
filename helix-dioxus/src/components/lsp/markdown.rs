//! Markdown to HTML conversion for LSP content.
//!
//! Converts markdown text (from hover, signature help, etc.) to HTML
//! for rendering via `dangerous_inner_html` in the `WebView`.
//! Optionally syntax-highlights fenced code blocks via a callback.

use comrak::nodes::{NodeHtmlBlock, NodeValue};
use comrak::{Arena, Options, format_html, parse_document};

/// Escape special HTML characters in text content.
fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Convert markdown text to an HTML string.
///
/// Uses `comrak` with strikethrough and raw HTML passthrough enabled.
///
/// If `highlight_code` is provided, fenced code blocks with a language tag
/// are passed to the callback for syntax highlighting. The callback receives
/// `(code_text, language)` and should return `Some(highlighted_html)` with
/// pre-escaped HTML spans, or `None` to fall back to plain text.
#[allow(clippy::type_complexity)] // dyn Fn callback type is inherently complex
pub fn markdown_to_html(md: &str, highlight_code: Option<&dyn Fn(&str, &str) -> Option<String>>) -> String {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.render.r#unsafe = true;
    let root = parse_document(&arena, md, &options);

    if let Some(highlight_fn) = highlight_code {
        // Collect code block nodes first to avoid mutating the tree while iterating.
        let code_nodes: Vec<_> = root
            .descendants()
            .filter(|node| matches!(node.data.borrow().value, NodeValue::CodeBlock(_)))
            .collect();

        for node in code_nodes {
            let replacement = {
                let data = node.data.borrow();
                let NodeValue::CodeBlock(ref cb) = data.value else {
                    continue;
                };
                if !cb.fenced || cb.info.is_empty() {
                    continue;
                }
                if let Some(highlighted) = highlight_fn(&cb.literal, &cb.info) {
                    format!(
                        "<pre><code class=\"language-{}\">{}</code></pre>\n",
                        escape_html(&cb.info),
                        highlighted,
                    )
                } else {
                    // Leave the node unchanged — comrak will render it normally.
                    continue;
                }
            };

            let html_node = arena.alloc(comrak::nodes::AstNode::from(
                NodeValue::HtmlBlock(NodeHtmlBlock {
                    block_type: 6,
                    literal: replacement,
                }),
            ));
            node.insert_before(html_node);
            node.detach();
        }
    }

    let mut html_output = String::new();
    format_html(root, &options, &mut html_output).expect("comrak format_html failed");
    html_output
}

/// Syntax-highlight a code block using tree-sitter.
///
/// Returns `Some(html)` with `<span style="color: #hex">...</span>` wrapped tokens,
/// or `None` if the language is not recognized or parsing fails.
#[allow(clippy::cast_possible_truncation)] // rope byte length fits in u32 for typical code blocks
pub fn highlight_code_block(
    code: &str,
    language: &str,
    theme: &helix_view::Theme,
    loader: &helix_core::syntax::Loader,
) -> Option<String> {
    use helix_core::syntax::HighlightEvent;
    use helix_core::{Rope, Syntax};

    use crate::state::color_to_css;

    let rope = Rope::from_str(code);
    let text_slice = rope.slice(..);

    let lang = loader.language_for_name(language)?;
    let syntax = Syntax::new(text_slice, lang, loader).ok()?;

    let end_byte = rope.len_bytes() as u32;
    let mut highlighter = syntax.highlighter(text_slice, loader, 0..end_byte);
    let text_style = helix_view::theme::Style::default();
    let mut current_style = text_style;
    let mut pos: u32 = 0;

    let mut html = String::with_capacity(code.len() * 2);

    loop {
        let next_event_pos = highlighter.next_event_offset();
        let span_end = if next_event_pos == u32::MAX {
            end_byte
        } else {
            next_event_pos
        };

        if span_end > pos {
            let start = rope.byte_to_char(pos as usize);
            let end = rope.byte_to_char(span_end as usize);
            let text = rope.slice(start..end).to_string();
            let escaped = escape_html(&text);

            if let Some(fg) = current_style.fg {
                if let Some(css_color) = color_to_css(fg) {
                    html.push_str("<span style=\"color: ");
                    html.push_str(&css_color);
                    html.push_str("\">");
                    html.push_str(&escaped);
                    html.push_str("</span>");
                } else {
                    html.push_str(&escaped);
                }
            } else {
                html.push_str(&escaped);
            }
        }

        if next_event_pos == u32::MAX || next_event_pos >= end_byte {
            break;
        }

        pos = next_event_pos;
        let (event, highlights) = highlighter.advance();

        let base = match event {
            HighlightEvent::Refresh => text_style,
            HighlightEvent::Push => current_style,
        };

        current_style = highlights.fold(base, |acc, highlight| acc.patch(theme.highlight(highlight)));
    }

    Some(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(markdown_to_html("", None), "");
    }

    #[test]
    fn test_plain_text() {
        let result = markdown_to_html("hello world", None);
        assert_eq!(result, "<p>hello world</p>\n");
    }

    #[test]
    fn test_heading() {
        let result = markdown_to_html("# Title", None);
        assert_eq!(result, "<h1>Title</h1>\n");
    }

    #[test]
    fn test_multiple_heading_levels() {
        let result = markdown_to_html("## Subtitle\n### Sub-sub", None);
        assert!(result.contains("<h2>Subtitle</h2>"));
        assert!(result.contains("<h3>Sub-sub</h3>"));
    }

    #[test]
    fn test_inline_code() {
        let result = markdown_to_html("Use `foo()` here", None);
        assert_eq!(result, "<p>Use <code>foo()</code> here</p>\n");
    }

    #[test]
    fn test_code_block() {
        let result = markdown_to_html("```rust\nfn main() {}\n```", None);
        assert!(result.contains("<pre><code class=\"language-rust\">"));
        assert!(result.contains("fn main() {}"));
        assert!(result.contains("</code></pre>"));
    }

    #[test]
    fn test_bold_and_italic() {
        let result = markdown_to_html("**bold** and *italic*", None);
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_unordered_list() {
        let result = markdown_to_html("- one\n- two", None);
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>one</li>"));
        assert!(result.contains("<li>two</li>"));
        assert!(result.contains("</ul>"));
    }

    #[test]
    fn test_ordered_list() {
        let result = markdown_to_html("1. first\n2. second", None);
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>first</li>"));
        assert!(result.contains("<li>second</li>"));
        assert!(result.contains("</ol>"));
    }

    #[test]
    fn test_horizontal_rule() {
        let result = markdown_to_html("above\n\n---\n\nbelow", None);
        assert!(result.contains("<hr />"));
    }

    #[test]
    fn test_strikethrough() {
        let result = markdown_to_html("~~deleted~~", None);
        assert!(result.contains("<del>deleted</del>"));
    }

    #[test]
    fn test_link() {
        let result = markdown_to_html("[click](https://example.com)", None);
        assert!(result.contains("<a href=\"https://example.com\">click</a>"));
    }

    #[test]
    fn test_html_code_tags_converted() {
        let result = markdown_to_html("<code>Vec</code>", None);
        assert!(result.contains("<code>Vec</code>"));
    }

    #[test]
    fn test_paragraph_separation() {
        let result = markdown_to_html("first\n\nsecond", None);
        assert!(result.contains("<p>first</p>"));
        assert!(result.contains("<p>second</p>"));
    }

    // --- Code highlighting callback tests ---

    #[test]
    fn test_code_block_with_highlight_callback() {
        let highlighter = |code: &str, lang: &str| -> Option<String> {
            if lang == "rust" {
                Some(format!("<span class=\"kw\">highlighted</span>: {}", escape_html(code)))
            } else {
                None
            }
        };

        let result = markdown_to_html("```rust\nfn main() {}\n```", Some(&highlighter));
        assert!(result.contains("<pre><code class=\"language-rust\">"));
        assert!(result.contains("<span class=\"kw\">highlighted</span>"));
        assert!(result.contains("</code></pre>"));
    }

    #[test]
    fn test_code_block_callback_returns_none_falls_back() {
        let highlighter = |_code: &str, _lang: &str| -> Option<String> { None };

        let result = markdown_to_html("```rust\nfn main() {}\n```", Some(&highlighter));
        // Should fall back to standard comrak rendering
        assert!(result.contains("<pre><code class=\"language-rust\">"));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn test_code_block_without_language_not_highlighted() {
        let called = std::cell::Cell::new(false);
        let highlighter = |_code: &str, _lang: &str| -> Option<String> {
            called.set(true);
            Some("should not appear".to_string())
        };

        let result = markdown_to_html("```\nplain code\n```", Some(&highlighter));
        assert!(
            !called.get(),
            "Highlighter should not be called for unlabeled code blocks"
        );
        assert!(result.contains("plain code"));
    }

    #[test]
    fn test_mixed_content_with_highlight() {
        let highlighter = |_code: &str, lang: &str| -> Option<String> {
            if lang == "rust" {
                Some("<span>colored</span>".to_string())
            } else {
                None
            }
        };

        let md = "# Title\n\nSome text.\n\n```rust\ncode\n```\n\nMore text.";
        let result = markdown_to_html(md, Some(&highlighter));
        assert!(result.contains("<h1>Title</h1>"));
        assert!(result.contains("<p>Some text.</p>"));
        assert!(result.contains("<span>colored</span>"));
        assert!(result.contains("<p>More text.</p>"));
    }

    // --- escape_html tests ---

    #[test]
    fn test_escape_html_special_chars() {
        assert_eq!(
            escape_html("<b>&\"hello\"</b>"),
            "&lt;b&gt;&amp;&quot;hello&quot;&lt;/b&gt;"
        );
    }

    #[test]
    fn test_escape_html_no_special_chars() {
        assert_eq!(escape_html("hello world"), "hello world");
    }
}
