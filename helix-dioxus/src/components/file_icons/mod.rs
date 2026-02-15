//! File type icons from the Material Icon Theme.
//!
//! Provides colorful SVG file icons based on filename/extension matching.

mod svgs;

use dioxus::prelude::*;

/// Returns the SVG string for a given filename.
///
/// Matches special filenames first (Dockerfile, Makefile, .gitignore, etc.),
/// then falls back to extension matching, then a default file icon.
pub fn icon_svg_for_filename(name: &str) -> &'static str {
    // Special filenames (case-insensitive)
    match name.to_lowercase().as_str() {
        "dockerfile" | "dockerfile.dev" | "dockerfile.prod" | "containerfile" => {
            return svgs::DOCKER
        }
        "makefile" | "gnumakefile" => return svgs::MAKEFILE,
        ".gitignore" | ".gitattributes" | ".gitmodules" => return svgs::GIT,
        "cargo.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml"
        | "composer.lock" | "gemfile.lock" | "flake.lock" => return svgs::LOCK,
        _ => {}
    }

    // Extension matching
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        // Languages
        "rs" => svgs::RUST,
        "py" | "pyi" | "pyw" | "pyx" => svgs::PYTHON,
        "js" | "mjs" | "cjs" | "jsx" => svgs::JAVASCRIPT,
        "ts" | "mts" | "cts" | "tsx" => svgs::TYPESCRIPT,
        "go" => svgs::GO,
        "java" => svgs::JAVA,
        "c" | "h" => svgs::C,
        "cpp" | "hpp" | "cc" | "cxx" | "hh" | "hxx" => svgs::CPP,
        "cs" => svgs::CSHARP,
        "rb" | "erb" | "gemspec" => svgs::RUBY,
        "php" => svgs::PHP,
        "swift" => svgs::SWIFT,
        "kt" | "kts" => svgs::KOTLIN,
        "scala" | "sc" => svgs::SCALA,
        "lua" => svgs::LUA,
        "ex" | "exs" | "heex" => svgs::ELIXIR,
        "hs" | "lhs" => svgs::HASKELL,
        "ml" | "mli" => svgs::OCAML,
        "zig" => svgs::ZIG,
        "nim" | "nims" | "nimble" => svgs::NIM,
        // Web
        "html" | "htm" => svgs::HTML,
        "css" => svgs::CSS,
        "scss" | "sass" | "less" => svgs::SCSS,
        "svg" => svgs::SVG,
        // Data / Config
        "json" | "json5" | "jsonc" => svgs::JSON,
        "yaml" | "yml" => svgs::YAML,
        "toml" => svgs::TOML,
        "xml" | "xsl" | "xslt" | "plist" => svgs::XML,
        "md" | "mdx" | "markdown" => svgs::MARKDOWN,
        // Build / DevOps
        "sh" | "bash" | "zsh" | "fish" | "nu" => svgs::SHELL,
        "nix" => svgs::NIX,
        // Other
        "lock" => svgs::LOCK,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "bmp" | "tiff" => svgs::IMAGE,
        // Default
        _ => svgs::DEFAULT_FILE,
    }
}

/// Returns the SVG string for a folder icon.
pub fn folder_icon_svg(is_open: bool) -> &'static str {
    if is_open {
        svgs::FOLDER_OPEN
    } else {
        svgs::FOLDER
    }
}

/// Dioxus component that renders a file type icon based on the filename.
#[component]
pub fn FileTypeIcon(name: String, #[props(default = 16)] size: u32) -> Element {
    let svg = icon_svg_for_filename(&name);
    rsx! {
        span {
            class: "file-type-icon",
            style: "width: {size}px; height: {size}px;",
            dangerous_inner_html: "{svg}",
        }
    }
}

