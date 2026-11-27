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
