use rune::{El, col};

use super::super::conversation::Conversation;
use super::super::gallery::{Ctx, StoryItem, spec, spec_wide, specs};
use super::super::message::{Message, words};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("conversation", "Conversation", "agent").story("Pinned to the newest message; scroll up for Jump to latest", all::<S>)
}

const LINES: [&str; 4] = ["Can you check why the nightly job failed?", "It ran out of disk on the runner: the cache grew to 38 GB. Clearing it and capping it at 10 GB fixes the next run.", "Do that, and tell me what else is large.", "Done. The next largest is the test fixtures folder at 2.1 GB; nothing else is over 500 MB."];

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let n = g.value("conversation-n", 6.0) as usize;
    let msgs = (0..n).map(|i| {
        let w = LINES[i % LINES.len()];
        if i % 2 == 0 { Message::user(format!("conversation-m{i}"), "Asha").child(words(t, w)).build(t) } else { Message::assistant(format!("conversation-m{i}"), "Assistant").child(words(t, w)).build(t) }
    });
    let live = Conversation::<S>::new("conversation-demo", !g.flag("conversation-away")).height(320.0).children(msgs).on_at_bottom(move |s: &mut S, at| { acc(s).flags.insert("conversation-away".to_owned(), !at); }).build(t);
    let add = super::super::button::Button::<S>::new("conversation-add", "Add a message").variant(super::super::base::ButtonVariant::Secondary).on_press(move |s: &mut S| {
        let g = acc(s);
        let n = g.value("conversation-n", 6.0);
        g.set_value("conversation-n", n + 1.0);
    }).build(t);
    let empty = Conversation::<S>::new("conversation-empty", true).height(240.0).empty("Ask anything", "Answers, tool calls and files appear here.").build(t);
    specs(t, vec![
        spec_wide(t, "Messages: stays at the bottom as they arrive", 520.0, col().w_full().items_start().gap(t.space.at(3.0)).child(col().w_full().fill(t.c.page.bg).child(live)).child(add)),
        spec(t, "Empty", col().w(360.0).child(empty)),
    ])
}
