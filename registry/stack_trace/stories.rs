use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::stack_trace::{Frame, StackTrace};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("stack_trace", "StackTrace", "agent").story("App frames, library frames folded, press a frame", all::<S>)
}

/// A placeholder trace: two of the app's frames, a run of a library's, one more of the app's, two of the runtime's.
pub fn frames() -> Vec<Frame> {
    vec![
        Frame::app("invoice_total", "src/billing/invoice.rs", 42, 17),
        Frame::app("render_row", "src/ui/invoices.rs", 118, 9),
        Frame::library("Iterator::fold", "library/core/src/iter/traits/iterator.rs", 2601, 21),
        Frame::library("Map::fold", "library/core/src/iter/adapters/map.rs", 128, 19),
        Frame::library("Vec::extend_trusted", "library/alloc/src/vec/mod.rs", 3108, 17),
        Frame::app("view", "src/main.rs", 77, 5),
        Frame::library("Runtime::frame", "rune/src/runtime/api.rs", 49, 9),
        Frame::library("main", "library/std/src/rt.rs", 175, 20),
    ]
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let f = frames();
    let live = StackTrace::<S>::new("trace-demo", "panicked: called `Option::unwrap()` on a `None` value", &f)
        .message("while rendering the invoice list for ENG-123")
        .show_library(g.flag("trace-libs")).on_toggle_library(move |s: &mut S| acc(s).toggle("trace-libs"))
        .on_frame(move |s: &mut S, file: &str, line, col| acc(s).choose("trace-opened", &format!("{file}:{line}:{col}")))
        .copied(g.flag("trace-copied")).on_copy(move |s: &mut S| { acc(s).flags.insert("trace-copied".to_owned(), true); })
        .build(t);
    let open = StackTrace::<S>::new("trace-open", "TypeError: total is undefined", &f[..5]).show_library(true).on_toggle_library(|_: &mut S| {}).on_frame(|_: &mut S, _: &str, _, _| {}).state(0, rune_ui_style::State::Hover).build(t);
    specs(t, vec![
        spec_wide(t, "Library frames folded; press a frame, a folded line, or Copy", 620.0, live),
        spec_wide(t, "Library frames shown, faint; the first frame hovered", 620.0, open),
    ])
}
