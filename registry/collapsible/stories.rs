use rune::text;
use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::collapsible::Collapsible;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("collapsible", "Collapsible", "structure").story("Open and closed", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let body = || text("Retries, timeouts and where the logs go.").font(t.ty.base).color(t.ink_muted());
    specs(t, vec![
        spec(t, "Open", rune::col().w(320.0).child(Collapsible::<S>::new("collapsible-open", "More options", g.flag("collapsible-open")).meta("3").child(body()).on_toggle(move |s: &mut S| acc(s).toggle("collapsible-open")).build(t))),
        spec(t, "Closed, hover", rune::col().w(320.0).child(Collapsible::<S>::new("collapsible-closed", "Advanced", false).state(Look::Hover).build(t))),
        spec(t, "Disabled", rune::col().w(320.0).child(Collapsible::<S>::new("collapsible-disabled", "Locked section", false).disabled(true).build(t))),
    ])
}
