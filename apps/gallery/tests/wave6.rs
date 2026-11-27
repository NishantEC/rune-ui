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

// ---- Split ----

fn val(rt: &Runtime<App>, k: &str, d: f32) -> f32 {
    rt.state.g.value(k, d)
}

/// Presses on a handle's middle and drags it `dx` points by pointer, slowly, then lets go.
fn drag_x(rt: &mut Runtime<App>, key: &str, dx: f32) {
    let r = drawn(rt, key);
    let (x, y) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
    rt.pointer_down(x, y);
    for i in 1..=10 {
        rt.pointer_move(x + dx * i as f32 / 10.0, y);
        rt.frame(1.0 / 60.0);
    }
    rt.pointer_up(x + dx, y);
    settle(rt);
}

#[test]
fn split_handles_drag_inside_their_least_and_most_and_the_regions_follow() {
    let mut rt = item("split");
    assert_eq!(drawn(&rt, "split-demo-left")[2], 120.0, "the sidebar starts at its width");
    drag_x(&mut rt, "split-demo-handle-left", 30.0);
    assert!((val(&rt, "split-side", 0.0) - 150.0).abs() < 0.5, "dragged 30 points: {}", val(&rt, "split-side", 0.0));
    assert!((drawn(&rt, "split-demo-left")[2] - 150.0).abs() < 0.5, "and the region follows");
    // The right region grows to the left.
    drag_x(&mut rt, "split-demo-handle-right", -20.0);
    assert!((val(&rt, "split-pane", 0.0) - 150.0).abs() < 0.5, "a right region grows when its handle moves left");
    // The sidebar stops at its most (170), or sooner where the centre would lose its 120 points: 440 - 151 - 120 - 1 = 168.
    drag_x(&mut rt, "split-demo-handle-left", 200.0);
    assert!((val(&rt, "split-side", 0.0) - 168.0).abs() < 0.5, "kept off the centre's room: {}", val(&rt, "split-side", 0.0));
    // It never takes the centre's room: 440 wide, the sidebar 170, so the pane stops where 120 is left for the centre.
    drag_x(&mut rt, "split-demo-handle-right", -300.0);
    let (left, centre, right) = (drawn(&rt, "split-demo-left"), drawn(&rt, "split-demo-center"), drawn(&rt, "split-demo-right"));
    assert!(centre[2] >= 119.5, "the centre keeps its least: {centre:?}");
    assert!((left[2] + centre[2] + right[2] + 2.0 - drawn(&rt, "split-demo")[2]).abs() < 1.0, "the three fill the split: {left:?} {centre:?} {right:?}");
}

#[test]
fn a_split_handle_is_a_named_separator_with_its_width_and_moves_by_keys() {
    let mut rt = item("split");
    let tree = rt.accessibility_json();
    let h = ax(&tree, "split-demo-handle-left").unwrap();
    assert_eq!(h["role"].as_str(), Some("splitter"), "{h}");
    assert!(h.to_string().contains("Resize sidebar") && h.to_string().contains("120"), "named, with its width as its value: {h}");
    focus(&mut rt, "split-demo-handle-left");
    key(&mut rt, Key::Right);
    assert_eq!(val(&rt, "split-side", 0.0), 130.0, "Right: one step");
    key_with(&mut rt, Key::Left, Mods { shift: true, ..Mods::default() });
    assert_eq!(val(&rt, "split-side", 0.0), 90.0, "Shift: four steps");
    key(&mut rt, Key::End);
    assert_eq!(val(&rt, "split-side", 0.0), 170.0, "End: the most");
    key(&mut rt, Key::Home);
    assert_eq!(val(&rt, "split-side", 0.0), 80.0, "Home: the least");
    focus(&mut rt, "split-demo-handle-right");
    key(&mut rt, Key::Left);
    assert_eq!(val(&rt, "split-pane", 0.0), 140.0, "a right region grows on Left");
}

