//! rune-ui registry: switch 0.1.0. Copied into your app by `rune-ui add switch`; yours to change.
//!
//! On or off, at once. The track is a **well** while off and fills with **ink** when on; the thumb is a small floating disc (the
//! **thumb** material off, the ink's own words colour on). Radii are concentric: the thumb's is the track's minus the padding. With
//! words, the whole line is the control (its hit area is the line); alone, the switch tops up to the hit size with an invisible
//! margin. The flip lands in the frame it was pressed: a frequent interaction answers at once.
//!
//! ```ignore
//! Switch::new("sync", "Sync", s.sync).label("Sync in the background").on_toggle(|s: &mut App| s.sync = !s.sync).build(t)
//! ```

use rune::{Color, El, Shadow, col, el, row, text};
use rune_ui_style::{Recipe, paint, paint_material, resolve};

use super::base::{Look, Press, Theme, hit_area, press};

const TRACK: (f32, f32) = (36.0, 22.0);
const PAD: f32 = 3.0;

pub struct Switch<S: 'static> {
    key: String,
    name: String,
    on: bool,
    label: Option<String>,
    help: Option<String>,
    disabled: bool,
    look: Look,
    press: Option<Press<S>>,
}

impl<S: 'static> Switch<S> {
    /// `name` is what it switches, for assistive technology (and its words, unless [`Switch::label`] gives others).
    pub fn new(key: impl Into<String>, name: impl Into<String>, on: bool) -> Switch<S> {
        Switch { key: key.into(), name: name.into(), on, label: None, help: None, disabled: false, look: Look::Live, press: None }
    }

    /// Words at its left; the switch sits at the line's end.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }

    pub fn help(mut self, h: impl Into<String>) -> Self {
        self.help = Some(h.into());
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.press = Some(press(f));
        self
    }

    fn track(&self, t: &Theme) -> El {
        let d = TRACK.1 - 2.0 * PAD;
        let thumb = if self.on {
            // On the ink, the thumb is the ink's words colour: white on a dark ink, near black on a light one.
            paint(el().size(d, d).radius(d / 2.0), &Recipe::new().fill(t.c.on_ink).shadow(Shadow { y: 1.0, blur: 2.0, color: Color::BLACK.alpha(0.22), ..Shadow::default() }))
        } else {
            paint(el().size(d, d).radius(d / 2.0), &t.mat.thumb.rest)
        }
        .a11y_hidden();
        let m = if self.on { &t.mat.ink } else { &t.mat.well };
        // The track carries the material's depth but not its press scale (the line or the hit area takes the press).
        let recipe = resolve(m, Look::Live, false, false).shadow_none();
        let track = paint(row().size(TRACK.0, TRACK.1).padding(PAD).center_y().radius(TRACK.1 / 2.0), &recipe).child(thumb).a11y_hidden();
        if self.on { track.justify_end() } else { track.justify_start() }
    }

    pub fn build(self, t: &Theme) -> El {
        let live = !self.disabled;
        let track = self.track(t);
        let mut e = row().key(self.key.clone()).role("switch").label(self.label.clone().unwrap_or_else(|| self.name.clone())).checked(self.on).focusable(live).center_y().radius(t.radius.md);
        if let Some(h) = &self.help {
            e = e.description(h.clone());
        }
        let has_words = self.label.is_some();
        e = if has_words {
            let mut words = col().grow(1.0).shrink().min_w(0.0).gap(1.0).child(text(self.label.clone().unwrap_or_default()).font(t.ty.base).color(t.c.ink).wrap().min_w(0.0).a11y_hidden());
            if let Some(h) = &self.help {
                words = words.child(text(h.clone()).font(t.ty.sm).color(t.c.ink_faint).wrap().min_w(0.0).a11y_hidden());
            }
            e.w_full().min_h(t.control.md).px(t.space.at(1.0)).mx(-t.space.at(1.0)).gap(t.space.at(3.0)).child(words).child(track)
        } else {
            e.child(track)
        };
        // A line washes like a row; a bare switch only rings. Neither sinks: the thumb moving is the answer.
        let mut line = t.mat.ghost.clone().pressed(|r| if has_words { r.wash(t.c.hover) } else { r });
        if !has_words {
            line = line.hover(|r| r);
        }
        e = paint_material(e, &line, self.look, self.disabled, false);
        if !live {
            e = e.disabled(true);
        }
        let press = self.press.filter(|_| live);
        if let Some(f) = press.clone() {
            e = e.on_press(move |s: &mut S| f(s));
        }
        if has_words { e } else { hit_area(&self.key, e, t.control.slop(TRACK.0), t.control.slop(TRACK.1), press) }
    }
}
