use rune_ui_style::Size;

use super::super::avatar::{self, Avatar};
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("avatar", "Avatar", "display").story("Sizes and a group", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    let mut items: Vec<rune::El> = Size::ALL.iter().map(|z| spec(t, z.word(), Avatar::new(format!("avatar-{}", z.word().to_lowercase()), "Ada Lovelace").size(*z).build(t))).collect();
    items.push(spec(t, "Group", avatar::group(t, "avatar-group", &["Ada Lovelace", "Alan Turing", "Grace Hopper", "Edsger Dijkstra", "Barbara Liskov"], 3)));
    specs(t, items)
}
