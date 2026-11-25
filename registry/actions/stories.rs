use rune::El;

use super::super::actions::Actions;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("actions", "Actions", "agent").story("Under an answer", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let rating = match g.choice("actions-rating", "").as_str() {
        "good" => Some(true),
        "bad" => Some(false),
        _ => None,
    };
    let live = Actions::<S>::new("actions-demo").copy("A placeholder answer.", g.flag("actions-copied"), move |s: &mut S| acc(s).toggle("actions-copied")).retry(move |s: &mut S| acc(s).presses += 1)
        .feedback(rating, move |s: &mut S, r| acc(s).choose("actions-rating", match r { Some(true) => "good", Some(false) => "bad", None => "" })).build(t);
    let rated = Actions::<S>::new("actions-rated").copy("", true, |_: &mut S| {}).feedback(Some(true), |_: &mut S, _| {}).action("share", "Share", &icons::LINK, |_: &mut S| {}).build(t);
    specs(t, vec![spec(t, "Copy, Retry, Good, Bad", live), spec(t, "Copied, rated good, one more action", rated)])
}
