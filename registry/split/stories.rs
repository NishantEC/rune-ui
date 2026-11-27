use rune::{El, col, text};
use rune_ui_style::{State as Look, Theme};

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::split::{Edge, Region, Split};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("split", "Split", "structure").story("Three regions, two and the handles", all::<S>)
}

fn body(t: &Theme, words: &str) -> El {
    col().w_full().h_full().center().child(text(words.to_owned()).font(t.ty.sm).color(t.ink_muted()).truncate().min_w(0.0))
}

/// The centre, on the page colour: the regions are told apart by their fills, as in a window.
fn page(t: &Theme, words: &str) -> El {
    body(t, words).fill(t.c.page.bg)
}

fn framed(t: &Theme, e: El, w: f32, h: f32) -> El {
    col().size(w, h).radius(t.radius.xl).clip_rounded(t.radius.xl).fill(t.c.surface.bg).child(e).child(rune::el().absolute(0.0, 0.0).size(w, h).radius(t.radius.xl).border(1.0, t.c.line).a11y_hidden())
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let side = g.value("split-side", 120.0);
    let pane = g.value("split-pane", 130.0);
    let hide_side = g.flag("split-side-hidden");
    let hide_pane = g.flag("split-pane-hidden");
    let three = Split::<S>::new("split-demo", "Window regions")
        .center_min(120.0)
        .step(10.0)
        .left(Region::new(body(t, "Sidebar")).name("sidebar").width(side).min(80.0).max(170.0).default(120.0).collapsible(true).collapsed(hide_side))
        .center(page(t, "Conversation"))
        .right(Region::new(body(t, "Pane")).name("pane").width(pane).min(90.0).max(200.0).max_fraction(0.5).default(130.0).collapsible(true).collapsed(hide_pane))
        .on_resize(move |s: &mut S, e, w| acc(s).set_value(if e == Edge::Left { "split-side" } else { "split-pane" }, w))
        .on_collapse(move |s: &mut S, e, hidden| {
            let g = acc(s);
            let k = if e == Edge::Left { "split-side-hidden" } else { "split-pane-hidden" };
            if g.flag(k) != hidden {
                g.toggle(k);
            }
        })
        .build(t);
    let two = Split::<S>::new("split-two", "Sidebar and content")
        .center_min(120.0)
        .left(Region::new(body(t, "Sidebar")).name("sidebar").width(g.value("split-two", 110.0)).min(80.0).max(170.0).default(110.0))
        .center(page(t, "Content"))
        .on_resize(move |s: &mut S, _, w| acc(s).set_value("split-two", w))
        .state(Edge::Left, Look::Hover)
        .build(t);
    let focus = Split::<S>::new("split-focus", "Content and pane")
        .center_min(120.0)
        .center(page(t, "Content"))
        .right(Region::new(body(t, "Pane")).name("pane").width(120.0).min(90.0).max(200.0).default(120.0))
        .on_resize(|_: &mut S, _, _| {})
        .state(Edge::Right, Look::FocusVisible)
        .build(t);
    specs(t, vec![
        spec(t, "Three regions: drag an edge; double-click resets; past the least it collapses", framed(t, three, 440.0, 200.0)),
        spec(t, "Two regions, handle hovered", framed(t, two, 300.0, 120.0)),
        spec(t, "Handle focused", framed(t, focus, 300.0, 120.0)),
    ])
}
