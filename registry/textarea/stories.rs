use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::textarea::Textarea;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("textarea", "Textarea", "form").story("States", all::<S>).story("A limit and a counter", limits::<S>).story("Mono", mono::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let a = |k: &str| Textarea::<S>::new(format!("textarea-{k}"), g.edit("notes"), move |s: &mut S| acc(s).edit_mut("notes")).name("Notes").width(260.0);
    specs(t, vec![
        spec(t, "Rest", a("rest").build(t)),
        spec(t, "Focus", a("focus").state(Look::FocusVisible).build(t)),
        spec(t, "Placeholder", Textarea::<S>::new("textarea-empty", g.edit("empty"), move |s: &mut S| acc(s).edit_mut("empty")).placeholder("Write a note").rows(2, 6).width(260.0).build(t)),
        spec(t, "Invalid", a("invalid").invalid(true).build(t)),
        spec(t, "Disabled", a("disabled").disabled(true).build(t)),
    ])
}

/// A live counter (type in it), then pinned: near the limit, over it, and a firm limit that takes the excess out again.
fn limits<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let pinned = |k: &'static str, words: &str, hard: bool| {
        let edit = rune::TextEdit::new(words);
        Textarea::<S>::new(format!("textarea-{k}"), &edit, move |s: &mut S| acc(s).edit_mut(k)).name("Summary").limit(40).hard_limit(hard).rows(2, 4).width(260.0).build(t)
    };
    specs(t, vec![
        spec(t, "Live: 40 characters; the counter follows the typing", Textarea::<S>::new("textarea-limit-live", g.edit("limit-live"), move |s: &mut S| acc(s).edit_mut("limit-live")).name("Summary").limit(40).rows(2, 4).width(260.0).build(t)),
        spec(t, "Near the limit: the warning ink", pinned("limit-near", "Fix the login test and its retries now", false)),
        spec(t, "Over a soft limit: danger ink, invalid", pinned("limit-over", "Fix the login test, its retries and the slow importer", false)),
        spec(t, "Firm: typing past 40 takes the excess out again", Textarea::<S>::new("textarea-limit-hard", g.edit("limit-hard"), move |s: &mut S| acc(s).edit_mut("limit-hard")).name("Summary").limit(40).hard_limit(true).rows(2, 4).width(260.0).build(t)),
    ])
}

/// Code and logs: the text in a monospaced face.
fn mono<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    specs(t, vec![
        spec(t, "Mono, to type in", Textarea::<S>::new("textarea-mono", g.edit("mono"), move |s: &mut S| acc(s).edit_mut("mono")).name("Command").mono(true).rows(3, 6).width(300.0).build(t)),
        spec(t, "Same words in the page's face", Textarea::<S>::new("textarea-mono-off", g.edit("mono"), move |s: &mut S| acc(s).edit_mut("mono")).name("Command").rows(3, 6).width(300.0).build(t)),
    ])
}
