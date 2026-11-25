//! rune-ui registry: toast 0.2.0. Copied into your app by `rune-ui add toast`; yours to change.
//!
//! A short message that comes and goes: the **floating** material, an optional meaning icon, a title and a line under it, an action and
//! a close button. It is a polite live region. [`Toaster`] is where they stack (bottom right, newest last), mounted into the overlay
//! layer; timing (how long each stays) is the app's.
//!
//! ```ignore
//! Toaster::new("toasts").toast(Toast::new("saved", "Saved").description("All changes are on disk.").build(t)).mount(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, Shadow, col, row, text};
use rune_ui_style::{ButtonVariant, Recipe, Size, weight};

use super::base::{Theme, Tone, glyph, paint};
use super::button::Button;
use super::icons;

pub struct Toast<S: 'static> {
    key: String,
    title: String,
    description: Option<String>,
    tone: Option<Tone>,
    action: Option<(String, Rc<dyn Fn(&mut S)>)>,
    close: Option<Rc<dyn Fn(&mut S)>>,
}

impl<S: 'static> Toast<S> {
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Toast<S> {
        Toast { key: key.into(), title: title.into(), description: None, tone: None, action: None, close: None }
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// A meaning's icon before the words.
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
        self
    }

    pub fn action(mut self, words: impl Into<String>, f: impl Fn(&mut S) + 'static) -> Self {
        self.action = Some((words.into(), Rc::new(f)));
        self
    }

    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.close = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let mut r = paint(row().key(self.key.clone()).role("status").label(self.title.clone()).w(360.0).padding(t.space.at(3.5)).gap(t.space.at(3.0)).items_start().radius(t.radius.lg).continuous_corners(), &Recipe::new().fill(t.c.floating.bg).shadows(&t.elevation.md).shadow(Shadow { spread: 1.0, color: t.c.line, ..Shadow::default() })).live_region(rune::LiveRegion::Polite);
        if let Some(tone) = self.tone {
            let icon = match tone {
                Tone::Success => &icons::CIRCLE_CHECK,
                Tone::Warning => &icons::WARNING,
                Tone::Danger => &icons::CIRCLE_X,
                _ => &icons::INFO,
            };
            r = r.child(glyph(icon, t.control.icon_lg, tone.colors(t).2).mt(1.0));
        }
        let mut words = col().grow(1.0).shrink().min_w(0.0).gap(t.space.at(0.5)).child(text(self.title).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).wrap().min_w(0.0));
        if let Some(d) = self.description {
            words = words.child(text(d).font(t.ty.sm).color(t.ink_muted()).wrap().min_w(0.0));
        }
        r = r.child(words);
        if let Some((w, f)) = self.action {
            r = r.child(Button::<S>::new(format!("{}-action", self.key), w).variant(ButtonVariant::Secondary).size(Size::Sm).on_press(move |s: &mut S| f(s)).build(t));
        }
        if let Some(f) = self.close {
            r = r.child(Button::<S>::new(format!("{}-close", self.key), "Dismiss").variant(ButtonVariant::Ghost).size(Size::Sm).icon_only(&icons::CLOSE).on_press(move |s: &mut S| f(s)).build(t));
        }
        r
    }
}

/// Where toasts stack: the bottom right of the window, newest last.
pub struct Toaster {
    key: String,
    toasts: Vec<El>,
}

impl Toaster {
    pub fn new(key: impl Into<String>) -> Toaster {
        Toaster { key: key.into(), toasts: Vec::new() }
    }

    pub fn toast(mut self, t: El) -> Self {
        self.toasts.push(t);
        self
    }

    /// Mounts the stack into the overlay layer (nothing while it is empty).
    pub fn mount(self, t: &Theme, layer: &mut Vec<El>) {
        if self.toasts.is_empty() {
            return;
        }
        layer.push(col().key(self.key).role("region").label("Notifications").absolute(0.0, 0.0).w_full().h_full().padding(t.space.at(4.0)).gap(t.space.at(2.0)).justify_end().items_end().children(self.toasts));
    }
}
