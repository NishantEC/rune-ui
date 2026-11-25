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

// ---- Pagination ----

#[test]
fn pagination_moves_pages_disables_its_ends_and_keeps_its_width() {
    let mut rt = item("pagination");
    let w = drawn(&rt, "pagination-demo")[2];
    press(&mut rt, "pagination-demo-next");
    assert_eq!(rt.state.g.index("pagination", 0), 7, "Next");
    press(&mut rt, "pagination-demo-page-20");
    assert_eq!(rt.state.g.index("pagination", 0), 20, "a page");
    assert!((drawn(&rt, "pagination-demo")[2] - w).abs() < 0.5, "the row keeps its width");
    let next = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("pagination-demo-next")).expect("next");
    assert!(!next.focusable, "Next is off on the last page");
    press(&mut rt, "pagination-demo-previous");
    assert_eq!(rt.state.g.index("pagination", 0), 19, "Previous");
}

// ---- Resizable ----

fn shares(rt: &Runtime<App>, k: &str) -> Vec<f32> {
    rt.state.g.choices.get(k).map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default()
}

#[test]
fn a_resizable_handle_drags_steps_with_keys_and_keeps_its_bounds() {
    let mut rt = item("resizable");
    let before = layout(&rt);
    let (h, g) = (drawn(&rt, "resizable-demo-handle-0"), drawn(&rt, "resizable-demo"));
    let (x, y) = (h[0] + h[2] / 2.0, h[1] + h[3] / 2.0);
    rt.pointer_down(x, y);
    rt.pointer_move(x + 20.0, y);
    rt.pointer_move(x + 60.0, y);
    rt.pointer_up(x + 60.0, y);
    settle(&mut rt);
    let s = shares(&rt, "resizable-demo");
    let want = 0.35 + 60.0 / (g[2] - 9.0);
    assert!((s[0] - want).abs() < 0.01 && (s[0] + s[1] - 1.0).abs() < 0.001, "dragged 60 points: {s:?}, wanted {want}");
    let left = drawn(&rt, "resizable-demo-pane-0");
    assert!((left[2] - s[0] * (g[2] - 9.0)).abs() < 1.0, "the pane follows: {left:?}");
    // Past the most share it stops.
    let h = drawn(&rt, "resizable-demo-handle-0");
    rt.pointer_down(h[0] + 4.0, y);
    rt.pointer_move(h[0] + 400.0, y);
    rt.pointer_up(h[0] + 400.0, y);
    settle(&mut rt);
    assert!((shares(&rt, "resizable-demo")[0] - 0.6).abs() < 0.001, "kept at its most");
    // Keys.
    focus(&mut rt, "resizable-demo-handle-0");
    key(&mut rt, Key::Left);
    assert!((shares(&rt, "resizable-demo")[0] - 0.55).abs() < 0.001, "Left steps 5%");
    key(&mut rt, Key::Home);
    assert!((shares(&rt, "resizable-demo")[0] - 0.2).abs() < 0.001, "Home goes to the least");
    // A press inside a pane does not resize.
    let p = drawn(&rt, "resizable-demo-pane-1");
    rt.pointer_down(p[0] + 100.0, p[1] + 50.0);
    rt.pointer_move(p[0] + 200.0, p[1] + 50.0);
    rt.pointer_up(p[0] + 200.0, p[1] + 50.0);
    settle(&mut rt);
    assert!((shares(&rt, "resizable-demo")[0] - 0.2).abs() < 0.001, "a drag inside a pane resizes nothing");
    // Vertical, with the keys.
    focus(&mut rt, "resizable-stack-handle-0");
    key(&mut rt, Key::Down);
    assert!((shares(&rt, "resizable-stack")[0] - 0.65).abs() < 0.001, "Down grows the top pane");
    // Only the resized groups changed size; everything outside them stayed.
    for (path, a) in layout(&rt) {
        if path.contains("resizable") {
            continue;
        }
        if let Some(b) = before.get(&path) {
            assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01), "resizing moved {path}");
        }
    }
}

// ---- HoverCard ----