#[test]
fn a_double_click_resets_a_split_region_and_a_collapsible_one_collapses_and_opens_again() {
    let mut rt = item("split");
    drag_x(&mut rt, "split-demo-handle-left", 40.0);
    assert_eq!(val(&rt, "split-side", 0.0), 160.0);
    let r = drawn(&rt, "split-demo-handle-left");
    let (x, y) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(&mut rt);
    assert_eq!(val(&rt, "split-side", 0.0), 120.0, "double-click: back to the default");
    // Collapse: drag past the least (80) by more than the snap.
    drag_x(&mut rt, "split-demo-handle-left", -200.0);
    assert!(rt.state.g.flag("split-side-hidden"), "collapsed");
    assert!(!has(&rt, "split-demo-left"), "the region is gone");
    assert!(has(&rt, "split-demo-handle-left"), "its handle stays at the edge");
    assert!(ax(&rt.accessibility_json(), "split-demo-handle-left").unwrap().to_string().contains("Collapsed"));
    // Open it again by dragging from the edge.
    drag_x(&mut rt, "split-demo-handle-left", 100.0);
    assert!(!rt.state.g.flag("split-side-hidden"), "opened");
    assert!(has(&rt, "split-demo-left") && drawn(&rt, "split-demo-left")[2] >= 80.0);
    // Return on the handle collapses and opens.
    focus(&mut rt, "split-demo-handle-left");
    key(&mut rt, Key::Enter);
    assert!(rt.state.g.flag("split-side-hidden"), "Return collapses");
    key(&mut rt, Key::Enter);
    assert!(!rt.state.g.flag("split-side-hidden"), "Return opens");
}

#[test]
fn split_clamped_applies_least_most_fraction_and_the_centres_room() {
    use ui::split::clamped;
    assert_eq!(clamped(50.0, 100.0, 300.0, 1.0, 0.0, 0.0, 0.0), 100.0);
    assert_eq!(clamped(500.0, 100.0, 300.0, 1.0, 0.0, 0.0, 0.0), 300.0);
    assert_eq!(clamped(500.0, 100.0, 600.0, 0.5, 800.0, 0.0, 0.0), 400.0, "half the window at most");
    assert_eq!(clamped(500.0, 100.0, 600.0, 1.0, 800.0, 256.0, 440.0), 103.0, "the centre keeps 440 beside a 256 sidebar");
    assert_eq!(clamped(50.0, 100.0, 600.0, 1.0, 300.0, 256.0, 440.0), 100.0, "a window too small for it: the least wins");
}

// ---- Toast ----

fn toast_poll(s: &mut App, clock: f64, tx: &mut rune::Tx) -> bool {
    s.g.slot_mut::<ui::toast::ToastHost>("toast-host").poll(clock, tx)
}

fn toasts() -> Runtime<App> {
    let mut rt = item("toast");
    rt.poll = Some(toast_poll);
    rt
}

fn host(rt: &Runtime<App>) -> ui::toast::ToastHost {
    rt.state.g.slot("toast-host")
}

/// Runs the clock `secs` forward in frames.
fn pass(rt: &mut Runtime<App>, secs: f64) {
    for _ in 0..(secs * 60.0) as usize {
        rt.frame(1.0 / 60.0);
    }
}

#[test]
fn toast_host_times_pauses_and_carries_on_from_where_it_was() {
    use ui::toast::{ToastHost, ToastSpec};
    let mut h = ToastHost::new();
    let a = h.push(ToastSpec::new("One").timeout(4.0));
    let b = h.push(ToastSpec::new("Two").sticky());
    let c = h.push(ToastSpec::new("Three").action("Undo", "undo"));
    assert!(!h.tick(10.0), "the timers start at the first tick");
    assert_eq!(h.next_due(), Some(14.0));
    h.set_hover(true, 12.0);
    assert!(h.paused() && h.next_due().is_none(), "paused: nothing is due");
    assert!(!h.tick(100.0), "and nothing runs out while paused");
    h.set_hover(false, 100.0);
    assert_eq!(h.next_due(), Some(102.0), "two seconds were left");
    assert!(h.tick(102.5));
    let left: Vec<u64> = h.items().map(|(id, _)| id).collect();
    assert_eq!(left, vec![b, c], "the first went; a sticky one stays; an action gives 8 seconds: {a}");
    assert!(!h.tick(105.5) && h.tick(106.5), "the one with an action had 8 seconds, 2 gone before the pause: 6 left from 100");
    assert_eq!(h.items().map(|(id, _)| id).collect::<Vec<_>>(), vec![b]);
    // Focus pauses too, and both must let go.
    h.set_focus(true, 200.0);
    h.set_hover(true, 200.0);
    h.set_hover(false, 201.0);
    assert!(h.paused(), "focus still holds it");
    assert_eq!(h.focus_target("t"), Some("t-2-close".to_owned()));
}

