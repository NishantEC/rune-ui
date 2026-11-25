//! rune-ui registry: carousel 0.5.1. Copied into your app by `rune-ui add carousel`; yours to change. Needs: button.
//!
//! Slides one at a time in a rounded viewport, with outline arrow buttons beside it and a row of dots under it. The track slides to
//! the slide chosen (Rune turns the motion off under Reduce Motion). The viewport is one Tab stop; Left and Right step, Home and End
//! jump. Arrows are off at the ends unless it loops. Slides off screen are hidden from assistive technology and cannot take focus.
//! Nothing moves around it: the viewport has a fixed size and every dot keeps its size.
//!
//! ```ignore
//! Carousel::new("shots", "Screenshots", s.shot).slide(a).slide(b).size(480.0, 270.0).on_index(|s: &mut App, i| s.shot = i).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, Tx, col, el, row};
use rune_ui_style::{ButtonVariant, Size};

use super::base::{Choose, Shape, Theme, hit_area};
use super::button::Button;
use super::icons;

pub struct Carousel<S: 'static> {
    key: String,
    name: String,
    index: usize,
    slides: Vec<El>,
    w: f32,
    h: f32,
    looped: bool,
    on_index: Option<Choose<S>>,
}

impl<S: 'static> Carousel<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, index: usize) -> Carousel<S> {
        Carousel { key: key.into(), name: name.into(), index, slides: Vec::new(), w: 360.0, h: 200.0, looped: false, on_index: None }
    }

    pub fn slide(mut self, e: El) -> Self {
        self.slides.push(e);
        self
    }

    /// The viewport's size (each slide's size).
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }

    /// From the last slide, Next goes to the first (and Previous the other way).
    pub fn looped(mut self, on: bool) -> Self {
        self.looped = on;
        self
    }

    pub fn on_index(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.on_index = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let k = self.key.clone();
        let n = self.slides.len().max(1);
        let at = self.index.min(n - 1);
        let looped = self.looped;
        let step = move |d: isize| -> Option<usize> {
            let to = at as isize + d;
            if looped { Some(to.rem_euclid(n as isize) as usize) } else { (0..n as isize).contains(&to).then_some(to as usize) }
        };
        let r = t.radius.xl;
        let mut track = row().key(format!("{k}-track")).h(self.h).offset(-(at as f32) * self.w, 0.0);
        for (i, s) in self.slides.into_iter().enumerate() {
            let mut slide = col().key(format!("{k}-slide-{i}")).role("group").label(format!("Slide {} of {n}", i + 1)).description("slide").size(self.w, self.h).child(s);
            if i != at {
                slide = slide.inert().a11y_hidden();
            }
            track = track.child(slide);
        }
        let mut view = col().key(format!("{k}-viewport")).role("region").label(self.name.clone()).description("carousel").a11y_value(format!("Slide {} of {n}", at + 1)).focusable(true).size(self.w, self.h).radius(r).round_as(t).clip_rounded(r).focus_ring(t.control.ring, t.control.ring_offset, t.c.ring).child(track);
        if let Some(f) = self.on_index.clone() {
            view = view.on_key(move |s: &mut S, ev: &KeyEvent, _: &mut Tx| {
                let to = match ev.key {
                    Key::Left => step(-1),
                    Key::Right => step(1),
                    Key::Home => Some(0),
                    Key::End => Some(n - 1),
                    _ => return false,
                };
                if let Some(i) = to {
                    f(s, i);
                }
                true
            });
        }
        let arrow = |d: isize, name: &str, icon| {
            let mut b = Button::<S>::new(format!("{k}-{}", if d < 0 { "prev" } else { "next" }), name.to_owned()).variant(ButtonVariant::Outline).size(Size::Sm).icon_only(icon).disabled(step(d).is_none());
            if let (Some(f), Some(to)) = (self.on_index.clone(), step(d)) {
                b = b.on_press(move |s: &mut S| f(s, to));
            }
            b.build(t)
        };
        let stage = row().gap(t.space.at(3.0)).center_y().child(arrow(-1, "Previous slide", &icons::CHEVRON_LEFT)).child(view).child(arrow(1, "Next slide", &icons::CHEVRON_RIGHT));
        let mut dots = row().key(format!("{k}-dots")).role("group").label("Choose a slide").gap(t.space.at(2.0)).center();
        for i in 0..n {
            let on = i == at;
            let mut d = el().key(format!("{k}-dot-{i}")).role("button").label(format!("Slide {}", i + 1)).checked(on).focusable(true).size(8.0, 8.0).radius(4.0).fill(if on { t.c.primary.bg } else { t.ink_faint().alpha(0.35) }).focus_ring(t.control.ring, t.control.ring_offset, t.c.ring);
            let press = self.on_index.clone().map(|f| super::base::press(move |s: &mut S| f(s, i)));
            if let Some(p) = press.clone() {
                d = d.on_press(move |s: &mut S| p(s));
            }
            // Dots sit 16 points apart; their hit areas fill the gap without changing layout.
            dots = dots.child(hit_area(&format!("{k}-dot-{i}"), d, 4.0, 8.0, press));
        }
        col().key(k).items_center().gap(t.space.at(3.0)).child(stage).child(dots.gap(t.space.at(4.0)))
    }
}
