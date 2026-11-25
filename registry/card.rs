//! rune-ui registry: card 0.1.0. Copied into your app by `rune-ui add card`; yours to change.
//!
//! One surface divided into rows, not a stack of cards. A **slab** card is the group's fill a step off the page, no edge and no shadow,
//! its rows split by hairlines inset from its sides. A **raised** card (something that stands out: a result, a question) is the
//! floating colour with a small lift. A [`SectionHeader`] titles a group; [`line`] is the usual line of a settings-like card (a title,
//! a quiet hint under it, a control at the right).
//!
//! ```ignore
//! SectionHeader::new("General").description("How the app starts").build(t)
//! Card::slab("general").rows(vec![card::line(t, "sync", "Sync", Some("In the background"), switch)]).build(t)
//! ```

use rune::{El, col, el, row, text};
use rune_ui_style::{Recipe, paint};

use super::base::Theme;

pub struct Card {
    key: String,
    raised: bool,
    padding: Option<f32>,
    rows: bool,
    children: Vec<El>,
}

impl Card {
    /// The group's slab.
    pub fn slab(key: impl Into<String>) -> Card {
        Card { key: key.into(), raised: false, padding: None, rows: false, children: Vec::new() }
    }

    /// A card that lifts a little off the page.
    pub fn raised(key: impl Into<String>) -> Card {
        Card { raised: true, ..Card::slab(key) }
    }

    /// Room inside (16 by default; rows bring their own).
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = Some(p);
        self
    }

    pub fn child(mut self, c: El) -> Self {
        self.children.push(c);
        self
    }

    /// Rows split by inset hairlines, no padding of the card's own.
    pub fn rows(mut self, rows: Vec<El>) -> Self {
        self.rows = true;
        self.children = rows;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let r = t.radius.xl;
        let pad = self.padding.unwrap_or(if self.rows { 0.0 } else { t.space.at(4.0) });
        let recipe = if self.raised { Recipe::new().fill(t.c.floating).shadows(&t.elevation.sm).shadow(rune::Shadow { spread: 1.0, color: t.c.line, ..rune::Shadow::default() }) } else { t.mat.slab.rest.clone() };
        let mut c = paint(col().key(self.key).w_full().radius(r).continuous_corners().padding(pad), &recipe);
        if self.rows {
            c = c.clip_rounded(r);
        }
        let inset = t.space.at(4.0);
        for (i, child) in self.children.into_iter().enumerate() {
            if self.rows && i > 0 {
                c = c.child(row().w_full().px(inset).child(divider(t)));
            }
            c = c.child(child);
        }
        c
    }
}

/// A hairline across what holds it (structure: between rows and groups).
pub fn divider(t: &Theme) -> El {
    el().w_full().h(1.0).fill(t.c.line).a11y_hidden()
}

/// A card's usual line: a title over a quiet hint at the left, a control at the right.
pub fn line(t: &Theme, key: impl Into<String>, title: impl Into<String>, hint: Option<&str>, control: Option<El>) -> El {
    let mut words = col().grow(1.0).shrink().min_w(0.0).gap(2.0).child(text(title.into()).font(rune_ui_style::weight(t.ty.base, t.ty.medium)).color(t.c.ink).wrap().min_w(0.0));
    if let Some(h) = hint {
        words = words.child(text(h.to_owned()).font(t.ty.sm).color(t.c.ink_muted).wrap().min_w(0.0));
    }
    let mut l = row().key(key.into()).w_full().min_h(52.0).px(t.space.at(4.0)).py(t.space.at(2.5)).gap(t.space.at(4.0)).center_y().child(words);
    if let Some(c) = control {
        l = l.child(c);
    }
    l
}

/// A group's title, and a line saying what the group is for.
pub struct SectionHeader {
    title: String,
    description: Option<String>,
}

impl SectionHeader {
    pub fn new(title: impl Into<String>) -> SectionHeader {
        SectionHeader { title: title.into(), description: None }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let mut c = col().gap(t.space.at(1.0)).pb(t.space.at(3.0)).child(text(self.title).font(t.ty.lg).color(t.c.ink).role("heading").level(2));
        if let Some(d) = self.description {
            c = c.child(text(d).font(t.ty.base).color(t.c.ink_muted).wrap().min_w(0.0));
        }
        c
    }
}
