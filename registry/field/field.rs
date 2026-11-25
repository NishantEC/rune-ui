//! rune-ui registry: field 0.2.0. Copied into your app by `rune-ui add field`; yours to change. Needs: label.
//!
//! A label, a control and its description laid out together; an error replaces the description, in the danger colour and announced
//! politely. Give the control the same words with `.name(…)` and `.description(…)`/`.invalid(true)` so assistive technology hears them.
//!
//! ```ignore
//! Field::new("budget", "Budget").description("Stops work that spends more").child(input).build(t)
//! ```

use rune::{El, col, text};

use super::base::Theme;
use super::label::Label;

pub struct Field {
    key: String,
    label: String,
    description: Option<String>,
    error: Option<String>,
    required: bool,
    disabled: bool,
    width: Option<f32>,
    control: Option<El>,
}

impl Field {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Field {
        Field { key: key.into(), label: label.into(), description: None, error: None, required: false, disabled: false, width: None, control: None }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// What is wrong, in place of the description.
    pub fn error(mut self, e: Option<String>) -> Self {
        self.error = e;
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn child(mut self, control: El) -> Self {
        self.control = Some(control);
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let mut label = Label::new(format!("{}-label", self.key), self.label).disabled(self.disabled);
        if self.required {
            label = label.required();
        }
        let mut c = col().key(self.key.clone()).gap(t.space.at(1.5)).child(label.build(t));
        c = match self.width {
            Some(w) => c.w(w),
            None => c.w_full(),
        };
        if let Some(control) = self.control {
            c = c.child(control);
        }
        if let Some(e) = self.error {
            c = c.child(text(e).key(format!("{}-error", self.key)).font(t.ty.sm).color(t.c.danger.fg).wrap().min_w(0.0).live_region(rune::LiveRegion::Polite));
        } else if let Some(d) = self.description {
            c = c.child(text(d).key(format!("{}-description", self.key)).font(t.ty.sm).color(t.ink_muted()).wrap().min_w(0.0));
        }
        c
    }
}
