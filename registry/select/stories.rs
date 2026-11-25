use rune_ui_style::State as Look;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::select::Select;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("select", "Select", "form").story("States", all::<S>)
}

fn levels() -> &'static [String] {
    static LEVELS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    LEVELS.get_or_init(|| ["Careful", "Balanced", "Hands-off"].map(str::to_owned).to_vec())
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    let level = g.choice("select", "Balanced");
    let mk = |k: &str| {
        let (a, b) = (k.to_owned(), k.to_owned());
        Select::<S>::new(k.to_owned(), "Starting level", levels()).current(&level).width(180.0).open(g.is_open(k)).on_toggle(move |s: &mut S| acc(s).toggle_open(&a)).on_close(move |s: &mut S| acc(s).close(&b)).on_choose(move |s: &mut S, v: &str| {
            let s = acc(s);
            s.choose("select", v);
            s.open = None;
        })
    };
    let items = vec![
        spec(t, "Rest", mk("select-demo").build(t, c.layer)),
        spec(t, "Focus", mk("select-focus").state(Look::FocusVisible).build(t, c.layer)),
        spec(t, "Placeholder", Select::<S>::new("select-empty", "Model", levels()).placeholder("Choose a level").width(180.0).build(t, c.layer)),
        spec(t, "Invalid", mk("select-invalid").invalid(true).build(t, c.layer)),
        spec(t, "Disabled", mk("select-disabled").disabled(true).build(t, c.layer)),
    ];
    rune::col().child(specs(t, items)).child(rune::el().h(if g.open.is_some() { 120.0 } else { 0.0 }))
}
