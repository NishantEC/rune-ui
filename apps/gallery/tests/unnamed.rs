//! What assistive technology reads, item by item: no story has a button without a name. A pressable wrapper (the 40 point hit area
//! round a control) once read as an unnamed button around the named one; this keeps every such wrapper out of the tree.

use gallery::{App, apply_scene, set_headless, view};
use rune::{Env, Runtime};
use serde_json::Value;

fn tree(item: &str, dark: bool) -> Value {
    set_headless(true);
    let mut s = App::default();
    apply_scene(&mut s, &format!("item-{item}"));
    s.g.scroll = false;
    let mut rt = Runtime::new(s, view, (1320.0, 6000.0), Env { system_dark: dark, scale: 1.0 });
    rt.frame(0.0);
    rt.accessibility_json()
}

fn unnamed_buttons(v: &Value, path: &str, out: &mut Vec<String>) {
    let role = v["role"].as_str().unwrap_or_default();
    let key = v["key"].as_str().unwrap_or("?");
    let here = format!("{path}/{key}");
    if role.contains("button") && v["name"].as_str().is_none_or(|n| n.trim().is_empty()) {
        out.push(format!("{role} {here}"));
    }
    if let Some(cs) = v["children"].as_array() {
        for c in cs {
            unnamed_buttons(c, &here, out);
        }
    }
}

#[test]
fn no_story_has_an_unnamed_button() {
    let mut failures = Vec::new();
    for (item, _) in gallery::A11Y {
        for dark in [false, true] {
            let mut found = Vec::new();
            unnamed_buttons(&tree(item, dark), "", &mut found);
            failures.extend(found.into_iter().map(|f| format!("{item}{}: {f}", if dark { " (dark)" } else { "" })));
        }
    }
    assert!(failures.is_empty(), "{} unnamed button(s):\n{}", failures.len(), failures.join("\n"));
}
