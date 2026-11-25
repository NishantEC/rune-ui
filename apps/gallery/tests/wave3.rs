//! Wave 3 in the live path (overlays at the root), driven by real presses, pointer moves, the wheel, pastes and keys: Reduce Motion,
//! the gallery at a narrow width, InputOTP's paste, and the wave 3 items.

use std::collections::HashMap;

use gallery::{App, apply_scene, set_headless, view};
use gallery::ui;
use rune::{Cx, El, Env, Key, KeyEvent, Mods, Runtime, col};
use rune_ui_style::Theme;

const W: f32 = 1320.0;
const H: f32 = 900.0;

fn settle<S: 'static>(rt: &mut Runtime<S>) {
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

fn layout<S: 'static>(rt: &Runtime<S>) -> HashMap<String, [f32; 4]> {
    rt.elements().into_iter().map(|e| (e.path, e.rect)).collect()
}

fn unmoved<S: 'static>(before: &HashMap<String, [f32; 4]>, rt: &Runtime<S>, what: &str) {
    for (path, a) in layout(rt) {
        if let Some(b) = before.get(&path) {
            assert!(a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01), "{what} moved {path}: {b:?} -> {a:?}");
        }
    }
}

fn drawn<S: 'static>(rt: &Runtime<S>, key: &str) -> [f32; 4] {
    rt.presented(key).unwrap_or_else(|| panic!("{key} is not drawn")).0
}

fn press<S: 'static>(rt: &mut Runtime<S>, key: &str) {
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("no {key} to press"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    settle(rt);
}

fn key_with<S: 'static>(rt: &mut Runtime<S>, k: Key, mods: Mods) {
    rt.key(KeyEvent { key: k, mods });
    settle(rt);
}

fn key<S: 'static>(rt: &mut Runtime<S>, k: Key) {
    key_with(rt, k, Mods::default());
}

fn shift<S: 'static>(rt: &mut Runtime<S>, k: Key) {
    key_with(rt, k, Mods { shift: true, ..Mods::default() });
}

fn typed<S: 'static>(rt: &mut Runtime<S>, words: &str) {
    for ch in words.chars() {
        key(rt, Key::Text(ch.to_string()));
    }
}

fn focused<S: 'static>(rt: &Runtime<S>) -> Option<String> {
    rt.elements().into_iter().find(|e| e.focused).and_then(|e| e.key)
}

fn focus<S: 'static>(rt: &mut Runtime<S>, k: &str) {
    let k = k.to_owned();
    rt.update(move |_, tx| tx.focus(k, true));
    settle(rt);
}

fn info<S: 'static>(rt: &Runtime<S>, key: &str) -> rune::ElementInfo {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("no {key}"))
}

// ---- Reduce Motion ----

/// Every moving item at once, in a theme with Reduce Motion on or off, and its overlays opened by `open`.
#[derive(Default)]
struct Motion {
    still: bool,
    open: bool,
}

fn motion_view(s: &Motion, _: &mut Cx<Motion>) -> El {
    let t = Theme::from_json(rune_ui_themes::DEFAULT, false).unwrap().with_reduced_motion(s.still);
    let mut layer = Vec::new();
    let page = col().key("page").w_full().h_full().gap(16.0).padding(24.0)
        .child(ui::shimmer::Shimmer::new("m-shimmer", "Working…").build(&t))
        .child(ui::shimmer::Loader::new("Waiting").build(&t))
        .child(ui::spinner::Spinner::new("Loading").build(&t))
        .child(ui::skeleton::Skeleton::lines(&t, "m-skeleton", 3, 200.0))
        .child(ui::response::Response::new("m-response", "Words that arrive while the agent answers").streaming(true).build(&t))
        .child(ui::reasoning::Reasoning::<Motion>::new("m-thinking", true, 3.0, false).build(&t))
        .child(ui::button::Button::<Motion>::new("m-trigger", "Menu").on_press(|s: &mut Motion| s.open = true).build(&t))
        .child(ui::segmented_control::SegmentedControl::<Motion>::new("m-seg", "View", &["Day", "Week", "Month"], usize::from(s.open)).build(&t));
    if s.open {
        let mut p = ui::popup::Popup::<Motion>::new("m-menu", "menu", "Menu");
        p.entries.push(ui::popup::Entry::Row(ui::popup::Row::new("Rename")));
        let close = ui::base::press(|s: &mut Motion| s.open = false);
        let surface = p.surface(&t, close.clone());
        ui::popup::mount(&t, &mut layer, surface, ui::popup::Mount { key: "m-menu", anchor: "m-trigger", side: rune::Side::Bottom, return_to: "m-trigger" }, close);
        ui::drawer::Drawer::<Motion>::new("m-drawer", "Filters").open(true).on_close(|s: &mut Motion| s.open = false).build(&t, &mut layer);
        ui::action_bar::ActionBar::<Motion>::new("m-bar", 2).action("Archive", None, |_: &mut Motion| {}).on_clear(|s: &mut Motion| s.open = false).build(&t, &mut layer);
    }
    ui::base::overlay_root(page, layer)
}

