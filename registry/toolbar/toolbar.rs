//! rune-ui registry: toolbar 0.8.0. Copied into your app by `rune-ui add toolbar`; yours to change.
//! Needs: popup.
//!
//! A row of controls in groups (formatting, alignment, history), split by hairlines. It is one Tab stop: Left and Right move between
//! its controls, Home and End go to the ends, and Tab leaves. When the row is narrower than its controls, the ones that do not fit
//! move, in order, into a More menu at its end, from the overlay layer; the menu's rows do what the controls did (a toggle shows its
//! check). Each control is a ghost button with a 40 point hit area; a toggle that is on is **selected** (a well), not coloured.
//!
//! Widths are not known when the view is built, so the toolbar measures itself and its controls (`El::on_layout`) into a
//! [`ToolbarState`] the app keeps, and the next frame moves what does not fit. The first frame shows everything.
//!
//! ```ignore
//! Toolbar::new("format", "Formatting", &s.bar, |s: &mut App| &mut s.bar)
//!     .group(vec![Tool::toggle("Bold", &icons::BOLD, s.bold, |s: &mut App| s.bold = !s.bold), Tool::button("Undo", &icons::RETRY, |s: &mut App| s.undo())])
//!     .build(t, &mut layer)
//! ```

use std::rc::Rc;

use rune::{El, Icon, Side, el, row, text};
use rune_ui_style::Radius;

use super::base::{Press, Shape, States, Theme, glyph, hit_area, paint_state, press};
use super::icons;
use super::popup::{Entry, Mount, Popup, Row, mount};

/// What the toolbar measured, and whether its More menu is open. Keep one per toolbar in the app's state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToolbarState {
    /// The toolbar's own width (0 before it was laid out: everything shows).
    pub width: f32,
    /// Each control's width, by its place among all the controls (0 until measured).
    pub widths: Vec<f32>,
    pub more_open: bool,
}

/// One control.
pub struct Tool<S: 'static> {
    label: String,
    icon: Option<&'static Icon>,
    words: bool,
    on: Option<bool>,
    disabled: bool,
    press: Option<Press<S>>,
}

impl<S: 'static> Tool<S> {
    /// An icon button; `label` is its name and tooltip.
    pub fn button(label: impl Into<String>, icon: &'static Icon, f: impl Fn(&mut S) + 'static) -> Tool<S> {
        Tool { label: label.into(), icon: Some(icon), words: false, on: None, disabled: false, press: Some(press(f)) }
    }

    /// A button that shows its words (and no icon).
    pub fn text(label: impl Into<String>, f: impl Fn(&mut S) + 'static) -> Tool<S> {
        Tool { label: label.into(), icon: None, words: true, on: None, disabled: false, press: Some(press(f)) }
    }

    /// An icon toggle that says whether it is down.
    pub fn toggle(label: impl Into<String>, icon: &'static Icon, on: bool, f: impl Fn(&mut S) + 'static) -> Tool<S> {
        Tool { on: Some(on), ..Tool::button(label, icon, f) }
    }

    /// Shows its words beside its icon.
    pub fn with_words(mut self) -> Self {
        self.words = true;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }
}

pub type Access<S> = Rc<dyn for<'a> Fn(&'a mut S) -> &'a mut ToolbarState>;

pub struct Toolbar<S: 'static> {
    key: String,
    name: String,
    state: ToolbarState,
    access: Access<S>,
    groups: Vec<Vec<Tool<S>>>,
}

/// How many of `widths` (with `gap` between them and `sep` more where a group starts, at `starts`) fit in `avail`, leaving room for a
/// More button of `more` when any are left out. A width of 0 (not measured yet) counts as `guess`.
pub fn fitting(widths: &[f32], starts: &[bool], avail: f32, gap: f32, sep: f32, more: f32) -> usize {
    if avail <= 0.0 {
        return widths.len();
    }
    let n = widths.len();
    let mut used = 0.0;
    for i in 0..n {
        let w = widths[i] + if i > 0 { gap + if starts[i] { sep } else { 0.0 } } else { 0.0 };
        let rest = if i + 1 < n { gap + more } else { 0.0 };
        if used + w + rest > avail + 0.5 {
            return i;
        }
        used += w;
    }
    n
}

