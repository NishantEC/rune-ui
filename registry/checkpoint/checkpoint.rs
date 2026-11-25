//! rune-ui registry: checkpoint 0.8.0 Copied into your app by `rune-ui add checkpoint`; yours to change.
//!
//! A divider in the conversation where the work was saved: a hairline, a flag with quiet words ("Checkpoint · 14:02"), a Restore button
//! that puts the files back as they were there, and the hairline again.
//!
//! ```ignore
//! Checkpoint::new("cp-3", "Checkpoint · 14:02").on_restore(|s: &mut App| s.restore(3)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, el, row, text};
use rune_ui_style::weight;

use super::base::{Press, Shape, Theme, glyph, hit_area, paint_state};
use super::icons;

pub struct Checkpoint<S: 'static> {
    key: String,
    words: String,
    restore: Option<Press<S>>,
}

impl<S: 'static> Checkpoint<S> {
    pub fn new(key: impl Into<String>, words: impl Into<String>) -> Checkpoint<S> {
        Checkpoint { key: key.into(), words: words.into(), restore: None }
    }

    /// Shows Restore.
    pub fn on_restore(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.restore = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let rule = || el().h(1.0).grow(1.0).fill(t.c.line).a11y_hidden();
        let mut r = row().key(self.key.clone()).role("group").label(self.words.clone()).w_full().gap(t.space.at(3.0)).center_y()
            .child(rule())
            .child(glyph(&icons::FLAG, t.control.icon_sm, t.ink_faint()))
            .child(text(self.words).font(t.ty.sm).color(t.ink_faint()).truncate().min_w(0.0).shrink().a11y_hidden());
        if let Some(f) = self.restore {
            let h = t.control.sm;
            let key = format!("{}-restore", self.key);
            let g = f.clone();
            let face = paint_state(row().key(key.clone()).role("button").label("Restore checkpoint").focusable(true).h(h).px(t.space.at(2.0)).gap(t.space.at(1.5)).center().radius(rune_ui_style::Radius::fit(t.radius.md, h)).round_as(t), &t.mat.ghost, Default::default())
                .child(glyph(&icons::RESTORE, t.control.icon_sm, t.ink_muted()))
                .child(text("Restore").font(weight(t.ty.sm, t.ty.medium)).color(t.ink_muted()).a11y_hidden())
                .on_press(move |s: &mut S| g(s));
            r = r.child(hit_area(&key, face, 0.0, t.control.slop(h), Some(f)));
        }
        r.child(rule())
    }
}