fn motion(still: bool) -> Runtime<Motion> {
    let mut rt = Runtime::new(Motion { still, open: false }, motion_view, (W, H), Env { system_dark: false, scale: 1.0 });
    settle(&mut rt);
    rt
}

/// Elements that loop (a sweep, a turning arc, dots), are fading words in, or are on their way in.
fn moving<S: 'static>(rt: &Runtime<S>) -> Vec<String> {
    rt.elements().into_iter().filter(|e| e.loop_phase.is_some() || e.revealing).map(|e| e.key.unwrap_or(e.path)).collect()
}

#[test]
fn a_theme_with_reduced_motion_draws_every_moving_item_still() {
    let mut rt = motion(true);
    assert!(moving(&rt).is_empty(), "under Reduce Motion nothing loops or fades in: {:?}", moving(&rt));
    assert!(info(&rt, "m-shimmer").label.is_some(), "the shimmer still names its work");
    // Opening overlays: on the very first frame each is where it ends at its full size (no grow, no slide). A short cross-fade is
    // allowed, as Reduce Motion itself keeps one.
    let (x, y) = rt.find("m-trigger").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    // Rune's element list does not report a presence transition's transform, so the overlays' still entry is also held by the
    // registry test `entry_motion_is_off_under_reduced_motion`; here: a menu under the flag fades on Rune's default only, slower
    // than the grow it would otherwise have.
    let still = info(&rt, "m-menu-content").opacity;
    let mut moving = motion(false);
    let (x, y) = moving.find("m-trigger").unwrap();
    moving.pointer_down(x, y);
    moving.pointer_up(x, y);
    moving.frame(1.0 / 60.0);
    assert!(still < info(&moving, "m-menu-content").opacity, "the menu does not run its grow-in under the flag ({still})");
    for k in ["m-menu-content", "m-drawer-drawer", "m-bar"] {
        let e = info(&rt, k);
        assert!((e.scale - 1.0).abs() < 0.01, "{k} appears at its size: scale {}", e.scale);
        assert!(e.rect.iter().zip(e.target.iter()).all(|(a, b)| (a - b).abs() < 0.5), "{k} does not travel: {:?} -> {:?}", e.rect, e.target);
    }
    // The segmented control's thumb lands at once when the choice changes.
    settle(&mut rt);
    let thumb = info(&rt, "m-seg-thumb");
    assert!(thumb.rect.iter().zip(thumb.target.iter()).all(|(a, b)| (a - b).abs() < 0.5), "the thumb is where it ends: {:?} -> {:?}", thumb.rect, thumb.target);
}

#[test]
fn without_reduced_motion_the_same_items_do_move() {
    // The control for the test above: the same view, motion on, shows motion, so the check is not vacuous.
    let mut rt = motion(false);
    assert!(!moving(&rt).is_empty(), "the shimmer, the dots and the spinner loop");
    let (x, y) = rt.find("m-trigger").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let e = info(&rt, "m-menu-content");
    assert!(e.opacity < 0.99 || e.scale < 0.99, "a menu grows in: opacity {} scale {}", e.opacity, e.scale);
    let e = info(&rt, "m-seg-thumb");
    assert!(e.rect.iter().zip(e.target.iter()).any(|(a, b)| (a - b).abs() > 0.5), "the thumb slides: {:?} -> {:?}", e.rect, e.target);
}


// ---- The gallery in a narrow region ----

fn window(w: f32, h: f32, scene: &str) -> Runtime<App> {
    set_headless(false);
    let mut s = App::default();
    apply_scene(&mut s, scene);
    let mut rt = Runtime::new(s, view, (w, h), Env { system_dark: false, scale: 1.0 });
    settle(&mut rt);
    rt
}

/// A text element is whole: its laid-out words fit its box and nothing above it cuts the box, or it truncates on purpose.
fn whole(e: &rune::ElementInfo) -> Result<(), String> {
    let Some([tw, th]) = e.text_size else { return Ok(()) };
    let [_, _, w, h] = e.rect;
    let cut_by_box = tw > w + 0.5 || th > h + 0.5;
    let cut_by_parent = e.visible[2] + 0.5 < w.min(tw) || e.visible[3] + 0.5 < h.min(th);
    if (cut_by_box || cut_by_parent) && !(e.truncate && !cut_by_parent) {
        return Err(format!("{:?} {:?}: text {tw}x{th} in {w}x{h}, visible {:?}", e.key, e.text, e.visible));
    }
    Ok(())
}

