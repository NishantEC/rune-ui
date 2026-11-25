//! rune-ui registry: avatar 0.4.0. Copied into your app by `rune-ui add avatar`; yours to change.
//!
//! A person as their initials in a circle (or an image, clipped to it, with the 1 point outline every image gets so it never bleeds into
//! the page). Three sizes. [`group`] stacks a few, overlapping, each ringed in the page colour, with a "+N" for the rest.
//!
//! ```ignore
//! Avatar::new("me", "Ada Lovelace").build(t)
//! avatar::group(t, "team", &["Ada Lovelace", "Alan Turing"], 3)
//! ```

use rune::{El, Shadow, row, text};
use rune_ui_style::{Recipe, Size, tabular, weight};

use super::base::{Shape, Theme, paint};

pub struct Avatar<'a> {
    key: String,
    name: String,
    image: Option<&'a rune::Pixels>,
    size: Size,
}

/// The first letters of the first two words.
pub fn initials(name: &str) -> String {
    name.split_whitespace().filter_map(|w| w.chars().next()).take(2).collect::<String>().to_uppercase()
}

impl<'a> Avatar<'a> {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Avatar<'a> {
        Avatar { key: key.into(), name: name.into(), image: None, size: Size::Md }
    }

    pub fn image(mut self, p: &'a rune::Pixels) -> Self {
        self.image = Some(p);
        self
    }

    pub fn size(mut self, s: Size) -> Self {
        self.size = s;
        self
    }

    fn diameter(&self) -> f32 {
        match self.size {
            Size::Sm => 24.0,
            Size::Lg => 40.0,
            _ => 32.0,
        }
    }

    pub fn build(self, t: &Theme) -> El {
        let d = self.diameter();
        let base = row().key(self.key).role("img").label(self.name.clone()).size(d, d).center().radius(d / 2.0).round_as(t);
        match self.image {
            // An image's outline is pure black or white at 10%, never a tinted grey (it would read as dirt).
            Some(p) => base.pixels(p).box_shadow(Shadow { spread: 1.0, inset: true, color: if t.dark { rune::Color::WHITE.alpha(0.1) } else { rune::Color::BLACK.alpha(0.1) }, ..Shadow::default() }), // guard: allow
            None => {
                let style = tabular(weight(if d < 30.0 { t.ty.xs } else { t.ty.sm }, t.ty.medium));
                paint(base, &Recipe::new().fill(t.c.surface.bg).fill(t.c.muted.bg)).child(text(initials(&self.name)).font(style).color(t.ink_muted()).a11y_hidden())
            }
        }
    }
}

/// A few avatars overlapping, ringed in the page colour, and "+N" for those not shown.
pub fn group(t: &Theme, key: &str, names: &[&str], show: usize) -> El {
    let mut r = row().key(key.to_owned()).role("group").label(format!("{} people", names.len()));
    for (i, n) in names.iter().take(show).enumerate() {
        let a = Avatar::new(format!("{key}-{i}"), *n).size(Size::Sm).build(t).box_shadow(Shadow { spread: 2.0, color: t.c.page.bg, ..Shadow::default() });
        r = r.child(if i > 0 { a.ml(-6.0) } else { a });
    }
    if names.len() > show {
        let more = paint(row().size(24.0, 24.0).center().radius(12.0).round_as(t).ml(-6.0), &Recipe::new().fill(t.c.surface.bg).shadow(Shadow { spread: 2.0, color: t.c.page.bg, ..Shadow::default() }));
        r = r.child(more.child(text(format!("+{}", names.len() - show)).font(tabular(weight(t.ty.xs, t.ty.medium))).color(t.ink_muted()).a11y_hidden()));
    }
    r
}
