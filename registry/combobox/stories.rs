use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::combobox::{Combobox, ComboboxOption};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("combobox", "Combobox", "form").story("Search and choose", all::<S>)
}

fn options() -> &'static [ComboboxOption] {
    static OPTIONS: std::sync::OnceLock<Vec<ComboboxOption>> = std::sync::OnceLock::new();
    OPTIONS.get_or_init(|| ["placeholder-app", "placeholder-docs", "placeholder-site", "placeholder-tools", "sample-service"].iter().map(|n| ComboboxOption::new(*n, *n)).collect())
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let blank = rune_kit::ComboboxState::default();
    let st = g.combos.get("combobox-demo").unwrap_or(&blank);
    let current = g.choice("combobox", "placeholder-docs");
    let combo = Combobox::<S>::new("combobox-demo", "Repository", st, options(), move |s: &mut S| acc(s).combos.entry("combobox-demo".to_owned()).or_default()).current(Some(&current)).on_choose(move |s: &mut S, id: &str| acc(s).choose("combobox", id)).build(t, c.layer);
    specs(t, vec![spec(t, "Rest (press it: the list floats over the page)", combo)])
}
