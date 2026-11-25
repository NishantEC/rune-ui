use rune::El;

use super::super::attachment::Attachment;
use super::super::gallery::{Ctx, GalleryState, StoryItem, spec_wide, specs};
use super::super::icons;
use super::super::model_selector::{Model, ModelSelector};
use super::super::prompt_input::{MenuState, Phase, PromptInput, Suggestion};

pub fn item<S: 'static>() -> StoryItem<S> {
    StoryItem::new("prompt_input", "PromptInput", "agent").story("Ready, streaming and disabled", all::<S>).story("Mentions with @, commands with /", menus::<S>)
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
    specs(t, vec![
        spec_wide(t, "Ready: Return sends, Shift+Return is a new line", 560.0, live),
        spec_wide(t, "Streaming: Stop, or Escape", 560.0, streaming),
        spec_wide(t, "Disabled", 560.0, off),
    ])
}

/// Placeholder people and files for `@`, and commands for `/`.
pub fn mentions() -> Vec<Suggestion> {
    vec![
        Suggestion::new("person:asha", "Asha").detail("Design").icon(&icons::USER),
        Suggestion::new("person:lee", "Lee").detail("Billing").icon(&icons::USER),
        Suggestion::new("file:src/main.rs", "main.rs").detail("src").icon(&icons::FILE_CODE),
        Suggestion::new("file:docs/notes.md", "notes.md").detail("docs").icon(&icons::FILE_TEXT),
    ]
}

pub fn commands() -> Vec<Suggestion> {
    vec![
        Suggestion::new("clear", "clear").detail("Start a new chat"),
        Suggestion::new("model", "model").detail("Choose the model"),
        Suggestion::new("review", "review").detail("Review the open diff"),
    ]
}

fn menu_state(g: &mut GalleryState) -> &mut MenuState {
    g.slot_mut::<MenuState>("prompt-menu-state")
}

fn menus<S: 'static>(c: &mut Ctx<S>) -> El {
    let (t, g, acc) = (c.t, c.g, c.access);
    let menu: MenuState = g.slot("prompt-menu-state");
    let composer = PromptInput::<S>::new("prompt-menu", g.edit("prompt-menu"), move |s: &mut S| acc(s).edit_mut("prompt-menu"))
        .placeholder("Type @ to mention, / for commands")
        .rows(1, 4)
        .menu(&menu, move |s: &mut S| menu_state(acc(s)))
        .mentions(mentions())
        .commands(commands(), move |s: &mut S, id: &str| acc(s).choose("prompt-command", id))
        .on_submit(move |s: &mut S| {
            let g = acc(s);
            let words = g.edit("prompt-menu").text.clone();
            g.choose("prompt-menu-sent", &words);
        })
        .build_in(t, c.layer);
    specs(t, vec![spec_wide(t, "Type @ for people and files, / at the start for commands; four lines at most, then it scrolls", 560.0, composer)])
}
