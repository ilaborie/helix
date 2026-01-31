use helix_loader::grammar::{build_grammars, fetch_grammars};

fn main() {
    if std::env::var("HELIX_DISABLE_AUTO_GRAMMAR_BUILD").is_err() {
        if let Err(err) = fetch_grammars() {
            panic!("Failed to fetch tree-sitter grammars: {err}");
        }
        let target = std::env::var("TARGET").ok();
        if let Err(err) = build_grammars(target) {
            panic!("Failed to compile tree-sitter grammars: {err}");
        }
    }
}
