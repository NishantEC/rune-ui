//! rune-ui registry: label 0.4.0. Copied into your app by `rune-ui add label`; yours to change.
//!
//! A control's visible name: the small medium style in the first ink, an optional required mark, quieter when the control is disabled.
//! Pair it with the control's own `.name(…)` (Rune's accessibility tree takes a control's name from the control).
//!
//! ```ignore
//! Label::new("name-label", "Project name").required().build(t)
//! ```

use rune::{El, row, text};
use rune_ui_style::weight;

use super::base::Theme;

pub struct Label {
    key: String,
    words: String,
    required: bool,
    disabled: bool,
}

impl Label {
    pub fn new(key: impl Into<String>, words: impl Into<String>) -> Label {
        Label { key: key.into(), words: words.into(), required: false, disabled: false }
    }

    /// A mark after the words; assistive technology hears "required" from the control itself.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let ink = if self.disabled { t.ink_faint() } else { t.ink() };
        let mut r = row().key(self.key).gap(t.space.at(0.5)).center_y().child(text(self.words).font(weight(t.ty.sm, t.ty.medium)).color(ink).a11y_hidden());
        if self.required {
            r = r.child(text("*").font(weight(t.ty.sm, t.ty.medium)).color(t.c.danger.fg).a11y_hidden());
        }
        r
    }
}
