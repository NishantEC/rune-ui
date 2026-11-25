use rune::{col, text};

use super::super::context_menu::{ContextMenu, ContextMenuItem};
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("context_menu", "ContextMenu", "overlays").story("Right-click area", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let area = rune_ui_style::paint(col().key("context-area").w(320.0).h(120.0).center().radius(t.radius.lg), &t.mat.slab.base).child(text("Right-click here").font(t.ty.base).color(t.ink_muted())).on_secondary_press(move |s: &mut S, p, _| {
        let g = acc(s);
        g.point = Some(p);
        g.open = Some("context-demo".to_owned());
    });
    let point = g.point.filter(|_| g.is_open("context-demo"));
    let close = move |s: &mut S| {
        let g = acc(s);
        g.point = None;
        g.close("context-demo");
    };
    let menu = ContextMenu::new("context-demo", area, point).item(ContextMenuItem::new("Copy").icon(&icons::COPY).shortcut("⌘C").on_select(close)).item(ContextMenuItem::new("Rename").on_select(close)).danger_item(ContextMenuItem::new("Delete").icon(&icons::TRASH).on_select(close)).on_close(close).build(t, c.layer);
    specs(t, vec![spec(t, "Right-click: the menu opens at the pointer, over the page", menu)])
}
