//! rune-ui registry: popover 0.3.0. Copied into your app by `rune-ui add popover`; yours to change.
//!
//! A floating panel beside its trigger: the **floating** material, a title, anything inside. Behaviour comes from Rune and rune-kit:
//! `rune_kit::popover::anchored` places it beside the trigger and keeps it in the window, `rune_kit::scrim` closes it on a press outside,
//! `El::modal` traps focus and closes it on Escape. The app owns whether it is open. `panel` draws it in place (galleries).
//!
//! ```ignore
//! Popover::new("filters").trigger(button).title("Filters").child(body).open(s.open == Some("filters"))
//!     .on_close(|s: &mut App| s.open = None).build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, col, row, text};
pub use rune_kit::Placement;
use rune_ui_style::weight;

use super::base::{Press, Theme, paint};

pub struct Popover<S: 'static> {
    key: String,
    trigger: Option<El>,
    title: Option<String>,
    body: Vec<El>,
    width: f32,
    open: bool,
    placement: Placement,
    close: Option<Press<S>>,
}

impl<S: 'static> Popover<S> {
    pub fn new(key: impl Into<String>) -> Popover<S> {
        Popover { key: key.into(), trigger: None, title: None, body: Vec::new(), width: 280.0, open: false, placement: Placement::BottomStart, close: None }
    }

    pub fn trigger(mut self, t: El) -> Self {
        self.trigger = Some(t);
        self
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn child(mut self, c: El) -> Self {
        self.body.push(c);
        self
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placement(mut self, p: Placement) -> Self {
        self.placement = p;
        self
    }

    /// A press outside or Escape.
    pub fn on_close(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.close = Some(Rc::new(f));
        self
    }

    /// The panel alone, drawn in place.
    pub fn panel(&mut self, t: &Theme) -> El {
        let mut p = col().key(format!("{}-panel", self.key)).role("dialog").w(self.width).padding(t.metrics.pad_md - 2.0).gap(t.metrics.gap).radius(t.radius.lg).continuous_corners();
        p = paint(p, &t.mat.floating.base);
        if let Some(title) = &self.title {
            p = p.label(title.clone()).child(text(title.clone()).font(weight(t.ty.base, t.ty.semibold)).color(t.c.floating.fg));
        }
        p.children(std::mem::take(&mut self.body))
    }

    /// The trigger in place; the open panel goes into `layer`.
    pub fn build(mut self, t: &Theme, layer: &mut Vec<El>) -> El {
        let anchor = format!("{}-trigger", self.key);
        let trigger = row().key(anchor.clone()).children(self.trigger.take()).expanded(self.open);
        if self.open {
            let close = self.close.clone();
            let mut panel = self.panel(t).focus_return(&anchor);
            if let Some(f) = close.clone() {
                let g = f.clone();
                panel = panel.modal(true, move |s: &mut S| g(s));
                layer.push(rune_kit::scrim(format!("{}-scrim", self.key), move |s: &mut S| f(s)));
            }
            layer.push(rune_kit::popover::anchored(t.to_kit(), panel, &anchor, self.placement, t.space.at(1.5)));
        }
        trigger
    }
}
