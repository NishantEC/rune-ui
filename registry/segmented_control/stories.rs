use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::segmented_control::SegmentedControl;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("segmented_control", "SegmentedControl", "form").story("A sliding thumb, one Tab stop, arrows choose", all::<S>)
}

const RANGE: [&str; 3] = ["Day", "Week", "Month"];
const VIEW: [&str; 4] = ["List", "Board", "Calendar", "Timeline"];

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let live = SegmentedControl::<S>::new("seg-demo", "Range", &RANGE, g.index("seg-demo", 1)).on_change(move |s: &mut S, i| acc(s).choose("seg-demo", &i.to_string())).build(t);
    let wide = SegmentedControl::<S>::new("seg-view", "View", &VIEW, g.index("seg-view", 0)).full().on_change(move |s: &mut S, i| acc(s).choose("seg-view", &i.to_string())).build(t);
    specs(t, vec![
        spec(t, "Press or use Left and Right: the thumb slides", live),
        spec(t, "Full width, four segments", rune::col().w(360.0).child(wide)),
        spec(t, "Small, hover on Week", SegmentedControl::<S>::new("seg-small", "Size", &["S", "M", "L"], 0).size(rune_ui_style::Size::Sm).state(1, Look::Hover).build(t)),
        spec(t, "Focus", SegmentedControl::<S>::new("seg-focus", "Range", &RANGE, 2).state(2, Look::FocusVisible).build(t)),
        spec(t, "Disabled", SegmentedControl::<S>::new("seg-off", "Range", &RANGE, 0).disabled(true).build(t)),
    ])
}
