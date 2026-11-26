use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::reorder::{Reorder, move_item};
use super::super::sidebar::SidebarItem;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("reorder", "Reorder", "structure").story("Drag a row, or Alt with Up and Down", all::<S>)
}

/// The demo's order: indexes into `NAMES`; empty means the order as written.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Order(pub Vec<usize>);

const NAMES: [&str; 4] = ["Acme web", "Acme mobile", "Docs", "Scratch"];

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let order = match g.slot::<Order>("reorder-order").0 {
        o if o.is_empty() => (0..NAMES.len()).collect::<Vec<_>>(),
        o => o,
    };
    let rows: Vec<El> = order.iter().map(|&k| SidebarItem::<S>::new(format!("reorder-item{k}"), NAMES[k]).icon(&icons::FOLDER).build(t)).collect();
    let list = Reorder::<S>::new("reorder-demo", "Workspaces", t.metrics.row + 1.0).rows(rows).on_move(move |s: &mut S, from, to| {
        let o = &mut acc(s).slot_mut::<Order>("reorder-order").0;
        if o.is_empty() {
            *o = (0..NAMES.len()).collect();
        }
        move_item(o, from, to);
    }).build(t);
    specs(t, vec![spec(t, "Focus a row and press Alt with Up, Down, Home or End; or drag it and let go where it belongs", rune::col().w(240.0).child(list))])
}
