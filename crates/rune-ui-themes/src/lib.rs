//! # rune-ui-themes
//!
//! Layer 4 of rune-ui: themes as data. A theme is one JSON file with a `light` and a `dark` mode, checked against
//! [`SCHEMA`] (`theme.schema.json`); `rune_ui_style::Theme::from_json` parses it strictly, so an unknown key is an error.
//!
//! One theme ships: [`DEFAULT`], an almost colourless page with raised controls lit from above and one ink object per view. Its dark
//! mode is graphite, not navy, and its materials are retuned rather than inverted: a narrower raised gradient, a much dimmer lit edge,
//! deeper inner and drop shadows, a near-white ink with a light rim, a light floating ring.
//!
//! To make a theme: copy `themes/default.json`, change values, and run this crate's tests (contrast, light/dark parity, schema).

pub use rune_ui_style::{Theme, ThemeError, ThemeFile};

/// The default theme file.
pub const DEFAULT: &str = include_str!("../themes/default.json");

/// The JSON Schema every theme file follows.
pub const SCHEMA: &str = include_str!("../theme.schema.json");

/// Every shipped theme file, by name.
pub const ALL: [(&str, &str); 1] = [("default", DEFAULT)];

/// The default theme in an appearance.
pub fn theme(dark: bool) -> Theme {
    Theme::from_json(DEFAULT, dark).expect("the default theme is valid (tested)")
}

/// A shipped theme by name, in an appearance.
pub fn by_name(name: &str, dark: bool) -> Option<Theme> {
    ALL.iter().find(|(n, _)| *n == name).and_then(|(_, json)| Theme::from_json(json, dark).ok())
}

/// A theme file from disk, for hot reload while developing: call it again when the file changes.
pub fn load(path: &std::path::Path, dark: bool) -> Result<Theme, ThemeError> {
    let json = std::fs::read_to_string(path).map_err(|e| ThemeError(vec![format!("{}: {e}", path.display())]))?;
    Theme::from_json(&json, dark)
}
