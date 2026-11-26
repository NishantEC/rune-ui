use rune::El;

use super::super::agent::Status;
use super::super::base::State;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::tool::{Tool, ToolCall};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("tool", "Tool", "agent")
        .story("One call in each state; press a line to fold it", all::<S>)
        .story("A step line that reads like a sentence", sentence::<S>)
}

fn call(name: &str, summary: &str, status: Status, secs: Option<f64>) -> ToolCall {
    ToolCall { name: name.to_owned(), summary: summary.to_owned(), status, secs, input: format!("{{ \"path\": \"{summary}\" }}"), output: Some("fn main() {\n    println!(\"placeholder\");\n}".to_owned()), error: None }
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let done = call("read_file", "src/main.rs", Status::Done, Some(1.4));
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

const LONG: &str = "cargo test --workspace --all-features -- --test-threads 1 importer::retry::gives_up_after_three_attempts";

fn sentence<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let quick = call("read_file", "src/main.rs", Status::Done, Some(0.3));
    let mut command = ToolCall { name: "run".to_owned(), summary: String::new(), status: Status::Error, secs: Some(0.8), input: LONG.to_owned(), output: None, error: Some("test importer::retry::gives_up_after_three_attempts ... FAILED".to_owned()) };
    let denied = ToolCall { name: "run".to_owned(), summary: String::new(), status: Status::Denied, secs: None, input: "git push --force origin main".to_owned(), output: None, error: None };
    let col = rune::col().w_full().gap(t.space.at(1.0))
        .child(Tool::<S>::new("tool-quick", &quick, false).label("Read src/main.rs").build(t))
        .child(Tool::<S>::new("tool-command", &command, g.flag("tool-command")).label(LONG).note("exit 1").on_toggle(move |s: &mut S| acc(s).toggle("tool-command")).build(t))
        .child(Tool::<S>::new("tool-denied", &denied, false).label("git push --force origin main").build(t));
    command.status = Status::Done;
    command.error = None;
    command.output = Some("test result: ok. 41 passed; 0 failed".to_owned());
    let open = Tool::<S>::new("tool-said", &command, true).label(LONG).note("41 passed").text("Ran the importer's tests one at a time, so the retry test cannot race the others.").on_toggle(|_: &mut S| {}).build(t);
    specs(t, vec![
        spec_wide(t, "Under a second: no time. A long command cut in the middle, with a note. Denied: neutral, not red", 480.0, col),
        spec_wide(t, "Open: the whole line, free words, the input and the output", 480.0, open),
    ])
}
