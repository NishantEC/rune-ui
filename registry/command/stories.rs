use rune::{El, el};
use rune_ui_style::{ButtonVariant, Theme, Tone};

use super::super::button::Button;
use super::super::command::{Command, CommandItem};
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::icons;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("command", "Command", "overlays").story("Palette", all::<S>).story("Fuzzy matches, recents and loading", search::<S>)
}

fn mark(t: &Theme, tone: Tone) -> El {
    el().size(8.0, 8.0).radius(4.0).fill(tone.colors(t).1)
}

/// Actions, settings and a few threads: rows with an icon, a status mark, a second line, a hint and shortcuts.
fn items(t: &Theme) -> Vec<CommandItem> {
    vec![
        CommandItem::new("new", "New task", "Suggestions").icon(&icons::PLUS).shortcut("⌘ N"),
        CommandItem::new("inbox", "Open inbox", "Suggestions").icon(&icons::INBOX),
        CommandItem::new("calendar", "Show calendar", "Suggestions").icon(&icons::CALENDAR),
        CommandItem::new("profile", "Profile", "Settings").icon(&icons::USER).shortcut("⇧ ⌘ P"),
        CommandItem::new("settings", "Settings", "Settings").icon(&icons::GEAR).shortcut("⌘ ,"),
        CommandItem::new("t1", "Fix the login redirect", "Threads").lead(mark(t, Tone::Info)).subtitle("acme-web · Working").hint("2m").keywords("auth bug"),
        CommandItem::new("t2", "Plan the release notes", "Threads").lead(mark(t, Tone::Warning)).subtitle("example.com · Needs you").hint("1h"),
        CommandItem::new("t3", "Review ENG-123", "Threads").lead(mark(t, Tone::Success)).subtitle("acme-web · Settled").hint("3d"),
    ]
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let items = items(t);
    let at: usize = g.choice("command-at", "0").parse().unwrap_or(0);
    let inline = Command::<S>::new("command-inline", g.edit("command"), move |s: &mut S| acc(s).edit_mut("command")).items(&items).highlight(at).on_highlight(move |s: &mut S, i| acc(s).choose("command-at", &i.to_string())).on_select(move |s: &mut S, _| acc(s).presses += 1).build(t);
    let none = Command::<S>::new("command-none", g.edit("command-miss"), move |s: &mut S| acc(s).edit_mut("command-miss")).items(&items).width(360.0).build(t);
    Command::<S>::new("command-demo", g.edit("command"), move |s: &mut S| acc(s).edit_mut("command")).items(&items).highlight(at).on_highlight(move |s: &mut S, i| acc(s).choose("command-at", &i.to_string()))
        .on_select(move |s: &mut S, k: &str| acc(s).choose("command-chose", k))
        .on_alt(move |s: &mut S, k: &str| acc(s).choose("command-chose", &format!("{k} (alternate)")))
        .hint("↩", "Open").hint("⌘ ↩", "Open in pane").hint("esc", "Close")
        .dialog(t, g.is_open("command-demo"), "command-open", move |s: &mut S| acc(s).close("command-demo"), c.layer);
    specs(t, vec![
        spec(t, "Inline", inline),
        spec(t, "Nothing matches", none),
        spec(t, "Trigger (press it: the palette opens over the page)", Button::<S>::new("command-open", "Search commands").variant(ButtonVariant::Outline).icon(&icons::SEARCH).on_press(move |s: &mut S| acc(s).toggle_open("command-demo")).build(t)),
    ])
}

fn search<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let items = items(t);
    let recents = vec!["t2".to_owned(), "settings".to_owned()];
    let at = |k: &str| g.index(k, 0);
    let keep = |key: &'static str, edit: &'static str| Command::<S>::new(key, g.edit(edit), move |s: &mut S| acc(s).edit_mut(edit)).items(&items).width(420.0).highlight(at(key)).on_highlight(move |s: &mut S, i| acc(s).choose(key, &i.to_string()));
    let recent = Command::<S>::new("command-recent", g.edit("command-recents"), move |s: &mut S| acc(s).edit_mut("command-recents")).items(&items).width(420.0).recents(&recents).build(t);
    let fuzzy = keep("command-fuzzy", "command-fuzzy").build(t);
    let loading = keep("command-loading", "command-loading").loading(true).build(t);
    let waiting = keep("command-wait", "command-wait").loading(true).build(t);
    specs(t, vec![
        spec(t, "Empty query: recents first", recent),
        spec(t, "Fuzzy: \"lgn\" finds the login thread; the letters that matched are semibold", fuzzy),
        spec(t, "Loading: the rows stay while slower ones arrive", loading),
        spec(t, "Loading with nothing yet", waiting),
    ])
}
