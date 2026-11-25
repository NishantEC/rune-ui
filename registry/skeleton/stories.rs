use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::skeleton::Skeleton;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("skeleton", "Skeleton", "display").story("Shapes", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    specs(t, vec![spec(t, "Lines", Skeleton::lines(t, "skeleton-lines", 3, 240.0)), spec(t, "Row", Skeleton::row(t, "skeleton-row", 260.0)), spec(t, "Block", Skeleton::block(t, "skeleton-block", 160.0, 90.0))])
}
