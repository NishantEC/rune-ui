use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::slider::Slider;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("slider", "Slider", "form").story("States", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let v = g.value("slider", 40.0);
    let sl = |k: &str, v: f32| Slider::<S>::new(format!("slider-{k}"), "Volume", v).width(220.0).on_change(move |s: &mut S, v| acc(s).set_value("slider", v));
    specs(t, vec![
        spec(t, "Live", sl("live", v).build(t)),
        spec(t, "Empty", sl("min", 0.0).build(t)),
        spec(t, "Full", sl("max", 100.0).build(t)),
        spec(t, "Focus", sl("focus", 65.0).state(Look::FocusVisible).build(t)),
        spec(t, "Disabled", sl("disabled", 30.0).disabled(true).build(t)),
    ])
}
