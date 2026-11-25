//! rune-ui registry: accordion 0.2.0. Copied into your app by `rune-ui add accordion`; yours to change.
//!
//! Sections that fold, in one slab split by hairlines: each trigger a row with its title and a chevron that turns, saying `expanded`;
//! the content under it while open. The app decides which are open (one at a time, or several).
//!
//! ```ignore
//! Accordion::new("faq").item("Is it free?", s.open[0], body).on_toggle(|s: &mut App, i| s.flip(i)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, col, row, text};
use rune_ui_style::weight;

use super::base::{Choose, State, States, Theme, glyph, paint, paint_state};
use super::icons;

pub struct Accordion<S: 'static> {
    key: String,
    items: Vec<(String, bool, El)>,
    pinned: Option<(usize, State)>,
    toggle: Option<Choose<S>>,
}

impl<S: 'static> Accordion<S> {
    pub fn new(key: impl Into<String>) -> Accordion<S> {
        Accordion { key: key.into(), items: Vec::new(), pinned: None, toggle: None }
    }

    /// A section: its title, whether it is open, and what it shows when open.
    pub fn item(mut self, title: impl Into<String>, open: bool, content: El) -> Self {
        self.items.push((title.into(), open, content));
        self
    }

    pub fn state(mut self, i: usize, s: State) -> Self {
        self.pinned = Some((i, s));
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.toggle = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let r = t.radius.xl;
        let mut c = paint(col().key(self.key.clone()).w_full().radius(r).continuous_corners().clip_rounded(r), &t.mat.slab.base);
        for (i, (title, open, content)) in self.items.into_iter().enumerate() {
            if i > 0 {
                c = c.child(row().w_full().px(t.space.at(4.0)).child(rune::el().w_full().h(1.0).fill(t.c.line)));
            }
            let mut st = States::default();
            if let Some((at, s)) = self.pinned
                && at == i
            {
                st.set(s, true);
            }
            let mut trigger = row().key(format!("{}-{i}", self.key)).role("button").label(title.clone()).expanded(open).focusable(true).w_full().h(44.0).px(t.space.at(4.0)).gap(t.space.at(2.0)).center_y();
            trigger = paint_state(trigger, &t.mat.row, st).child(text(title).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).grow(1.0).a11y_hidden()).child(glyph(if open { &icons::CHEVRON_DOWN } else { &icons::CHEVRON_RIGHT }, t.control.icon, t.ink_muted()));
            if let Some(f) = self.toggle.clone() {
                trigger = trigger.on_press(move |s: &mut S| f(s, i));
            }
            c = c.child(trigger);
            if open {
                c = c.child(col().key(format!("{}-{i}-content", self.key)).w_full().px(t.space.at(4.0)).pb(t.space.at(4.0)).child(content));
            }
        }
        c
    }
}
