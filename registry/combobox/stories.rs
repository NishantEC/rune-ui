use super::super::combobox::{Combobox, ComboboxOption};
use super::super::gallery::{Ctx, StoryItem, spec, specs};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("combobox", "Combobox", "form").story("Search and choose", all::<S>)
}

fn options() -> &'static [ComboboxOption] {
    static OPTIONS: std::sync::OnceLock<Vec<ComboboxOption>> = std::sync::OnceLock::new();
    OPTIONS.get_or_init(|| ["placeholder-app", "placeholder-docs", "placeholder-site", "placeholder-tools", "sample-service", "sample-worker"].iter().map(|n| ComboboxOption::new(*n, *n)).collect())
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let current = g.choice("combobox", "placeholder-docs");
    let combo = Combobox::<S>::new("combobox-demo", "Repository", options())
        .current(Some(&current))
        .query(g.edit("combobox-query"), move |s: &mut S| acc(s).edit_mut("combobox-query"))
        .highlight(g.index("combobox-hl", 0))
        .open(g.is_open("combobox-demo"))
        .on_toggle(move |s: &mut S| acc(s).toggle_open("combobox-demo"))
        .on_close(move |s: &mut S| acc(s).close("combobox-demo"))
        .on_highlight(move |s: &mut S, i| acc(s).choose("combobox-hl", &i.to_string()))
        .on_choose(move |s: &mut S, id: &str| acc(s).choose("combobox", id))
        .build(t, c.layer);
    specs(t, vec![spec(t, "Press it: a search over the options floats under it", combo)])
}
