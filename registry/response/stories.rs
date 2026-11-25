use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::response::Response;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("response", "Response", "agent").story("Markdown, whole and streaming", all::<S>)
}

const WHOLE: &str = "## What changed\n\nThe importer now **retries** a failed page three times, waiting longer each time, and logs the page it gave up on.\n\n- Retries: 3, after 1 s, 4 s and 9 s\n- A page that still fails is skipped, not fatal\n- [x] Tests for both paths\n\n> Pages over 5 MB are still read in one piece.\n\n```rust\nfor attempt in 1..=3 {\n    match fetch(page) {\n        Ok(body) => return Ok(body),\n        Err(e) => sleep(backoff(attempt)),\n    }\n}\n```\n\n| Step | Before | After |\n| --- | --- | --- |\n| Import | 14 min | 9 min |\n| Failures | 12 | 0 |";

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    specs(t, vec![
        spec(t, "Whole: headings, lists, a quote, code and a table", rune::col().w(560.0).child(Response::new("response-whole", WHOLE).build(t))),
        spec(t, "Streaming: the newest words fade in, a caret trails", rune::col().w(560.0).child(Response::new("response-live", "The importer now **retries** a failed page three times, waiting longer").streaming(true).build(t))),
    ])
}
