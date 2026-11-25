use super::super::gallery::{Ctx, StoryItem, labelled};
use super::super::pagination::Pagination;
use rune_ui_style::Size;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("pagination", "Pagination", "structure").story("Pages", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let page = g.index("pagination", 6);
    let live = Pagination::<S>::new("pagination-demo", page, 20).on_page(move |s: &mut S, p| acc(s).choose("pagination", &p.to_string())).build(t);
    rune::col().gap(t.space.at(4.0))
        .child(labelled(t, "middle", live))
        .child(labelled(t, "first", Pagination::<S>::new("pagination-first", 1, 20).build(t)))
        .child(labelled(t, "last", Pagination::<S>::new("pagination-last", 20, 20).build(t)))
        .child(labelled(t, "few", Pagination::<S>::new("pagination-few", 2, 4).build(t)))
        .child(labelled(t, "compact", Pagination::<S>::new("pagination-small", 3, 9).compact().size(Size::Sm).build(t)))
}
