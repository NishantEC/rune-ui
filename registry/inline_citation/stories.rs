use rune::{El, row, text};

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::hover_card::HoverCardState;
use super::super::inline_citation::{Citation, InlineCitation};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("inline_citation", "InlineCitation", "agent").story("Rest the pointer on the chip", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let sources = vec![
        Citation { title: "Retry with exponential backoff".to_owned(), url: "https://docs.example.com/guides/retries".to_owned(), quote: "Wait longer after each failure, and give up after a few tries.".to_owned() },
        Citation { title: "Release notes 4.2".to_owned(), url: "https://www.example.com/releases/4-2".to_owned(), quote: "The importer now skips a page that keeps failing.".to_owned() },
    ];
    let state = HoverCardState { trigger: g.flag("cite-trigger"), card: g.flag("cite-card"), pinned: g.flag("cite-pinned") || g.is_open("cite-demo") };
    let chip = InlineCitation::<S>::new("cite-demo", &sources, state).on_change(move |s: &mut S, h: HoverCardState| {
        let g = acc(s);
        g.flags.insert("cite-trigger".to_owned(), h.trigger);
        g.flags.insert("cite-card".to_owned(), h.card);
        g.flags.insert("cite-pinned".to_owned(), h.pinned);
        g.close("cite-demo");
    }).build(t, c.layer);
    let line = row().gap(t.space.at(1.5)).center_y().flex_wrap().child(text("Retries now wait longer after each failure").font(t.ty.md).color(t.ink())).child(chip);
    specs(t, vec![spec(t, "A citation inside a sentence; its card floats over the page", line)])
}
