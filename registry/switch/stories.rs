use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::switch::Switch;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("switch", "Switch", "form").story("States", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let sw = |k: &str, on: bool| Switch::<S>::new(format!("switch-{k}"), "Sync", on).on_toggle(move |s: &mut S| acc(s).toggle("switch-on"));
    specs(t, vec![
        spec(t, "Off", sw("off", false).build(t)),
        spec(t, "On", sw("on", g.flag("switch-on")).build(t)),
        spec(t, "Focus", sw("focus", true).state(Look::FocusVisible).build(t)),
        spec(t, "Disabled off", sw("disabled-off", false).disabled(true).build(t)),
        spec(t, "Disabled on", sw("disabled-on", true).disabled(true).build(t)),
        spec(t, "With words", rune::col().w(320.0).child(Switch::<S>::new("switch-line", "Sync in the background", g.flag("switch-on")).help("Keeps placeholder data fresh").on_toggle(move |s: &mut S| acc(s).toggle("switch-on")).build(t))),
    ])
}
