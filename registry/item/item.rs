//! rune-ui registry: item 0.5.1. Copied into your app by `rune-ui add item`; yours to change.
//!
//! One thing in a list, the way shadcn's Item lays it out: media at the left (an icon in a tinted square, or any element such as an
//! avatar), a title over a quiet description, and actions at the right. Three looks: **Default** (nothing behind it), **Outline** (an
//! inner rim) and **Muted** (a quiet tint). Two sizes. With `on_press` the whole item is a control (it washes on hover, rings on focus).
//!
//! ```ignore
//! Item::new("billing", "Billing").description("Plans, invoices and cards").icon(&icons::FILE).action(open).build(t)
//! ```

use rune::{El, Icon, col, row, text};
use rune_ui_style::{Material, Radius, Size, weight};

use super::base::{Press, Shape, States, Theme, glyph, paint_state, press};

/// shadcn's item looks.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemVariant {
    #[default]
    Default,
    Outline,
    Muted,
}

enum Media {
    Icon(&'static Icon),
    El(El),
}

pub struct Item<S: 'static> {
    key: String,
    title: String,
    description: Option<String>,
    media: Option<Media>,
    actions: Vec<El>,
    variant: ItemVariant,
    size: Size,
    press: Option<Press<S>>,
}

impl<S: 'static> Item<S> {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Item<S> {
        Item { key: key.into(), title: title.into(), description: None, media: None, actions: Vec::new(), variant: ItemVariant::Default, size: Size::Md, press: None }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// An icon in a tinted square.
    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.media = Some(Media::Icon(i));
        self
    }

    /// Any element as the media (an avatar, a picture).
    pub fn media(mut self, e: El) -> Self {
        self.media = Some(Media::El(e));
        self
    }

    /// A control at the right (a button, a switch, a badge).
    pub fn action(mut self, e: El) -> Self {
        self.actions.push(e);
        self
    }

    pub fn variant(mut self, v: ItemVariant) -> Self {
        self.variant = v;
        self
    }

    /// `Size::Sm` is the dense one.
    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// The whole item becomes a control.
    pub fn on_press(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(press(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let sm = self.size == Size::Sm;
        let (pad, gap) = if sm { (t.space.at(2.5), t.space.at(2.5)) } else { (t.metrics.pad_md, t.space.at(4.0)) };
        let r = t.radius.xl;
        let mut mat: Material = match self.variant {
            ItemVariant::Outline => t.mat.outline.clone(),
            _ => t.mat.row.clone(),
        };
        if self.variant == ItemVariant::Muted {
            mat.base = mat.base.fill(t.c.muted.bg);
        }
        let mut e = row().key(self.key.clone()).label(self.title.clone()).w_full().padding(pad).gap(gap).center_y().radius(r).round_as(t);
        if let Some(d) = &self.description {
            e = e.description(d.clone());
        }
        let live = self.press.is_some();
        e = if live { paint_state(e.role("button").focusable(true), &mat, States::default()) } else { rune_ui_style::paint(e.role("group"), &mat.base) };
        let box_size = if sm { 32.0 } else { 40.0 };
        match self.media {
            Some(Media::Icon(i)) => {
                e = e.child(row().size(box_size, box_size).center().radius(Radius::fit(t.radius.md, box_size)).round_as(t).fill(t.c.muted.bg).child(glyph(i, t.control.icon_lg, t.ink_muted())));
            }
            Some(Media::El(m)) => e = e.child(m),
            None => {}
        }
        let mut words = col().grow(1.0).shrink().min_w(0.0).gap(t.space.at(0.5)).child(text(self.title).font(weight(if sm { t.ty.sm } else { t.ty.base }, t.ty.medium)).color(t.ink()).truncate().a11y_hidden());
        if let Some(d) = self.description {
            words = words.child(text(d).font(t.ty.sm).color(t.ink_muted()).wrap().max_lines(2).min_w(0.0).a11y_hidden());
        }
        e = e.child(words);
        if !self.actions.is_empty() {
            e = e.child(row().gap(t.space.at(2.0)).center_y().children(self.actions));
        }
        if let Some(f) = self.press {
            e = e.on_press(move |s: &mut S| f(s));
        }
        e
    }
}
