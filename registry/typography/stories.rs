use rune::{El, col, row};

use super::super::gallery::{Ctx, StoryItem, labelled};
use super::super::typography as ty;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("typography", "Typography", "display").story("Helpers", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    col().w_full().gap(t.space.at(5.0))
        .child(labelled(t, "h1–h4", col().gap(t.space.at(2.0)).child(ty::h1(t, "The placeholder handbook").key("type-h1")).child(ty::h2(t, "Working together")).child(ty::h3(t, "Reviews")).child(ty::h4(t, "Who approves a change"))))
        .child(labelled(t, "lead, p", col().gap(t.space.at(3.0)).child(ty::lead(t, "A short guide to how the sample team plans, builds and ships.")).child(ty::p(t, "Plans live in one place. Each change gets one reviewer, who answers within a day; when nobody can, the change waits rather than going out unread.").key("type-p"))))
        .child(labelled(t, "quote", ty::blockquote(t, "Write it down, then make it smaller. The second draft is usually the one people read.")))
        .child(labelled(t, "list", ty::list(t, &["One reviewer for every change", "A day to answer", "Nothing ships unread"])))
        .child(labelled(t, "inline", row().gap(t.space.at(2.0)).center_y().child(ty::muted(t, "Run")).child(ty::inline_code(t, "rune-ui add typography").key("type-code")).child(ty::muted(t, "to copy it in."))))
        .child(labelled(t, "sizes", row().gap(t.space.at(6.0)).center_y().child(ty::large(t, "Large")).child(ty::small(t, "Small")).child(ty::muted(t, "Muted"))))
}
