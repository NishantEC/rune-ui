//! Component recipes: the variants a component offers, as enums, and the [`Parts`] each one paints with. A compound variant is a
//! `match` arm here, never an `if` in a component. Components ask `recipes::button(t, variant)` and change the result through their
//! `.recipe(|parts| …)` escape hatch if they must.

use rune::Color;

use crate::recipe::{Delta, Material, Parts, Recipe, State, Tint};
use crate::theme::Theme;

/// shadcn/ui's button variants.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// The ink: a view's one primary action.
    #[default]
    Default,
    /// Raised.
    Secondary,
    /// A quiet control with its rim drawn inside.
    Outline,
    /// Nothing until hovered.
    Ghost,
    /// The danger colour, for what cannot be taken back.
    Destructive,
    /// Words in the accent colour, no surface.
    Link,
}

impl ButtonVariant {
    pub const ALL: [ButtonVariant; 6] = [ButtonVariant::Default, ButtonVariant::Secondary, ButtonVariant::Outline, ButtonVariant::Ghost, ButtonVariant::Destructive, ButtonVariant::Link];

    pub fn word(self) -> &'static str {
        match self {
            ButtonVariant::Default => "Default",
            ButtonVariant::Secondary => "Secondary",
            ButtonVariant::Outline => "Outline",
            ButtonVariant::Ghost => "Ghost",
            ButtonVariant::Destructive => "Destructive",
            ButtonVariant::Link => "Link",
        }
    }
}

/// Control sizes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Size {
    Sm,
    #[default]
    Md,
    Lg,
}

impl Size {
    pub const ALL: [Size; 3] = [Size::Sm, Size::Md, Size::Lg];

    pub fn word(self) -> &'static str {
        match self {
            Size::Sm => "Small",
            Size::Md => "Medium",
            Size::Lg => "Large",
        }
    }
}

/// A button's parts in a variant.
pub fn button(t: &Theme, v: ButtonVariant) -> Parts {
    let c = &t.c;
    let (root, ink) = match v {
        ButtonVariant::Default => (t.mat.ink.clone(), c.primary.fg),
        ButtonVariant::Secondary => (t.mat.raised.clone(), c.secondary.fg),
        ButtonVariant::Outline => (t.mat.outline.clone(), c.page.fg),
        ButtonVariant::Ghost => (t.mat.ghost.clone(), c.page.fg),
        ButtonVariant::Destructive => (t.mat.danger.clone(), c.danger.on_solid),
        ButtonVariant::Link => (Material::new(Recipe::new()).on(State::FocusVisible, Delta::new().ring(t.control.ring, t.control.ring_offset, c.ring)).on(State::Disabled, Delta::new().opacity(t.control.disabled)).on(State::Pressed, Delta::new().opacity(0.7)), c.accent.fg),
    };
    Parts { root, label: Tint::new(ink), icon: Tint::new(ink) }
}

/// A toggle (a button that stays on): quiet until hovered, the selected wash when on.
pub fn toggle(t: &Theme, outline: bool) -> Parts {
    let root = if outline { t.mat.outline.clone() } else { t.mat.ghost.clone() };
    Parts { root, label: Tint::new(t.c.muted.fg).selected(t.c.page.fg), icon: Tint::new(t.c.muted.fg).selected(t.c.page.fg) }
}

/// A text field: the well, the text in the first ink, a placeholder in the third.
pub fn field(t: &Theme) -> Parts {
    Parts { root: t.mat.well.clone(), label: Tint::new(t.c.input.fg), icon: Tint::new(t.c.subtle.fg).invalid(t.c.danger.fg) }
}

/// A row in a list, a menu-like surface or a sidebar: the row material, the second ink, the first when selected.
pub fn row(t: &Theme) -> Parts {
    Parts { root: t.mat.row.clone(), label: Tint::new(t.c.muted.fg).selected(t.c.page.fg), icon: Tint::new(t.c.muted.fg).selected(t.c.page.fg) }
}

/// A check mark's box (checkbox) or a radio's dot holder: a small well off, the ink on.
pub fn check(t: &Theme, on: bool) -> Material {
    if on { t.mat.ink.clone().on(State::Pressed, Delta::new().shadow_none()) } else { t.mat.well.clone() }
}

/// A meaning's colours by name, for alerts, badges and toasts.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Tone {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

impl Tone {
    pub const ALL: [Tone; 5] = [Tone::Neutral, Tone::Info, Tone::Success, Tone::Warning, Tone::Danger];

    pub fn word(self) -> &'static str {
        match self {
            Tone::Neutral => "Neutral",
            Tone::Info => "Info",
            Tone::Success => "Success",
            Tone::Warning => "Warning",
            Tone::Danger => "Danger",
        }
    }

    /// (wash, words, solid, words on solid).
    pub fn colors(self, t: &Theme) -> (Color, Color, Color, Color) {
        let tone = match self {
            Tone::Neutral => return (t.c.muted.bg, t.c.muted.fg, t.c.primary.bg, t.c.primary.fg),
            Tone::Info => t.c.info,
            Tone::Success => t.c.success,
            Tone::Warning => t.c.warning,
            Tone::Danger => t.c.danger,
        };
        (tone.bg, tone.fg, tone.solid, tone.on_solid)
    }
}
