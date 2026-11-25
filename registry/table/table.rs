//! rune-ui registry: table 0.2.0. Copied into your app by `rune-ui add table`; yours to change.
//!
//! Rows of data under a quiet header, split by hairlines, in a slab. Columns take a width or share what is left, and align left or
//! right (figures right, in tabular figures). A header can show the sort and answer a press; rows wash on hover and may be selected.
//!
//! ```ignore
//! Table::new("runs", "Runs").column(Column::new("Name")).column(Column::new("Cost").right().width(80.0)).rows(rows)
//!     .sort(Some((1, false))).on_sort(|s: &mut App, c| s.sort_by(c)).selected(s.row).on_select(|s: &mut App, r| s.row = Some(r)).build(t)
//! ```

use std::rc::Rc;

use rune::{El, col, row, text};
use rune_ui_style::{tabular, weight};

use super::base::{Choose, States, Theme, glyph, paint, paint_state};
use super::icons;

/// A column: its title, width and alignment.
#[derive(Clone, Debug)]
pub struct Column {
    pub title: String,
    pub width: Option<f32>,
    pub right: bool,
}

impl Column {
    pub fn new(title: impl Into<String>) -> Column {
        Column { title: title.into(), width: None, right: false }
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }

    /// Right-aligned, tabular figures.
    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }
}

pub struct Table<S: 'static> {
    key: String,
    name: String,
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    sort: Option<(usize, bool)>,
    selected: Option<usize>,
    on_sort: Option<Choose<S>>,
    on_select: Option<Choose<S>>,
}

impl<S: 'static> Table<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Table<S> {
        Table { key: key.into(), name: name.into(), columns: Vec::new(), rows: Vec::new(), sort: None, selected: None, on_sort: None, on_select: None }
    }

    pub fn column(mut self, c: Column) -> Self {
        self.columns.push(c);
        self
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    /// The sorted column and whether ascending.
    pub fn sort(mut self, s: Option<(usize, bool)>) -> Self {
        self.sort = s;
        self
    }

    pub fn selected(mut self, r: Option<usize>) -> Self {
        self.selected = r;
        self
    }

    pub fn on_sort(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.on_sort = Some(Rc::new(f));
        self
    }

    pub fn on_select(mut self, f: impl Fn(&mut S, usize) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }

    fn cell(&self, t: &Theme, i: usize, content: El) -> El {
        let c = &self.columns[i];
        let mut cell = row().h_full().px(t.space.at(3.0)).center_y().min_w(0.0);
        cell = if c.right { cell.justify_end() } else { cell };
        cell = match c.width {
            Some(w) => cell.w(w),
            None => cell.grow(1.0).shrink().w(0.0),
        };
        cell.child(content)
    }

    pub fn build(self, t: &Theme) -> El {
        let r = t.radius.xl;
        let mut table = paint(col().key(self.key.clone()).role("table").label(self.name.clone()).w_full().radius(r).continuous_corners().clip_rounded(r), &t.mat.slab.base);
        let mut head = row().key(format!("{}-head", self.key)).role("row").w_full().h(36.0).border_sides([0.0, 0.0, 1.0, 0.0], t.c.line);
        for (i, c) in self.columns.iter().enumerate() {
            let sorted = self.sort.filter(|(at, _)| *at == i);
            let mut words = row().gap(t.space.at(1.0)).center_y().child(text(c.title.clone()).font(weight(t.ty.sm, t.ty.medium)).color(if sorted.is_some() { t.ink() } else { t.ink_muted() }).a11y_hidden());
            if let Some((_, asc)) = sorted {
                words = words.child(glyph(&icons::CHEVRON_DOWN, t.control.icon_sm, t.ink()).rotate(if asc { std::f32::consts::PI } else { 0.0 }));
            }
            let mut cell = self.cell(t, i, words).role("columnheader").label(c.title.clone());
            if let Some(f) = self.on_sort.clone() {
                cell = paint_state(cell.focusable(true), &t.mat.row, States::default()).on_press(move |s: &mut S| f(s, i));
            }
            head = head.child(cell);
        }
        table = table.child(head);
        for (ri, cells) in self.rows.iter().enumerate() {
            let mut line = row().key(format!("{}-row-{ri}", self.key)).role("row").w_full().h(40.0);
            if ri > 0 {
                line = line.border_sides([1.0, 0.0, 0.0, 0.0], t.c.line);
            }
            if let Some(f) = self.on_select.clone() {
                line = paint_state(line.focusable(true).label(cells.first().cloned().unwrap_or_default()), &t.mat.row, States { selected: self.selected == Some(ri), ..States::default() }).on_press(move |s: &mut S| f(s, ri));
            }
            for (i, v) in cells.iter().enumerate().take(self.columns.len()) {
                let style = if self.columns[i].right { tabular(t.ty.base) } else { t.ty.base };
                line = line.child(self.cell(t, i, text(v.clone()).font(style).color(if i == 0 { t.ink() } else { t.ink_muted() }).truncate().min_w(0.0)).role("cell"));
            }
            table = table.child(line);
        }
        table
    }
}
