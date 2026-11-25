use rune::El;

use super::super::agent::Status;
use super::super::base::State;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::tool::{Tool, ToolCall};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("tool", "Tool", "agent").story("One call in each state; press a line to fold it", all::<S>)
}

fn call(name: &str, summary: &str, status: Status, secs: Option<f64>) -> ToolCall {
    ToolCall { name: name.to_owned(), summary: summary.to_owned(), status, secs, input: format!("{{ \"path\": \"{summary}\" }}"), output: Some("fn main() {\n    println!(\"placeholder\");\n}".to_owned()), error: None }
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let done = call("read_file", "src/main.rs", Status::Done, Some(0.4));
    let running = call("search", "\"retry\" in src/", Status::Running, Some(2.0));
    let pending = call("write_file", "src/retry.rs", Status::Pending, None);
    let mut failed = call("run", "cargo test", Status::Error, Some(12.4));
    failed.error = Some("error[E0425]: cannot find value `backoff` in this scope\n --> src/retry.rs:14:19".to_owned());
    failed.input = "cargo test --quiet".to_owned();
    let line = |key: &'static str, call: &ToolCall| Tool::<S>::new(key, call, g.flag(key)).on_toggle(move |s: &mut S| acc(s).toggle(key)).build(t);
    let col = rune::col().w_full().gap(t.space.at(1.0))
        .child(line("tool-done", &done))
        .child(line("tool-running", &running))
        .child(line("tool-pending", &pending))
        .child(line("tool-error", &failed))
        .child(Tool::<S>::new("tool-hover", &done, false).state(State::Hover).build(t));
    let open = Tool::<S>::new("tool-open", &failed, true).on_toggle(|_: &mut S| {}).build(t);
    specs(t, vec![spec_wide(t, "Done, running, pending, failed, hovered", 560.0, col), spec_wide(t, "Open: input and error", 560.0, open)])
}
