use rune_ui_style::ButtonVariant;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::button::Button;
use super::super::dialog::Dialog;
use super::super::input::Input;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("dialog", "Dialog", "overlays").story("Panel and modal", all::<S>)
}

fn dialog<S: 'static>(c: &Ctx<S>, key: &str) -> Dialog<S> {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let k = key.to_owned();
    let cancel = Button::<S>::new(format!("{key}-cancel"), "Cancel").variant(ButtonVariant::Secondary).on_press(move |s: &mut S| acc(s).close(&k)).build(t);
    let save = Button::<S>::new(format!("{key}-save"), "Save").build(t);
    let k = key.to_owned();
    Dialog::<S>::new(key.to_owned(), "Rename project").description("A new name for the placeholder project. Links to it keep working.").child(Input::<S>::new(format!("{key}-name"), g.edit("name"), move |s: &mut S| acc(s).edit_mut("name")).name("Name").build(t)).footer(vec![cancel, save]).on_close(move |s: &mut S| acc(s).close(&k))
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let t = c.t;
    let mut inplace = dialog(c, "dialog-inplace");
    let panel = inplace.panel(t);
    let open = c.g.is_open("dialog-demo");
    dialog(c, "dialog-demo").open(open).return_focus("dialog-open").build(t, c.layer);
    let trigger = Button::<S>::new("dialog-open", "Rename…").variant(ButtonVariant::Secondary).on_press(move |s: &mut S| acc(s).toggle_open("dialog-demo")).build(t);
    specs(t, vec![spec(t, "Panel", rune::col().w(440.0).child(panel)), spec(t, "Trigger (press it: the dialog opens over a scrim)", trigger)])
}