#[test]
fn a_toast_shows_at_the_bottom_trailing_edge_is_announced_politely_and_goes_by_itself() {
    let mut rt = toasts();
    press(&mut rt, "toast-settle");
    assert!(has(&rt, "toasts-1"), "the toast shows");
    let r = drawn(&rt, "toasts-1");
    let t = rune_ui_themes::theme(false);
    assert!((r[0] + r[2] - (W - t.space.at(4.0))).abs() < 1.5, "its right edge is a margin from the window's: {r:?}");
    assert!((r[1] + r[3] - (H - t.space.at(4.0))).abs() < 1.5, "its bottom too: {r:?}");
    let node = ax(&rt.accessibility_json(), "toasts-1").unwrap();
    assert!(node.to_string().contains("polite"), "a polite live region: {node}");
    assert_eq!(node["role"].as_str(), Some("status"));
    pass(&mut rt, 10.0);
    assert!(!has(&rt, "toasts-1"), "gone after its 8 seconds");
    assert!(host(&rt).is_empty());
}

#[test]
fn a_toast_stops_while_the_pointer_or_focus_is_on_it_and_goes_when_both_leave() {
    let mut rt = toasts();
    press(&mut rt, "toast-settle");
    let r = drawn(&rt, "toasts-1");
    rt.pointer_move(r[0] + 20.0, r[1] + 10.0);
    pass(&mut rt, 20.0);
    assert!(has(&rt, "toasts-1"), "still there after 20 seconds under the pointer");
    assert!(host(&rt).paused());
    rt.pointer_move(40.0, 40.0);
    pass(&mut rt, 3.0);
    assert!(has(&rt, "toasts-1"), "the clock carried on from where it was");
    pass(&mut rt, 7.0);
    assert!(!has(&rt, "toasts-1"), "and ran out");
    // Focus inside does the same.
    press(&mut rt, "toast-settle");
    focus(&mut rt, "toasts-2-action");
    pass(&mut rt, 20.0);
    assert!(has(&rt, "toasts-2"), "focus holds it");
}

#[test]
fn a_toasts_action_dismisses_it_and_says_its_tag_and_escape_dismisses_a_focused_one() {
    let mut rt = toasts();
    press(&mut rt, "toast-settle");
    assert_eq!(info(&rt, "toasts-1-action").label.as_deref(), Some("Undo"));
    press(&mut rt, "toasts-1-action");
    assert_eq!(rt.state.g.choice("toast-said", ""), "undo-settle", "the app heard the tag");
    assert!(!has(&rt, "toasts-1"), "and the toast went");
    press(&mut rt, "toast-settle");
    let target = host(&rt).focus_target("toasts").unwrap();
    focus(&mut rt, &target);
    assert_eq!(focused(&rt).as_deref(), Some(target.as_str()), "the shortcut's target takes focus");
    key(&mut rt, Key::Escape);
    assert!(!has(&rt, "toasts-2"), "Escape dismisses it");
}

#[test]
fn toasts_stack_newest_last_centre_on_request_and_show_the_newest_four() {
    let mut rt = toasts();
    for _ in 0..5 {
        press(&mut rt, "toast-settle");
    }
    assert!(!has(&rt, "toasts-1") && has(&rt, "toasts-2") && has(&rt, "toasts-5"), "the newest four show");
    assert!(drawn(&rt, "toasts-5")[1] > drawn(&rt, "toasts-2")[1], "newest last (lowest)");
    press(&mut rt, "toast-place");
    let r = drawn(&rt, "toasts-5");
    assert!((r[0] + r[2] / 2.0 - W / 2.0).abs() < 1.5, "centred along the bottom: {r:?}");
}

struct ToastProbe {
    still: bool,
    host: ui::toast::ToastHost,
}

fn toast_probe_view(s: &ToastProbe, _: &mut Cx<ToastProbe>) -> El {
    let t = Theme::from_json(rune_ui_themes::DEFAULT, false).unwrap().with_reduced_motion(s.still);
    let mut layer = Vec::new();
    ui::toast::Toasts::<ToastProbe>::new("toasts", &s.host, |s: &mut ToastProbe| &mut s.host).mount(&t, &mut layer);
    ui::base::overlay_root(col().w_full().h_full(), layer)
}

