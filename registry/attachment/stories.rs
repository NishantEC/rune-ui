use rune::El;

use super::super::attachment::{Attachment, Kind};
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("attachment", "Attachment", "agent").story("Files and images", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let mut items = vec![spec(t, "A file, removable", Attachment::<S>::new("attachment-file", "quarterly-report-final-v3.pdf").detail("240 KB").on_remove(move |s: &mut S| acc(s).toggle("attachment-gone")).build(t))];
    if g.flag("attachment-gone") {
        items.clear();
    }
    items.push(spec(t, "An image", Attachment::<S>::new("attachment-image", "diagram.png").kind(Kind::Image).detail("1.2 MB").on_remove(|_: &mut S| {}).build(t)));
    items.push(spec(t, "Sent (no remove)", Attachment::<S>::new("attachment-sent", "notes.md").build(t)));
    specs(t, items)
}
