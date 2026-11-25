use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::reasoning::Reasoning;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("reasoning", "Reasoning", "agent").story("Thinking, then a fold", all::<S>)
}

const THOUGHTS: &str = "The failing test only fails on the second run, so something survives between runs. The fixture folder is not cleaned; check the teardown first.";

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    specs(t, vec![
        spec(t, "Thinking (shimmer)", rune::col().w(480.0).child(Reasoning::<S>::new("reasoning-live", true, 4.0, false).build(t))),
        spec(t, "Done; press to fold open", rune::col().w(480.0).child(Reasoning::<S>::new("reasoning-done", false, 12.0, g.flag("reasoning-done")).text(THOUGHTS).on_toggle(move |s: &mut S| acc(s).toggle("reasoning-done")).build(t))),
        spec(t, "Open", rune::col().w(480.0).child(Reasoning::<S>::new("reasoning-open", false, 75.0, true).text(THOUGHTS).build(t))),
    ])
}
