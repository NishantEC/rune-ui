//! Overlays float: in the live window path (the page in a scroll view, overlays at the root), opening each one leaves every element of
//! the page exactly where it was, and the overlay sits where it belongs: next to its trigger (flipping above it near the window's
//! bottom, staying inside the window, also when the page is scrolled), at the pointer, centred over a scrim, or against an edge.

use gallery::{App, apply_scene, set_headless, view};
use rune::{Env, Runtime};

const W: f32 = 1320.0;
const H: f32 = 800.0;

fn runtime(s: App, h: f32) -> Runtime<App> {
    set_headless(false);
    let mut rt = Runtime::new(s, view, (W, h), Env { system_dark: false, scale: 1.0 });
    settle(&mut rt);
    rt
}

fn settle(rt: &mut Runtime<App>) {
    for _ in 0..90 {
        rt.frame(1.0 / 60.0);
    }
}

/// Every element's layout rect, by its unique path.
fn layout(rt: &Runtime<App>) -> Vec<(String, [f32; 4])> {
    rt.elements().into_iter().map(|e| (e.path, e.rect)).collect()
}

fn drawn(rt: &Runtime<App>, key: &str) -> [f32; 4] {
    rt.presented(key).unwrap_or_else(|| panic!("{key} is not drawn")).0
}

fn inside_window(o: [f32; 4], h: f32, what: &str) {
    assert!(o[0] >= -0.5 && o[1] >= -0.5 && o[0] + o[2] <= W + 0.5 && o[1] + o[3] <= h + 0.5, "{what} leaves the window: {o:?}");
}

/// Beside the trigger: overlapping it across, and starting just under it or ending just over it. Returns whether it is above.
fn beside(o: [f32; 4], t: [f32; 4], what: &str) -> bool {
    assert!(o[0] < t[0] + t[2] && o[0] + o[2] > t[0], "{what} is not under or over its trigger across: {o:?} vs {t:?}");
    let below = (o[1] - (t[1] + t[3])).abs() <= 16.0;
    let above = ((o[1] + o[3]) - t[1]).abs() <= 16.0;
    assert!(below || above, "{what} is not next to its trigger: {o:?} vs {t:?}");
    above && !below
}

