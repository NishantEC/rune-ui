//! rune-ui registry: select 0.3.0. Copied into your app by `rune-ui add select`; yours to change.
//!
//! One choice of a few. Behaviour is rune-kit's `Select` (keys, typeahead, dismissal, a listbox placed beside its trigger and kept in
//! the window), handed our look through `Theme::to_kit()`. The trigger is a **well** (it holds a value) with the choice and a chevron;
//! the list is the **floating** material, with a check on the current row. The app owns whether it is open.
//!
//! The trigger keeps rune-kit's key (`<key>-trigger`): the list is anchored to it. The wrapper is keyed `<key>`.
//!
//! ```ignore
//! Select::new("level", "Starting level", &s.levels).current(&s.level).open(s.open == Some("level"))
//!     .on_toggle(|s: &mut App| s.toggle("level")).on_close(|s: &mut App| s.open = None)
//!     .on_choose(|s: &mut App, v: &str| s.level = v.to_owned()).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{Color, El, Tx, row};

use super::base::{Pick, Press, Size, SizeExt, State, Theme, paint, press, recipes, resolve, states};


pub struct Select<'a, S: 'static> {
    key: String,
    name: String,
    options: &'a [String],
    current: Option<String>,
    placeholder: String,
    width: f32,
    size: Size,
    disabled: bool,
    invalid: bool,
    pinned: Option<State>,
    open: bool,
    toggle: Option<Press<S>>,
    close: Option<Press<S>>,
    choose: Option<Pick<S>>,
}

impl<'a, S: 'static> Select<'a, S> {
    /// `name` names it for assistive technology.
    pub fn new(key: impl Into<String>, name: impl Into<String>, options: &'a [String]) -> Select<'a, S> {
        Select { key: key.into(), name: name.into(), options, current: None, placeholder: "Choose…".to_owned(), width: 200.0, size: Size::Md, disabled: false, invalid: false, pinned: None, open: false, toggle: None, close: None, choose: None }
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

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }

    pub fn invalid(mut self, on: bool) -> Self {
        self.invalid = on;
        self
    }

    pub fn state(mut self, s: State) -> Self {
        self.pinned = Some(s);
        self
    }

    /// Whether the list is showing (the app keeps this).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn on_toggle(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(press(f));
        self
    }

    /// Escape or a press outside the open list.
    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.close = Some(press(f));
        self
    }

    pub fn on_choose(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.choose = Some(Rc::new(f));
        self
    }

    /// The trigger in place; the open list goes into `layer`, which the view mounts over everything (`rune_kit::root`).
    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let mut kit = t.to_kit().clone();
        kit.ty.ui_compact = self.size.text(t, t.ty.regular);
        let open = self.open && !self.disabled;
        let mut select = rune_kit::Select::new(self.key.clone(), self.options).current(self.current.as_deref()).placeholder(self.placeholder).label(self.name).width(self.width).open(open).invalid(self.invalid).disabled(self.disabled);
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
        let parts = recipes::field(t);
        // rune-kit dims a disabled trigger itself, so the well is drawn at rest under it.
        let mut st = states(self.pinned, false, false);
        st.invalid = self.invalid;
        st.open = open;
        let trigger = select.build(&kit, layer).w(self.width).h(self.size.height(t)).pl(self.size.pad(t) - 4.0).pr(t.metrics.gap).py(0.0).gap(t.metrics.gap).radius(self.size.radius(t)).continuous_corners().justify_between().border(0.0, Color::TRANSPARENT).hover(Color::TRANSPARENT);
        let mut trigger = paint(trigger, &resolve(&parts.root, st));
        if !self.disabled
            && let Some((w, o, c)) = parts.root.live().focus_ring
        {
            trigger = trigger.focus_ring(w, o, c);
        }
        row().key(self.key).child(trigger)
    }
}