#[test]
fn a_toast_arrives_without_motion_under_reduce_motion() {
    for still in [true, false] {
        let mut host = ui::toast::ToastHost::new();
        host.push(ui::toast::ToastSpec::new("Saved"));
        let mut rt = Runtime::new(ToastProbe { still, host }, toast_probe_view, (W, 900.0), Env { system_dark: false, scale: 1.0 });
        rt.frame(0.0);
        let first = rt.presented("toasts-1").map(|p| p.0);
        settle(&mut rt);
        let last = drawn(&rt, "toasts-1");
        if still {
            assert_eq!(first, Some(last), "no motion under Reduce Motion");
        }
    }
}

// ---- Command ----

fn titles(rt: &Runtime<App>, key: &str) -> Vec<String> {
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

#[test]
fn fuzzy_matching_finds_letters_in_order_ranks_word_starts_and_runs_and_says_where() {
    use ui::command::fuzzy;
    let (score, at) = fuzzy("lgn", "Fix the login redirect").expect("lgn is in login");
    assert_eq!(at.len(), 3);
    assert!(at.windows(2).all(|w| w[0] < w[1]), "in order: {at:?}");
    assert!(fuzzy("nlg", "login").is_none(), "out of order does not match");
    assert!(fuzzy("xyz", "login").is_none());
    assert_eq!(fuzzy("", "anything"), Some((0, vec![])));
    let (word_starts, _) = fuzzy("fr", "Fix redirect").unwrap();
    let (scattered, _) = fuzzy("fr", "Defrost").unwrap();
    assert!(word_starts > scattered, "word starts rank above letters inside a word: {word_starts} vs {scattered}");
    let (run, _) = fuzzy("log", "login").unwrap();
    let (apart, _) = fuzzy("log", "lxoxg").unwrap();
    assert!(run > apart, "a touching run ranks above a spread one");
    assert!(score > 0);
    assert_eq!(fuzzy("É", "école").map(|(_, p)| p), Some(vec![0]), "case and accents of one letter fold the same way as the letter");
}

#[test]
fn search_keeps_sections_in_the_order_they_appear_ranks_inside_them_and_words_mode_is_the_old_rule() {
    use ui::command::{CommandItem, Match, search};
    let items = vec![
        CommandItem::new("a", "Open settings", "Actions"),
        CommandItem::new("b", "Settings", "Pages"),
        CommandItem::new("c", "Reset the setup", "Actions").keywords("settings"),
        CommandItem::new("d", "Show calendar", "Pages"),
    ];
    let hits = search(&items, "set", Match::Fuzzy);
    let keys: Vec<&str> = hits.iter().map(|h| h.item.key.as_str()).collect();
    assert_eq!(keys, ["a", "c", "b"], "Actions first (it came first), a title match above a hidden keyword, then Pages: {keys:?}");
    assert!(hits[2].title == vec![0, 1, 2], "the letters that matched are reported");
    let words = search(&items, "open set", Match::Words);
    assert_eq!(words.iter().map(|h| h.item.key.as_str()).collect::<Vec<_>>(), ["a"], "every word must appear in the label");
    assert_eq!(search(&items, "", Match::Fuzzy).len(), 4, "an empty query keeps everything, in order");
    assert!(search(&items, "zzz", Match::Fuzzy).is_empty());
}

#[test]
fn a_row_has_a_lead_a_title_over_a_second_line_a_hint_and_says_its_place() {
    let rt = item("command");
    let row = drawn(&rt, "command-fuzzy-t1");
    let t = rune_ui_themes::theme(false);
    assert_eq!(row[3], t.metrics.row + t.space.at(4.0), "two lines are the row height plus 16");
    let texts = titles(&rt, "command-fuzzy-t1");
    assert!(texts.contains(&"acme-web · Working".to_owned()) && texts.contains(&"2m".to_owned()), "second line and hint: {texts:?}");
    let node = ax(&rt.accessibility_json(), "command-fuzzy-t1").unwrap().to_string();
    assert!(node.contains("Fix the login redirect") && node.contains("acme-web · Working") && node.contains("\"position\":1"), "named, described and placed: {node}");
    assert_eq!(info(&rt, "command-fuzzy-t1").role, "option");
    // One-line rows keep the old height.
    assert_eq!(drawn(&rt, "command-inline-new")[3], t.metrics.row);
}

#[test]
fn recents_come_first_once_and_sections_are_groups_with_headings() {
    let rt = item("command");
    assert!(drawn(&rt, "command-recent-t2")[1] < drawn(&rt, "command-recent-settings")[1], "in the order given");
    assert!(drawn(&rt, "command-recent-settings")[1] < drawn(&rt, "command-recent-new")[1], "before the rest");
    let tree = rt.accessibility_json().to_string();
    assert!(tree.contains("\"name\":\"Recent\""), "a Recent heading: {}", &tree[..tree.len().min(200)]);
    let n = rt.elements().iter().filter(|e| e.key.as_deref() == Some("command-recent-t2")).count();
    assert_eq!(n, 1, "an item is not drawn twice");
}

#[test]
fn loading_keeps_rows_spins_in_the_field_and_says_searching_when_nothing_is_there() {
    let rt = item("command");
    assert!(has(&rt, "command-inline"), "palette shown");
    assert!(has(&rt, "command-loading-t1"), "the rows that are there stay");
    let spin = rt.elements().into_iter().filter(|e| e.role == "progress-bar" || e.label.as_deref() == Some("Searching")).count();
    assert!(spin >= 1, "a spinner in the field");
    assert!(titles(&rt, "command-wait").contains(&"Searching…".to_owned()), "and the empty state says so");
    assert!(!titles(&rt, "command-wait").contains(&"No results".to_owned()));
}

#[test]
fn the_palette_moves_with_up_and_down_chooses_with_return_and_cmd_return_and_closes_with_escape() {
    let mut rt = item("command");
    press(&mut rt, "command-open");
    assert!(has(&rt, "command-demo"), "open");
    assert_eq!(focused(&rt).as_deref(), Some("command-demo-input"), "the field has focus");
    assert!(has(&rt, "command-demo-hints"), "the footer says what the keys do");
    key(&mut rt, Key::Down);
    assert_eq!(rt.state.g.index("command-at", 0), 1, "Down");
    key(&mut rt, Key::Up);
    key(&mut rt, Key::Up);
    assert_eq!(rt.state.g.index("command-at", 0), 7, "Up wraps to the last of eight rows");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("command-chose", ""), "t3", "Return chooses the highlighted row");
    key_with(&mut rt, Key::Enter, cmd());
    assert_eq!(rt.state.g.choice("command-chose", ""), "t3 (alternate)", "Command+Return is the alternate");
    key(&mut rt, Key::Escape);
    assert!(!has(&rt, "command-demo"), "Escape closes");
}

