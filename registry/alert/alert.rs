//! rune-ui registry: alert 0.5.0. Copied into your app by `rune-ui add alert`; yours to change.
//!
//! A message that belongs to the page: a meaning's wash at low alpha, its icon, a title and a description in readable inks, and an
//! optional action at the right. Never a border; the wash and the words carry it. Danger and warning alerts announce themselves.
//!
//! ```ignore
//! Alert::new("quota", Tone::Warning, "Almost out of credits").description("Work pauses at the limit.").action(button).build(t)
//! ```

use rune::{El, col, row, text};
use rune_ui_style::{Recipe, weight};

use super::base::{Shape, Theme, Tone, glyph, paint};
use super::icons;

pub struct Alert {
    key: String,
    tone: Tone,
    title: String,
    description: Option<String>,
    action: Option<El>,
}

impl Alert {
    pub fn new(key: impl Into<String>, tone: Tone, title: impl Into<String>) -> Alert {
        Alert { key: key.into(), tone, title: title.into(), description: None, action: None }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn action(mut self, a: El) -> Self {
        self.action = Some(a);
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let (wash, words, solid, _) = self.tone.colors(t);
        let icon = match self.tone {
            Tone::Success => &icons::CIRCLE_CHECK,
            Tone::Warning => &icons::WARNING,
            Tone::Danger => &icons::CIRCLE_X,
            _ => &icons::INFO,
        };
        let neutral = self.tone == Tone::Neutral;
        let mut text_col = col().grow(1.0).shrink().min_w(0.0).gap(t.space.at(0.5)).child(text(self.title.clone()).font(weight(t.ty.base, t.ty.medium)).color(if neutral { t.ink() } else { words }).wrap().min_w(0.0));
        if let Some(d) = &self.description {
            text_col = text_col.child(text(d.clone()).font(t.ty.base).color(if neutral { t.ink_muted() } else { words }).wrap().min_w(0.0));
        }
        let mut a = paint(row().key(self.key).role("alert").label(self.title).w_full().padding(t.metrics.pad_md).gap(t.metrics.pad_md - 4.0).items_start().radius(t.radius.lg).round_as(t), &Recipe::new().fill(wash)).child(glyph(icon, t.control.icon_lg, if neutral { t.ink_muted() } else { solid }).mt(1.0)).child(text_col);
        if matches!(self.tone, Tone::Danger | Tone::Warning) {
            a = a.live_region(rune::LiveRegion::Polite);
        }
        if let Some(act) = self.action {
            a = a.child(act);
        }
        a
    }
}
