//! rune-ui registry: chat 0.6.0 (a block). Copied into your app by `rune-ui add chat`; yours to change.
//!
//! A whole agent screen made of the agent set, with placeholder content, so it can be seen put together and taken apart: a
//! Conversation holding a person's message, the agent's reasoning, tool calls, a run of steps, its answer with an action bar, a
//! permission request, and the PromptInput under it with a ModelSelector in its toolbar. The app keeps a [`ChatState`] and hands the
//! screen a way to reach it; everything the screen does lands there. Replace the placeholder turns with your own.
//!
//! ```ignore
//! struct App { chat: ui::chat::ChatState }
//! let mut layer = Vec::new();
//! let screen = ui::chat::screen(&t, &s.chat, |s: &mut App| &mut s.chat, &mut layer);
//! ui::base::overlay_root(screen, layer)
//! ```

use std::collections::HashSet;

use rune::{El, TextEdit, col};

use super::actions::Actions;
use super::agent::Status;
use super::attachment::Attachment;
use super::base::Theme;
use super::chain_of_thought::{ChainOfThought, Step};
use super::confirmation::{Confirmation, Decision};
use super::conversation::Conversation;
use super::message::{Message, words};
use super::model_selector::{Model, ModelSelector};
use super::prompt_input::{Phase, PromptInput};
use super::reasoning::Reasoning;
use super::response::Response;
use super::tool::{Tool, ToolCall};

/// Everything the screen shows and changes.
pub struct ChatState {
    pub draft: TextEdit,
    pub phase: Phase,
    /// Folds that are open (by key).
    pub open: HashSet<String>,
    pub decision: Decision,
    pub model: String,
    pub picking: bool,
    pub query: TextEdit,
    pub highlight: usize,
    pub at_bottom: bool,
    pub copied: bool,
    pub rating: Option<bool>,
    /// Messages the person sent in this session (placeholder: shown after the canned turns).
    pub sent: Vec<String>,
    pub attachments: Vec<String>,
}

impl Default for ChatState {
    fn default() -> Self {
        ChatState { draft: TextEdit::new(""), phase: Phase::Ready, open: HashSet::new(), decision: Decision::Pending, model: "b-medium".to_owned(), picking: false, query: TextEdit::new(""), highlight: 0, at_bottom: true, copied: false, rating: None, sent: Vec::new(), attachments: vec!["build-log.txt".to_owned()] }
    }
}

impl ChatState {
    pub fn is_open(&self, k: &str) -> bool {
        self.open.contains(k)
    }

    pub fn flip(&mut self, k: &str) {
        if !self.open.remove(k) {
            self.open.insert(k.to_owned());
        }
    }

    /// Sends the draft: it joins the conversation and the agent starts answering.
    pub fn send(&mut self) {
        let words = self.draft.text.trim().to_owned();
        if words.is_empty() {
            return;
        }
        self.sent.push(words);
        self.draft.clear();
        self.attachments.clear();
        self.phase = Phase::Streaming;
    }
}

/// Placeholder models: two providers, three tiers.
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

const ANSWER: &str = "The build spends most of its time linking. Two changes help:\n\n1. Turn on the faster linker in `.cargo/config.toml`.\n2. Split the `ui` crate so a change to one screen rebuilds less.\n\n```toml\n[target.aarch64-apple-darwin]\nrustflags = [\"-C\", \"link-arg=-fuse-ld=lld\"]\n```\n\nWith both, a one-line change rebuilds in about **9 seconds** instead of 41.";

fn read_call() -> ToolCall {
    ToolCall { name: "read_file".to_owned(), summary: "Cargo.toml".to_owned(), status: Status::Done, secs: Some(0.2), input: "{ \"path\": \"Cargo.toml\" }".to_owned(), output: Some("[workspace]\nmembers = [\"app\", \"ui\"]\n...".to_owned()), error: None }
}

fn timing_call() -> ToolCall {
    ToolCall { name: "run".to_owned(), summary: "cargo build --timings".to_owned(), status: Status::Done, secs: Some(41.3), input: "cargo build --timings".to_owned(), output: Some("Compiling ui v0.1.0\nCompiling app v0.1.0\nFinished dev in 41.3s\nlink: 29.8s".to_owned()), error: None }
}

fn steps() -> Vec<Step> {
    vec![
        Step::new("Read the workspace manifest", Status::Done),
        Step::new("Timed a clean build", Status::Done).detail("41.3 s, 29.8 s of it linking"),
        Step::new("Compared two linkers", Status::Done),
        Step::new("Wrote the suggested config", Status::Done),
    ]
}