#[test]
fn the_palettes_secondary_inks_reach_their_contrast_on_the_floating_surface() {
    for dark in [false, true] {
        let t = Theme::from_json(rune_ui_themes::DEFAULT, dark).unwrap();
        assert!(contrast(t.ink_faint(), t.c.floating.bg) >= 3.0 - 0.05, "hints and headings, dark {dark}");
        for tone in [rune_ui_style::Tone::Info, rune_ui_style::Tone::Warning, rune_ui_style::Tone::Success] {
            let c = contrast(tone.colors(&t).1, t.c.floating.bg);
            assert!(c >= 3.0 - 0.05, "a status mark, dark {dark}, {tone:?}: {c}");
        }
    }
}

// ---- The two-line list row (Sidebar) ----

fn text_of(rt: &Runtime<App>, row_key: &str, words: &str) -> ElementInfo {
    let r = drawn(rt, row_key);
    rt.elements().into_iter().find(|e| e.text.as_deref() == Some(words) && e.rect[1] >= r[1] - 0.5 && e.rect[1] < r[1] + r[3]).unwrap_or_else(|| panic!("no {words:?} in {row_key}"))
}

#[test]
fn a_thread_row_has_an_element_for_its_second_line_a_bold_unread_title_and_the_densitys_height() {
    let rt = item("sidebar");
    let t = rune_ui_themes::theme(false);
    assert_eq!(drawn(&rt, "sidebar-th1")[3], t.metrics.row + t.space.at(4.0), "two lines at the theme's density");
    let title = text_of(&rt, "sidebar-th1", "Fix the login redirect");
    let read = text_of(&rt, "sidebar-th3", "Review ENG-123");
    assert_eq!(title.text_style.unwrap().weight, t.ty.semibold, "unread: semibold");
    assert_eq!(read.text_style.unwrap().weight, t.ty.regular, "read: regular");
    assert_eq!(title.text_color, Some(t.ink()), "unread in the first ink");
    assert_eq!(read.text_color, Some(t.ink_muted()));
    let word = text_of(&rt, "sidebar-th1", "Needs you");
    let rest = text_of(&rt, "sidebar-th1", "· acme-web");
    assert!(word.rect[1] >= title.rect[1] + title.rect[3] - 1.0, "the second line is under the title");
    assert_ne!(word.text_color, rest.text_color, "the state word has its own ink");
    assert!(rest.rect[0] > word.rect[0] + word.rect[2], "and the quiet words follow it");
    let node = ax(&rt.accessibility_json(), "sidebar-th1").unwrap().to_string();
    assert!(node.contains("Needs you") && node.contains("Unread"), "assistive technology reads the state and Unread: {node}");
    // A compact theme gives a shorter row: 30 and 16.
    let rt = scene("item-sidebar-compact", false);
    assert_eq!(drawn(&rt, "sidebar-th1")[3], 46.0, "compact density: 30 + 16");
}

