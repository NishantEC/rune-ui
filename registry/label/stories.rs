use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::label::Label;

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("label", "Label", "form").story("Forms", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> rune::El {
    let t = c.t;
    specs(t, vec![spec(t, "Plain", Label::new("label-plain", "Project name").build(t)), spec(t, "Required", Label::new("label-required", "Email").required().build(t)), spec(t, "Disabled", Label::new("label-disabled", "Region").disabled(true).build(t))])
}
