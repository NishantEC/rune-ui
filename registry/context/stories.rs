use rune::El;

use super::super::context::{Context, Usage};
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("context", "Context", "agent").story("Press for the breakdown", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let usage = Usage { input: 48_200, output: 6_100, cache: 31_000, limit: 200_000, cost: 0.42 };
    let full = Usage { input: 150_000, output: 12_000, cache: 4_000, limit: 200_000, cost: 2.31 };
    let edge = Usage { input: 190_000, output: 2_000, cache: 0, limit: 200_000, cost: 3.05 };
    let live = Context::<S>::new("context-demo", &usage, g.is_open("context-demo")).on_open(move |s: &mut S, o| {
        let g = acc(s);
        g.open = o.then(|| "context-demo".to_owned());
    }).build(t, c.layer);
    specs(t, vec![
        spec(t, "43% used", live),
        spec(t, "Past 80%: the warning ink", Context::<S>::new("context-full", &full, false).build(t, c.layer)),
        spec(t, "Past 95%: the danger ink", Context::<S>::new("context-edge", &edge, false).build(t, c.layer)),
    ])
}
