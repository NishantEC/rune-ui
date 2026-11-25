//! rune-ui registry: actions 0.6.0. Copied into your app by `rune-ui add actions`; yours to change.
//!
//! The quiet bar under an answer: Copy (puts the words on the clipboard, then shows a check), Retry, and Good and Bad, which stay down
//! once chosen and say so. Each is a small ghost icon button with a tooltip and a 40 point hit area. Add your own with [`Actions::action`].
//!
//! ```ignore
//! Actions::new("m2-actions").copy(&answer, s.copied, |s: &mut App| s.copied = true).retry(|s: &mut App| s.retry())
//!     .feedback(s.rating, |s: &mut App, r| s.rating = r).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Icon, Tx, row};

use super::agent::{PressTx, icon_button};
use super::base::Theme;
use super::icons;

/// What a rating does with the new rating (`Some(true)` good, `Some(false)` bad, `None` cleared).
pub type Rate<S> = Rc<dyn Fn(&mut S, Option<bool>)>;

struct Action<S: 'static> {
    key: String,
    name: String,
    icon: &'static Icon,
    on: Option<bool>,
    press: PressTx<S>,
}

pub struct Actions<S: 'static> {
    key: String,
    actions: Vec<Action<S>>,
}

impl<S: 'static> Actions<S> {
    pub fn new(key: impl Into<String>) -> Actions<S> {
        Actions { key: key.into(), actions: Vec::new() }
    }

    /// Copy: `words` go to the clipboard and `copied` runs; while `done`, the icon is a check.
    pub fn copy(mut self, words: &str, done: bool, copied: impl Fn(&mut S) + 'static) -> Self {
        let words = words.to_owned();
        let press: PressTx<S> = Rc::new(move |s: &mut S, tx: &mut Tx| {
            tx.copy(words.clone());
            copied(s);
        });
        self.actions.push(Action { key: format!("{}-copy", self.key), name: if done { "Copied".to_owned() } else { "Copy".to_owned() }, icon: if done { &icons::CHECK } else { &icons::COPY }, on: None, press });
        self
    }

    pub fn retry(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.actions.push(Action { key: format!("{}-retry", self.key), name: "Retry".to_owned(), icon: &icons::RETRY, on: None, press: Rc::new(move |s: &mut S, _: &mut Tx| f(s)) });
        self
    }

    /// Good and Bad: `rating` is the current one; pressing the chosen one again clears it.
    pub fn feedback(mut self, rating: Option<bool>, f: impl Fn(&mut S, Option<bool>) + 'static) -> Self {
        let f: Rate<S> = Rc::new(f);
        for (good, name, icon) in [(true, "Good response", &icons::THUMB_UP), (false, "Bad response", &icons::THUMB_DOWN)] {
            let g = f.clone();
            let next = if rating == Some(good) { None } else { Some(good) };
            self.actions.push(Action { key: format!("{}-{}", self.key, if good { "good" } else { "bad" }), name: name.to_owned(), icon, on: Some(rating == Some(good)), press: Rc::new(move |s: &mut S, _: &mut Tx| g(s, next)) });
        }
        self
    }

    /// Any other action: a key suffix, its name (and tooltip), an icon.
    pub fn action(mut self, suffix: &str, name: &str, icon: &'static Icon, f: impl Fn(&mut S) + 'static) -> Self {
        self.actions.push(Action { key: format!("{}-{suffix}", self.key), name: name.to_owned(), icon, on: None, press: Rc::new(move |s: &mut S, _: &mut Tx| f(s)) });
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let mut bar = row().key(self.key).role("toolbar").label("Message actions").gap(t.space.at(3.5)).center_y().h(t.control.sm);
        for a in self.actions {
            bar = bar.child(icon_button(t, &a.key, &a.name, a.icon, a.on, false, Some(a.press)));
        }
        bar
    }
}
