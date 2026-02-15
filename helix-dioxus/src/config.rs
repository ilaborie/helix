//! GUI-specific configuration for helix-dioxus.
//!
//! Configuration is loaded from `~/.config/helix/dhx.toml` (global) and
//! `.helix/dhx.toml` (workspace), with workspace values overriding global.
//! Editor settings (keybindings, theme, LSP) are loaded from the standard
//! helix `config.toml` and `languages.toml` with the same merging strategy.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

/// GUI-specific configuration loaded from `dhx.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct DhxConfig {
    pub window: WindowConfig,
    pub font: FontConfig,
    pub logging: LoggingConfig,
    pub dialog: DialogConfig,
}

/// Dialog/picker interaction mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DialogSearchMode {
    /// Current behavior: typing filters directly, arrows navigate.
    #[default]
    Direct,
    /// Vim-style: j/k navigate, `/` focuses search input.
    VimStyle,
}

/// Dialog configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct DialogConfig {
    pub search_mode: DialogSearchMode,
}

/// Window configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub title: String,
    pub width: f64,
    pub height: f64,
}

/// Font configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    pub family: String,
    pub size: f64,
    pub weight: Option<u16>,
    pub ligatures: bool,
    /// OpenType feature tags (e.g. `["calt", "liga", "ss01"]`).
    /// When non-empty, overrides the `ligatures` boolean for `font-feature-settings`.
    pub features: Vec<String>,
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub log_file: Option<PathBuf>,
    pub level: String,
    pub suppressed_patterns: Vec<String>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "helix-dioxus".to_string(),
            width: 1200.0,
            height: 800.0,
        }
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "'JetBrains Mono', 'Fira Code', 'SF Mono', Menlo, Monaco, monospace"
                .to_string(),
            size: 14.0,
            weight: None,
            ligatures: true,
            features: Vec::new(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_file: Some(PathBuf::from("/tmp/helix-dioxus.log")),
            level: "info".to_string(),
            suppressed_patterns: vec![
                "SelectionDidChange".to_string(),
                "Dispatched unknown event".to_string(),
                "mousemove".to_string(),
                "mouseenter".to_string(),
                "mouseleave".to_string(),
                "pointermove".to_string(),
                "pointerenter".to_string(),
                "pointerleave".to_string(),
            ],
        }
    }
}

impl DhxConfig {
    /// Load configuration by merging global (`~/.config/helix/dhx.toml`) and
    /// workspace (`.helix/dhx.toml`) files. Workspace values override global.
    ///
    /// Falls back to defaults if neither file exists.
    pub fn load_default() -> Result<Self> {
        let global_path = helix_loader::config_dir().join("dhx.toml");
        let workspace_path = helix_loader::find_workspace()
            .0
            .join(".helix")
            .join("dhx.toml");

        let global = Self::read_toml_file(&global_path);
        let local = Self::read_toml_file(&workspace_path);

        match (global, local) {
            (Some(g), Some(l)) => {
                let merged = helix_loader::merge_toml_values(g, l, 3);
                Ok(merged.try_into()?)
            }
            (Some(v), None) | (None, Some(v)) => Ok(v.try_into()?),
            (None, None) => Ok(Self::default()),
        }
    }

