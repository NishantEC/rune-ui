use rune::El;

use super::super::attachment::Attachment;
use super::super::gallery::{Ctx, StoryItem, spec, specs};
use super::super::model_selector::{Model, ModelSelector};
use super::super::prompt_input::{Phase, PromptInput};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("prompt_input", "PromptInput", "agent").story("Ready, streaming and disabled", all::<S>)
}

fn all<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let phase = if g.flag("prompt-streaming") { Phase::Streaming } else { Phase::Ready };
    let models = vec![Model::new("a-medium", "Medium", "Provider A", "$3 / 1M"), Model::new("a-small", "Small", "Provider A", "Fast")];
    let chosen = g.choice("prompt-model", "a-medium");
    let selector = ModelSelector::<S>::new("prompt-model", &models, &chosen, g.edit("prompt-model-query"), move |s: &mut S| acc(s).edit_mut("prompt-model-query"))
        .open(g.is_open("prompt-model")).on_open(move |s: &mut S, o| acc(s).open = o.then(|| "prompt-model".to_owned()))
        .on_select(move |s: &mut S, k: &str| acc(s).choose("prompt-model", k)).build(t, c.layer);
    let live = PromptInput::<S>::new("prompt-demo", g.edit("prompt"), move |s: &mut S| acc(s).edit_mut("prompt")).phase(phase)
        .attachment(Attachment::<S>::new("prompt-file", "build-log.txt").detail("12 KB").on_remove(|_: &mut S| {}).build(t))
        .tool(selector)
        .on_attach(|_: &mut S| {})
        .on_submit(move |s: &mut S| {
            let g = acc(s);
            let words = g.edit("prompt").text.clone();
            g.choose("prompt-sent", &words);
            g.edit_mut("prompt").clear();
            g.flags.insert("prompt-streaming".to_owned(), true);
        })
        .on_stop(move |s: &mut S| {
            acc(s).flags.insert("prompt-streaming".to_owned(), false);
        })
        .build(t);
    let streaming = PromptInput::<S>::new("prompt-busy", g.edit("prompt-busy"), move |s: &mut S| acc(s).edit_mut("prompt-busy")).phase(Phase::Streaming).on_stop(|_: &mut S| {}).build(t);
    let off = PromptInput::<S>::new("prompt-off", g.edit("prompt-off"), move |s: &mut S| acc(s).edit_mut("prompt-off")).placeholder("Connect a model to start").phase(Phase::Disabled).on_attach(|_: &mut S| {}).build(t);
    let w = |e: El| rune::col().w(560.0).child(e);
    specs(t, vec![
        spec(t, "Ready: Return sends, Shift+Return is a new line", w(live)),
        spec(t, "Streaming: Stop, or Escape", w(streaming)),
        spec(t, "Disabled", w(off)),
    ])
}
