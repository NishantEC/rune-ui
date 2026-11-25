use rune::El;

use super::super::checkpoint::Checkpoint;
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("checkpoint", "Checkpoint", "agent").story("A divider with Restore", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, acc) = (c.t, c.access);
    specs(t, vec![
        spec(t, "Restorable", rune::col().w(560.0).child(Checkpoint::<S>::new("checkpoint-demo", "Checkpoint · 14:02").on_restore(move |s: &mut S| acc(s).presses += 1).build(t))),
        spec(t, "A marker only", rune::col().w(560.0).child(Checkpoint::<S>::new("checkpoint-mark", "Session resumed · yesterday").build(t))),
    ])
}
