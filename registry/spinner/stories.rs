use rune_ui_style::{Size, Tone};

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::spinner::Spinner;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("spinner", "Spinner", "display").story("Sizes and tones", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    let mut items: Vec<rune::El> = Size::ALL.iter().map(|z| spec(t, z.word(), Spinner::new("Loading").size(*z).build(t))).collect();
    items.push(spec(t, "Info", Spinner::new("Working").tone(Tone::Info).build(t)));
    specs(t, items)
}
