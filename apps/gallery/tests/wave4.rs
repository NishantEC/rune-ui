//! 0.9's changes in the live path (overlays at the root), driven by real presses, pointer moves and keys: actions that show on hover or
//! focus, the context ring and its breakdown, the tool line, quiet reasoning, the flush fold, muted words, the destructive permission
//! request, a run card's optional facts, sidebar list rows, radio descriptions in a wide column, and disabled ink.

use gallery::{App, apply_scene, set_headless, view};
use rune::{ElementInfo, Env, Key, KeyEvent, Mods, Runtime};

const W: f32 = 1320.0;
const H: f32 = 1800.0;

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

fn drawn(rt: &Runtime<App>, key: &str) -> [f32; 4] {
    rt.presented(key).unwrap_or_else(|| panic!("{key} is not drawn")).0
}

fn press(rt: &mut Runtime<App>, key: &str) {
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("no {key} to press"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

fn key(rt: &mut Runtime<App>, k: Key) {
    rt.key(KeyEvent { key: k, mods: Mods::default() });
    settle(rt);
}

fn space(rt: &mut Runtime<App>) {
    key(rt, Key::Text(" ".to_owned()));
}

fn focused(rt: &Runtime<App>) -> Option<String> {
    rt.elements().into_iter().find(|e| e.focused).and_then(|e| e.key)
}

fn focus(rt: &mut Runtime<App>, k: &str) {
    let k = k.to_owned();
    rt.update(move |_, tx| tx.focus(k, true));
    settle(rt);
}

fn info(rt: &Runtime<App>, key: &str) -> ElementInfo {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("no {key}"))
}

/// Every text drawn inside the element keyed `key`.
fn texts_in(rt: &Runtime<App>, key: &str) -> Vec<String> {
    let els = rt.elements();
    let Some(root) = els.iter().position(|e| e.key.as_deref() == Some(key)) else { return Vec::new() };
    let inside = |mut i: usize| loop {
        if i == root {
            return true;
        }
        match els[i].parent {
            Some(p) => i = p,
            None => return false,
        }
    };
    els.iter().enumerate().filter(|(i, e)| e.text.is_some() && inside(*i)).map(|(_, e)| e.text.clone().unwrap()).collect()
}

/// A node of the accessibility tree by key.
fn ax(v: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
    if v["key"].as_str() == Some(key) {
        return Some(v.clone());
    }
    v["children"].as_array()?.iter().find_map(|c| ax(c, key))
}

// ---- 1. Message: actions on hover or focus ----

#[test]
fn hidden_actions_take_no_room_and_show_on_hover_and_on_tab() {
    let mut rt = item("message");
    // Hidden: the bar's slot is out of the flow with no size, so the article ends where the bubble ends.
    let slot = drawn(&rt, "message-hidden-user-actions-slot");
    assert_eq!((slot[2], slot[3]), (0.0, 0.0), "a hidden bar takes no room");
    let (article, bubble) = (drawn(&rt, "message-hidden-user"), drawn(&rt, "message-hidden-user-bubble"));
    assert!((article[1] + article[3] - (bubble[1] + bubble[3])).abs() < 0.5, "no gap under a person's message: {article:?} {bubble:?}");
    // Shown: the bar sits one block gap under the words.
    let (shown, words) = (drawn(&rt, "message-shown-user-actions"), drawn(&rt, "message-shown-user-bubble"));
    assert!((shown[1] - (words[1] + words[3]) - 4.0).abs() < 0.5, "a shown bar sits 4 points under the bubble: {words:?} {shown:?}");

    // Live: the pointer over the message shows its bar; leaving hides it again.
    let live = drawn(&rt, "message-live-user");
    let below = drawn(&rt, "message-live-bot");
    rt.pointer_move(live[0] + live[2] - 40.0, live[1] + 10.0);
    settle(&mut rt);
    let slot = drawn(&rt, "message-live-user-actions-slot");
    assert!(slot[3] > 20.0, "hovered, the bar shows: {slot:?}");
    assert!(drawn(&rt, "message-live-bot")[1] > below[1] + 20.0, "and takes its room");
    rt.pointer_move(5.0, H - 5.0);
    settle(&mut rt);
    assert_eq!(drawn(&rt, "message-live-user-actions-slot")[3], 0.0, "the pointer gone, it hides");

    // Keyboard: Tab reaches the hidden bar's first button; the bar shows and focus stays on it.
    focus(&mut rt, "message-actions-retry");
    key(&mut rt, Key::Tab);
    assert_eq!(focused(&rt).as_deref(), Some("message-live-user-actions-copy"), "Tab reaches the hidden bar");
    assert!(drawn(&rt, "message-live-user-actions-slot")[3] > 20.0, "and the bar shows while it has focus");
    key(&mut rt, Key::Tab);
    key(&mut rt, Key::Tab);
    assert_eq!(focused(&rt).as_deref(), Some("message-live-bot-actions-copy"), "on to the answer's hidden bar");
    assert_eq!(drawn(&rt, "message-live-user-actions-slot")[3], 0.0, "the first bar hides once focus leaves it");
    assert!(drawn(&rt, "message-live-bot-actions-slot")[3] > 20.0, "the second shows");
}

// ---- 2. Context: the ring and its breakdown ----

#[test]
fn the_context_breakdown_opens_by_press_return_and_space_and_closes_by_escape_and_outside() {
    let mut rt = item("context");
    assert!(texts_in(&rt, "context-demo-trigger").contains(&"43%".to_owned()), "the share beside the ring");
    assert!(texts_in(&rt, "context-bare-trigger").is_empty(), "or the ring alone");
    focus(&mut rt, "context-demo-trigger");
    key(&mut rt, Key::Enter);
    assert!(rt.presented("context-demo-panel").is_some(), "Return opens it");
    let words = texts_in(&rt, "context-demo-panel");
    for w in ["Used", "86k", "Window", "200k", "In", "48.2k", "Out", "6.1k", "Cached", "31k", "Cost", "$0.42", "Prices are estimates."] {
        assert!(words.iter().any(|x| x == w), "the breakdown says {w}: {words:?}");
    }
    key(&mut rt, Key::Escape);
    assert!(rt.presented("context-demo-panel").is_none(), "Escape closes it");
    assert_eq!(focused(&rt).as_deref(), Some("context-demo-trigger"), "and focus is back on the trigger");
    space(&mut rt);
    assert!(rt.presented("context-demo-panel").is_some(), "Space opens it");
    let panel = drawn(&rt, "context-demo-panel");
    rt.pointer_down(panel[0] - 200.0, panel[1] + panel[3] + 300.0);
    rt.pointer_up(panel[0] - 200.0, panel[1] + panel[3] + 300.0);
    settle(&mut rt);
    assert!(rt.presented("context-demo-panel").is_none(), "a press outside closes it");
    press(&mut rt, "context-free-trigger");
    let words = texts_in(&rt, "context-free-panel");
    assert!(!words.iter().any(|x| x == "Cost"), "no cost, no cost row: {words:?}");
    assert!(words.iter().any(|x| x == "Runs on this computer: no cost."), "the note still shows");
    let ring = info(&rt, "context-demo-trigger");
    assert_eq!(ring.role, "meter");
}

// ---- 3. Tool: a line that reads like a sentence ----

#[test]
fn a_tool_line_takes_a_label_a_note_and_cuts_long_words_in_the_middle() {
    let mut rt = item("tool");
    let quick = texts_in(&rt, "tool-quick-trigger");
    assert_eq!(quick, vec!["Read src/main.rs".to_owned()], "under a second: no time, only the words");
    let line = texts_in(&rt, "tool-command-trigger");
    assert!(line.iter().any(|w| w == "exit 1") && line.iter().any(|w| w == "Failed"), "the note, then the word: {line:?}");
    let words = rt.elements().into_iter().find(|e| e.text.as_deref().is_some_and(|w| w.starts_with("cargo test --workspace")) && e.rect[1] < drawn(&rt, "tool-command-trigger")[1] + 40.0 && e.rect[1] >= drawn(&rt, "tool-command-trigger")[1]).expect("the command on the line");
    assert!(words.text_cut, "a long command is cut");
    assert_eq!(info(&rt, "tool-command-trigger").label.as_deref(), Some(super_long()), "its name is the whole line");
    press(&mut rt, "tool-command-trigger");
    assert!(rt.presented("tool-command-content").is_some(), "it folds open");
    assert!(texts_in(&rt, "tool-command-content").iter().any(|w| w == "test importer::retry::gives_up_after_three_attempts ... FAILED"), "to its error");
    assert!(texts_in(&rt, "tool-said-content").iter().any(|w| w.starts_with("Ran the importer's tests")), "free words in the fold");
    let denied = texts_in(&rt, "tool-denied-trigger");
    assert!(denied.iter().any(|w| w == "Denied"), "denied says so: {denied:?}");
    let t = rune_ui_themes::theme(false);
    let reds = rt.elements().into_iter().filter(|e| e.text_color == Some(t.c.danger.fg)).filter(|e| { let r = drawn(&rt, "tool-denied-trigger"); e.rect[1] >= r[1] && e.rect[1] < r[1] + r[3] }).count();
    assert_eq!(reds, 0, "and is neutral, not red");
}

fn super_long() -> &'static str {
    "cargo test --workspace --all-features -- --test-threads 1 importer::retry::gives_up_after_three_attempts"
}

