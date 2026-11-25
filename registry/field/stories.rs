use super::super::gallery::{Ctx, StoryItem, specs};
use super::super::field::Field;
use super::super::input::Input;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("field", "Field", "form").story("Label, control, words", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let acc = c.access;
    let (t, g) = (c.t, c.g);
    specs(t, vec![
        Field::new("field-budget", "Budget").description("Stops work that spends more").width(240.0).child(Input::<S>::new("field-budget-input", g.edit("budget"), move |s: &mut S| acc(s).edit_mut("budget")).name("Budget").description("Stops work that spends more").prefix("$").build(t)).build(t),
        Field::new("field-name", "Name").required().error(Some("A name is needed".to_owned())).width(240.0).child(Input::<S>::new("field-name-input", g.edit("empty"), move |s: &mut S| acc(s).edit_mut("empty")).name("Name").placeholder("Project name").invalid(true).build(t)).build(t),
        Field::new("field-off", "Region").disabled(true).description("Set by your administrator").width(240.0).child(Input::<S>::new("field-off-input", g.edit("name"), move |s: &mut S| acc(s).edit_mut("name")).name("Region").disabled(true).build(t)).build(t),
    ])
}
