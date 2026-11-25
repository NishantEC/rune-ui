//! rune-ui registry: data_table 0.8.0 Copied into your app by `rune-ui add data_table`; yours to change.
//! Needs: table, checkbox, pagination, popup, button.
//!
//! Table's look (a slab of rows split by hairlines under a quiet header, figures right in tabular figures) with what a data table
//! needs on top: headers that sort (a press goes ascending, descending, then back to the data's order), a checkbox per row and one in
//! the header for the page (mixed when some are chosen), a Columns menu that hides and shows columns, pages, a sticky header over a
//! body that scrolls, skeleton rows while loading and a quiet line when there are no rows.
//!
//! The component keeps its own state type ([`DataTableState`]); the app holds one and hands the table a way to reach it. Rows are
//! known by their place in the data you pass, so sorting and paging never lose a selection.
//!
//! ```ignore
//! DataTable::new("runs", "Runs", &s.runs, |s: &mut App| &mut s.runs).column(Column::new("Task")).column(Column::new("Cost").right())
//!     .rows(rows).page_size(10).build(t, &mut layer)
//! ```

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::rc::Rc;

use rune::{El, Side, col, el, row, text};
use rune_ui_style::{ButtonVariant, Size, tabular, weight};

use super::base::{Shape, States, Theme, glyph, paint, paint_state, press};
use super::button::Button;
use super::checkbox::Checkbox;
use super::icons;
use super::pagination::Pagination;
use super::popup::{Entry, Popup, Row, Mount, mount};
use super::table::Column;

/// What the table remembers between frames.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataTableState {
    /// The sorted column and whether ascending; `None` keeps the data's order.
    pub sort: Option<(usize, bool)>,
    /// Chosen rows, by their place in the data.
    pub selected: BTreeSet<usize>,
    /// Hidden columns.
    pub hidden: BTreeSet<usize>,
    /// The page shown, from 0.
    pub page: usize,
    /// Whether the Columns menu is open.
    pub columns_open: bool,
}

pub type Access<S> = Rc<dyn for<'a> Fn(&'a mut S) -> &'a mut DataTableState>;

