//! rune-ui registry: segmented 0.1.0. Copied into your app by `rune-ui add segmented`; yours to change.
//!
//! One of two to five, side by side: a **well** track holding one **raised** segment (the chosen one), so the two materials explain the
//! control. Radii are concentric (a segment's radius is the track's minus its padding). An unchosen segment washes on hover; the choice
//! lands at once. An icon may lead each word.
//!
//! ```ignore
//! Segmented::new("view", "View", &[("List", None), ("Board", None)], s.view).build(t, |s: &mut App, i| s.view = i)
//! ```

use rune::{El, Icon, row, text};
use rune_ui_style::{Radius, paint, paint_material};

use super::base::{Look, Size, Theme, glyph};

const PAD: f32 = 3.0;

pub struct Segmented<'a> {
    key: String,
    name: String,
    options: &'a [(&'a str, Option<&'static Icon>)],
    selected: usize,
    size: Size,
    full: bool,
    disabled: bool,
    look: Option<(usize, Look)>,
}

impl<'a> Segmented<'a> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, options: &'a [(&'a str, Option<&'static Icon>)], selected: usize) -> Segmented<'a> {
        Segmented { key: key.into(), name: name.into(), options, selected, size: Size::Md, full: false, disabled: false, look: None }
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// As wide as what holds it, the segments sharing it.
    pub fn full(mut self) -> Self {
        self.full = true;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    /// Pins a look on segment `i` (galleries).
    pub fn look(mut self, i: usize, look: Look) -> Self {
        self.look = Some((i, look));
        self
    }

    pub fn build<S: 'static>(self, t: &Theme, choose: impl Fn(&mut S, usize) + Clone + 'static) -> El {
        let h = self.size.height(t);
        let outer = self.size.radius(t);
        let inner = Radius::inner(outer, PAD);
        let well = &t.mat.well;
        let mut track = row().key(self.key.clone()).role("radiogroup").label(self.name.clone()).h(h).padding(PAD).gap(2.0).center_y().radius(outer).continuous_corners();
        track = paint(track, if self.disabled { &well.disabled } else { &well.rest });
        if self.full {
            track = track.w_full();
        }
        for (i, (label, icon)) in self.options.iter().enumerate() {
            let on = i == self.selected;
            let look = self.look.filter(|(at, _)| *at == i).map_or(Look::Live, |(_, l)| l);
            let mut seg = row().key(format!("{}-{i}", self.key)).role("radio").label((*label).to_owned()).checked(on).focusable(!self.disabled).h(h - 2.0 * PAD).px(self.size.pad(t) - 2.0).gap(t.space.at(1.5)).center().radius(inner).continuous_corners();
            if self.full {
                seg = seg.grow(1.0).min_w(0.0);
            }
            // The chosen segment is raised and still; the others wash on hover. Neither sinks: the track does not move.
            let m = if on { t.mat.raised.clone().pressed(|r| r) } else { t.mat.ghost.clone().pressed(|r| r.wash(t.c.hover)) };
            seg = paint_material(seg, &m, if self.disabled { Look::Live } else { look }, false, false);
            let ink = if on { t.c.ink } else { t.c.ink_muted };
            if let Some(g) = icon {
                seg = seg.child(glyph(g, self.size.icon(t), ink));
            }
            seg = seg.child(text((*label).to_owned()).font(self.size.text(t, if on { t.ty.medium } else { t.ty.regular })).color(ink).truncate().min_w(0.0).a11y_hidden());
            if !self.disabled {
                let choose = choose.clone();
                seg = seg.on_press(move |s: &mut S| choose(s, i));
            }
            track = track.child(seg);
        }
        if self.disabled {
            track = track.disabled(true);
        }
        track
    }
}
