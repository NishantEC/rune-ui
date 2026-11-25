use rune::{El, col, row, text};
use rune_ui_style::{ButtonVariant, Size, weight};

use super::super::button::Button;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::item::{Item, ItemVariant};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("item", "Item", "structure").story("Default, outline, muted, small, pressable", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, acc) = (c.t, c.access);
    let open = |k: &str| Button::<S>::new(k.to_owned(), "Open").variant(ButtonVariant::Outline).size(Size::Sm).build(t);
    let face = row().size(40.0, 40.0).radius(20.0).fill(t.c.muted.bg).center().child(text("AR").font(weight(t.ty.sm, t.ty.semibold)).color(t.ink_muted()));
    let list = col().w(440.0).gap(t.space.at(3.0))
        .child(Item::<S>::new("item-default", "Billing").description("Plans, invoices and the card on file").icon(&icons::FILE).action(open("item-default-open")).build(t))
        .child(Item::<S>::new("item-outline", "Asha Rao").description("asha@example.com · Admin").media(face).variant(ItemVariant::Outline).action(Button::<S>::new("item-outline-more", "More").variant(ButtonVariant::Ghost).size(Size::Sm).icon_only(&icons::MORE).build(t)).build(t))
        .child(Item::<S>::new("item-muted", "Two-factor sign-in is off").description("Turn it on to protect the placeholder workspace.").icon(&icons::WARNING).variant(ItemVariant::Muted).action(Button::<S>::new("item-muted-on", "Turn on").size(Size::Sm).build(t)).build(t))
        .child(Item::<S>::new("item-small", "Notifications").icon(&icons::BELL).size(Size::Sm).variant(ItemVariant::Outline).on_press(move |s: &mut S| acc(s).presses += 1).action(rune::el().child(super::super::base::glyph(&icons::CHEVRON_RIGHT, t.control.icon, t.ink_faint()))).build(t));
    specs(t, vec![spec(t, "The last one is pressable", list)])
}
