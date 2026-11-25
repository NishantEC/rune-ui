use rune::El;

use super::super::agent::Status;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::task::{Run, Task};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("task", "Task", "agent").story("Run cards", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, acc) = (c.t, c.access);
    let done = Run { title: "ENG-123: retry failed pages in the importer".to_owned(), status: Status::Done, files: 4, added: 120, removed: 34, passed: 212, failed: 0, cost: 0.42, secs: 252.0 };
    let failed = Run { title: "Upgrade the parser to 0.4".to_owned(), status: Status::Error, files: 9, added: 310, removed: 188, passed: 198, failed: 3, cost: 1.16, secs: 611.0 };
    let running = Run { title: "Write the migration guide".to_owned(), status: Status::Running, files: 1, added: 40, removed: 0, passed: 0, failed: 0, cost: 0.05, secs: 48.0 };
    let w = |e: El| rune::col().w(520.0).child(e);
    specs(t, vec![
        spec(t, "Done", w(Task::<S>::new("task-done", &done).action("Review changes", move |s: &mut S| acc(s).presses += 1).build(t))),
        spec(t, "Failed checks", w(Task::<S>::new("task-failed", &failed).action("Open the failures", |_: &mut S| {}).build(t))),
        spec(t, "Running", w(Task::<S>::new("task-running", &running).build(t))),
    ])
}
