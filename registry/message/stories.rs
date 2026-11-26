use rune::El;

use super::super::actions::Actions;
use super::super::attachment::{Attachment, Kind};
use super::super::avatar::Avatar;
use super::super::base::Size;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::message::{Message, Reveal, words};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("message", "Message", "agent")
        .story("A person's bubble and the agent's words on the page", all::<S>)
        .story("Actions on hover or focus: hidden, they take no height", reveal::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let user = Message::user("message-user", "Asha").child(words(t, "Here is the log from last night. What failed?"))
        .attachment(Attachment::<S>::new("message-file", "nightly-2026-11-24.log").detail("84 KB").build(t))
        .attachment(Attachment::<S>::new("message-image", "screenshot.png").kind(Kind::Image).build(t))
        .avatar(Avatar::new("message-asha", "Asha").size(Size::Sm).build(t)).build(t);
    let bot = Message::assistant("message-bot", "Assistant").child(words(t, "The migration step timed out after 10 minutes. It waits on a lock another job holds; running them one after the other fixes it."))
        .actions(Actions::<S>::new("message-actions").copy("The migration step timed out.", g.flag("message-copied"), move |s: &mut S| acc(s).toggle("message-copied")).retry(|_: &mut S| {}).build(t)).build(t);
    specs(t, vec![spec_wide(t, "User, with attachments and an avatar; assistant, plain on the page", 640.0, rune::col().w_full().gap(t.space.at(6.0)).child(user).child(bot))])
}

/// A person's message and the agent's answer whose actions show only while hovered or focused: live (point at them, or Tab to them),
/// then each pinned hidden and shown.
fn reveal<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let user_bar = |key: &str| Actions::<S>::new(key).copy("Why is the build slow?", false, |_: &mut S| {}).action("edit", "Edit", &super::super::icons::WRENCH, |_: &mut S| {}).build(t);
    let bot_bar = |key: &str| Actions::<S>::new(key).copy("Linking takes most of the time.", false, |_: &mut S| {}).retry(|_: &mut S| {}).build(t);
    let turn = |id: &str, user: Reveal, bot: Reveal| {
        let (uk, bk) = (format!("message-{id}-user"), format!("message-{id}-bot"));
        let (us, bs) = (format!("{uk}-reveal"), format!("{bk}-reveal"));
        let u = Message::user(uk.clone(), "Asha").child(words(t, "Why is the build slow?")).actions(user_bar(&format!("{uk}-actions")));
        let b = Message::assistant(bk.clone(), "Assistant").child(words(t, "Linking takes most of the time; a faster linker halves it.")).actions(bot_bar(&format!("{bk}-actions")));
        let (u, b) = if id == "live" {
            let (su, sb) = (us.clone(), bs.clone());
            (u.reveal_actions(g.slot::<Reveal>(&us), with(move |s: &mut S| acc(s).slot_mut::<Reveal>(&su))), b.reveal_actions(g.slot::<Reveal>(&bs), with(move |s: &mut S| acc(s).slot_mut::<Reveal>(&sb))))
        } else {
            (u.reveal_actions(user, with(move |s: &mut S| acc(s).slot_mut::<Reveal>("message-pinned"))), b.reveal_actions(bot, with(move |s: &mut S| acc(s).slot_mut::<Reveal>("message-pinned"))))
        };
        rune::col().key(format!("message-{id}")).w_full().gap(t.space.at(5.0)).child(u.build(t)).child(b.build(t))
    };
    let shown = Reveal { hovered: true, focused: false };
    specs(t, vec![
        spec_wide(t, "Live: point at a message, or Tab to its actions", 560.0, turn("live", Reveal::default(), Reveal::default())),
        spec_wide(t, "Hidden: no row, no gap", 560.0, turn("hidden", Reveal::default(), Reveal::default())),
        spec_wide(t, "Shown (hovered or focused)", 560.0, turn("shown", shown, shown)),
    ])
}

/// Names the closure's lifetimes for `reveal_actions` (a closure returning a borrow of its argument needs the hint).
fn with<S, F: for<'a> Fn(&'a mut S) -> &'a mut Reveal + Clone + 'static>(f: F) -> F {
    f
}
