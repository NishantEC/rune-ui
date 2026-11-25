//! rune-ui registry: text_field 0.1.0. Copied into your app by `rune-ui add text_field`; yours to change.
//!
//! One line ([`TextField`]) and the search field ([`SearchField`]), both a **well**: a tint pressed into the page with a soft shadow
//! inside its top edge, no outline at rest. Focus adds a ring that hugs the well (the input inside has focus, so the ring is the box's
//! `focus_within`); an error keeps the inner shadow and rings the well in the danger colour, with the message under it in place of
//! the help. A label sits above; a prefix (words or an icon) and a suffix sit inside; a clear button shows while there is text.
//! Editing is Rune's own `text_input` (caret, selection, IME, undo): this file only draws round it.
//!
//! ```ignore
//! TextField::new("budget", &s.budget, |s: &mut App| &mut s.budget).label("Budget").prefix("$").help("Per task").build(t)
//! SearchField::new("find", &s.query, |s: &mut App| &mut s.query).placeholder("Find a setting").build(t)
//! ```

use std::rc::Rc;

use rune::{El, Icon, TextEdit, TextStyle, col, row, text};
use rune_ui_style::{paint, weight};

use super::base::{Look, Press, Size, Theme, glyph, press};
use super::icons;

/// How a field reaches its text in the app's state.
pub type Access<S> = Rc<dyn for<'a> Fn(&'a mut S) -> &'a mut TextEdit>;

