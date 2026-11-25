//! The agent set in the live window path (the page in a scroll view, overlays at the root), driven by real presses, pointer moves, the
//! wheel and keys: folds, the composer, permission requests, the conversation's pinning, the model selector, copying and the queue.

use std::collections::HashMap;

use gallery::{App, apply_scene, set_headless, view};
use rune::{Env, Key, KeyEvent, Mods, Runtime};

const W: f32 = 1320.0;
const H: f32 = 900.0;

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..90 {
        rt.frame(1.0 / 60.0);
    }
}

fn item(name: &str) -> Runtime<App> {
    set_headless(false);
    let mut s = App::default();
    apply_scene(&mut s, &format!("item-{name}"));
    let mut rt = Runtime::new(s, view, (W, H), Env { system_dark: false, scale: 1.0 });
    settle(&mut rt);
    rt
}

fn layout(rt: &Runtime<App>) -> HashMap<String, [f32; 4]> {
    rt.elements().into_iter().map(|e| (e.path, e.rect)).collect()
}

fn unmoved(before: &HashMap<String, [f32; 4]>, rt: &Runtime<App>, what: &str) {
    for (path, a) in layout(rt) {
        if let Some(b) = before.get(&path) {
            assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01), "{what} moved {path}: {b:?} -> {a:?}");
        }
    }
}

fn drawn(rt: &Runtime<App>, key: &str) -> [f32; 4] {
    rt.presented(key).unwrap_or_else(|| panic!("{key} is not drawn")).0
}

fn press(rt: &mut Runtime<App>, key: &str) {
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("no {key} to press"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

fn key_with(rt: &mut Runtime<App>, k: Key, mods: Mods) {
    rt.key(KeyEvent { key: k, mods });
    settle(rt);
}

fn key(rt: &mut Runtime<App>, k: Key) {
    key_with(rt, k, Mods::default());
}

fn typed(rt: &mut Runtime<App>, words: &str) {
    for ch in words.chars() {
        key(rt, Key::Text(ch.to_string()));
    }
}

fn focused(rt: &Runtime<App>) -> Option<String> {
    rt.elements().into_iter().find(|e| e.focused).and_then(|e| e.key)
}

fn focus(rt: &mut Runtime<App>, k: &str) {
    let k = k.to_owned();
    rt.update(move |_, tx| tx.focus(k, true));
    settle(rt);
}

fn info(rt: &Runtime<App>, key: &str) -> rune::ElementInfo {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("no {key}"))
}

// ---- Folds ----

#[test]
fn folds_open_and_close_by_press_and_by_key_and_keep_focus() {
    for (item_name, fold) in [("tool", "tool-done"), ("reasoning", "reasoning-done"), ("chain_of_thought", "cot-done"), ("sources", "sources-demo")] {
        let mut rt = item(item_name);
        let trigger = format!("{fold}-trigger");
        let content = format!("{fold}-content");
        assert!(rt.presented(&content).is_none(), "{fold} starts folded");
        assert_eq!(info(&rt, &trigger).expanded, Some(false), "{fold} says it is folded");
        press(&mut rt, &trigger);
        assert!(rt.presented(&content).is_some(), "a press opens {fold}");
        assert_eq!(info(&rt, &trigger).expanded, Some(true));
        assert_eq!(focused(&rt).as_deref(), Some(trigger.as_str()), "{fold} keeps focus on its line");
        key(&mut rt, Key::Enter);
        assert!(rt.presented(&content).is_none(), "Return folds {fold}");
        assert_eq!(focused(&rt).as_deref(), Some(trigger.as_str()), "and focus stays");
        key(&mut rt, Key::Text(" ".to_owned()));
        assert!(rt.presented(&content).is_some(), "Space opens {fold}");
    }
    let mut rt = item("diff");
    assert!(rt.presented("diff-0-lines").is_some() && rt.presented("diff-1-lines").is_none(), "the first file starts open");
    press(&mut rt, "diff-1-trigger");
    assert!(rt.presented("diff-1-lines").is_some(), "a file's header opens it");
    key(&mut rt, Key::Enter);
    assert!(rt.presented("diff-1-lines").is_none(), "and Return folds it");
}

// ---- PromptInput ----

#[test]
fn the_composer_sends_on_return_breaks_lines_on_shift_return_and_stops_on_escape() {
    let mut rt = item("prompt_input");
    focus(&mut rt, "prompt-demo-input");
    key(&mut rt, Key::Enter);
    assert!(!rt.state.g.choices.contains_key("prompt-sent"), "an empty composer sends nothing");
    assert!(!info(&rt, "prompt-demo-submit").focusable, "and its Send is off");
    typed(&mut rt, "Fix");
    key_with(&mut rt, Key::Enter, Mods { shift: true, ..Mods::default() });
    typed(&mut rt, "it");
    assert_eq!(rt.state.g.edit("prompt").text, "Fix\nit", "Shift+Return makes a new line");
    assert!(!rt.state.g.choices.contains_key("prompt-sent"), "and does not send");
    assert!(info(&rt, "prompt-demo-submit").focusable, "words turn Send on");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("prompt-sent", ""), "Fix\nit", "Return sends");
    assert!(rt.state.g.flag("prompt-streaming"), "the agent answers");
    assert_eq!(focused(&rt).as_deref(), Some("prompt-demo-input"), "the field keeps focus after sending");
    let stop = info(&rt, "prompt-demo-submit");
    assert_eq!(stop.label.as_deref(), Some("Stop"), "Send became Stop");
    assert!(stop.focusable);
    typed(&mut rt, "more");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("prompt-sent", ""), "Fix\nit", "Return sends nothing while streaming");
    assert_eq!(focused(&rt).as_deref(), Some("prompt-demo-input"), "the field keeps focus; typed {:?}", rt.state.g.edit("prompt").text);
    key(&mut rt, Key::Escape);
    assert!(!rt.state.g.flag("prompt-streaming"), "Escape stops");
    assert_eq!(info(&rt, "prompt-demo-submit").label.as_deref(), Some("Send"));
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("prompt-sent", ""), "more", "ready again, Return sends");
    press(&mut rt, "prompt-demo-submit");
    assert!(!rt.state.g.flag("prompt-streaming"), "pressing Stop stops");
    focus(&mut rt, "prompt-off-input");
    typed(&mut rt, "x");
    assert_eq!(rt.state.g.edit("prompt-off").text, "", "a disabled composer takes no typing");
}

