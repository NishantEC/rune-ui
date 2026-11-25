//! rune-ui registry: input_otp 0.5.1. Copied into your app by `rune-ui add input_otp`; yours to change.
//!
//! A one-time code as a row of slots, each a small **well**, in groups with a dash between them. Typing fills the focused slot and
//! moves on; Backspace empties it (or the one before) and moves back; Left, Right, Home and End move between slots. Pasting is not
//! taken: Rune hands a paste only to a text field or the view's root at this rev. Only characters the pattern allows are taken (digits by default). The whole is one Tab stop, landing on the
//! first empty slot. The app keeps the code; it hears each change and the moment it is complete.
//!
//! ```ignore
//! InputOtp::new("code", "Verification code", &s.code).on_change(|s: &mut App, v: String| s.code = v)
//!     .on_complete(|s: &mut App, v: String| s.verify(v)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, Tx, row, text};
use rune_ui_style::{Radius, tabular, weight};

use super::base::{Shape, State, States, Theme, glyph, paint_state, recipes};
use super::icons;

pub type OnCode<S> = Rc<dyn Fn(&mut S, String)>;

pub struct InputOtp<S: 'static> {
    key: String,
    name: String,
    value: String,
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

impl<S: 'static> InputOtp<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, value: &str) -> InputOtp<S> {
        InputOtp { key: key.into(), name: name.into(), value: value.to_owned(), groups: vec![3, 3], pattern: |c| c.is_ascii_digit(), invalid: false, disabled: false, pinned: None, change: None, complete: None }
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

    /// Pins a live state on the slot the keyboard would land on (galleries).
    pub fn state(mut self, s: State) -> Self {
        self.pinned = Some(s);
        self
    }

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
        let n: usize = self.groups.iter().sum();
        let chars: Vec<char> = self.value.chars().take(n).collect();
        let next = chars.len().min(n.saturating_sub(1));
        let k = self.key.clone();
        let h = t.control.lg;
        let w = (h * 0.86).round();
        let parts = recipes::field(t);
        let r = Radius::fit(t.radius.md, h);
        let mut all = row().key(k.clone()).role("group").label(self.name.clone()).a11y_value(format!("{} of {n} entered", chars.len())).focus_group().gap(t.space.at(2.0)).center_y();
        let mut i = 0;
        for (gi, size) in self.groups.iter().enumerate() {
            if gi > 0 {
                all = all.child(glyph(&icons::MINUS, t.control.icon, t.ink_faint()));
            }
            let mut group = row().gap(t.space.at(1.5));
            for _ in 0..*size {
                let landing = i == next;
                let st = States { invalid: self.invalid, disabled: self.disabled, focus_visible: landing && self.pinned == Some(State::FocusVisible), ..States::default() };
                let mut slot = row().key(Self::slot_key(&k, i)).role("textbox").label(format!("Character {} of {n}", i + 1)).focusable(!self.disabled).focus_selected(landing).size(w, h).center().radius(r).round_as(t);
                if let Some(c) = chars.get(i) {
                    slot = slot.a11y_value(c.to_string());
                }
                slot = paint_state(slot, &parts.root, st);
                if let Some(c) = chars.get(i) {
                    slot = slot.child(text(c.to_string()).font(weight(tabular(t.ty.lg), t.ty.medium)).color(parts.label.resolve(st)).a11y_hidden());
                }
                if self.disabled {
                    slot = slot.disabled(true);
                }
                group = group.child(slot);
                i += 1;
            }
            all = all.child(group);
        }
        if self.disabled {
            return all;
        }
        let (code, pattern, prefix) = (self.value.clone(), self.pattern, format!("{k}-slot-"));
        let (change, complete) = (self.change.clone(), self.complete.clone());
        let report = move |s: &mut S, v: String| {
            let full = v.chars().count() == n;
            if let Some(f) = &change {
                f(s, v.clone());
            }
            if full && let Some(f) = &complete {
                f(s, v);
            }
        };
        all.on_key(move |s: &mut S, ev: &KeyEvent, tx: &mut Tx| {
            let Some(at) = tx.focused().and_then(|f| f.strip_prefix(&prefix)).and_then(|i| i.parse::<usize>().ok()) else { return false };
            let len = code.chars().count();
            match &ev.key {
                Key::Text(c) if !ev.mods.cmd && !ev.mods.ctrl && c.chars().count() == 1 => {
                    let ch = c.chars().next().unwrap_or(' ');
                    if !pattern(ch) {
                        return true;
                    }
                    let at = at.min(len);
                    report(s, typed(&code, at, ch, n));
                    tx.focus(format!("{prefix}{}", (at + 1).min(n - 1)), true);
                    true
                }
                Key::Backspace => {
                    let mut v: Vec<char> = code.chars().collect();
                    // A filled slot empties in place; an empty one empties the slot before it and moves there.
                    let to = if at < v.len() {
                        v.remove(at);
                        at
                    } else if at > 0 && at - 1 < v.len() {
                        v.remove(at - 1);
                        at - 1
                    } else {
                        return true;
                    };
                    report(s, v.into_iter().collect());
                    tx.focus(format!("{prefix}{to}"), true);
                    true
                }
                Key::Delete => {
                    let mut v: Vec<char> = code.chars().collect();
                    if at < v.len() {
                        v.remove(at);
                        report(s, v.into_iter().collect());
                    }
                    true
                }
                _ => false,
            }
        })
    }
}