impl<S: 'static> Toolbar<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, state: &ToolbarState, access: impl for<'a> Fn(&'a mut S) -> &'a mut ToolbarState + 'static) -> Toolbar<S> {
        Toolbar { key: key.into(), name: name.into(), state: state.clone(), access: Rc::new(access), groups: Vec::new() }
    }

    /// A group of controls; groups are split by a hairline.
    pub fn group(mut self, tools: Vec<Tool<S>>) -> Self {
        self.groups.push(tools);
        self
    }

    fn face(t: &Theme, key: String, tool: &Tool<S>, first: bool, measure: impl Fn(&mut S, [f32; 4]) + 'static) -> El {
        let h = t.control.md;
        let st = States { disabled: tool.disabled, selected: tool.on == Some(true), ..States::default() };
        let ink = if tool.disabled { t.ink_faint() } else if tool.on == Some(true) { t.ink() } else { t.ink_muted() };
        let mut face = row().key(key.clone()).role("button").label(tool.label.clone()).focusable(!tool.disabled).focus_selected(first).h(h).gap(t.metrics.gap - 2.0).center().radius(Radius::fit(t.radius.md, h)).round_as(t);
        if let Some(on) = tool.on {
            face = face.checked(on);
        }
        face = if tool.words { face.px(t.metrics.pad_sm) } else { face.w(h).help(tool.label.clone()) };
        face = paint_state(face, &t.mat.ghost, st);
        if let Some(i) = tool.icon {
            face = face.child(glyph(i, t.control.icon, ink));
        }
        if tool.words {
            face = face.child(text(tool.label.clone()).font(t.ty.base).color(ink).a11y_hidden());
        }
        if tool.disabled {
            face = face.disabled(true);
        }
        face = face.on_layout(measure);
        let press = tool.press.clone().filter(|_| !tool.disabled);
        if let Some(f) = press.clone() {
            face = face.on_press(move |s: &mut S| f(s));
        }
        let slop = t.control.slop(h);
        hit_area(&key, face, slop.min(t.space.at(0.5)), slop, press)
    }

    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let k = self.key.clone();
        let st = &self.state;
        let gap = t.space.at(1.0);
        let sep = 1.0 + 2.0 * t.space.at(1.0);
        let more_w = t.control.md;
        let tools: Vec<(bool, Tool<S>)> = self.groups.into_iter().flat_map(|g| g.into_iter().enumerate().map(|(i, tool)| (i == 0, tool))).collect();
        let starts: Vec<bool> = tools.iter().enumerate().map(|(i, (s, _))| *s && i > 0).collect();
        let widths: Vec<f32> = tools.iter().enumerate().map(|(i, (_, tool))| {
            let measured = st.widths.get(i).copied().unwrap_or(0.0);
            if measured > 0.0 { measured } else if tool.words { t.control.md + tool.label.chars().count() as f32 * t.ty.base.size * 0.55 } else { t.control.md }
        }).collect();
        let shown = fitting(&widths, &starts, st.width, gap, sep, more_w);
        let n = tools.len();
        let acc = self.access.clone();
        let mut bar = row().key(k.clone()).role("toolbar").label(self.name.clone()).focus_group().w_full().min_w(0.0).gap(gap).center_y().on_layout(move |s: &mut S, r: [f32; 4]| {
            let st = acc(s);
            if (st.width - r[2]).abs() > 0.5 {
                st.width = r[2];
            }
        });
        let mut rest: Vec<Tool<S>> = Vec::new();
        for (i, (start, tool)) in tools.into_iter().enumerate() {
            if i >= shown {
                rest.push(tool);
                continue;
            }
            if start && i > 0 {
                bar = bar.child(el().w(1.0).h(t.control.sm - 6.0).mx(t.space.at(1.0) - gap).fill(t.c.line).a11y_hidden());
            }
            let acc = self.access.clone();
            let face = Self::face(t, format!("{k}-{i}"), &tool, i == 0, move |s: &mut S, r: [f32; 4]| {
                let st = acc(s);
                if st.widths.len() < n {
                    st.widths.resize(n, 0.0);
                }
                if (st.widths[i] - r[2]).abs() > 0.5 {
                    st.widths[i] = r[2];
                }
            });
            bar = bar.child(face);
        }
        if !rest.is_empty() {
            let more = format!("{k}-more");
            let acc = self.access.clone();
            let open = st.more_open;
            let h = t.control.md;
            let mut face = row().key(more.clone()).role("button").label("More tools").help("More tools").expanded(open).focusable(true).size(h, h).center().radius(Radius::fit(t.radius.md, h)).round_as(t);
            face = paint_state(face, &t.mat.ghost, States { selected: open, ..States::default() }).child(glyph(&icons::MORE, t.control.icon, t.ink_muted()));
            let toggle = press(move |s: &mut S| {
                let st = acc(s);
                st.more_open = !st.more_open;
            });
            let f = toggle.clone();
            face = face.on_press(move |s: &mut S| f(s));
            let slop = t.control.slop(h);
            bar = bar.child(row().key(format!("{more}-anchor")).child(hit_area(&more, face, slop.min(t.space.at(0.5)), slop, Some(toggle))));
            if open {
                let acc = self.access.clone();
                let close = press(move |s: &mut S| acc(s).more_open = false);
                let mut popup = Popup::<S>::new(more.clone(), "menu", "More tools");
                for (j, tool) in rest.into_iter().enumerate() {
                    let mut r = Row::new(tool.label).key(format!("{more}-{}", shown + j)).disabled(tool.disabled);
                    if let Some(i) = tool.icon {
                        r = r.icon(i);
                    }
                    if let Some(on) = tool.on {
                        r = r.checked(on);
                    }
                    if let Some(f) = tool.press {
                        r = r.on_select(move |s: &mut S| f(s));
                    }
                    popup.entries.push(Entry::Row(r));
                }
                let surface = popup.surface(t, close.clone());
                let anchor = format!("{more}-anchor");
                mount(t, layer, surface, Mount { key: &more, anchor: &anchor, side: Side::Bottom, return_to: &more }, close);
            }
        }
        bar
    }
}