/// The gallery page alone, from a theme file whose name is long (an app's own theme), in a window `w` wide.
struct Long {
    g: ui::gallery::GalleryState,
}

fn long_view(s: &Long, cx: &mut Cx<Long>) -> El {
    thread_local! {
        static JSON: String = rune_ui_themes::DEFAULT.replacen("\"name\": \"", "\"name\": \"Placeholder Workspace Evening Long Name ", 1);
        static ITEMS: Vec<ui::gallery::StoryItem<Long>> = ui::stories::all();
    }
    let mut layer = Vec::new();
    let page = JSON.with(|json| ITEMS.with(|items| ui::gallery::page(json, ui::gallery::Appearance::of(cx.system_dark(), false), &s.g, |s: &mut Long| &mut s.g, items, &mut layer)));
    ui::base::overlay_root(page, layer)
}

#[test]
fn a_long_theme_name_wraps_in_the_header_at_any_width() {
    for w in (560..=1320).step_by(40) {
        let mut g = ui::gallery::GalleryState::default();
        g.show = "agent".to_owned();
        let mut rt = Runtime::new(Long { g }, long_view, (w as f32, 760.0), Env { system_dark: false, scale: 1.0 });
        settle(&mut rt);
        let e = info(&rt, "gallery-summary");
        assert!(e.text.as_deref().is_some_and(|t| t.contains("Long Name")), "the long name is shown: {:?}", e.text);
        whole(&e).unwrap_or_else(|m| panic!("at {w} wide: {m}"));
        let region = info(&rt, "gallery-scroll").visible;
        assert!(e.rect[0] + e.rect[2] <= region[0] + region[2] + 0.5, "at {w} wide the summary runs past its region: {:?} vs {region:?}", e.rect);
    }
}

#[test]
fn the_gallery_header_wraps_or_truncates_cleanly_at_any_width() {
    for w in [760.0, 1180.0] {
        let rt = window(w, 760.0, "agent");
        let e = info(&rt, "gallery-summary");
        whole(&e).unwrap_or_else(|m| panic!("at {w} wide: {m}"));
        let region = info(&rt, "gallery-scroll").visible;
        assert!(e.rect[0] + e.rect[2] <= region[0] + region[2] + 0.5, "at {w} wide the summary runs past its region: {:?} vs {region:?}", e.rect);
    }
}

#[test]
fn the_chat_block_fits_a_narrow_region_at_1180_by_760() {
    for (scene, w) in [("item-chat", 1180.0), ("chat-open", 1180.0), ("item-chat", 980.0)] {
        let rt = window(w, 760.0, scene);
        // The content column the page gives each item: the block must fit it, not run into the page's padding or past its edge.
        let column = info(&rt, "item-chat").rect;
        let chat = info(&rt, "chat");
        assert!(chat.rect[0] + chat.rect[2] <= column[0] + column[2] + 0.5, "{scene} at {w}: the chat block is wider than the page's column: {:?} vs {column:?}", chat.rect);
        let mut cut = Vec::new();
        for e in rt.elements() {
            // Words in the chat's own column (the conversation scrolls, so judge what is on screen sideways only).
            let inside = e.rect[0] >= chat.rect[0] - 0.5 && e.rect[1] >= chat.rect[1] - 0.5 && e.rect[1] + e.rect[3] <= chat.rect[1] + chat.rect[3] + 0.5;
            if inside && e.text_size.is_some_and(|[tw, _]| tw > e.rect[2] + 0.5 && !e.truncate) {
                cut.push(format!("{:?} {:?}", e.key, e.text));
            }
            if inside && e.text_size.is_some() && e.visible[3] > 0.0 && e.visible[2] + 0.5 < e.rect[2].min(e.text_size.unwrap()[0]) {
                cut.push(format!("clipped sideways: {:?} {:?} {:?} {:?}", e.key, e.text, e.rect, e.visible));
            }
        }
        assert!(cut.is_empty(), "{scene}: words cut at {w} by 760:\n{}", cut.join("\n"));
    }
}

// ---- InputOTP ----

fn caret<S: 'static>(rt: &Runtime<S>, key: &str) -> Option<[usize; 2]> {
    info(rt, key).edit_range
}

