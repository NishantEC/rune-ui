use rune::El;

use super::super::actions::Actions;
use super::super::attachment::{Attachment, Kind};
use super::super::avatar::Avatar;
use super::super::base::Size;
use super::super::gallery::{Ctx, StoryItem, spec_wide, specs};
use super::super::message::{Message, words};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("message", "Message", "agent").story("A person's bubble and the agent's words on the page", all::<S>)
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
