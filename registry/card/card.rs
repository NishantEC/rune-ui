//! rune-ui registry: card 0.3.0. Copied into your app by `rune-ui add card`; yours to change. Needs: separator.
//!
//! One surface divided into rows, not a stack of cards. A **slab** card is the group's fill a step off the page, no edge and no shadow,
//! its rows split by hairlines inset from its sides. A **raised** card (a result, a question) is the floating colour with a small lift
//! and its hairline baked in. A card may have a header (title, description, an action at the right), content and a footer.
//! [`line`] is the usual row of a settings-like card: a title, a hint under it, a control at the right.
//!
//! ```ignore
//! Card::slab("general").rows(vec![card::line(t, "sync", "Sync", Some("In the background"), Some(switch))]).build(t)
//! Card::raised("result").title("Done").description("Three files changed").child(body).footer(buttons).build(t)
//! ```

use rune::{El, Shadow, col, row, text};
use rune_ui_style::{Recipe, weight};

use super::base::{Theme, paint};
use super::separator::Separator;

pub struct Card {
    key: String,
    raised: bool,
    title: Option<String>,
    description: Option<String>,
    action: Option<El>,
    rows: Option<Vec<El>>,
    children: Vec<El>,
    footer: Option<El>,
}

impl Card {
    /// The group's slab.
    pub fn slab(key: impl Into<String>) -> Card {
        Card { key: key.into(), raised: false, title: None, description: None, action: None, rows: None, children: Vec::new(), footer: None }
    }

    /// A card that lifts a little off the page.
    pub fn raised(key: impl Into<String>) -> Card {
        Card { raised: true, ..Card::slab(key) }
    }

    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    /// Something at the header's right (a button, a badge).
    pub fn action(mut self, a: El) -> Self {
        self.action = Some(a);
        self
    }

    pub fn child(mut self, c: El) -> Self {
        self.children.push(c);
        self
    }

    /// Rows split by inset hairlines, edge to edge (each row brings its own padding: see [`line`]).
    pub fn rows(mut self, rows: Vec<El>) -> Self {
        self.rows = Some(rows);
        self
    }

    pub fn footer(mut self, f: El) -> Self {
        self.footer = Some(f);
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let r = t.radius.xl;
        let pad = t.metrics.inset;
        let recipe = if self.raised { Recipe::new().fill(t.c.floating.bg).shadows(&t.elevation.sm).shadow(Shadow { spread: 1.0, color: t.c.line, ..Shadow::default() }) } else { t.mat.slab.base.clone() };
        let mut c = paint(col().key(self.key).w_full().radius(r).continuous_corners(), &recipe);
        if self.title.is_some() || self.action.is_some() {
            let mut words = col().grow(1.0).shrink().min_w(0.0).gap(t.space.at(1.0));
            if let Some(ti) = self.title {
                words = words.child(text(ti).font(weight(t.ty.md, t.ty.semibold)).color(t.ink()).role("heading").level(3).wrap().min_w(0.0));
            }
            if let Some(d) = self.description {
                words = words.child(text(d).font(t.ty.base).color(t.ink_muted()).wrap().min_w(0.0));
            }
            let mut head = row().w_full().px(pad).pt(pad).pb(if self.rows.is_some() { pad } else { 0.0 }).gap(t.space.at(3.0)).items_start().child(words);
            if let Some(a) = self.action {
                head = head.child(a);
            }
            c = c.child(head);
        }
        if let Some(rows) = self.rows {
            let mut body = col().w_full().clip_rounded(r);
            for (i, row_el) in rows.into_iter().enumerate() {
                if i > 0 {
                    body = body.child(row().w_full().px(pad).child(Separator::horizontal().build(t)));
                }
                body = body.child(row_el);
            }
            c = c.child(body);
        }
        if !self.children.is_empty() {
            c = c.child(col().w_full().padding(pad).gap(t.space.at(3.0)).children(self.children));
        }
        if let Some(f) = self.footer {
            c = c.child(row().w_full().px(pad).pb(pad).gap(t.space.at(2.0)).justify_end().child(f));
        }
        c
    }
}

/// A card's usual row: a title over a quiet hint at the left, a control at the right.
pub fn line(t: &Theme, key: impl Into<String>, title: impl Into<String>, hint: Option<&str>, control: Option<El>) -> El {
    let mut words = col().grow(1.0).shrink().min_w(0.0).gap(2.0).child(text(title.into()).font(weight(t.ty.base, t.ty.medium)).color(t.ink()).wrap().min_w(0.0));
    if let Some(h) = hint {
        words = words.child(text(h.to_owned()).font(t.ty.sm).color(t.ink_muted()).wrap().min_w(0.0));
    }
    let mut l = row().key(key.into()).w_full().min_h(t.metrics.line).px(t.metrics.inset).py(t.metrics.line_py).gap(t.metrics.inset).center_y().child(words);
    if let Some(c) = control {
        l = l.child(c);
    }
    l
}