// ---- Confirmation ----

#[test]
fn a_permission_request_is_answered_once_by_press() {
    let mut rt = item("confirmation");
    press(&mut rt, "confirm-demo-allow");
    assert_eq!((rt.state.g.choice("confirm", ""), rt.state.g.presses), ("once".to_owned(), 1), "Allow once");
    assert!(rt.presented("confirm-demo-allow").is_none() && rt.presented("confirm-demo-deny").is_none(), "the answers are gone");
    let line = info(&rt, "confirm-demo");
    assert_eq!((line.role, line.label.as_deref()), ("status", Some("Run a shell command: Allowed once")), "it says what happened");
    focus(&mut rt, "confirm-demo");
    key(&mut rt, Key::Escape);
    let r = drawn(&rt, "confirm-demo");
    rt.pointer_down(r[0] + 60.0, r[1] + r[3] / 2.0);
    rt.pointer_up(r[0] + 60.0, r[1] + r[3] / 2.0);
    settle(&mut rt);
    assert_eq!((rt.state.g.choice("confirm", ""), rt.state.g.presses), ("once".to_owned(), 1), "a resolved request cannot be answered again");
}

#[test]
fn a_permission_request_is_answered_by_keys() {
    let mut rt = item("confirmation");
    focus(&mut rt, "confirm-demo-allow");
    key(&mut rt, Key::Tab);
    assert_eq!(focused(&rt).as_deref(), Some("confirm-demo-always"), "Tab moves to the next answer");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("confirm", ""), "always", "Return gives it");
    let mut rt = item("confirmation");
    focus(&mut rt, "confirm-demo-allow");
    key(&mut rt, Key::Escape);
    assert_eq!(rt.state.g.choice("confirm", ""), "deny", "Escape denies");
    assert_eq!(rt.state.g.presses, 1);
    key(&mut rt, Key::Escape);
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.presses, 1, "and nothing answers it twice");
}

// ---- Conversation ----

fn bottom(r: [f32; 4]) -> f32 {
    r[1] + r[3]
}

#[test]
fn a_conversation_stays_pinned_releases_on_scroll_up_and_jumps_back() {
    let mut rt = item("conversation");
    let view = drawn(&rt, "conversation-demo-scroll");
    let last = drawn(&rt, "conversation-m5");
    assert!(bottom(last) <= bottom(view) + 0.5 && bottom(last) > bottom(view) - 60.0, "opens at the newest message: {last:?} in {view:?}");
    assert!(rt.presented("conversation-demo-jump").is_none(), "no Jump at the bottom");
    press(&mut rt, "conversation-add");
    let added = drawn(&rt, "conversation-m6");
    assert!(bottom(added) <= bottom(view) + 0.5 && bottom(added) > bottom(view) - 60.0, "a new message keeps the view pinned: {added:?} in {view:?}");
    rt.wheel(view[0] + 100.0, view[1] + 100.0, 400.0);
    settle(&mut rt);
    assert!(rt.state.g.flag("conversation-away"), "scrolling up lets go");
    assert!(rt.presented("conversation-demo-jump").is_some(), "and shows Jump to latest");
    let away = drawn(&rt, "conversation-m6");
    assert!(away[1] > bottom(view), "the newest message is out of view: {away:?}");
    press(&mut rt, "conversation-add");
    let m7 = rt.presented("conversation-m7").map(|p| p.0);
    assert!(m7.is_none_or(|r| r[1] > bottom(view)), "a message arriving while away does not pull the view down");
    press(&mut rt, "conversation-demo-jump");
    assert!(!rt.state.g.flag("conversation-away"), "Jump returns to the bottom");
    assert!(rt.presented("conversation-demo-jump").is_none(), "and goes");
    let back = drawn(&rt, "conversation-m7");
    assert!(bottom(back) <= bottom(view) + 0.5 && bottom(back) > bottom(view) - 60.0, "the newest message is in view: {back:?}");
}

