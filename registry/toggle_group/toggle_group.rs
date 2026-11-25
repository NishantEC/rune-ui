//! rune-ui registry: toggle_group 0.3.0. Copied into your app by `rune-ui add toggle_group`; yours to change.
//!
//! Two to five toggles side by side: one choice ([`ToggleGroup::single`], a radio group to assistive technology) or several
//! ([`ToggleGroup::multiple`]). A **well** track holds the items; chosen ones are **raised**, so the two materials explain the control.
//! Radii are concentric: an item's radius is the track's minus its padding. An unchosen item washes on hover; choices land at once.
//!
//! ```ignore
//! ToggleGroup::single("view", "View", &[("List", None), ("Board", None)], Some(s.view)).on_change(|s: &mut App, i| s.view = i).build(t)
//! ```

use rune::{El, Icon, row, text};
use rune_ui_style::{Delta, Radius};

use super::base::{Choose, Size, SizeExt, State, States, Theme, glyph, paint, paint_state};

const PAD: f32 = 3.0;

pub struct ToggleGroup<'a, S: 'static> {
    key: String,
    name: String,
    items: &'a [(&'a str, Option<&'static Icon>)],
    on: Vec<bool>,
    multiple: bool,
    icon_only: bool,
    size: Size,
    full: bool,
    disabled: bool,
    pinned: Option<(usize, State)>,
    change: Option<Choose<S>>,
}

impl<'a, S: 'static> ToggleGroup<'a, S> {
    /// One of the items (or none).
    pub fn single(key: impl Into<String>, name: impl Into<String>, items: &'a [(&'a str, Option<&'static Icon>)], selected: Option<usize>) -> ToggleGroup<'a, S> {
        let on = (0..items.len()).map(|i| Some(i) == selected).collect();
        ToggleGroup { key: key.into(), name: name.into(), items, on, multiple: false, icon_only: false, size: Size::Md, full: false, disabled: false, pinned: None, change: None }
    }

    /// Any of the items.
    pub fn multiple(key: impl Into<String>, name: impl Into<String>, items: &'a [(&'a str, Option<&'static Icon>)], on: &[bool]) -> ToggleGroup<'a, S> {
        ToggleGroup { multiple: true, on: (0..items.len()).map(|i| on.get(i).copied().unwrap_or(false)).collect(), ..ToggleGroup::single(key, name, items, None) }
    }

    /// Icons only; the words become each item's name and tooltip.
    pub fn icon_only(mut self) -> Self {
        self.icon_only = true;
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    /// As wide as what holds it, items sharing it.
    pub fn full(mut self) -> Self {
        self.full = true;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn state(mut self, i: usize, s: State) -> Self {
        self.pinned = Some((i, s));
        self
    }

    /// The item pressed (for a single group, the one to choose; for a multiple group, the one to flip).
    pub fn on_change(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.change = Some(std::rc::Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let h = self.size.height(t);
        let outer = self.size.radius(t);
        let inner = Radius::inner(outer, PAD);
        let mut track = row().key(self.key.clone()).role(if self.multiple { "group" } else { "radiogroup" }).label(self.name.clone()).h(h).padding(PAD).gap(2.0).center_y().radius(outer).continuous_corners();
        track = paint(track, &t.mat.well.state(if self.disabled { State::Disabled } else { State::Rest }));
        if self.full {
            track = track.w_full();
        }
        // Chosen items are raised and still; the others wash on hover. Neither sinks: the track does not move.
        let chosen = t.mat.raised.clone().on(State::Pressed, Delta::new());
        let open = t.mat.row.clone();
        for (i, (label, icon)) in self.items.iter().enumerate() {
            let on = self.on[i];
            let mut st = States::default();
            if let Some((at, s)) = self.pinned
                && at == i
                && !self.disabled
            {
                st.set(s, true);
            }
            let mut item = row().key(format!("{}-{i}", self.key)).role(if self.multiple { "button" } else { "radio" }).label((*label).to_owned()).checked(on).focusable(!self.disabled).h(h - 2.0 * PAD).gap(t.metrics.gap).center().radius(inner).continuous_corners();
            item = if self.icon_only { item.w(h - 2.0 * PAD).help((*label).to_owned()) } else { item.px(self.size.pad(t) - 2.0) };
            if self.full {
                item = item.grow(1.0).min_w(0.0);
            }
            item = paint_state(item, if on { &chosen } else { &open }, st);
            let ink = if on { t.ink() } else { t.ink_muted() };
            if let Some(g) = icon {
                item = item.child(glyph(g, self.size.icon(t), ink));
            }
            if !self.icon_only {
                item = item.child(text((*label).to_owned()).font(self.size.text(t, if on { t.ty.medium } else { t.ty.regular })).color(ink).truncate().min_w(0.0).a11y_hidden());
            }
            if let Some(f) = self.change.clone().filter(|_| !self.disabled) {
                item = item.on_press(move |s: &mut S| f(s, i));
            }
            track = track.child(item);
        }
        if self.disabled {
            track = track.disabled(true);
        }
        track
    }
}
