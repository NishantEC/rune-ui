use rune::{El, col, text};

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::resizable::{Pane, Resizable};
use rune_ui_style::Theme;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("resizable", "Resizable", "structure").story("Horizontal and vertical", all::<S>)
}

fn body(t: &Theme, words: &str) -> El {
    col().w_full().h_full().center().child(text(words.to_owned()).font(t.ty.base).color(t.ink_muted()))
}

fn sizes(g: &super::super::gallery::GalleryState, k: &str, d: &[f32]) -> Vec<f32> {
    g.choices.get(k).map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_else(|| d.to_vec())
}

fn framed(t: &Theme, e: El, w: f32, h: f32) -> El {
    rune_ui_style::paint(col().size(w, h).radius(t.radius.xl).clip_rounded(t.radius.xl), &t.mat.slab.base).child(e)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let store = |k: &'static str| move |s: &mut S, v: &[f32]| acc(s).choose(k, &v.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>().join(","));
    let h = Resizable::<S>::new("resizable-demo", "Sidebar and content").pane(Pane::new(body(t, "Sidebar")).name("the sidebar").min(0.2).max(0.6)).pane(Pane::new(body(t, "Content")).min(0.3)).sizes(&sizes(g, "resizable-demo", &[0.35, 0.65])).on_resize(store("resizable-demo")).build(t);
    let v = Resizable::<S>::new("resizable-stack", "Editor and terminal").vertical().grip().pane(Pane::new(body(t, "Editor")).name("the editor").min(0.25)).pane(Pane::new(body(t, "Terminal")).min(0.2)).sizes(&sizes(g, "resizable-stack", &[0.6, 0.4])).on_resize(store("resizable-stack")).build(t);
    let three = Resizable::<S>::new("resizable-three", "Three panes").grip().pane(Pane::new(body(t, "One")).min(0.15)).pane(Pane::new(body(t, "Two")).min(0.15)).pane(Pane::new(body(t, "Three")).min(0.15)).sizes(&sizes(g, "resizable-three", &[0.25, 0.5, 0.25])).on_resize(store("resizable-three")).build(t);
    specs(t, vec![spec(t, "Drag the hairline, or Tab to it and press the arrows", framed(t, h, 420.0, 200.0)), spec(t, "Vertical, with a grip", framed(t, v, 260.0, 200.0)), spec(t, "Three panes", framed(t, three, 420.0, 120.0))])
}
