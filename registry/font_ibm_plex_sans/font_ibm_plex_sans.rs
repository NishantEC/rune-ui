//! rune-ui registry: font_ibm_plex_sans 0.5.0 (a font). Copied into your app by `rune-ui add font_ibm_plex_sans`; yours to change.
//!
//! IBM Plex Sans by Mike Abbink, Bold Monday, under the SIL Open Font License 1.1 (`OFL.txt` beside this file; keep it with the font). A grotesque with engineered, slightly quirky details, at home in enterprise tools. Variable weight 100 to 700 and width 75 to 100 (drawn at 100). "Plex" is a Reserved Font Name: ship the file unmodified. Its x-height is smaller than Inter's, so the ramp is drawn 5% larger (`size_adjust`).
//! A theme names it as `"font": { "$value": "ibm-plex-sans" }` once the app has installed it at start-up:
//! `AppSpec { fonts: rune_ui_style::fonts::install(ui::FONTS), .. }`. Source and version: this item's `item.toml`.

use rune_ui_style::Font;

/// IBM Plex Sans, 524 KB.
pub const FONT: Font = Font::embedded("ibm-plex-sans", "IBM Plex Sans", "IBM Plex Sans", &[include_bytes!("IBMPlexSans-Variable.ttf")]).size_adjust(1.05);