#[test]
fn a_hover_card_opens_after_a_rest_stays_over_the_card_and_closes_after_leaving() {
    let mut rt = item("hover_card");
    let before = layout(&rt);
    let t = drawn(&rt, "hover-demo-trigger");
    rt.pointer_move(t[0] + t[2] / 2.0, t[1] + t[3] / 2.0);
    rt.frame(0.1);
    assert!(rt.presented("hover-demo-card").is_none(), "not at once");
    settle(&mut rt);
    let c = drawn(&rt, "hover-demo-card");
    assert!(c[1] >= t[1] + t[3] && c[1] - (t[1] + t[3]) < 16.0, "below its trigger: {c:?} vs {t:?}");
    unmoved(&before, &rt, "opening the hover card");
    // Onto the card: it stays.
    rt.pointer_move(c[0] + c[2] / 2.0, c[1] + c[3] / 2.0);
    settle(&mut rt);
    assert!(rt.presented("hover-demo-card").is_some(), "the card stays while the pointer is on it");
    rt.pointer_move(W - 10.0, H - 10.0);
    settle(&mut rt);
    assert!(rt.presented("hover-demo-card").is_none(), "it closes after the pointer leaves both");
    // Keys: Return pins it, Escape hides it.
    focus(&mut rt, "hover-demo-trigger");
    key(&mut rt, Key::Enter);
    assert!(rt.presented("hover-demo-card").is_some(), "Return shows it: {:?} focus {:?}", rt.state.g.flags, focused(&rt));
    key(&mut rt, Key::Escape);
    assert!(rt.presented("hover-demo-card").is_none(), "Escape hides it");
    assert_eq!(focused(&rt).as_deref(), Some("hover-demo-trigger"), "focus stays on the trigger");
}

// ---- InputOTP ----

#[test]
fn an_otp_input_fills_moves_and_backspaces() {
    let mut rt = item("input_otp");
    focus(&mut rt, "otp-demo-slot-2");
    key(&mut rt, Key::Text("x".to_owned()));
    assert_eq!(rt.state.g.choice("otp", "12"), "12", "a letter is refused");
    for d in ["3", "4", "5", "6"] {
        key(&mut rt, Key::Text(d.to_owned()));
    }
    assert_eq!(rt.state.g.choice("otp", "12"), "123456");
    assert_eq!(rt.state.g.presses, 1, "complete is heard once");
    assert_eq!(focused(&rt).as_deref(), Some("otp-demo-slot-5"), "the last slot keeps focus");
    key(&mut rt, Key::Backspace);
    assert_eq!(rt.state.g.choice("otp", "12"), "12345", "Backspace empties a filled slot in place");
    key(&mut rt, Key::Backspace);
    assert_eq!((rt.state.g.choice("otp", "12"), focused(&rt)), ("1234".to_owned(), Some("otp-demo-slot-4".to_owned())), "then the one before");
    key(&mut rt, Key::Left);
    assert_eq!(focused(&rt).as_deref(), Some("otp-demo-slot-3"), "Left moves");
    key(&mut rt, Key::Text("9".to_owned()));
    assert_eq!(rt.state.g.choice("otp", "12"), "1239", "typing replaces a filled slot");
}

// ---- Drawer ----

#[test]
fn a_drawer_rises_against_the_bottom_traps_focus_and_closes_by_key_press_or_drag() {
    let mut rt = item("drawer");
    let before = layout(&rt);
    press(&mut rt, "drawer-open");
    let d = drawn(&rt, "drawer-demo-drawer");
    assert!((d[1] + d[3] - H).abs() < 0.5 && d[0] <= 0.5 && d[2] >= W - 0.5, "against the bottom, full width: {d:?}");
    unmoved(&before, &rt, "opening the drawer");
    key(&mut rt, Key::Tab);
    let f = focused(&rt).unwrap_or_default();
    assert!(f.starts_with("drawer-demo"), "Tab stays inside the drawer: {f}");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("drawer-demo-drawer").is_none(), "Escape closes");
    assert_eq!(focused(&rt).as_deref(), Some("drawer-open"), "focus back on the trigger");
    press(&mut rt, "drawer-open");
    rt.pointer_down(20.0, 20.0);
    rt.pointer_up(20.0, 20.0);
    settle(&mut rt);
    assert!(rt.presented("drawer-demo-drawer").is_none(), "a press outside closes");
    press(&mut rt, "drawer-open");
    let g = drawn(&rt, "drawer-demo-grab");
    let (x, y) = (g[0] + g[2] / 2.0, g[1] + g[3] / 2.0);
    rt.pointer_down(x, y);
    rt.pointer_move(x, y + 30.0);
    rt.pointer_up(x, y + 30.0);
    settle(&mut rt);
    assert!(rt.presented("drawer-demo-drawer").is_some(), "a short drag leaves it open");
    rt.pointer_down(x, y);
    rt.pointer_move(x, y + 50.0);
    rt.pointer_move(x, y + 100.0);
    rt.pointer_up(x, y + 100.0);
    settle(&mut rt);
    assert!(rt.presented("drawer-demo-drawer").is_none(), "dragging the bar down closes it");
}

// ---- Form ----

