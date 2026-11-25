//! Wave 2's interactive components in the live window path (the page in a scroll view, overlays at the root), driven by real presses,
//! pointer moves and keys: what opens, where it lands, where focus goes and comes back, and that nothing on the page moves.

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

/// Every element present in both layouts sits where it did.
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

fn key(rt: &mut Runtime<App>, k: Key) {
    rt.key(KeyEvent { key: k, mods: Mods::default() });
    settle(rt);
}

fn shift(rt: &mut Runtime<App>, k: Key) {
    rt.key(KeyEvent { key: k, mods: Mods { shift: true, ..Mods::default() } });
    settle(rt);
}

fn focused(rt: &Runtime<App>) -> Option<String> {
    rt.elements().into_iter().find(|e| e.focused).and_then(|e| e.key)
}

fn focus(rt: &mut Runtime<App>, k: &str) {
    let k = k.to_owned();
    rt.update(move |_, tx| tx.focus(k, true));
    settle(rt);
}

// ---- Calendar ----

#[test]
fn a_calendar_moves_by_day_week_and_month_and_steps_over_disabled_days() {
    let mut rt = item("calendar");
    let before = layout(&rt);
    press(&mut rt, "calendar-day-2026-11-20");
    assert_eq!(rt.state.g.choice("calendar-day", ""), "2026-11-20", "a press chooses");
    assert_eq!(focused(&rt).as_deref(), Some("calendar-day-2026-11-20"));
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-day-2026-11-21"), "Right is a day");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-day-2026-11-28"), "Down is a week");
    key(&mut rt, Key::Home);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-day-2026-11-22"), "Home is the week's Sunday");
    key(&mut rt, Key::End);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-day-2026-11-28"), "End is its Saturday");
    key(&mut rt, Key::Down);
    assert_eq!(rt.state.g.choice("calendar-month", ""), "2026-12-01", "crossing a month shows the next one");
    assert_eq!(focused(&rt).as_deref(), Some("calendar-day-2026-12-05"), "and the keyboard lands there");
    key(&mut rt, Key::PageUp);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-day-2026-11-05"), "Page Up is a month");
    shift(&mut rt, Key::PageDown);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-day-2027-11-05"), "Shift Page Down is a year");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("calendar-day", ""), "2027-11-05", "Return chooses");
    press(&mut rt, "calendar-prev");
    assert_eq!(rt.state.g.choice("calendar-month", ""), "2027-10-01", "the arrow steps a month");
    // Weekends and past days are skipped.
    focus(&mut rt, "calendar-limited-day-2026-11-27");
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-limited-day-2026-11-30"), "Right steps over the weekend");
    key(&mut rt, Key::Left);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-limited-day-2026-11-27"), "Left steps back over it");
    press(&mut rt, "calendar-limited-day-2026-11-28");
    assert!(!rt.state.g.choices.contains_key("calendar-limited-day"), "a disabled day cannot be chosen");
    // The grid keeps its size whatever the month (six weeks always).
    let r = drawn(&rt, "calendar-grid");
    let b = before.iter().find(|(p, _)| p.ends_with("calendar-grid")).map(|(_, r)| *r).expect("grid before");
    assert!((r[3] - b[3]).abs() < 0.01, "the grid's height changed with the month: {b:?} -> {r:?}");
}

#[test]
fn a_calendar_range_takes_two_presses_and_the_month_view_picks_a_month() {
    let mut rt = item("calendar");
    press(&mut rt, "calendar-range-day-2026-11-03");
    assert_eq!(rt.state.g.choice("calendar-range-a", ""), "2026-11-03");
    assert!(!rt.state.g.choices.contains_key("calendar-range-b"), "one end chosen");
    press(&mut rt, "calendar-range-day-2026-11-01");
    assert_eq!((rt.state.g.choice("calendar-range-a", ""), rt.state.g.choice("calendar-range-b", "")), ("2026-11-01".to_owned(), "2026-11-03".to_owned()), "an earlier second press becomes the start");
    let before = layout(&rt);
    press(&mut rt, "calendar-caption");
    assert!(rt.presented("calendar-months").is_some(), "the caption opens the month view");
    let (g, m) = (before.iter().find(|(p, _)| p.ends_with("calendar-grid")).unwrap().1, drawn(&rt, "calendar-months"));
    assert!((g[3] - m[3]).abs() < 0.5, "the month view is as tall as the days: {} vs {}", g[3], m[3]);
    focus(&mut rt, "calendar-month-11");
    key(&mut rt, Key::Up);
    assert_eq!(focused(&rt).as_deref(), Some("calendar-month-8"), "Up is three months");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("calendar-month", ""), "2026-08-01", "a month chosen");
    assert!(rt.presented("calendar-months").is_none() && rt.presented("calendar-grid").is_some(), "and the days come back");
}

// ---- DatePicker ----

