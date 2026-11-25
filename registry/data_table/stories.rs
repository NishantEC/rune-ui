use std::collections::BTreeSet;

use rune::El;

use super::super::data_table::{DataTable, DataTableState};
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::table::Column;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("data_table", "DataTable", "structure").story("Sort, select, columns, pages", all::<S>)
}

fn rows() -> Vec<Vec<String>> {
    [
        ("Add a retry to the sync job", "Done", "asha@example.com", "$42.10"),
        ("Fix the empty state", "Working", "lee@example.com", "$108.00"),
        ("Rename the export command", "Queued", "sam@example.com", "$0.00"),
        ("Cache the repository list", "Done", "asha@example.com", "$17.25"),
        ("Draw the chart legend", "Failed", "kim@example.com", "$9.80"),
        ("Page the audit log", "Working", "lee@example.com", "$63.40"),
        ("Trim the install script", "Done", "sam@example.com", "$5.15"),
        ("Rotate the signing key", "Queued", "kim@example.com", "$0.00"),
        ("Batch the email digests", "Done", "asha@example.com", "$211.90"),
        ("Index the search table", "Working", "lee@example.com", "$88.70"),
        ("Retire the old importer", "Done", "sam@example.com", "$1.05"),
        ("Translate the settings page", "Queued", "kim@example.com", "$0.00"),
    ]
    .iter()
    .map(|(a, b, c, d)| vec![(*a).to_owned(), (*b).to_owned(), (*c).to_owned(), (*d).to_owned()])
    .collect()
}

fn columns<S: 'static>(d: DataTable<S>) -> DataTable<S> {
    d.column(Column::new("Task")).column(Column::new("Status").width(100.0)).column(Column::new("Owner").width(170.0)).column(Column::new("Cost").right().width(100.0))
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let st: DataTableState = if g.slots.contains_key("data-table") { g.slot("data-table") } else { DataTableState { selected: BTreeSet::from([1, 3]), sort: Some((3, false)), ..Default::default() } };
    let seed = st.clone();
    let live = columns(DataTable::<S>::new("data-table", "Placeholder runs", &st, move |s: &mut S| {
        let g = acc(s);
        if !g.slots.contains_key("data-table") {
            *g.slot_mut::<DataTableState>("data-table") = seed.clone();
        }
        g.slot_mut::<DataTableState>("data-table")
    }))
    .rows(rows())
    .page_size(5)
    .height(220.0)
    .build(t, c.layer);
    let none = DataTableState::default();
    let loading = columns(DataTable::<S>::new("data-loading", "Loading runs", &none, move |s: &mut S| acc(s).slot_mut::<DataTableState>("data-loading"))).rows(rows()).page_size(3).loading(true).build(t, c.layer);
    let empty = columns(DataTable::<S>::new("data-empty", "No runs", &none, move |s: &mut S| acc(s).slot_mut::<DataTableState>("data-empty"))).rows(Vec::new()).build(t, c.layer);
    specs(t, vec![
        spec_wide(t, "Sorted by cost, two rows selected, the body scrolling under its header; press a header, a box, Columns or a page", 720.0, live),
        spec_wide(t, "Loading", 720.0, loading),
        spec_wide(t, "Empty", 720.0, empty),
    ])
}