#[test]
fn a_form_shows_its_messages_on_submit_and_succeeds_once_they_are_fixed() {
    let mut rt = item("form");
    let before = drawn(&rt, "form-demo-submit");
    press(&mut rt, "form-demo-submit");
    assert!(rt.presented("form-name-error").is_some() && rt.presented("form-email-error").is_some(), "the empty required fields say so");
    assert!(rt.presented("form-team-error").is_none(), "the empty optional field does not");
    let status = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("form-demo-status")).expect("a status line");
    assert_eq!((&*status.role, status.label.as_deref()), ("alert", Some("2 fields need attention")));
    for (field, words) in [("form-name-input-input", "Asha"), ("form-email-input-input", "asha@example")] {
        focus(&mut rt, field);
        for ch in words.chars() {
            key(&mut rt, Key::Text(ch.to_string()));
        }
    }
    press(&mut rt, "form-demo-submit");
    assert!(rt.presented("form-name-error").is_none(), "a fixed field clears");
    let e = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("form-email-error")).expect("email error");
    assert!(e.text.as_deref().unwrap_or_default().contains("email address"), "a malformed address is named: {:?}", e.text);
    focus(&mut rt, "form-email-input-input");
    for ch in ".com".chars() {
        key(&mut rt, Key::Text(ch.to_string()));
    }
    press(&mut rt, "form-demo-submit");
    let status = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("form-demo-status")).expect("a status line");
    assert_eq!(status.role, "status", "success is a polite status");
    let busy = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("form-busy-submit")).expect("busy submit");
    assert!(!busy.focusable, "a submitting form's button cannot be pressed again");
    let _ = before;
}

// ---- DataTable ----

fn table_state(rt: &Runtime<App>) -> gallery::ui::data_table::DataTableState {
    rt.state.g.slot("data-table")
}

#[test]
fn a_data_table_sorts_selects_hides_columns_pages_and_keeps_its_header() {
    let mut rt = item("data_table");
    press(&mut rt, "data-table-head-0");
    assert_eq!(table_state(&rt).sort, Some((0, true)), "a header sorts ascending");
    assert!(rt.presented("data-table-row-0").is_some(), "\"Add a retry…\" leads the page");
    press(&mut rt, "data-table-head-0");
    assert_eq!(table_state(&rt).sort, Some((0, false)), "then descending");
    press(&mut rt, "data-table-head-0");
    assert_eq!(table_state(&rt).sort, None, "then the data's order");
    assert_eq!(table_state(&rt).selected, [1, 3].into(), "sorting keeps the selection");
    press(&mut rt, "data-table-check-0");
    assert!(table_state(&rt).selected.contains(&0), "a row's box selects it");
    press(&mut rt, "data-table-select-all");
    assert_eq!(table_state(&rt).selected, [0, 1, 2, 3, 4].into(), "the header box selects the page");
    press(&mut rt, "data-table-select-all");
    assert!(table_state(&rt).selected.is_empty(), "and clears it");
    // Columns.
    press(&mut rt, "data-table-columns");
    assert!(rt.presented("data-table-columns-content").is_some(), "the Columns menu opens");
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Enter);
    assert!(table_state(&rt).hidden.contains(&1) && rt.presented("data-table-head-1").is_none(), "Status hides");
    assert!(rt.presented("data-table-columns-content").is_none(), "the menu closes after a choice");
    assert_eq!(focused(&rt).as_deref(), Some("data-table-columns"), "and focus is back on Columns");
    // Pages.
    press(&mut rt, "data-table-pages-next");
    assert_eq!(table_state(&rt).page, 1);
    assert!(rt.presented("data-table-row-5").is_some() && rt.presented("data-table-row-0").is_none(), "page two shows the next rows");
    // The header stays while the body scrolls.
    let (h, r) = (drawn(&rt, "data-table-head"), drawn(&rt, "data-table-row-5"));
    rt.wheel(h[0] + 100.0, r[1] + 10.0, -60.0);
    settle(&mut rt);
    let (h2, r2) = (drawn(&rt, "data-table-head"), drawn(&rt, "data-table-row-5"));
    assert!(r2[1] < r[1] - 10.0, "the body scrolled: {r:?} -> {r2:?}");
    assert!((h2[1] - h[1]).abs() < 0.5, "the header stayed: {h:?} -> {h2:?}");
}

// ---- Carousel ----

