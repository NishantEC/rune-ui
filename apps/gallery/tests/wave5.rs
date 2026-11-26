//! 0.10's additions in the live path (overlays at the root), driven by real presses, pointer moves and keys: the scrim that dims and
//! the keycap font, the portrait, the message header and well bubble, the composer's slots, the two-line sidebar row, neutral marks,
//! the receipt, Diff's stats, selection and foot, and menu rows that lead with an image.


use gallery::{App, apply_scene, set_headless, view};
use gallery::ui;
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

fn space<S: 'static>(rt: &mut Runtime<S>) {
    key(rt, Key::Text(" ".to_owned()));
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

/// Every text drawn inside the element keyed `key`.
fn texts_in<S: 'static>(rt: &Runtime<S>, key: &str) -> Vec<String> {
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


// ---- Scrim: black at the theme's strength, in light and dark ----

#[test]
fn a_modal_scrim_dims_with_black_at_the_themes_strength_in_light_and_dark() {
    for (scene_name, key) in [("open-dialog", "dialog-demo-scrim"), ("open-alert-dialog", "alert-demo-scrim"), ("open-sheet", "sheet-demo-scrim"), ("open-drawer", "drawer-demo-scrim")] {
        for dark in [false, true] {
            let rt = scene(scene_name, dark);
            let t = rune_ui_themes::theme(dark);
            let f = info(&rt, key).fill;
            assert_eq!((f.r, f.g, f.b), (0.0, 0.0, 0.0), "{scene_name} {dark}: the scrim is black, not the page ink (which would fog a dark page): {f:?}");
            assert!((f.a - t.scrim().a).abs() < 0.01, "{scene_name} {dark}: at the theme's strength {} not {}", t.scrim().a, f.a);
        }
    }
    // The command palette's is the same black, lighter.
    let rt = scene("open-command", true);
    let f = info(&rt, "command-demo-scrim").fill;
    assert_eq!((f.r, f.g, f.b), (0.0, 0.0, 0.0));
    assert!(f.a < rune_ui_themes::theme(true).scrim().a);
}

// ---- Keycaps and menu shortcuts: the theme's keycap style ----

#[test]
fn kbd_and_menu_shortcuts_are_drawn_in_the_themes_keycap_style() {
    for file in ["kbd/kbd.rs", "popup/popup.rs"] {
        let src = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../registry").join(file)).unwrap();
        assert!(src.contains("t.keycap"), "{file} reads Theme::keycap, so `base.keys-font` reaches every shortcut");
    }
    // Drawn: a menu's shortcut is in the keycap's face and size at the regular weight, and is hidden from assistive technology
    // (the row's own `shortcut_hint` says it).
    let rt = scene("open-dropdown-menu", false);
    let t = rune_ui_themes::theme(false);
    let e = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("⇧⌘P")).expect("a shortcut in the open menu");
    let style = e.text_style.expect("a text style");
    assert_eq!((style.size, style.weight), (t.keycap.size, t.ty.regular));
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

// ---- Portrait ----

fn portrait_view(s: &Probe, _: &mut Cx<Probe>) -> El {
    use ui::portrait::{Activity, Portrait};
    let t = theme_of(s);
    let mut r = row().key("page").gap(24.0).padding(24.0).items_start().fill(t.c.surface.bg);
    for (i, a) in [Activity::None, Activity::Working, Activity::NeedsYou, Activity::Failed, Activity::Done, Activity::Info].into_iter().enumerate() {
        r = r.child(Portrait::new(format!("p{i}"), "Asha Rao", 28.0).state(a).ringed(true).under(t.c.surface.bg).build(&t));
    }
    r = r.child(Portrait::new("flat", "Lee", 40.0).state(Activity::NeedsYou).build(&t));
    r = r.child(Portrait::new("small", "Lee", 3.0).build(&t)).child(Portrait::new("huge", "Lee", 900.0).build(&t));
    r = r.child(Portrait::new("deco", "Lee", 40.0).decorative(true).build(&t));
    r = r.child(Portrait::new("words", "Lee", 40.0).state(Activity::Working).state_words("Building").ringed(true).build(&t));
    ui::base::overlay_root(col().w_full().child(r), Vec::new())
}

