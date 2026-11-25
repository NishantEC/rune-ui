use rune::text;
use rune_ui_style::{State as Look, weight};

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::sidebar::{Sidebar, SidebarItem};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("sidebar", "Sidebar", "structure").story("An app's side column", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let at = g.choice("sidebar", "home");
    let item = |k: &'static str, label: &str| {
        let key = k.to_owned();
        SidebarItem::<S>::new(format!("sidebar-{k}"), label.to_owned()).selected(at == k).on_press(move |s: &mut S| acc(s).choose("sidebar", &key))
    };
    let bar = Sidebar::<S>::new("sidebar")
        .header(text("Placeholder").font(weight(t.ty.md, t.ty.semibold)).color(t.ink()))
        .group(None, vec![item("home", "Home").icon(&icons::HOME), item("inbox", "Inbox").icon(&icons::INBOX).count(4), item("calendar", "Calendar").icon(&icons::CALENDAR).state(Look::Hover)])
        .group(Some("Projects"), vec![item("app", "placeholder-app").icon(&icons::FOLDER), item("docs", "placeholder-docs").icon(&icons::FOLDER).count(12)])
        .footer(text("someone@example.com").font(t.ty.sm).color(t.ink_muted()))
        .build(t);
    specs(t, vec![spec(t, "Header, two groups, footer (Calendar pinned to hover)", rune::col().h(400.0).child(bar))])
}
