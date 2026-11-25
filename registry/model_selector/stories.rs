use rune::El;

use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::model_selector::{Model, ModelSelector};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("model_selector", "ModelSelector", "agent").story("Press to choose; type to search", all::<S>)
}

/// Placeholder models: two providers, three tiers each.
pub fn models() -> Vec<Model> {
    vec![
        Model::new("a-large", "Large", "Provider A", "$15 / 1M"),
        Model::new("a-medium", "Medium", "Provider A", "$3 / 1M"),
        Model::new("a-small", "Small", "Provider A", "Fast"),
        Model::new("b-large", "Large", "Provider B", "$12 / 1M"),
        Model::new("b-medium", "Medium", "Provider B", "$2.50 / 1M"),
        Model::new("b-small", "Small", "Provider B", "Fast"),
    ]
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let ms = models();
    let chosen = g.choice("model", "a-medium");
    let sel = ModelSelector::<S>::new("model-demo", &ms, &chosen, g.edit("model-query"), move |s: &mut S| acc(s).edit_mut("model-query"))
        .open(g.is_open("model-demo")).highlight(g.index("model-at", 0))
        .on_open(move |s: &mut S, o| {
            let g = acc(s);
            g.open = o.then(|| "model-demo".to_owned());
        })
        .on_highlight(move |s: &mut S, i| acc(s).choose("model-at", &i.to_string()))
        .on_select(move |s: &mut S, k: &str| acc(s).choose("model", k))
        .build(t, c.layer);
    specs(t, vec![spec(t, "The panel floats from the overlay layer, anchored to the trigger", sel)])
}
