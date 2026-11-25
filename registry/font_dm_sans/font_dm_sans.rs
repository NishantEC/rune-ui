//! rune-ui registry: font_dm_sans 0.5.0 (a font). Copied into your app by `rune-ui add font_dm_sans`; yours to change.
//!
//! DM Sans by Colophon Foundry, under the SIL Open Font License 1.1 (`OFL.txt` beside this file; keep it with the font). A low-contrast geometric sans, soft and even at small sizes. Variable weight 100 to 1000 and optical size 9 to 40 (Rune sets it to the point size). Its x-height is smaller than Inter's, so the ramp is drawn 4% larger (`size_adjust`).
//! A theme names it as `"font": { "$value": "dm-sans" }` once the app has installed it at start-up:
//! `AppSpec { fonts: rune_ui_style::fonts::install(ui::FONTS), .. }`. Source and version: this item's `item.toml`.

use rune_ui_style::Font;

/// DM Sans, 234 KB.
pub const FONT: Font = Font::embedded("dm-sans", "DM Sans", "DM Sans", &[include_bytes!("DMSans-Variable.ttf")]).size_adjust(1.04);
