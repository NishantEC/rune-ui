use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::alert_dialog::AlertDialog;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("alert_dialog", "AlertDialog", "overlays").story("Confirm and destructive", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let t = c.t;
    let confirm = AlertDialog::<S>::new("alert-confirm", "Publish the placeholder site?").description("Everyone with the link will see the new version.").cancel("Not yet", |_: &mut S| {}).action("Publish", move |s: &mut S| acc(s).presses += 1).panel(t);
    let destroy = AlertDialog::<S>::new("alert-destroy", "Delete the placeholder project?").description("Its tasks and history go too. This cannot be undone.").destructive().cancel("Keep it", |_: &mut S| {}).action("Delete", move |s: &mut S| acc(s).presses += 1).panel(t);
    specs(t, vec![spec(t, "Confirm", rune::col().w(400.0).child(confirm)), spec(t, "Destructive", rune::col().w(400.0).child(destroy))])
}
