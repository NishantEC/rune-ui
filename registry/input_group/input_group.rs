//! rune-ui registry: input_group 0.5.1. Copied into your app by `rune-ui add input_group`; yours to change.
//!
//! One **well** holding a text field and what belongs with it: words, icons or small buttons at its start and end (a protocol, a
//! unit, a search glyph, a copy button, a send button). The well rings when anything inside has focus. Editing is Rune's `text_input`.
//! For words or one icon only, `Input`'s prefix and suffix are enough; this is for controls inside the field.
//!
//! ```ignore
//! InputGroup::new("site", &s.site, |s: &mut App| &mut s.site).name("Website").start(words(t, "https://")).end(copy).build(t)
//! ```

use std::rc::Rc;

use rune::{El, TextEdit, row, text};

use super::base::{Shape, Size, SizeExt, Theme, recipes, states};

pub type Access<S> = Rc<dyn for<'a> Fn(&'a mut S) -> &'a mut TextEdit>;

pub struct InputGroup<'a, S: 'static> {
    key: String,
    edit: &'a TextEdit,
    access: Access<S>,
    name: String,
    placeholder: String,
    start: Vec<El>,
    end: Vec<El>,
    width: Option<f32>,
    invalid: bool,
    disabled: bool,
}

/// Quiet words for an addon (`https://`, `USD`).
pub fn words(t: &Theme, w: &str) -> El {
    text(w.to_owned()).font(t.ty.base).color(t.ink_faint()).a11y_hidden()
}

impl<'a, S: 'static> InputGroup<'a, S> {
    pub fn new(key: impl Into<String>, edit: &'a TextEdit, access: impl for<'b> Fn(&'b mut S) -> &'b mut TextEdit + 'static) -> InputGroup<'a, S> {
        InputGroup { key: key.into(), edit, access: Rc::new(access), name: String::new(), placeholder: String::new(), start: Vec::new(), end: Vec::new(), width: None, invalid: false, disabled: false }
    }

    pub fn name(mut self, n: impl Into<String>) -> Self {
        self.name = n.into();
        self
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    /// Something before the text (words, an icon, a button).
    pub fn start(mut self, e: El) -> Self {
        self.start.push(e);
        self
    }

    /// Something after the text.
    pub fn end(mut self, e: El) -> Self {
        self.end.push(e);
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
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

    pub fn build(self, t: &Theme) -> El {
        let size = Size::Md;
        let parts = recipes::field(t);
        let mut st = states(None, self.disabled, false);
        st.invalid = self.invalid;
        // Buttons inside sit 4 points from the well's edge; words get the usual side padding.
        let ends = |v: &Vec<El>| v.is_empty();
        let mut well = row().key(self.key.clone()).role("group").label(self.name.clone()).h(size.height(t)).gap(t.metrics.gap).pl(if ends(&self.start) { size.pad(t) - 2.0 } else { t.space.at(2.0) }).pr(if ends(&self.end) { size.pad(t) - 2.0 } else { t.space.at(1.0) }).center_y().radius(size.radius(t)).round_as(t);
        well = rune_ui_style::paint(well, &rune_ui_style::resolve(&parts.root, st));
        if st.live() && !self.invalid
            && let Some((w, o, c)) = parts.root.live().focus_ring
        {
            well = well.focus_within_ring(w, o, c);
        }
        well = match self.width {
            Some(w) => well.w(w),
            None => well.w_full(),
        };
        let access = self.access.clone();
        let mut input = rune::text_input(self.edit, size.text(t, t.ty.regular)).key(format!("{}-input", self.key)).single_line().grow(1.0).shrink().min_w(0.0).color(parts.label.resolve(st)).placeholder(self.placeholder.clone(), t.ink_faint()).caret(t.ink(), t.c.accent.fg.alpha(0.28)).label(self.name.clone()).invalid(self.invalid).on_edit(rune::edit_access(move |s: &mut S| access(s)));
        if self.disabled {
            input = input.disabled(true).read_only(true).focusable(false);
        }
        well.children(self.start).child(input).children(self.end)
    }
}
