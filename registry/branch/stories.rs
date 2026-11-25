use rune::El;

use super::super::branch::Branch;
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("branch", "Branch", "agent").story("Between alternative answers", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let at = g.index("branch", 1);
    specs(t, vec![
        spec(t, "Second of three", Branch::<S>::new("branch-demo", at, 3).on_change(move |s: &mut S, i| acc(s).choose("branch", &i.to_string())).build(t)),
        spec(t, "First of two (back is off)", Branch::<S>::new("branch-first", 0, 2).on_change(|_: &mut S, _| {}).build(t)),
    ])
}
