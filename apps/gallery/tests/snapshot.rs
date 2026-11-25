//! Layout snapshots: each item's stories laid out in the light theme, as `key role x y w h` lines relative to the item's block, compared
//! with `registry/<item>/snapshot.txt`. After an intended change: `RUNE_UI_BLESS=1 cargo test -p gallery --test snapshot`, then review
//! the diff.

mod common;

use std::path::PathBuf;

fn snapshot(item: &str) -> String {
    let els = common::elements(item, false);
    let origin = els.iter().find(|e| e.key.as_deref() == Some(&format!("item-{item}"))).map_or([0.0; 4], |e| e.rect);
    let mut out = String::new();
    for e in &els {
        let Some(k) = &e.key else { continue };
        if k.starts_with("gallery") || k.starts_with("group-") || k == "kit-root" || k == "canvas" {
            continue;
        }
        let r = e.rect;
        out.push_str(&format!("{k} {} {:.0} {:.0} {:.0} {:.0}\n", e.role, r[0] - origin[0], r[1] - origin[1], r[2], r[3]));
    }
    out
}

#[test]
fn every_item_matches_its_layout_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../registry");
    let bless = std::env::var_os("RUNE_UI_BLESS").is_some();
    let mut drift = Vec::new();
    for (item, _) in gallery::A11Y {
        let got = snapshot(item);
        let path = root.join(item).join("snapshot.txt");
        if bless {
            std::fs::write(&path, &got).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(&path).unwrap_or_default();
        if want != got {
            let first = want.lines().zip(got.lines()).position(|(a, b)| a != b).unwrap_or(want.lines().count().min(got.lines().count()));
            drift.push(format!("{item}: line {} differs\n  want: {}\n  got:  {}", first + 1, want.lines().nth(first).unwrap_or("(end)"), got.lines().nth(first).unwrap_or("(end)")));
        }
    }
    assert!(drift.is_empty(), "layout drifted (RUNE_UI_BLESS=1 to accept):\n{}", drift.join("\n"));
}
