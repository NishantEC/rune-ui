//! rune-ui registry: checkbox 0.4.0. Copied into your app by `rune-ui add checkbox`; yours to change.
//!
//! On, off or mixed. Off is a small **well**; on or mixed it fills with **ink** and shows a check or a dash in the ink's words colour.
//! With words, the whole line is the control (it washes like a row and its hit area is the line); alone, it tops up to the hit size.
//!
//! ```ignore
//! Checkbox::new("terms", "Accept the terms", s.terms).on_toggle(|s: &mut App| s.terms = !s.terms).build(t)
//! ```

use rune::{El, row};

use super::base::{Press, Shape, State, States, Theme, glyph, hit_area, paint, paint_state, press, recipes, resolve, states, words};
use super::icons;

const BOX: f32 = 16.0;

pub struct Checkbox<S: 'static> {
    key: String,
    label: String,
    on: bool,
    mixed: bool,
    show_label: bool,
    help: Option<String>,
    invalid: bool,
    disabled: bool,
    pinned: Option<State>,
    press: Option<Press<S>>,
}

impl<S: 'static> Checkbox<S> {
    /// `label` is shown beside the box, and is its name.
    pub fn new(key: impl Into<String>, label: impl Into<String>, on: bool) -> Checkbox<S> {
        Checkbox { key: key.into(), label: label.into(), on, mixed: false, show_label: true, help: None, invalid: false, disabled: false, pinned: None, press: None }
    }

    pub fn mixed(mut self, on: bool) -> Self {
        self.mixed = on;
        self
    }

    /// The box alone (the label stays its name).
    pub fn hide_label(mut self) -> Self {
        self.show_label = false;
        self
    }

    pub fn help(mut self, h: impl Into<String>) -> Self {
        self.help = Some(h.into());
        self
    }

    pub fn invalid(mut self, on: bool) -> Self {
        self.invalid = on;
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
        let filled = self.on || self.mixed;
        let mut st = states(self.pinned, self.disabled, false);
        st.invalid = self.invalid;
        let box_states = States { invalid: self.invalid, focus_visible: st.focus_visible && !self.show_label, ..States::default() };
        let mut b = row().size(BOX, BOX).center().radius(4.5).round_as(t).a11y_hidden();
        b = paint(b, &resolve(&recipes::check(t, filled), box_states).without(rune_ui_style::LayerKind::Scale));
        if filled {
            b = b.child(glyph(if self.mixed { &icons::MINUS } else { &icons::CHECK }, 12.0, t.c.primary.fg));
        }
        let live = !self.disabled;
        let mut e = row().key(self.key.clone()).role("checkbox").label(self.label.clone()).focusable(live).gap(t.space.at(2.0)).radius(t.radius.md).round_as(t);
        e = if self.mixed { e.checked_mixed() } else { e.checked(self.on) };
        if let Some(h) = &self.help {
            e = e.description(h.clone());
        }
        let line = if self.show_label { t.mat.row.clone() } else { rune_ui_style::Material::new(rune_ui_style::Recipe::new()).on(State::FocusVisible, t.mat.ghost.focus.clone()) };
        e = if self.show_label { e.min_h(t.control.md).px(t.space.at(1.0)).ml(-t.space.at(1.0)).center_y().child(b).child(words(t, &self.label, self.help.as_deref())) } else { e.center().child(b) };
        e = paint_state(e, &line, States { invalid: false, ..st });
        if self.disabled {
            e = e.disabled(true);
        }
        let press = self.press.filter(|_| live);
        if let Some(f) = press.clone() {
            e = e.on_press(move |s: &mut S| f(s));
        }
        if self.show_label { e } else { hit_area(&self.key, e, t.control.slop(BOX), t.control.slop(BOX), press) }
    }
}