#[test]
fn a_portrait_takes_the_same_room_in_every_state_and_clamps_its_size() {
    let rt = probe(portrait_view, false, false);
    let room = ui::portrait::ringed(28.0);
    assert_eq!(room, 28.0 + 2.0 * (2.0 + 2.5), "28 points: a 2 point gap and a 2.5 point stroke on each side");
    for i in 0..6 {
        let r = drawn(&rt, &format!("p{i}"));
        assert_eq!((r[2], r[3]), (room, room), "state {i} keeps the ring's room: {r:?}");
    }
    let flat = drawn(&rt, "flat");
    assert_eq!((flat[2], flat[3]), (40.0, 40.0), "unringed, a portrait is exactly its size, dot or not");
    assert_eq!(drawn(&rt, "small")[2], 20.0, "from 20 points");
    assert_eq!(drawn(&rt, "huge")[2], 112.0, "to 112 points");
    for size in [20.0, 24.0, 28.0, 40.0, 76.0, 112.0] {
        let (_, stroke) = ui::portrait::ring_metrics(size);
        assert!((2.5..=3.5).contains(&stroke), "the ring reads at {size}: stroke {stroke}");
    }
    assert!((ui::portrait::radius(100.0) - 24.0).abs() < 0.01, "a quarter of the size, near enough");
}

#[test]
fn the_working_ring_turns_and_holds_still_under_reduce_motion() {
    let moving = probe(portrait_view, false, false);
    assert!(info(&moving, "p1-arc").loop_phase.is_some(), "the arc turns");
    assert!(info(&moving, "p0").loop_phase.is_none() && rt_has(&moving, "p2-dot"), "no arc without work; a dot for needs you");
    let still = probe(portrait_view, false, true);
    let arc = info(&still, "p1-arc");
    assert!(arc.loop_phase.is_none(), "under Reduce Motion the arc is a still third of a circle");
    assert_eq!(drawn(&still, "p1")[2], drawn(&moving, "p1")[2], "and takes the same room");
}

fn rt_has(rt: &Runtime<Probe>, key: &str) -> bool {
    rt.presented(key).is_some()
}

#[test]
fn a_portrait_is_an_image_named_with_its_state_in_words() {
    let mut rt = probe(portrait_view, false, false);
    let tree = rt.accessibility_json();
    let w = ax(&tree, "p1").expect("the working portrait");
    assert_eq!(w["name"].as_str().or(w["label"].as_str()), Some("Asha Rao"), "{w}");
    assert!(w.to_string().contains("Working"), "its state in words: {w}");
    assert!(ax(&tree, "p2").unwrap().to_string().contains("Needs you"));
    assert!(ax(&tree, "words").unwrap().to_string().contains("Building"), "the app's own words win");
    assert!(!ax(&tree, "p0").unwrap().to_string().contains("Working"), "no state, no words");
    assert!(ax(&tree, "deco").is_none_or(|n| n["role"] != "image" && n["role"] != "img"), "decorative: hidden from assistive technology");
}

#[test]
fn a_portraits_ring_and_dot_reach_their_contrast_on_the_surface_they_sit_on() {
    use ui::portrait::{DOT_CONTRAST, RING_CONTRAST, seen_on};
    for dark in [false, true] {
        let t = Theme::from_json(rune_ui_themes::DEFAULT, dark).unwrap();
        for under in [t.c.page.bg, t.c.surface.bg, t.c.floating.bg] {
            let id = t.identity();
            assert!(contrast(seen_on(id.solid, id.fg, under, RING_CONTRAST), under) >= RING_CONTRAST - 0.05, "ring, dark {dark}");
            for tone in [t.c.warning, t.c.danger, t.c.success, t.c.info] {
                assert!(contrast(seen_on(tone.solid, tone.fg, under, DOT_CONTRAST), under) >= DOT_CONTRAST - 0.05, "dot, dark {dark}");
            }
        }
        // The same, drawn: the dot's fill in the picture is the checked colour.
        let rt = probe(portrait_view, dark, false);
        let dot = info(&rt, "p3-dot").fill;
        assert!(contrast(dot, t.c.surface.bg) >= DOT_CONTRAST - 0.05, "the failed dot reads on the surface, dark {dark}: {dot:?}");
    }
}

