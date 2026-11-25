//! rune-ui registry: aspect_ratio 0.5.1. Copied into your app by `rune-ui add aspect_ratio`; yours to change.
//!
//! A box that keeps a width-to-height ratio as its width changes (16:9 for video, 1:1 for avatars), rounded and clipping what it
//! holds. Rune's own `aspect_ratio` does the sizing; the content fills the box.
//!
//! ```ignore
//! AspectRatio::new("cover", 16.0 / 9.0).child(picture).build(t)
//! ```

use rune::{El, col};

use super::base::{Shape, Theme};

pub struct AspectRatio {
    key: String,
    ratio: f32,
    content: Option<El>,
    rounded: bool,
}

impl AspectRatio {
    /// `ratio` is width over height.
    pub fn new(key: impl Into<String>, ratio: f32) -> AspectRatio {
        AspectRatio { key: key.into(), ratio, content: None, rounded: true }
    }

    pub fn child(mut self, e: El) -> Self {
        self.content = Some(e);
        self
    }

    /// Square corners instead of the large radius.
    pub fn square(mut self) -> Self {
        self.rounded = false;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let r = if self.rounded { t.radius.lg } else { 0.0 };
        let mut b = col().key(self.key).w_full().aspect_ratio(self.ratio).radius(r).round_as(t).clip_rounded(r);
        if let Some(c) = self.content {
            b = b.child(c.w_full().h_full());
        }
        b
    }
}
