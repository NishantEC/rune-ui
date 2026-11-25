use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::sources::{Source, Sources};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("sources", "Sources", "agent").story("Used 4 sources", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let list = vec![Source::new("Retry with exponential backoff", "https://docs.example.com/guides/retries"), Source::new("Release notes 4.2", "https://www.example.com/releases/4-2"), Source::new("ENG-123: imports fail on large pages", "https://tracker.example.com/ENG-123"), Source::new("A placeholder blog post about rate limits", "https://blog.example.org/rate-limits")];
    specs(t, vec![
        spec(t, "Folded; press to open", rune::col().w(480.0).child(Sources::<S>::new("sources-demo", &list, g.flag("sources-demo")).on_toggle(move |s: &mut S| acc(s).toggle("sources-demo")).on_open(move |s: &mut S, url: &str| acc(s).choose("sources-url", url)).build(t))),
        spec(t, "Open", rune::col().w(480.0).child(Sources::<S>::new("sources-open", &list, true).on_open(|_: &mut S, _: &str| {}).build(t))),
    ])
}
