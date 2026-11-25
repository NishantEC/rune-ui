use std::collections::BTreeSet;

use rune::El;

use super::super::action_bar::ActionBar;
use super::super::data_table::{DataTable, DataTableState};
use super::super::gallery::{Ctx, GalleryState, StoryItem, spec, specs};
use super::super::icons;
use super::super::table::Column;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("action_bar", "ActionBar", "actions").story("Bulk actions on a table's selection", all::<S>)
}

fn rows() -> Vec<Vec<String>> {
    [("Add a retry to the sync job", "Done"), ("Fix the empty state", "Working"), ("Rename the export command", "Queued"), ("Cache the repository list", "Done"), ("Draw the chart legend", "Failed")]
        .iter()
        .map(|(a, b)| vec![(*a).to_owned(), (*b).to_owned()])
        .collect()
}

/// The table's state, seeded with two rows chosen.
fn bulk(g: &mut GalleryState) -> &mut DataTableState {
    if !g.slots.contains_key("bulk-table") {
        *g.slot_mut::<DataTableState>("bulk-table") = DataTableState { selected: BTreeSet::from([0, 2]), ..Default::default() };
    }
    g.slot_mut::<DataTableState>("bulk-table")
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let st: DataTableState = if g.slots.contains_key("bulk-table") { g.slot("bulk-table") } else { DataTableState { selected: BTreeSet::from([0, 2]), ..Default::default() } };
    let table = DataTable::<S>::new("bulk-table", "Placeholder runs", &st, move |s: &mut S| bulk(acc(s))).column(Column::new("Task")).column(Column::new("Status").width(110.0)).rows(rows()).page_size(5).build(t, c.layer);
    ActionBar::<S>::new("bulk", st.selected.len()).noun("run", "runs")
        .action("Retry", Some(&icons::RETRY), move |s: &mut S| {
            let n = bulk(acc(s)).selected.len();
            acc(s).choose("bulk-did", &format!("retried {n}"));
        })
        .action("Archive", Some(&icons::INBOX), move |s: &mut S| {
            bulk(acc(s)).selected.clear();
            acc(s).choose("bulk-did", "archived");
        })
        .danger("Delete", Some(&icons::TRASH), move |s: &mut S| acc(s).choose("bulk-did", "deleted"))
        .on_clear(move |s: &mut S| bulk(acc(s)).selected.clear())
        .return_focus("bulk-table-select-all")
        .build(t, c.layer);
    specs(t, vec![spec(t, "Choose rows: the bar rises at the bottom of the window. Tab to it, arrows move, Escape clears", rune::col().w_full().max_w(560.0).child(table))])
}
