use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::checkbox::Checkbox;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("checkbox", "Checkbox", "form").story("States", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let cb = |k: &str, on: bool| Checkbox::<S>::new(format!("check-{k}"), "Email me a summary", on).on_toggle(move |s: &mut S| acc(s).toggle("check-on"));
    specs(t, vec![
        spec(t, "Off", cb("off", false).build(t)),
        spec(t, "On", cb("on", g.flag("check-on")).build(t)),
        spec(t, "Mixed", cb("mixed", false).mixed(true).build(t)),
        spec(t, "Hover", cb("hover", false).state(Look::Hover).build(t)),
        spec(t, "Focus, box alone", cb("focus", true).hide_label().state(Look::FocusVisible).build(t)),
        spec(t, "Invalid", cb("invalid", false).invalid(true).build(t)),
        spec(t, "Disabled", cb("disabled", true).disabled(true).build(t)),
        spec(t, "With help", Checkbox::<S>::new("check-help", "Share usage", false).help("Counts only, never content").build(t)),
    ])
}
