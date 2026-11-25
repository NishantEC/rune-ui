//! The CLI on a scratch app: init writes the dependencies and Rune's patch table, add copies items with what they need and records them,
//! --dry-run writes nothing, diff --check fails on drift, a file rune-ui did not write is never overwritten, the gallery brings the
//! stories of what is installed, and update merges the registry's changes into local edits.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_rune-ui");

fn app(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("rune-ui-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("src")).unwrap();
    std::fs::write(d.join("Cargo.toml"), "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n").unwrap();
    std::fs::write(d.join("src/main.rs"), "fn main() {}\n").unwrap();
    d
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(BIN).args(args).arg("--app").arg(dir).output().unwrap()
}

fn ok(dir: &Path, args: &[&str]) -> String {
    let o = run(dir, args);
    assert!(o.status.success(), "{args:?}: {}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn init_writes_dependencies_the_patch_table_and_the_base() {
    let d = app("init");
    ok(&d, &["init"]);
    let manifest = std::fs::read_to_string(d.join("Cargo.toml")).unwrap();
    for k in ["rune = {", "rune-kit = {", "rune-ui-style = {", "rune-ui-themes = {", "[patch.crates-io]", "wgpu-hal = {", "accesskit_macos = {"] {
        assert!(manifest.contains(k), "missing {k} in\n{manifest}");
    }
    assert!(d.join("src/ui/base.rs").exists() && d.join("src/ui/icons.rs").exists());
    let lock = std::fs::read_to_string(d.join("rune-ui.lock")).unwrap();
    assert!(lock.contains("name = \"base\"") && lock.contains("sha256"));
    ok(&d, &["init"]);
    assert_eq!(std::fs::read_to_string(d.join("Cargo.toml")).unwrap(), manifest, "init twice changes nothing");
}

#[test]
fn add_dry_run_writes_nothing_and_add_records_what_it_copies() {
    let d = app("add");
    ok(&d, &["init"]);
    let out = ok(&d, &["add", "alert_dialog", "--dry-run"]);
    assert!(out.contains("would write") && !d.join("src/ui/alert_dialog.rs").exists());
    ok(&d, &["add", "alert_dialog"]);
    for f in ["alert_dialog", "dialog", "button"] {
        assert!(d.join(format!("src/ui/{f}.rs")).exists(), "{f} came with alert_dialog");
    }
    let m = std::fs::read_to_string(d.join("src/ui/mod.rs")).unwrap();
    assert!(m.contains("pub mod alert_dialog;") && m.contains("pub mod dialog;"));
    assert!(!d.join("src/ui/stories").exists(), "no stories without the gallery");
}

#[test]
fn diff_check_fails_on_drift_and_add_refuses_to_overwrite() {
    let d = app("diff");
    ok(&d, &["init"]);
    ok(&d, &["add", "switch"]);
    assert!(run(&d, &["diff", "--check"]).status.success());
    let f = d.join("src/ui/switch.rs");
    let mut s = std::fs::read_to_string(&f).unwrap();
    s.push_str("// my change\n");
    std::fs::write(&f, s).unwrap();
    let o = run(&d, &["diff", "--check"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stdout).contains("switch.rs differs"));
    assert!(ok(&d, &["diff", "switch"]).contains("+// my change"));
    let o = run(&d, &["add", "switch"]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("has your changes"));
    std::fs::write(d.join("src/ui/badge.rs"), "// someone else's file\n").unwrap();
    let o = run(&d, &["add", "badge"]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("did not write it"));
    ok(&d, &["add", "badge", "--force"]);
}

#[test]
fn the_gallery_brings_the_stories_of_what_is_installed() {
    let d = app("gallery");
    ok(&d, &["init"]);
    ok(&d, &["add", "switch", "dialog"]);
    ok(&d, &["add", "gallery"]);
    assert!(d.join("src/ui/stories/switch.rs").exists());
    assert!(!d.join("src/ui/stories/dialog.rs").exists(), "dialog's stories also show an input, which is not installed");
    ok(&d, &["add", "input"]);
    assert!(d.join("src/ui/stories/dialog.rs").exists(), "now they can come");
    let stories = std::fs::read_to_string(d.join("src/ui/stories/mod.rs")).unwrap();
    assert!(stories.contains("switch::item()") && stories.contains("dialog::item()") && !stories.contains("badge::item()"));
}

#[test]
fn update_merges_upstream_changes_into_local_edits() {
    let d = app("update");
    ok(&d, &["init"]);
    ok(&d, &["add", "kbd"]);
    // Pretend the copy was made from an older registry file: the base lacks the last line, so "upstream" added it.
    let target = d.join("src/ui/kbd.rs");
    let upstream = std::fs::read_to_string(&target).unwrap();
    let base_path = d.join(".rune-ui/base/src/ui/kbd.rs");
    let older: String = upstream.lines().take(upstream.lines().count() - 1).map(|l| format!("{l}\n")).collect();
    std::fs::write(&base_path, &older).unwrap();
    let mine = older.replacen("//! rune-ui registry: kbd", "//! (edited here) rune-ui registry: kbd", 1);
    std::fs::write(&target, &mine).unwrap();
    ok(&d, &["update", "kbd"]);
    let merged = std::fs::read_to_string(&target).unwrap();
    assert!(merged.contains("(edited here)"), "the local edit stays");
    assert_eq!(merged.lines().last(), upstream.lines().last(), "the upstream line arrives");
    assert!(run(&d, &["diff", "--check"]).status.success(), "the lock records the merge");
}

#[test]
fn list_shows_every_item_and_what_the_app_has() {
    let d = app("list");
    ok(&d, &["init"]);
    let out = ok(&d, &["list"]);
    assert!(out.lines().count() > 40);
    assert!(out.lines().any(|l| l.starts_with("base") && l.ends_with("ok")));
    assert!(out.lines().any(|l| l.starts_with("gallery") && l.contains("block")));
}
