use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::radio_group::RadioGroup;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("radio_group", "RadioGroup", "form").story("Choices", all::<S>)
}

const LEVELS: [(&str, Option<&str>); 3] = [("Careful", Some("Asks before anything leaves the app")), ("Balanced", Some("The default")), ("Hands-off", None)];
const SIZES: [(&str, Option<&str>); 3] = [("Small", None), ("Medium", None), ("Large", None)];

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let level: usize = g.choice("radio-level", "1").parse().unwrap_or(1);
    specs(t, vec![
        spec(t, "Vertical, with help", RadioGroup::<S>::new("radio-level", "Starting level", &LEVELS, Some(level)).on_change(move |s: &mut S, i| acc(s).choose("radio-level", &i.to_string())).build(t)),
        spec(t, "Hover and focus", RadioGroup::<S>::new("radio-pinned", "Size", &SIZES, Some(1)).state(0, Look::Hover).build(t)),
        spec(t, "Horizontal", RadioGroup::<S>::new("radio-row", "Size", &SIZES, Some(2)).horizontal().build(t)),
        spec(t, "Disabled", RadioGroup::<S>::new("radio-disabled", "Size", &SIZES, Some(0)).disabled(true).build(t)),
    ])
}
