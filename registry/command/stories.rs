use rune_ui_style::ButtonVariant;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::button::Button;
use super::super::command::{Command, CommandItem};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("command", "Command", "overlays").story("Palette", all::<S>)
}

fn items() -> &'static [CommandItem] {
    static ITEMS: std::sync::OnceLock<Vec<CommandItem>> = std::sync::OnceLock::new();
    ITEMS.get_or_init(|| {
        vec![
            CommandItem::new("new", "New task", "Suggestions").icon(&icons::PLUS).shortcut("⌘ N"),
            CommandItem::new("inbox", "Open inbox", "Suggestions").icon(&icons::INBOX),
            CommandItem::new("calendar", "Show calendar", "Suggestions").icon(&icons::CALENDAR),
            CommandItem::new("profile", "Profile", "Settings").icon(&icons::USER).shortcut("⇧ ⌘ P"),
            CommandItem::new("settings", "Settings", "Settings").icon(&icons::GEAR).shortcut("⌘ ,"),
        ]
    })
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let at: usize = g.choice("command-at", "0").parse().unwrap_or(0);
    let inline = Command::<S>::new("command-inline", g.edit("command"), move |s: &mut S| acc(s).edit_mut("command")).items(items()).highlight(at).on_highlight(move |s: &mut S, i| acc(s).choose("command-at", &i.to_string())).on_select(move |s: &mut S, _| acc(s).presses += 1).build(t);
    let none = Command::<S>::new("command-none", g.edit("command-miss"), move |s: &mut S| acc(s).edit_mut("command-miss")).items(items()).width(360.0).build(t);
    Command::<S>::new("command-demo", g.edit("command"), move |s: &mut S| acc(s).edit_mut("command")).items(items()).highlight(at).on_highlight(move |s: &mut S, i| acc(s).choose("command-at", &i.to_string())).dialog(t, g.is_open("command-demo"), move |s: &mut S| acc(s).close("command-demo"), c.layer);
    specs(t, vec![
        spec(t, "Inline", inline),
        spec(t, "Nothing matches", none),
        spec(t, "Trigger (open in the open-command scene)", Button::<S>::new("command-open", "Search commands").variant(ButtonVariant::Outline).icon(&icons::SEARCH).on_press(move |s: &mut S| acc(s).toggle_open("command-demo")).build(t)),
    ])
}
