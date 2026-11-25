//! rune-ui registry: prompt_input 0.6.0. Copied into your app by `rune-ui add prompt_input`; yours to change.
//!
//! Where a person writes to the agent: a well holding attachment chips, a text area that grows with the text (one line at first, eight
//! at most, then it scrolls), and a toolbar row (an attach button, whatever the app puts there, such as a model selector and tool
//! toggles, then the submit button at the right). Return sends; Shift+Return makes a new line. While the agent answers the submit
//! button becomes Stop and Escape stops; disabled, nothing can be typed or sent.
//!
//! ```ignore
//! PromptInput::new("composer", &s.draft, |s: &mut App| &mut s.draft).phase(s.phase).tool(model_selector)
//!     .on_submit(|s: &mut App| s.send()).on_stop(|s: &mut App| s.stop()).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Key, KeyEvent, TextEdit, Tx, col, row};

use super::agent::{PressTx, icon_button};
use super::base::{Press, Shape, Theme, glyph, hit_area, paint, paint_state, recipes, resolve, states};
use super::icons;

/// Whether the composer can send, is waiting on an answer, or is off.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Ready,
    /// An answer is arriving: submit is Stop.
    Streaming,
    Disabled,
}

pub struct PromptInput<'a, S: 'static> {
    key: String,
    edit: &'a TextEdit,
    access: Rc<dyn for<'b> Fn(&'b mut S) -> &'b mut TextEdit>,
    placeholder: String,
    phase: Phase,
    attachments: Vec<El>,
    tools: Vec<El>,
    rows: (usize, usize),
    submit: Option<Press<S>>,
    stop: Option<Press<S>>,
    attach: Option<Press<S>>,
}

impl<'a, S: 'static> PromptInput<'a, S> {
    pub fn new(key: impl Into<String>, edit: &'a TextEdit, access: impl for<'b> Fn(&'b mut S) -> &'b mut TextEdit + 'static) -> PromptInput<'a, S> {
        PromptInput { key: key.into(), edit, access: Rc::new(access), placeholder: "Ask anything".to_owned(), phase: Phase::Ready, attachments: Vec::new(), tools: Vec::new(), rows: (1, 8), submit: None, stop: None, attach: None }
    }

    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }

    pub fn phase(mut self, p: Phase) -> Self {
        self.phase = p;
        self
    }

    /// An attachment chip above the text (`Attachment`).
    pub fn attachment(mut self, e: El) -> Self {
        self.attachments.push(e);
        self
    }

    /// Something for the toolbar, after the attach button (a model selector, a tool toggle).
    pub fn tool(mut self, e: El) -> Self {
        self.tools.push(e);
        self
    }

    /// Lines at first and at most (1 and 8).
    pub fn rows(mut self, min: usize, max: usize) -> Self {
        self.rows = (min.max(1), max.max(min.max(1)));
        self
    }

    /// Return or the submit button, with words in the field and the composer ready.
    pub fn on_submit(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.submit = Some(Rc::new(f));
        self
    }

    /// Stop or Escape while streaming.
    pub fn on_stop(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.stop = Some(Rc::new(f));
        self
    }

    /// Shows the attach button.
    pub fn on_attach(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.attach = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let parts = recipes::field(t);
        let disabled = self.phase == Phase::Disabled;
        let streaming = self.phase == Phase::Streaming;
        let st = states(None, disabled, false);
        let mut well = col().key(self.key.clone()).role("group").label("Message composer").w_full().pt(t.space.at(2.5)).pb(t.space.at(2.0)).px(t.space.at(3.0)).gap(t.space.at(2.0)).radius(t.radius.xl).round_as(t);
        well = paint(well, &resolve(&parts.root, st));
        if !disabled && let Some((w, o, c)) = parts.root.live().focus_ring {
            well = well.focus_within_ring(w, o, c);
        }
        if !self.attachments.is_empty() {
            well = well.child(row().key(format!("{}-attachments", self.key)).w_full().gap(t.space.at(2.0)).flex_wrap().children(self.attachments));
        }
        let has_words = !self.edit.text.trim().is_empty();
        let can_send = self.phase == Phase::Ready && has_words;
        let send = self.submit.clone().filter(|_| can_send);
        let stop = self.stop.clone().filter(|_| streaming);
        let (access, words_in) = (self.access.clone(), self.access.clone());
        let (on_enter, on_escape) = (self.submit.clone(), self.stop.clone());
        let ready = self.phase == Phase::Ready;
        let mut input = rune::text_input(self.edit, t.ty.md)
            .key(format!("{}-input", self.key))
            .w_full()
            .color(t.ink())
            .placeholder(self.placeholder.clone(), t.ink_faint())
            .caret(t.ink(), t.c.accent.fg.alpha(0.28))
            .label("Message")
            .on_submit(move |s: &mut S, _: &mut Tx| {
                if ready && !words_in(s).text.trim().is_empty() && let Some(f) = &on_enter {
                    f(s);
                }
            })
            .on_input_key(move |s: &mut S, k: &KeyEvent, _: &mut Tx| {
                if k.key == Key::Escape && streaming && let Some(f) = &on_escape {
                    f(s);
                    return true;
                }
                false
            });
        // A disabled composer is not bound to the draft, so nothing typed can reach it.
        if disabled {
            input = input.disabled(true).read_only(true).focusable(false);
        } else {
            input = input.on_edit(rune::edit_access(move |s: &mut S| access(s)));
        }
        well = well.child(rune::auto_grow(input, self.rows.0, self.rows.1));
        let mut bar = row().key(format!("{}-toolbar", self.key)).w_full().gap(t.space.at(3.5)).center_y();
        if let Some(f) = self.attach.clone() {
            let p: PressTx<S> = Rc::new(move |s: &mut S, _: &mut Tx| f(s));
            bar = bar.child(icon_button(t, &format!("{}-attach", self.key), "Attach files", &icons::PAPERCLIP, None, disabled, Some(p)));
        }
        bar = bar.children(self.tools).child(row().grow(1.0));
        // The submit button: ink, round, an arrow; Stop while streaming.
        let h = t.control.sm + 4.0;
        let key = format!("{}-submit", self.key);
        let live = if streaming { stop.is_some() } else { send.is_some() };
        let face = row().key(key.clone()).role("button").label(if streaming { "Stop" } else { "Send" }).help(if streaming { "Stop (Escape)" } else { "Send (Return)" }).focusable(live).size(h, h).center().radius(h / 2.0);
        let mut face = paint_state(face, &t.mat.ink, states(None, !live, false)).child(glyph(if streaming { &icons::STOP } else { &icons::ARROW_UP }, t.control.icon, t.c.primary.fg));
        if !live {
            face = face.disabled(true);
        }
        let press = if streaming { stop } else { send };
        if let Some(f) = press.clone() {
            face = face.on_press(move |s: &mut S| f(s));
        }
        let slop = t.control.slop(h);
        bar = bar.child(hit_area(&key, face, slop, slop, press));
        well.child(bar)
    }
}