#[test]
fn pasting_a_code_into_an_otp_input_fills_its_slots() {
    let mut rt = item("input_otp");
    press(&mut rt, "otp-demo-slot-3");
    assert_eq!(focused(&rt).as_deref(), Some("otp-demo-input"), "a press on a slot focuses the code's one text field");
    rt.paste("98-76 54");
    settle(&mut rt);
    assert_eq!(rt.state.g.edit("otp").text, "987654", "the pasted code fills every slot; the dash and space are dropped");
    assert_eq!(rt.state.g.presses, 1, "and it is complete");
    for (i, d) in "987654".chars().enumerate() {
        assert_eq!(info(&rt, &format!("otp-demo-slot-{i}-char")).text.as_deref(), Some(d.to_string().as_str()), "slot {i} shows its digit");
    }
    // A code longer than the slots keeps what fits; letters are refused by a digits-only field.
    rt.update(|s, _| *s.g.edit_mut("otp") = rune::TextEdit::new(""));
    settle(&mut rt);
    focus(&mut rt, "otp-demo-input");
    rt.paste("1a2b3c4d5e6f7g8");
    settle(&mut rt);
    assert_eq!(rt.state.g.edit("otp").text, "123456");
    // Pasting into the letters field takes letters, upper-cased by the story's change handler.
    focus(&mut rt, "otp-letters-input");
    rt.paste("ab cd ef gh");
    settle(&mut rt);
    assert_eq!(rt.state.g.choice("otp-words", ""), "ABCDEFGH");
}

#[test]
fn an_otp_input_types_replaces_moves_and_backspaces_in_one_field() {
    let mut rt = item("input_otp");
    focus(&mut rt, "otp-demo-input");
    key(&mut rt, Key::End);
    key(&mut rt, Key::Text("x".to_owned()));
    assert_eq!(rt.state.g.edit("otp").text, "12", "a letter is refused");
    typed(&mut rt, "3456");
    assert_eq!(rt.state.g.edit("otp").text, "123456");
    assert_eq!(rt.state.g.presses, 1, "complete is heard once");
    typed(&mut rt, "7");
    assert_eq!(rt.state.g.edit("otp").text, "123456", "a full code takes no more");
    key(&mut rt, Key::Backspace);
    assert_eq!(rt.state.g.edit("otp").text, "12345", "Backspace empties the last slot");
    key(&mut rt, Key::Left);
    key(&mut rt, Key::Left);
    assert_eq!(caret(&rt, "otp-demo-input"), Some([3, 3]), "Left moves between slots");
    key(&mut rt, Key::Text("9".to_owned()));
    assert_eq!(rt.state.g.edit("otp").text, "12395", "typing on a filled slot replaces it");
    assert_eq!(caret(&rt, "otp-demo-input"), Some([4, 4]), "and moves on");
    key(&mut rt, Key::Tab);
    assert_ne!(focused(&rt).as_deref(), Some("otp-demo-input"), "the code is one Tab stop");
}

// ---- SegmentedControl ----

#[test]
fn a_segmented_control_slides_its_thumb_and_turns_it_round_mid_flight() {
    let mut rt = item("segmented_control");
    let at = |rt: &Runtime<App>, i: usize| drawn(rt, &format!("seg-demo-{i}"));
    let thumb = |rt: &Runtime<App>| info(rt, "seg-demo-thumb");
    assert!((thumb(&rt).rect[0] - at(&rt, 1)[0]).abs() < 0.5, "the thumb starts under the chosen segment");
    let (x, y) = rt.find("seg-demo-2").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    for _ in 0..4 {
        rt.frame(1.0 / 60.0);
    }
    assert_eq!(rt.state.g.index("seg-demo", 9), 2, "a press chooses");
    let mid = thumb(&rt).rect[0];
    assert!(mid > at(&rt, 1)[0] + 0.5 && mid < at(&rt, 2)[0] - 0.5, "on its way: {mid} between {} and {}", at(&rt, 1)[0], at(&rt, 2)[0]);
    // A second choice mid-flight: the thumb turns round from where it is, never jumping.
    let (x, y) = rt.find("seg-demo-0").unwrap();
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    rt.frame(1.0 / 60.0);
    let next = thumb(&rt).rect[0];
    assert!((next - mid).abs() < 40.0, "it retargets from {mid}, not from a segment: {next}");
    settle(&mut rt);
    assert!((thumb(&rt).rect[0] - at(&rt, 0)[0]).abs() < 0.5, "and lands under the new choice");
}

#[test]
fn a_segmented_control_is_one_tab_stop_and_the_arrows_choose() {
    let mut rt = item("segmented_control");
    focus(&mut rt, "gallery-group-actions");
    let mut seen = Vec::new();
    for _ in 0..12 {
        key(&mut rt, Key::Tab);
        let f = focused(&rt).unwrap_or_default();
        if f.starts_with("seg-demo") {
            seen.push(f);
        } else if !seen.is_empty() {
            break;
        }
    }
    assert_eq!(seen, vec!["seg-demo-1".to_owned()], "Tab lands once, on the chosen segment");
    focus(&mut rt, "seg-demo-1");
    key(&mut rt, Key::Right);
    assert_eq!((rt.state.g.index("seg-demo", 9), focused(&rt).as_deref()), (2, Some("seg-demo-2")), "Right chooses the next");
    key(&mut rt, Key::Right);
    assert_eq!(rt.state.g.index("seg-demo", 9), 0, "and wraps");
    key(&mut rt, Key::End);
    assert_eq!(rt.state.g.index("seg-demo", 9), 2, "End is the last");
    key(&mut rt, Key::Left);
    key(&mut rt, Key::Home);
    assert_eq!((rt.state.g.index("seg-demo", 9), focused(&rt).as_deref()), (0, Some("seg-demo-0")), "Home is the first");
}