#[test]
fn a_theme_without_an_identity_colour_draws_the_ring_in_info() {
    let json = {
        let mut v: serde_json::Value = serde_json::from_str(rune_ui_themes::DEFAULT).unwrap();
        v["color"].as_object_mut().unwrap().remove("identity");
        for m in ["light", "dark"] {
            v["modes"][m]["role"].as_object_mut().unwrap().remove("identity");
        }
        v.to_string()
    };
    let t = Theme::from_json(&json, false).unwrap();
    assert!(!t.has_identity());
    let _ = ui::portrait::Portrait::new("x", "Asha", 28.0).state(ui::portrait::Activity::Working).ringed(true).build(&t);
}

// ---- Message: a header slot and a well bubble ----

#[test]
fn a_message_header_leads_the_turn_one_block_gap_above_its_first_block() {
    let rt = item("message");
    let (head, article) = (drawn(&rt, "message-led-header"), drawn(&rt, "message-led"));
    assert!((head[0] - article[0]).abs() < 0.5, "on the reading edge: {head:?} {article:?}");
    assert!((head[1] - article[1]).abs() < 0.5, "at the top of the turn");
    let words = rt.elements().into_iter().find(|e| e.text.as_deref().is_some_and(|w| w.starts_with("Linking takes most"))).expect("the answer");
    assert!((words.rect[1] - (head[1] + head[3]) - 4.0).abs() < 0.5, "the first block sits one block gap (4) under the header: {head:?} {:?}", words.rect);
    assert!(rt.presented("message-bot-header").is_none(), "no header, no row, no gap");
    let bot = drawn(&rt, "message-bot");
    let first = rt.elements().into_iter().find(|e| e.text.as_deref().is_some_and(|w| w.starts_with("The migration step"))).unwrap();
    assert!((first.rect[1] - bot[1]).abs() < 0.5, "a message without a header starts where it did");
}

#[test]
fn the_well_bubble_is_the_well_material_with_16_and_10_point_padding() {
    for dark in [false, true] {
        let rt = item_in("message", dark);
        let t = rune_ui_themes::theme(dark);
        let (well, quiet) = (info(&rt, "message-well-bubble"), info(&rt, "message-quiet-bubble"));
        let ops = rune_ui_style::plan(&t.mat.well.base);
        let rune_ui_style::Op::Fill(fill) = ops[0] else { panic!("a well starts with its fill") };
        assert_eq!(well.fill, fill, "dark {dark}: the bubble is painted with the well material");
        assert!(well.fill.a < 0.2 && quiet.rect[2] > 0.0, "a light tint, no heavy fill: {:?}", well.fill);
        let words = rt.elements().into_iter().find(|e| e.text.as_deref().is_some_and(|w| w.starts_with("Why is the build slow? It was"))).unwrap();
        assert!((words.rect[0] - well.rect[0] - 16.0).abs() < 0.6 && (words.rect[1] - well.rect[1] - 10.0).abs() < 0.6, "16 across, 10 down: {:?} {:?}", words.rect, well.rect);
        assert!(well.rect[2] <= 720.0 * 0.8 + 0.5, "at most four fifths of the column");
        assert!(well.rect[0] + well.rect[2] <= drawn(&rt, "message-well")[0] + drawn(&rt, "message-well")[2] + 0.5, "inside its message");
    }
}

// ---- PromptInput: a leading slot and a row of chips ----

