//! 0.11's additions in the live path (overlays at the root), driven by real presses, pointer moves and keys: tabs (keys, closable,
//! overflow, the pane look), the split, toasts with an action, the palette, the two-line list row and the small asks.

use gallery::ui;
use gallery::{App, apply_scene, set_headless, view};
use rune::{Cx, El, ElementInfo, Env, Key, KeyEvent, Mods, Runtime, col, row};
use rune_ui_style::{Theme, contrast};

const W: f32 = 1320.0;
const H: f32 = 1800.0;

fn settle<S: 'static>(rt: &mut Runtime<S>) {
    for _ in 0..90 {
        rt.frame(1.0 / 60.0);
    }
}

fn item(name: &str) -> Runtime<App> {
    item_in(name, false)
}

fn item_in(name: &str, dark: bool) -> Runtime<App> {
    set_headless(false);
    let mut s = App::default();
    apply_scene(&mut s, &format!("item-{name}"));
    let mut rt = Runtime::new(s, view, (W, H), Env { system_dark: dark, scale: 1.0 });
    settle(&mut rt);
    rt
}

fn scene(name: &str, dark: bool) -> Runtime<App> {
    set_headless(false);
    let mut s = App::default();
    apply_scene(&mut s, name);
    let mut rt = Runtime::new(s, view, (W, H), Env { system_dark: dark, scale: 1.0 });
    settle(&mut rt);
    rt
}

fn drawn<S: 'static>(rt: &Runtime<S>, key: &str) -> [f32; 4] {
    rt.presented(key).unwrap_or_else(|| panic!("{key} is not drawn")).0
}

fn is_drawn<S: 'static>(rt: &Runtime<S>, key: &str) -> bool {
    rt.presented(key).is_some()
}

fn press<S: 'static>(rt: &mut Runtime<S>, key: &str) {
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("no {key} to press"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

fn key<S: 'static>(rt: &mut Runtime<S>, k: Key) {
    rt.key(KeyEvent { key: k, mods: Mods::default() });
    settle(rt);
}

fn key_with<S: 'static>(rt: &mut Runtime<S>, k: Key, mods: Mods) {
    rt.key(KeyEvent { key: k, mods });
    settle(rt);
}

fn cmd() -> Mods {
    Mods { cmd: true, logo: true, ..Mods::default() }
}

fn focused<S: 'static>(rt: &Runtime<S>) -> Option<String> {
    rt.elements().into_iter().find(|e| e.focused).and_then(|e| e.key)
}

fn focus<S: 'static>(rt: &mut Runtime<S>, k: &str) {
    let k = k.to_owned();
    rt.update(move |_, tx| tx.focus(k, true));
    settle(rt);
}

fn info<S: 'static>(rt: &Runtime<S>, key: &str) -> ElementInfo {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("no {key}"))
}

fn has<S: 'static>(rt: &Runtime<S>, key: &str) -> bool {
    rt.elements().into_iter().any(|e| e.key.as_deref() == Some(key) && !e.exiting)
}

/// A node of the accessibility tree by key.
fn ax(v: &serde_json::Value, key: &str) -> Option<serde_json::Value> {
    if v["key"].as_str() == Some(key) {
        return Some(v.clone());
    }
    v["children"].as_array()?.iter().find_map(|c| ax(c, key))
}

// ---- A probe: a few components in a theme chosen by the test ----

#[derive(Default)]
struct Probe {
    dark: bool,
    still: bool,
}

fn theme_of(s: &Probe) -> Theme {
    Theme::from_json(rune_ui_themes::DEFAULT, s.dark).unwrap().with_reduced_motion(s.still)
}

fn probe(view: fn(&Probe, &mut Cx<Probe>) -> El, dark: bool, still: bool) -> Runtime<Probe> {
    let mut rt = Runtime::new(Probe { dark, still }, view, (W, 900.0), Env { system_dark: dark, scale: 1.0 });
    settle(&mut rt);
    rt
}

// ---- Tabs ----

fn closable(rt: &Runtime<App>) -> Vec<String> {
    rt.state.g.choice("tabs-open", "Overview,Changes,Subagent 1,Subagent 2,Subagent 3,Subagent 4,Subagent 5").split(',').map(str::to_owned).collect()
}

#[test]
fn tabs_have_their_roles_say_which_is_chosen_and_label_the_panel_by_it() {
    let rt = item("tabs");
    let tree = rt.accessibility_json();
    assert_eq!(ax(&tree, "tabs-line").unwrap()["role"].as_str(), Some("tab-list"));
    let t0 = ax(&tree, "tabs-line-0").unwrap().to_string();
    let t1 = ax(&tree, "tabs-line-1").unwrap().to_string();
    assert!(t0.contains("\"selected\"") && !t1.contains("\"selected\""), "only the first tab is selected: {t0} / {t1}");
    let panel = ax(&tree, "tabs-line-panel").unwrap();
    assert_eq!(panel["role"].as_str(), Some("tab-panel"));
    assert!(panel.to_string().contains("Overview"), "the panel is named by the chosen tab: {panel}");
    // Only the chosen tab is a Tab stop (roving); the others are reached with the arrows.
    assert!(info(&rt, "tabs-line-0").focusable && info(&rt, "tabs-line-1").focusable);
}