// ---- Toolbar ----

#[test]
fn a_toolbar_is_one_tab_stop_with_arrow_keys_inside() {
    let mut rt = item("toolbar");
    focus(&mut rt, "toolbar-demo-0");
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("toolbar-demo-1"), "Right moves");
    key(&mut rt, Key::End);
    let last = focused(&rt).unwrap_or_default();
    assert!(last == "toolbar-demo-7", "End goes to the last control that can be used (Settings is disabled): {last}");
    key(&mut rt, Key::Home);
    assert_eq!(focused(&rt).as_deref(), Some("toolbar-demo-0"), "Home goes to the first");
    key(&mut rt, Key::Text(" ".to_owned()));
    assert!(rt.state.g.flag("tb-bold"), "Space presses the focused toggle");
    key(&mut rt, Key::Tab);
    assert!(!focused(&rt).unwrap_or_default().starts_with("toolbar-demo-"), "Tab leaves the toolbar: {:?}", focused(&rt));
}

#[test]
fn a_narrow_toolbar_moves_what_does_not_fit_into_more() {
    let mut rt = item("toolbar");
    assert!(rt.presented("toolbar-narrow-more").is_some(), "the narrow toolbar has a More button");
    assert!(rt.presented("toolbar-narrow-0").is_some() && rt.presented("toolbar-narrow-8").is_none(), "the first controls stay, the last go");
    let bar = drawn(&rt, "toolbar-narrow");
    let more = drawn(&rt, "toolbar-narrow-more");
    assert!(more[0] + more[2] <= bar[0] + bar[2] + 0.5, "More fits inside the bar: {more:?} in {bar:?}");
    let before = layout(&rt);
    press(&mut rt, "toolbar-narrow-more");
    unmoved(&before, &rt, "opening More");
    let menu = drawn(&rt, "toolbar-narrow-more-content");
    assert!(menu[1] > more[1] + more[3] - 0.5, "the menu opens under More: {menu:?} vs {more:?}");
    assert!(rt.presented("toolbar-narrow-more-6").is_some(), "Undo is in the menu");
    focus(&mut rt, "toolbar-narrow-more-6");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("toolbar-did", ""), "undo", "a row does what its control did");
    assert!(rt.presented("toolbar-narrow-more-content").is_none(), "and closes");
    press(&mut rt, "toolbar-narrow-more");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("toolbar-narrow-more-content").is_none(), "Escape closes");
    assert_eq!(focused(&rt).as_deref(), Some("toolbar-narrow-more"), "focus back on More");
}

// ---- ActionBar ----

#[test]
fn an_action_bar_follows_a_table_selection_and_escape_clears_it() {
    let mut rt = item("action_bar");
    let bar = drawn(&rt, "bulk");
    assert!(bar[1] + bar[3] <= H - 8.0 && bar[1] > H - 120.0, "at the bottom of the window: {bar:?}");
    assert!(((bar[0] + bar[2] / 2.0) - W / 2.0).abs() < 1.0, "centred: {bar:?}");
    assert_eq!(info(&rt, "bulk-count").text.as_deref(), Some("2 runs selected"));
    // The page under the bar still takes presses: choosing a third row updates the count.
    let before = layout(&rt);
    press(&mut rt, "bulk-table-check-1");
    assert_eq!(info(&rt, "bulk-count").text.as_deref(), Some("3 runs selected"));
    unmoved(&before, &rt, "a change of count");
    focus(&mut rt, "bulk-action-0");
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("bulk-action-1"), "Right moves inside the bar");
    key(&mut rt, Key::End);
    assert_eq!(focused(&rt).as_deref(), Some("bulk-clear"), "End reaches Clear");
    key(&mut rt, Key::Enter);
    key(&mut rt, Key::Escape);
    assert!(rt.state.g.slot::<ui::data_table::DataTableState>("bulk-table").selected.is_empty(), "Escape clears the selection");
    assert!(rt.presented("bulk").is_none(), "and the bar goes");
    assert_eq!(focused(&rt).as_deref(), Some("bulk-table-select-all"), "focus goes back to the table");
    press(&mut rt, "bulk-table-check-4");
    assert!(rt.presented("bulk").is_some(), "choosing again brings it back");
    press(&mut rt, "bulk-action-1");
    assert_eq!(rt.state.g.choice("bulk-did", ""), "archived");
    assert!(rt.presented("bulk").is_none(), "an action that empties the selection removes the bar");
}