#[test]
fn a_date_picker_opens_at_its_trigger_lands_on_the_date_and_gives_focus_back() {
    let mut rt = item("date_picker");
    let before = layout(&rt);
    press(&mut rt, "date-demo-trigger");
    let (t, o) = (drawn(&rt, "date-demo-trigger"), drawn(&rt, "date-demo-content"));
    assert!((o[0] - t[0]).abs() < 0.5 && o[1] > t[1] + t[3], "under the trigger, at its left edge: {o:?} vs {t:?}");
    unmoved(&before, &rt, "opening the date picker");
    assert_eq!(focused(&rt).as_deref(), Some("date-demo-calendar-day-2026-11-12"), "focus on the chosen day");
    key(&mut rt, Key::Right);
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("date-demo-value", ""), "2026-11-13", "Return chooses");
    assert!(rt.presented("date-demo-content").is_none(), "and closes");
    assert_eq!(focused(&rt).as_deref(), Some("date-demo-trigger"), "focus back on the trigger");
    press(&mut rt, "date-demo-trigger");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("date-demo-content").is_none(), "Escape closes");
    assert_eq!(focused(&rt).as_deref(), Some("date-demo-trigger"));
    press(&mut rt, "date-demo-trigger");
    rt.pointer_down(W - 20.0, H - 20.0);
    rt.pointer_up(W - 20.0, H - 20.0);
    settle(&mut rt);
    assert!(rt.presented("date-demo-content").is_none(), "a press outside closes");
}

#[test]
fn a_range_date_picker_stays_open_until_both_ends_are_chosen() {
    let mut rt = item("date_picker");
    press(&mut rt, "date-range-trigger");
    press(&mut rt, "date-range-calendar-day-2026-11-20");
    assert!(rt.presented("date-range-content").is_some(), "one end: still open");
    press(&mut rt, "date-range-calendar-day-2026-11-24");
    assert!(rt.presented("date-range-content").is_none(), "both ends: closed");
    assert_eq!((rt.state.g.choice("date-range-a", ""), rt.state.g.choice("date-range-b", "")), ("2026-11-20".to_owned(), "2026-11-24".to_owned()));
}

// ---- Menubar ----

#[test]
fn a_menubar_opens_from_the_keys_moves_across_menus_and_gives_focus_back() {
    let mut rt = item("menubar");
    let before = layout(&rt);
    focus(&mut rt, "menubar-demo-0-trigger");
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("menubar-demo-1-trigger"), "Right moves along the bar");
    key(&mut rt, Key::Down);
    assert!(rt.presented("menubar-demo-1-content").is_some(), "Down opens Edit");
    unmoved(&before, &rt, "opening a menu");
    let (t, o) = (drawn(&rt, "menubar-demo-1-trigger"), drawn(&rt, "menubar-demo-1-content"));
    assert!((o[0] - t[0]).abs() < 0.5 && o[1] > t[1] + t[3], "under its title: {o:?} vs {t:?}");
    assert_eq!(focused(&rt).as_deref(), Some("menubar-demo-1-item1"), "its first row is focused");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("menubar-demo-1-item2"), "Down moves in the menu");
    key(&mut rt, Key::Right);
    assert!(rt.presented("menubar-demo-1-content").is_none() && rt.presented("menubar-demo-2-content").is_some(), "Right opens View");
    assert_eq!(focused(&rt).as_deref(), Some("menubar-demo-2-item2"), "on its first row (after its label)");
    key(&mut rt, Key::Right);
    assert!(rt.presented("menubar-demo-0-content").is_some(), "Right skips the disabled menu and wraps to File");
    key(&mut rt, Key::Left);
    assert!(rt.presented("menubar-demo-2-content").is_some(), "Left wraps back to View");
    key(&mut rt, Key::Enter);
    assert!(rt.state.g.flag("menubar-toolbar") && rt.presented("menubar-demo-2-content").is_none(), "Return chooses and closes");
    assert_eq!(focused(&rt).as_deref(), Some("menubar-demo-2-trigger"), "focus back on the title");
    press(&mut rt, "menubar-demo-0-trigger");
    assert_eq!(focused(&rt).as_deref(), Some("menubar-demo-0-item1"), "a press opens with the first row focused");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("menubar-demo-0-content").is_none(), "Escape closes");
    assert_eq!(focused(&rt).as_deref(), Some("menubar-demo-0-trigger"), "and returns focus");
    press(&mut rt, "menubar-demo-3-trigger");
    assert!(rt.state.g.open.is_none(), "a disabled menu does not open");
}

// ---- NavigationMenu ----

#[test]
fn a_navigation_menu_opens_panels_moves_between_them_and_follows_links() {
    let mut rt = item("navigation_menu");
    let before = layout(&rt);
    press(&mut rt, "nav-demo-0-trigger");
    assert!(rt.presented("nav-demo-0-content").is_some(), "a press opens the panel");
    unmoved(&before, &rt, "opening a panel");
    assert_eq!(focused(&rt).as_deref(), Some("nav-demo-0-link0"), "its first link is focused");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("nav-demo-0-link1"));
    key(&mut rt, Key::Right);
    assert!(rt.presented("nav-demo-1-content").is_some(), "Right at the edge opens the next panel");
    assert_eq!(focused(&rt).as_deref(), Some("nav-demo-1-link0"));
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("nav-demo-1-link1"), "Right inside two columns moves across");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("nav-demo-1-link3"), "Down moves a row");
    let n = rt.state.g.presses;
    key(&mut rt, Key::Enter);
    assert!(rt.state.g.presses == n + 1 && rt.presented("nav-demo-1-content").is_none(), "Return follows the link and closes");
    focus(&mut rt, "nav-demo-1-trigger");
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Escape);
    assert_eq!(focused(&rt).as_deref(), Some("nav-demo-1-trigger"), "Escape returns focus to the title");
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("nav-demo-2-trigger"), "Right moves along the bar to the link");
}
