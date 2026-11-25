use rune_ui_style::ButtonVariant;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::button::Button;
use super::super::icons;
use super::super::tooltip::Tooltip;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("tooltip", "Tooltip", "overlays").story("Panels and a live trigger", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    specs(t, vec![
        spec(t, "Words", Tooltip::new("Copy link").panel(t, "tooltip-words")),
        spec(t, "With keys", Tooltip::new("Save").keys("⌘S").panel(t, "tooltip-keys")),
        spec(t, "Live: hover the button", Tooltip::new("Settings").keys("⌘,").wrap(Button::<S>::new("tooltip-trigger", "Settings").variant(ButtonVariant::Ghost).icon_only(&icons::GEAR).build(t))),
    ])
}
