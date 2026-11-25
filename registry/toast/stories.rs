use rune_ui_style::Tone;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::toast::Toast;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("toast", "Toast", "display").story("Messages", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, acc) = (c.t, c.access);
    specs(t, vec![
        spec(t, "Plain", Toast::<S>::new("toast-plain", "Saved").description("All changes are on disk.").build(t)),
        spec(t, "Meaning, action, close", Toast::<S>::new("toast-undo", "Task deleted").description("It is in the bin for 30 days.").tone(Tone::Success).action("Undo", move |s: &mut S| acc(s).presses += 1).on_close(move |s: &mut S| acc(s).presses += 1).build(t)),
    ])
}
