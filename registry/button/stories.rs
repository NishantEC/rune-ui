use rune_ui_style::{ButtonVariant, Size, State as Look};

use super::super::gallery::{Ctx, StoryItem, labelled, spec, specs};
use super::super::button::Button;
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("button", "Button", "actions").story("Variants and states", variants::<S>).story("Sizes and shapes", shapes::<S>)
}

fn variants<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let t = c.t;
    let mut col = rune::col().gap(t.space.at(4.0));
    for v in ButtonVariant::ALL {
        let k = v.word().to_lowercase();
        let b = |s: &str| Button::<S>::new(format!("button-{k}-{s}"), "Save changes").variant(v).on_press(move |s: &mut S| acc(s).presses += 1);
        let items = vec![
            spec(t, "Rest", b("rest").build(t)),
            spec(t, "Hover", b("hover").state(Look::Hover).build(t)),
            spec(t, "Pressed", b("pressed").state(Look::Pressed).build(t)),
            spec(t, "Focus", b("focus").state(Look::FocusVisible).build(t)),
            spec(t, "Disabled", b("disabled").disabled(true).build(t)),
            spec(t, "Loading", b("loading").loading(true).build(t)),
        ];
        col = col.child(labelled(t, v.word(), specs(t, items)));
    }
    col
}

fn shapes<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    let mut items: Vec<rune::El> = Size::ALL.iter().map(|z| spec(t, z.word(), Button::<S>::new(format!("button-size-{}", z.word().to_lowercase()), "Continue").variant(ButtonVariant::Secondary).size(*z).trailing_icon(&icons::ARROW_RIGHT).build(t))).collect();
    items.push(spec(t, "Icon and words", Button::<S>::new("button-icon", "New item").icon(&icons::PLUS).build(t)));
    items.push(spec(t, "Icon only", Button::<S>::new("button-icon-only", "Settings").variant(ButtonVariant::Ghost).icon_only(&icons::GEAR).build(t)));
    items.push(spec(t, "Selected", Button::<S>::new("button-selected", "Pinned").variant(ButtonVariant::Secondary).selected(true).build(t)));
    items.push(spec(t, "Destructive", Button::<S>::new("button-delete", "Delete").variant(ButtonVariant::Destructive).icon(&icons::TRASH).build(t)));
    specs(t, items)
}
