use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::toggle_group::ToggleGroup;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("toggle_group", "ToggleGroup", "actions").story("Single and multiple", all::<S>)
}

const VIEWS: [(&str, Option<&'static rune::Icon>); 3] = [("List", None), ("Board", None), ("Calendar", Some(&icons::CALENDAR))];
const MARKS: [(&str, Option<&'static rune::Icon>); 3] = [("Bold", Some(&icons::BOLD)), ("Italic", Some(&icons::ITALIC)), ("Underline", Some(&icons::UNDERLINE))];

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let view: usize = g.choice("tgroup-view", "1").parse().unwrap_or(1);
    let single = |k: &str| ToggleGroup::<S>::single(format!("tgroup-{k}"), "View", &VIEWS, Some(view)).on_change(move |s: &mut S, i| acc(s).choose("tgroup-view", &i.to_string()));
    let marks = [g.flag("tgroup-0"), g.flag("tgroup-1"), !g.flag("tgroup-2")];
    specs(t, vec![
        spec(t, "Single", single("single").build(t)),
        spec(t, "Hover on List", single("hover").state(0, Look::Hover).build(t)),
        spec(t, "Focus", single("focus").state(1, Look::FocusVisible).build(t)),
        spec(t, "Multiple, icons", ToggleGroup::<S>::multiple("tgroup-marks", "Text style", &MARKS, &marks).icon_only().on_change(move |s: &mut S, i| acc(s).toggle(&format!("tgroup-{i}"))).build(t)),
        spec(t, "Disabled", single("disabled").disabled(true).build(t)),
    ])
}