// ---- 4. Reasoning: nothing to show means nothing shown ----

#[test]
fn a_quick_thought_has_no_figure_and_an_empty_one_takes_no_room() {
    let rt = item("reasoning");
    assert_eq!(info(&rt, "reasoning-quick-trigger").label.as_deref(), Some("Thought"), "under a second: Thought");
    let empty = drawn(&rt, "reasoning-empty");
    assert_eq!((empty[2], empty[3]), (0.0, 0.0), "a finished thought with no words is nothing");
    assert!(rt.presented("reasoning-empty-trigger").is_none(), "no line");
    let col = drawn(&rt, "reasoning-between");
    let lines: Vec<[f32; 4]> = rt.elements().into_iter().filter(|e| matches!(e.text.as_deref(), Some("Above" | "Below")) && e.rect[0] >= col[0] && e.rect[1] >= col[1]).map(|e| e.rect).collect();
    assert_eq!(lines.len(), 2);
    assert!((lines[1][1] - (lines[0][1] + lines[0][3]) - 4.0).abs() < 0.5, "Below sits one gap under Above: {lines:?}");
    assert_eq!(info(&rt, "reasoning-live-trigger").label.as_deref(), Some("Thinking…"), "the live state stays");
}

// ---- 5. FoldLine: a flush body ----