/// Compares two cells: as numbers when both read as one (ignoring `$`, `,`, `%` and spaces), otherwise as words, ignoring case.
pub fn compare(a: &str, b: &str) -> Ordering {
    let num = |s: &str| s.chars().filter(|c| !matches!(c, '$' | ',' | '%' | ' ')).collect::<String>().parse::<f64>().ok();
    match (num(a), num(b)) {
        (Some(x), Some(y)) => x.partial_cmp(&y).unwrap_or(Ordering::Equal),
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

/// The order rows show in: their places in the data, sorted by `sort`.
pub fn order(rows: &[Vec<String>], sort: Option<(usize, bool)>) -> Vec<usize> {
    let mut ids: Vec<usize> = (0..rows.len()).collect();
    if let Some((c, asc)) = sort {
        let cell = |i: usize| rows[i].get(c).map_or("", String::as_str);
        ids.sort_by(|a, b| {
            let o = compare(cell(*a), cell(*b));
            if asc { o } else { o.reverse() }
        });
    }
    ids
}

pub struct DataTable<S: 'static> {
    key: String,
    name: String,
    state: DataTableState,
    access: Access<S>,
    columns: Vec<Column>,
    rows: Vec<Vec<String>>,
    page_size: usize,
    loading: bool,
    height: f32,
    empty: String,
}

const CHECK_W: f32 = 44.0;

impl<S: 'static> DataTable<S> {
    pub fn new(key: impl Into<String>, name: impl Into<String>, state: &DataTableState, access: impl for<'a> Fn(&'a mut S) -> &'a mut DataTableState + 'static) -> DataTable<S> {
        DataTable { key: key.into(), name: name.into(), state: state.clone(), access: Rc::new(access), columns: Vec::new(), rows: Vec::new(), page_size: 10, loading: false, height: 360.0, empty: "No results.".to_owned() }
    }

    pub fn column(mut self, c: Column) -> Self {
        self.columns.push(c);
        self
    }

    pub fn rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn page_size(mut self, n: usize) -> Self {
        self.page_size = n.max(1);
        self
    }

    /// Skeleton rows in place of the data.
    pub fn loading(mut self, on: bool) -> Self {
        self.loading = on;
        self
    }

    /// The body's most height; past it the rows scroll under the sticky header.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// What shows when there are no rows.
    pub fn empty(mut self, words: impl Into<String>) -> Self {
        self.empty = words.into();
        self
    }

    fn cell(&self, t: &Theme, i: usize, content: El) -> El {
        let c = &self.columns[i];
        let mut cell = row().h_full().px(t.metrics.pad_md - 2.0).center_y().min_w(0.0);
        cell = if c.right { cell.justify_end() } else { cell };
        cell = match c.width {
            Some(w) => cell.w(w),
            None => cell.grow(1.0).shrink().w(0.0),
        };
        cell.child(content)
    }

    pub fn build(self, t: &Theme, layer: &mut Vec<El>) -> El {
        let k = self.key.clone();
        let st = &self.state;
        let acc = self.access.clone();
        let shown: Vec<usize> = (0..self.columns.len()).filter(|c| !st.hidden.contains(c)).collect();
        let ids = order(&self.rows, st.sort);
        let pages = ids.len().div_ceil(self.page_size).max(1);
        let page = st.page.min(pages - 1);
        let on_page: Vec<usize> = ids.iter().copied().skip(page * self.page_size).take(self.page_size).collect();
        let chosen_here = on_page.iter().filter(|i| st.selected.contains(i)).count();

        // The toolbar: how many are chosen, and the Columns menu.
        let trigger = format!("{k}-columns");
        let words = format!("{} of {} row{} selected", st.selected.len(), self.rows.len(), if self.rows.len() == 1 { "" } else { "s" });
        let a = acc.clone();
        let columns = Button::<S>::new(trigger.clone(), "Columns").variant(ButtonVariant::Outline).size(Size::Sm).trailing_icon(&icons::CHEVRON_DOWN).on_press(move |s: &mut S| {
            let st = a(s);
            st.columns_open = !st.columns_open;
        });
        let mut tools = row().w_full().center_y().justify_between().child(text(words).key(format!("{k}-count")).font(tabular(t.ty.sm)).color(t.ink_muted()).live_region(rune::LiveRegion::Polite));
        tools = tools.child(row().key(format!("{trigger}-anchor")).expanded(st.columns_open).child(columns.build(t)));
        if st.columns_open {
            let a = acc.clone();
            let close = press(move |s: &mut S| a(s).columns_open = false);
            let mut popup = Popup::<S>::new(trigger.clone(), "menu", "Columns");
            popup.min_w = 180.0;
            popup.entries.push(Entry::Label("Show columns".to_owned()));
            for (i, c) in self.columns.iter().enumerate() {
                let a = acc.clone();
                let last = shown.len() == 1 && shown[0] == i;
                popup.entries.push(Entry::Row(Row::new(c.title.clone()).checked(!st.hidden.contains(&i)).disabled(last).on_select(move |s: &mut S| {
                    let st = a(s);
                    if !st.hidden.remove(&i) {
                        st.hidden.insert(i);
                    }
                })));
            }
            let surface = popup.surface(t, close.clone());
            let anchor = format!("{trigger}-anchor");
            mount(t, layer, surface, Mount { key: &trigger, anchor: &anchor, side: Side::Bottom, return_to: &trigger }, close);
        }

        // The header: the page's checkbox, then each shown column, sortable.
        let mut head = row().key(format!("{k}-head")).role("row").w_full().h(t.metrics.table_head).fill(t.c.surface.bg).border_sides([0.0, 0.0, 1.0, 0.0], t.c.line).sticky_top(0.0);
        let all_on = !on_page.is_empty() && chosen_here == on_page.len();
        let (a, ids_here) = (acc.clone(), on_page.clone());
        let mut all = Checkbox::<S>::new(format!("{k}-select-all"), "Select all on this page", all_on).hide_label().on_toggle(move |s: &mut S| {
            let st = a(s);
            if ids_here.iter().all(|i| st.selected.contains(i)) {
                for i in &ids_here {
                    st.selected.remove(i);
                }
            } else {
                st.selected.extend(ids_here.iter().copied());
            }
        });
        if chosen_here > 0 && !all_on {
            all = all.mixed(true);
        }
        if self.loading || on_page.is_empty() {
            all = all.disabled(true);
        }
        head = head.child(row().w(CHECK_W).h_full().center().role("columnheader").label("Select").child(all.build(t)));
        for &i in &shown {
            let c = &self.columns[i];
            let sorted = st.sort.filter(|(at, _)| *at == i);
            let mut words = row().gap(t.space.at(1.0)).center_y().child(text(c.title.clone()).font(weight(t.ty.sm, t.ty.medium)).color(if sorted.is_some() { t.ink() } else { t.ink_muted() }).a11y_hidden());
            words = words.child(match sorted {
                Some((_, asc)) => glyph(&icons::CHEVRON_DOWN, t.control.icon_sm, t.ink()).rotate(if asc { std::f32::consts::PI } else { 0.0 }),
                None => glyph(&icons::CHEVRON_UP_DOWN, t.control.icon_sm, t.ink_faint()),
            });
            let a = acc.clone();
            let mut cell = self.cell(t, i, words).key(format!("{k}-head-{i}")).role("columnheader").label(format!("{}, {}", c.title, match sorted {
                Some((_, true)) => "sorted ascending",
                Some((_, false)) => "sorted descending",
                None => "not sorted",
            }));
            cell = paint_state(cell.focusable(true), &t.mat.row, States::default()).on_press(move |s: &mut S| {
                let st = a(s);
                st.sort = match st.sort {
                    Some((at, true)) if at == i => Some((i, false)),
                    Some((at, false)) if at == i => None,
                    _ => Some((i, true)),
                };
            });
            head = head.child(cell);
        }

        // The body.
        let mut body = col().key(format!("{k}-body")).w_full();
        if self.loading {
            for r in 0..self.page_size.min(5) {
                let mut line = row().key(format!("{k}-skeleton-{r}")).w_full().h(t.metrics.table_row).border_sides([if r > 0 { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0], t.c.line).child(el().w(CHECK_W));
                for (n, &i) in shown.iter().enumerate() {
                    let w = if n == 0 { 160.0 } else { 64.0 };
                    line = line.child(self.cell(t, i, el().w(w).h(t.ty.base.size * 0.8).radius(t.radius.sm).round_as(t).fill(t.c.muted.bg).a11y_hidden()));
                }
                body = body.child(line);
            }
            body = body.busy(true).label("Loading");
        } else if on_page.is_empty() {
            body = body.child(row().key(format!("{k}-empty")).role("row").w_full().h(t.metrics.table_row * 3.0).center().child(text(self.empty.clone()).font(t.ty.base).color(t.ink_muted()).role("cell")));
        }
        for (n, &id) in on_page.iter().enumerate().filter(|_| !self.loading) {
            let on = st.selected.contains(&id);
            let cells = &self.rows[id];
            let first = cells.first().cloned().unwrap_or_default();
            let mut line = row().key(format!("{k}-row-{id}")).role("row").label(first.clone()).w_full().h(t.metrics.table_row).checked(on);
            if n > 0 {
                line = line.border_sides([1.0, 0.0, 0.0, 0.0], t.c.line);
            }
            line = paint_state(line, &t.mat.row, States { selected: on, ..States::default() });
            let a = acc.clone();
            let check = Checkbox::<S>::new(format!("{k}-check-{id}"), format!("Select {first}"), on).hide_label().on_toggle(move |s: &mut S| {
                let st = a(s);
                if !st.selected.remove(&id) {
                    st.selected.insert(id);
                }
            });
            line = line.child(row().w(CHECK_W).h_full().center().child(check.build(t)));
            for &i in &shown {
                let v = cells.get(i).cloned().unwrap_or_default();
                let style = if self.columns[i].right { tabular(t.ty.base) } else { t.ty.base };
                line = line.child(self.cell(t, i, text(v).font(style).color(if i == 0 { t.ink() } else { t.ink_muted() }).truncate().min_w(0.0)).role("cell"));
            }
            body = body.child(line);
        }
        let r = t.radius.xl;
        let scroller = col().key(format!("{k}-scroll")).w_full().max_h(self.height).scroll(rune::ScrollSpec { stick_bottom: false }).child(head).child(body);
        let table = paint(col().key(k.clone()).role("table").label(self.name.clone()).w_full().radius(r).round_as(t).clip_rounded(r), &t.mat.slab.base).child(scroller);

        // Pages.
        let a = acc.clone();
        let pager = Pagination::<S>::new(format!("{k}-pages"), page + 1, pages).compact().size(Size::Sm).on_page(move |s: &mut S, p| a(s).page = p - 1).build(t);
        let foot = row().w_full().center_y().justify_between().child(text(format!("Page {} of {pages}", page + 1)).font(tabular(t.ty.sm)).color(t.ink_muted())).child(pager);
        col().key(format!("{k}-data")).w_full().gap(t.space.at(3.0)).child(tools).child(table).child(foot)
    }
}
