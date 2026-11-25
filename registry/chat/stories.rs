use rune::El;

use super::super::chat::{ChatState, screen};
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("chat", "Chat", "agent").lead().story("A whole agent screen, made of the set", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let fresh = ChatState::default();
    let state = g.slots.get("chat").and_then(|b| b.downcast_ref::<ChatState>()).unwrap_or(&fresh);
    let page = screen(t, state, move |s: &mut S| acc(s).slot_mut::<ChatState>("chat"), c.layer);
    specs(t, vec![spec_wide(t, "Conversation, reasoning, tools, steps, an answer, a permission request and the composer", 820.0, rune::col().w_full().h(760.0).radius(t.radius.xl).clip_rounded(t.radius.xl).fill(t.c.page.bg).child(page))])
}
