use rune::El;

use super::super::agent::Status;
use super::super::chain_of_thought::{ChainOfThought, Step};
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("chain_of_thought", "ChainOfThought", "agent").story("Worked for…, open and working", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let done = vec![Step::new("Read the failing test", Status::Done), Step::new("Found the fixture left behind", Status::Done).detail("tests/fixtures/tmp is never removed"), Step::new("Added a teardown", Status::Done), Step::new("Ran the suite twice", Status::Done).detail("212 passed, twice")];
    let working = vec![Step::new("Read the failing test", Status::Done), Step::new("Looking for what survives between runs", Status::Running), Step::new("Fix it", Status::Pending), Step::new("Ask before deleting fixtures", Status::Waiting), Step::new("Update the docs", Status::Skipped)];
    specs(t, vec![
        spec_wide(t, "Done; press to fold", 520.0, ChainOfThought::<S>::new("cot-done", &done, 134.0, g.flag("cot-done")).on_toggle(move |s: &mut S| acc(s).toggle("cot-done")).build(t)),
        spec_wide(t, "Working, open: every step state", 520.0, ChainOfThought::<S>::new("cot-working", &working, 47.0, true).running(true).build(t)),
    ])
}
