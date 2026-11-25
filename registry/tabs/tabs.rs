//! rune-ui registry: tabs 0.4.1. Copied into your app by `rune-ui add tabs`; yours to change.
//!
//! Views of one place. **Line**: words over a hairline, the chosen tab in the first ink with a 2 point ink line under it. **Contained**:
//! a well track holding the chosen tab raised (like a toggle group). A tab may lead with an icon and end with a count. Left and Right
//! move between tabs; the content under them is yours (`content`).
//!
//! ```ignore
//! Tabs::new("project", "Project views").tab("Overview", None).tab("Activity", Some(3)).selected(s.tab).on_change(|s: &mut App, i| s.tab = i).build(t)
//! ```

use std::rc::Rc;

use rune::{Color, El, Icon, Key, KeyEvent, col, el, row, text};
use rune_ui_style::{Delta, Radius, tabular};

use super::base::{Choose, Shape, State, States, Theme, glyph, paint, paint_state};

/// The two looks.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    #[default]
    Line,
    Contained,
}

pub struct Tabs<S: 'static> {
    key: String,
    name: String,
    tabs: Vec<(String, Option<&'static Icon>, Option<usize>)>,
    selected: usize,
    variant: TabsVariant,
    pinned: Option<(usize, State)>,
    content: Option<El>,
    change: Option<Choose<S>>,
}

impl<S: 'static> Tabs<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Tabs<S> {
        Tabs { key: key.into(), name: name.into(), tabs: Vec::new(), selected: 0, variant: TabsVariant::Line, pinned: None, content: None, change: None }
    }

    /// A tab and an optional count after its words.
    pub fn tab(mut self, label: impl Into<String>, count: Option<usize>) -> Self {
        self.tabs.push((label.into(), None, count));
        self
    }

    /// An icon before the last tab's words.
    pub fn icon(mut self, i: &'static Icon) -> Self {
        if let Some(t) = self.tabs.last_mut() {
            t.1 = Some(i);
        }
        self
    }

    pub fn selected(mut self, i: usize) -> Self {
        self.selected = i;
        self
    }

    pub fn variant(mut self, v: TabsVariant) -> Self {
        self.variant = v;
        self
    }

    pub fn state(mut self, i: usize, s: State) -> Self {
        self.pinned = Some((i, s));
        self
    }

    /// What the chosen tab shows, under the tabs.
    pub fn content(mut self, c: El) -> Self {
        self.content = Some(c);
        self
    }

    pub fn on_change(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.change = Some(Rc::new(f));
        self
    }

    pub fn build(self, t: &Theme) -> El {
        let line = self.variant == TabsVariant::Line;
        let h = t.control.md;
        let n = self.tabs.len();
        let mut strip = row().key(self.key.clone()).role("tablist").label(self.name.clone());
        strip = if line { strip.w_full().gap(t.metrics.inset - 4.0).border_sides([0.0, 0.0, 1.0, 0.0], t.c.line) } else { paint(strip.h(h).padding(3.0).gap(2.0).center_y().radius(t.radius.md).round_as(t), &t.mat.well.base) };
        let chosen = t.mat.raised.clone().on(State::Pressed, Delta::new());
        for (i, (label, icon, count)) in self.tabs.into_iter().enumerate() {
            let on = i == self.selected;
            let mut st = States::default();
            if let Some((at, s)) = self.pinned
                && at == i
            {
                st.set(s, true);
            }
            let ink = if on { t.ink() } else { t.ink_muted() };
            let mut face = row().gap(t.space.at(1.5)).center_y();
            if let Some(ic) = icon {
                face = face.child(glyph(ic, t.control.icon, ink));
            }
            face = face.child(text(label.clone()).font(rune_ui_style::weight(t.ty.base, if on { t.ty.medium } else { t.ty.regular })).color(ink).a11y_hidden());
            if let Some(c) = count {
                face = face.child(text(c.to_string()).font(tabular(t.ty.xs)).color(t.ink_faint()).a11y_hidden());
            }
            let mut cell = if line {
                let face = paint_state(face.h(h).px(t.space.at(1.5)).radius(t.radius.sm).round_as(t), &t.mat.row, States { focus_visible: false, ..st });
                let bar = el().w_full().h(2.0).radius(1.0).fill(if on { t.c.primary.bg } else { Color::TRANSPARENT }).a11y_hidden();
                let mut c = col().gap(t.space.at(0.5)).child(face).child(bar);
                if st.focus_visible {
                    c = paint(c, &t.mat.ghost.state(State::FocusVisible));
                }
                c.focus_ring(t.control.ring, t.control.ring_offset, t.c.ring)
            } else {
                paint_state(face.h(h - 6.0).px(t.space.at(2.5)).radius(Radius::inner(t.radius.md, 3.0)).round_as(t), if on { &chosen } else { &t.mat.row }, st)
            };
            cell = cell.key(format!("{}-{i}", self.key)).role("tab").label(label).checked(on).focusable(true);
            if let Some(f) = self.change.clone() {
                let g = f.clone();
                cell = cell.on_press(move |s: &mut S| g(s, i)).on_key(move |s: &mut S, k: &KeyEvent, _| match k.key {
                    Key::Right => {
                        f(s, (i + 1) % n);
                        true
                    }
                    Key::Left => {
                        f(s, (i + n - 1) % n);
                        true
                    }
                    _ => false,
                });
            }
            strip = strip.child(cell);
        }
        match self.content {
            Some(c) => col().w_full().gap(t.space.at(4.0)).child(strip).child(col().key(format!("{}-panel", self.key)).role("tabpanel").w_full().child(c)),
            None => strip,
        }
    }
}
