use rune::El;

use super::super::agent::Status;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::task::{Run, Task};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("task", "Task", "agent").story("Run cards", all::<S>).story("Only the facts given", sparse::<S>).story("Stopped, and secondary actions beside the primary", stopped::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, acc) = (c.t, c.access);
    let done = Run::new("ENG-123: retry failed pages in the importer", Status::Done).changes(4, 120, 34).checks(212, 0).cost(0.42).secs(252.0);
    let failed = Run::new("Upgrade the parser to 0.4", Status::Error).changes(9, 310, 188).checks(198, 3).cost(1.16).secs(611.0);
    let running = Run::new("Write the migration guide", Status::Running).changes(1, 40, 0).cost(0.05).secs(48.0);
    specs(t, vec![
        spec_wide(t, "Done", 520.0, Task::<S>::new("task-done", &done).action("Review changes", move |s: &mut S| acc(s).presses += 1).build(t)),
        spec_wide(t, "Failed checks", 520.0, Task::<S>::new("task-failed", &failed).action("Open the failures", |_: &mut S| {}).build(t)),
        spec_wide(t, "Running: no checks yet, so none shown", 520.0, Task::<S>::new("task-running", &running).build(t)),
    ])
}

fn sparse<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let tokens = Run::new("Answer a question about the build", Status::Done).tokens(18_400).secs(14.0).branch("asha/build-notes");
    let noted = Run::new("Draft the release notes", Status::Waiting).branch("release/0.9").note("Waiting on a review from Lee before it opens a pull request.");
    let bare = Run::new("Tidy the fixtures folder", Status::Pending);
    specs(t, vec![
        spec_wide(t, "Tokens in the cost's place, time and branch", 520.0, Task::<S>::new("task-tokens", &tokens).build(t)),
        spec_wide(t, "A branch and a note line", 520.0, Task::<S>::new("task-noted", &noted).build(t)),
        spec_wide(t, "Nothing but the title and status", 520.0, Task::<S>::new("task-bare", &bare).build(t)),
    ])
}

/// A run the person stopped (neutral, a square) with a primary action and two secondary ones.
fn stopped<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let run = Run::new("Reorganise the docs folder", Status::Stopped).changes(2, 14, 3).secs(80.0).note("Stopped before it finished; nothing was lost.");
    let card = Task::<S>::new("task-stopped", &run).action("Resume", |_: &mut S| {}).secondary("Undo", |_: &mut S| {}).secondary("Open the branch", |_: &mut S| {}).build(t);
    specs(t, vec![spec_wide(t, "Status::Stopped; Resume in ink, Undo and Open the branch raised", 520.0, card)])
}
