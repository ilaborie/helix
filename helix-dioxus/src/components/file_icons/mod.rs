//! File type icons using dioxus-iconify generated icon data.
//!
//! Provides colorful file icons based on filename/extension matching.

use dioxus::prelude::*;

use crate::icons::{logos, vscode_icons, Icon, IconData};

/// Returns the `IconData` for a given filename.
///
/// Matches special filenames first (Dockerfile, Makefile, .gitignore, etc.),
/// then falls back to extension matching, then a default file icon.
pub fn icon_for_filename(name: &str) -> IconData {
    // Special filenames (case-insensitive)
    match name.to_lowercase().as_str() {
        "dockerfile" | "dockerfile.dev" | "dockerfile.prod" | "containerfile" => {
            return vscode_icons::FileTypeDocker;
        }
        "makefile" | "gnumakefile" => return vscode_icons::FileTypeShell,
        ".gitignore" | ".gitattributes" | ".gitmodules" => return vscode_icons::FileTypeGit,
        "cargo.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" | "composer.lock" | "gemfile.lock"
        | "flake.lock" => return vscode_icons::DefaultFile,
        "cargo.toml" => return vscode_icons::FileTypeCargo,
        _ => {}
    }

    // Extension matching
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        // Languages
        "rs" => vscode_icons::FileTypeRust,
        "py" | "pyi" | "pyw" | "pyx" => vscode_icons::FileTypePython,
        "js" | "mjs" | "cjs" | "jsx" => vscode_icons::FileTypeJs,
        "ts" | "mts" | "cts" | "tsx" => vscode_icons::FileTypeTypescript,
        "go" => logos::Go,
        "java" => vscode_icons::FileTypeJava,
        "c" | "h" => vscode_icons::FileTypeC,
        "cpp" | "hpp" | "cc" | "cxx" | "hh" | "hxx" => vscode_icons::FileTypeCpp,
        "cs" => vscode_icons::FileTypeCsharp,
        "rb" | "erb" | "gemspec" => vscode_icons::FileTypeRuby,
        "php" => vscode_icons::FileTypePhp,
        "swift" => vscode_icons::FileTypeSwift,
        "kt" | "kts" => vscode_icons::FileTypeKotlin,
        "scala" | "sc" => vscode_icons::FileTypeScala,
        "lua" => vscode_icons::FileTypeLua,
        "ex" | "exs" | "heex" => vscode_icons::FileTypeElixir,
        "hs" | "lhs" => vscode_icons::FileTypeHaskell,
        "ml" | "mli" => vscode_icons::FileTypeOcaml,
        "zig" => vscode_icons::FileTypeZig,
        "nim" | "nims" | "nimble" => vscode_icons::FileTypeNim,
        // Web
        "html" | "htm" => vscode_icons::FileTypeHtml,
        "css" => vscode_icons::FileTypeCss,
        "scss" | "sass" | "less" => vscode_icons::FileTypeScss,
        "svg" => vscode_icons::FileTypeSvg,
        // Data / Config
        "json" | "json5" | "jsonc" => vscode_icons::FileTypeJson,
        "yaml" | "yml" => vscode_icons::FileTypeYaml,
        "toml" => vscode_icons::FileTypeToml,
        "xml" | "xsl" | "xslt" | "plist" => vscode_icons::FileTypeXml,
        "md" | "mdx" | "markdown" => vscode_icons::FileTypeMarkdown,
        // Build / DevOps
        "sh" | "bash" | "zsh" | "fish" | "nu" => vscode_icons::FileTypeShell,
        "nix" => vscode_icons::FileTypeNix,
        // Other
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "bmp" | "tiff" => vscode_icons::FileTypeImage,
        // Default (includes .lock and unknown extensions)
        _ => vscode_icons::DefaultFile,
    }
}

/// Returns the `IconData` for a folder icon.
pub fn folder_icon(is_open: bool) -> IconData {
    if is_open {
        vscode_icons::DefaultFolderOpened
    } else {
        vscode_icons::DefaultFolder
    }
}

/// Dioxus component that renders a file type icon based on the filename.
#[component]
pub fn FileTypeIcon(name: String, #[props(default = 16)] size: u32) -> Element {
    let data = icon_for_filename(&name);
    rsx! {
        span { class: "file-type-icon",
            Icon { data, size: "{size}" }
        }
    }
}

