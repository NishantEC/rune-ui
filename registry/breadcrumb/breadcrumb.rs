//! rune-ui registry: breadcrumb 0.2.0. Copied into your app by `rune-ui add breadcrumb`; yours to change.
//!
//! Where you are: links to the places above, each a quiet row that washes on hover, separated by small chevrons, and the current page
//! last in the first ink (it is the page's heading). A navigation landmark named "Breadcrumb".
//!
//! ```ignore
//! Breadcrumb::new("crumbs", "Appearance").part("Settings", |s: &mut App| s.go("settings")).build(t)
//! ```

use std::rc::Rc;

use rune::{El, row, text};
use rune_ui_style::weight;

use super::base::{Theme, glyph, paint_state};
use super::icons;

pub struct Breadcrumb<S: 'static> {
    key: String,
    parts: Vec<(String, Option<Rc<dyn Fn(&mut S)>>)>,
    current: String,
}

impl<S: 'static> Breadcrumb<S> {
    pub fn new(key: impl Into<String>, current: impl Into<String>) -> Breadcrumb<S> {
        Breadcrumb { key: key.into(), parts: Vec::new(), current: current.into() }
    }

    /// A place above, with where a press goes.
    pub fn part(mut self, words: impl Into<String>, f: impl Fn(&mut S) + 'static) -> Self {
        self.parts.push((words.into(), Some(Rc::new(f))));
        self
    }

    /// A place above that is not a link.
    pub fn plain(mut self, words: impl Into<String>) -> Self {
        self.parts.push((words.into(), None));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let mut r = row().key(self.key.clone()).role("navigation").label("Breadcrumb").gap(t.space.at(0.5)).center_y();
        for (i, (words, press)) in self.parts.into_iter().enumerate() {
            let w = text(words.clone()).font(t.ty.base).color(t.ink_muted()).a11y_hidden();
            r = r.child(match press {
                Some(f) => paint_state(row().key(format!("{}-{i}", self.key)).role("link").label(words).focusable(true).h(t.control.sm).px(t.space.at(1.5)).center_y().radius(t.radius.sm), &t.mat.row, Default::default()).on_press(move |s: &mut S| f(s)).child(w),
                None => row().px(t.space.at(1.5)).child(w),
            });
            r = r.child(glyph(&icons::CHEVRON_RIGHT, t.control.icon_sm, t.ink_faint()));
        }
        r.child(text(self.current).key(format!("{}-current", self.key)).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).role("heading").px(t.space.at(1.5)))
    }
}
