//! rune-ui registry: badge 0.1.0. Copied into your app by `rune-ui add badge`; yours to change.
//!
//! Colour says a state, at low alpha, and never alone. A [`Badge`] is a count ("3") or a dot; a [`StatusPill`] is a meaning's wash with
//! a 6 point dot and its words (hollow dot for something off or a draft, a turning ring for work in progress). Figures are tabular.
//!
//! ```ignore
//! Badge::count("unread", 3).tone(Tone::Info).build(t)
//! StatusPill::new("state", Tone::Success, "Ready").build(t)
//! ```

use rune::{Color, El, el, row, text};
use rune_ui_style::{tabular, weight};

use super::base::{Theme, spinner};

/// A meaning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

impl Tone {
    pub const ALL: [Tone; 5] = [Tone::Neutral, Tone::Info, Tone::Success, Tone::Warning, Tone::Danger];

    /// Words, dot and wash.
    pub fn colors(self, t: &Theme) -> (Color, Color, Color) {
        let tone = match self {
            Tone::Neutral => return (t.c.ink_muted, t.c.ink_faint, t.c.selected),
            Tone::Info => t.c.info,
            Tone::Success => t.c.success,
            Tone::Warning => t.c.warning,
            Tone::Danger => t.c.danger,
        };
        (tone.ink, tone.dot, tone.wash)
    }

    pub fn word(self) -> &'static str {
        match self {
            Tone::Neutral => "Neutral",
            Tone::Info => "Info",
            Tone::Success => "Success",
            Tone::Warning => "Warning",
            Tone::Danger => "Danger",
        }
    }
}

/// A count or a dot: 16 high, at least 16 wide, fully round.
pub struct Badge {
    key: String,
    count: Option<usize>,
    tone: Tone,
    solid: bool,
    max: usize,
}

impl Badge {
    pub fn count(key: impl Into<String>, n: usize) -> Badge {
        Badge { key: key.into(), count: Some(n), tone: Tone::Neutral, solid: false, max: 99 }
    }

    /// A 6 point dot with no figure (something changed since you looked).
    pub fn dot(key: impl Into<String>) -> Badge {
        Badge { key: key.into(), count: None, tone: Tone::Info, solid: false, max: 99 }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// The loud form, for what failed: the solid danger colour with its own words colour.
    pub fn solid(mut self) -> Self {
        self.solid = true;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let (ink, dot, wash) = self.tone.colors(t);
        let (ink, wash) = if self.solid { (t.c.on_danger, t.c.danger_solid) } else { (ink, wash) };
        match self.count {
            None => el().key(self.key).size(6.0, 6.0).radius(3.0).fill(dot).a11y_hidden(),
            Some(n) => {
                let words = if n > self.max { format!("{}+", self.max) } else { n.to_string() };
                row().key(self.key).role("status").label(words.clone()).h(16.0).min_w(16.0).px(5.0).center().radius(8.0).fill(wash).child(text(words).font(tabular(weight(t.ty.xs, t.ty.medium))).color(ink).a11y_hidden())
            }
        }
    }
}

/// How a status pill's dot is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mark {
    #[default]
    Dot,
    /// Off, a draft, not started.
    Hollow,
    /// Work in progress (the one thing that loops).
    Working,
}

/// A meaning's wash, a 6 point dot and the words: 20 high, fully round.
pub struct StatusPill {
    key: String,
    tone: Tone,
    words: String,
    mark: Mark,
}

impl StatusPill {
    pub fn new(key: impl Into<String>, tone: Tone, words: impl Into<String>) -> StatusPill {
        StatusPill { key: key.into(), tone, words: words.into(), mark: Mark::Dot }
    }

    pub fn mark(mut self, m: Mark) -> Self {
        self.mark = m;
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let (ink, dot, wash) = self.tone.colors(t);
        let mark = match self.mark {
            Mark::Dot => el().size(6.0, 6.0).radius(3.0).fill(dot).a11y_hidden(),
            Mark::Hollow => el().size(6.0, 6.0).radius(3.0).border(1.5, dot).a11y_hidden(),
            Mark::Working => spinner(9.0, dot, &self.words),
        };
        row()
            .key(self.key)
            .role("status")
            .label(self.words.clone())
            .h(20.0)
            .pl(t.space.at(1.75))
            .pr(t.space.at(2.0))
            .gap(t.space.at(1.25))
            .center_y()
            .radius(10.0)
            .fill(wash)
            .child(mark)
            .child(text(self.words).font(weight(t.ty.sm, t.ty.medium)).color(ink).truncate().min_w(0.0).a11y_hidden())
    }
}