#[test]
fn a_fold_can_set_its_body_on_its_own_left_edge() {
    let rt = item("chain_of_thought");
    let (flush, line) = (drawn(&rt, "cot-flush-steps"), drawn(&rt, "cot-flush"));
    assert!((flush[0] - line[0]).abs() < 0.5, "not indented: {flush:?} {line:?}");
    let (indented, line) = (drawn(&rt, "cot-working-steps"), drawn(&rt, "cot-working"));
    assert!(indented[0] > line[0] + 20.0, "the default still indents: {indented:?} {line:?}");
}

// ---- 6. Response: muted words ----

#[test]
fn muted_words_draw_in_the_second_ink() {
    let rt = item("response");
    let t = rune_ui_themes::theme(false);
    let r = drawn(&rt, "response-muted");
    let inks: Vec<_> = rt.elements().into_iter().filter(|e| e.text.as_deref().is_some_and(|w| w.contains("read the importer"))).filter(|e| e.rect[1] >= r[1]).filter_map(|e| e.text_color).collect();
    assert!(!inks.is_empty() && inks.iter().all(|c| *c == t.ink_muted()), "muted: {inks:?}");
    let whole = drawn(&rt, "response-whole");
    let ink: Vec<_> = rt.elements().into_iter().filter(|e| e.text.as_deref().is_some_and(|w| w.starts_with("The importer now")) && e.rect[1] >= whole[1] && e.rect[1] < whole[1] + whole[3]).filter_map(|e| e.text_color).collect();
    assert!(ink.iter().all(|c| *c == t.ink()), "an answer stays in the first ink");
}

// ---- 7. Confirmation: why and destructive ----

#[test]
fn a_destructive_request_has_no_always_allow_and_return_answers_nothing() {
    let mut rt = item("confirmation");
    assert!(rt.presented("confirm-danger-always").is_none(), "no Always allow");
    assert!(texts_in(&rt, "confirm-danger").iter().any(|w| w == "Asked because commands can change anything on this Mac."), "the why line");
    focus(&mut rt, "confirm-danger-allow");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("confirm-danger", ""), "", "Return gives no answer");
    key(&mut rt, Key::Escape);
    assert_eq!(rt.state.g.choice("confirm-danger", ""), "deny", "Escape denies");
    let mut rt = item("confirmation");
    focus(&mut rt, "confirm-danger-allow");
    space(&mut rt);
    assert_eq!(rt.state.g.choice("confirm-danger", ""), "once", "Space on the focused answer gives it");
    let mut rt = item("confirmation");
    press(&mut rt, "confirm-danger-allow");
    assert_eq!(rt.state.g.choice("confirm-danger", ""), "once", "and so does a press");
}

