//! rune-ui registry: sources 0.6.0. Copied into your app by `rune-ui add sources`; yours to change.
//!
//! What an answer drew on, behind one quiet line ("Used 4 sources"). Open, each source is a link row: its title, then its site in the
//! third ink. Pressing one (or Return on it) hands its address to the app, which decides how to open it.
//!
//! ```ignore
//! Sources::new("m2-sources", &sources, s.open("src")).on_toggle(|s: &mut App| s.flip("src")).on_open(|s: &mut App, url: &str| s.open_url(url)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, col, row, text};

use super::agent::FoldLine;
use super::base::{Pick, Press, Shape, Theme, glyph, paint_state};
use super::icons;

/// One source.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Source {
    pub title: String,
    pub url: String,
}

impl Source {
    pub fn new(title: impl Into<String>, url: impl Into<String>) -> Source {
        Source { title: title.into(), url: url.into() }
    }

    /// The site: the address without its scheme, `www.` and path.
    pub fn site(&self) -> &str {
        let u = self.url.split("://").nth(1).unwrap_or(&self.url);
        let host = u.split('/').next().unwrap_or(u);
        host.strip_prefix("www.").unwrap_or(host)
    }
}

pub struct Sources<'a, S: 'static> {
    key: String,
    sources: &'a [Source],
    open: bool,
    toggle: Option<Press<S>>,
    follow: Option<Pick<S>>,
}

impl<'a, S: 'static> Sources<'a, S> {
    pub fn new(key: impl Into<String>, sources: &'a [Source], open: bool) -> Sources<'a, S> {
        Sources { key: key.into(), sources, open, toggle: None, follow: None }
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(Rc::new(f));
        self
    }

    /// A source chosen: its address.
    pub fn on_open(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.follow = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let n = self.sources.len();
        let mut line = FoldLine::<S>::new(self.key.clone(), format!("Used {n} source{}", if n == 1 { "" } else { "s" }), self.open).lead(glyph(&icons::GLOBE, t.control.icon, t.ink_faint()));
        if let Some(f) = self.toggle {
            line = line.on_toggle(move |s: &mut S| f(s));
        }
        let mut list = col().key(format!("{}-list", self.key)).role("list").label("Sources").w_full().gap(1.0);
        for (i, src) in self.sources.iter().enumerate() {
            let mut r = row().key(format!("{}-{i}", self.key)).role("link").label(src.title.clone()).description(src.url.clone()).focusable(true).w_full().min_h(t.metrics.row.max(t.control.hit)).px(t.space.at(2.0)).gap(t.space.at(2.5)).center_y().radius(t.radius.md).round_as(t);
            r = paint_state(r, &t.mat.row, Default::default())
                .child(glyph(&icons::LINK, t.control.icon_sm, t.ink_faint()))
                .child(text(src.title.clone()).font(t.ty.base).color(t.ink()).truncate().min_w(0.0).shrink().a11y_hidden())
                .child(text(src.site().to_owned()).font(t.ty.sm).color(t.ink_faint()).truncate().min_w(0.0).a11y_hidden());
            if let Some(f) = self.follow.clone() {
                let url = src.url.clone();
                r = r.on_press(move |s: &mut S| f(s, &url));
            }
            list = list.child(r);
        }
        line.build(t, vec![list])
    }
}
