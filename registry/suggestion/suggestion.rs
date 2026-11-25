//! rune-ui registry: suggestion 0.6.0. Copied into your app by `rune-ui add suggestion`; yours to change.
//!
//! A row of suggested prompts: raised chips (they are pressed), wrapping onto more lines when the row is narrow. Pressing one hands its
//! words to the app.
//!
//! ```ignore
//! Suggestions::new("ideas", &["Summarise this file", "Write tests"]).on_pick(|s: &mut App, w: &str| s.send(w)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, row, text};

use super::agent::chip;
use super::base::{Pick, Theme, hit_area};

pub struct Suggestions<'a, S: 'static> {
    key: String,
    words: &'a [&'a str],
    pick: Option<Pick<S>>,
}

impl<'a, S: 'static> Suggestions<'a, S> {
    pub fn new(key: impl Into<String>, words: &'a [&'a str]) -> Suggestions<'a, S> {
        Suggestions { key: key.into(), words, pick: None }
    }

    pub fn on_pick(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.pick = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let mut r = row().key(self.key.clone()).role("group").label("Suggestions").w_full().gap(t.space.at(2.0)).flex_wrap();
        for (i, w) in self.words.iter().enumerate() {
            let key = format!("{}-{i}", self.key);
            let mut face = chip(t, key.clone(), None).role("button").label((*w).to_owned()).focusable(true).max_w(360.0).child(text((*w).to_owned()).font(t.ty.sm).color(t.ink()).truncate().min_w(0.0).a11y_hidden());
            let press = self.pick.clone().map(|f| {
                let w = (*w).to_owned();
                super::base::press(move |s: &mut S| f(s, &w))
            });
            if let Some(p) = press.clone() {
                face = face.on_press(move |s: &mut S| p(s));
            }
            r = r.child(hit_area(&key, face, 0.0, t.control.slop(t.control.sm), press));
        }
        r
    }
}
