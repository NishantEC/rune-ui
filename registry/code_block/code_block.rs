//! rune-ui registry: code_block 0.6.0. Copied into your app by `rune-ui add code_block`; yours to change.
//!
//! Code on the slab: a header with the language in quiet words and a Copy button (it puts the code on the clipboard, runs your handler
//! and shows a check while `copied`), then the code in a monospaced face, one line per line, scrolling sideways instead of wrapping.
//! Optional line numbers in the third ink. Colour comes free from Rune's built-in highlighter (`rune_kit::syntax`, 80 languages) when
//! the language is one it knows; `.plain()` turns it off.
//!
//! ```ignore
//! CodeBlock::new("snippet", code, Some("rust")).line_numbers(true).copied(s.copied).on_copy(|s: &mut App| s.copied = true).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Span, Tx, col, rich, row, text};
use rune_kit::syntax::{SyntaxTheme, highlight};
use rune_ui_style::{tabular, weight};

use super::agent::{PressTx, icon_button, mono, slab};
use super::base::{Press, Theme};
use super::icons;

pub struct CodeBlock<S: 'static> {
    key: String,
    code: String,
    language: Option<String>,
    numbers: bool,
    colour: bool,
    copied: bool,
    copy: Option<Press<S>>,
    max_h: f32,
}

impl<S: 'static> CodeBlock<S> {
    pub fn new(key: impl Into<String>, code: impl Into<String>, language: Option<&str>) -> CodeBlock<S> {
        CodeBlock { key: key.into(), code: code.into(), language: language.map(str::to_owned), numbers: false, colour: true, copied: false, copy: None, max_h: 0.0 }
    }

    pub fn line_numbers(mut self, on: bool) -> Self {
        self.numbers = on;
        self
    }

    /// No syntax colour.
    pub fn plain(mut self) -> Self {
        self.colour = false;
        self
    }

    /// Just copied: the button shows a check.
    pub fn copied(mut self, on: bool) -> Self {
        self.copied = on;
        self
    }

    /// Runs after the code is on the clipboard.
    pub fn on_copy(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.copy = Some(Rc::new(f));
        self
    }

    /// At most this tall, then it scrolls (0: no limit).
    pub fn max_height(mut self, h: f32) -> Self {
        self.max_h = h;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let style = mono(t, t.ty.sm);
        let code = self.code.trim_end_matches('\n').to_owned();
        let spans: Vec<Span> = match (&self.language, self.colour) {
            (Some(lang), true) => SyntaxTheme::from_kit(t.to_kit()).spans(&highlight(Some(lang), &code), 0),
            _ => Vec::new(),
        };
        let mut lines = col().key(format!("{}-code", self.key)).gap(0.0);
        let mut gutter = col().items_end().gap(0.0).pr(t.space.at(3.0)).a11y_hidden();
        let mut start = 0usize;
        for (i, l) in code.split('\n').enumerate() {
            let end = start + l.len();
            let mine: Vec<Span> = spans.iter().filter(|s| (s.end as usize) > start && (s.start as usize) < end).map(|s| Span { start: (s.start as usize).max(start) as u32 - start as u32, end: (s.end as usize).min(end) as u32 - start as u32, ..s.clone() }).collect();
            let words = if l.is_empty() { " ".to_owned() } else { l.to_owned() };
            let line = if mine.is_empty() { text(words).font(style).color(t.ink()) } else { rich(words, mine).font(style).color(t.ink()) };
            lines = lines.child(row().child(line));
            gutter = gutter.child(text((i + 1).to_string()).font(tabular(style)).color(t.ink_faint()));
            start = end + 1;
        }
        let mut body = row().items_start().py(t.space.at(3.0)).px(t.space.at(3.5));
        if self.numbers {
            body = body.child(gutter);
        }
        let mut view = col().key(format!("{}-scroll", self.key)).w_full().scroll_x().child(body.child(lines));
        if self.max_h > 0.0 {
            view = view.max_h(self.max_h).scroll_xy();
        }
        let copy_words = self.code.clone();
        let then = self.copy.clone();
        let press: PressTx<S> = Rc::new(move |s: &mut S, tx: &mut Tx| {
            tx.copy(copy_words.clone());
            if let Some(f) = &then {
                f(s);
            }
        });
        let head = row().w_full().h(t.control.sm + 8.0).pl(t.space.at(3.5)).pr(t.space.at(1.5)).center_y().border_sides([0.0, 0.0, 1.0, 0.0], t.c.line)
            .child(text(self.language.clone().unwrap_or_else(|| "text".to_owned())).font(weight(t.ty.xs, t.ty.medium)).color(t.ink_faint()).truncate().min_w(0.0).grow(1.0))
            .child(icon_button(t, &format!("{}-copy", self.key), if self.copied { "Copied" } else { "Copy code" }, if self.copied { &icons::CHECK } else { &icons::COPY }, None, false, Some(press)));
        slab(t, self.key.clone()).role("figure").label(format!("{} code", self.language.as_deref().unwrap_or("Plain"))).child(head).child(view)
    }
}
