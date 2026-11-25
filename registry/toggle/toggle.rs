//! rune-ui registry: toggle 0.5.0. Copied into your app by `rune-ui add toggle`; yours to change.
//!
//! A two-state button (bold on or off). Quiet until hovered; on, it takes the selected wash and the first ink. It says `pressed` to
//! assistive technology through `checked`. Icon-only toggles take their label as name and tooltip.
//!
//! ```ignore
//! Toggle::new("bold", "Bold", s.bold).icon_only(&icons::BOLD).on_toggle(|s: &mut App| s.bold = !s.bold).build(t)
//! ```

use rune::{El, Icon, row, text};

use super::base::{Press, Shape, Size, SizeExt, State, Theme, glyph, hit_area, paint_state, press, recipes, states};

pub struct Toggle<S: 'static> {
    key: String,
    label: String,
    on: bool,
    icon: Option<&'static Icon>,
    icon_only: bool,
    outline: bool,
    size: Size,
    disabled: bool,
    pinned: Option<State>,
    press: Option<Press<S>>,
}

impl<S: 'static> Toggle<S> {
    pub fn new(key: impl Into<String>, label: impl Into<String>, on: bool) -> Toggle<S> {
        Toggle { key: key.into(), label: label.into(), on, icon: None, icon_only: false, outline: false, size: Size::Md, disabled: false, pinned: None, press: None }
    }

    pub fn icon(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self
    }

    pub fn icon_only(mut self, i: &'static Icon) -> Self {
        self.icon = Some(i);
        self.icon_only = true;
        self
    }

    /// shadcn's outline variant.
    pub fn outline(mut self) -> Self {
        self.outline = true;
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn state(mut self, s: State) -> Self {
        self.pinned = Some(s);
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(press(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let parts = recipes::toggle(t, self.outline);
        let st = states(self.pinned, self.disabled, self.on);
        let h = self.size.height(t);
        let ink = parts.label.resolve(st);
        let mut face = row().key(self.key.clone()).role("button").label(self.label.clone()).checked(self.on).focusable(!self.disabled).h(h).gap(t.metrics.gap).center().radius(self.size.radius(t)).round_as(t);
        face = if self.icon_only { face.w(h).help(self.label.clone()) } else { face.px(self.size.pad(t) - if self.icon.is_some() { 1.0 } else { 0.0 }) };
        face = paint_state(face, &parts.root, st);
        if let Some(i) = self.icon {
            face = face.child(glyph(i, if self.icon_only { t.control.icon_lg } else { self.size.icon(t) }, parts.icon.resolve(st)));
        }
        if !self.icon_only {
            face = face.child(text(self.label.clone()).font(self.size.text(t, t.ty.medium)).color(ink).a11y_hidden());
        }
        if self.disabled {
            face = face.disabled(true);
        }
        let press = self.press.filter(|_| !self.disabled);
        if let Some(f) = press.clone() {
            face = face.on_press(move |s: &mut S| f(s));
        }
        let x = if self.icon_only { t.control.slop(h) } else { 0.0 };
        hit_area(&self.key, face, x, t.control.slop(h), press)
    }
}
