//! rune-ui registry: progress 0.4.0. Copied into your app by `rune-ui add progress`; yours to change.
//!
//! How far along: a thin **well** track and an **ink** fill (or a meaning's solid colour), an optional label and figure above it in
//! tabular figures. It reports its value to assistive technology.
//!
//! ```ignore
//! Progress::new("upload", 0.42).label("Uploading").figure("42%").build(t)
//! ```

use rune::{El, col, el, row, text};
use rune_ui_style::{Recipe, tabular};

use super::base::{Shape, Theme, Tone, paint};

pub struct Progress {
    key: String,
    value: f32,
    label: Option<String>,
    figure: Option<String>,
    tone: Option<Tone>,
    width: Option<f32>,
}

impl Progress {
    /// `value` from 0 to 1.
    pub fn new(key: impl Into<String>, value: f32) -> Progress {
        Progress { key: key.into(), value: value.clamp(0.0, 1.0), label: None, figure: None, tone: None, width: None }
    }

    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn figure(mut self, f: impl Into<String>) -> Self {
        self.figure = Some(f.into());
        self
    }

    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let h = 6.0;
        let ink = self.tone.map_or(t.c.primary.bg, |tone| tone.colors(t).2);
        let track = paint(row().key(format!("{}-track", self.key)).w_full().h(h).radius(h / 2.0).round_as(t), &t.mat.well.base).child(paint(el().w_frac(self.value).h(h).radius(h / 2.0).round_as(t), &Recipe::new().fill(ink)).a11y_hidden());
        let name = self.label.clone().unwrap_or_else(|| "Progress".to_owned());
        let mut c = col().key(self.key).role("progressbar").label(name).range_value(f64::from(self.value) * 100.0, 0.0, 100.0).gap(t.space.at(1.5));
        c = match self.width {
            Some(w) => c.w(w),
            None => c.w_full(),
        };
        if self.label.is_some() || self.figure.is_some() {
            let mut line = row().w_full().gap(t.space.at(2.0)).center_y();
            if let Some(l) = self.label {
                line = line.child(text(l).font(t.ty.sm).color(t.ink_muted()).grow(1.0).truncate().min_w(0.0).a11y_hidden());
            }
            if let Some(f) = self.figure {
                line = line.child(text(f).font(tabular(t.ty.sm)).color(t.ink()).a11y_hidden());
            }
            c = c.child(line);
        }
        c.child(track)
    }
}
