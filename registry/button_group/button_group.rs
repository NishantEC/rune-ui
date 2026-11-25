//! rune-ui registry: button_group 0.3.0. Copied into your app by `rune-ui add button_group`; yours to change. Needs: button's recipes.
//!
//! Related actions joined into one object: a single raised (or outline) surface divided by hairlines, each segment washing on hover and
//! answering a press. One object, not a row of separate buttons, so the group reads as one choice of action.
//!
//! ```ignore
//! ButtonGroup::new("history", "History").item("undo", "Undo", None, |s: &mut App| s.undo()).item("redo", "Redo", None, |s: &mut App| s.redo()).build(t)
//! ```

use rune::{El, Icon, row, text};

use super::base::{ButtonVariant, Press, Size, SizeExt, State, States, Theme, glyph, paint, paint_state, press, recipes, resolve};

struct Segment<S: 'static> {
    key: String,
    label: String,
    icon: Option<&'static Icon>,
    press: Press<S>,
}

pub struct ButtonGroup<S: 'static> {
    key: String,
    name: String,
    variant: ButtonVariant,
    size: Size,
    disabled: bool,
    pinned: Option<(usize, State)>,
    items: Vec<Segment<S>>,
}

impl<S: 'static> ButtonGroup<S> {
    /// `name` names the group for assistive technology.
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> ButtonGroup<S> {
        ButtonGroup { key: key.into(), name: name.into(), variant: ButtonVariant::Secondary, size: Size::Md, disabled: false, pinned: None, items: Vec::new() }
    }

    /// One segment. With an icon and no visible words, pass the words anyway: they become its name.
    pub fn item(mut self, key: impl Into<String>, label: impl Into<String>, icon: Option<&'static Icon>, f: impl Fn(&mut S) + 'static) -> Self {
        self.items.push(Segment { key: key.into(), label: label.into(), icon, press: press(f) });
        self
    }

    /// Secondary (raised, the default) or Outline.
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
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

    /// Pins a look on segment `i` (galleries).
    pub fn state(mut self, i: usize, s: State) -> Self {
        self.pinned = Some((i, s));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let parts = recipes::button(t, if self.variant == ButtonVariant::Outline { ButtonVariant::Outline } else { ButtonVariant::Secondary });
        let h = self.size.height(t);
        let r = self.size.radius(t);
        let base = States { disabled: self.disabled, ..States::default() };
        // The surface without its press scale: segments answer a press, the group does not move.
        let surface = resolve(&parts.root, base).without(rune_ui_style::LayerKind::Scale);
        let mut g = paint(row().key(self.key.clone()).role("group").label(self.name.clone()).h(h).radius(r).continuous_corners().center_y(), &surface);
        let n = self.items.len();
        for (i, seg) in self.items.into_iter().enumerate() {
            if i > 0 {
                g = g.child(rune::el().w(1.0).h(h - 2.0 * t.space.at(1.5)).fill(t.c.line).a11y_hidden());
            }
            let mut st = States { disabled: self.disabled, ..States::default() };
            if let Some((at, s)) = self.pinned
                && at == i
            {
                st.set(s, true);
            }
            let corners = [if i == 0 { r } else { 0.0 }, if i + 1 == n { r } else { 0.0 }, if i + 1 == n { r } else { 0.0 }, if i == 0 { r } else { 0.0 }];
            let ink = parts.label.resolve(st);
            let mut face = row().key(seg.key.clone()).role("button").label(seg.label.clone()).focusable(!self.disabled).h(h).px(self.size.pad(t)).gap(t.metrics.gap).center().corners(corners);
            face = paint_state(face, &t.mat.row, st);
            if let Some(ic) = seg.icon {
                face = face.child(glyph(ic, self.size.icon(t), ink));
            }
            face = face.child(text(seg.label).font(self.size.text(t, t.ty.medium)).color(ink).a11y_hidden());
            if !self.disabled {
                let f = seg.press.clone();
                face = face.on_press(move |s: &mut S| f(s));
            } else {
                face = face.disabled(true);
            }
            g = g.child(face);
        }
        g
    }
}