enum Side {
    Words(String),
    Glyph(&'static Icon),
}

struct Frame<'a, S: 'static> {
    key: String,
    edit: &'a TextEdit,
    access: Access<S>,
    label: Option<String>,
    name: String,
    placeholder: String,
    help: Option<String>,
    error: Option<String>,
    prefix: Option<Side>,
    suffix: Option<Side>,
    clear: bool,
    size: Size,
    disabled: bool,
    look: Look,
    width: Option<f32>,
    submit: Option<Press<S>>,
}

impl<'a, S: 'static> Frame<'a, S> {
    fn new(key: String, edit: &'a TextEdit, access: Access<S>) -> Frame<'a, S> {
        Frame { name: key.clone(), key, edit, access, label: None, placeholder: String::new(), help: None, error: None, prefix: None, suffix: None, clear: false, size: Size::Md, disabled: false, look: Look::Live, width: None, submit: None }
    }

    fn side(&self, t: &Theme, key: String, side: &Side) -> El {
        match side {
            Side::Words(w) => text(w.clone()).key(key).font(self.size.text(t, t.ty.regular)).color(t.c.ink_faint).a11y_hidden(),
            Side::Glyph(i) => glyph(i, self.size.icon(t), t.c.ink_faint),
        }
    }

    fn input(&self, t: &Theme) -> El {
        let access = self.access.clone();
        let mut input = rune::text_input(self.edit, self.size.text(t, t.ty.regular))
            .key(format!("{}-input", self.key))
            .single_line()
            .grow(1.0)
            .shrink()
            .min_w(0.0)
            .color(t.c.ink)
            .placeholder(self.placeholder.clone(), t.c.ink_faint)
            .caret(t.c.ink, t.c.accent.alpha(0.28))
            .label(self.label.clone().unwrap_or_else(|| self.name.clone()))
            .invalid(self.error.is_some())
            .on_edit(rune::edit_access(move |s: &mut S| access(s)));
        if let Some(d) = self.error.as_ref().or(self.help.as_ref()) {
            input = input.description(d.clone());
        }
        if let Some(f) = self.submit.clone() {
            input = input.on_submit(move |s: &mut S, _| f(s));
        }
        if self.disabled {
            input = input.disabled(true).read_only(true).focusable(false);
        }
        input
    }

    fn build(self, t: &Theme) -> El {
        let m = &t.mat.well;
        let invalid = self.error.is_some();
        let h = self.size.height(t);
        let mut well = row().key(format!("{}-box", self.key)).role("group").h(h).gap(t.space.at(1.5)).px(self.size.pad(t) - 2.0).center_y().radius(self.size.radius(t)).continuous_corners();
        let mut recipe = if self.disabled { m.disabled.clone() } else if self.look == Look::Focused { m.focused.clone() } else { m.rest.clone() };
        if invalid && !self.disabled {
            recipe = recipe.at_rest().ring(t.control.ring, 0.0, t.c.danger.dot.alpha(0.75));
        }
        well = paint(well, &recipe);
        if self.look == Look::Live && !self.disabled && !invalid {
            if let Some((w, o, c)) = m.live().focus_ring {
                well = well.focus_within_ring(w, o, c);
            }
        }
        well = match self.width {
            Some(w) => well.w(w),
            None => well.w_full(),
        };
        if let Some(p) = &self.prefix {
            well = well.child(self.side(t, format!("{}-prefix", self.key), p));
        }
        well = well.child(self.input(t));
        if self.clear && !self.edit.text.is_empty() && !self.disabled {
            let access = self.access.clone();
            let x = row()
                .key(format!("{}-clear", self.key))
                .role("button")
                .label("Clear")
                .help("Clear")
                .focusable(true)
                .size(18.0, 18.0)
                .center()
                .radius(9.0)
                .hover(t.c.hover)
                .focus_ring(t.control.ring, t.control.ring_offset, t.c.focus)
                .on_press(move |s: &mut S| access(s).clear())
                .child(glyph(&icons::CLOSE, t.control.icon_sm, t.c.ink_faint));
            well = well.child(x);
        } else if let Some(sfx) = &self.suffix {
            well = well.child(self.side(t, format!("{}-suffix", self.key), sfx));
        }
        if self.label.is_none() && self.help.is_none() && self.error.is_none() {
            return well.key(self.key);
        }
        let mut c = col().key(self.key.clone()).gap(t.space.at(1.5));
        c = match self.width {
            Some(w) => c.w(w),
            None => c.w_full(),
        };
        if let Some(l) = &self.label {
            c = c.child(text(l.clone()).key(format!("{}-label", self.key)).font(weight(t.ty.sm, t.ty.medium)).color(t.c.ink).a11y_hidden());
        }
        c = c.child(well);
        let note = |words: &str, ink, key: String| text(words.to_owned()).key(key).font(TextStyle { ..t.ty.sm }).color(ink).wrap().min_w(0.0);
        if let Some(e) = &self.error {
            c = c.child(note(e, t.c.danger.ink, format!("{}-error", self.key)).live_region(rune::LiveRegion::Polite));
        } else if let Some(h) = &self.help {
            c = c.child(note(h, t.c.ink_faint, format!("{}-help", self.key)));
        }
        c
    }
}

macro_rules! shared_setters {
    () => {
        /// A visible label above the well (also the field's name).
        pub fn label(mut self, l: impl Into<String>) -> Self {
            self.f.label = Some(l.into());
            self
        }
        /// Its name for assistive technology when it has no visible label.
        pub fn name(mut self, n: impl Into<String>) -> Self {
            self.f.name = n.into();
            self
        }
        pub fn placeholder(mut self, p: impl Into<String>) -> Self {
            self.f.placeholder = p.into();
            self
        }
        /// A quiet line under the well.
        pub fn help(mut self, h: impl Into<String>) -> Self {
            self.f.help = Some(h.into());
            self
        }
        /// What is wrong, under the well in place of the help; the well is ringed in the danger colour.
        pub fn error(mut self, e: Option<String>) -> Self {
            self.f.error = e;
            self
        }
        pub fn disabled(mut self, on: bool) -> Self {
            self.f.disabled = on;
            self
        }
        pub fn look(mut self, look: Look) -> Self {
            self.f.look = look;
            self
        }
        pub fn width(mut self, w: f32) -> Self {
            self.f.width = Some(w);
            self
        }
        pub fn size(mut self, size: Size) -> Self {
            self.f.size = size;
            self
        }
        /// Return runs this.
        pub fn on_submit(mut self, f: impl Fn(&mut S) + 'static) -> Self {
            self.f.submit = Some(press(f));
            self
        }
    };
}

/// A one-line text field.
pub struct TextField<'a, S: 'static> {
    f: Frame<'a, S>,
}

impl<'a, S: 'static> TextField<'a, S> {
    pub fn new(key: impl Into<String>, edit: &'a TextEdit, access: impl for<'b> Fn(&'b mut S) -> &'b mut TextEdit + 'static) -> TextField<'a, S> {
        TextField { f: Frame::new(key.into(), edit, Rc::new(access)) }
    }

    shared_setters!();

    /// Words inside the well before the text ("$", "https://").
    pub fn prefix(mut self, words: impl Into<String>) -> Self {
        self.f.prefix = Some(Side::Words(words.into()));
        self
    }

    pub fn prefix_icon(mut self, i: &'static Icon) -> Self {
        self.f.prefix = Some(Side::Glyph(i));
        self
    }

    /// Words inside the well after the text ("per day").
    pub fn suffix(mut self, words: impl Into<String>) -> Self {
        self.f.suffix = Some(Side::Words(words.into()));
        self
    }

    /// A clear button while there is text.
    pub fn clearable(mut self) -> Self {
        self.f.clear = true;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        self.f.build(t)
    }
}

/// The search field: the magnifier, the words, and a clear button once it has text.
pub struct SearchField<'a, S: 'static> {
    f: Frame<'a, S>,
}

impl<'a, S: 'static> SearchField<'a, S> {
    pub fn new(key: impl Into<String>, edit: &'a TextEdit, access: impl for<'b> Fn(&'b mut S) -> &'b mut TextEdit + 'static) -> SearchField<'a, S> {
        let mut f = Frame::new(key.into(), edit, Rc::new(access));
        f.prefix = Some(Side::Glyph(&icons::SEARCH));
        f.clear = true;
        f.name = "Search".to_owned();
        f.placeholder = "Search".to_owned();
        SearchField { f }
    }

    shared_setters!();

    /// Words at its right while it is empty (the keys that reach it: "⌘K").
    pub fn hint(mut self, keys: impl Into<String>) -> Self {
        self.f.suffix = Some(Side::Words(keys.into()));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        self.f.build(t)
    }
}