// ---- Tree and FileTree ----

fn tree_state(rt: &Runtime<App>, key: &str) -> ui::tree::TreeState {
    rt.state.g.slot(key)
}

#[test]
fn a_tree_moves_opens_closes_and_jumps_by_keys() {
    let mut rt = item("tree");
    focus(&mut rt, "gallery-group-actions");
    let mut landed = None;
    for _ in 0..12 {
        key(&mut rt, Key::Tab);
        if let Some(f) = focused(&rt).filter(|f| f.starts_with("tree-demo-row-")) {
            landed = Some(f);
            break;
        }
    }
    assert_eq!(landed.as_deref(), Some("tree-demo-row-budget"), "Tab lands on the tree's own row");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-launch"));
    assert!(tree_state(&rt, "tree-demo").selected.contains("launch"), "with one choice, the choice follows the keys");
    key(&mut rt, Key::Right);
    assert!(tree_state(&rt, "tree-demo").expanded.contains("launch"), "Right opens a closed branch");
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-launch"), "and stays on it");
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-checklist"), "Right again goes to its first child");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-people"), "Down skips a disabled row");
    key(&mut rt, Key::Up);
    key(&mut rt, Key::Left);
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-launch"), "Left goes to the parent");
    key(&mut rt, Key::Left);
    assert!(!tree_state(&rt, "tree-demo").expanded.contains("launch"), "Left closes an open branch");
    key(&mut rt, Key::End);
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-notes"), "End is the last row");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-notes"), "Down stops at the end");
    key(&mut rt, Key::Home);
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-plans"), "Home is the first row");
    key(&mut rt, Key::Text("p".to_owned()));
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-people"), "typing a letter jumps to the next row starting with it");
    key(&mut rt, Key::Text("n".to_owned()));
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("tree-opened", ""), "notes", "Return opens a leaf");
    // A lazy branch asks for its children and shows a loading row until they come.
    focus(&mut rt, "tree-demo-row-archive");
    key(&mut rt, Key::Right);
    assert_eq!(rt.state.g.choice("tree-asked", ""), "archive", "opening a lazy branch asks the app");
    assert!(rt.presented("tree-demo-row-archive-loading").is_some(), "and shows that it is loading");
    rt.update(|s, _| s.g.toggle("tree-loaded"));
    settle(&mut rt);
    assert!(rt.presented("tree-demo-row-q1").is_some() && rt.presented("tree-demo-row-archive-loading").is_none(), "the children arrive");
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("tree-demo-row-q1"));
}

#[test]
fn a_tree_with_several_choices_keeps_focus_and_choice_apart() {
    let mut rt = item("tree");
    focus(&mut rt, "tree-multi-row-plans");
    key(&mut rt, Key::Down);
    assert_eq!(focused(&rt).as_deref(), Some("tree-multi-row-roadmap"));
    assert!(tree_state(&rt, "tree-multi").selected.is_empty(), "moving does not choose");
    assert!(info(&rt, "tree-multi-row-roadmap").focused && !tree_state(&rt, "tree-multi").selected.contains("roadmap"), "focused and not chosen");
    key(&mut rt, Key::Text(" ".to_owned()));
    assert_eq!(tree_state(&rt, "tree-multi").selected.iter().cloned().collect::<Vec<_>>(), vec!["roadmap".to_owned()], "Space chooses");
    shift(&mut rt, Key::Down);
    shift(&mut rt, Key::Down);
    let chosen: Vec<String> = tree_state(&rt, "tree-multi").selected.iter().cloned().collect();
    assert_eq!(chosen, vec!["budget".to_owned(), "launch".to_owned(), "roadmap".to_owned()], "Shift and Down extend");
    key(&mut rt, Key::Text(" ".to_owned()));
    assert!(!tree_state(&rt, "tree-multi").selected.contains("launch"), "Space unchooses");
    press(&mut rt, "tree-multi-row-notes");
    assert!(tree_state(&rt, "tree-multi").selected.contains("notes") && tree_state(&rt, "tree-multi").selected.contains("roadmap"), "a press flips one row");
}

#[test]
fn a_file_tree_sorts_folders_first_in_natural_order_and_opens_a_file() {
    let mut rt = item("file_tree");
    let order: Vec<String> = rt.elements().into_iter().filter_map(|e| e.key).filter_map(|k| k.strip_prefix("file-tree-row-").map(str::to_owned)).collect();
    assert_eq!(order, ["docs", "src", "src/ui", "src/ui/button.rs", "src/ui/file2.rs", "src/ui/file10.rs", "src/ui/tree.rs", "src/main.rs", ".gitignore", "Cargo.toml"], "folders first, then natural order");
    focus(&mut rt, "file-tree-row-src/ui/tree.rs");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("file-opened", ""), "src/ui/tree.rs", "Return hands the path to the app");
    focus(&mut rt, "file-tree-row-docs");
    key(&mut rt, Key::Right);
    key(&mut rt, Key::Right);
    assert_eq!(focused(&rt).as_deref(), Some("file-tree-row-docs/screens"), "into the folder");
}

