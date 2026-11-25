//! rune-ui registry: combobox 0.2.0. Copied into your app by `rune-ui add combobox`; yours to change.
//!
//! One choice of many, with a search. Behaviour is rune-kit's `Combobox` (its state machine: open, filter, highlight, choose; keys and
//! dismissal), handed our look through `Theme::to_kit()`: the trigger is a **well**, the popover the **floating** material with a search
//! well over the matching rows. The app keeps a `rune_kit::ComboboxState` per combobox.
//!
//! ```ignore
//! Combobox::new("repo", "Repository", &s.repo_state, &options, |s: &mut App| &mut s.repo_state)
//!     .current(s.repo.as_deref()).on_choose(|s: &mut App, id: &str| s.repo = Some(id.to_owned())).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{Color, El, Tx};
pub use rune_kit::{ComboboxOption, ComboboxState};

use super::base::{Theme, paint, recipes, resolve};

type Choose<S> = Rc<dyn Fn(&mut S, &str)>;

pub struct Combobox<'a, S: 'static> {
    key: String,
    name: String,
    state: &'a ComboboxState,
    access: Rc<dyn for<'b> Fn(&'b mut S) -> &'b mut ComboboxState>,
    options: &'a [ComboboxOption],
    current: Option<String>,
    placeholder: String,
    width: f32,
    choose: Option<Choose<S>>,
}

impl<'a, S: 'static> Combobox<'a, S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, state: &'a ComboboxState, options: &'a [ComboboxOption], access: impl for<'b> Fn(&'b mut S) -> &'b mut ComboboxState + 'static) -> Combobox<'a, S> {
        Combobox { key: key.into(), name: name.into(), state, access: Rc::new(access), options, current: None, placeholder: "Choose…".to_owned(), width: 240.0, choose: None }
    }

    /// The chosen option's id.
    pub fn current(mut self, id: Option<&str>) -> Self {
        self.current = id.map(str::to_owned);
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

    pub fn on_choose(mut self, f: impl Fn(&mut S, &str) + 'static) -> Self {
        self.choose = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let mut kit = t.to_kit().clone();
        kit.ty.ui_compact = t.ty.base;
        let access = self.access.clone();
        let mut combo = rune_kit::Combobox::new(self.key.clone(), self.state, self.options, move |s: &mut S| access(s)).label(self.name).current(self.current.as_deref()).placeholder(self.placeholder).search_placeholder("Search").empty_message("Nothing matches").width(self.width.max(220.0));
        if let Some(f) = self.choose {
            combo = combo.on_choose(move |s: &mut S, id: &str, _: &mut Tx| f(s, id));
        }
        let parts = recipes::field(t);
        let trigger = combo.build(&kit, layer).w(self.width).h(t.control.md).radius(t.radius.md).continuous_corners().border(0.0, Color::TRANSPARENT).hover(Color::TRANSPARENT);
        let mut trigger = paint(trigger, &resolve(&parts.root, Default::default()));
        if let Some((w, o, c)) = parts.root.live().focus_ring {
            trigger = trigger.focus_ring(w, o, c);
        }
        trigger
    }
}