#[test]
fn tab_arrows_wrap_home_and_end_go_to_the_ends_and_move_focus_with_the_choice() {
    let mut rt = item("tabs");
    focus(&mut rt, "tabs-line-0");
    key(&mut rt, Key::Right);
    assert_eq!(rt.state.g.index("tabs", 0), 1, "Right chooses the next tab (automatic activation)");
    assert_eq!(focused(&rt).as_deref(), Some("tabs-line-1"), "and focus goes with it");
    key(&mut rt, Key::End);
    assert_eq!((rt.state.g.index("tabs", 0), focused(&rt).as_deref()), (2, Some("tabs-line-2")));
    key(&mut rt, Key::Right);
    assert_eq!(rt.state.g.index("tabs", 0), 0, "Right wraps");
    key(&mut rt, Key::Left);
    assert_eq!(rt.state.g.index("tabs", 0), 2, "Left wraps");
    key(&mut rt, Key::Home);
    assert_eq!(rt.state.g.index("tabs", 0), 0, "Home");
}

#[test]
fn manual_activation_moves_focus_and_return_or_space_chooses() {
    let mut rt = item("tabs");
    focus(&mut rt, "tabs-manual-0");
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("tabs-manual-1"), "focus moved");
    assert_eq!(rt.state.g.index("tabs-manual", 0), 0, "but nothing was chosen");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.index("tabs-manual", 0), 1, "Return chooses");
    key(&mut rt, Key::Right);
    key(&mut rt, Key::Text(" ".into()));
    assert_eq!(rt.state.g.index("tabs-manual", 0), 2, "Space chooses");
}

#[test]
fn a_closable_tab_has_a_named_close_button_cmd_w_closes_it_and_the_first_cannot_close() {
    let mut rt = item("tabs");
    assert!(!has(&rt, "tabs-pane-0-close"), "a permanent tab has no close button");
    assert_eq!(info(&rt, "tabs-pane-1-close").label.as_deref(), Some("Close Changes"));
    assert_eq!(info(&rt, "tabs-pane-1-close").role, "button");
    let before = closable(&rt);
    // The close button is out of the Tab order: it is reached by the pointer, and Cmd+W closes from the keys.
    press(&mut rt, "tabs-pane-1");
    assert_eq!(rt.state.g.index("tabs-closable", 0), 1);
    press(&mut rt, "tabs-pane-1-close");
    assert_eq!(closable(&rt).len(), before.len() - 1, "pressing the close button closes");
    assert!(!closable(&rt).contains(&"Changes".to_owned()));
    focus(&mut rt, "tabs-pane-1");
    key_with(&mut rt, Key::Text("w".into()), cmd());
    assert_eq!(closable(&rt).len(), before.len() - 2, "Cmd+W closes the focused tab");
    focus(&mut rt, "tabs-pane-0");
    key_with(&mut rt, Key::Text("w".into()), cmd());
    assert_eq!(closable(&rt).len(), before.len() - 2, "Cmd+W leaves a permanent tab");
}

#[test]
fn tabs_that_do_not_fit_fold_into_a_more_menu_that_chooses_and_the_chosen_tab_stays() {
    let mut rt = item("tabs");
    let strip = drawn(&rt, "tabs-overflow");
    assert!(has(&rt, "tabs-overflow-more"), "a More button");
    assert!(!has(&rt, "tabs-overflow-6"), "the last tab is folded away");
    let more = drawn(&rt, "tabs-overflow-more");
    assert!(more[0] + more[2] <= strip[0] + strip[2] + 0.5, "the button is inside the strip");
    for i in 0..7 {
        if has(&rt, &format!("tabs-overflow-{i}")) {
            let r = drawn(&rt, &format!("tabs-overflow-{i}"));
            assert!(r[0] + r[2] <= more[0] + 0.5, "tab {i} ends before the More button");
        }
    }
    press(&mut rt, "tabs-overflow-more");
    assert!(has(&rt, "tabs-overflow-more-6"), "the menu lists the hidden tabs");
    press(&mut rt, "tabs-overflow-more-6");
    assert_eq!(rt.state.g.index("tabs-closable", 0), 6, "choosing a row chooses the tab");
    assert!(has(&rt, "tabs-overflow-6"), "the chosen tab now stays in the strip");
    let r = drawn(&rt, "tabs-overflow-6");
    let strip = drawn(&rt, "tabs-overflow");
    assert!(r[0] + r[2] <= strip[0] + strip[2] + 0.5);
}

#[test]
fn tabs_fitting_keeps_the_chosen_and_leaves_room_for_more() {
    use ui::tabs::fitting;
    let w = [60.0, 60.0, 60.0, 60.0];
    assert_eq!(fitting(&w, 0.0, 4.0, 30.0, 0), vec![0, 1, 2, 3], "not measured: all");
    assert_eq!(fitting(&w, 300.0, 4.0, 30.0, 0), vec![0, 1, 2, 3], "all fit");
    assert_eq!(fitting(&w, 200.0, 4.0, 30.0, 0), vec![0, 1]);
    assert_eq!(fitting(&w, 200.0, 4.0, 30.0, 3), vec![0, 3], "the chosen tab takes the last place");
}

#[test]
fn the_pane_look_is_one_control_height_with_a_two_point_line_under_the_chosen_tab() {
    let rt = item("tabs");
    let t = rune_ui_themes::theme(false);
    let tab = drawn(&rt, "tabs-pane-0");
    assert!((tab[3] - (t.control.sm + 2.0 + t.space.at(0.5))).abs() < 0.6, "compact: control height plus the line: {tab:?}");
    let line = drawn(&rt, "tabs-pane");
    assert!(line[3] <= t.control.sm + 6.0, "the strip is as tall as one row and its line");
}
