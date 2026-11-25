//! rune-ui registry: chip 0.1.0. Copied into your app by `rune-ui add chip`; yours to change.
//!
//! A chip names a thing (a tag, a filter, a suggestion). A plain tag is a quiet tint with words in the second ink. A pressable chip is the
//! **chip** material: raised, and the one control that lifts off the page with a soft shadow; selected, it stays down (a well) and its
//! words take the first ink. An icon may lead; a removable chip ends in a small close button with its own name.
//!
//! ```ignore
//! Chip::new("tag-docs", "docs").icon(&icons::TAG).build(t)
//! Chip::new("filter-open", "Open").selected(s.open_only).on_press(|s: &mut App| s.open_only = !s.open_only).build(t)
//! ```

use rune::{El, Icon, row, text};
use rune_ui_style::{Recipe, paint, paint_material, tabular};

use super::base::{Look, Press, Theme, glyph, press};
use super::icons;

const HEIGHT: f32 = 24.0;

pub struct Chip<S: 'static> {
    key: String,
    words: String,
    icon: Option<&'static Icon>,
    selected: bool,
    disabled: bool,
    look: Look,
    press: Option<Press<S>>,
    remove: Option<Press<S>>,
    max_w: f32,
}

impl<S: 'static> Chip<S> {
    pub fn new(key: impl Into<String>, words: impl Into<String>) -> Chip<S> {
        Chip { key: key.into(), words: words.into(), icon: None, selected: false, disabled: false, look: Look::Live, press: None, remove: None, max_w: 220.0 }
    }

    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self
    }

    pub fn selected(mut self, on: bool) -> Self {
        self.selected = on;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn max_w(mut self, w: f32) -> Self {
        self.max_w = w;
        self
    }

    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    /// Makes it a filter or a suggestion: raised, it lifts.
    pub fn on_press(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(press(f));
        self
    }

    pub fn on_remove(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.remove = Some(press(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let pressable = self.press.is_some();
        let ink = if self.selected || pressable { t.c.ink } else { t.c.ink_muted };
        let mut c = row().key(self.key.clone()).h(HEIGHT).pl(t.space.at(if self.icon.is_some() { 2.0 } else { 2.5 })).pr(t.space.at(if self.remove.is_some() { 0.5 } else { 2.5 })).gap(t.space.at(1.0)).center_y().max_w(self.max_w).shrink().min_w(0.0);
        c = match &self.press {
            Some(f) => {
                let live = !self.disabled;
                let mut c = c.role("button").label(self.words.clone()).focusable(live).checked(self.selected).radius(HEIGHT / 2.0);
                c = paint_material(c, &t.mat.chip, self.look, self.disabled, self.selected);
                if live {
                    let f = f.clone();
                    c = c.on_press(move |s: &mut S| f(s));
                } else {
                    c = c.disabled(true);
                }
                c
            }
            None => paint(c.role("note").label(self.words.clone()).radius(t.radius.sm), &Recipe::new().fill(t.c.selected)),
        };
        if let Some(i) = self.icon {
            c = c.child(glyph(i, t.control.icon_sm, ink));
        }
        c = c.child(text(self.words.clone()).font(tabular(t.ty.sm)).color(ink).truncate().min_w(0.0).a11y_hidden());
        if let Some(f) = self.remove.filter(|_| !self.disabled) {
            let x = row()
                .key(format!("{}-remove", self.key))
                .role("button")
                .label(format!("Remove {}", self.words))
                .focusable(true)
                .size(18.0, 18.0)
                .center()
                .radius(9.0)
                .hover(t.c.hover)
                .focus_ring(t.control.ring, t.control.ring_offset, t.c.focus)
                .on_press(move |s: &mut S| f(s))
                .child(glyph(&icons::CLOSE, t.control.icon_sm, ink));
            c = c.child(x);
        }
        c
    }
}
