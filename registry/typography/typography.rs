//! rune-ui registry: typography 0.5.1. Copied into your app by `rune-ui add typography`; yours to change.
//!
//! shadcn's typography as functions over the theme's type ramp: headings one to four (with their levels for assistive technology),
//! a paragraph, a lead, large, small and muted words, a blockquote, inline code and a list. Reading text wraps at a comfortable
//! measure; headings stay in the first ink, supporting words in the second.
//!
//! ```ignore
//! col().child(typography::h2(t, "Pricing")).child(typography::p(t, "Every plan includes …"))
//! ```

use rune::{El, TextStyle, col, el, row, text};
use rune_ui_style::{tabular, weight};

use super::base::{Shape, Theme};

/// The longest a line of reading text runs, in points (about 70 characters).
pub const MEASURE: f32 = 620.0;

fn heading(words: &str, style: TextStyle, level: u32, t: &Theme) -> El {
    text(words.to_owned()).font(style).color(t.ink()).role("heading").level(level).wrap().min_w(0.0)
}

pub fn h1(t: &Theme, words: &str) -> El {
    heading(words, weight(t.ty.xxl, t.ty.semibold), 1, t)
}

pub fn h2(t: &Theme, words: &str) -> El {
    heading(words, weight(t.ty.xl, t.ty.semibold), 2, t)
}

pub fn h3(t: &Theme, words: &str) -> El {
    heading(words, weight(t.ty.lg, t.ty.semibold), 3, t)
}

pub fn h4(t: &Theme, words: &str) -> El {
    heading(words, weight(t.ty.md, t.ty.semibold), 4, t)
}

/// A paragraph of reading text.
pub fn p(t: &Theme, words: &str) -> El {
    text(words.to_owned()).font(t.ty.md).color(t.ink()).role("paragraph").wrap().max_w(MEASURE).min_w(0.0)
}

/// The paragraph that introduces a page: larger, in the second ink.
pub fn lead(t: &Theme, words: &str) -> El {
    text(words.to_owned()).font(weight(t.ty.lg, t.ty.regular)).color(t.ink_muted()).role("paragraph").wrap().max_w(MEASURE).min_w(0.0)
}

pub fn large(t: &Theme, words: &str) -> El {
    text(words.to_owned()).font(weight(t.ty.lg, t.ty.semibold)).color(t.ink())
}

pub fn small(t: &Theme, words: &str) -> El {
    text(words.to_owned()).font(weight(t.ty.sm, t.ty.medium)).color(t.ink())
}

pub fn muted(t: &Theme, words: &str) -> El {
    text(words.to_owned()).font(t.ty.sm).color(t.ink_muted())
}

/// A quotation: a rule at its left, the words in the second ink.
pub fn blockquote(t: &Theme, words: &str) -> El {
    row().role("blockquote").max_w(MEASURE).gap(t.space.at(4.0)).child(el().w(2.0).self_stretch().radius(1.0).fill(t.c.line).a11y_hidden()).child(text(words.to_owned()).font(t.ty.md).color(t.ink_muted()).wrap().min_w(0.0).grow(1.0).shrink())
}

/// Code inside a line: monospaced on a quiet tint.
pub fn inline_code(t: &Theme, words: &str) -> El {
    let mono = TextStyle { family: rune_ui_style::fonts::find("mono").map_or(t.ty.sm.family, |f| f.family), ..tabular(t.ty.sm) };
    row().role("code").px(t.space.at(1.5)).py(t.space.at(0.5)).radius(t.radius.xs).round_as(t).fill(t.c.muted.bg).child(text(words.to_owned()).font(mono).color(t.ink()))
}

/// A bulleted list.
pub fn list(t: &Theme, items: &[&str]) -> El {
    let mut c = col().role("list").gap(t.space.at(2.0)).max_w(MEASURE);
    for i in items {
        c = c.child(row().role("listitem").gap(t.space.at(3.0)).items_start().child(el().mt(t.ty.md.line_height / 2.0 - 2.0).size(4.0, 4.0).radius(2.0).fill(t.ink_muted()).a11y_hidden()).child(text((*i).to_owned()).font(t.ty.md).color(t.ink()).wrap().min_w(0.0).grow(1.0).shrink()));
    }
    c
}