// ---- StackTrace, TestResults, Commit ----

#[test]
fn a_stack_trace_hands_a_pressed_frame_to_the_app_and_unfolds_library_frames() {
    let mut rt = item("stack_trace");
    assert!(rt.presented("trace-demo-frame-2").is_none() && rt.presented("trace-demo-library-2").is_some(), "library frames start folded");
    press(&mut rt, "trace-demo-frame-1");
    assert_eq!(rt.state.g.choice("trace-opened", ""), "src/ui/invoices.rs:118:9", "path, line and column");
    focus(&mut rt, "trace-demo-frame-5");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("trace-opened", ""), "src/main.rs:77:5", "Return on a frame");
    press(&mut rt, "trace-demo-library-2");
    assert!(rt.presented("trace-demo-frame-2").is_some() && rt.presented("trace-demo-frame-7").is_some(), "the library frames show");
    press(&mut rt, "trace-demo-frame-3");
    assert_eq!(rt.state.g.choice("trace-opened", ""), "library/core/src/iter/adapters/map.rs:128:19", "a library frame hands its place too");
    press(&mut rt, "trace-demo-library-hide");
    assert!(rt.presented("trace-demo-frame-2").is_none(), "and fold away again");
    let clip = rune::clipboard::memory();
    rt.use_clipboard(&clip);
    press(&mut rt, "trace-demo-copy");
    let copied = clip.text().map(|t| t.to_string()).unwrap_or_default();
    assert!(copied.starts_with("panicked:") && copied.contains("at invoice_total (src/billing/invoice.rs:42:17)"), "Copy puts the trace on the clipboard: {copied}");
    assert!(rt.state.g.flag("trace-copied"));
}

#[test]
fn test_results_put_failures_first_and_fold_by_suite() {
    let mut rt = item("test_results");
    assert_eq!(info(&rt, "tests-demo-summary").label.as_deref(), Some("1 failed, 5 passed, 1 skipped"));
    assert_eq!(info(&rt, "tests-demo-suite-0-trigger").label.as_deref(), Some("billing::invoice"), "the failed suite comes first");
    assert!(rt.presented("tests-demo-suite-0-test-0-message").is_some(), "open, with its failed test first and its message");
    assert!(rt.presented("tests-demo-suite-0-test-0-expected").is_some(), "and expected and received");
    press(&mut rt, "tests-demo-suite-0-trigger");
    assert!(rt.presented("tests-demo-suite-0-test-0").is_none(), "a press folds the suite");
    focus(&mut rt, "tests-demo-suite-1-trigger");
    key(&mut rt, Key::Enter);
    assert!(rt.presented("tests-demo-suite-1-test-0").is_some(), "Return opens another");
}

#[test]
fn a_commit_folds_to_its_files_and_diff() {
    let mut rt = item("commit");
    assert!(rt.presented("commit-demo-files").is_none());
    press(&mut rt, "commit-demo-trigger");
    assert!(rt.presented("commit-demo-files").is_some() && rt.presented("commit-demo-diff-0").is_some(), "open: files and diff");
    press(&mut rt, "commit-demo-file-1");
    assert_eq!(rt.state.g.choice("commit-file", ""), "tests/invoice.rs");
    focus(&mut rt, "commit-demo-trigger");
    key(&mut rt, Key::Enter);
    assert!(rt.presented("commit-demo-files").is_none(), "Return folds it");
    let clip = rune::clipboard::memory();
    rt.use_clipboard(&clip);
    press(&mut rt, "commit-demo-copy");
    assert_eq!(clip.text().map(|t| t.to_string()).as_deref(), Some("3f9c2a71d04e8b6a5c1f0e9d8b7a6c5d4e3f2a10"), "Copy hash");
}

// ---- PromptInput: mentions, commands, growth ----

