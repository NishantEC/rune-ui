//! rune-ui registry: stack_trace 0.8.0. Copied into your app by `rune-ui add stack_trace`; yours to change.
//!
//! An error and where it happened, on the slab: the error's title in the danger ink with a mark, its message, then the frames, the
//! newest first. A frame is its function and its place (`file:line:column`) in a monospaced face. Frames in the app's own code are
//! drawn in the first ink; runs of library frames are folded into one quiet line ("4 library frames") and drawn faint when shown.
//! Pressing a frame (or Return or Space on it) hands its path, line and column to the app (`on_frame`): the trace never opens
//! anything itself. Copy puts the whole trace on the clipboard as text.
//!
//! ```ignore
//! StackTrace::new("err", "TypeError: total is undefined", &frames).message("while rendering the invoice list")
//!     .show_library(s.libs).on_toggle_library(|s: &mut App| s.libs = !s.libs)
//!     .on_frame(|s: &mut App, file: &str, line, col| s.open_at(file, line, col)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Tx, col, row, text};
use rune_ui_style::weight;

use super::agent::{PressTx, icon_button, mono, return_presses, slab};
use super::base::{Press, Shape, State, States, Theme, glyph, hit_area, paint_state};
use super::icons;

/// One place on the stack.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Frame {
    pub function: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    /// In the app's own code (otherwise a library's, folded by default).
    pub app: bool,
}

impl Frame {
    pub fn app(function: impl Into<String>, file: impl Into<String>, line: u32, column: u32) -> Frame {
        Frame { function: function.into(), file: file.into(), line, column, app: true }
    }

    pub fn library(function: impl Into<String>, file: impl Into<String>, line: u32, column: u32) -> Frame {
        Frame { app: false, ..Frame::app(function, file, line, column) }
    }

    /// `file:line:column`.
    pub fn place(&self) -> String {
        format!("{}:{}:{}", self.file, self.line, self.column)
    }
}

/// The trace as text: the title, the message, then `    at function (file:line:column)` per frame.
pub fn as_text(title: &str, message: Option<&str>, frames: &[Frame]) -> String {
    let mut s = title.to_owned();
    if let Some(m) = message {
        s.push('\n');
        s.push_str(m);
    }
    for f in frames {
        s.push_str(&format!("\n    at {} ({})", f.function, f.place()));
    }
    s
}

pub type OnFrame<S> = Rc<dyn Fn(&mut S, &str, u32, u32)>;

pub struct StackTrace<'a, S: 'static> {
    key: String,
    title: String,
    message: Option<String>,
    frames: &'a [Frame],
    library: bool,
    toggle: Option<Press<S>>,
    frame: Option<OnFrame<S>>,
    copied: bool,
    copy: Option<Press<S>>,
    pinned: Option<(usize, State)>,
}

