//! Every item's accessibility check (`registry/<item>/a11y.rs`): the keyed elements have the roles they promise, and every focusable
//! element in every story has a name, in both appearances.

mod common;

#[test]
fn every_item_meets_its_accessibility_check() {
    let mut failures = Vec::new();
    for (item, checks) in gallery::A11Y {
        for dark in [false, true] {
            let els = common::elements(item, dark);
            for (key, role) in *checks {
                match els.iter().find(|e| e.key.as_deref() == Some(*key)) {
                    None => failures.push(format!("{item}: no element keyed {key}")),
                    Some(e) if e.role != *role => failures.push(format!("{item}: {key} is a {} not a {role}", e.role)),
                    Some(_) => {}
                }
            }
            for e in els.iter().filter(|e| e.focusable) {
                if e.label.as_deref().is_none_or(|l| l.trim().is_empty()) && e.text.as_deref().is_none_or(|l| l.trim().is_empty()) {
                    failures.push(format!("{item}: focusable {} ({}) has no name", e.key.as_deref().unwrap_or(&e.path), e.role));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} problem(s):\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn every_component_item_has_stories_on_the_page() {
    for (item, _) in gallery::A11Y {
        let els = common::elements(item, false);
        assert!(els.iter().any(|e| e.key.as_deref() == Some(&format!("item-{item}"))), "{item} is not on the page");
    }
}