// ---- 8. Task: only the facts given ----

#[test]
fn a_run_card_shows_only_the_facts_it_was_given() {
    let rt = item("task");
    let running = texts_in(&rt, "task-running");
    assert!(!running.iter().any(|w| w.contains("passed")), "no checks, no \"0 passed\": {running:?}");
    let tokens = texts_in(&rt, "task-tokens");
    for w in ["18.4k tokens", "14s", "asha/build-notes"] {
        assert!(tokens.iter().any(|x| x == w), "{w} in {tokens:?}");
    }
    assert!(!tokens.iter().any(|w| w.contains("file")), "no files when none were given");
    assert!(texts_in(&rt, "task-noted").iter().any(|w| w.starts_with("Waiting on a review")), "the note line");
    assert!(rt.presented("task-bare-facts").is_none() && rt.presented("task-bare-note").is_none(), "a bare card has no facts row");
}

// ---- 9. Sidebar: a lead mark and list rows ----

#[test]
fn sidebar_rows_lead_with_a_mark_and_read_as_a_list() {
    let mut rt = item("sidebar");
    let tree = rt.accessibility_json();
    let row = ax(&tree, "sidebar-t2").expect("the row in the tree");
    assert_eq!((row["role"].as_str(), row["position"].as_u64()), (Some("list-box-option"), Some(2)), "an option, second: {row}");
    assert_eq!(ax(&tree, "sidebar-tasks-list-0").expect("the list in the tree")["setSize"].as_u64(), Some(4), "of four");
    assert!(row["states"].as_array().is_some_and(|s| s.iter().any(|x| x == "selected")), "selected: {row}");
    let list = rt.elements().into_iter().find(|e| e.role == "listbox").expect("a listbox");
    assert_eq!(list.label.as_deref(), Some("Tasks from today"));
    // The lead mark sits on the icon's box: the same left edge as the icon in the row under it.
    let lead_x = rt.elements().into_iter().filter(|e| e.role == "progressbar").map(|e| e.rect[0]).find(|x| *x > drawn(&rt, "sidebar-t1")[0]).expect("a spinner leads t1");
    let r4 = drawn(&rt, "sidebar-t4");
    let icon_x = rt.elements().into_iter().filter(|e| e.role == "icon" && e.rect[0] >= r4[0] && e.rect[0] < r4[0] + r4[2] && e.rect[1] >= r4[1] && e.rect[1] < r4[1] + r4[3]).map(|e| e.rect[0]).next().expect("t4's icon");
    assert!((lead_x - icon_x).abs() < 0.5, "lead {lead_x} on the icon's box {icon_x}");
    focus(&mut rt, "sidebar-t3");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("sidebar-task", ""), "t3", "Return opens a row");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("sidebar-t4"), "Down moves within the list");
}

// ---- 10. RadioGroup: descriptions as wide as the group ----

#[test]
fn radio_descriptions_wrap_to_the_groups_width() {
    let rt = item("radio_group");
    let group = drawn(&rt, "radio-wide");
    let line = drawn(&rt, "radio-wide-0");
    assert!(line[2] >= group[2] - 0.5, "the line is as wide as the group: {line:?} {group:?}");
    let help = rt.elements().into_iter().find(|e| e.text.as_deref().is_some_and(|w| w.starts_with("Before a command runs"))).expect("the description");
    assert!(help.rect[2] > 400.0, "it wraps to the group, not at 320 points: {:?}", help.rect);
}

// ---- 12. Disabled ink keeps its material ----

#[test]
fn a_disabled_ink_button_is_the_same_object_dimmed_not_a_faded_disc() {
    let rt = item("button");
    for k in ["button-default-disabled", "button-destructive-disabled"] {
        let b = info(&rt, k);
        assert_eq!(b.opacity, 1.0, "{k} keeps its own opacity (its body and words dim instead)");
        assert!(!b.focusable, "{k} is off");
    }
    assert!(info(&rt, "button-secondary-disabled").opacity < 1.0, "other materials still fade");
}