#[test]
fn the_trailing_slot_swaps_the_age_for_a_menu_button_under_the_pointer_and_under_focus() {
    let mut rt = item("sidebar");
    let (age, menu) = (text_of(&rt, "sidebar-th2", "14m"), info(&rt, "sidebar-th2-menu"));
    assert_eq!((age.opacity, menu.opacity), (1.0, 0.0), "at rest: the age shows");
    let r = drawn(&rt, "sidebar-th2");
    rt.pointer_move(r[0] + 60.0, r[1] + r[3] / 2.0);
    settle(&mut rt);
    let (age, menu) = (text_of(&rt, "sidebar-th2", "14m"), info(&rt, "sidebar-th2-menu"));
    assert_eq!((age.opacity, menu.opacity), (0.0, 1.0), "under the pointer: the menu button");
    let (mx, my) = rt.find("sidebar-th2-menu").unwrap();
    assert!(mx > r[0] + r[2] - 40.0 && (my - (r[1] + r[3] / 2.0)).abs() < 1.0, "at the row's end, centred on it");
    rt.pointer_move(5.0, 5.0);
    settle(&mut rt);
    assert_eq!(info(&rt, "sidebar-th2-menu").opacity, 0.0, "and back");
    // Focus inside shows it too.
    focus(&mut rt, "sidebar-th2-menu");
    assert_eq!(info(&rt, "sidebar-th2-menu").opacity, 1.0, "focus on the button shows it");
    // The menu button is its own press: it does not choose the row.
    let before = rt.state.g.presses;
    press(&mut rt, "sidebar-th2-menu");
    assert_eq!(rt.state.g.presses, before + 1);
    // Pinned to hover: the slot shows the button and no age.
    assert_eq!(info(&rt, "sidebar-th3-menu").opacity, 1.0);
    let r3 = drawn(&rt, "sidebar-th3");
    assert!(!rt.elements().into_iter().any(|e| e.text.as_deref() == Some("3d") && e.rect[1] >= r3[1] && e.rect[1] < r3[1] + r3[3]), "no age while pinned");
}

#[test]
fn a_nested_row_is_set_in_by_sixteen_points_a_level() {
    let rt = item("sidebar");
    let (a, b) = (drawn(&rt, "sidebar-th2"), drawn(&rt, "sidebar-th4"));
    let t2 = text_of(&rt, "sidebar-th2", "Plan the release notes");
    let t4 = text_of(&rt, "sidebar-th4", "Fix flaky tests");
    assert_eq!(a[0], b[0], "the row itself spans the column");
    assert!((t4.rect[0] - t2.rect[0] - 16.0).abs() < 0.6, "its words are 16 points in: {} vs {}", t4.rect[0], t2.rect[0]);
}

// ---- The small asks: icons, Textarea mono, Dialog scroll, Kbd glyphs ----

#[test]
fn the_icon_set_has_a_plug_the_panels_and_the_modifier_keys() {
    use ui::icons::*;
    for i in [&PLUG, &CHEVRONS_RIGHT, &PANEL_LEFT, &PANEL_RIGHT, &KEY_COMMAND, &KEY_OPTION, &KEY_SHIFT, &KEY_RETURN, &KEY_CONTROL] {
        assert!(i.name.starts_with("rune-ui-") && i.body.contains("currentColor") && i.view == 16.0, "{}", i.name);
    }
    assert_eq!(PLUG.name, "rune-ui-plug");
}