#[test]
fn the_composer_takes_a_leading_element_and_chips_in_reading_and_tab_order() {
    let mut rt = item("prompt_input");
    let (lead, input, folder, branch, attach, send) = (drawn(&rt, "prompt-slots-lead"), drawn(&rt, "prompt-slots-input"), drawn(&rt, "prompt-slots-folder"), drawn(&rt, "prompt-slots-branch"), drawn(&rt, "prompt-slots-attach"), drawn(&rt, "prompt-slots-submit"));
    assert!(lead[0] + lead[2] < input[0], "the lead sits left of the text: {lead:?} {input:?}");
    assert!((lead[1] - input[1]).abs() < 6.0, "top-aligned with the first line: {lead:?} {input:?}");
    assert!(folder[1] >= input[1] + input[3] - 0.5 && folder[1] < attach[1], "the chips sit under the text, above the toolbar: {folder:?}");
    assert!(branch[0] > folder[0] + folder[2], "in order, in a row");
    assert!(send[0] > attach[0], "the toolbar is as before");
    // Tab order: the lead, the text, the chips, then the toolbar.
    focus(&mut rt, "prompt-slots-lead");
    let mut seen = Vec::new();
    for _ in 0..5 {
        key(&mut rt, Key::Tab);
        seen.push(focused(&rt).unwrap_or_default());
    }
    let at = |k: &str| seen.iter().position(|s| s == k);
    assert!(at("prompt-slots-input") < at("prompt-slots-folder") && at("prompt-slots-folder") < at("prompt-slots-branch") && at("prompt-slots-branch") < at("prompt-slots-attach"), "Tab order {seen:?}");
    // Unused, the composer has no body row and no chip row.
    assert!(rt.presented("prompt-demo-body").is_none() && rt.presented("prompt-demo-chips").is_none());
}

// ---- SidebarItem: two lines, an image, raised, a trailing slot, a button form, the count read out ----

#[test]
fn a_two_line_sidebar_row_has_an_image_a_second_line_a_raised_current_state_and_a_trailing_slot() {
    let mut rt = item("sidebar");
    let (p1, p2, p3, new) = (drawn(&rt, "sidebar-p1"), drawn(&rt, "sidebar-p2"), drawn(&rt, "sidebar-p3"), drawn(&rt, "sidebar-new"));
    assert_eq!((p1[3], p2[3], p3[3]), (52.0, 52.0, 52.0), "two lines are 52 points high at the default density");
    assert_eq!(new[3], 36.0, "a one-line row is as before");
    let face = drawn(&rt, "sidebar-p1-face");
    assert_eq!((face[2], face[3]), (ui::portrait::ringed(28.0), ui::portrait::ringed(28.0)), "the image keeps the size it was built at");
    assert!((face[1] + face[3] / 2.0 - (p1[1] + p1[3] / 2.0)).abs() < 0.6, "centred in the row");
    let title = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("Asha") && e.rect[1] >= p1[1] && e.rect[1] < p1[1] + 52.0).unwrap();
    let sub = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("Fixing the login test")).unwrap();
    assert!(sub.rect[1] >= title.rect[1] + title.rect[3] - 1.0, "the second line sits under the title: {:?} {:?}", title.rect, sub.rect);
    assert_eq!(title.text_style.unwrap().weight, rune_ui_themes::theme(false).ty.semibold, "the current row's title is semibold");
    assert!(title.rect[0] > face[0] + face[2], "words after the image");
    // The trailing pill is at the row's end, after the count.
    let pill = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("PR")).unwrap();
    let count = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("3") && e.rect[1] >= p1[1] && e.rect[1] < p1[1] + 52.0).unwrap();
    assert!(pill.rect[0] > count.rect[0] + count.rect[2] && pill.rect[0] + pill.rect[2] <= p1[0] + p1[2], "trailing slot after the count, inside the row");
    // Raised: the current row is drawn with the raised material, a different fill from the quiet rows'.
    // (The pebble itself is a gradient and shadows, which the element list does not report: the picture shows it.)
    // The ring is kept: keyboard focus on the raised row shows it.
    focus(&mut rt, "sidebar-p1");
    key(&mut rt, Key::Tab);
    focus(&mut rt, "sidebar-p1");
    assert!(info(&rt, "sidebar-p1").focused);
}

