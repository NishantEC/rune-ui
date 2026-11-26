use rune::text;
use rune_ui_style::{State as Look, weight};

use super::super::base::{glyph, spinner_in};
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;
use super::super::portrait::{Activity, Portrait};
use super::super::sidebar::{Selected, Sidebar, SidebarItem};

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
    // Two-line rows: a portrait, a title over a second line, a count and a trailing pill; the current one raised; a button row.
    let people = g.choice("sidebar-people", "p1");
    let pill = |words: &str| rune::row().h(20.0).px(t.space.at(2.0)).center().radius(10.0).fill(t.c.selected).child(text(words.to_owned()).font(weight(t.ty.xs, t.ty.medium)).color(t.ink_muted()).a11y_hidden());
    let person = |k: &'static str, name: &str, sub: &str, a: Activity| {
        let key = k.to_owned();
        SidebarItem::<S>::new(format!("sidebar-{k}"), name.to_owned()).image(Portrait::new(format!("sidebar-{k}-face"), name.to_owned(), 28.0).state(a).ringed(true).under(t.c.surface.bg).decorative(true).build(t))
            .subtitle(sub.to_owned()).selected(people == k).selected_style(Selected::Raised).on_press(move |s: &mut S| acc(s).choose("sidebar-people", &key))
    };
    let two = Sidebar::<S>::new("sidebar-people-bar")
        .group(None, vec![SidebarItem::<S>::new("sidebar-new", "New task").icon(&icons::PLUS).button(true).on_press(|_: &mut S| {})])
        .list(Some("People"), "People", vec![
            person("p1", "Asha", "Fixing the login test", Activity::Working).count(3).count_label("3 unread").trail(pill("PR")),
            person("p2", "Lee", "Needs your answer", Activity::NeedsYou).trail(pill("2 min")),
            person("p3", "Sam", "Finished 10 minutes ago", Activity::None).count(12),
        ])
        .build(t);
    specs(t, vec![
        spec(t, "Header, two groups, footer (Calendar pinned to hover)", rune::col().h(400.0).child(bar)),
        spec(t, "A list of tasks: status marks lead each row; one list for assistive technology", rune::col().h(400.0).child(tasks)),
        spec(t, "Two lines: a portrait, a second line, a count, a trailing pill; the current row raised; New task is a button", rune::col().h(400.0).child(two)),
    ])
}
