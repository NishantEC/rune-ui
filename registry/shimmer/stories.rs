use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::shimmer::{Loader, Shimmer};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("shimmer", "Shimmer and Loader", "agent").story("Work in progress", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    specs(t, vec![
        spec(t, "Shimmer", Shimmer::new("shimmer-demo", "Working…").build(t)),
        spec(t, "Larger, in the first ink", Shimmer::new("shimmer-large", "Reading 14 files").font(t.ty.md).color(t.ink()).build(t)),
        spec(t, "Off (plain words)", Shimmer::new("shimmer-off", "Paused").on(false).build(t)),
        spec(t, "Loader", Loader::new("Waiting for the model").build(t)),
    ])
}