#[test]
fn a_mono_textarea_sets_its_text_in_a_monospaced_face_a_little_smaller_than_the_page_face() {
    let rt = item("textarea");
    let (mono, plain) = (info(&rt, "textarea-mono-input").text_style.unwrap(), info(&rt, "textarea-mono-off-input").text_style.unwrap());
    assert_ne!(format!("{:?}", mono.family), format!("{:?}", plain.family), "a different face");
    assert!(mono.size < plain.size && mono.size >= plain.size * 0.9, "a little smaller: {} against {}", mono.size, plain.size);
    assert_eq!(drawn(&rt, "textarea-mono")[2], 300.0);
}

#[test]
fn kbd_draws_modifier_glyphs_as_icons_one_cap_each_and_speaks_them() {
    let rt = item("kbd");
    let run = ax(&rt.accessibility_json(), "kbd-glyph-run").unwrap();
    assert!(run.to_string().contains("Shift Command P"), "spoken: {run}");
    assert_eq!(titles(&rt, "kbd-glyph-run"), vec!["P".to_owned()], "only the letter is text: the two modifiers are icons");
    let caps = |key: &str| rt.elements().into_iter().filter(|e| e.parent.is_some() && e.rect[3] == 20.0 && e.rect[0] >= drawn(&rt, key)[0] && e.rect[0] + e.rect[2] <= drawn(&rt, key)[0] + drawn(&rt, key)[2] + 0.5 && e.rect[1] >= drawn(&rt, key)[1] - 0.5 && e.rect[1] < drawn(&rt, key)[1] + 20.0).count();
    assert_eq!(caps("kbd-glyph-run"), 4, "⇧⌘P is three caps (and the row round them), though it is written without spaces");
    let t = rune_ui_themes::theme(false);
    let p = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("P") && e.rect[1] > drawn(&rt, "kbd-glyph-run")[1] - 1.0 && e.rect[1] < drawn(&rt, "kbd-glyph-run")[1] + 20.0).unwrap();
    assert_eq!(p.text_style.unwrap().size, t.ty.xs.size, "the word is in the keycap face and size");
}

struct DialogProbe;

fn scrolling_dialog_view(_: &DialogProbe, _: &mut Cx<DialogProbe>) -> El {
    let t = Theme::from_json(rune_ui_themes::DEFAULT, false).unwrap();
    let mut layer = Vec::new();
    let mut d = ui::dialog::Dialog::<DialogProbe>::new("d", "Project settings").description("Everything.").scroll_body(true).open(true).footer(vec![ui::button::Button::<DialogProbe>::new("d-save", "Save").build(&t)]).on_close(|_: &mut DialogProbe| {});
    for i in 0..30 {
        d = d.child(rune::text(format!("Row {i}")).key(format!("row{i}")));
    }
    d.build(&t, &mut layer);
    ui::base::overlay_root(col().w_full().h_full(), layer)
}

#[test]
fn a_dialog_with_a_scrolling_body_keeps_its_header_and_footer_in_the_window_and_scrolls_between() {
    let mut rt = Runtime::new(DialogProbe, scrolling_dialog_view, (700.0, 420.0), Env { system_dark: false, scale: 1.0 });
    settle(&mut rt);
    let (panel, body, foot, head) = (drawn(&rt, "d-dialog"), drawn(&rt, "d-body"), drawn(&rt, "d-footer"), drawn(&rt, "d-header"));
    assert!(panel[1] >= 0.0 && panel[1] + panel[3] <= 420.0 + 0.5, "the panel fits the window: {panel:?}");
    assert!(head[1] < body[1] && body[1] + body[3] <= foot[1] + 0.5, "header, then body, then footer");
    assert!(foot[1] + foot[3] <= panel[1] + panel[3], "the footer is inside the panel");
    let last = drawn(&rt, "row29");
    assert!(last[1] > body[1] + body[3], "most rows are past the body's edge: they scroll");
    let (x, y) = (body[0] + body[2] / 2.0, body[1] + body[3] / 2.0);
    let first = drawn(&rt, "row0")[1];
    rt.pointer_move(x, y);
    rt.wheel(x, y, -120.0);
    settle(&mut rt);
    assert!(drawn(&rt, "row0")[1] < first, "the wheel scrolled the body: {} -> {}", first, drawn(&rt, "row0")[1]);
    assert_eq!(drawn(&rt, "d-footer")[1], foot[1], "and the footer did not move");
    assert_eq!(drawn(&rt, "d-header")[1], head[1], "nor the header");
}
