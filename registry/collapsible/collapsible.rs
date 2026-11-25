//! rune-ui registry: collapsible 0.5.0. Copied into your app by `rune-ui add collapsible`; yours to change.
//!
//! One section that folds behind its trigger: a row with a chevron that turns, saying `expanded` to assistive technology, and the
//! content under it while open. The fold lands at once.
//!
//! ```ignore
//! Collapsible::new("more", "More options", s.more).child(body).on_toggle(|s: &mut App| s.more = !s.more).build(t)
//! ```

use std::rc::Rc;

use rune::{El, col, row, text};
use rune_ui_style::weight;

use super::base::{Press, Shape, State, Theme, glyph, paint_state, states};
use super::icons;

pub struct Collapsible<S: 'static> {
    key: String,
    title: String,
    open: bool,
    meta: Option<String>,
    body: Vec<El>,
    disabled: bool,
    pinned: Option<State>,
    toggle: Option<Press<S>>,
}

impl<S: 'static> Collapsible<S> {
    pub fn new(key: impl Into<String>, title: impl Into<String>, open: bool) -> Collapsible<S> {
        Collapsible { key: key.into(), title: title.into(), open, meta: None, body: Vec::new(), disabled: false, pinned: None, toggle: None }
    }

    /// Quiet words at the trigger's right ("3 items").
    pub fn meta(mut self, m: impl Into<String>) -> Self {
        self.meta = Some(m.into());
        self
    }

    pub fn child(mut self, c: El) -> Self {
        self.body.push(c);
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn state(mut self, s: State) -> Self {
        self.pinned = Some(s);
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let st = states(self.pinned, self.disabled, false);
        let mut trigger = row().key(format!("{}-trigger", self.key)).role("button").label(self.title.clone()).expanded(self.open).focusable(!self.disabled).w_full().h(t.control.md).px(t.metrics.gap - 2.0).gap(t.metrics.gap).center_y().radius(t.radius.md).round_as(t);
        trigger = paint_state(trigger, &t.mat.row, st);
        let chevron = glyph(if self.open { &icons::CHEVRON_DOWN } else { &icons::CHEVRON_RIGHT }, t.control.icon, t.ink_muted());
        trigger = trigger.child(chevron).child(text(self.title).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).grow(1.0).a11y_hidden());
        if let Some(m) = self.meta {
            trigger = trigger.child(text(m).font(t.ty.sm).color(t.ink_faint()).a11y_hidden());
        }
        if let Some(f) = self.toggle.filter(|_| !self.disabled) {
            trigger = trigger.on_press(move |s: &mut S| f(s));
        }
        if self.disabled {
            trigger = trigger.disabled(true);
        }
        let mut c = col().key(self.key.clone()).w_full().gap(t.space.at(1.0)).child(trigger);
        if self.open {
            c = c.child(col().key(format!("{}-content", self.key)).w_full().pl(t.space.at(7.0)).pr(t.space.at(1.5)).pb(t.space.at(2.0)).gap(t.space.at(2.0)).children(self.body));
        }
        c
    }
}