#[test]
fn a_carousel_steps_by_arrows_dots_and_keys_and_keeps_its_place() {
    let mut rt = item("carousel");
    let before = layout(&rt);
    let v = drawn(&rt, "carousel-demo-viewport");
    press(&mut rt, "carousel-demo-next");
    assert_eq!(rt.state.g.index("carousel", 0), 1, "Next");
    let s1 = drawn(&rt, "carousel-demo-slide-1");
    assert!((s1[0] - v[0]).abs() < 0.5, "the track slid slide 2 into the viewport: {s1:?} vs {v:?}");
    press(&mut rt, "carousel-demo-dot-4");
    assert_eq!(rt.state.g.index("carousel", 0), 4, "a dot");
    let next = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("carousel-demo-next")).expect("next");
    assert!(!next.focusable, "Next is off on the last slide");
    focus(&mut rt, "carousel-demo-viewport");
    key(&mut rt, Key::Left);
    assert_eq!(rt.state.g.index("carousel", 0), 3, "Left steps back");
    key(&mut rt, Key::Home);
    assert_eq!(rt.state.g.index("carousel", 0), 0, "Home");
    focus(&mut rt, "carousel-loop-viewport");
    key(&mut rt, Key::Right);
    assert_eq!(rt.state.g.index("carousel-loop", 2), 0, "a looping carousel wraps");
    let hidden = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("carousel-demo-slide-2"));
    assert!(hidden.is_none_or(|e| !e.focusable), "an off-screen slide takes no focus");
    for (path, a) in layout(&rt) {
        if path.contains("-track") || path.contains("-slide-") {
            continue;
        }
        if let Some(b) = before.get(&path) {
            assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01), "stepping moved {path}");
        }
    }
}

// ---- Chart ----

#[test]
fn a_chart_shows_a_tooltip_where_the_pointer_rests_and_follows_the_keys() {
    let mut rt = item("chart");
    let before = layout(&rt);
    let p = drawn(&rt, "chart-bar-plot");
    rt.pointer_move(p[0] + p[2] * (3.5 / 6.0), p[1] + p[3] / 2.0);
    settle(&mut rt);
    assert_eq!(rt.state.g.choice("chart-bar", ""), "3", "the pointer picks April");
    let tip = drawn(&rt, "chart-bar-tooltip");
    let a = drawn(&rt, "chart-bar-anchor-3");
    assert!(tip[0] >= a[0] && tip[0] - a[0] < 24.0, "the tooltip sits beside April: {tip:?} vs {a:?}");
    let label = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("chart-bar-tooltip")).and_then(|e| e.label).unwrap_or_default();
    assert!(label.contains("Desktop 341") && label.contains("Mobile 202"), "it names each value: {label}");
    unmoved(&before, &rt, "hovering a chart");
    rt.pointer_move(W - 10.0, H - 10.0);
    settle(&mut rt);
    assert!(rt.presented("chart-bar-tooltip").is_none(), "it goes when the pointer leaves");
    focus(&mut rt, "chart-line-plot");
    key(&mut rt, Key::Right);
    key(&mut rt, Key::Right);
    assert_eq!(rt.state.g.choice("chart-line", ""), "1", "Right moves the highlight");
    assert!(rt.presented("chart-line-tooltip").is_some());
    key(&mut rt, Key::Escape);
    assert!(rt.presented("chart-line-tooltip").is_none(), "Escape clears it");
    // The donut picks the slice under the pointer: Free starts at the top and runs clockwise.
    let d = drawn(&rt, "chart-donut-plot");
    rt.pointer_move(d[0] + d[2] * 0.85, d[1] + d[3] * 0.5);
    settle(&mut rt);
    assert_eq!(rt.state.g.choice("chart-donut", ""), "0", "the right side of the ring is Free");
    rt.pointer_move(d[0] + d[2] * 0.5, d[1] + d[3] * 0.5);
    settle(&mut rt);
    assert!(rt.presented("chart-donut-tooltip").is_none(), "the hole is no slice");
    let fig = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("chart-area-plot")).and_then(|e| e.label).unwrap_or_default();
    assert!(fig.contains("Jun: Revenue $2960"), "the figure's name lists its values: {fig}");
}

// ---- Item, InputGroup ----

#[test]
fn a_pressable_item_presses_and_an_input_group_takes_typing_beside_its_buttons() {
    let mut rt = item("item");
    press(&mut rt, "item-small");
    assert_eq!(rt.state.g.presses, 1, "a pressable item runs its press");
    let mut rt = item("input_group");
    press(&mut rt, "group-send-input");
    for ch in "hi".chars() {
        key(&mut rt, Key::Text(ch.to_string()));
    }
    assert_eq!(rt.state.g.edit("group-send").text, "hi", "the field inside takes typing");
    key(&mut rt, Key::Tab);
    assert_eq!(focused(&rt).as_deref(), Some("group-send-go"), "Tab reaches the button inside the well");
}
