//! GUI-specific configuration for helix-dioxus.
//!
//! Configuration is loaded from `~/.config/helix/dhx.toml` and provides
//! window, font, and logging settings. Editor settings (keybindings, theme,
//! LSP) are loaded from the standard helix `config.toml` and `languages.toml`.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

/// GUI-specific configuration loaded from `dhx.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
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
    pub ligatures: bool,
}

/// Logging configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub log_file: Option<PathBuf>,
    pub level: String,
    pub suppressed_patterns: Vec<String>,
}

impl Default for DhxConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            font: FontConfig::default(),
            logging: LoggingConfig::default(),
            dialog: DialogConfig::default(),
        }
    }
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
            ligatures: true,
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
    /// Load configuration from the default location (`~/.config/helix/dhx.toml`).
    ///
    /// Falls back to defaults if the file doesn't exist.
    /// Returns an error only if the file exists but is malformed.
    pub fn load_default() -> Result<Self> {
        let config_path = helix_loader::config_dir().join("dhx.toml");
        if config_path.exists() {
            Self::load_from(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific file path.
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str::<DhxConfig>(&content)?;
        Ok(config)
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
    /// Returns a `<style>` block that overrides the CSS `:root` defaults.
    #[must_use]
    pub fn font_css(&self) -> String {
        let ligatures = if self.font.ligatures {
            "normal"
        } else {
            "none"
        };
        format!(
            "<style>:root {{ --font-mono: {}; --font-size: {}px; --font-ligatures: {}; }}</style>",
            self.font.family, self.font.size, ligatures
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
        assert!(css.contains("<style>"));
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
}
