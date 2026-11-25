use rune_ui_style::{ButtonVariant, State as Look};

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::button_group::ButtonGroup;
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("button_group", "ButtonGroup", "actions").story("Joined actions", all::<S>)
}

fn group<S: 'static>(acc: super::super::gallery::Access<S>, key: &str) -> ButtonGroup<S> {
    ButtonGroup::<S>::new(key.to_owned(), "History").item(format!("{key}-undo"), "Undo", None, move |s: &mut S| acc(s).presses += 1).item(format!("{key}-redo"), "Redo", None, move |s: &mut S| acc(s).presses += 1).item(format!("{key}-copy"), "Copy", Some(&icons::COPY), move |s: &mut S| acc(s).presses += 1)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let t = c.t;
    specs(t, vec![
        spec(t, "Secondary", group(acc, "bgroup").build(t)),
        spec(t, "Hover on Redo", group(acc, "bgroup-hover").state(1, Look::Hover).build(t)),
        spec(t, "Outline", group(acc, "bgroup-outline").variant(ButtonVariant::Outline).build(t)),
        spec(t, "Disabled", group(acc, "bgroup-disabled").disabled(true).build(t)),
    ])
}
