//! 0.10's additions in the live path (overlays at the root), driven by real presses, pointer moves and keys: the scrim that dims and
//! the keycap font, the portrait, the message header and well bubble, the composer's slots, the two-line sidebar row, neutral marks,
//! the receipt, Diff's stats, selection and foot, and menu rows that lead with an image.


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