enum Place {
    /// Next to the element with this key.
    Trigger(&'static str),
    /// Near the pointer that opened it.
    Point(f32, f32),
    /// Centred, with this scrim covering the window.
    Centred(&'static str),
    /// Against the window's right edge, full height.
    RightEdge,
    /// In the window's bottom right corner.
    Corner,
}

/// (open scene, overlay key, where it goes).
const CASES: [(&str, &str, Place); 10] = [
    ("open-select", "select-demo-content", Place::Trigger("select-demo-trigger")),
    ("open-combobox", "combobox-demo-content", Place::Trigger("combobox-demo-trigger")),
    ("open-dropdown-menu", "dropdown-demo-content", Place::Trigger("dropdown-demo-trigger")),
    ("open-popover", "popover-demo-panel", Place::Trigger("popover-demo-trigger")),
    ("open-context-menu", "context-demo-content", Place::Point(260.0, 330.0)),
    ("open-dialog", "dialog-demo-dialog", Place::Centred("dialog-demo-scrim")),
    ("open-alert-dialog", "alert-demo-dialog", Place::Centred("alert-demo-scrim")),
    ("open-command", "command-demo", Place::Centred("command-demo-scrim")),
    ("open-sheet", "sheet-demo-sheet", Place::RightEdge),
    ("open-toast", "toast-demo", Place::Corner),
];

#[test]
fn opening_an_overlay_moves_nothing_and_it_lands_where_it_belongs() {
    for (scene, overlay, place) in CASES {
        // The same item closed, then open.
        let mut closed = App::default();
        apply_scene(&mut closed, scene);
        let (open_key, point, toasts) = (closed.g.open.take(), closed.g.point, std::mem::take(&mut closed.g.toasts));
        let mut rt = runtime(closed, H);
        let before = layout(&rt);
        assert!(rt.presented(overlay).is_none(), "{scene}: {overlay} shows before it opens");
        rt.update(move |s, _| {
            s.g.open = open_key;
            s.g.point = point;
            s.g.toasts = toasts;
        });
        settle(&mut rt);
        let after: std::collections::HashMap<String, [f32; 4]> = layout(&rt).into_iter().collect();
        for (path, r) in &before {
            if let Some(a) = after.get(path) {
                assert!(r.iter().zip(a).all(|(x, y)| (x - y).abs() < 0.01), "{scene}: opening moved {path}: {r:?} -> {a:?}");
            }
        }
        let o = drawn(&rt, overlay);
        inside_window(o, H, overlay);
        match place {
            Place::Trigger(t) => {
                beside(o, drawn(&rt, t), overlay);
            }
            Place::Point(x, y) => assert!((o[0] - x).abs() <= 16.0 && (o[1] - y).abs() <= 16.0, "{overlay} is not at the pointer: {o:?}"),
            Place::Centred(scrim) => {
                let s = drawn(&rt, scrim);
                assert!(s[0] <= 0.5 && s[1] <= 0.5 && s[2] >= W - 0.5 && s[3] >= H - 0.5, "{scene}: the scrim does not cover the window: {s:?}");
                assert!(((o[0] + o[2] / 2.0) - W / 2.0).abs() <= 2.0, "{overlay} is not centred across: {o:?}");
            }
            Place::RightEdge => assert!((o[0] + o[2] - W).abs() <= 0.5 && o[1] <= 0.5 && o[3] >= H - 0.5, "{overlay} is not against the right edge: {o:?}"),
            Place::Corner => assert!(o[0] + o[2] >= W - 40.0 && o[1] + o[3] >= H - 40.0, "{overlay} is not in the bottom right: {o:?}"),
        }
    }
}

/// The page scrolled so the select's trigger sits at `y` (window points).
fn scrolled_to(y: f32, h: f32) -> Runtime<App> {
    let mut s = App::default();
    s.g.show = "form".to_owned();
    let mut rt = runtime(s, h);
    let start = drawn(&rt, "select-demo-trigger")[1];
    // A negative wheel delta scrolls the content up.
    rt.wheel(W / 2.0, h / 2.0, y - start);
    settle(&mut rt);
    let at = drawn(&rt, "select-demo-trigger")[1];
    assert!((at - y).abs() < 40.0 && (at - start).abs() > 100.0, "the page scrolled the trigger to {at} (wanted {y}, from {start})");
    rt
}

#[test]
fn inside_a_scrolled_page_the_list_stays_with_its_trigger_and_flips_near_the_bottom() {
    let h = 640.0;
    for (y, want_above) in [(160.0, false), (h - 50.0, true)] {
        let mut rt = scrolled_to(y, h);
        let before = layout(&rt);
        rt.update(|s, _| s.g.open = Some("select-demo".to_owned()));
        settle(&mut rt);
        let after: std::collections::HashMap<String, [f32; 4]> = layout(&rt).into_iter().collect();
        for (path, r) in &before {
            if let Some(a) = after.get(path) {
                assert!(r.iter().zip(a).all(|(x, y)| (x - y).abs() < 0.01), "opening moved {path} in the scrolled page");
            }
        }
        let o = drawn(&rt, "select-demo-content");
        inside_window(o, h, "the list");
        let above = beside(o, drawn(&rt, "select-demo-trigger"), "the list");
        assert_eq!(above, want_above, "trigger at {y}: the list is {} it", if above { "above" } else { "below" });
    }
}

// ---- the lists, through real presses and keys ----

use rune::{Key, KeyEvent, Mods};

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

fn focused(rt: &Runtime<App>) -> Option<String> {
    rt.elements().into_iter().find(|e| e.focused).and_then(|e| e.key)
}

fn item(name: &str) -> Runtime<App> {
    let mut s = App::default();
    apply_scene(&mut s, &format!("item-{name}"));
    runtime(s, H)
}

#[test]
fn a_select_list_shares_its_triggers_left_edge_and_width_and_follows_the_keys() {
    let mut rt = item("select");
    press(&mut rt, "select-demo-trigger");
    let (t, o) = (drawn(&rt, "select-demo-trigger"), drawn(&rt, "select-demo-content"));
    assert!((o[0] - t[0]).abs() < 0.5, "left edges: list {} trigger {}", o[0], t[0]);
    assert!(o[2] >= t[2] - 0.5, "the list ({}) is at least the trigger's width ({})", o[2], t[2]);
    assert!(o[1] - (t[1] + t[3]) >= 3.5 && o[1] - (t[1] + t[3]) <= 6.5, "a 4 to 6 point gap: {}", o[1] - (t[1] + t[3]));
    assert_eq!(focused(&rt).as_deref(), Some("select-demo-item2"), "the current choice is focused on open");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("select-demo-item3"), "Down moves");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("select-demo-item1"), "and wraps");
    key(&mut rt, Key::End);
    assert_eq!(focused(&rt).as_deref(), Some("select-demo-item3"), "End");
    key(&mut rt, Key::Home);
    assert_eq!(focused(&rt).as_deref(), Some("select-demo-item1"), "Home");
    key(&mut rt, Key::Text("h".to_owned()));
    assert_eq!(focused(&rt).as_deref(), Some("select-demo-item3"), "a typed letter jumps to Hands-off");
    key(&mut rt, Key::Enter);
    assert!(rt.presented("select-demo-content").is_none(), "Return chooses and closes");
    assert_eq!(rt.state.g.choice("select", ""), "Hands-off");
    assert_eq!(focused(&rt).as_deref(), Some("select-demo-trigger"), "focus is back on the trigger");
    press(&mut rt, "select-demo-trigger");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("select-demo-content").is_none(), "Escape closes");
    assert_eq!(focused(&rt).as_deref(), Some("select-demo-trigger"), "Escape returns focus");
    press(&mut rt, "select-demo-trigger");
    key(&mut rt, Key::Tab);
    assert!(rt.presented("select-demo-content").is_none(), "Tab closes");
}

