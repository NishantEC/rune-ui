use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::suggestion::Suggestions;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("suggestion", "Suggestion", "agent").story("Suggested prompts", all::<S>)
}

const WORDS: [&str; 4] = ["Summarise this file", "Write tests for the parser", "Why is the build slow?", "Explain the last commit"];

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, acc) = (c.t, c.access);
    specs(t, vec![spec(t, "A press sends the words", rune::col().w(520.0).child(Suggestions::<S>::new("suggestion-demo", &WORDS).on_pick(move |s: &mut S, w: &str| acc(s).choose("suggestion", w)).build(t)))])
}
