//! rune-ui registry: branch 0.6.0. Copied into your app by `rune-ui add branch`; yours to change.
//!
//! "2 / 3": which of several alternative answers is shown, with a step back and a step on (off at the ends). Tabular figures, so the
//! count keeps its width. Left and Right step while either button has focus.
//!
//! ```ignore
//! Branch::new("m2-branch", s.at, 3).on_change(|s: &mut App, i| s.at = i).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, Tx, row, text};
use rune_ui_style::tabular;

use super::agent::{PressTx, icon_button};
use super::base::{Choose, Theme};
use super::icons;

pub struct Branch<S: 'static> {
    key: String,
    at: usize,
    count: usize,
    change: Option<Choose<S>>,
}

impl<S: 'static> Branch<S> {
    /// `at` counts from 0.
    pub fn new(key: impl Into<String>, at: usize, count: usize) -> Branch<S> {
        Branch { key: key.into(), at, count: count.max(1), change: None }
    }

    pub fn on_change(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.change = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let (at, n) = (self.at.min(self.count - 1), self.count);
        let step = |to: Option<usize>| -> Option<PressTx<S>> {
            let f = self.change.clone()?;
            let to = to?;
            Some(Rc::new(move |s: &mut S, _: &mut Tx| f(s, to)))
        };
        let back = at.checked_sub(1);
        let on = (at + 1 < n).then_some(at + 1);
        let words = format!("{} / {n}", at + 1);
        let f = self.change.clone();
        row().key(self.key.clone()).role("group").label(format!("Answer {} of {n}", at + 1)).gap(t.space.at(1.5)).center_y()
            .child(icon_button(t, &format!("{}-prev", self.key), "Previous answer", &icons::CHEVRON_LEFT, None, back.is_none(), step(back)))
            .child(text(words).key(format!("{}-count", self.key)).font(tabular(t.ty.sm)).color(t.ink_muted()).a11y_hidden())
            .child(icon_button(t, &format!("{}-next", self.key), "Next answer", &icons::CHEVRON_RIGHT, None, on.is_none(), step(on)))
            .on_key(move |s: &mut S, k: &KeyEvent, _: &mut Tx| {
                let to = match k.key {
                    Key::Left => back,
                    Key::Right => on,
                    _ => return false,
                };
                match (to, f.as_ref()) {
                    (Some(i), Some(f)) => {
                        f(s, i);
                        true
                    }
                    _ => false,
                }
            })
    }
}
