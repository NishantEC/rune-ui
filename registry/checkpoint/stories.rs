use rune::El;

use super::super::checkpoint::Checkpoint;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("checkpoint", "Checkpoint", "agent").story("A divider with Restore", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, acc) = (c.t, c.access);
    specs(t, vec![
        spec_wide(t, "Restorable", 560.0, Checkpoint::<S>::new("checkpoint-demo", "Checkpoint · 14:02").on_restore(move |s: &mut S| acc(s).presses += 1).build(t)),
        spec_wide(t, "A marker only", 560.0, Checkpoint::<S>::new("checkpoint-mark", "Session resumed · yesterday").build(t)),
    ])
}
