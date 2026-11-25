use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::terminal::Terminal;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("terminal", "Terminal", "agent").story("Running, passed, failed", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let w = |e: El| rune::col().w(560.0).child(e);
    specs(t, vec![
        spec(t, "Running", w(Terminal::new("term-running", "cargo build --release", "   Compiling parser v0.3.1\n   Compiling importer v0.9.0").running(true).secs(8.0).build(t))),
        spec(t, "Exit 0", w(Terminal::new("term-ok", "cargo test", "running 212 tests\ntest result: ok. 212 passed; 0 failed").exit(Some(0)).secs(12.4).build(t))),
        spec(t, "Exit 101", w(Terminal::new("term-fail", "cargo test retry", "test retry::gives_up ... FAILED\nthread 'retry::gives_up' panicked at src/retry.rs:31:5").exit(Some(101)).secs(3.1).build(t))),
    ])
}