#[test]
fn a_mention_opens_over_the_composer_and_goes_in_as_one_token() {
    let mut rt = item("prompt_input");
    focus(&mut rt, "prompt-menu-input");
    let before = layout(&rt);
    typed(&mut rt, "ask @");
    let list = drawn(&rt, "prompt-menu-mentions");
    let composer = drawn(&rt, "prompt-menu");
    assert!(list[1] + list[3] <= composer[1] + 0.5 && (list[0] - composer[0]).abs() < 0.5, "over the composer, at its left: {list:?} vs {composer:?}");
    unmoved(&before, &rt, "opening the mention list");
    assert_eq!(focused(&rt).as_deref(), Some("prompt-menu-input"), "the caret stays in the text");
    assert!(rt.presented("prompt-menu-mentions-3").is_some(), "everything it offers, before a letter is typed");
    key(&mut rt, Key::Down);
    key(&mut rt, Key::Down);
    assert_eq!(rt.state.g.slot::<ui::prompt_input::MenuState>("prompt-menu-state").highlight, 2, "Down moves the highlight");
    key(&mut rt, Key::Up);
    assert_eq!(rt.state.g.slot::<ui::prompt_input::MenuState>("prompt-menu-state").highlight, 1, "Up moves it back");
    typed(&mut rt, "e");
    assert_eq!(info(&rt, "prompt-menu-mentions-0").label.as_deref(), Some("Lee"), "a letter filters: names that start with it, then names that hold it");
    assert_eq!(info(&rt, "prompt-menu-mentions-1").label.as_deref(), Some("notes.md"));
    assert_eq!(rt.state.g.slot::<ui::prompt_input::MenuState>("prompt-menu-state").highlight, 0, "typing puts the highlight back on the best match");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.edit("prompt-menu").text, "ask @Lee ", "Return puts the mention in");
    assert!(rt.state.g.choice("prompt-menu-sent", "").is_empty(), "and does not send");
    assert!(rt.presented("prompt-menu-mentions").is_none(), "the list closes");
    assert_eq!(rt.state.g.edit("prompt-menu").pills().map(|(_, id)| id.to_owned()).collect::<Vec<_>>(), vec!["person:lee".to_owned()], "as one token carrying its id");
    key(&mut rt, Key::Backspace);
    key(&mut rt, Key::Backspace);
    assert_eq!(rt.state.g.edit("prompt-menu").text, "ask ", "Backspace deletes the token whole");
    typed(&mut rt, "@zz");
    assert!(rt.presented("prompt-menu-mentions-none").is_some(), "no matches says so");
    key(&mut rt, Key::Escape);
    assert!(rt.presented("prompt-menu-mentions").is_none(), "Escape closes the list");
    assert_eq!(rt.state.g.edit("prompt-menu").text, "ask @zz", "and only the list");
    press(&mut rt, "prompt-menu-submit");
}

#[test]
fn a_slash_command_runs_and_inserts_nothing() {
    let mut rt = item("prompt_input");
    focus(&mut rt, "prompt-menu-input");
    typed(&mut rt, "/");
    assert!(rt.presented("prompt-menu-commands").is_some(), "a / at the start opens the commands");
    typed(&mut rt, "re");
    assert_eq!(info(&rt, "prompt-menu-commands-0").label.as_deref(), Some("review"));
    key(&mut rt, Key::Enter);
    assert_eq!(rt.state.g.choice("prompt-command", ""), "review", "Return runs the command");
    assert_eq!(rt.state.g.edit("prompt-menu").text, "", "and inserts nothing");
    typed(&mut rt, "/nope");
    key(&mut rt, Key::Enter);
    assert!(rt.state.g.choice("prompt-menu-sent", "").is_empty(), "Return with nothing to pick does not send /nope");
    assert_eq!(rt.state.g.edit("prompt-menu").text, "/nope");
    // A / that is not at the start opens nothing.
    rt.update(|s, _| *s.g.edit_mut("prompt-menu") = rune::TextEdit::new(""));
    settle(&mut rt);
    focus(&mut rt, "prompt-menu-input");
    typed(&mut rt, "a/b");
    assert!(rt.presented("prompt-menu-commands").is_none());
    // Without sources nothing opens: the plain composer is unchanged.
    focus(&mut rt, "prompt-demo-input");
    typed(&mut rt, "/x @y");
    assert!(rt.presented("prompt-demo-commands").is_none() && rt.presented("prompt-demo-mentions").is_none());
}

#[test]
fn the_composer_grows_to_its_cap_then_scrolls_with_the_toolbar_in_place() {
    let mut rt = item("prompt_input");
    focus(&mut rt, "prompt-menu-input");
    let one = info(&rt, "prompt-menu-input").visible[3];
    let mut heights = Vec::new();
    for _ in 0..7 {
        typed(&mut rt, "line");
        shift(&mut rt, Key::Enter);
        heights.push(info(&rt, "prompt-menu-input").visible[3]);
    }
    let bar = |rt: &Runtime<App>| drawn(rt, "prompt-menu-toolbar");
    assert!(heights[0] > one + 4.0, "a second line grows it: {one} -> {}", heights[0]);
    let cap = heights[2];
    assert!(heights[3..].iter().all(|h| (h - cap).abs() < 0.5), "four lines at most, then it holds: {heights:?}");
    let b = bar(&rt);
    typed(&mut rt, "more");
    assert!((bar(&rt)[1] - b[1]).abs() < 0.5, "the toolbar stays put while the text scrolls");
}
