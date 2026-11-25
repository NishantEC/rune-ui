//! rune-ui registry: font_jetbrains_mono 0.5.0 (a font). Copied into your app by `rune-ui add font_jetbrains_mono`; yours to change.
//!
//! JetBrains Mono by JetBrains, Philipp Nurullin, Konstantin Bulenkov, under the SIL Open Font License 1.1 (`OFL.txt` beside this file; keep it with the font). A monospace with a tall x-height and clear shapes, drawn for reading code. Variable weight 100 to 800.
//! A theme names it as `"font": { "$value": "jetbrains-mono" }` once the app has installed it at start-up:
//! `AppSpec { fonts: rune_ui_style::fonts::install(ui::FONTS), .. }`. Source and version: this item's `item.toml`.

use rune_ui_style::Font;

/// JetBrains Mono, 182 KB.
pub const FONT: Font = Font::embedded("jetbrains-mono", "JetBrains Mono", "JetBrains Mono", &[include_bytes!("JetBrainsMono-Variable.ttf")]).mono();
