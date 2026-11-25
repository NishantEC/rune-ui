use rune::{El, col, text};
use rune_ui_style::{Recipe, Theme, paint, tabular, weight};

use super::super::carousel::Carousel;
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("carousel", "Carousel", "structure").story("Slides", all::<S>)
}

fn card(t: &Theme, i: usize, w: f32, h: f32) -> El {
    paint(col().size(w, h).center().gap(t.space.at(1.0)), &Recipe::new().fill(t.c.surface.bg)).child(text(format!("{}", i + 1)).font(weight(tabular(t.ty.xxl), t.ty.semibold)).color(t.ink())).child(text("A placeholder slide").font(t.ty.sm).color(t.ink_muted()))
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let at = g.index("carousel", 0);
    let mut live = Carousel::<S>::new("carousel-demo", "Placeholder slides", at).size(360.0, 200.0).on_index(move |s: &mut S, i| acc(s).choose("carousel", &i.to_string()));
    for i in 0..5 {
        live = live.slide(card(t, i, 360.0, 200.0));
    }
    let lat = g.index("carousel-loop", 2);
    let mut looped = Carousel::<S>::new("carousel-loop", "Looping slides", lat).size(240.0, 140.0).looped(true).on_index(move |s: &mut S, i| acc(s).choose("carousel-loop", &i.to_string()));
    for i in 0..3 {
        looped = looped.slide(card(t, i, 240.0, 140.0));
    }
    specs(t, vec![spec(t, "Arrows, dots, or Tab to it and press Left and Right", live.build(t)), spec(t, "Looping", looped.build(t))])
}