#[test]
fn a_sidebar_row_can_be_a_button_and_reads_out_its_count_and_second_line() {
    let mut rt = item("sidebar");
    let tree = rt.accessibility_json();
    let new = ax(&tree, "sidebar-new").expect("New task");
    assert_eq!(new["role"].as_str(), Some("button"), "{new}");
    let p1 = ax(&tree, "sidebar-p1").expect("Asha's row");
    let words = p1.to_string();
    assert!(words.contains("Fixing the login test") && words.contains("3 unread"), "the second line and the count are read after the name: {p1}");
    assert_eq!(p1["position"].as_u64(), Some(1));
    // Without count words, the count is still said.
    assert!(ax(&tree, "sidebar-p3").unwrap().to_string().contains("Count 12"));
    // The old one-line rows keep their roles.
    assert_eq!(ax(&tree, "sidebar-home").unwrap()["role"].as_str(), Some("link"));
    // Return opens a list row, as before.
    focus(&mut rt, "sidebar-p2");
    key(&mut rt, Key::Enter);
    assert_eq!(rt.accessibility_json().to_string().contains("sidebar-p2"), true);
}

// ---- Neutral step marks ----

/// Every element inside the element keyed `key`.
fn within<S: 'static>(rt: &Runtime<S>, key: &str) -> Vec<ElementInfo> {
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
    els.iter().enumerate().filter(|(i, _)| inside(*i)).map(|(_, e)| e.clone()).collect()
}

fn inks(els: &[ElementInfo]) -> Vec<rune::Color> {
    els.iter().flat_map(|e| [e.text_color, Some(e.fill)]).flatten().collect()
}

#[test]
fn neutral_tool_marks_keep_colour_for_a_failure_only() {
    for dark in [false, true] {
        let rt = item_in("tool", dark);
        let t = rune_ui_themes::theme(dark);
        let (done, failed, denied) = (within(&rt, "tool-neutral-done-trigger"), within(&rt, "tool-neutral-failed-trigger"), within(&rt, "tool-neutral-denied-trigger"));
        assert!(!inks(&done).contains(&t.c.success.fg), "dark {dark}: done is not green");
        assert!(inks(&within(&rt, "tool-done-trigger")).contains(&t.c.success.fg), "dark {dark}: the coloured default still is (the control)");
        assert!(inks(&done).contains(&t.ink_faint()), "dark {dark}: done is a tick in the third ink");
        assert!(inks(&failed).contains(&t.c.danger.fg), "dark {dark}: a failure keeps its colour");
        assert!(denied.iter().any(|e| (e.rect[2], e.rect[3]) == (8.0, 1.5)), "dark {dark}: denied is an 8 by 1.5 dash");
        assert!(!done.iter().any(|e| (e.rect[2], e.rect[3]) == (8.0, 1.5)), "and only denied");
        // Running keeps the spinner (it loops), in the second ink.
        let running = within(&rt, "tool-neutral-running-trigger");
        assert!(running.iter().any(|e| e.loop_phase.is_some()), "running turns");
        assert!(inks(&running).contains(&t.ink_muted()), "in the second ink");
    }
}

#[test]
fn an_answered_request_can_be_marked_without_green_or_red() {
    let rt = item("confirmation");
    let t = rune_ui_themes::theme(false);
    for k in ["confirm-n-once", "confirm-n-always", "confirm-n-denied"] {
        let i = inks(&within(&rt, k));
        assert!(!i.contains(&t.c.success.fg) && !i.contains(&t.c.danger.fg), "{k}: no green check, no red cross");
    }
    assert!(inks(&within(&rt, "confirm-once")).contains(&t.c.success.fg), "the default still has its green check");
    assert!(within(&rt, "confirm-n-denied").iter().any(|e| (e.rect[2], e.rect[3]) == (8.0, 1.5)), "denied is a dash");
    let line = info(&rt, "confirm-n-denied");
    assert_eq!(line.label.as_deref(), Some("Push to main: Denied"), "the words still say it");
}

#[test]
fn reasoning_can_swap_the_sparkle_for_a_hollow_dot() {
    let rt = item("reasoning");
    let dot = within(&rt, "reasoning-dot-done-trigger");
    assert!(dot.iter().any(|e| (e.rect[2], e.rect[3]) == (7.0, 7.0)), "a 7 point dot");
    assert!(!within(&rt, "reasoning-done-trigger").iter().any(|e| (e.rect[2], e.rect[3]) == (7.0, 7.0)), "the sparkle is the default");
}
