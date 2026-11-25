use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::queue::Queue;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("queue", "Queue", "agent").story("Waiting to be sent: Alt+Up and Alt+Down move, Delete removes", all::<S>)
}

/// The queued words, kept in the gallery as one choice joined by newlines.
pub fn queued(g: &super::super::gallery::GalleryState) -> Vec<String> {
    g.choice("queue", "Also run the linter\nThen open a pull request\nTag Lee for review").lines().map(str::to_owned).collect()
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let items = queued(g);
    let q = Queue::<S>::new("queue-demo", &items)
        .on_move(move |s: &mut S, from, to| {
            let g = acc(s);
            let mut v = queued(g);
            v.swap(from, to);
            g.choose("queue", &v.join("\n"));
        })
        .on_remove(move |s: &mut S, i| {
            let g = acc(s);
            let mut v = queued(g);
            if i < v.len() {
                v.remove(i);
            }
            g.choose("queue", &v.join("\n"));
        })
        .build(t);
    specs(t, vec![spec(t, "Three queued", rune::col().w(480.0).child(q))])
}
