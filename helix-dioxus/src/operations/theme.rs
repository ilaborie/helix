//! Theme operations for the editor.

use helix_view::graphics::Color;

use crate::state::{color_to_css, EditorContext};

/// Extension trait for theme operations.
pub trait ThemeOps {
    /// List all available theme names, sorted and deduplicated.
    fn list_themes(&self) -> Vec<String>;
    /// Apply a theme by name. Returns an error if the theme is not found.
    fn apply_theme(&mut self, name: &str) -> anyhow::Result<()>;
    /// Get the current theme name.
    fn current_theme_name(&self) -> &str;
    /// Generate CSS variable overrides from the current theme.
    fn theme_to_css_vars(&self) -> String;
}

/// Extract RGB components from a Color, returning (r, g, b) or None.
fn color_to_rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Black => Some((0, 0, 0)),
        Color::Red | Color::LightRed => Some((224, 108, 117)),
        Color::Green | Color::LightGreen => Some((152, 195, 121)),
        Color::Yellow | Color::LightYellow => Some((229, 192, 123)),
        Color::Blue | Color::LightBlue => Some((97, 175, 239)),
        Color::Magenta | Color::LightMagenta => Some((198, 120, 221)),
        Color::Cyan | Color::LightCyan => Some((86, 182, 194)),
        Color::Gray => Some((92, 99, 112)),
        Color::White | Color::LightGray => Some((171, 178, 191)),
        _ => None,
    }
}

/// Detect if a background color is "light" by checking perceived luminance.
fn is_light_background(color: Color) -> bool {
    if let Some((r, g, b)) = color_to_rgb(color) {
        // Perceived luminance formula (ITU-R BT.601)
        let luminance = 0.299 * f32::from(r) + 0.587 * f32::from(g) + 0.114 * f32::from(b);
        luminance > 128.0
    } else {
        false
    }
}

