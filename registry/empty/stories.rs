use super::super::button::Button;
use super::super::empty::Empty;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("empty", "Empty", "display").story("Nothing yet", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, acc) = (c.t, c.access);
    let slab = |e: rune::El| rune_ui_style::paint(rune::col().w(380.0).radius(t.radius.xl), &t.mat.slab.base).child(e);
    specs(t, vec![
        spec(t, "With an action", slab(Empty::new("empty-tasks", &icons::INBOX, "No tasks yet").description("Tasks you start show up here, newest first.").action(Button::<S>::new("empty-new", "New task").icon(&icons::PLUS).on_press(move |s: &mut S| acc(s).presses += 1).build(t)).build(t))),
        spec(t, "Words only", slab(Empty::new("empty-search", &icons::SEARCH, "Nothing matches").description("Try fewer words.").build(t))),
    ])
}
