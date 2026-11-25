use super::super::breadcrumb::Breadcrumb;
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("breadcrumb", "Breadcrumb", "structure").story("Paths", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, acc) = (c.t, c.access);
    specs(t, vec![
        spec(t, "Two levels", Breadcrumb::<S>::new("crumbs", "Appearance").part("Settings", move |s: &mut S| acc(s).presses += 1).build(t)),
        spec(t, "Three levels", Breadcrumb::<S>::new("crumbs-deep", "Placeholder task").part("Workspace", move |s: &mut S| acc(s).presses += 1).plain("Projects").build(t)),
    ])
}
