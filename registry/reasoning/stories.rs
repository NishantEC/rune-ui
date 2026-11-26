use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::reasoning::Reasoning;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("reasoning", "Reasoning", "agent").story("Thinking, then a fold", all::<S>).story("Nothing to show means nothing shown", quiet::<S>)
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

/// Under a second it says "Thought"; with no words when done it draws nothing at all (the column keeps no gap for it).
fn quiet<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let between = |e: El| rune::col().key("reasoning-between").w(480.0).gap(t.space.at(1.0)).child(rune::text("Above").font(t.ty.sm).color(t.ink_faint())).child(e).child(rune::text("Below").font(t.ty.sm).color(t.ink_faint()));
    specs(t, vec![
        spec(t, "Under a second: \"Thought\", no figure", rune::col().w(480.0).child(Reasoning::<S>::new("reasoning-quick", false, 0.4, false).text(THOUGHTS).build(t))),
        spec(t, "Done with no words: nothing between the two lines", between(Reasoning::<S>::new("reasoning-empty", false, 3.0, false).build(t))),
    ])
}
