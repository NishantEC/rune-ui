use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::input::Input;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("input", "Input", "form").story("States", states::<S>).story("Inside the well", inside::<S>)
}

fn states<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let i = |k: &str| Input::<S>::new(format!("input-{k}"), g.edit("name"), move |s: &mut S| acc(s).edit_mut("name")).name("Project name").width(220.0);
    specs(t, vec![
        spec(t, "Rest", i("rest").build(t)),
        spec(t, "Focus", i("focus").state(Look::FocusVisible).build(t)),
        spec(t, "Placeholder", Input::<S>::new("input-empty", g.edit("empty"), move |s: &mut S| acc(s).edit_mut("empty")).placeholder("Project name").width(220.0).build(t)),
        spec(t, "Invalid", i("invalid").invalid(true).build(t)),
        spec(t, "Disabled", i("disabled").disabled(true).build(t)),
    ])
}

fn inside<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    specs(t, vec![
        spec(t, "Prefix and suffix", Input::<S>::new("input-budget", g.edit("budget"), move |s: &mut S| acc(s).edit_mut("budget")).name("Budget").prefix("$").suffix("per day").width(220.0).build(t)),
        spec(t, "Clearable", Input::<S>::new("input-clear", g.edit("name"), move |s: &mut S| acc(s).edit_mut("name")).clearable().width(220.0).build(t)),
        spec(t, "Search, empty", Input::<S>::search("input-search", g.edit("empty"), move |s: &mut S| acc(s).edit_mut("empty")).placeholder("Find a setting").suffix("⌘F").width(220.0).build(t)),
        spec(t, "Search, with text", Input::<S>::search("input-search-full", g.edit("query"), move |s: &mut S| acc(s).edit_mut("query")).width(220.0).build(t)),
        spec(t, "Password", Input::<S>::new("input-secret", g.edit("secret"), move |s: &mut S| acc(s).edit_mut("secret")).name("Password").secure().width(180.0).build(t)),
    ])
}
