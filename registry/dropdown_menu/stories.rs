use rune_ui_style::ButtonVariant;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::button::Button;
use super::super::dropdown_menu::{DropdownMenu, DropdownMenuItem};
use super::super::icons;
use super::super::portrait::Portrait;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("dropdown_menu", "DropdownMenu", "overlays").story("Account menu", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let open = g.is_open("dropdown-demo");
    let trigger = Button::<S>::new("dropdown-trigger", "Account").variant(ButtonVariant::Secondary).trailing_icon(&icons::CHEVRON_DOWN).on_press(move |s: &mut S| acc(s).toggle_open("dropdown-demo")).build(t);
    let menu = DropdownMenu::<S>::new("dropdown-demo")
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
    // Rows that lead with a picture: a portrait for each person, a swatch for the colour.
    let who = g.choice("dropdown-people", "asha");
    let face = |k: &str, name: &str| Portrait::new(format!("dropdown-people-{k}-face"), name.to_owned(), 20.0).decorative(true).build(t);
    // Built in a 20 point box like the portraits, so every label starts at the same place.
    let swatch = rune::row().size(20.0, 20.0).center().child(rune::el().size(12.0, 12.0).radius(6.0).fill(t.identity().solid));
    let person = |k: &'static str, name: &str| DropdownMenuItem::new(name.to_owned()).key(format!("dropdown-people-{k}")).image(face(k, name)).checked(who == k).on_select(move |s: &mut S| acc(s).choose("dropdown-people", k));
    let people_trigger = Button::<S>::new("dropdown-people-open", "Assign to").variant(ButtonVariant::Secondary).trailing_icon(&icons::CHEVRON_DOWN).on_press(move |s: &mut S| acc(s).toggle_open("dropdown-people")).build(t);
    let people = DropdownMenu::<S>::new("dropdown-people")
        .trigger(people_trigger)
        .open(g.is_open("dropdown-people"))
        .on_close(move |s: &mut S| acc(s).close("dropdown-people"))
        .min_width(220.0)
        .label("Assign to")
        .item(person("asha", "Asha"))
        .item(person("lee", "Lee"))
        .item(DropdownMenuItem::new("Identity colour").image(swatch).shortcut("⌥ ⌘ I"))
        .build(t, c.layer);
    specs(t, vec![spec(t, "Trigger (press it: the menu floats over the page)", menu), spec(t, "Rows that lead with a picture or an element", people)])
}
