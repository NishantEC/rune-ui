//! rune-ui registry: queue 0.6.0. Copied into your app by `rune-ui add queue`; yours to change.
//!
//! Messages waiting to be sent while the agent is busy: a quiet heading with the count, then one row each. A row takes focus; Alt+Up and
//! Alt+Down move it (focus moves with it), Delete or Backspace removes it, and its remove button does the same by pointer.
//!
//! ```ignore
//! Queue::new("queue", &s.queued).on_move(|s: &mut App, from, to| s.queued.swap(from, to))
//!     .on_remove(|s: &mut App, i| { s.queued.remove(i); }).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, Tx, col, row, text};

use super::agent::{PressTx, caption, icon_button};
use super::base::{Choose, Shape, Theme, glyph, paint_state};
use super::icons;

/// What a move does: (from, to).
pub type Move<S> = Rc<dyn Fn(&mut S, usize, usize)>;

pub struct Queue<'a, S: 'static> {
    key: String,
    items: &'a [String],
    moved: Option<Move<S>>,
    removed: Option<Choose<S>>,
}

impl<'a, S: 'static> Queue<'a, S> {
    pub fn new(key: impl Into<String>, items: &'a [String]) -> Queue<'a, S> {
        Queue { key: key.into(), items, moved: None, removed: None }
    }

    pub fn on_move(mut self, f: impl Fn(&mut S, usize, usize) + 'static) -> Self {
        self.moved = Some(Rc::new(f));
        self
    }

    pub fn on_remove(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.removed = Some(Rc::new(f));
        self
    }

    /// The key of the row at `i` (to focus it).
    pub fn row_key(key: &str, i: usize) -> String {
        format!("{key}-item-{i}")
    }

    pub fn build(self, t: &Theme) -> El {
        let n = self.items.len();
        let mut list = col().key(format!("{}-list", self.key)).role("list").label("Queued messages").w_full().gap(1.0);
        for (i, words) in self.items.iter().enumerate() {
            let key = Queue::<S>::row_key(&self.key, i);
            let (moved, removed, base) = (self.moved.clone(), self.removed.clone(), self.key.clone());
            let face = row().key(key.clone()).role("listitem").label(format!("Queued {} of {n}: {words}", i + 1)).description("Alt Up or Down moves it, Delete removes it").focusable(true).w_full().h(t.metrics.row).pl(t.space.at(2.0)).gap(t.metrics.gap).center_y().radius(t.radius.md).round_as(t);
            let mut face = paint_state(face, &t.mat.row, Default::default())
                .child(glyph(&icons::GRIP, t.control.icon_sm, t.ink_faint()))
                .child(text(words.clone()).font(t.ty.base).color(t.ink_muted()).truncate().min_w(0.0).grow(1.0).a11y_hidden())
                .on_key(move |s: &mut S, k: &KeyEvent, tx: &mut Tx| {
                    let to = match k.key {
                        Key::Up if k.mods.alt && i > 0 => i - 1,
                        Key::Down if k.mods.alt && i + 1 < n => i + 1,
                        Key::Delete | Key::Backspace => {
                            let Some(f) = removed.as_ref() else { return false };
                            f(s, i);
                            if n > 1 {
                                tx.focus(Queue::<S>::row_key(&base, i.min(n - 2)), true);
                            }
                            return true;
                        }
                        _ => return false,
                    };
                    let Some(f) = moved.as_ref() else { return false };
                    f(s, i, to);
                    tx.focus(Queue::<S>::row_key(&base, to), true);
                    true
                });
            if let Some(f) = self.removed.clone() {
                let press: PressTx<S> = Rc::new(move |s: &mut S, _: &mut Tx| f(s, i));
                face = face.child(icon_button(t, &format!("{key}-remove"), &format!("Remove “{words}”"), &icons::CLOSE, None, false, Some(press)));
            }
            list = list.child(face);
        }
        col().key(self.key).w_full().gap(t.space.at(1.5)).child(caption(t, &format!("{n} queued"))).child(list)
    }
}
