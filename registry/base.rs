//! rune-ui registry: base 0.1.0. What every component shares: the press type, sizes, the 40 point hit area, a glyph and a spinner.
//! Copied into your app by `rune-ui init`; yours to change.

use std::rc::Rc;

use rune::{Color, El, Icon, LoopSpec, TextStyle, el, icon, row};
pub use rune_ui_style::{Look, Theme};

use super::icons;

/// What a press does, shared so a component can hand it to the drawn control and to its hit area.
pub type Press<S> = Rc<dyn Fn(&mut S)>;

/// A press from a closure.
pub fn press<S: 'static>(f: impl Fn(&mut S) + 'static) -> Press<S> {
    Rc::new(f)
}

/// Three control sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Size {
    Sm,
    #[default]
    Md,
    Lg,
}

impl Size {
    pub const ALL: [Size; 3] = [Size::Sm, Size::Md, Size::Lg];

    pub fn height(self, t: &Theme) -> f32 {
        match self {
            Size::Sm => t.control.sm,
            Size::Md => t.control.md,
            Size::Lg => t.control.lg,
        }
    }

    /// Words at `weight` for a control of this size.
    pub fn text(self, t: &Theme, weight: f32) -> TextStyle {
        let base = match self {
            Size::Sm => t.ty.sm,
            Size::Md => t.ty.base,
            Size::Lg => t.ty.md,
        };
        TextStyle { weight, ..base }
    }

    pub fn icon(self, t: &Theme) -> f32 {
        match self {
            Size::Sm => t.control.icon_sm,
            Size::Md => t.control.icon,
            Size::Lg => t.control.icon_lg,
        }
    }

    /// Side padding: words get this, the icon's side 2 points less (optical alignment).
    pub fn pad(self, t: &Theme) -> f32 {
        match self {
            Size::Sm => t.space.at(2.5),
            Size::Md => t.space.at(3.0),
            Size::Lg => t.space.at(4.0),
        }
    }

    /// The corner radius, kept under the share of the height that keeps it a rounded rectangle.
    pub fn radius(self, t: &Theme) -> f32 {
        rune_ui_style::Radius::fit(if self == Size::Lg { t.radius.lg } else { t.radius.md }, self.height(t))
    }

    pub fn word(self) -> &'static str {
        match self {
            Size::Sm => "Small",
            Size::Md => "Medium",
            Size::Lg => "Large",
        }
    }
}

/// The hit area round a drawn control: a wrapper as much larger as the control is short of the theme's hit size on each axis,
/// pulled back by the same negative margin, so layout keeps the control's own size and the pointer gets the larger target. It is
/// keyed `hit(<key>)`, not focusable and silent to assistive technology; the control keeps its key, name, focus ring and press.
/// Neighbours must sit twice `x` apart or two hit areas overlap.
pub fn hit_area<S: 'static>(key: &str, face: El, x: f32, y: f32, on_press: Option<Press<S>>) -> El {
    if x <= 0.0 && y <= 0.0 {
        return face;
    }
    let mut w = row().key(format!("hit({key})")).mx(-x).my(-y).px(x).py(y).center().child(face);
    if let Some(f) = on_press {
        w = w.on_press(move |s: &mut S| f(s));
    }
    w
}

/// An icon in one colour, hidden from assistive technology (the control it sits in has the name).
pub fn glyph(i: &'static Icon, size: f32, color: Color) -> El {
    icon(i, size).color(color).a11y_hidden()
}

/// Work in progress: a faint ring with a turning quarter. The one thing that loops.
pub fn spinner(size: f32, color: Color, label: &str) -> El {
    let track = icon(&icons::RING, size).absolute(0.0, 0.0).color(color.alpha(0.25));
    let arc = icon(&icons::ARC, size).absolute(0.0, 0.0).color(color).looping(LoopSpec::rotation(1.0));
    el().size(size, size).role("progressbar").label(label.to_owned()).child(track).child(arc)
}
