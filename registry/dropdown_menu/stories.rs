use rune_ui_style::ButtonVariant;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::button::Button;
use super::super::dropdown_menu::{DropdownMenu, DropdownMenuItem};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("dropdown_menu", "DropdownMenu", "overlays").story("Account menu", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let open = g.is_open("dropdown-demo");
    let trigger = Button::<S>::new("dropdown-trigger", "Account").variant(ButtonVariant::Secondary).trailing_icon(&icons::CHEVRON_DOWN).on_press(move |s: &mut S| acc(s).toggle_open("dropdown-demo")).build(t);
    let menu = DropdownMenu::new("dropdown-demo")
        .trigger(trigger)
        .open(open)
        .on_close(move |s: &mut S| acc(s).close("dropdown-demo"))
        .min_width(200.0)
        .label("Placeholder workspace")
        .item(DropdownMenuItem::new("Profile").icon(&icons::USER).shortcut("⇧⌘P"))
        .item(DropdownMenuItem::new("Settings").icon(&icons::GEAR).shortcut("⌘,"))
        .item(DropdownMenuItem::new("Notifications").icon(&icons::BELL).checked(true))
        .item(DropdownMenuItem::new("Billing").disabled(true))
        .danger_item(DropdownMenuItem::new("Sign out"))
        .build(t, c.layer);
    specs(t, vec![spec(t, "Trigger (press it: the menu floats over the page)", menu)])
}
