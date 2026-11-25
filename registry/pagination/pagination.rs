//! rune-ui registry: pagination 0.5.1. Copied into your app by `rune-ui add pagination`; yours to change. Needs: button.
//!
//! Previous, the pages near the current one with the first and the last always shown and gaps marked, and Next. Pages are ghost
//! buttons; the current one is drawn down (an outline that becomes a well) and says so. Previous and Next are off at the ends. The
//! row keeps its width as the page changes (every slot is there, gaps included), so nothing beside it shifts.
//!
//! ```ignore
//! Pagination::new("runs-pages", s.page, 12).on_page(|s: &mut App, p| s.page = p).build(t)
//! ```

use std::rc::Rc;

use rune::{El, row};
use rune_ui_style::{ButtonVariant, Size};

use super::base::{Choose, SizeExt, Theme, glyph};
use super::button::Button;
use super::icons;

pub struct Pagination<S: 'static> {
    key: String,
    page: usize,
    pages: usize,
    siblings: usize,
    words: bool,
    size: Size,
    on_page: Option<Choose<S>>,
}

/// One slot in the row: a page (1-based) or a gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Page(usize),
    Gap,
}

/// The slots for `page` of `pages` with `siblings` either side: always `2 * siblings + 5` long once there are that many pages.
pub fn slots(page: usize, pages: usize, siblings: usize) -> Vec<Slot> {
    let n = 2 * siblings + 5;
    if pages <= n {
        return (1..=pages).map(Slot::Page).collect();
    }
    let page = page.clamp(1, pages);
    let left = page.saturating_sub(siblings).max(1);
    let right = (page + siblings).min(pages);
    if left <= 3 {
        let mut v: Vec<Slot> = (1..=n - 2).map(Slot::Page).collect();
        v.extend([Slot::Gap, Slot::Page(pages)]);
        v
    } else if right >= pages - 2 {
        let mut v = vec![Slot::Page(1), Slot::Gap];
        v.extend((pages + 3 - n..=pages).map(Slot::Page));
        v
    } else {
        let mut v = vec![Slot::Page(1), Slot::Gap];
        v.extend((left..=right).map(Slot::Page));
        v.extend([Slot::Gap, Slot::Page(pages)]);
        v
    }
}

impl<S: 'static> Pagination<S> {
    /// `page` is 1-based.
    pub fn new(key: impl Into<String>, page: usize, pages: usize) -> Pagination<S> {
        Pagination { key: key.into(), page, pages: pages.max(1), siblings: 1, words: true, size: Size::Md, on_page: None }
    }

    /// Pages shown either side of the current one (1 by default).
    pub fn siblings(mut self, n: usize) -> Self {
        self.siblings = n;
        self
    }

    /// Previous and Next as arrows alone.
    pub fn compact(mut self) -> Self {
        self.words = false;
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn on_page(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.on_page = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let k = self.key.clone();
        let page = self.page.clamp(1, self.pages);
        let go = |p: usize| self.on_page.clone().map(|f| move |s: &mut S| f(s, p));
        let edge = |name: &str, icon, to: Option<usize>, trailing: bool| {
            let mut b = Button::<S>::new(format!("{k}-{}", name.to_lowercase()), name.to_owned()).variant(ButtonVariant::Ghost).size(self.size).disabled(to.is_none());
            b = match (self.words, trailing) {
                (false, _) => b.icon_only(icon),
                (true, false) => b.icon(icon),
                (true, true) => b.trailing_icon(icon),
            };
            if let Some(f) = to.and_then(go) {
                b = b.on_press(f);
            }
            b.build(t)
        };
        let mut r = row().key(k.clone()).role("navigation").label("Pages").gap(t.space.at(1.0)).center_y();
        r = r.child(edge("Previous", &icons::CHEVRON_LEFT, (page > 1).then(|| page - 1), false));
        let h = SizeExt::height(self.size, t);
        let digits = self.pages.to_string().len() as f32;
        let slot = h.max(digits * t.ty.base.size * 0.7 + 2.0 * SizeExt::pad(self.size, t));
        for (i, s) in slots(page, self.pages, self.siblings).into_iter().enumerate() {
            r = r.child(match s {
                Slot::Page(p) => {
                    let current = p == page;
                    let mut b = Button::<S>::new(format!("{k}-page-{p}"), p.to_string()).variant(if current { ButtonVariant::Outline } else { ButtonVariant::Ghost }).size(self.size).selected(current);
                    if let Some(f) = go(p).filter(|_| !current) {
                        b = b.on_press(f);
                    }
                    // Every page slot has one width (the widest page number's), so the row keeps its width as the page changes.
                    row().w(slot).center().child(b.build(t))
                }
                Slot::Gap => row().key(format!("{k}-gap{i}")).w(h).h(h).center().a11y_hidden().child(glyph(&icons::MORE, t.control.icon, t.ink_faint())),
            });
        }
        r = r.child(edge("Next", &icons::CHEVRON_RIGHT, (page < self.pages).then(|| page + 1), true));
        r
    }
}
