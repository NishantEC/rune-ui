use rune::text;
use rune_ui_style::{State as Look, weight};

use super::super::base::{glyph, spinner_in};
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
    let open = g.choice("sidebar-task", "t2");
    let task = |k: &'static str, label: &str, word: &str| {
        let key = k.to_owned();
        SidebarItem::<S>::new(format!("sidebar-{k}"), label.to_owned()).selected(open == k).description(word.to_owned()).on_press(move |s: &mut S| acc(s).choose("sidebar-task", &key))
    };
    let dot = |c| rune::el().size(8.0, 8.0).radius(4.0).fill(c);
    let tasks = Sidebar::<S>::new("sidebar-tasks")
        .list(Some("Today"), "Tasks from today", vec![
            task("t1", "Retry failed pages", "Running").lead(spinner_in(t, t.control.icon, t.ink_muted(), "Running")),
            task("t2", "Upgrade the parser", "Done").lead(dot(t.c.success.solid)),
            task("t3", "Delete old fixtures", "Needs you").lead(glyph(&icons::WARNING, t.control.icon, t.c.warning.fg)),
            task("t4", "Write the migration guide", "Pending").icon(&icons::FILE_TEXT),
        ])
        .build(t);
    specs(t, vec![
        spec(t, "Header, two groups, footer (Calendar pinned to hover)", rune::col().h(400.0).child(bar)),
        spec(t, "A list of tasks: status marks lead each row; one list for assistive technology", rune::col().h(400.0).child(tasks)),
    ])
}
