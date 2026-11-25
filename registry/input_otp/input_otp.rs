//! rune-ui registry: input_otp 0.8.0 Copied into your app by `rune-ui add input_otp`; yours to change.
//!
//! A one-time code as a row of slots, each a small **well**, in groups with a dash between them. Behind the slots is one text field
//! (Rune's `text_input`, its words and caret not drawn), so everything a text field does works: typing fills the slot at the caret and
//! moves on (on a filled slot it replaces it), Backspace and Delete empty one, Left, Right, Home and End move between slots, undo
//! works, and **pasting a code fills the slots** (`on_input_paste`: characters the pattern refuses, such as a dash or a space, are
//! dropped, and the rest fills from the caret up to the last slot). Only characters the pattern allows are taken (digits by default).
//! A whole code pasted anywhere replaces the code. The slot at the caret shows the focus ring while the field has focus; a press on a
//! slot puts the caret there (or on the first empty slot). The whole is one Tab stop. The app keeps the text (a `TextEdit`); it hears each change and the moment the code is complete.
//!
//! ```ignore
//! InputOtp::new("code", "Verification code", &s.code, |s: &mut App| &mut s.code).on_change(|s: &mut App, v: String| s.check(v))
//!     .on_complete(|s: &mut App, v: String| s.verify(v)).build(t)
//! ```

use std::rc::Rc;

use rune::{ClipboardContent, El, Key, KeyEvent, TextEdit, Tx, el, row, text};
use rune_ui_style::{Radius, tabular, weight};

use super::base::{Shape, State, States, Theme, glyph, paint_state, recipes};
use super::icons;

pub type OnCode<S> = Rc<dyn Fn(&mut S, String)>;
type Access<S> = Rc<dyn for<'b> Fn(&'b mut S) -> &'b mut TextEdit>;

pub struct InputOtp<'a, S: 'static> {
    key: String,
    name: String,
    edit: &'a TextEdit,
    access: Access<S>,
    groups: Vec<usize>,
    pattern: fn(char) -> bool,
    invalid: bool,
    disabled: bool,
    pinned: Option<State>,
    change: Option<OnCode<S>>,
    complete: Option<OnCode<S>>,
}

/// What typing `ch` at slot `i` makes of `code` (at most `n` long): it fills or replaces that slot.
pub fn typed(code: &str, i: usize, ch: char, n: usize) -> String {
    let mut v: Vec<char> = code.chars().collect();
    if i < v.len() {
        v[i] = ch;
    } else {
        v.push(ch);
    }
    v.truncate(n);
    v.into_iter().collect()
}

/// What pasting `pasted` at slot `at` makes of `code`: the characters `pattern` allows, from `at` on, replacing what was there, and
/// the slot after the last one filled. A whole code (as many allowed characters as slots, or more) replaces the code from the first
/// slot, wherever the caret was.
pub fn pasted(code: &str, at: usize, pasted: &str, n: usize, pattern: fn(char) -> bool) -> (String, usize) {
    let whole = pasted.chars().filter(|c| pattern(*c)).count() >= n;
    let mut v: Vec<char> = if whole { Vec::new() } else { code.chars().take(n).collect() };
    let mut i = if whole { 0 } else { at.min(v.len()) };
    for ch in pasted.chars().filter(|c| pattern(*c)) {
        if i >= n {
            break;
        }
        if i < v.len() {
            v[i] = ch;
        } else {
            v.push(ch);
        }
        i += 1;
    }
    (v.into_iter().collect(), i)
}

/// The byte offset of character `i` in `s` (its length past the end).
fn byte_at(s: &str, i: usize) -> usize {
    s.char_indices().nth(i).map_or(s.len(), |(b, _)| b)
}

