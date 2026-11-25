//! rune-ui registry: slider 0.5.0. Copied into your app by `rune-ui add slider`; yours to change.
//!
//! A value in a range. The track is a **well**, the part up to the value is **ink**, the thumb a raised disc. Drag anywhere on it (a
//! press without moving jumps there), or use the arrow keys (Shift for ten steps) and Home/End. It reports its value, minimum and
//! maximum to assistive technology.
//!
//! ```ignore
//! Slider::new("volume", "Volume", s.volume).range(0.0, 100.0).step(1.0).on_change(|s: &mut App, v| s.volume = v).build(t)
//! ```

use std::rc::Rc;

use rune::{Drag, El, Key, KeyEvent, Phase, Tx, el, row};

use super::base::{Slide, State, States, Theme, paint, resolve, states};

const H: f32 = 20.0;
const TRACK: f32 = 6.0;
const THUMB: f32 = 16.0;

pub struct Slider<S: 'static> {
    key: String,
    name: String,
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    width: Option<f32>,
    disabled: bool,
    pinned: Option<State>,
    change: Option<Slide<S>>,
}

impl<S: 'static> Slider<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, value: f32) -> Slider<S> {
        Slider { key: key.into(), name: name.into(), value, min: 0.0, max: 100.0, step: 1.0, width: None, disabled: false, pinned: None, change: None }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max.max(min + f32::EPSILON);
        self
    }

    pub fn step(mut self, s: f32) -> Self {
        self.step = s.max(f32::EPSILON);
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
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

    pub fn on_change(mut self, f: impl Fn(&mut S, f32) + 'static) -> Self {
        self.change = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let (min, max, step) = (self.min, self.max, self.step);
        let snap = move |v: f32| (((v - min) / step).round() * step + min).clamp(min, max);
        let v = snap(self.value);
        let frac = (v - min) / (max - min);
        let st = states(self.pinned, self.disabled, false);
        let well = paint(row().w_full().h(TRACK).radius(TRACK / 2.0), &resolve(&t.mat.well, States::default()));
        let fill = paint(el().w_frac(frac).h(TRACK).radius(TRACK / 2.0), &resolve(&t.mat.ink, States::default()).shadow_none()).a11y_hidden();
        let thumb = paint(el().size(THUMB, THUMB).radius(THUMB / 2.0), &t.mat.thumb.base).a11y_hidden();
        // The thumb's centre travels the track's width less the thumb.
        let thumb = row().absolute(0.0, (H - THUMB) / 2.0).w_full().h(THUMB).child(el().w_frac(frac).h(THUMB)).child(rune::el().ml(-THUMB * frac).child(thumb));
        let mut e = row().key(self.key.clone()).role("slider").label(self.name.clone()).range_value(v as f64, min as f64, max as f64).focusable(!self.disabled).h(H).center_y().radius(H / 2.0).child(well.child(fill)).child(thumb);
        e = match self.width {
            Some(w) => e.w(w),
            None => e.w_full(),
        };
        // Focus rings the whole slider: live from Rune, or pinned for a gallery.
        let ring = t.mat.ghost.state(State::FocusVisible);
        e = if st.focus_visible && !self.disabled { paint(e, &ring) } else { e };
        if let Some((w, o, c)) = ring.ring_of() {
            e = e.focus_ring(w, o, c);
        }
        if self.disabled {
            return e.disabled(true).opacity(t.control.disabled);
        }
        if let Some(f) = self.change {
            let g = f.clone();
            e = e.on_drag(move |s: &mut S, d: &mut Drag, tx: &mut Tx| {
                tx.instant();
                if matches!(d.phase, Phase::Down | Phase::Move) {
                    g(s, snap(min + (d.x / d.w.max(1.0)).clamp(0.0, 1.0) * (max - min)));
                }
            });
            e = e.on_key(move |s: &mut S, k: &KeyEvent, _| {
                let big = if k.mods.shift { 10.0 } else { 1.0 };
                let to = match k.key {
                    Key::Right | Key::Up => v + step * big,
                    Key::Left | Key::Down => v - step * big,
                    Key::Home => min,
                    Key::End => max,
                    _ => return false,
                };
                f(s, snap(to));
                true
            });
        }
        e
    }
}
