//! rune-ui registry: skeleton 0.4.0. Copied into your app by `rune-ui add skeleton`; yours to change.
//!
//! The shape of what is loading, still (no shimmer: a frequent wait should not move). A block, lines of text (the last one shorter), a
//! circle, or a row (a circle and two lines). Each group says "Loading" to assistive technology once.
//!
//! ```ignore
//! Skeleton::lines(t, "loading", 3, 280.0)
//! ```

use rune::{El, col, el, row};

use super::base::{Shape, Theme};

pub struct Skeleton;

impl Skeleton {
    pub fn block(t: &Theme, key: impl Into<String>, w: f32, h: f32) -> El {
        el().key(key.into()).w(w).h(h).radius(t.radius.sm).round_as(t).fill(t.c.muted.bg).a11y_hidden()
    }

    pub fn circle(t: &Theme, key: impl Into<String>, d: f32) -> El {
        el().key(key.into()).size(d, d).radius(d / 2.0).round_as(t).fill(t.c.muted.bg).a11y_hidden()
    }

    pub fn lines(t: &Theme, key: impl Into<String>, n: usize, w: f32) -> El {
        let key = key.into();
        let mut c = col().key(key.clone()).role("progressbar").label("Loading").gap(t.space.at(2.0)).w(w);
        for i in 0..n {
            c = c.child(Skeleton::block(t, format!("{key}-{i}"), if i + 1 == n && n > 1 { w * 0.6 } else { w }, 10.0));
        }
        c
    }

    pub fn row(t: &Theme, key: impl Into<String>, w: f32) -> El {
        let key = key.into();
        row().key(key.clone()).role("progressbar").label("Loading").w(w).gap(t.space.at(3.0)).center_y().child(Skeleton::circle(t, format!("{key}-face"), 32.0)).child(col().grow(1.0).gap(t.space.at(1.5)).child(Skeleton::block(t, format!("{key}-a"), w * 0.6, 10.0)).child(Skeleton::block(t, format!("{key}-b"), w * 0.35, 8.0)))
    }
}
