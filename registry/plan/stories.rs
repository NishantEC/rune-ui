use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::plan::{Plan, Todo};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("plan", "Plan", "agent").story("Every step state", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let steps: Vec<(String, Todo)> = [("Reproduce the failure on a clean checkout", Todo::Done), ("Find what survives between runs", Todo::Done), ("Add a teardown for the fixtures folder", Todo::Doing), ("Run the suite twice", Todo::Todo), ("Update the contributing guide", Todo::Skipped)].into_iter().map(|(w, s)| (w.to_owned(), s)).collect();
    specs(t, vec![spec(t, "Done, doing, to do, skipped", rune::col().w(480.0).child(Plan::new("plan-demo", "Fix the flaky test", &steps).build(t)))])
}
