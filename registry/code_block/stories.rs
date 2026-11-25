use rune::El;

use super::super::code_block::CodeBlock;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("code_block", "CodeBlock", "agent").story("Coloured, numbered, plain", all::<S>)
}

const RUST: &str = "/// Waits longer after each failed try.\nfn backoff(attempt: u32) -> Duration {\n    let secs = attempt.pow(2);\n    Duration::from_secs(u64::from(secs)) // 1, 4, 9 seconds, then the importer gives up on this page and moves on\n}";

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    specs(t, vec![
        spec_wide(t, "Rust, line numbers, Copy; the long line scrolls", 520.0, CodeBlock::<S>::new("code-demo", RUST, Some("rust")).line_numbers(true).copied(g.flag("code-copied")).on_copy(move |s: &mut S| acc(s).toggle("code-copied")).build(t)),
        spec_wide(t, "Shell, no numbers", 520.0, CodeBlock::<S>::new("code-shell", "cargo test --workspace\ncargo clippy --all-targets", Some("sh")).build(t)),
        spec_wide(t, "Plain text", 520.0, CodeBlock::<S>::new("code-plain", "Finished in 41.3 s\nlink: 29.8 s", None).plain().build(t)),
    ])
}
