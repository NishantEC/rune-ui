use rune::text;
use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::tabs::{Activation, Tabs, TabsState, TabsVariant};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("tabs", "Tabs", "structure").story("Line and contained", all::<S>).story("Pane, closable, manual and overflow", more::<S>)
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

/// The open tabs of the closable demo, by name.
fn open(g: &super::super::gallery::GalleryState) -> Vec<String> {
    g.choice("tabs-open", "Overview,Changes,Subagent 1,Subagent 2,Subagent 3,Subagent 4,Subagent 5").split(',').map(str::to_owned).collect()
}

fn more<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let names = open(g);
    let at = g.index("tabs-closable", 0).min(names.len().saturating_sub(1));
    // A pane's header: compact, closable, the first tab permanent.
    let build = |key: &str, variant: TabsVariant, names: &[String], at: usize, state: Option<(usize, Look)>| {
        let mut tabs = Tabs::<S>::new(key.to_owned(), "Pane tabs").variant(variant).selected(at);
        for (i, n) in names.iter().enumerate() {
            tabs = tabs.tab(n.clone(), None);
            if i == 0 {
                tabs = tabs.permanent();
            }
        }
        if let Some((i, s)) = state {
            tabs = tabs.state(i, s);
        }
        tabs.on_change(move |s: &mut S, i| acc(s).choose("tabs-closable", &i.to_string())).on_close(move |s: &mut S, i| {
            let g = acc(s);
            let mut v = open(g);
            if i < v.len() && i > 0 {
                v.remove(i);
                g.choose("tabs-open", &v.join(","));
                let cur = g.index("tabs-closable", 0);
                g.choose("tabs-closable", &(if cur > i || cur >= v.len() { cur.saturating_sub(1) } else { cur }).to_string());
            }
        })
    };
    let few: Vec<String> = names.iter().take(3).cloned().collect();
    let pane = build("tabs-pane", TabsVariant::Pane, &few, at.min(2), None).build(t);
    let pane_hover = build("tabs-pane-hover", TabsVariant::Pane, &few, 0, Some((1, Look::Hover))).build(t);
    let manual = Tabs::<S>::new("tabs-manual", "Manual activation").tab("Overview", None).tab("Changes", Some(3)).tab("Files", None).selected(g.index("tabs-manual", 0)).activation(Activation::Manual).on_change(move |s: &mut S, i| acc(s).choose("tabs-manual", &i.to_string())).build(t);
    let state: TabsState = g.slot("tabs-overflow");
    let mut overflow = Tabs::<S>::new("tabs-overflow", "Subagents").variant(TabsVariant::Pane).selected(at).overflow(&state, move |s: &mut S| acc(s).slot_mut::<TabsState>("tabs-overflow"));
    for (i, n) in names.iter().enumerate() {
        overflow = overflow.tab(n.clone(), None);
        if i == 0 {
            overflow = overflow.permanent();
        }
    }
    let overflow = overflow.on_change(move |s: &mut S, i| acc(s).choose("tabs-closable", &i.to_string())).on_close(move |s: &mut S, i| {
        let g = acc(s);
        let mut v = open(g);
        if i > 0 && i < v.len() {
            v.remove(i);
            g.choose("tabs-open", &v.join(","));
            g.choose("tabs-closable", "0");
        }
    });
    let overflow = overflow.build_in(t, c.layer);
    specs(t, vec![
        spec(t, "Pane: compact, the first tab cannot close (Cmd+W on a tab closes it)", rune::col().w(360.0).child(pane)),
        spec(t, "Pane, hover shows the close button", rune::col().w(360.0).child(pane_hover)),
        spec(t, "Manual activation: arrows move focus, Return or Space chooses", rune::col().w(360.0).child(manual)),
        spec(t, "Overflow: what does not fit folds into More tabs", rune::col().w(300.0).child(overflow)),
    ])
}