    /// Load configuration from a specific file path.
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str::<DhxConfig>(&content)?;
        Ok(config)
    }

    /// Read and parse a TOML file, returning `None` on missing file or parse error.
    fn read_toml_file(path: &Path) -> Option<toml::Value> {
        let content = std::fs::read_to_string(path).ok()?;
        match toml::from_str::<toml::Value>(&content) {
            Ok(v) => Some(v),
            Err(err) => {
                log::warn!("Failed to parse {}: {err}", path.display());
                None
            }
        }
    }

    /// Set the window title.
    #[must_use]
    pub fn with_window_title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    /// Set the window dimensions.
    #[must_use]
    pub fn with_window_size(mut self, width: f64, height: f64) -> Self {
        self.window.width = width;
        self.window.height = height;
        self
    }

    /// Set the font family.
    #[must_use]
    pub fn with_font_family(mut self, family: impl Into<String>) -> Self {
        self.font.family = family.into();
        self
    }

    /// Set the font size in pixels.
    #[must_use]
    pub fn with_font_size(mut self, size: f64) -> Self {
        self.font.size = size;
        self
    }

    /// Set whether font ligatures are enabled.
    #[must_use]
    pub fn with_font_ligatures(mut self, enabled: bool) -> Self {
        self.font.ligatures = enabled;
        self
    }

    /// Set the log file path.
    #[must_use]
    pub fn with_log_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.logging.log_file = Some(path.into());
        self
    }

    /// Set the log level (e.g., "info", "debug", "warn").
    #[must_use]
    pub fn with_log_level(mut self, level: impl Into<String>) -> Self {
        self.logging.level = level.into();
        self
    }

    /// Generate CSS custom properties for font configuration.
    ///
    /// Returns a raw CSS `:root` block (no `<style>` wrapper) that overrides defaults.
    #[must_use]
    pub fn font_css(&self) -> String {
        let features = if self.font.features.is_empty() {
            if self.font.ligatures {
                "normal".to_string()
            } else {
                "none".to_string()
            }
        } else {
            self.font
                .features
                .iter()
                .map(|f| format!("\"{f}\" 1"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let weight = self
            .font
            .weight
            .map_or(String::new(), |w| format!(" --font-weight: {w};"));
        format!(
            ":root {{ --font-mono: {}; --font-size: {}px; --font-ligatures: {};{} }}",
            self.font.family, self.font.size, features, weight
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = DhxConfig::default();
        assert_eq!(config.window.title, "helix-dioxus");
        assert!((config.window.width - 1200.0).abs() < f64::EPSILON);
        assert!((config.window.height - 800.0).abs() < f64::EPSILON);
        assert!((config.font.size - 14.0).abs() < f64::EPSILON);
        assert!(config.font.ligatures);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn builder_methods_override_defaults() {
        let config = DhxConfig::default()
            .with_window_title("My IDE")
            .with_window_size(800.0, 600.0)
            .with_font_family("'Fira Code'")
            .with_font_size(16.0)
            .with_font_ligatures(false)
            .with_log_level("debug");

        assert_eq!(config.window.title, "My IDE");
        assert!((config.window.width - 800.0).abs() < f64::EPSILON);
        assert!((config.window.height - 600.0).abs() < f64::EPSILON);
        assert_eq!(config.font.family, "'Fira Code'");
        assert!((config.font.size - 16.0).abs() < f64::EPSILON);
        assert!(!config.font.ligatures);
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn font_css_generates_valid_style() {
        let config = DhxConfig::default();
        let css = config.font_css();
        assert!(css.starts_with(":root {"));
        assert!(css.contains("--font-mono:"));
        assert!(css.contains("--font-size: 14px"));
        assert!(css.contains("--font-ligatures: normal"));
    }

    #[test]
    fn font_css_with_ligatures_disabled() {
        let config = DhxConfig::default().with_font_ligatures(false);
        let css = config.font_css();
        assert!(css.contains("--font-ligatures: none"));
    }

    #[test]
    fn deserialize_partial_config() {
        let toml_str = r#"
[window]
title = "custom"

[font]
size = 18.0
"#;
        let config = toml::from_str::<DhxConfig>(toml_str).expect("should deserialize");
        assert_eq!(config.window.title, "custom");
        // Width should be default
        assert!((config.window.width - 1200.0).abs() < f64::EPSILON);
        assert!((config.font.size - 18.0).abs() < f64::EPSILON);
        // Ligatures should be default
        assert!(config.font.ligatures);
    }

    #[test]
    fn load_from_nonexistent_path_returns_error() {
        let result = DhxConfig::load_from(Path::new("/nonexistent/dhx.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn default_dialog_search_mode_is_direct() {
        let config = DhxConfig::default();
        assert_eq!(config.dialog.search_mode, DialogSearchMode::Direct);
    }

    #[test]
    fn deserialize_vim_style_dialog_mode() {
        let toml_str = r#"
[dialog]
search_mode = "vim-style"
"#;
        let config = toml::from_str::<DhxConfig>(toml_str).expect("should deserialize");
        assert_eq!(config.dialog.search_mode, DialogSearchMode::VimStyle);
    }

    #[test]
    fn deserialize_direct_dialog_mode() {
        let toml_str = r#"
[dialog]
search_mode = "direct"
"#;
        let config = toml::from_str::<DhxConfig>(toml_str).expect("should deserialize");
        assert_eq!(config.dialog.search_mode, DialogSearchMode::Direct);
    }

    /// Helper: parse TOML string into Value.
    fn parse_toml(s: &str) -> toml::Value {
        toml::from_str::<toml::Value>(s).expect("valid TOML")
    }

    /// Helper: merge two TOML strings and deserialize into DhxConfig.
    fn merge_and_deserialize(global: &str, workspace: &str) -> DhxConfig {
        let merged =
            helix_loader::merge_toml_values(parse_toml(global), parse_toml(workspace), 3);
        merged.try_into().expect("should deserialize merged config")
    }

    #[test]
    fn merge_global_and_workspace_configs() {
        let global = r#"
[window]
title = "global-title"
width = 1000.0

[font]
size = 14.0
"#;
        let workspace = r#"
[font]
size = 20.0
"#;
        let config = merge_and_deserialize(global, workspace);
        // Workspace overrides font.size
        assert!((config.font.size - 20.0).abs() < f64::EPSILON);
        // Global window.title preserved
        assert_eq!(config.window.title, "global-title");
        // Global window.width preserved
        assert!((config.window.width - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn workspace_only_config() {
        let workspace = r#"
[font]
size = 18.0
ligatures = false
"#;
        let config: DhxConfig = parse_toml(workspace)
            .try_into()
            .expect("should deserialize");
        assert!((config.font.size - 18.0).abs() < f64::EPSILON);
        assert!(!config.font.ligatures);
        // Window should be default
        assert_eq!(config.window.title, "helix-dioxus");
    }

    #[test]
    fn global_only_config() {
        let global = r#"
[window]
title = "my-editor"
width = 800.0
"#;
        let config: DhxConfig = parse_toml(global).try_into().expect("should deserialize");
        assert_eq!(config.window.title, "my-editor");
        assert!((config.window.width - 800.0).abs() < f64::EPSILON);
        // Font should be default
        assert!((config.font.size - 14.0).abs() < f64::EPSILON);
    }

    #[test]
    fn neither_config_returns_default() {
        let config = DhxConfig::default();
        assert_eq!(config.window.title, "helix-dioxus");
        assert!((config.font.size - 14.0).abs() < f64::EPSILON);
    }

    #[test]
    fn malformed_workspace_ignored_global_still_loads() {
        // Simulate: global is valid, workspace fails to parse → only global used
        let global = r#"
[window]
title = "from-global"
"#;
        // If workspace TOML is malformed, read_toml_file returns None,
        // so only global is used. We test the single-value path here.
        let config: DhxConfig = parse_toml(global).try_into().expect("should deserialize");
        assert_eq!(config.window.title, "from-global");
    }

    #[test]
    fn merge_workspace_overrides_nested_sections() {
        let global = r#"
[window]
title = "global"
width = 1200.0
height = 800.0

[font]
family = "'Fira Code'"
size = 14.0
ligatures = true

[logging]
level = "info"
"#;
        let workspace = r#"
[window]
title = "workspace"

[font]
size = 20.0
ligatures = false

[logging]
level = "debug"
"#;
        let config = merge_and_deserialize(global, workspace);
        // Workspace overrides
        assert_eq!(config.window.title, "workspace");
        assert!((config.font.size - 20.0).abs() < f64::EPSILON);
        assert!(!config.font.ligatures);
        assert_eq!(config.logging.level, "debug");
        // Global preserved where workspace doesn't override
        assert!((config.window.width - 1200.0).abs() < f64::EPSILON);
        assert!((config.window.height - 800.0).abs() < f64::EPSILON);
        assert_eq!(config.font.family, "'Fira Code'");
    }

    #[test]
    fn read_toml_file_returns_none_for_missing_file() {
        let result = DhxConfig::read_toml_file(Path::new("/nonexistent/dhx.toml"));
        assert!(result.is_none());
    }

    #[test]
    fn font_css_with_features_overrides_ligatures() {
        let mut config = DhxConfig::default();
        config.font.features = vec!["calt".into(), "liga".into(), "ss01".into()];
        let css = config.font_css();
        assert!(css.contains(r#""calt" 1, "liga" 1, "ss01" 1"#));
        assert!(!css.contains("normal"));
    }

    #[test]
    fn font_css_with_weight() {
        let mut config = DhxConfig::default();
        config.font.weight = Some(400);
        let css = config.font_css();
        assert!(css.contains("--font-weight: 400;"));
    }

    #[test]
    fn font_css_without_weight_omits_var() {
        let config = DhxConfig::default();
        let css = config.font_css();
        assert!(!css.contains("--font-weight"));
    }

    #[test]
    fn deserialize_font_features_and_weight() {
        let toml_str = r#"
[font]
family = "'MonaspiceNe Nerd Font', monospace"
size = 18.0
weight = 400
features = ["calt", "liga", "dlig", "ss01", "ss02"]
"#;
        let config = toml::from_str::<DhxConfig>(toml_str).expect("should deserialize");
        assert_eq!(config.font.weight, Some(400));
        assert_eq!(config.font.features.len(), 5);
        assert_eq!(config.font.features[0], "calt");
    }
}
