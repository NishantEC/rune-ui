//! rune-ui registry: button 0.1.0. Copied into your app by `rune-ui add button`; yours to change.
//!
//! Four variants on the materials: **primary** (ink: a view's one call to action), **secondary** (raised), **ghost** (nothing until
//! hovered) and **destructive** (the ink recipe in the danger colour). Three sizes. Words, an icon and words, words and a trailing
//! icon, or an icon alone (its name becomes its tooltip). A press sinks it (the material's press scale); raised and ink buttons do not
//! change colour on hover, by design; a ghost washes. Loading keeps the width and puts a spinner where the icon was; a disabled or
//! loading button neither sinks nor runs.
//!
//! ```ignore
//! Button::primary("save", "Save").icon(&icons::CHECK).on_press(|s: &mut App| s.save()).build(t)
//! Button::ghost("more", "More").icon_only(&icons::PLUS).size(Size::Sm).build(t)
//! ```

use rune::{El, Icon, row, text};
use rune_ui_style::{Material, paint, paint_material, resolve};

use super::base::{Look, Press, Size, Theme, glyph, hit_area, press, spinner};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Variant {
    Primary,
    #[default]
    Secondary,
    Ghost,
    Destructive,
}

impl Variant {
    pub const ALL: [Variant; 4] = [Variant::Primary, Variant::Secondary, Variant::Ghost, Variant::Destructive];

    pub fn word(self) -> &'static str {
        match self {
            Variant::Primary => "Primary",
            Variant::Secondary => "Secondary",
            Variant::Ghost => "Ghost",
            Variant::Destructive => "Destructive",
        }
    }

    pub fn material(self, t: &Theme) -> &Material {
        match self {
            Variant::Primary => &t.mat.ink,
            Variant::Secondary => &t.mat.raised,
            Variant::Ghost => &t.mat.ghost,
            Variant::Destructive => &t.mat.danger,
        }
    }
}

pub struct Button<S: 'static> {
    key: String,
    label: String,
    variant: Variant,
    size: Size,
    icon: Option<&'static Icon>,
    trailing: Option<&'static Icon>,
    icon_only: bool,
    loading: bool,
    disabled: bool,
    selected: Option<bool>,
    look: Look,
    help: Option<String>,
    name: Option<String>,
    full: bool,
    press: Option<Press<S>>,
}

impl<S: 'static> Button<S> {
    pub fn new(variant: Variant, key: impl Into<String>, label: impl Into<String>) -> Button<S> {
        Button { key: key.into(), label: label.into(), variant, size: Size::Md, icon: None, trailing: None, icon_only: false, loading: false, disabled: false, selected: None, look: Look::Live, help: None, name: None, full: false, press: None }
    }

    pub fn primary(key: impl Into<String>, label: impl Into<String>) -> Button<S> {
        Button::new(Variant::Primary, key, label)
    }

    pub fn secondary(key: impl Into<String>, label: impl Into<String>) -> Button<S> {
        Button::new(Variant::Secondary, key, label)
    }

    pub fn ghost(key: impl Into<String>, label: impl Into<String>) -> Button<S> {
        Button::new(Variant::Ghost, key, label)
    }

    pub fn destructive(key: impl Into<String>, label: impl Into<String>) -> Button<S> {
        Button::new(Variant::Destructive, key, label)
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// An icon before the words.
    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self
    }

    /// An icon after the words (a chevron, an arrow).
    pub fn trailing_icon(mut self, i: &'static Icon) -> Self {
        self.trailing = Some(i);
        self
    }

    /// The icon alone, square; the label is its name and its tooltip.
    pub fn icon_only(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self.icon_only = true;
        self
    }

    pub fn loading(mut self, on: bool) -> Self {
        self.loading = on;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    /// A toggle that stays down while on (it says so to assistive technology too).
    pub fn selected(mut self, on: bool) -> Self {
        self.selected = Some(on);
        self
    }

    /// Pins a live state (galleries, tests).
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    pub fn help(mut self, h: impl Into<String>) -> Self {
        self.help = Some(h.into());
        self
    }

    /// Its name for assistive technology when the words alone are ambiguous ("Delete draft" for "Delete").
    pub fn name(mut self, n: impl Into<String>) -> Self {
        self.name = Some(n.into());
        self
    }

    /// As wide as what holds it.
    pub fn full(mut self) -> Self {
        self.full = true;
        self
    }

    pub fn on_press(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(press(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let h = self.size.height(t);
        let on = self.selected == Some(true);
        let m = self.variant.material(t);
        let ink = match self.variant {
            Variant::Primary => t.c.on_ink,
            Variant::Destructive => t.c.on_danger,
            Variant::Secondary => t.c.ink,
            Variant::Ghost if on => t.c.ink,
            Variant::Ghost => t.c.ink_muted,
        };
        let strong = matches!(self.variant, Variant::Primary | Variant::Destructive);
        let words = self.size.text(t, if strong || on { t.ty.medium } else { t.ty.regular });
        let live = !self.disabled && !self.loading;
        let mut face = row()
            .key(self.key.clone())
            .role("button")
            .label(self.name.clone().unwrap_or_else(|| self.label.clone()))
            .focusable(live)
            .h(h)
            .gap(t.space.at(if self.size == Size::Sm { 1.0 } else { 1.5 }))
            .center()
            .radius(self.size.radius(t))
            .continuous_corners();
        if let Some(v) = self.selected {
            face = face.checked(v);
        }
        if let Some(help) = self.help.clone().or_else(|| self.icon_only.then(|| self.label.clone())) {
            face = face.help(help);
        }
        // Optical padding: the icon's side is 2 points tighter than the words' side.
        let pad = self.size.pad(t);
        face = if self.icon_only { face.w(h) } else { face.pl(if self.icon.is_some() || self.loading { pad - 2.0 } else { pad }).pr(if self.trailing.is_some() { pad - 2.0 } else { pad }) };
        if self.full {
            face = face.w_full();
        }
        face = if live || self.disabled {
            paint_material(face, m, self.look, self.disabled, on)
        } else {
            // Loading: the rest look, nothing live.
            paint(face, &resolve(m, Look::Live, false, on))
        };
        let size = if self.icon_only { t.control.icon_lg } else { self.size.icon(t) };
        if self.loading {
            face = face.busy(true).child(spinner(size, ink, &self.label));
        } else if let Some(i) = self.icon {
            face = face.child(glyph(i, size, ink));
        }
        if !self.icon_only {
            face = face.child(text(self.label.clone()).font(words).color(ink).truncate().min_w(0.0).a11y_hidden());
        }
        if let Some(i) = self.trailing {
            face = face.child(glyph(i, size, ink));
        }
        if !live {
            face = face.disabled(true);
        }
        let press = self.press.filter(|_| live);
        if let Some(f) = press.clone() {
            face = face.on_press(move |s: &mut S| f(s));
        }
        let x = if self.icon_only { t.control.slop(h) } else { 0.0 };
        let hit = hit_area(&self.key, face, x, t.control.slop(h), press);
        if self.full { hit.w_full() } else { hit }
    }
}
