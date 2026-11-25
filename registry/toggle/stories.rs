use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::toggle::Toggle;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("toggle", "Toggle", "actions").story("States", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let bold = |k: &str, on: bool| Toggle::<S>::new(format!("toggle-{k}"), "Bold", on).icon_only(&icons::BOLD).on_toggle(move |s: &mut S| acc(s).toggle("toggle-bold"));
    specs(t, vec![
        spec(t, "Off", bold("off", false).build(t)),
        spec(t, "On", bold("on", g.flag("toggle-bold")).build(t)),
        spec(t, "Hover", bold("hover", false).state(Look::Hover).build(t)),
        spec(t, "Focus", bold("focus", false).state(Look::FocusVisible).build(t)),
        spec(t, "Disabled", bold("disabled", false).disabled(true).build(t)),
        spec(t, "With words", Toggle::<S>::new("toggle-words", "Italic", true).icon(&icons::ITALIC).build(t)),
        spec(t, "Outline", Toggle::<S>::new("toggle-outline", "Underline", false).icon(&icons::UNDERLINE).outline().build(t)),
    ])
}