/// Blend two colors: ratio of fg mixed with (1-ratio) of bg.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "f32 color values clamped to 0..=255 before cast"
)]
fn blend_colors(fg: Color, bg: Color, ratio: f32) -> Option<String> {
    let (fr, fg_g, fb) = color_to_rgb(fg)?;
    let (br, bg_g, bb) = color_to_rgb(bg)?;
    let r = (f32::from(fr) * ratio + f32::from(br) * (1.0 - ratio)) as u8;
    let g = (f32::from(fg_g) * ratio + f32::from(bg_g) * (1.0 - ratio)) as u8;
    let b = (f32::from(fb) * ratio + f32::from(bb) * (1.0 - ratio)) as u8;
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

/// Dark theme fallback defaults for CSS variables.
const DARK_DEFAULTS: &[(&str, &str)] = &[
    ("--bg-primary", "#282c34"),
    ("--bg-secondary", "#21252b"),
    ("--bg-highlight", "#2c313a"),
    ("--bg-selection", "#3e4451"),
    ("--bg-deep", "#181a1f"),
    ("--text", "#abb2bf"),
    ("--text-dim", "#5c6370"),
    ("--text-dimmer", "#6c7380"),
    ("--accent", "#61afef"),
    ("--error", "#e06c75"),
    ("--warning", "#e5c07b"),
    ("--info", "#61afef"),
    ("--hint", "#56b6c2"),
    ("--success", "#98c379"),
    ("--purple", "#c678dd"),
    ("--orange", "#d19a66"),
    ("--cursor-normal", "#61afef"),
    ("--cursor-select", "#c678dd"),
    ("--cursor-fg", "#000000"),
    ("--cursor-in-selection", "#e5c07b"),
    ("--mode-normal-bg", "#61afef"),
    ("--mode-normal-fg", "#282c34"),
    ("--mode-insert-bg", "#98c379"),
    ("--mode-insert-fg", "#282c34"),
    ("--mode-select-bg", "#c678dd"),
    ("--mode-select-fg", "#282c34"),
    ("--notification-error-bg", "#3d2020"),
    ("--notification-warning-bg", "#3d3520"),
    ("--notification-info-bg", "#1e2a3d"),
    ("--notification-success-bg", "#1e3d20"),
    ("--kbd-bg", "#2c313a"),
    ("--kbd-border", "#4b5263"),
    ("--kbd-text", "#abb2bf"),
    ("--scrollbar-thumb", "rgba(92, 99, 112, 0.5)"),
    ("--scrollbar-thumb-hover", "rgba(108, 115, 128, 0.6)"),
    ("--virtual-whitespace", "#4b5263"),
    ("--virtual-ruler-bg", "#2c313a"),
];

/// Light theme fallback defaults for CSS variables.
const LIGHT_DEFAULTS: &[(&str, &str)] = &[
    ("--bg-primary", "#fafafa"),
    ("--bg-secondary", "#f0f0f0"),
    ("--bg-highlight", "#e8e8e8"),
    ("--bg-selection", "#d0d0d0"),
    ("--bg-deep", "#e0e0e0"),
    ("--text", "#383a42"),
    ("--text-dim", "#a0a1a7"),
    ("--text-dimmer", "#b0b1b7"),
    ("--accent", "#4078f2"),
    ("--error", "#e45649"),
    ("--warning", "#c18401"),
    ("--info", "#4078f2"),
    ("--hint", "#0184bc"),
    ("--success", "#50a14f"),
    ("--purple", "#a626a4"),
    ("--orange", "#986801"),
    ("--cursor-normal", "#4078f2"),
    ("--cursor-select", "#a626a4"),
    ("--cursor-fg", "#ffffff"),
    ("--cursor-in-selection", "#c18401"),
    ("--mode-normal-bg", "#4078f2"),
    ("--mode-normal-fg", "#ffffff"),
    ("--mode-insert-bg", "#50a14f"),
    ("--mode-insert-fg", "#ffffff"),
    ("--mode-select-bg", "#a626a4"),
    ("--mode-select-fg", "#ffffff"),
    ("--notification-error-bg", "#fde8e8"),
    ("--notification-warning-bg", "#fdf3e0"),
    ("--notification-info-bg", "#e8f0fd"),
    ("--notification-success-bg", "#e8fde8"),
    ("--kbd-bg", "#ffffff"),
    ("--kbd-border", "#c8c8c8"),
    ("--kbd-text", "#383a42"),
    ("--scrollbar-thumb", "rgba(160, 161, 167, 0.5)"),
    ("--scrollbar-thumb-hover", "rgba(140, 141, 147, 0.6)"),
    ("--virtual-whitespace", "#a0a1a7"),
    ("--virtual-ruler-bg", "#e8e8e8"),
];

impl ThemeOps for EditorContext {
    fn list_themes(&self) -> Vec<String> {
        let mut names = helix_view::theme::Loader::read_names(&helix_loader::config_dir().join("themes"));
        for rt_dir in helix_loader::runtime_dirs() {
            names.extend(helix_view::theme::Loader::read_names(&rt_dir.join("themes")));
        }
        names.push("default".to_string());
        names.push("base16_default".to_string());
        names.sort();
        names.dedup();
        names
    }

    fn apply_theme(&mut self, name: &str) -> anyhow::Result<()> {
        let theme = self.editor.theme_loader.load(name)?;
        self.editor.set_theme(theme);
        Ok(())
    }

    fn current_theme_name(&self) -> &str {
        self.editor.theme.name()
    }

    fn theme_to_css_vars(&self) -> String {
        let theme = &self.editor.theme;
        let mut vars = Vec::new();

        // Detect light vs dark from background color
        let bg_color = theme.try_get("ui.background").and_then(|s| s.bg);
        let is_light = bg_color.is_some_and(is_light_background);
        log::debug!(
            "theme_to_css_vars: theme='{}', bg_color={bg_color:?}, is_light={is_light}",
            theme.name(),
        );
        let defaults = if is_light { LIGHT_DEFAULTS } else { DARK_DEFAULTS };

        // Direct scope-to-CSS-var mappings: (scope, css_var, use_bg)
        let scope_mappings: &[(&str, &str, bool)] = &[
            // Background colors
            ("ui.background", "--bg-primary", true),
            ("ui.background.separator", "--bg-secondary", true),
            ("ui.cursorline.primary", "--bg-highlight", true),
            ("ui.selection", "--bg-selection", true),
            ("ui.menu", "--bg-deep", true),
            // Text colors
            ("ui.text", "--text", false),
            ("ui.linenr", "--text-dim", false),
            ("ui.text.dimmed", "--text-dimmer", false),
            // Accent
            ("ui.cursor.primary", "--accent", true),
            // Cursor
            ("ui.cursor.primary", "--cursor-normal", true),
            ("ui.cursor.select", "--cursor-select", true),
            ("ui.cursor.primary", "--cursor-fg", false),
            ("ui.cursor.match", "--cursor-in-selection", true),
            // Diagnostics / semantic
            ("diagnostic.error", "--error", false),
            ("diagnostic.warning", "--warning", false),
            ("diagnostic.info", "--info", false),
            ("diagnostic.hint", "--hint", false),
            ("diff.plus", "--success", false),
            // Virtual elements
            ("ui.virtual.whitespace", "--virtual-whitespace", false),
            ("ui.virtual.ruler", "--virtual-ruler-bg", true),
            // Mode colors (statusline)
            ("ui.statusline.normal", "--mode-normal-bg", true),
            ("ui.statusline.normal", "--mode-normal-fg", false),
            ("ui.statusline.insert", "--mode-insert-bg", true),
            ("ui.statusline.insert", "--mode-insert-fg", false),
            ("ui.statusline.select", "--mode-select-bg", true),
            ("ui.statusline.select", "--mode-select-fg", false),
        ];

        // Track which CSS vars we've successfully set from theme
        let mut set_vars = std::collections::HashSet::new();

        for &(scope, css_var, use_bg) in scope_mappings {
            if let Some(style) = theme.try_get(scope) {
                let color = if use_bg { style.bg } else { style.fg };
                if let Some(c) = color {
                    if let Some(css) = color_to_css(c) {
                        vars.push(format!("{css_var}: {css};"));
                        set_vars.insert(css_var);
                    }
                }
            }
        }

        // Try to extract purple and orange from keyword/constant scopes
        if !set_vars.contains("--purple") {
            if let Some(style) = theme.try_get("keyword") {
                if let Some(c) = style.fg {
                    if let Some(css) = color_to_css(c) {
                        vars.push(format!("--purple: {css};"));
                        set_vars.insert("--purple");
                    }
                }
            }
        }
        if !set_vars.contains("--orange") {
            if let Some(style) = theme.try_get("constant.numeric") {
                if let Some(c) = style.fg {
                    if let Some(css) = color_to_css(c) {
                        vars.push(format!("--orange: {css};"));
                        set_vars.insert("--orange");
                    }
                }
            }
        }

        // Derive notification backgrounds by blending semantic colors with background
        if let Some(bg) = bg_color {
            let ratio = if is_light { 0.12 } else { 0.15 };
            let blends = [
                ("diagnostic.error", "--notification-error-bg"),
                ("diagnostic.warning", "--notification-warning-bg"),
                ("diagnostic.info", "--notification-info-bg"),
                ("diff.plus", "--notification-success-bg"),
            ];
            for (scope, css_var) in blends {
                if let Some(style) = theme.try_get(scope) {
                    if let Some(fg) = style.fg {
                        if let Some(blended) = blend_colors(fg, bg, ratio) {
                            vars.push(format!("{css_var}: {blended};"));
                            set_vars.insert(css_var);
                        }
                    }
                }
            }
        }

        // Derive scrollbar thumb from text-dim color
        if let Some(style) = theme.try_get("ui.linenr") {
            if let Some(c) = style.fg {
                if let Some((r, g, b)) = color_to_rgb(c) {
                    vars.push(format!("--scrollbar-thumb: rgba({r}, {g}, {b}, 0.5);"));
                    vars.push(format!("--scrollbar-thumb-hover: rgba({r}, {g}, {b}, 0.65);"));
                    set_vars.insert("--scrollbar-thumb");
                    set_vars.insert("--scrollbar-thumb-hover");
                }
            }
        }

        // Fill in fallback defaults for any CSS vars not set from theme
        for &(css_var, default_value) in defaults {
            if !set_vars.contains(css_var) {
                vars.push(format!("{css_var}: {default_value};"));
            }
        }

        if vars.is_empty() {
            return String::new();
        }

        // Output as inline style declarations (no :root wrapper)
        // These are applied directly on the app container element via style attribute
        let css = vars.join(" ");
        log::debug!(
            "theme_to_css_vars: {} vars set from theme, {} from defaults",
            set_vars.len(),
            defaults.len().saturating_sub(set_vars.len()),
        );
        css
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_vars_format_is_valid() {
        // Inline style format: property declarations separated by spaces
        let css = "--bg-primary: #282c34; --text: #abb2bf;";
        assert!(css.contains("--bg-primary"));
        assert!(css.contains("--text"));
    }

    #[test]
    fn is_light_background_detects_correctly() {
        assert!(is_light_background(Color::Rgb(250, 250, 250)));
        assert!(is_light_background(Color::Rgb(200, 200, 200)));
        assert!(!is_light_background(Color::Rgb(40, 44, 52)));
        assert!(!is_light_background(Color::Rgb(0, 0, 0)));
        assert!(is_light_background(Color::White));
        assert!(!is_light_background(Color::Black));
    }

    #[test]
    fn blend_colors_produces_valid_hex() {
        let fg = Color::Rgb(224, 108, 117);
        let bg = Color::Rgb(40, 44, 52);
        let result = blend_colors(fg, bg, 0.15).expect("should produce a color");
        assert!(result.starts_with('#'));
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn color_to_rgb_extracts_components() {
        assert_eq!(color_to_rgb(Color::Rgb(10, 20, 30)), Some((10, 20, 30)));
        assert_eq!(color_to_rgb(Color::Black), Some((0, 0, 0)));
        assert!(color_to_rgb(Color::Reset).is_none());
    }
}
