//! rune-ui registry: font_plus_jakarta_sans 0.5.0 (a font). Copied into your app by `rune-ui add font_plus_jakarta_sans`; yours to change.
//!
//! Plus Jakarta Sans by Tokotype, under the SIL Open Font License 1.1 (`OFL.txt` beside this file; keep it with the font). A wide, modern sans with tall capitals and a confident, product-site feel. Variable weight 200 to 800. Its x-height is smaller than Inter's, so the ramp is drawn 2% larger (`size_adjust`).
//! A theme names it as `"font": { "$value": "plus-jakarta-sans" }` once the app has installed it at start-up:
//! `AppSpec { fonts: rune_ui_style::fonts::install(ui::FONTS), .. }`. Source and version: this item's `item.toml`.

use rune_ui_style::Font;

/// Plus Jakarta Sans, 172 KB.
pub const FONT: Font = Font::embedded("plus-jakarta-sans", "Plus Jakarta Sans", "Plus Jakarta Sans", &[include_bytes!("PlusJakartaSans-Variable.ttf")]).size_adjust(1.02);
