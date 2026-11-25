use rune::{row, text};

use super::super::base::Shape;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::scroll_area::ScrollArea;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("scroll_area", "ScrollArea", "structure").story("A long list", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    let mut area = ScrollArea::new("scroll-area").height(180.0).width(260.0);
    for i in 1..=20 {
        area = area.child(row().w_full().h(32.0).px(t.space.at(3.0)).center_y().border_sides([0.0, 0.0, 1.0, 0.0], t.c.line).child(text(format!("Placeholder row {i}")).font(t.ty.base).color(t.ink())));
    }
    let slab = rune_ui_style::paint(rune::col().radius(t.radius.lg).round_as(t), &t.mat.slab.base).child(area.build(t));
    specs(t, vec![spec(t, "Twenty rows in 180 points, edges fading", slab)])
}
