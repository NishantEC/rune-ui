use rune_ui_style::Tone;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::progress::Progress;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("progress", "Progress", "display").story("Values", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    specs(t, vec![
        spec(t, "Bare", Progress::new("progress-bare", 0.3).width(200.0).build(t)),
        spec(t, "Label and figure", Progress::new("progress-upload", 0.62).label("Uploading").figure("62%").width(220.0).build(t)),
        spec(t, "Near the limit", Progress::new("progress-budget", 0.86).label("Budget used").figure("$8.60 of $10").tone(Tone::Warning).width(220.0).build(t)),
        spec(t, "Done", Progress::new("progress-done", 1.0).label("Synced").figure("100%").tone(Tone::Success).width(200.0).build(t)),
    ])
}