/// Dioxus component that renders a folder icon (open or closed).
#[component]
pub fn FolderTypeIcon(#[props(default = false)] is_open: bool, #[props(default = 16)] size: u32) -> Element {
    let data = folder_icon(is_open);
    rsx! {
        span { class: "file-type-icon",
            Icon { data, size: "{size}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_file() {
        assert_eq!(icon_for_filename("main.rs").name, "vscode-icons:file-type-rust");
    }

    #[test]
    fn test_python_file() {
        assert_eq!(icon_for_filename("app.py").name, "vscode-icons:file-type-python");
    }

    #[test]
    fn test_javascript_file() {
        assert_eq!(icon_for_filename("index.js").name, "vscode-icons:file-type-js");
    }

    #[test]
    fn test_typescript_file() {
        assert_eq!(icon_for_filename("app.tsx").name, "vscode-icons:file-type-typescript");
    }

    #[test]
    fn test_special_filename_dockerfile() {
        assert_eq!(icon_for_filename("Dockerfile").name, "vscode-icons:file-type-docker");
    }

    #[test]
    fn test_special_filename_makefile() {
        assert_eq!(icon_for_filename("Makefile").name, "vscode-icons:file-type-shell");
    }

    #[test]
    fn test_special_filename_gitignore() {
        assert_eq!(icon_for_filename(".gitignore").name, "vscode-icons:file-type-git");
    }

    #[test]
    fn test_lock_file() {
        assert_eq!(icon_for_filename("Cargo.lock").name, "vscode-icons:default-file");
    }

    #[test]
    fn test_lock_extension() {
        assert_eq!(icon_for_filename("something.lock").name, "vscode-icons:default-file");
    }

    #[test]
    fn test_default_file() {
        assert_eq!(icon_for_filename("unknown.xyz").name, "vscode-icons:default-file");
    }

    #[test]
    fn test_folder_closed() {
        assert_eq!(folder_icon(false).name, "vscode-icons:default-folder");
    }

    #[test]
    fn test_folder_open() {
        assert_eq!(folder_icon(true).name, "vscode-icons:default-folder-opened");
    }

    #[test]
    fn test_web_extensions() {
        assert_eq!(icon_for_filename("index.html").name, "vscode-icons:file-type-html");
        assert_eq!(icon_for_filename("styles.css").name, "vscode-icons:file-type-css");
        assert_eq!(icon_for_filename("styles.scss").name, "vscode-icons:file-type-scss");
    }

    #[test]
    fn test_config_extensions() {
        assert_eq!(icon_for_filename("config.json").name, "vscode-icons:file-type-json");
        assert_eq!(icon_for_filename("config.yaml").name, "vscode-icons:file-type-yaml");
        assert_eq!(icon_for_filename("config.toml").name, "vscode-icons:file-type-toml");
    }

    #[test]
    fn test_image_extensions() {
        assert_eq!(icon_for_filename("photo.png").name, "vscode-icons:file-type-image");
        assert_eq!(icon_for_filename("photo.jpg").name, "vscode-icons:file-type-image");
    }

    #[test]
    fn test_shell_extensions() {
        assert_eq!(icon_for_filename("script.sh").name, "vscode-icons:file-type-shell");
        assert_eq!(icon_for_filename("config.fish").name, "vscode-icons:file-type-shell");
    }

    #[test]
    fn test_case_insensitive_special_filenames() {
        assert_eq!(icon_for_filename("dockerfile").name, "vscode-icons:file-type-docker");
        assert_eq!(icon_for_filename("DOCKERFILE").name, "vscode-icons:file-type-docker");
    }

    #[test]
    fn test_c_family() {
        assert_eq!(icon_for_filename("main.c").name, "vscode-icons:file-type-c");
        assert_eq!(icon_for_filename("lib.h").name, "vscode-icons:file-type-c");
        assert_eq!(icon_for_filename("main.cpp").name, "vscode-icons:file-type-cpp");
        assert_eq!(icon_for_filename("main.cc").name, "vscode-icons:file-type-cpp");
        assert_eq!(icon_for_filename("app.cs").name, "vscode-icons:file-type-csharp");
    }

    #[test]
    fn test_go_file() {
        assert_eq!(icon_for_filename("main.go").name, "logos:go");
    }

    #[test]
    fn test_cargo_toml() {
        assert_eq!(icon_for_filename("Cargo.toml").name, "vscode-icons:file-type-cargo");
    }
}
