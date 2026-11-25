//! Shared by the gallery's tests: one item's page in a headless runtime.

use gallery::{App, apply_scene, set_headless, view};
use rune::{ElementInfo, Env, Runtime};

pub fn elements(item: &str, dark: bool) -> Vec<ElementInfo> {
    set_headless(true);
    let mut s = App::default();
    apply_scene(&mut s, &format!("item-{item}"));
    let mut rt = Runtime::new(s, view, (1320.0, 6000.0), Env { system_dark: dark, scale: 1.0 });
    rt.frame(0.0);
    rt.elements()
}