/// The screen: the conversation filling the space above the composer. `layer` takes the model selector's panel.
/// `access` reaches the screen's state inside the app's (`|s: &mut App| &mut s.chat`).
pub fn screen<S: 'static, F>(t: &Theme, c: &ChatState, access: F, layer: &mut Vec<El>) -> El
where
    F: for<'a> Fn(&'a mut S) -> &'a mut ChatState + Copy + 'static,
{
    let (read, timing, steps, models) = (read_call(), timing_call(), steps(), models());
    let mut turns = vec![
        Message::user("chat-m1", "Asha").child(words(t, "Why does a one-line change take 40 seconds to build?")).build(t),
        Message::assistant("chat-m2", "Assistant")
            .child(Reasoning::<S>::new("chat-thinking", false, 12.0, c.is_open("chat-thinking")).text("The question is about incremental builds. Look at the workspace layout first, then time a build to see which phase dominates.").on_toggle(move |s: &mut S| access(s).flip("chat-thinking")).build(t))
            .child(Tool::<S>::new("chat-read", &read, c.is_open("chat-read")).on_toggle(move |s: &mut S| access(s).flip("chat-read")).build(t))
            .child(Tool::<S>::new("chat-time", &timing, c.is_open("chat-time")).on_toggle(move |s: &mut S| access(s).flip("chat-time")).build(t))
            .child(ChainOfThought::<S>::new("chat-steps", &steps, 134.0, c.is_open("chat-steps")).on_toggle(move |s: &mut S| access(s).flip("chat-steps")).build(t))
            .child(Response::new("chat-answer", ANSWER).build(t))
            .actions(Actions::<S>::new("chat-actions").copy(ANSWER, c.copied, move |s: &mut S| access(s).copied = true).retry(|_: &mut S| {}).feedback(c.rating, move |s: &mut S, r| access(s).rating = r).build(t))
            .build(t),
        Message::user("chat-m3", "Asha").child(words(t, "Apply the linker change.")).build(t),
        Message::assistant("chat-m4", "Assistant")
            .child(Confirmation::<S>::new("chat-permission", "Edit .cargo/config.toml", c.decision).detail("[target.aarch64-apple-darwin]\nrustflags = [\"-C\", \"link-arg=-fuse-ld=lld\"]").on_decide(move |s: &mut S, d| access(s).decision = d).build(t))
            .build(t),
    ];
    for (i, w) in c.sent.iter().enumerate() {
        turns.push(Message::user(format!("chat-sent-{i}"), "Asha").child(words(t, w.clone())).build(t));
    }
    if c.phase == Phase::Streaming {
        turns.push(Message::assistant("chat-live", "Assistant").child(Reasoning::<S>::new("chat-live-thinking", true, 3.0, false).build(t)).build(t));
    }
    let conversation = Conversation::<S>::new("chat-conversation", c.at_bottom).children(turns).on_at_bottom(move |s: &mut S, at| access(s).at_bottom = at).build(t);
    let selector = ModelSelector::<S>::new("chat-model", &models, &c.model, &c.query, move |s: &mut S| &mut access(s).query)
        .open(c.picking).highlight(c.highlight)
        .on_open(move |s: &mut S, o| access(s).picking = o)
        .on_highlight(move |s: &mut S, i| access(s).highlight = i)
        .on_select(move |s: &mut S, k: &str| access(s).model = k.to_owned())
        .build(t, layer);
    let mut composer = PromptInput::<S>::new("chat-composer", &c.draft, move |s: &mut S| &mut access(s).draft).phase(c.phase).tool(selector)
        .on_submit(move |s: &mut S| access(s).send())
        .on_stop(move |s: &mut S| access(s).phase = Phase::Ready)
        .on_attach(move |s: &mut S| access(s).attachments.push("notes.md".to_owned()));
    for (i, a) in c.attachments.iter().enumerate() {
        composer = composer.attachment(Attachment::<S>::new(format!("chat-file-{i}"), a.clone()).detail("12 KB").on_remove(move |s: &mut S| {
            let c = access(s);
            if i < c.attachments.len() {
                c.attachments.remove(i);
            }
        }).build(t));
    }
    let dock = col().w_full().max_w(720.0 + 2.0 * t.metrics.inset).px(t.metrics.inset).pb(t.metrics.inset).child(composer.build(t));
    col().key("chat").w_full().h_full().items_center().fill(t.c.page.bg).child(col().w_full().grow(1.0).min_h(0.0).child(conversation)).child(dock)
}
