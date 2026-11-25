use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::table::{Column, Table};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("table", "Table", "structure").story("Sortable, selectable", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let mut rows: Vec<Vec<String>> = [("Add a retry to the sync job", "Done", "$0.42"), ("Fix the empty state", "Working", "$1.08"), ("Rename the export command", "Queued", "$0.00"), ("Cache the repository list", "Done", "$0.17")].iter().map(|(a, b, c)| vec![(*a).to_owned(), (*b).to_owned(), (*c).to_owned()]).collect();
    let asc = g.flag("table-asc");
    rows.sort_by(|a, b| if asc { a[2].cmp(&b[2]) } else { b[2].cmp(&a[2]) });
    let selected = g.choices.get("table-row").and_then(|v| v.parse().ok());
    let table = Table::<S>::new("table", "Placeholder runs").column(Column::new("Task")).column(Column::new("State").width(110.0)).column(Column::new("Cost").right().width(90.0)).rows(rows).sort(Some((2, asc))).on_sort(move |s: &mut S, _| acc(s).toggle("table-asc")).selected(selected.or(Some(1))).on_select(move |s: &mut S, r| acc(s).choose("table-row", &r.to_string())).build(t);
    specs(t, vec![spec(t, "Four rows, sorted by cost, one selected", rune::col().w(560.0).child(table))])
}
