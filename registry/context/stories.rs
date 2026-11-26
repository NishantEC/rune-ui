use rune::El;

use super::super::context::{Context, Face, Usage};
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("context", "Context", "agent").story("Press for the breakdown", all::<S>).story("The bar, and the breakdown's own figures", bars::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let usage = Usage { context: 86_000, limit: 200_000, input: 48_200, output: 6_100, cache: 31_000, cost: Some(0.42) };
    let full = Usage { context: 166_000, limit: 200_000, input: 150_000, output: 12_000, cache: 4_000, cost: Some(2.31) };
    let edge = Usage { context: 192_000, limit: 200_000, input: 190_000, output: 2_000, cache: 0, cost: Some(3.05) };
    let live = Context::<S>::new("context-demo", &usage, g.is_open("context-demo")).note("Prices are estimates.").on_open(move |s: &mut S, o| {
        let g = acc(s);
        g.open = o.then(|| "context-demo".to_owned());
    }).build(t, c.layer);
    specs(t, vec![
        spec(t, "43% used: the ring (press, Return or Space)", live),
        spec(t, "From 80%: the warning ink", Context::<S>::new("context-full", &full, false).build(t, c.layer)),
        spec(t, "From 95%: the danger ink", Context::<S>::new("context-edge", &edge, false).build(t, c.layer)),
        spec(t, "The ring alone", Context::<S>::new("context-bare", &usage, false).percent(false).build(t, c.layer)),
    ])
}

/// The bar face; and a breakdown with no cost (no cost row), opened with a press.
fn bars<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let usage = Usage { context: 86_000, limit: 200_000, input: 48_200, output: 6_100, cache: 31_000, cost: Some(0.42) };
    let free = Usage { context: 21_000, limit: 128_000, input: 30_500, output: 4_200, cache: 12_800, cost: None };
    let local = Context::<S>::new("context-free", &free, g.is_open("context-free")).face(Face::Bar).note("Runs on this computer: no cost.").on_open(move |s: &mut S, o| {
        let g = acc(s);
        g.open = o.then(|| "context-free".to_owned());
    }).build(t, c.layer);
    specs(t, vec![
        spec(t, "The bar", Context::<S>::new("context-bar", &usage, false).face(Face::Bar).build(t, c.layer)),
        spec(t, "No cost: the breakdown draws no cost row", local),
    ])
}
