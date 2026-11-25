//! rune-ui registry: select 0.1.0. Copied into your app by `rune-ui add select`; yours to change.
//!
//! One choice of a few. Behaviour is rune-kit's `Select` (layer 1: keys, focus, typeahead, dismissal, the listbox placed beside its
//! trigger and kept in the window); the look is rune-ui's. The trigger is a **well** (it holds a value) with the choice and a chevron;
//! the list is the **floating** material from `Theme::kit()`, with a check on the current row. The app owns whether it is open.
//!
//! The trigger keeps rune-kit's own key (`<key>-trigger`): the list is anchored to it. Find the select by its wrapper, `select-<key>`.
//!
//! ```ignore
//! Select::new("level", "Starting level", &s.levels).current(&s.level).open(s.open == Some("level"))
//!     .on_toggle(|s: &mut App| s.toggle("level")).on_close(|s: &mut App| s.open = None)
//!     .on_choose(|s: &mut App, v: &str| s.level = v.to_owned()).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{Color, El, Tx, row};
use rune_ui_style::paint;

use super::base::{Look, Press, Size, Theme, press};

type Choose<S> = Rc<dyn Fn(&mut S, &str)>;

pub struct Select<'a, S: 'static> {
    key: String,
    label: String,
    options: &'a [String],
    current: Option<String>,
    placeholder: String,
    width: f32,
    size: Size,
    disabled: bool,
    error: bool,
    look: Look,
    open: bool,
    toggle: Option<Press<S>>,
    close: Option<Press<S>>,
    choose: Option<Choose<S>>,
}

impl<'a, S: 'static> Select<'a, S> {
    /// `label` is its name for assistive technology.
    pub fn new(key: impl Into<String>, label: impl Into<String>, options: &'a [String]) -> Select<'a, S> {
        Select { key: key.into(), label: label.into(), options, current: None, placeholder: "Choose…".to_owned(), width: 200.0, size: Size::Md, disabled: false, error: false, look: Look::Live, open: false, toggle: None, close: None, choose: None }
    }

    pub fn current(mut self, c: &str) -> Self {
        self.current = Some(c.to_owned());
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn error(mut self, on: bool) -> Self {
        self.error = on;
        self
    }

    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }

    /// Whether the list is showing (the app keeps this).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// A press on the trigger.
    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(press(f));
        self
    }

    /// Escape or a press outside the open list.
    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.close = Some(press(f));
        self
    }

    /// An option chosen (its words).
    pub fn on_choose(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.choose = Some(Rc::new(f));
        self
    }

    /// The trigger in place; the open list goes into `layer`, which the view mounts over everything (`rune_kit::root`).
    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let mut kit = t.kit().clone();
        kit.ty.ui_compact = self.size.text(t, t.ty.regular);
        // The well draws its own depth: rune-kit's trigger shadow would sit on top of it.
        kit.elevation.button_neutral = Vec::new();
        let open = self.open && !self.disabled;
        let mut select = rune_kit::Select::new(self.key.clone(), self.options).current(self.current.as_deref()).placeholder(self.placeholder).label(self.label).width(self.width).open(open).invalid(self.error).disabled(self.disabled);
        if !self.disabled {
            if let Some(f) = self.toggle {
                select = select.on_toggle(move |s: &mut S| f(s));
            }
            if let Some(f) = self.close {
                select = select.on_close(move |s: &mut S| f(s));
            }
            if let Some(f) = self.choose {
                select = select.on_choose(move |s: &mut S, v: &str, _: &mut Tx| f(s, v));
            }
        }
        let h = self.size.height(t);
        let m = &t.mat.well;
        // rune-kit dims a disabled trigger itself.
        let mut recipe = if self.disabled { m.rest.clone() } else if self.look == Look::Focused || open { m.focused.clone() } else { m.rest.clone() };
        if self.error && !self.disabled {
            recipe = recipe.at_rest().ring(t.control.ring, 0.0, t.c.danger.dot.alpha(0.75));
        }
        let trigger = select.build(&kit, layer).w(self.width).h(h).pl(self.size.pad(t) - 4.0).pr(t.space.at(1.5)).py(0.0).gap(t.space.at(1.5)).radius(self.size.radius(t)).continuous_corners().justify_between().border(0.0, Color::TRANSPARENT).hover(Color::TRANSPARENT);
        let mut trigger = paint(trigger, &recipe);
        if self.look == Look::Live && !self.disabled && !self.error {
            if let Some((w, o, c)) = m.live().focus_ring {
                trigger = trigger.focus_ring(w, o, c);
            }
        }
        row().key(format!("select-{}", self.key)).child(trigger)
    }
}
