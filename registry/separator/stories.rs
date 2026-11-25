use rune::{row, text};

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::separator::Separator;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("separator", "Separator", "structure").story("Across, down, with words", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    let w = |s: &str| text(s.to_owned()).font(t.ty.base).color(t.ink());
    specs(t, vec![
        spec(t, "Horizontal", rune::col().w(240.0).gap(t.space.at(3.0)).child(w("Account")).child(Separator::horizontal().build(t)).child(w("Billing"))),
        spec(t, "Vertical", row().h(20.0).gap(t.space.at(3.0)).center_y().child(w("Docs")).child(Separator::vertical().length(16.0).build(t)).child(w("Source")).child(Separator::vertical().length(16.0).build(t)).child(w("Help"))),
        spec(t, "With words", rune::col().w(240.0).child(Separator::horizontal().label("or").build(t))),
    ])
}
