//! rune-ui registry: separator 0.5.0. Copied into your app by `rune-ui add separator`; yours to change.
//!
//! A hairline between groups: across (`horizontal`) or down (`vertical`), optionally with quiet words in the middle ("or").
//! Structure, not depth, so it is the one line colour.
//!
//! ```ignore
//! Separator::horizontal().build(t)
//! Separator::horizontal().label("or").build(t)
//! ```

use rune::{El, el, row, text};

use super::base::Theme;

pub struct Separator {
    vertical: bool,
    label: Option<String>,
    length: Option<f32>,
}

impl Separator {
    pub fn horizontal() -> Separator {
        Separator { vertical: false, label: None, length: None }
    }

    pub fn vertical() -> Separator {
        Separator { vertical: true, label: None, length: None }
    }

    /// Words in the middle of a horizontal separator.
    pub fn label(mut self, w: impl Into<String>) -> Self {
        self.label = Some(w.into());
        self
    }

    /// A fixed length (otherwise it fills what holds it).
    pub fn length(mut self, l: f32) -> Self {
        self.length = Some(l);
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let line = || if self.vertical { el().w(1.0).h_full() } else { el().h(1.0).w_full() }.fill(t.c.line).a11y_hidden();
        let mut e = match &self.label {
            Some(w) if !self.vertical => row().w_full().gap(t.space.at(3.0)).center_y().child(line().grow(1.0)).child(text(w.clone()).font(t.ty.sm).color(t.ink_faint())).child(line().grow(1.0)),
            _ => line(),
        };
        e = e.role("separator");
        match (self.length, self.vertical) {
            (Some(l), true) => e.h(l),
            (Some(l), false) => e.w(l),
            _ => e,
        }
    }
}
