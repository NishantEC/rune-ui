//! Typefaces a theme can name (`base.font`): four built in, and any an app adds from a `font` registry item.
//!
//! **What Rune can do at this rev.** An app hands Rune font files as bytes when it starts (`AppSpec::fonts`, TrueType or OpenType, not
//! WOFF); Rune registers them and finds them by family name (`Family::Named`). There is no call to add a font to a running app, and a
//! non-empty list *replaces* Rune's built-in Inter, so every list must carry Inter too (each font item needs `font_inter`). Rune also
//! loads fonts installed on the Mac by family name on first use, which is how `system` (SF) and `serif` (New York, Georgia) work.
//! Variable fonts work: Rune sets the `wght` axis to the weight asked for, and `opsz` to the point size.
//!
//! **Using it.** `rune-ui add font_geist` copies the font, its licence and a `Font` constant into `src/ui/font_geist/`, and lists it in
//! `ui::FONTS`. At start-up:
//!
//! ```ignore
//! let spec = AppSpec { fonts: rune_ui_style::fonts::install(ui::FONTS), ..AppSpec::new(/* ... */) };
//! ```
//!
//! `install` makes the names known to theme files (`"font": { "$value": "geist" }`) and returns every installed file for Rune. A theme
//! naming a font that is neither built in nor installed fails to load, with the names it could have used.
//!
//! **Font smoothing** is Rune's (`App::font_smoothing`, `Tx::set_font_smoothing`): `On` by default on macOS (Core Graphics' dilation),
//! `Off` draws as `-webkit-font-smoothing: antialiased` does. It is a window setting, not a token, so a theme does not carry it.

use std::sync::RwLock;

use rune::Family;

/// A typeface: built in, or registered from an app's font item.
#[derive(Clone, Copy)]
pub struct Font {
    /// The theme file's word: `"geist"`, `"ibm-plex-sans"`.
    pub name: &'static str,
    /// What an editor shows: `"Geist"`.
    pub title: &'static str,
    /// The family Rune draws in: a registered or installed family name, or one of Rune's slots.
    pub family: Family,
    /// Monospaced (every glyph one width).
    pub mono: bool,
    /// The files to register with Rune when the app starts; none for a built-in.
    pub files: &'static [&'static [u8]],
    /// Multiplies every type size, so a face with a small x-height reads at the size the ramp was tuned for in Inter (CSS's
    /// `size-adjust`). 1 for most faces.
    pub size_adjust: f32,
    /// Multiplies every line height. 1 for most faces: the ramp's lines already leave room for tall ascenders.
    pub line_height: f32,
}

impl Font {
    /// Inter: Rune's built-in sans (the `Sans` slot). The `font_inter` item ships the same files for apps that register fonts.
    pub const INTER: Font = Font::built_in("inter", "Inter", Family::Sans, false);
    /// The platform's UI font (SF on macOS), Inter where it is missing.
    pub const SYSTEM: Font = Font::built_in("system", "System", Family::Named("System Font"), false);
    /// A serif: New York, then Georgia, Inter where neither is installed.
    pub const SERIF: Font = Font::built_in("serif", "Serif", Family::Named("New York, Georgia"), false);
    /// The platform's monospaced font (SF Mono on macOS).
    pub const MONO: Font = Font::built_in("mono", "Mono", Family::SystemMono, true);
    /// The four every theme can name.
    pub const BUILT_IN: [Font; 4] = [Font::INTER, Font::SYSTEM, Font::SERIF, Font::MONO];

    const fn built_in(name: &'static str, title: &'static str, family: Family, mono: bool) -> Font {
        Font { name, title, family, mono, files: &[], size_adjust: 1.0, line_height: 1.0 }
    }

    /// A font from files the app embeds: `family` is the name in the files (their `name` table's typographic family).
    pub const fn embedded(name: &'static str, title: &'static str, family: &'static str, files: &'static [&'static [u8]]) -> Font {
        Font { name, title, family: Family::Named(family), mono: false, files, size_adjust: 1.0, line_height: 1.0 }
    }

    /// Marks the font monospaced.
    pub const fn mono(mut self) -> Font {
        self.mono = true;
        self
    }

    /// Sets [`Font::size_adjust`].
    pub const fn size_adjust(mut self, k: f32) -> Font {
        self.size_adjust = k;
        self
    }

    /// Sets [`Font::line_height`].
    pub const fn line_height(mut self, k: f32) -> Font {
        self.line_height = k;
        self
    }
}

impl Default for Font {
    fn default() -> Font {
        Font::INTER
    }
}

/// Fonts are one by their theme word.
impl PartialEq for Font {
    fn eq(&self, other: &Font) -> bool {
        self.name == other.name
    }
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes: usize = self.files.iter().map(|b| b.len()).sum();
        f.debug_struct("Font").field("name", &self.name).field("family", &self.family).field("mono", &self.mono).field("files", &self.files.len()).field("bytes", &bytes).field("size_adjust", &self.size_adjust).field("line_height", &self.line_height).finish()
    }
}

static INSTALLED: RwLock<Vec<Font>> = RwLock::new(Vec::new());

/// Makes `fonts` known to theme files (a font with a built-in's name replaces it) and returns every installed font's files, for
/// `AppSpec::fonts`. Call it once, before the first window and before loading a theme that names one of them. Rune drops its own Inter
/// when this list is not empty: install `font_inter` with the others (each font item needs it).
pub fn install(fonts: &[Font]) -> &'static [&'static [u8]] {
    let mut all = INSTALLED.write().unwrap_or_else(|e| e.into_inner());
    for f in fonts {
        match all.iter_mut().find(|x| x.name == f.name) {
            Some(x) => *x = *f,
            None => all.push(*f),
        }
    }
    let files: Vec<&'static [u8]> = all.iter().flat_map(|f| f.files.iter().copied()).collect();
    if files.is_empty() { &[] } else { Box::leak(files.into_boxed_slice()) }
}

/// Every font a theme can name: the built-ins (or the installed fonts that replace them), then the installed ones, in install order.
pub fn all() -> Vec<Font> {
    let installed = INSTALLED.read().unwrap_or_else(|e| e.into_inner());
    let mut out: Vec<Font> = Font::BUILT_IN.iter().map(|b| installed.iter().find(|f| f.name == b.name).copied().unwrap_or(*b)).collect();
    out.extend(installed.iter().filter(|f| !Font::BUILT_IN.iter().any(|b| b.name == f.name)).copied());
    out
}

/// The font a theme word names.
pub fn find(name: &str) -> Option<Font> {
    all().into_iter().find(|f| f.name == name)
}

/// Every theme word a font answers to.
pub fn names() -> Vec<&'static str> {
    all().iter().map(|f| f.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE: Font = Font::embedded("test-face", "Test Face", "Test Face", &[b"not a real font"]).size_adjust(1.05);

    #[test]
    fn built_ins_are_always_known_and_installing_adds_and_replaces_by_name() {
        for n in ["inter", "system", "serif", "mono"] {
            assert!(find(n).is_some(), "{n}");
        }
        let files = install(&[FAKE]);
        assert!(files.contains(&&b"not a real font"[..]));
        assert_eq!(find("test-face").map(|f| f.size_adjust), Some(1.05));
        assert!(find("no-such-face").is_none());
        let again = install(&[FAKE]);
        assert_eq!(names().iter().filter(|n| **n == "test-face").count(), 1, "installed once");
        assert_eq!(again.len(), files.len());
    }
}
