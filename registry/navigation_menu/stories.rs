use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::navigation_menu::{NavLink, NavigationMenu};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("navigation_menu", "NavigationMenu", "overlays").story("Panels and links", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let open = g.open.as_deref().and_then(|o| o.strip_prefix("nav-demo-")).and_then(|i| i.parse().ok());
    let go = move |s: &mut S| acc(s).presses += 1;
    let learn = vec![
        NavLink::new("Introduction", "Reusable parts styled by tokens, copied into your app.").on_press(go),
        NavLink::new("Installation", "Add the style crate and the command line tool.").on_press(go),
        NavLink::new("Typography", "Headings, reading text and figures in the theme's font.").on_press(go),
    ];
    let parts = vec![
        NavLink::new("Calendar", "A month of days with ranges and disabled days.").on_press(go),
        NavLink::new("Date picker", "A date in a field, chosen from a calendar.").on_press(go),
        NavLink::new("Data table", "Sorting, selection, columns and pages.").on_press(go),
        NavLink::new("Chart", "Line, bar, area and donut in theme colours.").on_press(go),
        NavLink::new("Resizable", "Panes split by a handle you drag or step.").on_press(go),
        NavLink::new("Carousel", "Slides with arrows, dots and keys.").on_press(go),
    ];
    let nav = NavigationMenu::<S>::new("nav-demo", "Main").panel("Getting started", learn).panel("Components", parts).link("Docs", go).open(open).on_open(move |s: &mut S, i: Option<usize>| acc(s).open = i.map(|i| format!("nav-demo-{i}"))).build(t, c.layer);
    specs(t, vec![spec(t, "Two panels (one column, two columns) and a link", nav)])
}
