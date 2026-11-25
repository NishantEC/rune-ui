//! rune-ui registry: terminal 0.6.0. Copied into your app by `rune-ui add terminal`; yours to change.
//!
//! A command and what it printed, on the slab: a header with the prompt and command in a monospaced face, then its state at the right
//! (a turning ring and "Running" while it runs; "exit 0" in the success ink, any other exit code in the danger ink) and how long it
//! took; under it the output, which follows the newest line while the command runs and scrolls once it is long.
//!
//! ```ignore
//! Terminal::new("t1", "cargo test", &output).running(false).exit(Some(0)).secs(12.4).build(t)
//! ```

use rune::{El, col, row, text};
use rune_ui_style::tabular;

use super::agent::{elapsed, meta, mono, slab};
use super::base::{Theme, glyph, spinner};
use super::icons;

pub struct Terminal<'a> {
    key: String,
    command: String,
    output: &'a str,
    running: bool,
    exit: Option<i32>,
    secs: Option<f64>,
    max_h: f32,
}

impl<'a> Terminal<'a> {
    pub fn new(key: impl Into<String>, command: impl Into<String>, output: &'a str) -> Terminal<'a> {
        Terminal { key: key.into(), command: command.into(), output, running: false, exit: None, secs: None, max_h: 240.0 }
    }

    pub fn running(mut self, on: bool) -> Self {
        self.running = on;
        self
    }

    /// The exit status once it ended.
    pub fn exit(mut self, code: Option<i32>) -> Self {
        self.exit = code;
        self
    }

    pub fn secs(mut self, s: f64) -> Self {
        self.secs = Some(s);
        self
    }

    /// The output's tallest before it scrolls (240).
    pub fn max_height(mut self, h: f32) -> Self {
        self.max_h = h;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let style = mono(t, t.ty.sm);
        let state = if self.running {
            row().gap(t.space.at(1.5)).center_y().child(spinner(t.control.icon_sm, t.ink_muted(), "Running")).child(text("Running").font(t.ty.sm).color(t.ink_muted()).a11y_hidden())
        } else {
            match self.exit {
                Some(code) => text(format!("exit {code}")).font(tabular(t.ty.sm)).color(if code == 0 { t.c.success.fg } else { t.c.danger.fg }),
                None => text("Stopped").font(t.ty.sm).color(t.ink_faint()),
            }
        };
        let mut head = row().w_full().h(t.control.sm + 10.0).px(t.space.at(3.0)).gap(t.space.at(2.0)).center_y().border_sides([0.0, 0.0, 1.0, 0.0], t.c.line)
            .child(glyph(&icons::TERMINAL, t.control.icon, t.ink_faint()))
            .child(text(format!("$ {}", self.command)).font(style).color(t.ink()).truncate().min_w(0.0).grow(1.0))
            .child(state);
        if let Some(s) = self.secs {
            head = head.child(meta(t, elapsed(s)));
        }
        let mut lines = col().py(t.space.at(2.5)).px(t.space.at(3.0)).gap(0.0);
        for l in self.output.lines() {
            lines = lines.child(text(if l.is_empty() { " ".to_owned() } else { l.to_owned() }).font(style).color(t.ink_muted()));
        }
        let status = if self.running { "running".to_owned() } else { self.exit.map_or("stopped".to_owned(), |c| format!("exit {c}")) };
        let mut s = slab(t, self.key.clone()).role("terminal").label(format!("{}, {status}", self.command)).busy(self.running).child(head);
        if !self.output.is_empty() {
            s = s.child(col().key(format!("{}-output", self.key)).w_full().max_h(self.max_h).scroll_xy().child(lines));
        }
        s
    }
}
