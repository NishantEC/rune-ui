use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::menubar::{Menu, Menubar};
use super::super::popup::Row;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("menubar", "Menubar", "overlays").story("File, Edit, View", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let open = g.open.as_deref().and_then(|o| o.strip_prefix("menubar-demo-")).and_then(|i| i.parse().ok());
    let done = move |s: &mut S| {
        let g = acc(s);
        g.presses += 1;
        g.open = None;
    };
    let toolbar = g.flag("menubar-toolbar");
    let bar = Menubar::<S>::new("menubar-demo", "Application menu")
        .menu(Menu::new("File").row(Row::new("New tab").shortcut("⌘T").on_select(done)).row(Row::new("New window").shortcut("⌘N").on_select(done)).row(Row::new("New private window").disabled(true)).separator().row(Row::new("Share").icon(&icons::COPY).on_select(done)).separator().row(Row::new("Print…").shortcut("⌘P").on_select(done)))
        .menu(Menu::new("Edit").row(Row::new("Undo").shortcut("⌘Z").on_select(done)).row(Row::new("Redo").shortcut("⇧⌘Z").on_select(done)).separator().row(Row::new("Cut").shortcut("⌘X").on_select(done)).row(Row::new("Copy").shortcut("⌘C").on_select(done)).row(Row::new("Paste").shortcut("⌘V").on_select(done)))
        .menu(Menu::new("View").label("Show").row(Row::new("Toolbar").checked(toolbar).on_select(move |s: &mut S| acc(s).toggle("menubar-toolbar"))).row(Row::new("Status bar").checked(true).on_select(done)).separator().row(Row::new("Enter full screen").shortcut("⌃⌘F").on_select(done)))
        .menu(Menu::new("Profiles").disabled(true))
        .open(open)
        .on_open(move |s: &mut S, m: Option<usize>| acc(s).open = m.map(|i| format!("menubar-demo-{i}")))
        .build(t, c.layer);
    specs(t, vec![spec(t, "Press a title, or Tab to it and press Down; Left and Right move between menus", bar)])
}
