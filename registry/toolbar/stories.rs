use rune::El;

use super::super::gallery::{Ctx, GalleryState, StoryItem, spec, spec_wide, specs};
use super::super::icons;
use super::super::toolbar::{Tool, Toolbar, ToolbarState};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("toolbar", "Toolbar", "actions").story("Groups, one Tab stop, overflow into More", all::<S>)
}

fn bar<S: 'static>(c: &mut Ctx<S>, key: &'static str) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let flip = move |k: &'static str| move |s: &mut S| acc(s).toggle(k);
    let did = move |what: &'static str| move |s: &mut S| acc(s).choose("toolbar-did", what);
    let state: ToolbarState = g.slot(key);
    Toolbar::<S>::new(key, "Formatting", &state, move |s: &mut S| state_of(acc(s), key))
        .group(vec![
            Tool::toggle("Bold", &icons::BOLD, g.flag("tb-bold"), flip("tb-bold")),
            Tool::toggle("Italic", &icons::ITALIC, g.flag("tb-italic"), flip("tb-italic")),
            Tool::toggle("Underline", &icons::UNDERLINE, g.flag("tb-underline"), flip("tb-underline")),
        ])
        .group(vec![Tool::button("Link", &icons::LINK, did("link")), Tool::button("Image", &icons::IMAGE, did("image")), Tool::button("Attach", &icons::PAPERCLIP, did("attach"))])
        .group(vec![Tool::button("Undo", &icons::RETRY, did("undo")), Tool::text("Clear formatting", did("clear")), Tool::button("Settings", &icons::GEAR, did("settings")).disabled(true)])
        .build(t, c.layer)
}

fn state_of<'a>(g: &'a mut GalleryState, key: &str) -> &'a mut ToolbarState {
    g.slot_mut::<ToolbarState>(key)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let t = c.t;
    let wide = bar(c, "toolbar-demo");
    let narrow = bar(c, "toolbar-narrow");
    specs(t, vec![
        spec_wide(t, "Everything fits: Tab once, then Left and Right", 520.0, wide),
        spec(t, "Narrow: the rest is under More", rune::col().w(220.0).child(narrow)),
    ])
}
