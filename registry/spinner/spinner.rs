//! rune-ui registry: spinner 0.5.0. Copied into your app by `rune-ui add spinner`; yours to change.
//!
//! Work in progress: a faint ring with a turning arc, the one thing in the library that loops (Rune stops it under Reduce Motion). Three
//! sizes; the second ink by default, or a meaning's colour. It says what is loading.
//!
//! ```ignore
//! Spinner::new("Loading results").build(t)
//! ```

use rune::El;
use rune_ui_style::Size;

use super::base::{Theme, Tone};

pub struct Spinner {
    label: String,
    size: Size,
    tone: Option<Tone>,
}

impl Spinner {
    pub fn new(label: impl Into<String>) -> Spinner {
        Spinner { label: label.into(), size: Size::Md, tone: None }
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let d = match self.size {
            Size::Sm => 12.0,
            Size::Lg => 24.0,
            _ => 16.0,
        };
        let color = self.tone.map_or(t.ink_muted(), |tone| tone.colors(t).2);
        super::base::spinner(d, color, &self.label)
    }
}
