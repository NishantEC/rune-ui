//! rune-ui registry: inline_citation 0.6.0. Copied into your app by `rune-ui add inline_citation`; yours to change.
//!
//! A small chip inside an answer naming where a claim came from ("example.com +2"). Resting the pointer on it, or Return on it, shows a
//! hover card from the shared overlay layer with each source's title, site and a line from it; it never moves the words around it.
//!
//! ```ignore
//! InlineCitation::new("c1", &sources, s.cite).on_change(|s: &mut App, h| s.cite = h).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, col, row, text};
use rune_ui_style::{tabular, weight};

use super::base::{Theme, glyph};
use super::hover_card::{HoverCard, HoverCardState};
use super::icons;

/// One cited source.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Citation {
    pub title: String,
    pub url: String,
    /// A line from it.
    pub quote: String,
}

impl Citation {
    pub fn site(&self) -> &str {
        let u = self.url.split("://").nth(1).unwrap_or(&self.url);
        let host = u.split('/').next().unwrap_or(u);
        host.strip_prefix("www.").unwrap_or(host)
    }
}

pub struct InlineCitation<'a, S: 'static> {
    key: String,
    sources: &'a [Citation],
    state: HoverCardState,
    change: Option<super::hover_card::OnChange<S>>,
}

impl<'a, S: 'static> InlineCitation<'a, S> {
    pub fn new(key: impl Into<String>, sources: &'a [Citation], state: HoverCardState) -> InlineCitation<'a, S> {
        InlineCitation { key: key.into(), sources, state, change: None }
    }

    pub fn on_change(mut self, f: impl Fn(&mut S, HoverCardState) + 'static) -> Self {
        self.change = Some(Rc::new(f));
        self
    }

    /// The chip in place; its card goes into `layer`.
    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let first = self.sources.first().map(|c| c.site().to_owned()).unwrap_or_default();
        let words = if self.sources.len() > 1 { format!("{first} +{}", self.sources.len() - 1) } else { first };
        let chip = row().h(t.ty.sm.line_height.max(18.0)).px(t.space.at(1.5)).center().radius(t.radius.sm).fill(t.c.muted.bg).child(text(words.clone()).font(tabular(t.ty.xs)).color(t.ink_muted()));
        let mut card = HoverCard::<S>::new(self.key, format!("Source: {words}"), self.state).trigger(chip).width(320.0);
        for c in self.sources {
            card = card.child(col().w_full().gap(t.space.at(1.0))
                .child(row().gap(t.space.at(1.5)).center_y().child(glyph(&icons::GLOBE, t.control.icon_sm, t.ink_faint())).child(text(c.site().to_owned()).font(t.ty.xs).color(t.ink_faint()).truncate().min_w(0.0)))
                .child(text(c.title.clone()).font(weight(t.ty.base, t.ty.medium)).color(t.c.floating.fg).wrap().min_w(0.0))
                .child(text(c.quote.clone()).font(t.ty.sm).color(t.ink_muted()).wrap().min_w(0.0)));
        }
        if let Some(f) = self.change {
            card = card.on_change(move |s: &mut S, h| f(s, h));
        }
        card.build(t, layer)
    }
}