/// Dioxus component that renders a folder icon (open or closed).
#[component]
pub fn FolderTypeIcon(
    #[props(default = false)] is_open: bool,
    #[props(default = 16)] size: u32,
) -> Element {
    let svg = folder_icon_svg(is_open);
    rsx! {
        span {
            class: "file-type-icon",
            style: "width: {size}px; height: {size}px;",
            dangerous_inner_html: "{svg}",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_file() {
        assert_eq!(icon_svg_for_filename("main.rs"), svgs::RUST);
    }

    #[test]
    fn test_python_file() {
        assert_eq!(icon_svg_for_filename("app.py"), svgs::PYTHON);
    }

    #[test]
    fn test_javascript_file() {
        assert_eq!(icon_svg_for_filename("index.js"), svgs::JAVASCRIPT);
    }

    #[test]
    fn test_typescript_file() {
        assert_eq!(icon_svg_for_filename("app.tsx"), svgs::TYPESCRIPT);
    }

    #[test]
    fn test_special_filename_dockerfile() {
        assert_eq!(icon_svg_for_filename("Dockerfile"), svgs::DOCKER);
    }

    #[test]
    fn test_special_filename_makefile() {
        assert_eq!(icon_svg_for_filename("Makefile"), svgs::MAKEFILE);
    }

    #[test]
    fn test_special_filename_gitignore() {
        assert_eq!(icon_svg_for_filename(".gitignore"), svgs::GIT);
    }

    #[test]
    fn test_lock_file() {
        assert_eq!(icon_svg_for_filename("Cargo.lock"), svgs::LOCK);
    }

    #[test]
    fn test_lock_extension() {
        assert_eq!(icon_svg_for_filename("something.lock"), svgs::LOCK);
    }

    #[test]
    fn test_default_file() {
        assert_eq!(icon_svg_for_filename("unknown.xyz"), svgs::DEFAULT_FILE);
    }

    #[test]
    fn test_folder_closed() {
        assert_eq!(folder_icon_svg(false), svgs::FOLDER);
    }

    #[test]
    fn test_folder_open() {
        assert_eq!(folder_icon_svg(true), svgs::FOLDER_OPEN);
    }

    #[test]
    fn test_web_extensions() {
        assert_eq!(icon_svg_for_filename("index.html"), svgs::HTML);
        assert_eq!(icon_svg_for_filename("styles.css"), svgs::CSS);
        assert_eq!(icon_svg_for_filename("styles.scss"), svgs::SCSS);
    }

    #[test]
    fn test_config_extensions() {
        assert_eq!(icon_svg_for_filename("config.json"), svgs::JSON);
        assert_eq!(icon_svg_for_filename("config.yaml"), svgs::YAML);
        assert_eq!(icon_svg_for_filename("config.toml"), svgs::TOML);
    }

    #[test]
    fn test_image_extensions() {
        assert_eq!(icon_svg_for_filename("photo.png"), svgs::IMAGE);
        assert_eq!(icon_svg_for_filename("photo.jpg"), svgs::IMAGE);
    }

    #[test]
    fn test_shell_extensions() {
        assert_eq!(icon_svg_for_filename("script.sh"), svgs::SHELL);
        assert_eq!(icon_svg_for_filename("config.fish"), svgs::SHELL);
    }

    #[test]
    fn test_case_insensitive_special_filenames() {
        assert_eq!(icon_svg_for_filename("dockerfile"), svgs::DOCKER);
        assert_eq!(icon_svg_for_filename("DOCKERFILE"), svgs::DOCKER);
    }

    #[test]
    fn test_c_family() {
        assert_eq!(icon_svg_for_filename("main.c"), svgs::C);
        assert_eq!(icon_svg_for_filename("lib.h"), svgs::C);
        assert_eq!(icon_svg_for_filename("main.cpp"), svgs::CPP);
        assert_eq!(icon_svg_for_filename("main.cc"), svgs::CPP);
        assert_eq!(icon_svg_for_filename("app.cs"), svgs::CSHARP);
    }
}
