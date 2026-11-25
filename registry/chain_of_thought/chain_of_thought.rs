//! rune-ui registry: chain_of_thought 0.6.0. Copied into your app by `rune-ui add chain_of_thought`; yours to change.
//!
//! The steps an agent took, behind one line: "Working…" with a shimmer and the time so far while it works, "Worked for 2m 14s" once it
//! is done. Open, it lists each step with its mark, its words and an optional detail under them. Only the step in progress is in the
//! first ink; the rest are quiet.
//!
//! ```ignore
//! ChainOfThought::new("steps", &steps, 134.0, s.open("steps")).running(false).on_toggle(|s: &mut App| s.flip("steps")).build(t)
//! ```

use std::rc::Rc;

use rune::{El, col, row, text};

use super::agent::{FoldLine, Status, elapsed, mark};
use super::base::{Press, Theme, glyph};
use super::icons;
use super::shimmer::Shimmer;

/// One step.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Step {
    pub label: String,
    pub status: Status,
    pub detail: Option<String>,
}

impl Step {
    pub fn new(label: impl Into<String>, status: Status) -> Step {
        Step { label: label.into(), status, detail: None }
    }

    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }
}

pub struct ChainOfThought<'a, S: 'static> {
    key: String,
    steps: &'a [Step],
    secs: f64,
    open: bool,
    running: bool,
    toggle: Option<Press<S>>,
}

impl<'a, S: 'static> ChainOfThought<'a, S> {
    pub fn new(key: impl Into<String>, steps: &'a [Step], secs: f64, open: bool) -> ChainOfThought<'a, S> {
        ChainOfThought { key: key.into(), steps, secs, open, running: false, toggle: None }
    }

    /// Still working.
    pub fn running(mut self, on: bool) -> Self {
        self.running = on;
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let label = if self.running { "Working…".to_owned() } else { format!("Worked for {}", elapsed(self.secs.round())) };
        let mut line = FoldLine::<S>::new(self.key.clone(), label.clone(), self.open).lead(glyph(&icons::CLOCK, t.control.icon, t.ink_faint()));
        if self.running {
            line = line.words(Shimmer::new(format!("{}-shimmer", self.key), label).build(t)).meta(elapsed(self.secs.round()));
        }
        let n = self.steps.len();
        line = line.meta(format!("{n} step{}", if n == 1 { "" } else { "s" }));
        if let Some(f) = self.toggle {
            line = line.on_toggle(move |s: &mut S| f(s));
        }
        let mut list = col().key(format!("{}-steps", self.key)).role("list").label("Steps").w_full().gap(t.space.at(2.5));
        for (i, s) in self.steps.iter().enumerate() {
            let ink = if s.status == Status::Running { t.ink() } else { t.ink_muted() };
            let mut words = col().gap(t.space.at(0.5)).min_w(0.0).shrink().child(text(s.label.clone()).font(t.ty.base).color(ink).wrap().min_w(0.0));
            if let Some(d) = &s.detail {
                words = words.child(text(d.clone()).font(t.ty.sm).color(t.ink_faint()).wrap().min_w(0.0));
            }
            list = list.child(row().key(format!("{}-step-{i}", self.key)).role("listitem").label(format!("{}: {}", s.status.word(), s.label)).w_full().gap(t.metrics.gap).items_start()
                .child(row().size(t.control.icon, t.ty.base.line_height.max(t.control.icon)).center().child(mark(t, s.status, t.control.icon_sm)))
                .child(words));
        }
        line.build(t, vec![list])
    }
}
