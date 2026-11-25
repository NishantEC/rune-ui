use rune::text;
use rune_ui_style::State as Look;

use super::super::accordion::Accordion;
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("accordion", "Accordion", "structure").story("Sections", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let words = |w: &str| text(w.to_owned()).font(t.ty.base).color(t.ink_muted()).wrap().min_w(0.0);
    let live = Accordion::<S>::new("accordion")
        .item("What does the placeholder plan include?", g.flag("accordion-0"), words("Everything in the sample, for one person."))
        .item("Can I change plans later?", g.flag("accordion-1"), words("Yes, at any time."))
        .item("Where is my data stored?", g.flag("accordion-2"), words("On this computer, in a placeholder folder."))
        .on_toggle(move |s: &mut S, i| acc(s).toggle(&format!("accordion-{i}")))
        .build(t);
    let pinned = Accordion::<S>::new("accordion-pinned").item("Hover on a closed section", false, words("")).item("Focus", false, words("")).state(0, Look::Hover).build(t);
    specs(t, vec![spec(t, "Live", rune::col().w(420.0).child(live)), spec(t, "Hover", rune::col().w(320.0).child(pinned))])
}