impl<'a, S: 'static> InputOtp<'a, S> {
    /// `edit` is the code's text, kept by the app; `access` reaches it.
    pub fn new(key: impl Into<String>, name: impl Into<String>, edit: &'a TextEdit, access: impl for<'b> Fn(&'b mut S) -> &'b mut TextEdit + 'static) -> InputOtp<'a, S> {
        InputOtp { key: key.into(), name: name.into(), edit, access: Rc::new(access), groups: vec![3, 3], pattern: |c| c.is_ascii_digit(), invalid: false, disabled: false, pinned: None, change: None, complete: None }
    }

    /// Slots per group (`[3, 3]` by default: six slots, a dash in the middle).
    pub fn groups(mut self, g: &[usize]) -> Self {
        self.groups = g.to_vec();
        self
    }

    /// Which characters it takes (digits by default).
    pub fn pattern(mut self, f: fn(char) -> bool) -> Self {
        self.pattern = f;
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

    /// Pins a live state on the slot at the caret (galleries).
    pub fn state(mut self, s: State) -> Self {
        self.pinned = Some(s);
        self
    }

    /// The code after each change.
    pub fn on_change(mut self, f: impl Fn(&mut S, String) + 'static) -> Self {
        self.change = Some(Rc::new(f));
        self
    }

    /// Every slot filled.
    pub fn on_complete(mut self, f: impl Fn(&mut S, String) + 'static) -> Self {
        self.complete = Some(Rc::new(f));
        self
    }

    pub fn slot_key(key: &str, i: usize) -> String {
        format!("{key}-slot-{i}")
    }

    pub fn build(self, t: &Theme) -> El {
        let n: usize = self.groups.iter().sum::<usize>().max(1);
        let code = self.edit.text.clone();
        let chars: Vec<char> = code.chars().take(n).collect();
        let caret = code[..self.edit.focus.min(code.len())].chars().count();
        let at = caret.min(n - 1);
        let k = self.key.clone();
        let h = t.control.lg;
        let w = (h * 0.86).round();
        let parts = recipes::field(t);
        let r = Radius::fit(t.radius.md, h);
        let ring = self.edit.focused || self.pinned == Some(State::FocusVisible);
        let mut slots = row().gap(t.space.at(2.0)).center_y().a11y_hidden();
        let mut i = 0;
        for (gi, size) in self.groups.iter().enumerate() {
            if gi > 0 {
                slots = slots.child(glyph(&icons::MINUS, t.control.icon, t.ink_faint()));
            }
            let mut group = row().gap(t.space.at(1.5));
            for _ in 0..*size {
                let st = States { invalid: self.invalid, disabled: self.disabled, focus_visible: ring && i == at && !self.disabled, hover: self.pinned == Some(State::Hover) && i == at, ..States::default() };
                let mut slot = row().key(Self::slot_key(&k, i)).size(w, h).center().radius(r).round_as(t);
                slot = paint_state(slot, &parts.root, st);
                if let Some(c) = chars.get(i) {
                    slot = slot.child(text(c.to_string()).key(format!("{}-char", Self::slot_key(&k, i))).font(weight(tabular(t.ty.lg), t.ty.medium)).color(parts.label.resolve(st)));
                }
                if !self.disabled {
                    // A press on a slot puts the caret there (the first empty slot at most) and focuses the field.
                    let (acc, input, slot_at) = (self.access.clone(), format!("{k}-input"), i);
                    slot = slot.on_press_tx(move |s: &mut S, tx: &mut Tx| {
                        let e = acc(s);
                        let to = slot_at.min(e.text.chars().count());
                        let b = byte_at(&e.text, to);
                        e.set_caret(b);
                        tx.focus(input.clone(), true);
                    });
                }
                group = group.child(slot);
                i += 1;
            }
            slots = slots.child(group);
        }
        // The one text field, over the slots and not drawn: it takes the keys, the pastes and the presses.
        let clear = t.ink().alpha(0.0);
        let mut input = rune::text_input(self.edit, t.ty.lg).key(format!("{k}-input")).single_line().absolute(0.0, 0.0).w_full().h_full().color(clear).caret(clear, clear).label(self.name.clone()).a11y_value(format!("{} of {n} entered", chars.len()));
        if self.disabled {
            input = input.disabled(true).read_only(true).focusable(false);
        } else {
            let (pattern, acc) = (self.pattern, self.access.clone());
            let keys = acc.clone();
            input = input.on_edit(rune::edit_access(move |s: &mut S| acc(s))).on_input_key(move |s: &mut S, ev: &KeyEvent, _: &mut Tx| {
                let Key::Text(c) = &ev.key else { return false };
                if ev.mods.cmd || ev.mods.ctrl || c.chars().count() != 1 {
                    return false;
                }
                let ch = c.chars().next().unwrap_or(' ');
                if !pattern(ch) {
                    return true;
                }
                let e = keys(s);
                let (a, b) = e.range();
                let len = e.text.chars().count();
                if a == b {
                    let i = e.text[..a].chars().count();
                    if i < len {
                        // On a filled slot the character replaces it.
                        e.set_selection(a, byte_at(&e.text, i + 1));
                    } else if len >= n {
                        return true;
                    }
                }
                false
            });
            let (pattern, paste_acc) = (self.pattern, self.access.clone());
            input = input.on_input_paste(move |s: &mut S, content: &ClipboardContent, _: &mut Tx| {
                let e = paste_acc(s);
                let at = e.text[..e.range().0].chars().count();
                let (next, to) = pasted(&e.text, at, content.text(), n, pattern);
                e.replace_text(next);
                let b = byte_at(&e.text, to);
                e.set_caret(b);
                true
            });
            let (before, pattern, change_acc) = (code.clone(), self.pattern, self.access.clone());
            let (change, complete) = (self.change.clone(), self.complete.clone());
            input = input.on_input_change(move |s: &mut S, _: &mut Tx| {
                let e = change_acc(s);
                // Whatever arrived (an IME, an undo), only allowed characters and at most one per slot stay.
                let clean: String = e.text.chars().filter(|c| pattern(*c)).take(n).collect();
                if clean != e.text {
                    let caret = e.text[..e.focus.min(e.text.len())].chars().filter(|c| pattern(*c)).count().min(clean.chars().count());
                    e.replace_text(clean.clone());
                    let b = byte_at(&e.text, caret);
                    e.set_caret(b);
                }
                if clean == before {
                    return;
                }
                if let Some(f) = &change {
                    f(s, clean.clone());
                }
                if clean.chars().count() == n && let Some(f) = &complete {
                    f(s, clean);
                }
            });
        }
        // The field under the slots: they take the presses, it takes the keys and the pastes.
        el().key(k).role("group").label(self.name).child(input).child(slots)
    }
}
