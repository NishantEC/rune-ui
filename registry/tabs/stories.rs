use rune::text;
use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::tabs::{Tabs, TabsVariant};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("tabs", "Tabs", "structure").story("Line and contained", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let at: usize = g.choice("tabs", "0").parse().unwrap_or(0);
    let tabs = |k: &str| Tabs::<S>::new(format!("tabs-{k}"), "Project views").tab("Overview", None).icon(&icons::HOME).tab("Activity", Some(12)).tab("Settings", None).selected(at).on_change(move |s: &mut S, i| acc(s).choose("tabs", &i.to_string()));
    let words = ["The placeholder project at a glance.", "Twelve things happened this week.", "How the project behaves."];
    specs(t, vec![
        spec(t, "Line, with content", rune::col().w(420.0).child(tabs("line").content(text(words[at.min(2)]).font(t.ty.base).color(t.ink_muted())).build(t))),
        spec(t, "Hover and focus", rune::col().w(420.0).child(tabs("pinned").state(1, Look::FocusVisible).build(t))),
        spec(t, "Contained", tabs("contained").variant(TabsVariant::Contained).build(t)),
    ])
}
