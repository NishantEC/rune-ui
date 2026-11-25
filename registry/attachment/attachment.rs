//! rune-ui registry: attachment 0.6.0. Copied into your app by `rune-ui add attachment`; yours to change.
//!
//! A file or image sent with a message: a flat muted pill (it holds something, so it is not raised) with an icon, the name (cut in the
//! middle when long, so the extension stays), a quiet size, and a remove button while the message is still being written.
//!
//! ```ignore
//! Attachment::new("a1", "report.pdf").detail("240 KB").on_remove(|s: &mut App| s.files.remove(0)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, Tx, row, text};

use super::agent::{PressTx, icon_button, meta};
use super::base::{Press, Theme, glyph};
use super::icons;

/// What the attachment is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Kind {
    #[default]
    File,
    Image,
}

pub struct Attachment<S: 'static> {
    key: String,
    name: String,
    kind: Kind,
    detail: Option<String>,
    remove: Option<Press<S>>,
}

impl<S: 'static> Attachment<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Attachment<S> {
        Attachment { key: key.into(), name: name.into(), kind: Kind::File, detail: None, remove: None }
    }

    pub fn kind(mut self, k: Kind) -> Self {
        self.kind = k;
        self
    }

    /// Quiet words after the name (a size, a page count).
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }

    /// Shows the remove button.
    pub fn on_remove(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.remove = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let h = t.control.sm + 6.0;
        let icon = if self.kind == Kind::Image { &icons::IMAGE } else { &icons::FILE };
        let mut chip = row().key(self.key.clone()).role("group").label(format!("Attachment {}", self.name)).h(h).max_w(260.0).pl(t.space.at(2.5)).pr(if self.remove.is_some() { t.space.at(0.5) } else { t.space.at(3.0) }).gap(t.space.at(2.0)).center_y().radius(h / 2.0).fill(t.c.muted.bg)
            .child(glyph(icon, t.control.icon, t.ink_muted()))
            .child(text(self.name.clone()).font(t.ty.sm).color(t.ink()).truncate_middle().min_w(0.0).shrink().a11y_hidden());
        if let Some(d) = self.detail {
            chip = chip.child(meta(t, d).a11y_hidden());
        }
        if let Some(f) = self.remove {
            let press: PressTx<S> = Rc::new(move |s: &mut S, _: &mut Tx| f(s));
            chip = chip.child(icon_button(t, &format!("{}-remove", self.key), &format!("Remove {}", self.name), &icons::CLOSE, None, false, Some(press)));
        }
        chip
    }
}