#[test]
fn a_dropdown_menu_opens_at_its_trigger_moves_with_the_keys_and_gives_focus_back() {
    let mut rt = item("dropdown_menu");
    press(&mut rt, "dropdown-trigger");
    let (t, o) = (drawn(&rt, "dropdown-trigger"), drawn(&rt, "dropdown-demo-content"));
    assert!((o[0] - t[0]).abs() < 0.5 && o[2] >= t[2], "left edge and width: {o:?} vs {t:?}");
    key(&mut rt, Key::Down);
    let first = focused(&rt);
    key(&mut rt, Key::Down);
    assert_ne!(focused(&rt), first, "the highlight follows Down");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("dropdown-demo-content").is_none());
    assert_eq!(focused(&rt).as_deref(), Some("dropdown-trigger"), "Escape returns focus to the trigger");
}

#[test]
fn a_combobox_filters_as_you_type_and_its_highlight_follows() {
    let mut rt = item("combobox");
    press(&mut rt, "combobox-demo-trigger");
    let (t, o) = (drawn(&rt, "combobox-demo-trigger"), drawn(&rt, "combobox-demo-content"));
    assert!((o[0] - t[0]).abs() < 0.5 && o[2] >= t[2] - 0.5, "left edge and width: {o:?} vs {t:?}");
    assert_eq!(focused(&rt).as_deref(), Some("combobox-demo-input"), "focus is in the search");
    key(&mut rt, Key::Down);
    assert_eq!(rt.state.g.index("combobox-hl", 0), 1, "Down moves the highlight");
    for ch in "sample".chars() {
        key(&mut rt, Key::Text(ch.to_string()));
    }
    assert!(rt.presented("combobox-demo-option-placeholder-app").is_none() && rt.presented("combobox-demo-option-sample-worker").is_some(), "typing filters");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("combobox", ""), "sample-worker", "Return chooses the highlighted match");
    assert!(rt.presented("combobox-demo-content").is_none());
    press(&mut rt, "combobox-demo-trigger");
    for ch in "zzz".chars() {
        key(&mut rt, Key::Text(ch.to_string()));
    }
    assert!(rt.presented("combobox-demo-empty").is_some(), "No results");
    key(&mut rt, Key::Escape);
    assert_eq!(focused(&rt).as_deref(), Some("combobox-demo-trigger"), "Escape returns focus");
}
