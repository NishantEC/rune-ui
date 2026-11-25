use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::textarea::Textarea;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("textarea", "Textarea", "form").story("States", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let a = |k: &str| Textarea::<S>::new(format!("textarea-{k}"), g.edit("notes"), move |s: &mut S| acc(s).edit_mut("notes")).name("Notes").width(260.0);
    specs(t, vec![
        spec(t, "Rest", a("rest").build(t)),
        spec(t, "Focus", a("focus").state(Look::FocusVisible).build(t)),
        spec(t, "Placeholder", Textarea::<S>::new("textarea-empty", g.edit("empty"), move |s: &mut S| acc(s).edit_mut("empty")).placeholder("Write a note").rows(2, 6).width(260.0).build(t)),
        spec(t, "Invalid", a("invalid").invalid(true).build(t)),
        spec(t, "Disabled", a("disabled").disabled(true).build(t)),
    ])
}
