//! rune-ui registry: font_geist_mono 0.5.0 (a font). Copied into your app by `rune-ui add font_geist_mono`; yours to change.
//!
//! Geist Mono by Andrés Briganti, Mateo Zaragoza, Guillermo Rauch, Evil Rabbit, José Rago, Facundo Santana, under the SIL Open Font License 1.1 (`OFL.txt` beside this file; keep it with the font). Geist's monospaced companion, for code, keys and tables of figures. Variable weight 100 to 900. Its x-height is smaller than Inter's, so the ramp is drawn 3% larger (`size_adjust`).
//! A theme names it as `"font": { "$value": "geist-mono" }` once the app has installed it at start-up:
//! `AppSpec { fonts: rune_ui_style::fonts::install(ui::FONTS), .. }`. Source and version: this item's `item.toml`.

use rune_ui_style::Font;

/// Geist Mono, 167 KB.
pub const FONT: Font = Font::embedded("geist-mono", "Geist Mono", "Geist Mono", &[include_bytes!("GeistMono-Variable.ttf")]).mono().size_adjust(1.03);