impl<'a, S: 'static> StackTrace<'a, S> {
    pub fn new(key: impl Into<String>, title: impl Into<String>, frames: &'a [Frame]) -> StackTrace<'a, S> {
        StackTrace { key: key.into(), title: title.into(), message: None, frames, library: false, toggle: None, frame: None, copied: false, copy: None, pinned: None }
    }

    /// A line under the title.
    pub fn message(mut self, m: impl Into<String>) -> Self {
        self.message = Some(m.into());
        self
    }

    /// Library frames shown (otherwise each run of them is one folded line).
    pub fn show_library(mut self, on: bool) -> Self {
        self.library = on;
        self
    }

    /// A folded line was pressed: show or hide the library frames.
    pub fn on_toggle_library(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.toggle = Some(Rc::new(f));
        self
    }

    /// A frame was pressed: its path, line and column.
    pub fn on_frame(mut self, f: impl Fn(&mut S, &str, u32, u32) + 'static) -> Self {
        self.frame = Some(Rc::new(f));
        self
    }

    /// Copy shows a check while this is on.
    pub fn copied(mut self, on: bool) -> Self {
        self.copied = on;
        self
    }

    /// After the trace is copied.
    pub fn on_copy(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.copy = Some(Rc::new(f));
        self
    }

    /// Pins a live state on frame `i` (galleries).
    pub fn state(mut self, i: usize, s: State) -> Self {
        self.pinned = Some((i, s));
        self
    }

    fn frame_row(&self, t: &Theme, i: usize, f: &Frame) -> El {
        let key = format!("{}-frame-{i}", self.key);
        let h = t.control.sm;
        let mut st = States::default();
        if let Some((at, s)) = self.pinned
            && at == i
        {
            st.set(s, true);
        }
        let live = self.frame.is_some();
        let (ink, place_ink) = if f.app { (t.ink(), t.ink_muted()) } else { (t.ink_muted(), t.ink_faint()) };
        let style = mono(t, t.ty.sm);
        let mut face = row().key(key.clone()).role(if live { "button" } else { "listitem" }).label(format!("{} at {}", f.function, f.place())).focusable(live).w_full().min_h(h).px(t.space.at(2.0)).gap(t.space.at(3.0)).center_y().radius(rune_ui_style::Radius::fit(t.radius.md, h)).round_as(t);
        face = paint_state(face, &t.mat.row, st);
        face = face.child(text(f.function.clone()).font(if f.app { weight(style, t.ty.medium) } else { style }).color(ink).truncate().shrink().min_w(0.0).a11y_hidden());
        face = face.child(row().grow(1.0)).child(text(f.place()).font(style).color(place_ink).truncate().shrink().min_w(0.0).a11y_hidden());
        let press: Option<Press<S>> = self.frame.clone().map(|cb| {
            let (file, line, column) = (f.file.clone(), f.line, f.column);
            let p: Press<S> = Rc::new(move |s: &mut S| cb(s, &file, line, column));
            p
        });
        if let Some(p) = press.clone() {
            let q = p.clone();
            face = face.on_press(move |s: &mut S| p(s)).on_key(move |s: &mut S, k: &rune::KeyEvent, _: &mut Tx| return_presses(k, || q(s)));
        }
        hit_area(&key, face, 0.0, t.control.slop(h), press)
    }

    pub fn build(self, t: &Theme) -> El {
        let k = self.key.clone();
        let words = as_text(&self.title, self.message.as_deref(), self.frames);
        let then = self.copy.clone();
        let copy: PressTx<S> = Rc::new(move |s: &mut S, tx: &mut Tx| {
            tx.copy(words.clone());
            if let Some(f) = &then {
                f(s);
            }
        });
        let head = row().w_full().gap(t.space.at(2.0)).center_y()
            .child(glyph(&icons::CIRCLE_X, t.control.icon, t.c.danger.fg))
            .child(text(self.title.clone()).key(format!("{k}-title")).font(weight(t.ty.base, t.ty.medium)).color(t.c.danger.fg).wrap().grow(1.0).shrink().min_w(0.0))
            .child(icon_button(t, &format!("{k}-copy"), if self.copied { "Copied" } else { "Copy trace" }, if self.copied { &icons::CHECK } else { &icons::COPY }, None, false, Some(copy)));
        let mut c = col().w_full().px(t.space.at(3.0)).pt(t.space.at(2.5)).pb(t.space.at(2.0)).gap(t.space.at(1.0)).child(head);
        if let Some(m) = &self.message {
            c = c.child(text(m.clone()).font(t.ty.sm).color(t.ink_muted()).wrap().min_w(0.0).pl(t.control.icon + t.space.at(2.0)));
        }
        // Rows sit twice their hit slop apart, so neighbouring hit areas never overlap.
        let mut list = col().key(format!("{k}-frames")).role("list").label("Frames").w_full().gap(2.0 * t.control.slop(t.control.sm)).pt(t.space.at(1.0));
        let mut i = 0;
        while i < self.frames.len() {
            let f = &self.frames[i];
            if f.app || self.library {
                list = list.child(self.frame_row(t, i, f));
                i += 1;
                continue;
            }
            // A run of library frames, folded into one line.
            let start = i;
            while i < self.frames.len() && !self.frames[i].app {
                i += 1;
            }
            let n = i - start;
            let key = format!("{k}-library-{start}");
            let h = t.control.sm;
            let mut line = row().key(key.clone()).role("button").label(format!("{n} library frame{}", if n == 1 { "" } else { "s" })).expanded(false).focusable(self.toggle.is_some()).h(h).px(t.space.at(2.0)).gap(t.space.at(1.5)).center_y().radius(rune_ui_style::Radius::fit(t.radius.md, h)).round_as(t);
            line = paint_state(line, &t.mat.row, States::default()).child(glyph(&icons::CHEVRON_RIGHT, t.control.icon_sm, t.ink_faint())).child(text(format!("{n} library frame{}", if n == 1 { "" } else { "s" })).font(t.ty.sm).color(t.ink_faint()).a11y_hidden());
            if let Some(f) = self.toggle.clone() {
                let g = f.clone();
                line = line.on_press(move |s: &mut S| f(s)).on_key(move |s: &mut S, k: &rune::KeyEvent, _: &mut Tx| return_presses(k, || g(s)));
            }
            list = list.child(hit_area(&key, line, 0.0, t.control.slop(h), self.toggle.clone()));
        }
        if self.library && self.frames.iter().any(|f| !f.app) && let Some(f) = self.toggle.clone() {
            let key = format!("{k}-library-hide");
            let h = t.control.sm;
            let g = f.clone();
            let line = paint_state(row().key(key.clone()).role("button").label("Hide library frames").expanded(true).focusable(true).h(h).px(t.space.at(2.0)).gap(t.space.at(1.5)).center_y().radius(rune_ui_style::Radius::fit(t.radius.md, h)).round_as(t), &t.mat.row, States::default())
                .child(glyph(&icons::CHEVRON_DOWN, t.control.icon_sm, t.ink_faint())).child(text("Hide library frames").font(t.ty.sm).color(t.ink_faint()).a11y_hidden())
                .on_press(move |s: &mut S| f(s)).on_key(move |s: &mut S, k: &rune::KeyEvent, _: &mut Tx| return_presses(k, || g(s)));
            list = list.child(line);
        }
        slab(t, k.clone()).role("group").label(self.title.clone()).child(c.child(list))
    }
}
