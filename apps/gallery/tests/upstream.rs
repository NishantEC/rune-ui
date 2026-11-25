//! Fixes found when the first app installed rune-ui: the gallery in a narrow region, its sliders, its Reset and its Export, driven
//! through the live window path with real presses.

use gallery::{App, apply_scene, set_headless, view};
use rune::{Env, Runtime};

fn page(name: &str, w: f32) -> Runtime<App> {
    set_headless(false);
    let mut s = App::default();
    apply_scene(&mut s, name);
    let mut rt = Runtime::new(s, view, (w, 900.0), Env { system_dark: false, scale: 1.0 });
    for _ in 0..90 {
        rt.frame(1.0 / 60.0);
    }
    rt
}

fn rect(rt: &Runtime<App>, key: &str) -> [f32; 4] {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("no {key}")).rect
}

fn press(rt: &mut Runtime<App>, key: &str) {
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("no {key} to press"));
    rt.pointer_down(x, y);
    rt.pointer_up(x, y);
    for _ in 0..30 {
        rt.frame(1.0 / 60.0);
    }
}

#[test]
fn in_a_narrow_region_the_header_and_captions_stay_inside_the_page() {
    let rt = page("item-dropdown_menu", 720.0);
    let pane = rect(&rt, "gallery-scroll");
    let right = pane[0] + pane[2] + 0.5;
    let summary = rect(&rt, "gallery-summary");
    assert!(summary[0] + summary[2] <= right, "the summary wraps inside the page: {summary:?} in {pane:?}");
    for e in rt.elements().into_iter().filter(|e| e.text.is_some() && e.path.contains("item-dropdown_menu")) {
        assert!(e.rect[0] + e.rect[2] <= right, "{:?} runs past the page: {:?} in {pane:?}", e.text, e.rect);
    }
}

#[test]
fn a_slider_thumb_sits_between_its_spacers_without_overlapping_them() {
    let rt = page("actions", 1320.0);
    let bar = rect(&rt, "editor-radius");
    let thumb = rect(&rt, "editor-radius-thumb");
    let frac = (rt.state.g.editor.radius.unwrap_or(8.0) / 16.0).clamp(0.0, 1.0);
    let want = bar[0] + frac * (bar[2] - 16.0);
    assert!((thumb[0] - want).abs() < 1.0, "the thumb's left is its share of the free track: {} vs {want}", thumb[0]);
    assert!(thumb[0] >= bar[0] - 0.01 && thumb[0] + thumb[2] <= bar[0] + bar[2] + 0.01, "inside the track: {thumb:?} in {bar:?}");
    let row = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("editor-radius-thumb")).unwrap();
    let parent = row.path.rsplit_once('/').map_or(String::new(), |(p, _)| p.to_owned());
    let siblings: Vec<[f32; 4]> = rt.elements().into_iter().filter(|e| e.path.rsplit_once('/').map(|(p, _)| p) == Some(parent.as_str()) && e.path != row.path).map(|e| e.rect).collect();
    for s in siblings {
        let overlap = (thumb[0] + thumb[2]).min(s[0] + s[2]) - thumb[0].max(s[0]);
        assert!(overlap <= 0.01, "the thumb overlaps a sibling: {thumb:?} and {s:?}");
    }
}

#[test]
fn reset_returns_the_editor_to_the_theme_file() {
    let mut rt = page("actions", 1320.0);
    press(&mut rt, "editor-font-serif");
    press(&mut rt, "editor-corners-1");
    press(&mut rt, "editor-density-1");
    assert_eq!((rt.state.g.editor.font, rt.state.g.compact), (Some("serif"), Some(true)));
    press(&mut rt, "editor-reset");
    assert_eq!(rt.state.g.editor, gallery::ui::gallery::Editor::default(), "every value unset: the theme file's own");
    assert_eq!(rt.state.g.compact, None);
}

#[test]
fn export_hands_the_theme_to_the_app_and_is_gone_without_a_callback() {
    let mut rt = page("actions", 1320.0);
    rt.update(|s, _| s.g.on_export = Some(std::rc::Rc::new(|json: &str| format!("got {} bytes", json.len()))));
    for _ in 0..10 {
        rt.frame(1.0 / 60.0);
    }
    press(&mut rt, "editor-export");
    assert!(rt.state.g.exported.as_deref().is_some_and(|n| n.starts_with("got ")), "the callback got the JSON: {:?}", rt.state.g.exported);
    rt.update(|s, _| s.g.on_export = None);
    for _ in 0..10 {
        rt.frame(1.0 / 60.0);
    }
    assert!(rt.find("editor-export").is_none(), "no callback, no Export button");
    assert!(rt.find("editor-copy").is_some(), "Copy theme stays");
}
