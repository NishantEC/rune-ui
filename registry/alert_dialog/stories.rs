use rune_ui_style::ButtonVariant;

use super::super::alert_dialog::AlertDialog;
use super::super::button::Button;
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("alert_dialog", "AlertDialog", "overlays").story("Confirm and destructive", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let confirm = AlertDialog::<S>::new("alert-confirm", "Publish the placeholder site?").description("Everyone with the link will see the new version.").cancel("Not yet", |_: &mut S| {}).action("Publish", move |s: &mut S| acc(s).presses += 1).panel(t);
    let destroy = AlertDialog::<S>::new("alert-destroy", "Delete the placeholder project?").description("Its tasks and history go too. This cannot be undone.").destructive().cancel("Keep it", |_: &mut S| {}).action("Delete", move |s: &mut S| acc(s).presses += 1).panel(t);
    AlertDialog::<S>::new("alert-demo", "Delete the placeholder project?").description("Its tasks and history go too. This cannot be undone.").destructive().open(g.is_open("alert-demo")).return_focus("alert-open").cancel("Keep it", move |s: &mut S| acc(s).close("alert-demo")).action("Delete", move |s: &mut S| acc(s).close("alert-demo")).build(t, c.layer);
    let trigger = Button::<S>::new("alert-open", "Delete project…").variant(ButtonVariant::Destructive).on_press(move |s: &mut S| acc(s).toggle_open("alert-demo")).build(t);
    specs(t, vec![spec(t, "Confirm", rune::col().w(400.0).child(confirm)), spec(t, "Destructive", rune::col().w(400.0).child(destroy)), spec(t, "Live (press it: it needs an answer; a press outside does nothing)", trigger)])
}
