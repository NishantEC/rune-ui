use rune::{col, text};

use super::super::gallery::{Ctx, StoryItem};
use super::super::context_menu::{ContextMenu, ContextMenuItem};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("context_menu", "ContextMenu", "overlays").story("Right-click area", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let area = rune_ui_style::paint(col().key("context-area").w(320.0).h(120.0).center().radius(t.radius.lg), &t.mat.slab.base).child(text("Right-click here").font(t.ty.base).color(t.ink_muted()));
    let point = None;
    let mut menu = ContextMenu::new("context-demo", area, point).item(ContextMenuItem::new("Copy").icon(&icons::COPY).shortcut("⌘C")).item(ContextMenuItem::new("Rename")).danger_item(ContextMenuItem::new("Delete").icon(&icons::TRASH)).on_close(move |s: &mut S| acc(s).close("context-demo"));
    if g.is_open("context-demo") {
        menu = menu.at(220.0, 260.0);
    }
    menu.build(t, c.layer)
}
