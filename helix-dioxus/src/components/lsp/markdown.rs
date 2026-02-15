//! Markdown to HTML conversion for LSP content.
//!
//! Converts markdown text (from hover, signature help, etc.) to HTML
//! for rendering via `dangerous_inner_html` in the `WebView`.

use pulldown_cmark::{Event, Options, Parser};

/// Convert markdown text to an HTML string.
///
/// Uses `pulldown-cmark` with strikethrough support enabled.
/// Filters `<code>...</code>` HTML tags into `Event::Code` events
/// (matching helix-term behavior) so they render as inline code.
pub fn markdown_to_html(md: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(md, options);

    // Transform text in `<code>` blocks into `Event::Code`
    // (same approach as helix-term/src/ui/markdown.rs)
    let mut in_code = false;
    let parser = parser.filter_map(|event| match event {
        Event::Html(tag)
            if tag.starts_with("<code") && matches!(tag.chars().nth(5), Some(' ' | '>')) =>
        {
            in_code = true;
            None
        }
        Event::Html(tag) if *tag == *"</code>" => {
            in_code = false;
            None
        }
        Event::Text(text) if in_code => Some(Event::Code(text)),
        _ => Some(event),
    });

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert_eq!(markdown_to_html(""), "");
    }

    #[test]
    fn test_plain_text() {
        let result = markdown_to_html("hello world");
        assert_eq!(result, "<p>hello world</p>\n");
    }

    #[test]
    fn test_heading() {
        let result = markdown_to_html("# Title");
        assert_eq!(result, "<h1>Title</h1>\n");
    }

    #[test]
    fn test_multiple_heading_levels() {
        let result = markdown_to_html("## Subtitle\n### Sub-sub");
        assert!(result.contains("<h2>Subtitle</h2>"));
        assert!(result.contains("<h3>Sub-sub</h3>"));
    }

    #[test]
    fn test_inline_code() {
        let result = markdown_to_html("Use `foo()` here");
        assert_eq!(result, "<p>Use <code>foo()</code> here</p>\n");
    }

    #[test]
    fn test_code_block() {
        let result = markdown_to_html("```rust\nfn main() {}\n```");
        assert!(result.contains("<pre><code class=\"language-rust\">"));
        assert!(result.contains("fn main() {}"));
        assert!(result.contains("</code></pre>"));
    }

    #[test]
    fn test_bold_and_italic() {
        let result = markdown_to_html("**bold** and *italic*");
        assert!(result.contains("<strong>bold</strong>"));
        assert!(result.contains("<em>italic</em>"));
    }

    #[test]
    fn test_unordered_list() {
        let result = markdown_to_html("- one\n- two");
        assert!(result.contains("<ul>"));
        assert!(result.contains("<li>one</li>"));
        assert!(result.contains("<li>two</li>"));
        assert!(result.contains("</ul>"));
    }

    #[test]
    fn test_ordered_list() {
        let result = markdown_to_html("1. first\n2. second");
        assert!(result.contains("<ol>"));
        assert!(result.contains("<li>first</li>"));
        assert!(result.contains("<li>second</li>"));
        assert!(result.contains("</ol>"));
    }

    #[test]
    fn test_horizontal_rule() {
        let result = markdown_to_html("above\n\n---\n\nbelow");
        assert!(result.contains("<hr />"));
    }

    #[test]
    fn test_strikethrough() {
        let result = markdown_to_html("~~deleted~~");
        assert!(result.contains("<del>deleted</del>"));
    }

    #[test]
    fn test_link() {
        let result = markdown_to_html("[click](https://example.com)");
        assert!(result.contains("<a href=\"https://example.com\">click</a>"));
    }

    #[test]
    fn test_html_code_tags_converted() {
        let result = markdown_to_html("<code>Vec</code>");
        assert!(result.contains("<code>Vec</code>"));
    }

    #[test]
    fn test_paragraph_separation() {
        let result = markdown_to_html("first\n\nsecond");
        assert!(result.contains("<p>first</p>"));
        assert!(result.contains("<p>second</p>"));
    }
}
