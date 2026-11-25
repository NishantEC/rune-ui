//! rune-ui registry: font_manrope 0.5.0 (a font). Copied into your app by `rune-ui add font_manrope`; yours to change.
//!
//! Manrope by Mikhail Sharanda, under the SIL Open Font License 1.1 (`OFL.txt` beside this file; keep it with the font). A semi-condensed geometric sans with open, modern shapes. Variable weight 200 to 800.
//! A theme names it as `"font": { "$value": "manrope" }` once the app has installed it at start-up:
//! `AppSpec { fonts: rune_ui_style::fonts::install(ui::FONTS), .. }`. Source and version: this item's `item.toml`.

use rune_ui_style::Font;

/// Manrope, 160 KB.
pub const FONT: Font = Font::embedded("manrope", "Manrope", "Manrope", &[include_bytes!("Manrope-Variable.ttf")]);
