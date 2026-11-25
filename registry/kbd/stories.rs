use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::kbd::Kbd;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("kbd", "Kbd", "display").story("Keys", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    specs(t, vec![spec(t, "One key", Kbd::new("kbd-one", "Esc").build(t)), spec(t, "A chord", Kbd::new("kbd-chord", "⌘ K").build(t)), spec(t, "Three keys", Kbd::new("kbd-three", "Ctrl Shift P").build(t))])
}
