//! rune-ui registry: plan 0.6.0. Copied into your app by `rune-ui add plan`; yours to change.
//!
//! A checklist of what the agent means to do: a title, how many are done, and each step as todo (a hollow ring), doing (a filled
//! ring, in the first ink), done (a check in the success ink, the words quiet) or skipped (a slashed ring, faint).
//!
//! ```ignore
//! Plan::new("plan", "Plan", &steps).build(t)
//! ```

use rune::{El, Icon, col, row, text};
use rune_ui_style::weight;

use super::agent::meta;
use super::base::{Theme, glyph};
use super::icons;

/// Where a step of the plan stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Todo {
    #[default]
    Todo,
    Doing,
    Done,
    Skipped,
}

impl Todo {
    pub fn word(self) -> &'static str {
        match self {
            Todo::Todo => "To do",
            Todo::Doing => "In progress",
            Todo::Done => "Done",
            Todo::Skipped => "Skipped",
        }
    }
}

pub struct Plan<'a> {
    key: String,
    title: String,
    steps: &'a [(String, Todo)],
}

impl<'a> Plan<'a> {
    pub fn new(key: impl Into<String>, title: impl Into<String>, steps: &'a [(String, Todo)]) -> Plan<'a> {
        Plan { key: key.into(), title: title.into(), steps }
    }

    pub fn build(self, t: &Theme) -> El {
        let done = self.steps.iter().filter(|(_, s)| *s == Todo::Done).count();
        let head = row().w_full().gap(t.space.at(2.0)).items_end()
            .child(text(self.title.clone()).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).truncate().min_w(0.0).shrink())
            .child(meta(t, format!("{done} of {} done", self.steps.len())));
        let mut list = col().key(format!("{}-steps", self.key)).role("list").label(self.title.clone()).w_full().gap(t.space.at(2.0));
        for (i, (words, s)) in self.steps.iter().enumerate() {
            let (icon, ink, mark): (&'static Icon, _, _) = match s {
                Todo::Todo => (&icons::CIRCLE, t.ink(), t.ink_faint()),
                Todo::Doing => (&icons::CIRCLE_DOT, t.ink(), t.ink()),
                Todo::Done => (&icons::CIRCLE_CHECK, t.ink_muted(), t.c.success.fg),
                Todo::Skipped => (&icons::CIRCLE_SLASH, t.ink_faint(), t.ink_faint()),
            };
            let style = if *s == Todo::Doing { weight(t.ty.base, t.ty.medium) } else { t.ty.base };
            list = list.child(row().key(format!("{}-step-{i}", self.key)).role("listitem").label(format!("{}: {words}", s.word())).w_full().gap(t.metrics.gap).items_start()
                .child(row().size(t.control.icon, style.line_height.max(t.control.icon)).center().child(glyph(icon, t.control.icon, mark)))
                .child(text(words.clone()).font(style).color(ink).wrap().min_w(0.0).shrink().a11y_hidden()));
        }
        col().key(self.key).role("group").label(format!("{}: {done} of {} done", self.title, self.steps.len())).w_full().gap(t.space.at(3.0)).child(head).child(list)
    }
}
