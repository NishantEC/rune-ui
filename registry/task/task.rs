//! rune-ui registry: task 0.8.0 Copied into your app by `rune-ui add task`; yours to change.
//!
//! A run card: the one thing on an agent screen that should stand out once work is finished. A slab with the run's title and a status
//! pill (its meaning as a wash, a dot and words), then quiet facts in tabular figures (files changed with `+n −n`, checks passed and
//! failed, cost, time) and one primary action in ink.
//!
//! ```ignore
//! Task::new("run", &run).action("Review changes", |s: &mut App| s.review()).build(t)
//! ```

use std::rc::Rc;

use rune::{El, col, row, text};
use rune_ui_style::{tabular, weight};

use super::agent::{Status, elapsed, money};
use super::badge::Badge;
use super::base::{Press, Shape, Theme, glyph, paint};
use super::button::Button;
use super::icons;

/// What the app knows about a run.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Run {
    pub title: String,
    pub status: Status,
    pub files: usize,
    pub added: usize,
    pub removed: usize,
    pub passed: usize,
    pub failed: usize,
    /// Dollars.
    pub cost: f64,
    pub secs: f64,
}

pub struct Task<'a, S: 'static> {
    key: String,
    run: &'a Run,
    action: Option<(String, Press<S>)>,
}

impl<'a, S: 'static> Task<'a, S> {
    pub fn new(key: impl Into<String>, run: &'a Run) -> Task<'a, S> {
        Task { key: key.into(), run, action: None }
    }

    /// The one primary action.
    pub fn action(mut self, words: impl Into<String>, f: impl Fn(&mut S) + 'static) -> Self {
        self.action = Some((words.into(), Rc::new(f)));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let r = self.run;
        let fact = |icon, words: El| row().gap(t.space.at(1.5)).center_y().child(glyph(icon, t.control.icon_sm, t.ink_faint())).child(words);
        let figures = |s: String| text(s).font(tabular(t.ty.sm)).color(t.ink_muted());
        let changes = row().gap(t.space.at(1.5)).child(figures(format!("{} file{}", r.files, if r.files == 1 { "" } else { "s" })))
            .child(text(format!("+{}", r.added)).font(tabular(t.ty.sm)).color(t.c.success.fg))
            .child(text(format!("\u{2212}{}", r.removed)).font(tabular(t.ty.sm)).color(t.c.danger.fg));
        let checks = row().gap(t.space.at(1.5)).child(figures(format!("{} passed", r.passed))).children((r.failed > 0).then(|| text(format!("{} failed", r.failed)).font(tabular(t.ty.sm)).color(t.c.danger.fg)));
        let facts = row().key(format!("{}-facts", self.key)).w_full().gap(t.space.at(5.0)).flex_wrap()
            .child(fact(&icons::FILE, changes))
            .child(fact(&icons::CIRCLE_CHECK, checks))
            .child(fact(&icons::TAG, figures(money(r.cost))))
            .child(fact(&icons::CLOCK, figures(elapsed(r.secs))));
        let head = row().w_full().gap(t.space.at(3.0)).items_start()
            .child(text(r.title.clone()).font(weight(t.ty.md, t.ty.medium)).color(t.ink()).wrap().min_w(0.0).grow(1.0).shrink())
            .child(Badge::new(format!("{}-status", self.key), r.status.word()).tone(r.status.tone()).dot().build(t));
        let mut card = paint(col().key(self.key.clone()).role("group").label(format!("{}: {}", r.title, r.status.word())).w_full().padding(t.metrics.inset).gap(t.space.at(4.0)).radius(t.radius.xl).round_as(t), &t.mat.slab.base).child(head).child(facts);
        if let Some((words, f)) = self.action {
            card = card.child(row().child(Button::new(format!("{}-action", self.key), words).on_press(move |s: &mut S| f(s)).build(t)));
        }
        card
    }
}