// ---- ModelSelector ----

#[test]
fn the_model_selector_opens_over_the_page_at_its_trigger_and_chooses_by_key() {
    let mut rt = item("model_selector");
    let before = layout(&rt);
    press(&mut rt, "model-demo-trigger");
    let (t, p) = (drawn(&rt, "model-demo-trigger"), drawn(&rt, "model-demo-panel"));
    assert!((p[0] - t[0]).abs() < 0.5 && p[1] >= t[1] + t[3], "under the trigger at its left edge: {p:?} vs {t:?}");
    unmoved(&before, &rt, "opening the model selector");
    assert_eq!(focused(&rt).as_deref(), Some("model-demo-search"), "the search takes focus");
    typed(&mut rt, "small");
    assert!(rt.presented("model-demo-a-large").is_none() && rt.presented("model-demo-a-small").is_some(), "typing filters");
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("model", ""), "b-small", "Down then Return chooses the second match");
    assert!(rt.presented("model-demo-panel").is_none(), "and closes");
    assert_eq!(focused(&rt).as_deref(), Some("model-demo-trigger"), "focus is back on the trigger");
    press(&mut rt, "model-demo-trigger");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("model-demo-panel").is_none(), "Escape closes");
    press(&mut rt, "model-demo-trigger");
    rt.pointer_down(W - 20.0, H - 20.0);
    rt.pointer_up(W - 20.0, H - 20.0);
    settle(&mut rt);
    assert!(rt.presented("model-demo-panel").is_none(), "a press outside closes");
}

// ---- CodeBlock, Actions, Branch, Queue, Suggestion, Context ----

#[test]
fn copy_buttons_fire_their_events() {
    let mut rt = item("code_block");
    press(&mut rt, "code-demo-copy");
    assert!(rt.state.g.flag("code-copied"), "CodeBlock's copy runs its handler");
    assert_eq!(info(&rt, "code-demo-copy").label.as_deref(), Some("Copied"), "and shows it");
    let mut rt = item("actions");
    press(&mut rt, "actions-demo-copy");
    press(&mut rt, "actions-demo-good");
    assert!(rt.state.g.flag("actions-copied") && rt.state.g.choice("actions-rating", "") == "good");
    press(&mut rt, "actions-demo-good");
    assert_eq!(rt.state.g.choice("actions-rating", "x"), "", "pressing it again clears it");
}

#[test]
fn a_branch_steps_by_press_and_arrow_and_stops_at_its_ends() {
    let mut rt = item("branch");
    press(&mut rt, "branch-demo-next");
    assert_eq!(rt.state.g.index("branch", 9), 2);
    assert!(!info(&rt, "branch-demo-next").focusable, "Next is off at the last answer");
    focus(&mut rt, "branch-demo-prev");
    key(&mut rt, Key::Left);
    assert_eq!(rt.state.g.index("branch", 9), 1, "Left steps back");
}

#[test]
fn the_queue_moves_and_removes_by_keys_and_focus_follows() {
    let mut rt = item("queue");
    focus(&mut rt, "queue-demo-item-0");
    key_with(&mut rt, Key::Down, Mods { alt: true, ..Mods::default() });
    assert_eq!(rt.state.g.choice("queue", ""), "Then open a pull request\nAlso run the linter\nTag Lee for review", "Alt+Down moves it down");
    assert_eq!(focused(&rt).as_deref(), Some("queue-demo-item-1"), "focus moves with it");
    key(&mut rt, Key::Delete);
    assert_eq!(rt.state.g.choice("queue", ""), "Then open a pull request\nTag Lee for review", "Delete removes it");
    press(&mut rt, "queue-demo-item-0-remove");
    assert_eq!(rt.state.g.choice("queue", ""), "Tag Lee for review", "and so does its button");
}

#[test]
fn suggestions_and_the_context_popover_respond_to_presses() {
    let mut rt = item("suggestion");
    press(&mut rt, "suggestion-demo-2");
    assert_eq!(rt.state.g.choice("suggestion", ""), "Why is the build slow?");
    let mut rt = item("context");
    let before = layout(&rt);
    press(&mut rt, "context-demo-trigger");
    assert!(rt.presented("context-demo-panel").is_some(), "the breakdown opens");
    unmoved(&before, &rt, "opening the context popover");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("context-demo-panel").is_none(), "Escape closes it");
    assert_eq!(focused(&rt).as_deref(), Some("context-demo-trigger"));
}

#[test]
fn the_chat_block_sends_and_stops() {
    let mut rt = item("chat");
    focus(&mut rt, "chat-composer-input");
    typed(&mut rt, "Thanks");
    key(&mut rt, Key::Enter);
    assert!(rt.presented("chat-sent-0").is_some() && rt.presented("chat-live-thinking-trigger").is_some(), "the message joins the conversation and the agent thinks");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("chat-live-thinking-trigger").is_none(), "Escape stops it");
    press(&mut rt, "chat-permission-deny");
    assert!(rt.presented("chat-permission-deny").is_none(), "the permission is answered");
}
